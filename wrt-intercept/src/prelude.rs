//! Prelude module for wrt-intercept
//!
//! This module provides a unified set of imports for both std and no_std
//! environments. It re-exports commonly used types and traits to ensure
//! consistency across all crates in the WRT project and simplify imports in
//! individual modules.

// Core imports for both std and no_std environments
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

// Re-export from wrt-error
pub use wrt_error::{codes, kinds, Error, ErrorCategory, Result};
// Re-export from wrt-foundation (for component model)
#[cfg(feature = "alloc")]
pub use wrt_foundation::component_value::ValType;
// Re-export from wrt-foundation
pub use wrt_foundation::{
    builtin::BuiltinType,
    resource::ResourceCanonicalOperation,
    // SafeMemory types
    safe_memory::{SafeMemoryHandler, SafeSlice, SafeStack},
    // Core types
    values::Value,
};

// When no alloc, we need some basic types
#[cfg(not(feature = "alloc"))]
pub use wrt_foundation::bounded::BoundedVec;
#[cfg(not(feature = "alloc"))]
pub use wrt_foundation::BoundedMap as BoundedHashMap;
#[cfg(feature = "alloc")]
pub use wrt_foundation::component_value::ComponentValue;
// Import synchronization primitives for no_std
#[cfg(not(feature = "std"))]
pub use wrt_sync::{Mutex, RwLock};

// Conditional imports
#[cfg(feature = "std")]
pub use crate::strategies::StatisticsStrategy;
// Re-export from this crate
pub use crate::{
    // Builtin interceptors
    builtins::InterceptContext,
    // Strategies
    strategies::{FirewallConfig, FirewallRule, FirewallStrategy, LoggingStrategy},
    InterceptionResult,

    // Core interception types
    LinkInterceptor,
    LinkInterceptorStrategy,
    Modification,
};

// Re-export builtin types when alloc is available
#[cfg(feature = "alloc")]
pub use crate::builtins::{BeforeBuiltinResult, BuiltinInterceptor, BuiltinSerialization};
