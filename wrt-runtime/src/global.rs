//! WebAssembly global value implementation
//!
//! This module provides the implementation for WebAssembly globals.

use wrt_error::kinds;
use wrt_error::{Error, Result};
use wrt_types::values::Value;

/// Represents a WebAssembly global instance
#[derive(Debug, Clone)]
pub struct Global {
    /// The type of the global, including mutability
    pub ty: GlobalType,
    /// The current value of the global
    value: Value,
}

impl Global {
    /// Create a new global with the given type and initial value
    pub fn new(ty: GlobalType, value: Value) -> Self {
        Self { ty, value }
    }

    /// Get the current value of the global
    pub fn get(&self) -> Value {
        self.value.clone()
    }

    /// Set the value of the global if it's mutable
    pub fn set(&mut self, value: Value) -> Result<()> {
        if !self.ty.mutable {
            return Err(Error::new(kinds::ValidationError(
                "Cannot modify immutable global".to_string(),
            )));
        }

        // Ensure the value type matches the global's type
        if !value.matches_type(&self.ty.value_type) {
            return Err(Error::new(kinds::ValidationError(format!(
                "Value type doesn't match global type: {:?} vs {}",
                value, self.ty.value_type
            ))));
        }

        self.value = value;
        Ok(())
    }
}

/// Represents a WebAssembly global type
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalType {
    /// The value type of the global
    pub value_type: wrt_types::types::ValueType,
    /// Whether the global is mutable
    pub mutable: bool,
}

impl GlobalType {
    /// Create a new global type
    pub fn new(value_type: wrt_types::types::ValueType, mutable: bool) -> Self {
        Self {
            value_type,
            mutable,
        }
    }
}
