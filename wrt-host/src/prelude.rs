// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Prelude module for wrt-host
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

// For pure no_std (no alloc), use bounded collections
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub use wrt_foundation::{
    bounded::{BoundedVec as Vec, BoundedString as String},
    BoundedMap as HashMap,
    BoundedSet as HashSet,
};

// Additional imports for pure no_std
#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
pub use core::fmt::Write as FmtWrite;

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
}

#[cfg(all(not(feature = "std"), not(feature = "alloc")))]
impl<T> core::ops::Deref for Arc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// In pure no_std mode, we need a minimal Box implementation for trait objects
// Simple Box implementation for no_std environments
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

// Drop and Debug are automatically derived for our simple Box implementation
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
// Re-export from wrt-foundation
pub use wrt_foundation::{
    // Builtin types
    builtin::BuiltinType,
    // SafeMemory types
    safe_memory::{SafeMemoryHandler, SafeSlice, SafeStack},
    // Common types
    types::{BlockType, FuncType, GlobalType, MemoryType, TableType, ValueType},
    values::Value,
    // Verification types
    verification::VerificationLevel,
};

// Component model types (only available with alloc)
#[cfg(any(feature = "std", feature = "alloc"))]
pub use wrt_foundation::component_value::ComponentValue;
// Re-export from wrt-intercept (only available with alloc)
#[cfg(any(feature = "std", feature = "alloc"))]
pub use wrt_intercept::{
    BeforeBuiltinResult, BuiltinInterceptor, InterceptContext, LinkInterceptor,
    LinkInterceptorStrategy,
};
// Import synchronization primitives for no_std
#[cfg(not(feature = "std"))]
pub use wrt_sync::{Mutex, RwLock};

// Re-export from this crate
pub use crate::{
    builder::HostBuilder,
    callback::{CallbackRegistry, CallbackType},
    function::{CloneableFn, HostFunctionHandler},
    host::BuiltinHost,
};
