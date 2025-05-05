//! Prelude module for wrt-runtime
//!
//! This module provides a unified set of imports for both std and no_std environments.
//! It re-exports commonly used types and traits to ensure consistency across all crates
//! in the WRT project and simplify imports in individual modules.

// Core imports for both std and no_std environments
pub use core::{
    any::Any,
    cmp::{Eq, Ord, PartialEq, PartialOrd},
    convert::{TryFrom, TryInto},
    fmt,
    fmt::Debug,
    fmt::Display,
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    slice, str,
};

// Re-export from std when the std feature is enabled
#[cfg(feature = "std")]
pub use std::{
    boxed::Box,
    collections::{HashMap, HashSet},
    format,
    string::{String, ToString},
    sync::{Arc, Mutex, RwLock},
    vec,
    vec::Vec,
};

// Re-export from alloc when no_std but alloc is available
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use alloc::{
    boxed::Box,
    collections::{BTreeMap as HashMap, BTreeSet as HashSet},
    format,
    string::{String, ToString},
    sync::Arc,
    vec,
    vec::Vec,
};

// Import synchronization primitives for no_std
#[cfg(not(feature = "std"))]
pub use wrt_sync::{Mutex, RwLock};

// Re-export from wrt-error
pub use wrt_error::{codes, kinds, Error, ErrorCategory, Result};

// Re-export from wrt-types
pub use wrt_types::{
    bounded::{BoundedStack, BoundedVec},
    // Component model types
    component::{ComponentType, ExternType},
    component_value::{ComponentValue, ValType},
    // Common types
    types::{BlockType, FuncType, GlobalType, Limits, MemoryType, TableType, ValueType},
    values::Value,
    // Verification types
    verification::VerificationLevel,
    // Result type
    Result as TypesResult,
    // SafeMemory types
    SafeMemoryHandler,
    SafeSlice,
    SafeStack,
};

// Re-export from this crate
pub use crate::{
    // Component runtime implementations
    component_impl::{ComponentRuntimeImpl, DefaultHostFunctionFactory},
    component_traits::{ComponentInstance, ComponentRuntime, HostFunction, HostFunctionFactory},
    // Core runtime types
    global::Global,
    memory::Memory,
    memory_helpers::ArcMemoryExt,
    table::Table,
    // Runtime-specific types
    types::{
        GlobalType as RuntimeGlobalType, MemoryType as RuntimeMemoryType,
        TableType as RuntimeTableType,
    },
};
