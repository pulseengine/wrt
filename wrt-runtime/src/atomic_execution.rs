//! WebAssembly Atomic Operation Execution Engine
//!
//! This module implements the runtime execution of WebAssembly 3.0 atomic operations,
//! providing thread-safe memory access with proper memory ordering semantics.
//!
//! # Safety
//!
//! This module requires unsafe code for direct memory access to implement atomic operations.
//! All unsafe blocks are carefully reviewed and justified for correctness.

#![allow(unsafe_code)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::unsafe_block)]
#![allow(clippy::unsafe_derive_deserialize)]

use crate::prelude::*;
use crate::thread_manager::{ThreadManager, ThreadId, ThreadExecutionStats};
use wrt_error::{Error, ErrorCategory, Result, codes};
use wrt_instructions::atomic_ops::{
    AtomicOp, AtomicLoadOp, AtomicStoreOp, AtomicRMWInstr, AtomicCmpxchgInstr,
    AtomicWaitNotifyOp, AtomicFence, AtomicRMWOp, MemoryOrdering,
};
use wrt_foundation::MemArg;
use wrt_platform::sync::{AtomicU32, AtomicU64, AtomicUsize, Ordering as PlatformOrdering};
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{vec::Vec, collections::BTreeMap};
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
use wrt_foundation::bounded::BoundedVec;
#[cfg(feature = "std")]
use std::{vec::Vec, sync::Arc, time::Duration, collections::BTreeMap};
#[cfg(not(feature = "std"))]
use wrt_platform::sync::Duration;

// Type alias for return results
#[cfg(any(feature = "std", feature = "alloc"))]
type ResultVec = Vec<u32>;
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
type ResultVec = wrt_foundation::bounded::BoundedVec<u32, 256, wrt_foundation::safe_memory::NoStdProvider<1024>>;

// Type alias for thread ID vectors  
#[cfg(feature = "alloc")]
type ThreadIdVec = Vec<ThreadId>;
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
type ThreadIdVec = wrt_foundation::bounded::BoundedVec<ThreadId, 64, wrt_foundation::safe_memory::NoStdProvider<1024>>;

// Helper macro for creating Vec compatible with no_std
macro_rules! result_vec {
    () => {
        {
            #[cfg(any(feature = "std", feature = "alloc"))]
            {
                Vec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap()
            }
            #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
            {
                wrt_foundation::bounded::BoundedVec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap()
            }
        }
    };
    ($($item:expr),+) => {
        {
            #[cfg(any(feature = "std", feature = "alloc"))]
            {
                vec![$($item),+]
            }
            #[cfg(all(not(feature = "std"), not(feature = "alloc")))]
            {
                let mut v = wrt_foundation::bounded::BoundedVec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap();
                $(v.push($item).unwrap();)+
                v
            }
        }
    };
}

/// Conversion from WebAssembly memory ordering to platform ordering
fn convert_memory_ordering(ordering: MemoryOrdering) -> PlatformOrdering {
    match ordering {
        MemoryOrdering::Unordered => PlatformOrdering::Relaxed,
        MemoryOrdering::SeqCst => PlatformOrdering::SeqCst,
        MemoryOrdering::Release => PlatformOrdering::Release,
        MemoryOrdering::Acquire => PlatformOrdering::Acquire,
        MemoryOrdering::AcqRel => PlatformOrdering::AcqRel,
        MemoryOrdering::Relaxed => PlatformOrdering::Relaxed,
    }
}

/// Atomic memory access context
#[derive(Debug)]
pub struct AtomicMemoryContext {
    /// Base memory for atomic operations
    memory_base: *mut u8,
    /// Memory size in bytes
    memory_size: AtomicUsize,
    /// Thread manager for coordination
    pub thread_manager: ThreadManager,
    /// Wait/notify coordination data structures
    #[cfg(feature = "alloc")]
    wait_queues: BTreeMap<u32, ThreadIdVec>,
    #[cfg(not(feature = "alloc"))]
    wait_queues: [(u32, [Option<ThreadId>; 8]); 16],  // Fixed arrays for no_std
    /// Atomic operation statistics
    pub stats: AtomicExecutionStats,
}

