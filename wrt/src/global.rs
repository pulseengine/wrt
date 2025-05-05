//! Module for WebAssembly global instances
//!
//! This module provides re-exports and adapters for wrt-runtime Global

// Re-export Global and GlobalType from wrt-runtime
use crate::error::kinds;
use crate::prelude::TypesValue as Value;
use wrt_error::{Error, Result};
pub use wrt_runtime::{Global, GlobalType};
use wrt_types::types::ValueType;

/// Utility function to create a new global instance
pub fn new_global(ty: GlobalType, value: Value) -> Result<Global> {
    // Check that the value matches the global type
    if !value.matches_type(&ty.value_type) {
        return Err(Error::new(kinds::ExecutionError(format!(
            "Value type {:?} does not match global type {:?}",
            value.type_(),
            ty.value_type
        ))));
    }

    // Convert Value to wrt_types::values::Value
    let runtime_value = value.into();

    // Create a new Global
    Ok(Global::new(ty, runtime_value))
}

/// Create a new global with i32 type
pub fn new_i32_global(value: i32, mutable: bool) -> Global {
    let ty = GlobalType::new(ValueType::I32, mutable);
    Global::new(ty, wrt_types::values::Value::I32(value))
}

/// Create a new global with i64 type
pub fn new_i64_global(value: i64, mutable: bool) -> Global {
    let ty = GlobalType::new(ValueType::I64, mutable);
    Global::new(ty, wrt_types::values::Value::I64(value))
}

/// Create a new global with f32 type
pub fn new_f32_global(value: f32, mutable: bool) -> Global {
    let ty = GlobalType::new(ValueType::F32, mutable);
    Global::new(ty, wrt_types::values::Value::F32(value))
}

/// Create a new global with f64 type
pub fn new_f64_global(value: f64, mutable: bool) -> Global {
    let ty = GlobalType::new(ValueType::F64, mutable);
    Global::new(ty, wrt_types::values::Value::F64(value))
}
