//! Bounded Infrastructure for Runtime
//!
//! This module provides bounded alternatives for runtime collections
//! to ensure static memory allocation throughout the runtime execution.

#![cfg_attr(not(feature = "std"), no_std)]

use wrt_foundation::{
    bounded::{BoundedVec, BoundedString},
    no_std_hashmap::BoundedHashMap,
    budget_provider::BudgetProvider,
    budget_aware_provider::{BudgetAwareProviderFactory, CrateId},
    WrtResult,
};

/// Budget-aware memory provider for runtime (256KB)
pub type RuntimeProvider = BudgetProvider<262144>;

/// Maximum number of runtime instances
pub const MAX_RUNTIME_INSTANCES: usize = 64;

/// Maximum number of modules per runtime
pub const MAX_MODULES_PER_RUNTIME: usize = 256;

/// Maximum number of function instances
pub const MAX_FUNCTION_INSTANCES: usize = 8192;

/// Maximum number of memory instances
pub const MAX_MEMORY_INSTANCES: usize = 64;

/// Maximum number of table instances
pub const MAX_TABLE_INSTANCES: usize = 64;

/// Maximum number of global instances
pub const MAX_GLOBAL_INSTANCES: usize = 2048;

/// Maximum number of threads in thread manager
pub const MAX_MANAGED_THREADS: usize = 128;

/// Maximum call stack depth per thread
pub const MAX_CALL_STACK_DEPTH: usize = 2048;

/// Maximum number of execution contexts
pub const MAX_EXECUTION_CONTEXTS: usize = 256;

/// Maximum number of branch predictions
pub const MAX_BRANCH_PREDICTIONS: usize = 4096;

/// Maximum number of function predictors
pub const MAX_FUNCTION_PREDICTORS: usize = 1024;

/// Maximum number of memory allocations to track
pub const MAX_MEMORY_ALLOCATIONS: usize = 2048;

/// Maximum number of wait queue entries
pub const MAX_WAIT_QUEUE_ENTRIES: usize = 512;

/// Maximum number of atomic operations
pub const MAX_ATOMIC_OPERATIONS: usize = 1024;

/// Maximum number of interpreter optimizations
pub const MAX_INTERPRETER_OPTIMIZATIONS: usize = 2048;

/// Maximum module name length
pub const MAX_MODULE_NAME_LEN: usize = 256;

/// Maximum function name length
pub const MAX_FUNCTION_NAME_LEN: usize = 256;

/// Maximum import/export name length
pub const MAX_IMPORT_EXPORT_NAME_LEN: usize = 256;

/// Maximum number of imports per module
pub const MAX_IMPORTS_PER_MODULE: usize = 1024;

/// Maximum number of exports per module
pub const MAX_EXPORTS_PER_MODULE: usize = 1024;

/// Maximum stack frame locals
pub const MAX_FRAME_LOCALS: usize = 512;

/// Maximum block context depth
pub const MAX_BLOCK_CONTEXT_DEPTH: usize = 256;

/// Bounded vector for runtime instances
pub type BoundedRuntimeVec<T> = BoundedVec<T, MAX_RUNTIME_INSTANCES, RuntimeProvider>;

/// Bounded vector for modules
pub type BoundedModuleVec<T> = BoundedVec<T, MAX_MODULES_PER_RUNTIME, RuntimeProvider>;

/// Bounded vector for functions
pub type BoundedFunctionVec<T> = BoundedVec<T, MAX_FUNCTION_INSTANCES, RuntimeProvider>;

/// Bounded vector for memory instances
pub type BoundedMemoryVec<T> = BoundedVec<T, MAX_MEMORY_INSTANCES, RuntimeProvider>;

/// Bounded vector for table instances
pub type BoundedTableVec<T> = BoundedVec<T, MAX_TABLE_INSTANCES, RuntimeProvider>;

/// Bounded vector for global instances
pub type BoundedGlobalVec<T> = BoundedVec<T, MAX_GLOBAL_INSTANCES, RuntimeProvider>;

/// Bounded vector for managed threads
pub type BoundedThreadVec<T> = BoundedVec<T, MAX_MANAGED_THREADS, RuntimeProvider>;

/// Bounded vector for call stack
pub type BoundedCallStackVec<T> = BoundedVec<T, MAX_CALL_STACK_DEPTH, RuntimeProvider>;

/// Bounded vector for execution contexts
pub type BoundedExecutionContextVec<T> = BoundedVec<T, MAX_EXECUTION_CONTEXTS, RuntimeProvider>;