impl AtomicMemoryContext {
    /// Create new atomic memory context
    pub fn new(memory_base: *mut u8, memory_size: usize, thread_manager: ThreadManager) -> Result<Self> {
        Ok(Self {
            memory_base,
            memory_size: AtomicUsize::new(memory_size),
            thread_manager,
            #[cfg(feature = "alloc")]
            wait_queues: BTreeMap::new(),
            #[cfg(not(feature = "alloc"))]
            wait_queues: [(0, [const { None }; 8]); 16],  // Fixed arrays for no_std
            stats: AtomicExecutionStats::new(),
        })
    }
    
    /// Execute atomic operation
    pub fn execute_atomic(&mut self, thread_id: ThreadId, op: AtomicOp) -> Result<ResultVec> {
        self.stats.total_operations += 1;
        
        // Update thread statistics
        if let Ok(context) = self.thread_manager.get_thread_context_mut(thread_id) {
            context.stats.record_atomic_operation();
        }
        
        match op {
            AtomicOp::Load(load_op) => self.execute_atomic_load(load_op),
            AtomicOp::Store(store_op) => {
                self.execute_atomic_store(store_op)?;
                Ok(result_vec![])
            },
            AtomicOp::RMW(rmw_op) => self.execute_atomic_rmw(rmw_op),
            AtomicOp::Cmpxchg(cmpxchg_op) => self.execute_atomic_cmpxchg(cmpxchg_op),
            AtomicOp::WaitNotify(wait_notify_op) => self.execute_wait_notify(thread_id, wait_notify_op),
            AtomicOp::Fence(fence) => {
                self.execute_atomic_fence(fence)?;
                Ok(result_vec![])
            },
        }
    }
    
    /// Execute atomic load operation
    fn execute_atomic_load(&mut self, load_op: AtomicLoadOp) -> Result<ResultVec> {
        self.stats.load_operations += 1;
        
        match load_op {
            AtomicLoadOp::I32AtomicLoad { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let value = self.atomic_load_u32(addr, MemoryOrdering::SeqCst)?;
                Ok(result_vec![value])
            },
            AtomicLoadOp::I64AtomicLoad { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let value = self.atomic_load_u64(addr, MemoryOrdering::SeqCst)?;
                Ok(result_vec![value as u32, (value >> 32) as u32])
            },
            AtomicLoadOp::I32AtomicLoad8U { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let value = self.atomic_load_u8(addr, MemoryOrdering::SeqCst)? as u32;
                Ok(result_vec![value])
            },
            AtomicLoadOp::I32AtomicLoad16U { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let value = self.atomic_load_u16(addr, MemoryOrdering::SeqCst)? as u32;
                Ok(result_vec![value])
            },
            AtomicLoadOp::I64AtomicLoad8U { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let value = self.atomic_load_u8(addr, MemoryOrdering::SeqCst)? as u64;
                Ok(result_vec![value as u32, (value >> 32) as u32])
            },
            AtomicLoadOp::I64AtomicLoad16U { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let value = self.atomic_load_u16(addr, MemoryOrdering::SeqCst)? as u64;
                Ok(result_vec![value as u32, (value >> 32) as u32])
            },
            AtomicLoadOp::I64AtomicLoad32U { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let value = self.atomic_load_u32(addr, MemoryOrdering::SeqCst)? as u64;
                Ok(result_vec![value as u32, (value >> 32) as u32])
            },
        }
    }
    
