//! WebAssembly instruction implementations for the WRT runtime.
//!
//! This crate provides implementations of WebAssembly instructions
//! that can be used by various execution engines.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

// When no_std but alloc is available
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Import either from alloc or from std

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{boxed::Box, vec::Vec};

pub mod memory_ops;

// Re-exports for convenience
pub use wrt_error::{Error, Result};
pub use wrt_types::types::ValueType;
pub use wrt_types::values::Value;
