//! VxWorks integration utilities
//!
//! This module provides high-level integration utilities that demonstrate
//! how to create cohesive VxWorks platform adapters.

use crate::{
    ExecutionContext, VxWorksConfig, VxWorksRtpAllocator, VxWorksLkmAllocator,
    VxWorksRtpFutex, VxWorksLkmFutex, VxWorksTask, VxWorksThread, TaskPriority
};
use wrt_platform::{PageAllocator, FutexLike, WASM_PAGE_SIZE};
use wrt_error::{Error, ErrorKind};

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec::Vec};

/// High-level VxWorks runtime adapter
///
/// This provides a unified interface for VxWorks platform operations
/// that automatically selects the appropriate implementation based on context.
pub struct VxWorksRuntimeAdapter {
    config: VxWorksConfig,
    allocator: Box<dyn PageAllocator>,
    futex: Box<dyn FutexLike>,
}

impl VxWorksRuntimeAdapter {
    /// Create a new runtime adapter
    pub fn new(config: VxWorksConfig) -> Result<Self, Error> {
        let allocator: Box<dyn PageAllocator> = match config.context {
            ExecutionContext::Rtp => {
                let rtp_allocator = VxWorksRtpAllocator::new()
                    .with_max_pages(config.max_pages)
                    .with_stack_size(config.stack_size)
                    .build()?;
                Box::new(rtp_allocator)
            }
            ExecutionContext::Lkm => {
                let lkm_allocator = VxWorksLkmAllocator::new()
                    .with_max_pages(config.max_pages)
                    .with_memory_partitions(config.use_memory_partitions)
                    .with_priority_inheritance(config.priority_inheritance)
                    .build()?;
                Box::new(lkm_allocator)
            }
        };

        let futex: Box<dyn FutexLike> = match config.context {
            ExecutionContext::Rtp => {
                let rtp_futex = VxWorksRtpFutex::new()
                    .with_priority_inheritance(config.priority_inheritance)
                    .build()?;
                Box::new(rtp_futex)
            }
            ExecutionContext::Lkm => {
                let lkm_futex = VxWorksLkmFutex::new()
                    .with_priority_inheritance(config.priority_inheritance)
                    .build()?;
                Box::new(lkm_futex)
            }
        };

        Ok(Self {
            config,
            allocator,
            futex,
        })
    }

    /// Get the execution context
    pub fn context(&self) -> ExecutionContext {
        self.config.context
    }

    /// Get the configuration
    pub fn config(&self) -> &VxWorksConfig {
        &self.config
    }

    /// Get mutable access to the allocator
    pub fn allocator_mut(&mut self) -> &mut dyn PageAllocator {
        self.allocator.as_mut()
    }

    /// Get access to the futex
    pub fn futex(&self) -> &dyn FutexLike {
        self.futex.as_ref()
    }

    /// Allocate WASM memory pages
    pub fn allocate_wasm_memory(&mut self, pages: usize) -> Result<*mut u8, Error> {
        let ptr = self.allocator.allocate_pages(pages)?;
        Ok(ptr.as_ptr())
    }

    /// Deallocate WASM memory pages
    pub fn deallocate_wasm_memory(&mut self, ptr: *mut u8, pages: usize) -> Result<(), Error> {
        use core::ptr::NonNull;
        let non_null_ptr = NonNull::new(ptr)
            .ok_or_else(|| Error::new(ErrorKind::Memory, "Cannot deallocate null pointer"))?;
        self.allocator.deallocate_pages(non_null_ptr, pages)
    }

    /// Create a task or thread based on context
    #[cfg(feature = "alloc")]
    pub fn spawn_execution_unit<F>(&self, name: &str, priority: TaskPriority, f: F) -> Result<VxWorksExecutionUnit, Error>
    where
        F: FnOnce() + Send + 'static,
    {
        let config = crate::threading::VxWorksTaskConfig {
            context: self.config.context,
            stack_size: self.config.stack_size,
            priority,
            name: Some(name.to_string()),
            floating_point: false,
            detached: false,
        };

        match self.config.context {
            ExecutionContext::Lkm => {
                let task = VxWorksTask::spawn(config, f)?;
                Ok(VxWorksExecutionUnit::Task(task))
            }
            ExecutionContext::Rtp => {
                let thread = VxWorksThread::spawn(config, f)?;
                Ok(VxWorksExecutionUnit::Thread(thread))
            }
        }
    }

