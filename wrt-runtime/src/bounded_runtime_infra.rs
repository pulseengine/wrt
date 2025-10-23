//! Bounded Infrastructure for Runtime
//!
//! This module provides bounded alternatives for runtime collections
//! to ensure static memory allocation throughout the runtime execution.

use wrt_error::{
    Error,
    ErrorCategory,
};
#[cfg(any(feature = "std", feature = "alloc"))]
use wrt_foundation::capabilities::factory::CapabilityGuardedProvider;
// Box is re-exported by wrt_foundation
use wrt_foundation::Box;
use wrt_foundation::{
    bounded::{
        BoundedString,
        BoundedVec,
    },
    bounded_collections::BoundedMap,
    budget_aware_provider::CrateId,
    capabilities::CapabilityAwareProvider,
    capability_context,
    safe_capability_alloc,
    safe_memory::NoStdProvider,
    traits::{
        Checksummable,
        FromBytes,
        ToBytes,
    },
    WrtResult,
};

// Memory size for runtime provider (4KB to avoid stack overflow)
// Previously was 131072 (128KB) which caused stack overflow
// 4096 was too small for runtime providers - increased to 32KB
pub const RUNTIME_MEMORY_SIZE: usize = 32768;

// Stack allocation threshold - use platform allocator for sizes above this
const STACK_ALLOCATION_THRESHOLD: usize = 4096; // 4KB

/// Base memory provider for runtime
/// Always uses NoStdProvider as the base provider
pub type BaseRuntimeProvider = NoStdProvider<RUNTIME_MEMORY_SIZE>;

/// Budget-aware memory provider for runtime
/// Uses CapabilityAwareProvider wrapper in std/alloc environments
#[cfg(any(feature = "std", feature = "alloc"))]
pub type RuntimeProvider = CapabilityAwareProvider<BaseRuntimeProvider>;

/// Budget-aware memory provider for runtime
/// Uses CapabilityAwareProvider wrapper in no_std environments too
#[cfg(not(any(feature = "std", feature = "alloc")))]
pub type RuntimeProvider = CapabilityAwareProvider<BaseRuntimeProvider>;

/// Default runtime provider alias for backward compatibility
pub type DefaultRuntimeProvider = RuntimeProvider;

/// Helper function to create a runtime provider using an existing context
pub fn create_runtime_provider_with_context(
    _context: &wrt_foundation::capabilities::MemoryCapabilityContext,
) -> WrtResult<RuntimeProvider> {
    use wrt_foundation::{
        capabilities::{
            DynamicMemoryCapability,
            MemoryCapability,
        },
        verification::VerificationLevel,
    };

    #[cfg(any(feature = "std", feature = "alloc"))]
    {
        // Create provider directly without going through global context to avoid
        // recursion
        let base_provider = BaseRuntimeProvider::default();

        // Create a simple capability for the runtime
        let capability = DynamicMemoryCapability::new(
            RUNTIME_MEMORY_SIZE,
            CrateId::Runtime,
            VerificationLevel::Standard,
        );

        let provider =
            CapabilityAwareProvider::new(base_provider, Box::new(capability), CrateId::Runtime);
        Ok(provider)
    }
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    {
        // In no_std environments, use the lightweight provider creation
        let base_provider = BaseRuntimeProvider::default();

        // Create a simple capability for the runtime
        let capability = DynamicMemoryCapability::new(
            RUNTIME_MEMORY_SIZE,
            CrateId::Runtime,
            VerificationLevel::Standard,
        );

        let provider =
            CapabilityAwareProvider::new(base_provider, Box::new(capability), CrateId::Runtime);
        Ok(provider)
    }
}

