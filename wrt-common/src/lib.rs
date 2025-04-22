//! Common types for WebAssembly Runtime (WRT)
//!
//! This crate provides shared type definitions used across
//! the WebAssembly Runtime (WRT) crates, particularly to avoid
//! circular dependencies between wrt-types and wrt-format.

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "kani", feature(kani))]

// When no_std but alloc is available
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

pub mod component_type;
pub mod component_value;

/// Re-export module for component value types
pub mod component {
    pub use crate::component_type::*;
    pub use crate::component_value::*;
}
