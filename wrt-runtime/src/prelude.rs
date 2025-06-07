//! Prelude module for wrt-runtime
//!
//! This module provides a unified set of imports for both std and no_std
//! environments. It re-exports commonly used types and traits to ensure
//! consistency across all crates in the WRT project and simplify imports in
//! individual modules.

// Core imports for both std and no_std environments

// Binary std/no_std choice
#[cfg(not(feature = "std"))]
pub use wrt_foundation::{
    NoStdProvider,
};

// Define HashMap and HashSet type aliases with all required generics
#[cfg(not(feature = "std"))]
pub type HashMap<K, V> = wrt_foundation::BoundedMap<K, V, 128, wrt_foundation::safe_memory::NoStdProvider<1024>>;

#[cfg(not(feature = "std"))]
pub type HashSet<T> = wrt_foundation::BoundedSet<T, 128, wrt_foundation::safe_memory::NoStdProvider<1024>>;

// For pure no_std, we'll rely on explicit BoundedVec usage instead of Vec alias
// to avoid conflicts with other crates' Vec definitions
#[cfg(not(feature = "std"))]
pub use wrt_foundation::bounded::BoundedString;

#[cfg(not(feature = "std"))]
pub type String = wrt_foundation::bounded::BoundedString<256, wrt_foundation::safe_memory::NoStdProvider<1024>>;

#[cfg(not(feature = "std"))]
pub type Vec<T> = wrt_foundation::bounded::BoundedVec<T, 256, wrt_foundation::safe_memory::NoStdProvider<1024>>;

// Helper macro to create BoundedVec with standard parameters
#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! vec_new {
    () => {
        wrt_foundation::bounded::BoundedVec::<_, 256, wrt_foundation::safe_memory::NoStdProvider<1024>>::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap()
    };
}

// Helper function to create BoundedVec with capacity (capacity is ignored in bounded collections)
#[cfg(not(feature = "std"))]
pub fn vec_with_capacity<T: wrt_foundation::traits::Checksummable + wrt_foundation::traits::ToBytes + wrt_foundation::traits::FromBytes + Default + Clone + core::fmt::Debug + PartialEq + Eq>(_capacity: usize) -> wrt_foundation::bounded::BoundedVec<T, 256, wrt_foundation::safe_memory::NoStdProvider<1024>> {
    wrt_foundation::bounded::BoundedVec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap()
}

// Add vec! macro for no_std environments
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
#[macro_export]
macro_rules! vec {
    () => {
        Vec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap()
    };
    ($elem:expr; $n:expr) => {
        {
            let mut v = Vec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap();
            for _ in 0..$n {
                v.push($elem).unwrap();
            }
            v
        }
    };
    ($($x:expr),*) => {
        {
            let mut v = Vec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap();
            $(v.push($x).unwrap();)*
            v
        }
    };
}

// Simple format! implementation for no_std mode using a fixed buffer
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
#[cfg(not(feature = "std"))]
pub fn option_value_as_i32(value: &Option<wrt_foundation::Value>) -> Option<i32> {
    match value {
        Some(wrt_foundation::Value::I32(val)) => Some(*val),
        _ => None,
    }
}

#[cfg(not(feature = "std"))]
pub fn option_value_as_i64(value: &Option<wrt_foundation::Value>) -> Option<i64> {
    match value {
        Some(wrt_foundation::Value::I64(val)) => Some(*val),
        _ => None,
    }
}

#[cfg(not(feature = "std"))]
pub fn option_value_as_f32(value: &Option<wrt_foundation::Value>) -> Option<f32> {
    match value {
        Some(wrt_foundation::Value::F32(val)) => Some(val.to_f32()),
        _ => None,
    }
}

#[cfg(not(feature = "std"))]
pub fn option_value_as_f64(value: &Option<wrt_foundation::Value>) -> Option<f64> {
    match value {
        Some(wrt_foundation::Value::F64(val)) => Some(val.to_f64()),
        _ => None,
    }
}

// Add ToString trait for no_std
#[cfg(not(feature = "std"))]
pub trait ToString {
    fn to_string(&self) -> String;
}

#[cfg(not(feature = "std"))]
impl ToString for &str {
    fn to_string(&self) -> String {
        let mut bounded_string = String::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap();
        // Copy characters up to the capacity limit
        for ch in self.chars().take(256) {
            if bounded_string.push(ch).is_err() {
                break;
            }
        }
        bounded_string
    }
}

#[cfg(not(feature = "std"))]
impl ToString for str {
    fn to_string(&self) -> String {
        let mut bounded_string = String::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap();
        // Copy characters up to the capacity limit
        for ch in self.chars().take(256) {
            if bounded_string.push(ch).is_err() {
                break;
            }
        }
        bounded_string
    }
}

// Arc and Mutex for no_std with alloc
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use alloc::sync::Arc;

// For pure no_std without alloc, use reference wrapper
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
#[derive(Debug, Clone)]
pub struct Arc<T> {
    inner: T,
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl<T> Arc<T> {
    pub fn new(value: T) -> Self {
        Self { inner: value }
    }
    
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

// Type aliases with default memory provider for the runtime
pub type DefaultProvider = wrt_foundation::safe_memory::NoStdProvider<1024>;
pub type Instruction = wrt_foundation::types::Instruction<DefaultProvider>;
pub type FuncType = wrt_foundation::types::FuncType<DefaultProvider>;
pub type RuntimeFuncType = wrt_foundation::types::FuncType<DefaultProvider>;
pub type GlobalType = wrt_foundation::types::GlobalType;
pub type MemoryType = wrt_foundation::types::MemoryType;
pub type TableType = wrt_foundation::types::TableType;
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
pub use crate::memory_adapter::MemoryAdapter;
// Module items specific to wrt-runtime module structure
pub use crate::module::{Data, Element, Export, ExportItem, ExportKind, Import, OtherExport};
// Stackless execution engine components
pub use crate::stackless::{
    StacklessCallbackRegistry, StacklessEngine, StacklessExecutionState, StacklessFrame,
    StacklessStack,
};
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