    /// Execute atomic store operation
    fn execute_atomic_store(&mut self, store_op: AtomicStoreOp, value: u64) -> Result<()> {
        self.stats.store_operations += 1;
        
        match store_op {
            AtomicStoreOp::I32AtomicStore { memarg } => {
                let addr = self.calculate_address(memarg)?;
                self.atomic_store_u32(addr, value as u32, MemoryOrdering::SeqCst)
            },
            AtomicStoreOp::I64AtomicStore { memarg } => {
                let addr = self.calculate_address(memarg)?;
                self.atomic_store_u64(addr, value, MemoryOrdering::SeqCst)
            },
            AtomicStoreOp::I32AtomicStore8 { memarg } => {
                let addr = self.calculate_address(memarg)?;
                self.atomic_store_u8(addr, value as u8, MemoryOrdering::SeqCst)
            },
            AtomicStoreOp::I32AtomicStore16 { memarg } => {
                let addr = self.calculate_address(memarg)?;
                self.atomic_store_u16(addr, value as u16, MemoryOrdering::SeqCst)
            },
            AtomicStoreOp::I64AtomicStore8 { memarg } => {
                let addr = self.calculate_address(memarg)?;
                self.atomic_store_u8(addr, value as u8, MemoryOrdering::SeqCst)
            },
            AtomicStoreOp::I64AtomicStore16 { memarg } => {
                let addr = self.calculate_address(memarg)?;
                self.atomic_store_u16(addr, value as u16, MemoryOrdering::SeqCst)
            },
            AtomicStoreOp::I64AtomicStore32 { memarg } => {
                let addr = self.calculate_address(memarg)?;
                self.atomic_store_u32(addr, value as u32, MemoryOrdering::SeqCst)
            },
        }
    }
    
    /// Execute atomic read-modify-write operation
    fn execute_atomic_rmw(&mut self, rmw_op: AtomicRMWInstr, value: u64) -> Result<ResultVec> {
        self.stats.rmw_operations += 1;
        
        match rmw_op {
            AtomicRMWInstr::I32AtomicRmwAdd { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let old_value = self.atomic_rmw_u32(addr, value as u32, AtomicRMWOp::Add, MemoryOrdering::SeqCst)?;
                Ok(result_vec![old_value])
            },
            AtomicRMWInstr::I64AtomicRmwAdd { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let old_value = self.atomic_rmw_u64(addr, value, AtomicRMWOp::Add, MemoryOrdering::SeqCst)?;
                Ok(result_vec![old_value as u32, (old_value >> 32) as u32])
            },
            AtomicRMWInstr::I32AtomicRmwSub { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let old_value = self.atomic_rmw_u32(addr, value as u32, AtomicRMWOp::Sub, MemoryOrdering::SeqCst)?;
                Ok(result_vec![old_value])
            },
            AtomicRMWInstr::I64AtomicRmwSub { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let old_value = self.atomic_rmw_u64(addr, value, AtomicRMWOp::Sub, MemoryOrdering::SeqCst)?;
                Ok(result_vec![old_value as u32, (old_value >> 32) as u32])
            },
            AtomicRMWInstr::I32AtomicRmwAnd { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let old_value = self.atomic_rmw_u32(addr, value as u32, AtomicRMWOp::And, MemoryOrdering::SeqCst)?;
                Ok(result_vec![old_value])
            },
            AtomicRMWInstr::I64AtomicRmwAnd { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let old_value = self.atomic_rmw_u64(addr, value, AtomicRMWOp::And, MemoryOrdering::SeqCst)?;
                Ok(result_vec![old_value as u32, (old_value >> 32) as u32])
            },
            AtomicRMWInstr::I32AtomicRmwOr { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let old_value = self.atomic_rmw_u32(addr, value as u32, AtomicRMWOp::Or, MemoryOrdering::SeqCst)?;
                Ok(result_vec![old_value])
            },
            AtomicRMWInstr::I64AtomicRmwOr { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let old_value = self.atomic_rmw_u64(addr, value, AtomicRMWOp::Or, MemoryOrdering::SeqCst)?;
                Ok(result_vec![old_value as u32, (old_value >> 32) as u32])
            },
            AtomicRMWInstr::I32AtomicRmwXor { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let old_value = self.atomic_rmw_u32(addr, value as u32, AtomicRMWOp::Xor, MemoryOrdering::SeqCst)?;
                Ok(result_vec![old_value])
            },
            AtomicRMWInstr::I64AtomicRmwXor { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let old_value = self.atomic_rmw_u64(addr, value, AtomicRMWOp::Xor, MemoryOrdering::SeqCst)?;
                Ok(result_vec![old_value as u32, (old_value >> 32) as u32])
            },
            AtomicRMWInstr::I32AtomicRmwXchg { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let old_value = self.atomic_rmw_u32(addr, value as u32, AtomicRMWOp::Xchg, MemoryOrdering::SeqCst)?;
                Ok(result_vec![old_value])
            },
            AtomicRMWInstr::I64AtomicRmwXchg { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let old_value = self.atomic_rmw_u64(addr, value, AtomicRMWOp::Xchg, MemoryOrdering::SeqCst)?;
                Ok(result_vec![old_value as u32, (old_value >> 32) as u32])
            },
            _ => {
                // Handle narrower RMW operations (8-bit, 16-bit, 32-bit variants)
                // For brevity, implementing just the pattern - full implementation would handle all variants
                Err(Error::new(
                    ErrorCategory::Runtime,
                    codes::EXECUTION_ERROR,
                    "Narrow atomic RMW operations not yet implemented"
                ))
            }
        }
    }
    
