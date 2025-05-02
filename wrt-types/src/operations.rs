//! Operation tracking for bounded collections and memory operations
//!
//! This module provides infrastructure for tracking operations performed
//! on bounded collections and memory, supporting WCET analysis and fuel
//! consumption calculations.

use crate::verification::VerificationLevel;

#[cfg(feature = "std")]
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(not(feature = "std"))]
use core::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "std")]
use std::sync::OnceLock;

#[cfg(not(feature = "std"))]
mod once_cell {
    use core::cell::UnsafeCell;
    use core::sync::atomic::{AtomicBool, Ordering};

    pub struct OnceCell<T> {
        is_initialized: AtomicBool,
        value: UnsafeCell<Option<T>>,
    }

    impl<T> OnceCell<T> {
        pub const fn new() -> Self {
            Self {
                is_initialized: AtomicBool::new(false),
                value: UnsafeCell::new(None),
            }
        }

        pub fn get_or_init<F>(&self, f: F) -> &T
        where
            F: FnOnce() -> T,
        {
            if !self.is_initialized.load(Ordering::Acquire) {
                // Only one thread will be able to store true here
                if !self.is_initialized.swap(true, Ordering::AcqRel) {
                    unsafe {
                        *self.value.get() = Some(f());
                    }
                } else {
                    // Wait for the other thread to finish initialization
                    while !self.is_initialized.load(Ordering::Acquire) {
                        core::hint::spin_loop();
                    }
                }
            }

            unsafe {
                match &*self.value.get() {
                    Some(value) => value,
                    None => unreachable!("Value should be initialized"),
                }
            }
        }
    }

    // This is safe because we handle synchronization with AtomicBool
    unsafe impl<T: Send + Sync> Sync for OnceCell<T> {}
    unsafe impl<T: Send> Send for OnceCell<T> {}
}

#[cfg(not(feature = "std"))]
use self::once_cell::OnceCell as OnceLock;

/// Operation types that can be tracked for fuel consumption
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    /// Memory read operation
    MemoryRead,
    /// Memory write operation
    MemoryWrite,
    /// Memory grow operation
    MemoryGrow,
    /// Collection push operation
    CollectionPush,
    /// Collection pop operation
    CollectionPop,
    /// Collection lookup operation
    CollectionLookup,
    /// Collection insert operation
    CollectionInsert,
    /// Collection remove operation
    CollectionRemove,
    /// Collection validation operation
    CollectionValidate,
    /// Collection mutation operation
    CollectionMutate,
    /// Checksum calculation
    ChecksumCalculation,
    /// Function call operation
    FunctionCall,
    /// Control flow operation
    ControlFlow,
    /// Arithmetic operation
    Arithmetic,
    /// Other operation type
    Other,
}

impl OperationType {
    /// Get the fuel cost for this operation type
    pub fn fuel_cost(&self) -> u64 {
        match self {
            Self::MemoryRead => 1,
            Self::MemoryWrite => 1,
            Self::MemoryGrow => 10,
            Self::CollectionPush => 2,
            Self::CollectionPop => 2,
            Self::CollectionLookup => 1,
            Self::CollectionInsert => 3,
            Self::CollectionRemove => 3,
            Self::CollectionValidate => 5,
            Self::CollectionMutate => 4,
            Self::ChecksumCalculation => 10,
            Self::FunctionCall => 5,
            Self::ControlFlow => 2,
            Self::Arithmetic => 1,
            Self::Other => 1,
        }
    }

    /// Get the importance level for verification purposes
    pub fn importance(&self) -> u8 {
        match self {
            Self::MemoryRead => 100,
            Self::MemoryWrite => 150,
            Self::MemoryGrow => 200,
            Self::CollectionPush => 150,
            Self::CollectionPop => 150,
            Self::CollectionLookup => 100,
            Self::CollectionInsert => 150,
            Self::CollectionRemove => 150,
            Self::CollectionValidate => 200,
            Self::CollectionMutate => 150,
            Self::ChecksumCalculation => 180,
            Self::FunctionCall => 150,
            Self::ControlFlow => 120,
            Self::Arithmetic => 100,
            Self::Other => 100,
        }
    }
}

