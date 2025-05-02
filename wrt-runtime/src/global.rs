//! WebAssembly global value implementation
//!
//! This module provides the implementation for WebAssembly globals.

use wrt_error::{codes, kinds, Error, ErrorCategory, Result};
use wrt_types::types::GlobalType as TypesGlobalType;
use wrt_types::values::Value;

/// Represents a WebAssembly global variable
#[derive(Debug, Clone, PartialEq)]
pub struct Global {
    /// The global type
    ty: TypesGlobalType,
    /// The global value
    value: Value,
}

impl Global {
    /// Create a new global value
    pub fn new(ty: TypesGlobalType, value: Value) -> Self {
        Self { ty, value }
    }

    /// Get global value
    pub fn get(&self) -> &Value {
        &self.value
    }

    /// Set global value
    pub fn set(&mut self, value: &Value) -> Result<()> {
        if !self.ty.mutable {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                kinds::ValidationError("Cannot modify immutable global".to_string()),
            ));
        }

        if !value.matches_type(&self.ty.value_type) {
            return Err(Error::new(
                ErrorCategory::Type,
                codes::TYPE_MISMATCH,
                kinds::ValidationError(format!(
                    "Value type doesn't match global type: {:?} vs {}",
                    value, self.ty.value_type
                )),
            ));
        }

        self.value = value.clone();
        Ok(())
    }

    /// Get the global type
    pub fn global_type(&self) -> &TypesGlobalType {
        &self.ty
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