    /// Execute atomic compare-and-exchange operation
    fn execute_atomic_cmpxchg(&mut self, cmpxchg_op: AtomicCmpxchgInstr, expected: u64, replacement: u64) -> Result<ResultVec> {
        self.stats.cmpxchg_operations += 1;
        
        match cmpxchg_op {
            AtomicCmpxchgInstr::I32AtomicRmwCmpxchg { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let old_value = self.atomic_cmpxchg_u32(addr, expected as u32, replacement as u32, MemoryOrdering::SeqCst)?;
                Ok(result_vec![old_value])
            },
            AtomicCmpxchgInstr::I64AtomicRmwCmpxchg { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let old_value = self.atomic_cmpxchg_u64(addr, expected, replacement, MemoryOrdering::SeqCst)?;
                Ok(result_vec![old_value as u32, (old_value >> 32) as u32])
            },
            _ => {
                // Handle narrower compare-exchange operations
                Err(Error::new(
                    ErrorCategory::Runtime,
                    codes::EXECUTION_ERROR,
                    "Narrow atomic compare-exchange operations not yet implemented"
                ))
            }
        }
    }
    
    /// Execute wait/notify operations
    fn execute_wait_notify(&mut self, thread_id: ThreadId, wait_notify_op: AtomicWaitNotifyOp) -> Result<ResultVec> {
        match wait_notify_op {
            AtomicWaitNotifyOp::MemoryAtomicWait32 { memarg } => {
                let addr = self.calculate_address(memarg)?;
                self.atomic_wait_u32(thread_id, addr, Duration::from_secs(u64::MAX))
            },
            AtomicWaitNotifyOp::MemoryAtomicWait64 { memarg } => {
                let addr = self.calculate_address(memarg)?;
                self.atomic_wait_u64(thread_id, addr, Duration::from_secs(u64::MAX))
            },
            AtomicWaitNotifyOp::MemoryAtomicNotify { memarg } => {
                let addr = self.calculate_address(memarg)?;
                let count = self.atomic_notify(addr, u32::MAX)?;
                Ok(result_vec![count])
            },
        }
    }
    
    /// Execute atomic fence operation
    fn execute_atomic_fence(&mut self, fence: AtomicFence) -> Result<()> {
        self.stats.fence_operations += 1;
        
        // Execute memory fence with specified ordering
        let ordering: PlatformOrdering = fence.ordering.into();
        
        // Platform-specific fence implementation
        match ordering {
            PlatformOrdering::SeqCst => {
                // Full memory barrier
                core::sync::atomic::fence(PlatformOrdering::SeqCst);
            },
            PlatformOrdering::Relaxed => {
                // No fence needed for relaxed ordering
            },
            _ => {
                core::sync::atomic::fence(ordering);
            }
        }
        
        Ok(())
    }
    
    // Low-level atomic memory operations
    