/// Helper function to create a runtime provider
///
/// This creates a new context which can cause recursion. Use
/// create_runtime_provider_with_context instead.
pub fn create_runtime_provider() -> WrtResult<RuntimeProvider> {
    // For small sizes, use the normal capability system
    #[cfg(any(feature = "std", feature = "alloc"))]
    {
        // In std/alloc environments, safe_capability_alloc! returns
        // CapabilityAwareProvider
        let context = capability_context!(dynamic(CrateId::Runtime, RUNTIME_MEMORY_SIZE))?;
        let provider = safe_capability_alloc!(context, CrateId::Runtime, RUNTIME_MEMORY_SIZE)?;
        Ok(provider)
    }

    #[cfg(not(any(feature = "std", feature = "alloc")))]
    {
        // In no_std, safe_capability_alloc! returns the base provider
        let context = capability_context!(dynamic(CrateId::Runtime, RUNTIME_MEMORY_SIZE))?;
        let provider = safe_capability_alloc!(context, CrateId::Runtime, RUNTIME_MEMORY_SIZE)?;
        Ok(provider)
    }
}

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

/// Maximum number of memory allocations to track
pub const MAX_MEMORY_ALLOCATIONS: usize = 2048;

/// Maximum number of wait queue entries
pub const MAX_WAIT_QUEUE_ENTRIES: usize = 512;

/// Maximum number of atomic operations
pub const MAX_ATOMIC_OPERATIONS: usize = 1024;

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
pub type BoundedModuleName = BoundedString<MAX_MODULE_NAME_LEN>;

/// Bounded string for function names
pub type BoundedFunctionName = BoundedString<MAX_FUNCTION_NAME_LEN>;

/// Bounded string for import/export names
pub type BoundedImportExportName = BoundedString<MAX_IMPORT_EXPORT_NAME_LEN>;

/// Bounded map for atomic operations
pub type BoundedAtomicOpMap<V> = BoundedMap<
    u64, // Memory address
    V,
    MAX_ATOMIC_OPERATIONS,
    RuntimeProvider,
>;

/// Bounded map for modules
pub type BoundedModuleMap<V> =
    BoundedMap<BoundedModuleName, V, MAX_MODULES_PER_RUNTIME, RuntimeProvider>;

/// Bounded map for imports
pub type BoundedImportMap<V> =
    BoundedMap<BoundedImportExportName, V, MAX_IMPORTS_PER_MODULE, RuntimeProvider>;

/// Bounded map for exports
pub type BoundedExportMap<V> =
    BoundedMap<BoundedImportExportName, V, MAX_EXPORTS_PER_MODULE, RuntimeProvider>;

/// Bounded map for thread management
pub type BoundedThreadMap<V> = BoundedMap<
    u32, // Thread ID
    V,
    MAX_MANAGED_THREADS,
    RuntimeProvider,
>;

/// Create a new bounded runtime vector
pub fn new_runtime_vec<T>() -> WrtResult<BoundedRuntimeVec<T>>
where
    T: Sized + Checksummable + ToBytes + FromBytes + Default + Clone + PartialEq + Eq,
{
    let provider = create_runtime_provider()?;
    BoundedVec::new(provider)
}

/// Create a new bounded module vector
pub fn new_module_vec<T>() -> WrtResult<BoundedModuleVec<T>>
where
    T: Sized + Checksummable + ToBytes + FromBytes + Default + Clone + PartialEq + Eq,
{
    let provider = create_runtime_provider()?;
    BoundedVec::new(provider)
}

/// Create a new bounded function vector
pub fn new_function_vec<T>() -> WrtResult<BoundedFunctionVec<T>>
where
    T: Sized + Checksummable + ToBytes + FromBytes + Default + Clone + PartialEq + Eq,
{
    let provider = create_runtime_provider()?;
    BoundedVec::new(provider)
}

/// Create a new bounded memory vector
pub fn new_memory_vec<T>() -> WrtResult<BoundedMemoryVec<T>>
where
    T: Sized + Checksummable + ToBytes + FromBytes + Default + Clone + PartialEq + Eq,
{
    let provider = create_runtime_provider()?;
    BoundedVec::new(provider)
}

/// Create a new bounded table vector
pub fn new_table_vec<T>() -> WrtResult<BoundedTableVec<T>>
where
    T: Sized + Checksummable + ToBytes + FromBytes + Default + Clone + PartialEq + Eq,
{
    let provider = create_runtime_provider()?;
    BoundedVec::new(provider)
}

