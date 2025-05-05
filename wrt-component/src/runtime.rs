//! Runtime types for WebAssembly Component Model.
//!
//! This module provides simplified implementations of runtime types
//! needed for the component model implementation.

use crate::prelude::*;

// Re-export Memory and Table from wrt-runtime
pub use wrt_runtime::{GlobalType, Memory, MemoryType, Table, TableType};

/// WebAssembly function type
#[derive(Debug, Clone, PartialEq)]
pub struct FuncType {
    /// Parameter types
    pub params: Vec<ValueType>,
    /// Result types
    pub results: Vec<ValueType>,
}

/// WebAssembly global instance
#[derive(Debug, Clone)]
pub struct Global {
    /// Global type
    pub ty: GlobalType,
    /// Global value (simplified as u64)
    value: u64,
}

impl Global {
    /// Creates a new global instance
    pub fn new(ty: GlobalType) -> Result<Self> {
        Ok(Self { ty, value: 0 })
    }

    /// Gets the global value
    pub fn get(&self) -> u64 {
        self.value
    }

    /// Sets the global value
    pub fn set(&mut self, value: u64) -> Result<()> {
        if !self.ty.mutable {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                kinds::ValidationError("Cannot modify immutable global".to_string()),
            ));
        }

        self.value = value;
        Ok(())
    }
}
