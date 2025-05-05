//! Prelude module for wrt-instructions
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
    sync::Arc,
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

// Import synchronization primitives for both std and no_std
pub use wrt_sync::{
    WrtMutex as Mutex, WrtMutexGuard as MutexGuard, WrtRwLock as RwLock,
    WrtRwLockReadGuard as RwLockReadGuard, WrtRwLockWriteGuard as RwLockWriteGuard,
};

// Re-export from wrt-error
pub use wrt_error::{codes, kinds, Error, ErrorCategory, Result};

// Re-export from wrt-types
pub use wrt_types::{
    bounded::{BoundedStack, BoundedVec},
    // SafeMemory types
    safe_memory::{SafeMemoryHandler, SafeSlice, SafeStack},
    // Common types
    types::{BlockType, FuncType, GlobalType, MemoryType, RefType, TableType, ValueType},
    values::Value,
    // Verification types
    verification::VerificationLevel,
    // Result type
    Result as TypesResult,
};

// Re-export instruction specific types
pub use crate::{
    arithmetic_ops::ArithmeticOp,
    comparison_ops::ComparisonOp,
    control_ops::ControlOp,
    conversion_ops::ConversionOp,
    instruction_traits::PureInstruction,
    memory_ops::{MemoryLoad, MemoryStore},
    table_ops::TableOp,
    variable_ops::VariableOp,
};