/// A counter for tracking operation counts
#[derive(Debug)]
pub struct OperationCounter {
    /// Counter for memory read operations
    memory_reads: AtomicU64,
    /// Counter for memory write operations
    memory_writes: AtomicU64,
    /// Counter for memory grow operations
    memory_grows: AtomicU64,
    /// Counter for collection push operations
    collection_pushes: AtomicU64,
    /// Counter for collection pop operations
    collection_pops: AtomicU64,
    /// Counter for collection lookup operations
    collection_lookups: AtomicU64,
    /// Counter for collection insert operations
    collection_inserts: AtomicU64,
    /// Counter for collection remove operations
    collection_removes: AtomicU64,
    /// Counter for collection validate operations
    collection_validates: AtomicU64,
    /// Counter for collection mutate operations
    collection_mutates: AtomicU64,
    /// Counter for checksum calculations
    checksum_calculations: AtomicU64,
    /// Counter for function calls
    function_calls: AtomicU64,
    /// Counter for control flow operations
    control_flows: AtomicU64,
    /// Counter for arithmetic operations
    arithmetic_ops: AtomicU64,
    /// Counter for other operations
    other_ops: AtomicU64,
    /// Total fuel consumed by operations
    fuel_consumed: AtomicU64,
}

// Manual implementation of Clone for OperationCounter since AtomicU64 doesn't implement Clone
impl Clone for OperationCounter {
    fn clone(&self) -> Self {
        Self {
            memory_reads: AtomicU64::new(self.memory_reads.load(Ordering::Relaxed)),
            memory_writes: AtomicU64::new(self.memory_writes.load(Ordering::Relaxed)),
            memory_grows: AtomicU64::new(self.memory_grows.load(Ordering::Relaxed)),
            collection_pushes: AtomicU64::new(self.collection_pushes.load(Ordering::Relaxed)),
            collection_pops: AtomicU64::new(self.collection_pops.load(Ordering::Relaxed)),
            collection_lookups: AtomicU64::new(self.collection_lookups.load(Ordering::Relaxed)),
            collection_inserts: AtomicU64::new(self.collection_inserts.load(Ordering::Relaxed)),
            collection_removes: AtomicU64::new(self.collection_removes.load(Ordering::Relaxed)),
            collection_validates: AtomicU64::new(self.collection_validates.load(Ordering::Relaxed)),
            collection_mutates: AtomicU64::new(self.collection_mutates.load(Ordering::Relaxed)),
            checksum_calculations: AtomicU64::new(
                self.checksum_calculations.load(Ordering::Relaxed),
            ),
            function_calls: AtomicU64::new(self.function_calls.load(Ordering::Relaxed)),
            control_flows: AtomicU64::new(self.control_flows.load(Ordering::Relaxed)),
            arithmetic_ops: AtomicU64::new(self.arithmetic_ops.load(Ordering::Relaxed)),
            other_ops: AtomicU64::new(self.other_ops.load(Ordering::Relaxed)),
            fuel_consumed: AtomicU64::new(self.fuel_consumed.load(Ordering::Relaxed)),
        }
    }
}

impl Default for OperationCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl OperationCounter {
    /// Create a new operation counter with all counts at zero
    pub fn new() -> Self {
        Self {
            memory_reads: AtomicU64::new(0),
            memory_writes: AtomicU64::new(0),
            memory_grows: AtomicU64::new(0),
            collection_pushes: AtomicU64::new(0),
            collection_pops: AtomicU64::new(0),
            collection_lookups: AtomicU64::new(0),
            collection_inserts: AtomicU64::new(0),
            collection_removes: AtomicU64::new(0),
            collection_validates: AtomicU64::new(0),
            collection_mutates: AtomicU64::new(0),
            checksum_calculations: AtomicU64::new(0),
            function_calls: AtomicU64::new(0),
            control_flows: AtomicU64::new(0),
            arithmetic_ops: AtomicU64::new(0),
            other_ops: AtomicU64::new(0),
            fuel_consumed: AtomicU64::new(0),
        }
    }