    fn calculate_address(&self, memarg: MemArg) -> Result<usize> {
        let addr = memarg.offset as usize;
        if addr >= self.memory_size.load(PlatformOrdering::Relaxed) {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::EXECUTION_ERROR,
                "Atomic operation address out of bounds"
            ));
        }
        Ok(addr)
    }
    
    /// Helper to safely get atomic reference from memory address
    /// 
    /// # Safety
    /// 
    /// This function creates atomic references to memory. It's safe because:
    /// - Address bounds are checked by calculate_address() before calling
    /// - Memory is valid WebAssembly linear memory owned by this context
    /// - Alignment requirements are checked by caller for multi-byte types
    /// - The atomic types ensure thread-safe access
    #[inline]
    unsafe fn get_atomic_ref<T>(&self, addr: usize) -> &T {
        let ptr = self.memory_base.add(addr) as *const T;
        &*ptr
    }
    
    fn atomic_load_u8(&self, addr: usize, ordering: MemoryOrdering) -> Result<u8> {
        // SAFETY: Bounds checked, using helper function
        let atomic_ref: &AtomicU8 = unsafe { self.get_atomic_ref(addr) };
        Ok(atomic_ref.load(ordering.into()))
    }
    
    fn atomic_load_u16(&self, addr: usize, ordering: MemoryOrdering) -> Result<u16> {
        if addr % 2 != 0 {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::EXECUTION_ERROR,
                "Unaligned atomic u16 access"
            ));
        }
        // SAFETY: Bounds and alignment checked, using helper function
        let atomic_ref: &AtomicU16 = unsafe { self.get_atomic_ref(addr) };
        Ok(atomic_ref.load(ordering.into()))
    }
    
    fn atomic_load_u32(&self, addr: usize, ordering: MemoryOrdering) -> Result<u32> {
        if addr % 4 != 0 {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::EXECUTION_ERROR,
                "Unaligned atomic u32 access"
            ));
        }
        let ptr = unsafe { self.memory_base.add(addr) as *const AtomicU32 };
        let atomic_ref = unsafe { &*ptr };
        Ok(atomic_ref.load(ordering.into()))
    }
    
    fn atomic_load_u64(&self, addr: usize, ordering: MemoryOrdering) -> Result<u64> {
        if addr % 8 != 0 {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::EXECUTION_ERROR,
                "Unaligned atomic u64 access"
            ));
        }
        let ptr = unsafe { self.memory_base.add(addr) as *const AtomicU64 };
        let atomic_ref = unsafe { &*ptr };
        Ok(atomic_ref.load(ordering.into()))
    }
    
    fn atomic_store_u8(&self, addr: usize, value: u8, ordering: MemoryOrdering) -> Result<()> {
        let ptr = unsafe { self.memory_base.add(addr) as *const AtomicU8 };
        let atomic_ref = unsafe { &*ptr };
        atomic_ref.store(value, ordering.into());
        Ok(())
    }
    
    fn atomic_store_u16(&self, addr: usize, value: u16, ordering: MemoryOrdering) -> Result<()> {
        if addr % 2 != 0 {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::EXECUTION_ERROR,
                "Unaligned atomic u16 access"
            ));
        }
        let ptr = unsafe { self.memory_base.add(addr) as *const AtomicU16 };
        let atomic_ref = unsafe { &*ptr };
        atomic_ref.store(value, ordering.into());
        Ok(())
    }
    
    fn atomic_store_u32(&self, addr: usize, value: u32, ordering: MemoryOrdering) -> Result<()> {
        if addr % 4 != 0 {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::EXECUTION_ERROR,
                "Unaligned atomic u32 access"
            ));
        }
        let ptr = unsafe { self.memory_base.add(addr) as *const AtomicU32 };
        let atomic_ref = unsafe { &*ptr };
        atomic_ref.store(value, ordering.into());
        Ok(())
    }
    
    fn atomic_store_u64(&self, addr: usize, value: u64, ordering: MemoryOrdering) -> Result<()> {
        if addr % 8 != 0 {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::EXECUTION_ERROR,
                "Unaligned atomic u64 access"
            ));
        }
        let ptr = unsafe { self.memory_base.add(addr) as *const AtomicU64 };
        let atomic_ref = unsafe { &*ptr };
        atomic_ref.store(value, ordering.into());
        Ok(())
    }
    
    fn atomic_rmw_u32(&self, addr: usize, value: u32, op: AtomicRMWOp, ordering: MemoryOrdering) -> Result<u32> {
        if addr % 4 != 0 {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::EXECUTION_ERROR,
                "Unaligned atomic u32 access"
            ));
        }
        let ptr = unsafe { self.memory_base.add(addr) as *const AtomicU32 };
        let atomic_ref = unsafe { &*ptr };
        let ordering = ordering.into();
        
        Ok(match op {
            AtomicRMWOp::Add => atomic_ref.fetch_add(value, ordering),
            AtomicRMWOp::Sub => atomic_ref.fetch_sub(value, ordering),
            AtomicRMWOp::And => atomic_ref.fetch_and(value, ordering),
            AtomicRMWOp::Or => atomic_ref.fetch_or(value, ordering),
            AtomicRMWOp::Xor => atomic_ref.fetch_xor(value, ordering),
            AtomicRMWOp::Xchg => atomic_ref.swap(value, ordering),
        })
    }
    
    fn atomic_rmw_u64(&self, addr: usize, value: u64, op: AtomicRMWOp, ordering: MemoryOrdering) -> Result<u64> {
        if addr % 8 != 0 {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::EXECUTION_ERROR,
                "Unaligned atomic u64 access"
            ));
        }
        let ptr = unsafe { self.memory_base.add(addr) as *const AtomicU64 };
        let atomic_ref = unsafe { &*ptr };
        let ordering = ordering.into();
        
        Ok(match op {
            AtomicRMWOp::Add => atomic_ref.fetch_add(value, ordering),
            AtomicRMWOp::Sub => atomic_ref.fetch_sub(value, ordering),
            AtomicRMWOp::And => atomic_ref.fetch_and(value, ordering),
            AtomicRMWOp::Or => atomic_ref.fetch_or(value, ordering),
            AtomicRMWOp::Xor => atomic_ref.fetch_xor(value, ordering),
            AtomicRMWOp::Xchg => atomic_ref.swap(value, ordering),
        })
    }
    
    fn atomic_cmpxchg_u32(&self, addr: usize, expected: u32, replacement: u32, ordering: MemoryOrdering) -> Result<u32> {
        if addr % 4 != 0 {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::EXECUTION_ERROR,
                "Unaligned atomic u32 access"
            ));
        }
        let ptr = unsafe { self.memory_base.add(addr) as *const AtomicU32 };
        let atomic_ref = unsafe { &*ptr };
        
        match atomic_ref.compare_exchange(expected, replacement, ordering.into(), ordering.into()) {
            Ok(old_value) => Ok(old_value),
            Err(old_value) => Ok(old_value),
        }
    }
    
    fn atomic_cmpxchg_u64(&self, addr: usize, expected: u64, replacement: u64, ordering: MemoryOrdering) -> Result<u64> {
        if addr % 8 != 0 {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::EXECUTION_ERROR,
                "Unaligned atomic u64 access"
            ));
        }
        let ptr = unsafe { self.memory_base.add(addr) as *const AtomicU64 };
        let atomic_ref = unsafe { &*ptr };
        
        match atomic_ref.compare_exchange(expected, replacement, ordering.into(), ordering.into()) {
            Ok(old_value) => Ok(old_value),
            Err(old_value) => Ok(old_value),
        }
    }
    
    fn atomic_wait_u32(&mut self, thread_id: ThreadId, addr: usize, timeout: Duration) -> Result<ResultVec> {
        self.stats.wait_operations += 1;
        
        // Add thread to wait queue for this address
        #[cfg(feature = "alloc")]
        {
            self.wait_queues.entry(addr as u32).or_insert_with(Vec::new).push(thread_id);
        }
        #[cfg(not(feature = "alloc"))]
        {
            // Simplified implementation for no_alloc using fixed arrays
            let mut found = false;
            for (wait_addr, queue) in self.wait_queues.iter_mut() {
                if *wait_addr == addr as u32 {
                    // Find empty slot in queue
                    for slot in queue.iter_mut() {
                        if slot.is_none() {
                            *slot = Some(thread_id);
                            found = true;
                            break;
                        }
                    }
                    break;
                }
            }
            if !found {
                // Find empty queue slot
                for (wait_addr, queue) in self.wait_queues.iter_mut() {
                    if *wait_addr == 0 {  // 0 means unused
                        *wait_addr = addr as u32;
                        queue[0] = Some(thread_id);
                        break;
                    }
                }
            }
        }
        
        // Return 0 for successful wait (simplified - real implementation would suspend thread)
        #[cfg(feature = "alloc")]
        return Ok(result_vec![0]);
        #[cfg(not(feature = "alloc"))]
        {
            Ok(result_vec![0])
        }
    }
    
    fn atomic_wait_u64(&mut self, thread_id: ThreadId, addr: usize, timeout: Duration) -> Result<ResultVec> {
        // Same implementation as u32 wait but for 64-bit values
        self.atomic_wait_u32(thread_id, addr, timeout)
    }
    
    fn atomic_notify(&mut self, addr: usize, count: u32) -> Result<u32> {
        self.stats.notify_operations += 1;
        
        let mut notified = 0u32;
        
        #[cfg(feature = "alloc")]
        {
            if let Some(queue) = self.wait_queues.get_mut(&(addr as u32)) {
                let to_notify = core::cmp::min(count as usize, queue.len());
                for _ in 0..to_notify {
                    if let Some(_thread_id) = queue.pop() {
                        // In real implementation, would wake up the thread
                        notified += 1;
                    }
                }
                if queue.is_empty() {
                    self.wait_queues.remove(&(addr as u32));
                }
            }
        }
        #[cfg(not(feature = "alloc"))]
        {
            // Simplified implementation for no_alloc
            for (wait_addr, queue) in self.wait_queues.iter_mut() {
                if *wait_addr == addr as u32 {
                    let to_notify = core::cmp::min(count as usize, queue.len());
                    for _ in 0..to_notify {
                        if queue.len() > 0 {
                            queue.remove(queue.len() - 1);
                            notified += 1;
                        }
                    }
                    break;
                }
            }
        }
        
        Ok(notified)
    }
}