    /// Get runtime statistics
    pub fn get_statistics(&self) -> VxWorksRuntimeStats {
        VxWorksRuntimeStats {
            allocated_pages: self.allocator.allocated_pages(),
            max_pages: self.allocator.max_pages(),
            context: self.config.context,
            memory_utilization: (self.allocator.allocated_pages() as f32 / self.allocator.max_pages() as f32) * 100.0,
        }
    }
}

/// Unified execution unit (task or thread)
#[cfg(feature = "alloc")]
pub enum VxWorksExecutionUnit {
    /// VxWorks task (LKM context)
    Task(VxWorksTask),
    /// VxWorks thread (RTP context)
    Thread(VxWorksThread),
}

#[cfg(feature = "alloc")]
impl VxWorksExecutionUnit {
    /// Get the execution unit ID
    pub fn id(&self) -> u64 {
        match self {
            Self::Task(task) => task.task_id().unwrap_or(0) as u64,
            Self::Thread(_) => VxWorksThread::current_thread_id(),
        }
    }

    /// Join the execution unit
    pub fn join(self) -> Result<(), Error> {
        match self {
            Self::Task(_task) => {
                // VxWorks tasks don't have a direct join mechanism
                // In practice, you'd implement a synchronization mechanism
                Ok(())
            }
            Self::Thread(thread) => thread.join(),
        }
    }
}

/// VxWorks runtime statistics
#[derive(Debug, Clone)]
pub struct VxWorksRuntimeStats {
    /// Number of allocated pages
    pub allocated_pages: usize,
    /// Maximum pages allowed
    pub max_pages: usize,
    /// Execution context
    pub context: ExecutionContext,
    /// Memory utilization percentage
    pub memory_utilization: f32,
}

impl VxWorksRuntimeStats {
    /// Check if memory usage is critical (>80%)
    pub fn is_memory_critical(&self) -> bool {
        self.memory_utilization > 80.0
    }

    /// Get available pages
    pub fn available_pages(&self) -> usize {
        self.max_pages.saturating_sub(self.allocated_pages)
    }

    /// Get memory usage in bytes
    pub fn memory_usage_bytes(&self) -> usize {
        self.allocated_pages * WASM_PAGE_SIZE
    }

    /// Get maximum memory in bytes
    pub fn max_memory_bytes(&self) -> usize {
        self.max_pages * WASM_PAGE_SIZE
    }
}

/// VxWorks memory manager with automatic context detection
pub struct VxWorksMemoryManager {
    adapters: Vec<VxWorksRuntimeAdapter>,
    current_context: ExecutionContext,
}

impl VxWorksMemoryManager {
    /// Create a new memory manager
    pub fn new() -> Result<Self, Error> {
        let current_context = Self::detect_execution_context();
        
        Ok(Self {
            adapters: Vec::new(),
            current_context,
        })
    }

    /// Detect the current VxWorks execution context
    fn detect_execution_context() -> ExecutionContext {
        #[cfg(target_os = "vxworks")]
        {
            // In a real implementation, this would check VxWorks-specific APIs
            // to determine if we're running in kernel or user space
            
            // Check if we can access kernel functions
            extern "C" {
                fn taskIdSelf() -> usize;
                fn kernelAlloc(size: usize) -> *mut u8;
            }
            
            // Try to call a kernel function
            let task_id = unsafe { taskIdSelf() };
            if task_id != 0 {
                // Try kernel allocation to detect context
                let ptr = unsafe { kernelAlloc(4096) };
                if !ptr.is_null() {
                    // Can allocate kernel memory - we're in LKM context
                    unsafe {
                        extern "C" {
                            fn kernelFree(ptr: *mut u8);
                        }
                        kernelFree(ptr);
                    }
                    return ExecutionContext::Lkm;
                }
            }
            
            // Fall back to RTP context
            ExecutionContext::Rtp
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            // For non-VxWorks platforms, default to RTP
            ExecutionContext::Rtp
        }
    }

