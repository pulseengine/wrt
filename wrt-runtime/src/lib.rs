//! WebAssembly Runtime (WRT) - Runtime Implementation
//!
//! This crate provides the core runtime types and implementations for WebAssembly,
//! shared between both the core WebAssembly and Component Model implementations.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::missing_panics_doc)]

// Import std when available
#[cfg(feature = "std")]
extern crate std;

// Import alloc for no_std
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Re-export prelude module publicly
pub use prelude::*;

// Core modules
pub mod func;
pub mod global;
pub mod memory;
pub mod memory_helpers;
pub mod prelude;
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
#[cfg(test)]
mod tests;

// Re-export trait definitions
pub use component_traits::{
    ComponentInstance, ComponentRuntime, HostFunction, HostFunctionFactory,
};

// Re-export implementations
pub use component_impl::{ComponentRuntimeImpl, DefaultHostFunctionFactory};

// Re-export prelude for convenience
pub use prelude::*;