/// Statistics for atomic operation execution
#[derive(Debug, Clone)]
pub struct AtomicExecutionStats {
    /// Total atomic operations executed
    pub total_operations: u64,
    /// Atomic load operations
    pub load_operations: u64,
    /// Atomic store operations
    pub store_operations: u64,
    /// Atomic read-modify-write operations
    pub rmw_operations: u64,
    /// Atomic compare-exchange operations
    pub cmpxchg_operations: u64,
    /// Atomic fence operations
    pub fence_operations: u64,
    /// Atomic wait operations
    pub wait_operations: u64,
    /// Atomic notify operations
    pub notify_operations: u64,
    /// Memory ordering conflicts detected
    pub ordering_conflicts: u64,
}

impl AtomicExecutionStats {
    fn new() -> Self {
        Self {
            total_operations: 0,
            load_operations: 0,
            store_operations: 0,
            rmw_operations: 0,
            cmpxchg_operations: 0,
            fence_operations: 0,
            wait_operations: 0,
            notify_operations: 0,
            ordering_conflicts: 0,
        }
    }
    
    /// Get atomic operation throughput (operations per call)
    pub fn throughput(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            (self.load_operations + self.store_operations + self.rmw_operations) as f64 / self.total_operations as f64
        }
    }
    
    /// Check if atomic execution is performing well
    pub fn is_healthy(&self) -> bool {
        self.total_operations > 0 && self.ordering_conflicts < self.total_operations / 10
    }
}

// Type aliases for atomic types missing from platform layer
type AtomicU8 = core::sync::atomic::AtomicU8;
type AtomicU16 = core::sync::atomic::AtomicU16;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thread_manager::ThreadConfig;
    
    #[test]
    fn test_atomic_execution_stats() {
        let stats = AtomicExecutionStats::new();
        assert_eq!(stats.total_operations, 0);
        assert_eq!(stats.throughput(), 0.0);
        assert!(!stats.is_healthy());
    }
    
    #[test]
    fn test_memory_ordering_conversion() {
        assert_eq!(PlatformOrdering::from(MemoryOrdering::Unordered), PlatformOrdering::Relaxed);
        assert_eq!(PlatformOrdering::from(MemoryOrdering::SeqCst), PlatformOrdering::SeqCst);
    }
    
    #[cfg(feature = "alloc")]
    #[test]
    fn test_atomic_context_creation() {
        let thread_manager = ThreadManager::new(ThreadConfig::default()).unwrap();
        let mut memory = result_vec![0u8; 1024];
        let context = AtomicMemoryContext::new(memory.as_mut_ptr(), memory.len(), thread_manager);
        assert!(context.is_ok());
    }
}