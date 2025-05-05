//! Prelude module for wrt-format
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
pub use wrt_error::{codes, kinds, Error, ErrorCategory, FromError, Result, ToErrorCategory};

// Re-export from wrt-types
pub use wrt_types::{
    // Component model types
    component_value::{ComponentValue, ValType},
    // Verification types
    verification::VerificationLevel,
    // SafeMemory types
    SafeMemoryHandler,
    SafeSlice,
    SafeStack,
};

// Conditional imports for safety features
#[cfg(feature = "safety")]
pub use wrt_types::safe_memory::{MemoryProvider, StdMemoryProvider};

// No-std memory provider
#[cfg(all(feature = "safety", not(feature = "std")))]
pub use wrt_types::safe_memory::NoStdMemoryProvider;

// Re-export from this crate's modules
pub use crate::{
    // Binary module constants and functions
    binary::{
        read_leb128_u32, read_string, write_leb128_u32, write_string, WASM_MAGIC, WASM_VERSION,
    },
    // Error conversion utilities
    error::{
        parse_error, runtime_error, to_wrt_error, type_error, validation_error, wrt_parse_error,
        wrt_runtime_error, wrt_type_error, wrt_validation_error, IntoError,
    },
    // Section constants
    section::{
        CODE_ID, CUSTOM_ID, DATA_COUNT_ID, DATA_ID, ELEMENT_ID, EXPORT_ID, FUNCTION_ID, GLOBAL_ID,
        IMPORT_ID, MEMORY_ID, START_ID, TABLE_ID, TYPE_ID,
    },
};

// Helper functions for memory safety

/// Create a SafeSlice from a byte slice
#[cfg(feature = "safety")]
pub fn safe_slice(data: &[u8]) -> wrt_types::safe_memory::SafeSlice<'_> {
    wrt_types::safe_memory::SafeSlice::new(data)
}

/// Create a SafeSlice with specific verification level
#[cfg(feature = "safety")]
pub fn safe_slice_with_verification(
    data: &[u8],
    level: wrt_types::verification::VerificationLevel,
) -> wrt_types::safe_memory::SafeSlice<'_> {
    wrt_types::safe_memory::SafeSlice::with_verification_level(data, level)
}

/// Create a memory provider from a byte vector
#[cfg(feature = "safety")]
pub fn memory_provider(data: Vec<u8>) -> wrt_types::safe_memory::StdMemoryProvider {
    wrt_types::safe_memory::StdMemoryProvider::new(data)
}

/// Create a memory provider with specific capacity
#[cfg(feature = "safety")]
pub fn memory_provider_with_capacity(capacity: usize) -> wrt_types::safe_memory::StdMemoryProvider {
    wrt_types::safe_memory::StdMemoryProvider::with_capacity(capacity)
}
