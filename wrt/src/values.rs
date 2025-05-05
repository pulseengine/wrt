//! Module for WebAssembly values.
//!
//! This module provides definitions and utilities for WebAssembly runtime values.
//! It re-exports and extends the Value type from wrt-types.

#![cfg_attr(not(feature = "std"), no_std)]

// Use the prelude to import common types
use crate::prelude::*;
use crate::types::Value;

// Re-export appropriate types from wrt-types (now through prelude)
// pub use wrt_types::types::ValueType;
// pub use wrt_types::values::{ExternRef, FuncRef, Value, V128};

/// This module contains utility functions for working with WebAssembly values
/// and extends the functionality of the Value type from wrt-types.
///
/// Rather than redefining the Value type, we use the one from wrt-types,
/// which implements the standard value types for WebAssembly:
/// - i32: 32-bit integer
/// - i64: 64-bit integer
/// - f32: 32-bit float
/// - f64: 64-bit float
/// - v128: 128-bit vector
/// - Ref: Reference types (funcref, externref)

// Implementation for Vec-based stack when std is available
#[cfg(feature = "std")]
pub type ValueStack = Vec<Value>;

// Implementation for SafeStack-based stack for memory safety
#[cfg(feature = "safety")]
pub type ValueStack = SafeStack<Value>;

/// Create a new standard value stack
#[cfg(feature = "std")]
pub fn new_value_stack() -> ValueStack {
    Vec::new()
}

/// Create a new safety-enhanced value stack
#[cfg(feature = "safety")]
pub fn new_value_stack() -> ValueStack {
    SafeStack::new()
}

/// Create a new value stack with the specified capacity
#[cfg(feature = "std")]
pub fn new_value_stack_with_capacity(capacity: usize) -> ValueStack {
    Vec::with_capacity(capacity)
}

/// Create a new safety-enhanced value stack with the specified capacity
#[cfg(feature = "safety")]
pub fn new_value_stack_with_capacity(capacity: usize) -> ValueStack {
    SafeStack::with_capacity(capacity)
}

/// Create a helper function to create a v128 value
/// Helper function to create a new V128 value
pub fn v128(bytes: [u8; 16]) -> V128 {
    V128::new(bytes)
}

// Add missing convenience functions if needed
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_creation_and_type() {
        let i32_val = Value::I32(42);
        let i64_val = Value::I64(42);
        let f32_val = Value::F32(3.14);
        let f64_val = Value::F64(3.14);
        let v128_val = Value::V128([1; 16]);
        let func_ref_val = Value::FuncRef(Some(FuncRef::from_index(0)));
        let extern_ref_val = Value::ExternRef(Some(ExternRef { index: 0 }));

        assert_eq!(i32_val.type_(), ValueType::I32);
        assert_eq!(i64_val.type_(), ValueType::I64);
        assert_eq!(f32_val.type_(), ValueType::F32);
        assert_eq!(f64_val.type_(), ValueType::F64);
        assert!(func_ref_val.matches_type(&ValueType::FuncRef));
        assert!(extern_ref_val.matches_type(&ValueType::ExternRef));
    }

    #[test]
    fn test_value_default_creation() {
        let i32_default = Value::default_for_type(&ValueType::I32);
        let i64_default = Value::default_for_type(&ValueType::I64);
        let f32_default = Value::default_for_type(&ValueType::F32);
        let f64_default = Value::default_for_type(&ValueType::F64);
        let func_ref_default = Value::default_for_type(&ValueType::FuncRef);
        let extern_ref_default = Value::default_for_type(&ValueType::ExternRef);

        assert_eq!(i32_default.as_i32(), Some(0));
        assert_eq!(i64_default.as_i64(), Some(0));
        assert_eq!(f32_default.as_f32(), Some(0.0));
        assert_eq!(f64_default.as_f64(), Some(0.0));
        assert_eq!(func_ref_default.as_func_ref(), Some(None));
        assert_eq!(extern_ref_default.as_extern_ref(), Some(None));
    }

    #[test]
    fn test_value_type_matching() {
        let i32_val = Value::I32(42);
        assert!(i32_val.matches_type(&ValueType::I32));
        assert!(!i32_val.matches_type(&ValueType::I64));

        let f64_val = Value::F64(3.14);
        assert!(f64_val.matches_type(&ValueType::F64));
        assert!(!f64_val.matches_type(&ValueType::F32));
    }

    #[test]
    fn test_numeric_value_extraction() {
        let i32_val = Value::I32(42);
        let i64_val = Value::I64(42);
        let f32_val = Value::F32(3.14);
        let f64_val = Value::F64(3.14);

        assert_eq!(i32_val.as_i32(), Some(42));
        assert_eq!(i32_val.as_i64(), None);
        assert_eq!(i32_val.as_f32(), None);
        assert_eq!(i32_val.as_f64(), None);

        assert_eq!(i64_val.as_i64(), Some(42));
        assert_eq!(i64_val.as_i32(), None);

        assert!(f32_val.as_f32().unwrap() > 3.13 && f32_val.as_f32().unwrap() < 3.15);
        assert_eq!(f32_val.as_i32(), None);

        assert!(f64_val.as_f64().unwrap() > 3.13 && f64_val.as_f64().unwrap() < 3.15);
        assert_eq!(f64_val.as_i32(), None);
    }

    #[test]
    fn test_reference_value_extraction() {
        let func_ref_val = Value::FuncRef(Some(FuncRef::from_index(42)));
        let extern_ref_val = Value::ExternRef(Some(ExternRef { index: 42 }));

        assert_eq!(func_ref_val.as_func_ref(), Some(Some(42)));
        assert_eq!(func_ref_val.as_extern_ref(), None);
        assert_eq!(func_ref_val.as_i32(), None);

        assert_eq!(extern_ref_val.as_extern_ref(), Some(Some(42)));
        assert_eq!(extern_ref_val.as_func_ref(), None);
        assert_eq!(extern_ref_val.as_i32(), None);

        let null_func_ref = Value::FuncRef(None);
        assert_eq!(null_func_ref.as_func_ref(), Some(None));
    }

    #[test]
    fn test_value_display() {
        let i32_val = Value::I32(42);
        let i64_val = Value::I64(42);
        let f32_val = Value::F32(3.14);
        let f64_val = Value::F64(3.14);
        let func_ref_val = Value::FuncRef(Some(FuncRef::from_index(42)));
        let extern_ref_val = Value::ExternRef(Some(ExternRef { index: 42 }));

        assert_eq!(format!("{}", i32_val), "i32(42)");
        assert_eq!(format!("{}", i64_val), "i64(42)");
        assert!(format!("{}", f32_val).starts_with("f32(3.14"));
        assert!(format!("{}", f64_val).starts_with("f64(3.14"));
        assert!(format!("{}", func_ref_val).contains("func_ref"));
        assert!(format!("{}", extern_ref_val).contains("extern_ref"));
    }
}