    /// Record an operation and update fuel consumption
    pub fn record_operation(&self, op_type: OperationType, verification_level: VerificationLevel) {
        // Record the specific operation
        match op_type {
            OperationType::MemoryRead => self.memory_reads.fetch_add(1, Ordering::Relaxed),
            OperationType::MemoryWrite => self.memory_writes.fetch_add(1, Ordering::Relaxed),
            OperationType::MemoryGrow => self.memory_grows.fetch_add(1, Ordering::Relaxed),
            OperationType::CollectionPush => self.collection_pushes.fetch_add(1, Ordering::Relaxed),
            OperationType::CollectionPop => self.collection_pops.fetch_add(1, Ordering::Relaxed),
            OperationType::CollectionLookup => {
                self.collection_lookups.fetch_add(1, Ordering::Relaxed)
            }
            OperationType::CollectionInsert => {
                self.collection_inserts.fetch_add(1, Ordering::Relaxed)
            }
            OperationType::CollectionRemove => {
                self.collection_removes.fetch_add(1, Ordering::Relaxed)
            }
            OperationType::CollectionValidate => {
                self.collection_validates.fetch_add(1, Ordering::Relaxed)
            }
            OperationType::CollectionMutate => {
                self.collection_mutates.fetch_add(1, Ordering::Relaxed)
            }
            OperationType::ChecksumCalculation => {
                self.checksum_calculations.fetch_add(1, Ordering::Relaxed)
            }
            OperationType::FunctionCall => self.function_calls.fetch_add(1, Ordering::Relaxed),
            OperationType::ControlFlow => self.control_flows.fetch_add(1, Ordering::Relaxed),
            OperationType::Arithmetic => self.arithmetic_ops.fetch_add(1, Ordering::Relaxed),
            OperationType::Other => self.other_ops.fetch_add(1, Ordering::Relaxed),
        };

        // Calculate base fuel cost
        let base_cost = op_type.fuel_cost();

        // Adjust cost based on verification level
        let verification_multiplier = match verification_level {
            VerificationLevel::None => 1.0,
            VerificationLevel::Sampling => 1.2,
            VerificationLevel::Standard => 1.5,
            VerificationLevel::Full => 2.0,
        };

        // Convert to integer with rounding
        let total_cost = (base_cost as f64 * verification_multiplier).round() as u64;

        // Update fuel consumed
        self.fuel_consumed.fetch_add(total_cost, Ordering::Relaxed);
    }

    /// Get the total fuel consumed by all operations
    pub fn get_fuel_consumed(&self) -> u64 {
        self.fuel_consumed.load(Ordering::Relaxed)
    }

    /// Reset all counters to zero
    pub fn reset(&self) {
        self.memory_reads.store(0, Ordering::Relaxed);
        self.memory_writes.store(0, Ordering::Relaxed);
        self.memory_grows.store(0, Ordering::Relaxed);
        self.collection_pushes.store(0, Ordering::Relaxed);
        self.collection_pops.store(0, Ordering::Relaxed);
        self.collection_lookups.store(0, Ordering::Relaxed);
        self.collection_inserts.store(0, Ordering::Relaxed);
        self.collection_removes.store(0, Ordering::Relaxed);
        self.collection_validates.store(0, Ordering::Relaxed);
        self.collection_mutates.store(0, Ordering::Relaxed);
        self.checksum_calculations.store(0, Ordering::Relaxed);
        self.function_calls.store(0, Ordering::Relaxed);
        self.control_flows.store(0, Ordering::Relaxed);
        self.arithmetic_ops.store(0, Ordering::Relaxed);
        self.other_ops.store(0, Ordering::Relaxed);
        self.fuel_consumed.store(0, Ordering::Relaxed);
    }

    /// Get a summary of all operation counts
    pub fn get_summary(&self) -> OperationSummary {
        OperationSummary {
            memory_reads: self.memory_reads.load(Ordering::Relaxed),
            memory_writes: self.memory_writes.load(Ordering::Relaxed),
            memory_grows: self.memory_grows.load(Ordering::Relaxed),
            collection_pushes: self.collection_pushes.load(Ordering::Relaxed),
            collection_pops: self.collection_pops.load(Ordering::Relaxed),
            collection_lookups: self.collection_lookups.load(Ordering::Relaxed),
            collection_inserts: self.collection_inserts.load(Ordering::Relaxed),
            collection_removes: self.collection_removes.load(Ordering::Relaxed),
            collection_validates: self.collection_validates.load(Ordering::Relaxed),
            collection_mutates: self.collection_mutates.load(Ordering::Relaxed),
            checksum_calculations: self.checksum_calculations.load(Ordering::Relaxed),
            function_calls: self.function_calls.load(Ordering::Relaxed),
            control_flows: self.control_flows.load(Ordering::Relaxed),
            arithmetic_ops: self.arithmetic_ops.load(Ordering::Relaxed),
            other_ops: self.other_ops.load(Ordering::Relaxed),
            fuel_consumed: self.fuel_consumed.load(Ordering::Relaxed),
        }
    }
}

