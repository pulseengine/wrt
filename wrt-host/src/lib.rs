//! Host function infrastructure for the WebAssembly Runtime (WRT).
//!
//! This crate provides the core infrastructure for registering and managing host functions
//! that can be called from WebAssembly components. It follows the Component Model specification
//! for host functions and the Canonical ABI.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::missing_panics_doc)]

// When no_std but alloc is available
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Re-exports for alloc types
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use alloc::{
    boxed::Box, collections::BTreeMap as HashMap, collections::BTreeSet as HashSet, fmt, format,
    string::String, string::ToString, sync::Arc, vec, vec::Vec,
};

// Re-exports for std types
#[cfg(feature = "std")]
pub use std::{
    boxed::Box, collections::HashMap, collections::HashSet, fmt, format, string::String,
    string::ToString, sync::Arc, vec, vec::Vec,
};

// Reexports for convenience
pub use crate::callback::CallbackType;
pub use wrt_error::{kinds, Error, Result};
pub use wrt_types::{Value, ValueType};

// Export modules
pub mod builder;
pub mod callback;
pub mod function;
pub mod host;

// Reexport types
pub use builder::HostBuilder;
pub use callback::CallbackRegistry;
pub use function::{CloneableFn, HostFunctionHandler};
pub use host::BuiltinHost;

// Include verification module conditionally, but exclude during coverage builds
#[cfg(all(not(coverage), any(doc, kani)))]
pub mod verify;