/// Bounded vector for memory allocations
pub type BoundedMemoryAllocationVec<T> = BoundedVec<T, MAX_MEMORY_ALLOCATIONS, RuntimeProvider>;

/// Bounded vector for wait queue
pub type BoundedWaitQueueVec<T> = BoundedVec<T, MAX_WAIT_QUEUE_ENTRIES, RuntimeProvider>;

/// Bounded vector for frame locals
pub type BoundedFrameLocalsVec<T> = BoundedVec<T, MAX_FRAME_LOCALS, RuntimeProvider>;

/// Bounded vector for block contexts
pub type BoundedBlockContextVec<T> = BoundedVec<T, MAX_BLOCK_CONTEXT_DEPTH, RuntimeProvider>;

/// Bounded string for module names
pub type BoundedModuleName = BoundedString<MAX_MODULE_NAME_LEN, RuntimeProvider>;

/// Bounded string for function names
pub type BoundedFunctionName = BoundedString<MAX_FUNCTION_NAME_LEN, RuntimeProvider>;

/// Bounded string for import/export names
pub type BoundedImportExportName = BoundedString<MAX_IMPORT_EXPORT_NAME_LEN, RuntimeProvider>;

/// Bounded map for branch predictions
pub type BoundedBranchPredictionMap<V> = BoundedHashMap<
    u32, // PC address
    V,
    MAX_BRANCH_PREDICTIONS,
    RuntimeProvider
>;

/// Bounded map for function predictors
pub type BoundedFunctionPredictorMap<V> = BoundedHashMap<
    u32, // Function index
    V,
    MAX_FUNCTION_PREDICTORS,
    RuntimeProvider
>;

/// Bounded map for interpreter optimizations
pub type BoundedInterpreterOptMap<V> = BoundedHashMap<
    u32, // Instruction index
    V,
    MAX_INTERPRETER_OPTIMIZATIONS,
    RuntimeProvider
>;

/// Bounded map for atomic operations
pub type BoundedAtomicOpMap<V> = BoundedHashMap<
    u64, // Memory address
    V,
    MAX_ATOMIC_OPERATIONS,
    RuntimeProvider
>;

/// Bounded map for modules
pub type BoundedModuleMap<V> = BoundedHashMap<
    BoundedModuleName,
    V,
    MAX_MODULES_PER_RUNTIME,
    RuntimeProvider
>;

/// Bounded map for imports
pub type BoundedImportMap<V> = BoundedHashMap<
    BoundedImportExportName,
    V,
    MAX_IMPORTS_PER_MODULE,
    RuntimeProvider
>;

/// Bounded map for exports
pub type BoundedExportMap<V> = BoundedHashMap<
    BoundedImportExportName,
    V,
    MAX_EXPORTS_PER_MODULE,
    RuntimeProvider
>;

/// Bounded map for thread management
pub type BoundedThreadMap<V> = BoundedHashMap<
    u32, // Thread ID
    V,
    MAX_MANAGED_THREADS,
    RuntimeProvider
>;

/// Create a new bounded runtime vector
pub fn new_runtime_vec<T>() -> WrtResult<BoundedRuntimeVec<T>> {
    let provider = RuntimeProvider::new(CrateId::Runtime)?;
    BoundedVec::new(provider)
}

/// Create a new bounded module vector
pub fn new_module_vec<T>() -> BoundedModuleVec<T> {
    BoundedVec::new(RuntimeProvider::new(CrateId::Runtime)?).unwrap_or_else(|_| {
        panic!("Failed to create bounded module vector");
    })
}

/// Create a new bounded function vector
pub fn new_function_vec<T>() -> BoundedFunctionVec<T> {
    BoundedVec::new(RuntimeProvider::new(CrateId::Runtime)?).unwrap_or_else(|_| {
        panic!("Failed to create bounded function vector");
    })
}

/// Create a new bounded memory vector
pub fn new_memory_vec<T>() -> BoundedMemoryVec<T> {
    BoundedVec::new(RuntimeProvider::new(CrateId::Runtime)?).unwrap_or_else(|_| {
        panic!("Failed to create bounded memory vector");
    })
}

/// Create a new bounded table vector
pub fn new_table_vec<T>() -> BoundedTableVec<T> {
    BoundedVec::new(RuntimeProvider::new(CrateId::Runtime)?).unwrap_or_else(|_| {
        panic!("Failed to create bounded table vector");
    })
}

