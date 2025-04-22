//! WebAssembly Runtime (WRT) - Runtime Implementation
//!
//! This crate provides the core runtime types and implementations for WebAssembly,
//! shared between both the core WebAssembly and Component Model implementations.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "kani-verifier", feature(kani))]
#![deny(unsafe_code)]
#![warn(missing_docs)]

// Import std when available
#[cfg(feature = "std")]
extern crate std;

// Import alloc for no_std
#[cfg(not(feature = "std"))]
extern crate alloc;

// Core re-exports based on environment
#[cfg(feature = "std")]
pub use std::{boxed::Box, collections::HashMap, string::String, vec::Vec};

#[cfg(not(feature = "std"))]
pub use alloc::{boxed::Box, collections::BTreeMap as HashMap, string::String, vec::Vec};

// Re-export error types for convenience
pub use wrt_error::{Error, Result};

// Core modules
pub mod func;
pub mod global;
pub mod memory;
pub mod memory_helpers;
pub mod table;
pub mod types;

// Re-export commonly used types
pub use func::FuncType;
pub use global::Global;
pub use memory::Memory;
pub use memory_helpers::ArcMemoryExt;
pub use table::Table;
pub use types::{GlobalType, MemoryType, TableType};

/// The WebAssembly memory page size (64KiB)
pub const PAGE_SIZE: usize = 65536;

/// Component Model implementations of runtime interfaces
pub mod component_impl;
/// Component Model trait definitions for runtime interfaces
pub mod component_traits;

// Internal modules
// Tests should be created when needed
// #[cfg(test)]
// mod tests;

// Re-export trait definitions
pub use component_traits::{
    ComponentInstance, ComponentRuntime, HostFunction, HostFunctionFactory,
};

// Re-export implementations
pub use component_impl::{
    ComponentInstanceImpl, ComponentRuntimeImpl, DefaultHostFunctionFactory, HostFunctionImpl,
};
