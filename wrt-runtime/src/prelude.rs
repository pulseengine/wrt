//! Prelude module for wrt-runtime
//!
//! This module provides a unified set of imports for both std and `no_std`
//! environments. It re-exports commonly used types and traits to ensure
//! consistency across all crates in the WRT project and simplify imports in
//! individual modules.

// Core imports for both std and no_std environments

#[cfg(not(feature = "std"))]
extern crate alloc;

// Binary std/no_std choice
#[cfg(not(feature = "std"))]
pub use wrt_foundation::{
    NoStdProvider,
};

// Platform-aware collection type aliases that adapt to target platform capabilities
/// `HashMap` type for `no_std` environments with bounded capacity
#[cfg(not(feature = "std"))]
pub type HashMap<K, V> = wrt_foundation::BoundedMap<K, V, 128, wrt_foundation::memory_system::MediumProvider>;

/// `HashSet` type for `no_std` environments with bounded capacity
#[cfg(not(feature = "std"))]
pub type HashSet<T> = wrt_foundation::BoundedSet<T, 128, wrt_foundation::memory_system::MediumProvider>;

// Platform-aware string and vector types
#[cfg(not(feature = "std"))]
pub use wrt_foundation::bounded::BoundedString;

#[cfg(not(feature = "std"))]
pub use alloc::string::{String, ToString};

// Note: Use alloc::vec::Vec directly for no_std mode
#[cfg(not(feature = "std"))]
pub use alloc::vec::Vec;

// Helper macro to create Vec 
/// Create a new Vec for `no_std` environments
#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! vec_new {
    () => {
        Vec::new()
    };
}

// Helper function to create Vec with capacity
/// Create a Vec with specified capacity for `no_std` environments
#[cfg(not(feature = "std"))]
#[must_use] pub fn vec_with_capacity<T>(capacity: usize) -> Vec<T> {
    Vec::with_capacity(capacity)
}

// Add vec! macro for no_std environments without alloc
/// Vec creation macro for pure `no_std` environments without alloc
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
#[macro_export]
macro_rules! vec {
    () => {
        Vec::new()
    };
    ($elem:expr; $n:expr) => {
        {
            let mut v = Vec::new();
            for _ in 0..$n {
                v.push($elem);
            }
            v
        }
    };
    ($($x:expr),*) => {
        {
            let mut v = Vec::new();
            $(v.push($x);)*
            v
        }
    };
}

// Simple format! implementation for no_std mode using a fixed buffer
/// Simplified format macro for `no_std` environments
#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! format {
    ($fmt:expr) => {{
        $fmt
    }};
    ($fmt:expr, $($arg:tt)*) => {{
        $fmt  // Simplified - just return the format string for no_std
    }};
}


// Re-export the macros for no_std
#[cfg(not(feature = "std"))]
pub use crate::format;

// Helper functions for Option<Value> conversion
/// Convert Option<Value> to Option<i32> for `no_std` environments
#[cfg(not(feature = "std"))]
pub fn option_value_as_i32(value: &Option<wrt_foundation::Value>) -> Option<i32> {
    match value {
        Some(wrt_foundation::Value::I32(val)) => Some(*val),
        _ => None,
    }
}

/// Convert Option<Value> to Option<i64> for `no_std` environments
#[cfg(not(feature = "std"))]
pub fn option_value_as_i64(value: &Option<wrt_foundation::Value>) -> Option<i64> {
    match value {
        Some(wrt_foundation::Value::I64(val)) => Some(*val),
        _ => None,
    }
}

/// Convert Option<Value> to Option<f32> for `no_std` environments
#[cfg(not(feature = "std"))]
pub fn option_value_as_f32(value: &Option<wrt_foundation::Value>) -> Option<f32> {
    match value {
        Some(wrt_foundation::Value::F32(val)) => Some(val.value()),
        _ => None,
    }
}

/// Convert Option<Value> to Option<f64> for `no_std` environments
#[cfg(not(feature = "std"))]
pub fn option_value_as_f64(value: &Option<wrt_foundation::Value>) -> Option<f64> {
    match value {
        Some(wrt_foundation::Value::F64(val)) => Some(val.value()),
        _ => None,
    }
}

