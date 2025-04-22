//! Module for WebAssembly values.
//!
//! This module provides definitions for WebAssembly runtime values.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

// Re-export appropriate types from wrt-types
pub use wrt_types::types::ValueType;
pub use wrt_types::values::{ExternRef, FuncRef, Value, V128};

// Import std when available
#[cfg(feature = "std")]
use std::fmt;

// Import alloc for no_std
#[cfg(not(feature = "std"))]
use core::fmt;

// Convenience re-exports for error types
pub use crate::error::{Error, Result};

use crate::error::kinds;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

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
        let func_ref_val = Value::FuncRef(Some(FuncRef::new(0)));
        let extern_ref_val = Value::ExternRef(Some(ExternRef::new(0)));

        assert_eq!(i32_val.ty(), ValueType::I32);
        assert_eq!(i64_val.ty(), ValueType::I64);
        assert_eq!(f32_val.ty(), ValueType::F32);
        assert_eq!(f64_val.ty(), ValueType::F64);
        assert_eq!(v128_val.ty(), ValueType::V128);
        assert_eq!(func_ref_val.ty(), ValueType::FuncRef);
        assert_eq!(extern_ref_val.ty(), ValueType::ExternRef);
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
        let func_ref_val = Value::FuncRef(Some(FuncRef::new(42)));
        let extern_ref_val = Value::ExternRef(Some(ExternRef::new(42)));

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
        let func_ref_val = Value::FuncRef(Some(FuncRef::new(42)));
        let extern_ref_val = Value::ExternRef(Some(ExternRef::new(42)));

        assert_eq!(format!("{}", i32_val), "i32(42)");
        assert_eq!(format!("{}", i64_val), "i64(42)");
        assert!(format!("{}", f32_val).starts_with("f32(3.14"));
        assert!(format!("{}", f64_val).starts_with("f64(3.14"));
        assert!(format!("{}", func_ref_val).contains("func_ref"));
        assert!(format!("{}", extern_ref_val).contains("extern_ref"));
    }
}