/// Create a new bounded global vector
pub fn new_global_vec<T>() -> BoundedGlobalVec<T> {
    BoundedVec::new(RuntimeProvider::new(CrateId::Runtime)?).unwrap_or_else(|_| {
        panic!("Failed to create bounded global vector");
    })
}

/// Create a new bounded thread vector
pub fn new_thread_vec<T>() -> BoundedThreadVec<T> {
    BoundedVec::new(RuntimeProvider::new(CrateId::Runtime)?).unwrap_or_else(|_| {
        panic!("Failed to create bounded thread vector");
    })
}

/// Create a new bounded call stack vector
pub fn new_call_stack_vec<T>() -> BoundedCallStackVec<T> {
    BoundedVec::new(RuntimeProvider::new(CrateId::Runtime)?).unwrap_or_else(|_| {
        panic!("Failed to create bounded call stack vector");
    })
}

/// Create a new bounded execution context vector
pub fn new_execution_context_vec<T>() -> BoundedExecutionContextVec<T> {
    BoundedVec::new(RuntimeProvider::new(CrateId::Runtime)?).unwrap_or_else(|_| {
        panic!("Failed to create bounded execution context vector");
    })
}

/// Create a new bounded module name
pub fn new_module_name() -> BoundedModuleName {
    BoundedString::new(RuntimeProvider::new(CrateId::Runtime)?)
}

/// Create a bounded module name from str
pub fn bounded_module_name_from_str(s: &str) -> wrt_error::Result<BoundedModuleName> {
    BoundedString::from_str(s, RuntimeProvider::new(CrateId::Runtime)?)
}

/// Create a new bounded function name
pub fn new_function_name() -> BoundedFunctionName {
    BoundedString::new(RuntimeProvider::new(CrateId::Runtime)?)
}

/// Create a bounded function name from str
pub fn bounded_function_name_from_str(s: &str) -> wrt_error::Result<BoundedFunctionName> {
    BoundedString::from_str(s, RuntimeProvider::new(CrateId::Runtime)?)
}

/// Create a new bounded branch prediction map
pub fn new_branch_prediction_map<V>() -> BoundedBranchPredictionMap<V> {
    BoundedHashMap::new(RuntimeProvider::new(CrateId::Runtime)?).unwrap_or_else(|_| {
        panic!("Failed to create bounded branch prediction map");
    })
}

/// Create a new bounded function predictor map
pub fn new_function_predictor_map<V>() -> BoundedFunctionPredictorMap<V> {
    BoundedHashMap::new(RuntimeProvider::new(CrateId::Runtime)?).unwrap_or_else(|_| {
        panic!("Failed to create bounded function predictor map");
    })
}

/// Create a new bounded interpreter optimization map
pub fn new_interpreter_opt_map<V>() -> BoundedInterpreterOptMap<V> {
    BoundedHashMap::new(RuntimeProvider::new(CrateId::Runtime)?).unwrap_or_else(|_| {
        panic!("Failed to create bounded interpreter optimization map");
    })
}

/// Create a new bounded atomic operation map
pub fn new_atomic_op_map<V>() -> BoundedAtomicOpMap<V> {
    BoundedHashMap::new(RuntimeProvider::new(CrateId::Runtime)?).unwrap_or_else(|_| {
        panic!("Failed to create bounded atomic operation map");
    })
}

/// Create a new bounded module map
pub fn new_module_map<V>() -> BoundedModuleMap<V> {
    BoundedHashMap::new(RuntimeProvider::new(CrateId::Runtime)?).unwrap_or_else(|_| {
        panic!("Failed to create bounded module map");
    })
}

/// Create a new bounded import map
pub fn new_import_map<V>() -> BoundedImportMap<V> {
    BoundedHashMap::new(RuntimeProvider::new(CrateId::Runtime)?).unwrap_or_else(|_| {
        panic!("Failed to create bounded import map");
    })
}

/// Create a new bounded export map
pub fn new_export_map<V>() -> BoundedExportMap<V> {
    BoundedHashMap::new(RuntimeProvider::new(CrateId::Runtime)?).unwrap_or_else(|_| {
        panic!("Failed to create bounded export map");
    })
}

/// Create a new bounded thread map
pub fn new_thread_map<V>() -> BoundedThreadMap<V> {
    BoundedHashMap::new(RuntimeProvider::new(CrateId::Runtime)?).unwrap_or_else(|_| {
        panic!("Failed to create bounded thread map");
    })
}