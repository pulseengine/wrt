//! WebAssembly function type implementation
//!
//! This module provides the implementation for WebAssembly function types.

use wrt_error::kinds;
use wrt_error::{Error, Result};
use wrt_types::{types::ValueType, values::Value};

/// Represents a WebAssembly function type
#[derive(Debug, Clone, PartialEq)]
pub struct FuncType {
    /// The parameter types of the function
    pub params: Vec<ValueType>,
    /// The result types of the function
    pub results: Vec<ValueType>,
}

impl FuncType {
    /// Create a new function type
    pub fn new(params: Vec<ValueType>, results: Vec<ValueType>) -> Self {
        Self { params, results }
    }

    /// Validate that the given parameters match this function type
    pub fn validate_params(&self, params: &[Value]) -> Result<()> {
        if params.len() != self.params.len() {
            return Err(Error::new(kinds::ValidationError(format!(
                "Parameter count mismatch: {} vs {}",
                params.len(),
                self.params.len()
            ))));
        }

        for (i, param) in params.iter().enumerate() {
            if !param.matches_type(&self.params[i]) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Parameter type mismatch at position {}: {:?} vs {}",
                    i, param, self.params[i]
                ))));
            }
        }

        Ok(())
    }

    /// Validate that the given results match this function type
    pub fn validate_results(&self, results: &[Value]) -> Result<()> {
        if results.len() != self.results.len() {
            return Err(Error::new(kinds::ValidationError(format!(
                "Result count mismatch: {} vs {}",
                results.len(),
                self.results.len()
            ))));
        }

        for (i, result) in results.iter().enumerate() {
            if !result.matches_type(&self.results[i]) {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Result type mismatch at position {}: {:?} vs {}",
                    i, result, self.results[i]
                ))));
            }
        }

        Ok(())
    }
}
