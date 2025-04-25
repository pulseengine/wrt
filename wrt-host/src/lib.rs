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

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{boxed::Box, string::String, vec::Vec};

#[cfg(feature = "std")]
use std::{boxed::Box, string::String, vec::Vec};

// Reexports for convenience
pub use wrt_error::{Error, Result};

// Export modules
pub mod callback;
pub mod function;

// Reexport types
pub use callback::CallbackRegistry;
pub use function::{CloneableFn, HostFunctionHandler};

// Include verification module conditionally, but exclude during coverage builds
#[cfg(all(not(coverage), any(doc, kani)))]
pub mod verify;