/// Create a new bounded global vector
pub fn new_global_vec<T>() -> WrtResult<BoundedGlobalVec<T>>
where
    T: Sized + Checksummable + ToBytes + FromBytes + Default + Clone + PartialEq + Eq,
{
    let provider = create_runtime_provider()?;
    BoundedVec::new(provider)
}

/// Create a new bounded thread vector
pub fn new_thread_vec<T>() -> WrtResult<BoundedThreadVec<T>>
where
    T: Sized + Checksummable + ToBytes + FromBytes + Default + Clone + PartialEq + Eq,
{
    let provider = create_runtime_provider()?;
    BoundedVec::new(provider)
}

/// Create a new bounded call stack vector
pub fn new_call_stack_vec<T>() -> WrtResult<BoundedCallStackVec<T>>
where
    T: Sized + Checksummable + ToBytes + FromBytes + Default + Clone + PartialEq + Eq,
{
    let provider = create_runtime_provider()?;
    BoundedVec::new(provider)
}

/// Create a new bounded execution context vector
pub fn new_execution_context_vec<T>() -> WrtResult<BoundedExecutionContextVec<T>>
where
    T: Sized + Checksummable + ToBytes + FromBytes + Default + Clone + PartialEq + Eq,
{
    let provider = create_runtime_provider()?;
    BoundedVec::new(provider)
}

/// Create a new bounded module name
pub fn new_module_name() -> WrtResult<BoundedModuleName> {
    BoundedString::from_str("")
        .map_err(|e| Error::memory_serialization_error("Failed to create bounded string"))
}

/// Create a bounded module name from str
pub fn bounded_module_name_from_str(s: &str) -> WrtResult<BoundedModuleName> {
    BoundedString::from_str(s)
        .map_err(|e| Error::memory_serialization_error("Failed to create bounded string"))
}

/// Create a new bounded function name
pub fn new_function_name() -> WrtResult<BoundedFunctionName> {
    BoundedString::from_str("")
        .map_err(|e| Error::memory_serialization_error("Failed to create bounded string"))
}

/// Create a bounded function name from str
pub fn bounded_function_name_from_str(s: &str) -> WrtResult<BoundedFunctionName> {
    BoundedString::from_str(s)
        .map_err(|e| Error::memory_serialization_error("Failed to create bounded string"))
}

/// Create a new bounded atomic operation map
pub fn new_atomic_op_map<V>() -> WrtResult<BoundedAtomicOpMap<V>>
where
    V: Sized + Checksummable + ToBytes + FromBytes + Default + Clone + PartialEq + Eq,
{
    let provider = create_runtime_provider()?;
    BoundedMap::new(provider)
}

/// Create a new bounded module map
pub fn new_module_map<V>() -> WrtResult<BoundedModuleMap<V>>
where
    V: Sized + Checksummable + ToBytes + FromBytes + Default + Clone + PartialEq + Eq,
{
    let provider = create_runtime_provider()?;
    BoundedMap::new(provider)
}

/// Create a new bounded import map
pub fn new_import_map<V>() -> WrtResult<BoundedImportMap<V>>
where
    V: Sized + Checksummable + ToBytes + FromBytes + Default + Clone + PartialEq + Eq,
{
    let provider = create_runtime_provider()?;
    BoundedMap::new(provider)
}

/// Create a new bounded export map
pub fn new_export_map<V>() -> WrtResult<BoundedExportMap<V>>
where
    V: Sized + Checksummable + ToBytes + FromBytes + Default + Clone + PartialEq + Eq,
{
    let provider = create_runtime_provider()?;
    BoundedMap::new(provider)
}

/// Create a new bounded thread map
pub fn new_thread_map<V>() -> WrtResult<BoundedThreadMap<V>>
where
    V: Sized + Checksummable + ToBytes + FromBytes + Default + Clone + PartialEq + Eq,
{
    let provider = create_runtime_provider()?;
    BoundedMap::new(provider)
}