    /// Add a runtime adapter
    pub fn add_adapter(&mut self, adapter: VxWorksRuntimeAdapter) {
        self.adapters.push(adapter);
    }

    /// Create and add a default adapter for the current context
    pub fn add_default_adapter(&mut self) -> Result<(), Error> {
        let config = VxWorksConfig {
            context: self.current_context,
            ..Default::default()
        };
        
        let adapter = VxWorksRuntimeAdapter::new(config)?;
        self.add_adapter(adapter);
        Ok(())
    }

    /// Get the current execution context
    pub fn current_context(&self) -> ExecutionContext {
        self.current_context
    }

    /// Get statistics for all adapters
    pub fn get_global_statistics(&self) -> VxWorksGlobalStats {
        let mut total_allocated = 0;
        let mut total_max = 0;
        let mut total_memory_bytes = 0;

        for adapter in &self.adapters {
            let stats = adapter.get_statistics();
            total_allocated += stats.allocated_pages;
            total_max += stats.max_pages;
            total_memory_bytes += stats.memory_usage_bytes();
        }

        VxWorksGlobalStats {
            adapter_count: self.adapters.len(),
            total_allocated_pages: total_allocated,
            total_max_pages: total_max,
            total_memory_bytes,
            current_context: self.current_context,
            global_utilization: if total_max > 0 {
                (total_allocated as f32 / total_max as f32) * 100.0
            } else {
                0.0
            },
        }
    }
}

impl Default for VxWorksMemoryManager {
    fn default() -> Self {
        Self::new().expect("Failed to create VxWorks memory manager")
    }
}

/// Global VxWorks statistics
#[derive(Debug, Clone)]
pub struct VxWorksGlobalStats {
    /// Number of runtime adapters
    pub adapter_count: usize,
    /// Total allocated pages across all adapters
    pub total_allocated_pages: usize,
    /// Total maximum pages across all adapters
    pub total_max_pages: usize,
    /// Total memory usage in bytes
    pub total_memory_bytes: usize,
    /// Current execution context
    pub current_context: ExecutionContext,
    /// Global memory utilization percentage
    pub global_utilization: f32,
}

impl VxWorksGlobalStats {
    /// Check if global memory usage is critical
    pub fn is_globally_critical(&self) -> bool {
        self.global_utilization > 85.0
    }
}

/// VxWorks execution context detector
pub struct VxWorksExecutionContext;

impl VxWorksExecutionContext {
    /// Detect execution context with detailed information
    pub fn detect_detailed() -> VxWorksContextInfo {
        let context = VxWorksMemoryManager::detect_execution_context();
        
        #[cfg(target_os = "vxworks")]
        {
            VxWorksContextInfo {
                context,
                kernel_version: Self::get_kernel_version(),
                available_memory: Self::get_available_memory(),
                supports_memory_partitions: context == ExecutionContext::Lkm,
                supports_priority_inheritance: true,
                tick_rate: Self::get_tick_rate(),
            }
        }
        
        #[cfg(not(target_os = "vxworks"))]
        {
            VxWorksContextInfo {
                context,
                kernel_version: "Mock 7.0.0".to_string(),
                available_memory: 64 * 1024 * 1024, // 64MB
                supports_memory_partitions: context == ExecutionContext::Lkm,
                supports_priority_inheritance: true,
                tick_rate: 60,
            }
        }
    }

    #[cfg(target_os = "vxworks")]
    fn get_kernel_version() -> String {
        // In a real implementation, this would query VxWorks version info
        "7.0.0".to_string()
    }

    #[cfg(target_os = "vxworks")]
    fn get_available_memory() -> usize {
        // In a real implementation, this would query system memory info
        64 * 1024 * 1024 // 64MB
    }

    #[cfg(target_os = "vxworks")]
    fn get_tick_rate() -> u32 {
        extern "C" {
            fn sysClkRateGet() -> i32;
        }
        
        unsafe { sysClkRateGet() as u32 }
    }
}

