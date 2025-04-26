//! WebAssembly instruction implementations for the WRT runtime.
//!
//! This crate provides implementations of WebAssembly instructions
//! that can be used by various execution engines. It offers pure, stateless
//! implementations of WebAssembly instructions that can be integrated with
//! different execution engines.
//!
//! # Architecture
//!
//! The crate is organized into modules for different instruction categories:
//!
//! - `memory_ops`: Memory operations (load, store)
//! - `arithmetic_ops`: Arithmetic operations (add, subtract, multiply, divide)
//! - `control_ops`: Control flow operations (block, loop, if, branch)
//! - `comparison_ops`: Comparison operations (equality, relational)
//! - `conversion_ops`: Conversion operations (type conversions)
//! - `variable_ops`: Variable access operations (local, global)
//! - `table_ops`: Table operations (get, set, grow)
//!
//! Each module provides pure implementations that can be used with the
//! traits defined in `instruction_traits`.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
#![warn(clippy::missing_panics_doc)]

// When no_std but alloc is available
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Make standard utilities available for either std or alloc
#[cfg(feature = "std")]
pub use std::{boxed::Box, format, string::String, vec, vec::Vec};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use alloc::{boxed::Box, format, string::String, vec, vec::Vec};

pub mod arithmetic_ops;
pub mod comparison_ops;
pub mod control_ops;
pub mod conversion_ops;
pub mod execution;
pub mod instruction_traits;
pub mod memory_ops;
pub mod table_ops;
pub mod variable_ops;

// Test module for arithmetic operations
#[cfg(test)]
mod arithmetic_test;

// Re-exports for convenience
pub use wrt_error::{Error, Result};
pub use wrt_types::types::ValueType;
pub use wrt_types::values::Value;
pub use wrt_types::Result as TypesResult;