/// A snapshot of operation counts
#[derive(Debug, Clone, Copy)]
pub struct OperationSummary {
    /// Number of memory read operations
    pub memory_reads: u64,
    /// Number of memory write operations
    pub memory_writes: u64,
    /// Number of memory grow operations
    pub memory_grows: u64,
    /// Number of collection push operations
    pub collection_pushes: u64,
    /// Number of collection pop operations
    pub collection_pops: u64,
    /// Number of collection lookup operations
    pub collection_lookups: u64,
    /// Number of collection insert operations
    pub collection_inserts: u64,
    /// Number of collection remove operations
    pub collection_removes: u64,
    /// Number of collection validate operations
    pub collection_validates: u64,
    /// Number of collection mutate operations
    pub collection_mutates: u64,
    /// Number of checksum calculations
    pub checksum_calculations: u64,
    /// Number of function calls
    pub function_calls: u64,
    /// Number of control flow operations
    pub control_flows: u64,
    /// Number of arithmetic operations
    pub arithmetic_ops: u64,
    /// Number of other operations
    pub other_ops: u64,
    /// Total fuel consumed by operations
    pub fuel_consumed: u64,
}

/// Trait for types that need to track operation counts
pub trait OperationTracking {
    /// Record an operation occurred
    fn record_operation(&self, op_type: OperationType);

    /// Get the current operation statistics
    fn operation_stats(&self) -> OperationSummary;

    /// Reset operation counters
    fn reset_operation_stats(&self);
}

// Global operation counter for use when a local counter isn't available
#[cfg(feature = "std")]
static GLOBAL_COUNTER: OnceLock<OperationCounter> = OnceLock::new();

// For no_std environment, we'll use a safer approach with atomic flags for initialization
#[cfg(not(feature = "std"))]
mod global_counter {
    use super::*;
    use alloc::boxed::Box;
    use core::sync::atomic::{AtomicBool, Ordering};

    // Storage for our global counter
    static mut COUNTER_STORAGE: Option<Box<OperationCounter>> = None;
    // Flag to track initialization
    static INITIALIZED: AtomicBool = AtomicBool::new(false);
    // Flag to track if initialization is in progress to avoid reentrancy issues
    static INITIALIZING: AtomicBool = AtomicBool::new(false);

    /// Get or initialize the counter with proper synchronization
    pub fn get_counter() -> &'static OperationCounter {
        if !INITIALIZED.load(Ordering::Acquire) {
            // Use compare_exchange to ensure only one thread initializes
            if INITIALIZING
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_err()
            {
                // Another thread is initializing - spin wait
                while !INITIALIZED.load(Ordering::Acquire) && INITIALIZING.load(Ordering::Relaxed) {
                    core::hint::spin_loop();
                }
            } else {
                // We're the initializing thread
                let counter = Box::new(OperationCounter::new());
                unsafe {
                    COUNTER_STORAGE = Some(counter);
                    // Before marking as initialized, ensure all writes are visible
                    core::sync::atomic::fence(Ordering::Release);
                    INITIALIZED.store(true, Ordering::Release);
                    INITIALIZING.store(false, Ordering::Release);
                }
            }
        }

        // Counter is now guaranteed to be initialized
        unsafe {
            // This is safe because we've guaranteed initialization
            // and we never modify or drop the Box after initialization
            COUNTER_STORAGE.as_ref().unwrap()
        }
    }

    /// Get a mutable reference to the counter - only used for reset operations
    pub fn get_counter_mut() -> &'static mut OperationCounter {
        if !INITIALIZED.load(Ordering::Acquire) {
            // Initialize if not already done
            let _ = get_counter();
        }

        unsafe {
            // This is safe because we've guaranteed initialization,
            // and we use appropriate atomic operations to ensure synchronization
            // for mutation operations
            COUNTER_STORAGE.as_mut().unwrap()
        }
    }
}

/// Get the global counter instance, initializing it if necessary
#[cfg(feature = "std")]
fn global_counter() -> &'static OperationCounter {
    // Compatibility implementation for MSRV < 1.86.0
    // This manually checks and initializes if not already set
    if let Some(counter) = GLOBAL_COUNTER.get() {
        counter
    } else {
        // This has a race condition, but OnceLock ensures only the first setter wins
        let _ = GLOBAL_COUNTER.set(OperationCounter::new());
        // This is now guaranteed to return Some
        GLOBAL_COUNTER.get().unwrap()
    }
}