// ToString is provided by alloc when available

// Arc and Mutex for no_std with alloc
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use alloc::sync::Arc;

// For pure no_std without alloc, use reference wrapper
/// Arc-like reference wrapper for pure `no_std` environments without alloc
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
#[derive(Debug, Clone)]
pub struct Arc<T> {
    /// Inner value being wrapped
    inner: T,
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl<T> Arc<T> {
    /// Create a new Arc wrapper for pure `no_std` environments
    pub fn new(value: T) -> Self {
        Self { inner: value }
    }
    
    /// Compare Arc pointers (always returns false in `no_std` mode)
    pub fn ptr_eq(_this: &Self, _other: &Self) -> bool {
        // In no_std mode, we can't do pointer comparison, so just return false
        false
    }
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl<T: PartialEq> PartialEq for Arc<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl<T: Eq> Eq for Arc<T> {}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl<T> core::ops::Deref for Arc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
pub use core::{
    any::Any,
    cmp::{Eq, Ord, PartialEq, PartialOrd},
    convert::{TryFrom, TryInto},
    fmt,
    fmt::{Debug, Display},
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    slice, str,
    sync::atomic::{AtomicUsize, Ordering as AtomicOrdering},
};
// Re-export from std when the std feature is enabled
#[cfg(feature = "std")]
pub use std::{
    boxed::Box,
    collections::{HashMap, HashSet},
    format,
    string::{String, ToString},
    sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard},
    vec,
    vec::Vec,
};

// Re-export from alloc when available but not std
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

// Remove duplicate definitions - Vec and String are already defined above

// Re-export from wrt-decoder (aliased to avoid name clashes)
// Component module is temporarily disabled in wrt-decoder
// #[cfg(feature = "std")]
// pub use wrt_decoder::component::Component as DecoderComponentDefinition;
// Re-export from wrt-instructions for instruction types
// Decoder imports are optional and may not be available
// pub use wrt_decoder::instructions::Instruction;
// pub use wrt_decoder::prelude::Module as DecoderModule;
// Re-export from wrt-error for error handling
pub use wrt_error::prelude::{
    codes,
    kinds::{
        self, ComponentError, InvalidType, OutOfBoundsError, ParseError, ResourceError,
        RuntimeError, ValidationError,
    },
    Error, ErrorCategory, Result,
};
// Re-export from wrt-format for format specifications (aliased to avoid name clashes)
#[cfg(feature = "std")]
pub use wrt_format::component::Component as FormatComponent;
#[cfg(feature = "std")]
pub use wrt_format::{
    module::{
        Data as FormatData, Element as FormatElement, Export as FormatExport,
        ExportKind as FormatExportKind, Function as FormatFunction, Global as FormatGlobal,
        Import as FormatImport, ImportDesc as FormatImportDesc, Memory as FormatMemory,
        Table as FormatTable,
    },
    section::CustomSection as FormatCustomSection,
};
// Re-export from wrt-foundation for core types
#[cfg(feature = "std")]
pub use wrt_foundation::component::ComponentType;
// Re-export core types from wrt_foundation instead of wrt_format
pub use wrt_foundation::types::{
    CustomSection, /* Assuming this is the intended replacement for FormatCustomSection
                   * Add other direct re-exports from wrt_foundation::types if they were
                   * previously from wrt_format::module e.g., DataSegment,
                   * ElementSegment, Export, GlobalType, Import, MemoryType, TableType,
                   * FuncType For now, only replacing what was directly
                   * aliased or used in a way that implies a direct replacement need. */
};
pub use wrt_foundation::{
    prelude::{
        BoundedStack, BoundedVec,
        GlobalType as CoreGlobalType, MemoryType as CoreMemoryType, ResourceType,
        SafeMemoryHandler, SafeSlice, TableType as CoreTableType,
        Value, ValueType, VerificationLevel,
    },
    safe_memory::SafeStack,
    types::{Limits, RefValue, ElementSegment, DataSegment},
    traits::BoundedCapacity, // Add trait for len(), is_empty(), etc.
    MemoryStats,
};

// Type aliases with platform-aware memory provider for the runtime
/// Default memory provider for runtime operations (64KB buffer)
pub type DefaultProvider = wrt_foundation::safe_memory::NoStdProvider<65536>;
/// WebAssembly instruction type with default provider
pub type Instruction = wrt_foundation::types::Instruction<DefaultProvider>;
/// Function type with default provider
pub type FuncType = wrt_foundation::types::FuncType<DefaultProvider>;
/// Runtime function type alias for consistency
pub type RuntimeFuncType = wrt_foundation::types::FuncType<DefaultProvider>;
/// WebAssembly global variable type
pub type GlobalType = wrt_foundation::types::GlobalType;
/// WebAssembly memory type
pub type MemoryType = wrt_foundation::types::MemoryType;
/// WebAssembly table type
pub type TableType = wrt_foundation::types::TableType;
/// External type for component model with default provider
pub type ExternType = wrt_foundation::component::ExternType<DefaultProvider>;

// Safety-critical wrapper types for runtime (deterministic, verifiable)
pub use crate::module::{TableWrapper as RuntimeTable, MemoryWrapper as RuntimeMemory, GlobalWrapper as RuntimeGlobal};

// Binary std/no_std choice
#[cfg(feature = "std")]
pub use wrt_foundation::prelude::{ComponentValue, ValType as ComponentValType};
// Re-export from wrt-host (for runtime host interaction items)
pub use wrt_host::prelude::CallbackRegistry as HostFunctionRegistry;
pub use wrt_host::prelude::HostFunctionHandler as HostFunction;
pub use wrt_instructions::{
    control_ops::BranchTarget as Label, instruction_traits::PureInstruction as InstructionExecutor,
};
// Re-export from wrt-intercept (for runtime interception items)
pub use wrt_intercept::prelude::LinkInterceptor as InterceptorRegistry;
pub use wrt_intercept::prelude::LinkInterceptorStrategy as InterceptStrategy;
// Binary std/no_std choice
#[cfg(not(feature = "std"))]
pub use wrt_sync::{
    WrtMutex as Mutex, WrtMutexGuard as MutexGuard, WrtRwLock as RwLock,
    WrtRwLockReadGuard as RwLockReadGuard, WrtRwLockWriteGuard as RwLockWriteGuard,
};

// Execution related types defined in wrt-runtime
pub use crate::execution::{ExecutionContext, ExecutionStats}; /* Removed ExecutionResult as
                                                                * it's not defined in
                                                                * execution.rs */
// --- Local definitions from wrt-runtime ---
// These are types defined within wrt-runtime itself, re-exported for convenience if used
// widely.

// Core runtime structures
// pub use crate::func::Function; // Removed: func.rs only re-exports FuncType from
// wrt_foundation, which is already in prelude. RuntimeFunction (from module.rs) is the primary
// Function struct for the runtime.
pub use crate::global::Global;
// Adapters and helpers if they are part of the public API exported by this prelude
// Temporarily disabled - memory_adapter module is disabled
// pub use crate::memory_adapter::MemoryAdapter;
// Module items specific to wrt-runtime module structure
pub use crate::module::{Data, Element, Export, ExportItem, ExportKind, Import, OtherExport};
// Stackless execution engine components - temporarily disabled
// pub use crate::stackless::{
//     StacklessCallbackRegistry, StacklessEngine, StacklessExecutionState, StacklessFrame,
//     StacklessStack,
// };
pub use crate::{
    memory::Memory, module::Module as RuntimeModule,
    module_instance::ModuleInstance as RuntimeModuleInstance, table::Table,
};

// The following re-exports from wrt_format are removed as wrt-runtime should
// not depend on wrt-format. pub use wrt_format::module::{ // REMOVED
// DataMode as FormatDataMode, Element as FormatElement, Export as FormatExport,
// // ... and others }; // REMOVED
// pub use wrt_format::section::CustomSection as FormatCustomSection; // REMOVED
// pub use wrt_format::component::Component as FormatComponent; // REMOVED
// (Component model types should come from wrt_component or wrt_foundation if
// foundational)