/// Detailed VxWorks context information
#[derive(Debug, Clone)]
pub struct VxWorksContextInfo {
    /// Execution context (RTP or LKM)
    pub context: ExecutionContext,
    /// VxWorks kernel version
    #[cfg(feature = "alloc")]
    pub kernel_version: String,
    #[cfg(not(feature = "alloc"))]
    pub kernel_version: &'static str,
    /// Available system memory
    pub available_memory: usize,
    /// Whether memory partitions are supported
    pub supports_memory_partitions: bool,
    /// Whether priority inheritance is supported
    pub supports_priority_inheritance: bool,
    /// System tick rate
    pub tick_rate: u32,
}

impl VxWorksContextInfo {
    /// Check if this context supports real-time requirements
    pub fn supports_realtime(&self) -> bool {
        self.tick_rate >= 50 && self.supports_priority_inheritance
    }

    /// Get recommended configuration for this context
    pub fn recommended_config(&self) -> VxWorksConfig {
        VxWorksConfig {
            context: self.context,
            max_pages: (self.available_memory / WASM_PAGE_SIZE) / 4, // Use 25% of available memory
            use_memory_partitions: self.supports_memory_partitions,
            priority_inheritance: self.supports_priority_inheritance,
            task_priority: Some(100), // Normal priority
            stack_size: 64 * 1024,   // 64KB stack
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_stats() {
        let stats = VxWorksRuntimeStats {
            allocated_pages: 800,
            max_pages: 1000,
            context: ExecutionContext::Rtp,
            memory_utilization: 80.0,
        };

        assert_eq!(stats.allocated_pages, 800);
        assert_eq!(stats.available_pages(), 200);
        assert_eq!(stats.memory_usage_bytes(), 800 * WASM_PAGE_SIZE);
        assert!(!stats.is_memory_critical()); // Exactly 80%, not >80%

        let critical_stats = VxWorksRuntimeStats {
            memory_utilization: 85.0,
            ..stats
        };
        assert!(critical_stats.is_memory_critical());
    }

    #[test]
    fn test_context_detection() {
        let context_info = VxWorksExecutionContext::detect_detailed();
        
        // Context should be valid
        assert!(matches!(context_info.context, ExecutionContext::Rtp | ExecutionContext::Lkm));
        
        // Should have reasonable values
        assert!(context_info.available_memory > 0);
        assert!(context_info.tick_rate > 0);
        
        // Test recommended config
        let config = context_info.recommended_config();
        assert_eq!(config.context, context_info.context);
        assert!(config.max_pages > 0);
        assert!(config.stack_size > 0);
    }

    #[test]
    fn test_memory_manager() {
        let mut manager = VxWorksMemoryManager::new().unwrap();
        assert!(matches!(manager.current_context(), ExecutionContext::Rtp | ExecutionContext::Lkm));
        
        // Test adding default adapter
        assert!(manager.add_default_adapter().is_ok());
        
        let global_stats = manager.get_global_statistics();
        assert_eq!(global_stats.adapter_count, 1);
        assert!(global_stats.total_max_pages > 0);
    }

    #[test]
    fn test_global_stats() {
        let stats = VxWorksGlobalStats {
            adapter_count: 3,
            total_allocated_pages: 2000,
            total_max_pages: 3000,
            total_memory_bytes: 2000 * WASM_PAGE_SIZE,
            current_context: ExecutionContext::Lkm,
            global_utilization: 66.67,
        };

        assert_eq!(stats.adapter_count, 3);
        assert!(!stats.is_globally_critical()); // 66.67% < 85%
        
        let critical_stats = VxWorksGlobalStats {
            global_utilization: 90.0,
            ..stats
        };
        assert!(critical_stats.is_globally_critical());
    }

    #[test]
    fn test_context_info_realtime_support() {
        let rt_context = VxWorksContextInfo {
            context: ExecutionContext::Lkm,
            #[cfg(feature = "alloc")]
            kernel_version: "7.0.0".to_string(),
            #[cfg(not(feature = "alloc"))]
            kernel_version: "7.0.0",
            available_memory: 1024 * 1024,
            supports_memory_partitions: true,
            supports_priority_inheritance: true,
            tick_rate: 60,
        };

        assert!(rt_context.supports_realtime());

        let non_rt_context = VxWorksContextInfo {
            tick_rate: 10, // Too low for real-time
            ..rt_context.clone()
        };
        assert!(!non_rt_context.supports_realtime());
    }
}