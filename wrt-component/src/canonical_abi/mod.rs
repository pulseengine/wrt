//! WebAssembly Component Model Canonical ABI
//!
//! This module provides implementations of the Canonical ABI for the
//! WebAssembly Component Model, including lifting, lowering, and memory
//! allocation functions.

pub mod canonical;
pub mod canonical_abi;
pub mod canonical_options;
pub mod canonical_realloc;

pub use canonical::*;
pub use canonical_abi::*;
pub use canonical_options::*;
pub use canonical_realloc::*;