/// Record an operation in the global counter
#[cfg(feature = "std")]
pub fn record_global_operation(op_type: OperationType, level: VerificationLevel) {
    global_counter().record_operation(op_type, level);
}

/// Record an operation in the global counter
#[cfg(not(feature = "std"))]
pub fn record_global_operation(op_type: OperationType, level: VerificationLevel) {
    global_counter::get_counter().record_operation(op_type, level);
}

/// Get the summary from the global counter
#[cfg(feature = "std")]
pub fn global_operation_summary() -> OperationSummary {
    global_counter().get_summary()
}

/// Get the summary from the global counter
#[cfg(not(feature = "std"))]
pub fn global_operation_summary() -> OperationSummary {
    global_counter::get_counter().get_summary()
}

/// Reset the global counter
#[cfg(feature = "std")]
pub fn reset_global_operations() {
    global_counter().reset();
}

/// Reset the global counter
#[cfg(not(feature = "std"))]
pub fn reset_global_operations() {
    // For reset, we need to call an atomic method of OperationCounter
    // which is safe because OperationCounter uses atomics internally
    global_counter::get_counter().reset();
}

/// Get the total fuel consumed from the global counter
#[cfg(feature = "std")]
pub fn global_fuel_consumed() -> u64 {
    global_counter().get_fuel_consumed()
}

/// Get the total fuel consumed from the global counter
#[cfg(not(feature = "std"))]
pub fn global_fuel_consumed() -> u64 {
    global_counter::get_counter().get_fuel_consumed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_counter() {
        let counter = OperationCounter::new();

        // Record some operations
        counter.record_operation(OperationType::MemoryRead, VerificationLevel::Standard);
        counter.record_operation(OperationType::MemoryWrite, VerificationLevel::Standard);
        counter.record_operation(OperationType::CollectionPush, VerificationLevel::Standard);

        // Get the summary
        let summary = counter.get_summary();

        // Check individual counts
        assert_eq!(summary.memory_reads, 1);
        assert_eq!(summary.memory_writes, 1);
        assert_eq!(summary.collection_pushes, 1);

        // Fuel consumed should be sum of operations with verification multiplier
        let expected_fuel = (OperationType::MemoryRead.fuel_cost() as f64 * 1.5).round() as u64
            + (OperationType::MemoryWrite.fuel_cost() as f64 * 1.5).round() as u64
            + (OperationType::CollectionPush.fuel_cost() as f64 * 1.5).round() as u64;

        assert_eq!(summary.fuel_consumed, expected_fuel);

        // Test reset
        counter.reset();
        let summary_after_reset = counter.get_summary();
        assert_eq!(summary_after_reset.memory_reads, 0);
        assert_eq!(summary_after_reset.fuel_consumed, 0);
    }

    #[test]
    fn test_verification_level_impact() {
        let counter = OperationCounter::new();

        // Same operation with different verification levels
        counter.record_operation(OperationType::MemoryRead, VerificationLevel::None);
        counter.record_operation(OperationType::MemoryRead, VerificationLevel::Standard);
        counter.record_operation(OperationType::MemoryRead, VerificationLevel::Full);

        // The fuel cost should reflect the different verification levels
        let summary = counter.get_summary();

        // Memory reads should be 3
        assert_eq!(summary.memory_reads, 3);

        // Expected fuel: 1*1.0 + 1*1.5 + 1*2.0 = 4.5 -> 5 (rounded)
        let expected_fuel = (OperationType::MemoryRead.fuel_cost() as f64 * 1.0).round() as u64
            + (OperationType::MemoryRead.fuel_cost() as f64 * 1.5).round() as u64
            + (OperationType::MemoryRead.fuel_cost() as f64 * 2.0).round() as u64;

        assert_eq!(summary.fuel_consumed, expected_fuel);
    }

    #[test]
    fn test_global_counter() {
        // Reset global counter
        reset_global_operations();

        // Record some operations
        record_global_operation(OperationType::FunctionCall, VerificationLevel::Standard);
        record_global_operation(
            OperationType::CollectionValidate,
            VerificationLevel::Standard,
        );

        // Get global summary
        let summary = global_operation_summary();

        // Check counts
        assert_eq!(summary.function_calls, 1);
        assert_eq!(summary.collection_validates, 1);

        // Check global fuel consumed
        let fuel = global_fuel_consumed();
        assert_eq!(fuel, summary.fuel_consumed);

        // Reset and check again
        reset_global_operations();
        assert_eq!(global_fuel_consumed(), 0);
    }
}
