//! Prelude module for wrt-runtime
//!
//! This module provides a unified set of imports for both std and no_std
//! environments. It re-exports commonly used types and traits to ensure
//! consistency across all crates in the WRT project and simplify imports in
//! individual modules.

// Core imports for both std and no_std environments
// Re-export from alloc when the alloc feature is enabled and std is not
#[cfg(all(feature = "alloc", not(feature = "std")))]
pub use alloc::{
    boxed::Box,
    collections::{BTreeMap as HashMap, BTreeSet as HashSet},
    format,
    string::{String, ToString},
    sync::Arc,
    vec,
    vec::Vec,
};

// For pure no_std (no alloc), use bounded collections
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub use wrt_foundation::{
    BoundedMap as HashMap,
    BoundedSet as HashSet,
    NoStdProvider,
};

// For pure no_std, we'll rely on explicit BoundedVec usage instead of Vec alias
// to avoid conflicts with other crates' Vec definitions
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub use wrt_foundation::bounded::BoundedString;

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub type String = wrt_foundation::bounded::BoundedString<256, wrt_foundation::safe_memory::NoStdProvider<1024>>;

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub type Vec<T> = wrt_foundation::bounded::BoundedVec<T, 256, wrt_foundation::safe_memory::NoStdProvider<1024>>;

// Helper macro to create BoundedVec with standard parameters
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
#[macro_export]
macro_rules! vec_new {
    () => {
        wrt_foundation::bounded::BoundedVec::<_, 256, wrt_foundation::safe_memory::NoStdProvider<1024>>::new_with_provider(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap()
    };
}

// Helper function to create BoundedVec with capacity (capacity is ignored in bounded collections)
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub fn vec_with_capacity<T>(_capacity: usize) -> wrt_foundation::bounded::BoundedVec<T, 256, wrt_foundation::safe_memory::NoStdProvider<1024>> {
    wrt_foundation::bounded::BoundedVec::new_with_provider(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap()
}

// Simple format! implementation for no_std mode using a fixed buffer
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
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
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub use crate::format;

// Add missing Option methods for Value enum matching
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub trait OptionValueExt {
    fn as_i32(&self) -> Option<i32>;
    fn as_i64(&self) -> Option<i64>;
    fn as_f32(&self) -> Option<f32>;
    fn as_f64(&self) -> Option<f64>;
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl OptionValueExt for Option<wrt_foundation::Value> {
    fn as_i32(&self) -> Option<i32> {
        match self {
            Some(wrt_foundation::Value::I32(val)) => Some(*val),
            _ => None,
        }
    }
    
    fn as_i64(&self) -> Option<i64> {
        match self {
            Some(wrt_foundation::Value::I64(val)) => Some(*val),
            _ => None,
        }
    }
    
    fn as_f32(&self) -> Option<f32> {
        match self {
            Some(wrt_foundation::Value::F32(val)) => Some(val.to_f32()),
            _ => None,
        }
    }
    
    fn as_f64(&self) -> Option<f64> {
        match self {
            Some(wrt_foundation::Value::F64(val)) => Some(val.to_f64()),
            _ => None,
        }
    }
}

// Add ToString trait for no_std
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub trait ToString {
    fn to_string(&self) -> String;
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
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

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
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

// Arc is not available in pure no_std, use a reference wrapper
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

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
#[derive(Debug)]
pub struct Box<T> {
    inner: T,
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl<T> Box<T> {
    pub fn new(value: T) -> Self {
        Self { inner: value }
    }
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl<T> core::ops::Deref for Box<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl<T> core::ops::DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
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

// Re-export from wrt-decoder (aliased to avoid name clashes)
// Component module is temporarily disabled in wrt-decoder
// #[cfg(feature = "alloc")]
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
#[cfg(feature = "alloc")]
pub use wrt_format::component::Component as FormatComponent;
#[cfg(feature = "alloc")]
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
#[cfg(feature = "alloc")]
pub use wrt_foundation::component::{ComponentType, ExternType};
// Also export for std feature
#[cfg(feature = "std")]
pub use wrt_foundation::component::{ComponentType, ExternType};
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
        BoundedStack, BoundedVec, FuncType,
        GlobalType as CoreGlobalType, MemoryType as CoreMemoryType, ResourceType,
        SafeMemoryHandler, SafeSlice, TableType as CoreTableType,
        Value, ValueType, VerificationLevel,
    },
    safe_memory::SafeStack,
    types::{Limits, RefValue, ElementSegment, DataSegment},
    MemoryStats,
};

// Type alias for Instruction with default memory provider
pub type Instruction = wrt_foundation::types::Instruction<wrt_foundation::NoStdProvider<1024>>;

// Conditionally import alloc-dependent types
#[cfg(feature = "alloc")]
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
// Synchronization primitives for no_std (if alloc is enabled but not std)
#[cfg(all(feature = "alloc", not(feature = "std")))]
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
