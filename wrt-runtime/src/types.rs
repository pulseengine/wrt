//! WebAssembly runtime type definitions
//!
//! This module defines the core type structures for WebAssembly runtime.

use crate::prelude::*;

/// Represents a WebAssembly memory type
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryType {
    /// The limits of the memory
    pub limits: Limits,
}

/// Represents a WebAssembly table type
#[derive(Debug, Clone, PartialEq)]
pub struct TableType {
    /// The element type of the table
    pub element_type: ValueType,
    /// The limits of the table
    pub limits: Limits,
}

/// Re-export global type
pub use crate::global::GlobalType;
