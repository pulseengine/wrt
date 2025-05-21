// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Prelude module for wrt-{CRATE}
//!
//! This module provides a unified set of imports for both std and no_std environments.
//! It re-exports commonly used types and traits to ensure consistency across all crates
//! in the WRT project and simplify imports in individual modules.

// Core imports for both std and no_std environments
pub use core::{
    fmt,
    fmt::Debug,
    fmt::Display,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    convert::{TryFrom, TryInto},
    mem,
    any::Any,
    cmp::{Eq, Ord, PartialEq, PartialOrd},
    slice,
    str,
    // Add any other core imports needed by this specific crate
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
    // Add any other std-specific imports needed by this crate
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
    // Add any other alloc-specific imports needed by this crate
};

// Import synchronization primitives for no_std
#[cfg(not(feature = "std"))]
pub use wrt_sync::{Mutex, RwLock};

// Re-export from wrt-error
pub use wrt_error::{
    codes, 
    Error, 
    ErrorCategory, 
    kinds,
    Result,
};

// Re-export from wrt-types
pub use wrt_types::{
    // SafeMemory types
    safe_memory::{SafeMemoryHandler, SafeSlice, SafeStack},
    // Verification types
    verification::VerificationLevel,
    // Common types
    types::{ValueType, FuncType, BlockType, GlobalType, MemoryType, TableType},
    values::Value,
    // Component model types
    component_value::{ComponentValue, ValType},
    // Add other wrt-types imports specific to this crate
};

// Re-export from wrt-format
#[cfg(feature = "format")]
pub use wrt_format::{
    // Add format-specific imports if this crate uses wrt-format
};

// Re-export from this crate's modules
pub use crate::{
    // Add re-exports specific to this crate's modules
};