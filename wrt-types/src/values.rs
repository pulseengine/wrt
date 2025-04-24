//! WebAssembly runtime value types.
//!
//! This module provides types for representing WebAssembly runtime values.

use crate::types::ValueType;
use core::fmt;
use wrt_error::{kinds, Error, Result};

#[cfg(feature = "std")]
use std::{boxed::Box, cell::RefCell};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec::Vec,
};

#[cfg(feature = "std")]
use std::thread_local;

#[cfg(not(feature = "std"))]
use core::cell::RefCell;

/// Represents a WebAssembly runtime value
#[derive(Debug, Clone)]
pub enum Value {
    /// 32-bit integer
    I32(i32),
    /// 64-bit integer
    I64(i64),
    /// 32-bit float
    F32(f32),
    /// 64-bit float
    F64(f64),
    /// 128-bit vector
    V128([u8; 16]),
    /// Function reference
    FuncRef(Option<FuncRef>),
    /// External reference
    ExternRef(Option<ExternRef>),
}

// Manual PartialEq implementation for Value
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::I32(a), Value::I32(b)) => a == b,
            (Value::I64(a), Value::I64(b)) => a == b,
            // Handle NaN comparison for floats: NaN != NaN
            (Value::F32(a), Value::F32(b)) => (a.is_nan() && b.is_nan()) || (a == b),
            (Value::F64(a), Value::F64(b)) => (a.is_nan() && b.is_nan()) || (a == b),
            (Value::V128(a), Value::V128(b)) => a == b,
            (Value::FuncRef(a), Value::FuncRef(b)) => a == b,
            (Value::ExternRef(a), Value::ExternRef(b)) => a == b,
            _ => false, // Different types are not equal
        }
    }
}

impl Eq for Value {}

/// A WebAssembly v128 value used for SIMD operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V128 {
    /// The 128-bit value represented as 16 bytes
    pub bytes: [u8; 16],
}

impl V128 {
    /// Create a new v128 value from 16 bytes
    pub fn new(bytes: [u8; 16]) -> Self {
        Self { bytes }
    }

    /// Create a v128 filled with zeros
    pub fn zero() -> Self {
        Self { bytes: [0; 16] }
    }
}

// Create a helper function for creating a v128 value
/// Helper function to create a new V128 value
pub fn v128(bytes: [u8; 16]) -> V128 {
    V128::new(bytes)
}

/// Function reference type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncRef {
    /// Function index
    pub index: u32,
}

/// External reference type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternRef {
    /// Reference index
    pub index: u32,
}

impl Value {
    /// Creates a default value for the given WebAssembly value type.
    ///
    /// This function returns a zero value for numeric types and None for reference types.
    #[must_use]
    pub const fn default_for_type(ty: &ValueType) -> Self {
        match ty {
            ValueType::I32 => Value::I32(0),
            ValueType::I64 => Value::I64(0),
            ValueType::F32 => Value::F32(0.0),
            ValueType::F64 => Value::F64(0.0),
            ValueType::V128 => Value::V128([0; 16]),
            ValueType::FuncRef => Value::FuncRef(None), // Default for FuncRef is null
            ValueType::ExternRef => Value::ExternRef(None), // Default for ExternRef is null
        }
    }

    /// Returns the WebAssembly type of this value
    #[must_use]
    pub const fn type_(&self) -> ValueType {
        match self {
            Self::I32(_) => ValueType::I32,
            Self::I64(_) => ValueType::I64,
            Self::F32(_) => ValueType::F32,
            Self::F64(_) => ValueType::F64,
            Self::FuncRef(_) => ValueType::FuncRef,
            Self::ExternRef(_) => ValueType::ExternRef,
            Self::V128(_) => ValueType::V128,
        }
    }

    /// Checks if this Value matches the specified `ValueType`
    ///
    /// # Returns
    ///
    /// `true` if the value matches the type, `false` otherwise
    #[must_use]
    pub const fn matches_type(&self, ty: &ValueType) -> bool {
        matches!(
            (self, ty),
            (Self::I32(_), ValueType::I32)
                | (Self::I64(_), ValueType::I64)
                | (Self::F32(_), ValueType::F32)
                | (Self::F64(_), ValueType::F64)
                | (Self::FuncRef(_), ValueType::FuncRef)
                | (Self::ExternRef(_), ValueType::ExternRef)
                | (Self::V128(_), ValueType::V128)
        )
    }

    /// Attempts to extract an i32 value if this Value is an I32.
    #[must_use]
    pub const fn as_i32(&self) -> Option<i32> {
        match self {
            Self::I32(v) => Some(*v),
            _ => None,
        }
    }

    /// Attempts to extract an i64 value if this Value is an I64.
    #[must_use]
    pub const fn as_i64(&self) -> Option<i64> {
        match self {
            Self::I64(v) => Some(*v),
            _ => None,
        }
    }

    /// Attempts to extract an f32 value if this Value is an F32.
    #[must_use]
    pub const fn as_f32(&self) -> Option<f32> {
        match self {
            Self::F32(v) => Some(*v),
            _ => None,
        }
    }

    /// Attempts to extract an f64 value if this Value is an F64.
    #[must_use]
    pub const fn as_f64(&self) -> Option<f64> {
        match self {
            Self::F64(v) => Some(*v),
            _ => None,
        }
    }

    /// Attempts to extract a function reference if this Value is a FuncRef.
    #[must_use]
    pub const fn as_func_ref(&self) -> Option<Option<u32>> {
        match self {
            Self::FuncRef(Some(func_ref)) => Some(Some(func_ref.index)),
            Self::FuncRef(None) => Some(None),
            _ => None,
        }
    }

    /// Attempts to extract an external reference if this Value is an ExternRef.
    #[must_use]
    pub const fn as_extern_ref(&self) -> Option<Option<u32>> {
        match self {
            Self::ExternRef(Some(extern_ref)) => Some(Some(extern_ref.index)),
            Self::ExternRef(None) => Some(None),
            _ => None,
        }
    }

    /// Convenience method to get the type of a value
    #[must_use]
    pub const fn value_type(&self) -> ValueType {
        self.type_()
    }

    /// Attempts to extract a u32 value if this Value is an I32.
    #[must_use]
    pub const fn as_u32(&self) -> Option<u32> {
        match self {
            Self::I32(v) => Some(*v as u32),
            _ => None,
        }
    }

    /// Convert to i32, returning an error if this is not an I32 value
    pub fn into_i32(self) -> Result<i32> {
        match self {
            Self::I32(v) => Ok(v),
            _ => Err(Error::new(kinds::InvalidType(format!(
                "Expected I32, got {:?}",
                self.type_()
            )))),
        }
    }

    /// Attempts to extract a bool value if this Value is an I32 with value 0 or 1.
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::I32(0) => Some(false),
            Self::I32(1) => Some(true),
            _ => None,
        }
    }

    /// Attempts to extract a v128 value if this Value is a V128.
    pub fn as_v128(&self) -> Result<[u8; 16]> {
        match self {
            Self::V128(v) => Ok(*v),
            _ => Err(Error::new(kinds::InvalidType(format!(
                "Expected V128, got {:?}",
                self.type_()
            )))),
        }
    }

    /// Convert from F32 to I32, returning an error if this is not an F32 value
    pub fn into_i32_from_f32(self) -> Result<i32> {
        match self {
            Self::F32(v) => Ok(v as i32),
            _ => Err(Error::new(kinds::InvalidType(format!(
                "Expected F32, got {:?}",
                self.type_()
            )))),
        }
    }

    /// Convert from F64 to I64, returning an error if this is not an F64 value
    pub fn into_i64_from_f64(self) -> Result<i64> {
        match self {
            Self::F64(v) => Ok(v as i64),
            _ => Err(Error::new(kinds::InvalidType(format!(
                "Expected F64, got {:?}",
                self.type_()
            )))),
        }
    }

    /// Creates a FuncRef value with the given function index
    ///
    /// # Arguments
    ///
    /// * `func_idx` - The function index to reference, or None for a null reference
    ///
    /// # Returns
    ///
    /// A new FuncRef value
    pub fn func_ref(func_idx: Option<u32>) -> Self {
        match func_idx {
            Some(idx) => Self::FuncRef(Some(FuncRef { index: idx })),
            None => Self::FuncRef(None),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::I32(v) => write!(f, "i32({})", v),
            Self::I64(v) => write!(f, "i64({})", v),
            Self::F32(v) => write!(f, "f32({})", v),
            Self::F64(v) => write!(f, "f64({})", v),
            Self::V128(v) => write!(f, "v128({:?})", v),
            Self::FuncRef(Some(func_ref)) => write!(f, "funcref({})", func_ref.index),
            Self::FuncRef(None) => write!(f, "funcref(null)"),
            Self::ExternRef(Some(extern_ref)) => write!(f, "externref({})", extern_ref.index),
            Self::ExternRef(None) => write!(f, "externref(null)"),
        }
    }
}

// Add AsRef<[u8]> implementation for Value to work with BoundedVec
impl AsRef<[u8]> for Value {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::I32(v) => {
                match *v {
                    // Common values as static byte arrays to avoid allocation
                    0 => &[0, 0, 0, 0],
                    1 => &[1, 0, 0, 0],
                    -1 => &[255, 255, 255, 255],
                    // Use thread-local storage for dynamic values
                    #[cfg(feature = "std")]
                    _ => {
                        // Using thread_local for thread safety
                        thread_local! {
                            static BYTES: RefCell<[u8; 4]> = const { RefCell::new([0; 4]) };
                        }

                        BYTES.with(|cell| {
                            let mut bytes = cell.borrow_mut();
                            *bytes = v.to_le_bytes();
                            // Leak a copy of the bytes with a 'static lifetime
                            let leaked: &'static [u8] = Box::leak(Box::new(*bytes));
                            leaked
                        })
                    }
                    #[cfg(not(feature = "std"))]
                    _ => {
                        // For no_std, just return the bytes directly
                        // This is not thread-safe but works for single-threaded environments
                        static mut BYTES: [u8; 4] = [0; 4];
                        unsafe {
                            BYTES = v.to_le_bytes();
                            &BYTES
                        }
                    }
                }
            }
            Self::I64(v) => match *v {
                0 => &[0, 0, 0, 0, 0, 0, 0, 0],
                1 => &[1, 0, 0, 0, 0, 0, 0, 0],
                -1 => &[255, 255, 255, 255, 255, 255, 255, 255],
                #[cfg(feature = "std")]
                _ => {
                    thread_local! {
                        static BYTES: RefCell<[u8; 8]> = const { RefCell::new([0; 8]) };
                    }

                    BYTES.with(|cell| {
                        let mut bytes = cell.borrow_mut();
                        *bytes = v.to_le_bytes();
                        let leaked: &'static [u8] = Box::leak(Box::new(*bytes));
                        leaked
                    })
                }
                #[cfg(not(feature = "std"))]
                _ => {
                    static mut BYTES: [u8; 8] = [0; 8];
                    unsafe {
                        BYTES = v.to_le_bytes();
                        &BYTES
                    }
                }
            },
            Self::F32(v) => {
                if *v == 0.0 {
                    &[0, 0, 0, 0]
                } else {
                    #[cfg(feature = "std")]
                    {
                        thread_local! {
                            static BYTES: RefCell<[u8; 4]> = const { RefCell::new([0; 4]) };
                        }

                        BYTES.with(|cell| {
                            let mut bytes = cell.borrow_mut();
                            *bytes = v.to_le_bytes();
                            let leaked: &'static [u8] = Box::leak(Box::new(*bytes));
                            leaked
                        })
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        static mut BYTES: [u8; 4] = [0; 4];
                        unsafe {
                            BYTES = v.to_le_bytes();
                            &BYTES
                        }
                    }
                }
            }
            Self::F64(v) => {
                if *v == 0.0 {
                    &[0, 0, 0, 0, 0, 0, 0, 0]
                } else {
                    #[cfg(feature = "std")]
                    {
                        thread_local! {
                            static BYTES: RefCell<[u8; 8]> = const { RefCell::new([0; 8]) };
                        }

                        BYTES.with(|cell| {
                            let mut bytes = cell.borrow_mut();
                            *bytes = v.to_le_bytes();
                            let leaked: &'static [u8] = Box::leak(Box::new(*bytes));
                            leaked
                        })
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        static mut BYTES: [u8; 8] = [0; 8];
                        unsafe {
                            BYTES = v.to_le_bytes();
                            &BYTES
                        }
                    }
                }
            }
            Self::V128(v) => v.as_ref(),
            Self::FuncRef(func_ref) => {
                if let Some(func) = func_ref {
                    #[cfg(feature = "std")]
                    {
                        thread_local! {
                            static BYTES: RefCell<[u8; 4]> = const { RefCell::new([0; 4]) };
                        }

                        BYTES.with(|cell| {
                            let mut bytes = cell.borrow_mut();
                            *bytes = func.index.to_le_bytes();
                            let leaked: &'static [u8] = Box::leak(Box::new(*bytes));
                            leaked
                        })
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        static mut BYTES: [u8; 4] = [0; 4];
                        unsafe {
                            BYTES = func.index.to_le_bytes();
                            &BYTES
                        }
                    }
                } else {
                    &[0, 0, 0, 0]
                }
            }
            Self::ExternRef(extern_ref) => {
                if let Some(ext) = extern_ref {
                    #[cfg(feature = "std")]
                    {
                        thread_local! {
                            static BYTES: RefCell<[u8; 4]> = const { RefCell::new([0; 4]) };
                        }

                        BYTES.with(|cell| {
                            let mut bytes = cell.borrow_mut();
                            *bytes = ext.index.to_le_bytes();
                            let leaked: &'static [u8] = Box::leak(Box::new(*bytes));
                            leaked
                        })
                    }
                    #[cfg(not(feature = "std"))]
                    {
                        static mut BYTES: [u8; 4] = [0; 4];
                        unsafe {
                            BYTES = ext.index.to_le_bytes();
                            &BYTES
                        }
                    }
                } else {
                    &[0, 0, 0, 0]
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_creation_and_type() {
        let i32_val = Value::I32(42);
        let i64_val = Value::I64(42);
        let f32_val = Value::F32(3.14);
        let f64_val = Value::F64(3.14);
        let v128_val = Value::V128([0; 16]);
        let funcref_val = Value::FuncRef(Some(FuncRef { index: 1 }));
        let externref_val = Value::ExternRef(Some(ExternRef { index: 1 }));

        assert_eq!(i32_val.type_(), ValueType::I32);
        assert_eq!(i64_val.type_(), ValueType::I64);
        assert_eq!(f32_val.type_(), ValueType::F32);
        assert_eq!(f64_val.type_(), ValueType::F64);
        assert_eq!(v128_val.type_(), ValueType::V128);
        assert_eq!(funcref_val.type_(), ValueType::FuncRef);
        assert_eq!(externref_val.type_(), ValueType::ExternRef);
    }

    #[test]
    fn test_value_default_creation() {
        let i32_default = Value::default_for_type(&ValueType::I32);
        let i64_default = Value::default_for_type(&ValueType::I64);
        let f32_default = Value::default_for_type(&ValueType::F32);
        let f64_default = Value::default_for_type(&ValueType::F64);
        let _v128_default = Value::default_for_type(&ValueType::V128);
        let funcref_default = Value::default_for_type(&ValueType::FuncRef);
        let externref_default = Value::default_for_type(&ValueType::ExternRef);

        assert_eq!(i32_default.as_i32(), Some(0));
        assert_eq!(i64_default.as_i64(), Some(0));
        assert_eq!(f32_default.as_f32(), Some(0.0));
        assert_eq!(f64_default.as_f64(), Some(0.0));
        assert_eq!(funcref_default.as_func_ref(), Some(None));
        assert_eq!(externref_default.as_extern_ref(), Some(None));
    }

    #[test]
    fn test_value_type_matching() {
        let i32_val = Value::I32(42);

        assert!(i32_val.matches_type(&ValueType::I32));
        assert!(!i32_val.matches_type(&ValueType::I64));
        assert!(!i32_val.matches_type(&ValueType::F32));
        assert!(!i32_val.matches_type(&ValueType::F64));
        assert!(!i32_val.matches_type(&ValueType::V128));
        assert!(!i32_val.matches_type(&ValueType::FuncRef));
        assert!(!i32_val.matches_type(&ValueType::ExternRef));
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
        assert_eq!(i32_val.as_u32(), Some(42));

        assert_eq!(i64_val.as_i64(), Some(42));
        assert_eq!(i64_val.as_i32(), None);

        assert_eq!(f32_val.as_f32(), Some(3.14));
        assert_eq!(f32_val.as_f64(), None);

        assert_eq!(f64_val.as_f64(), Some(3.14));
        assert_eq!(f64_val.as_f32(), None);
    }

    #[test]
    fn test_reference_value_extraction() {
        let funcref_val = Value::FuncRef(Some(FuncRef { index: 1 }));
        let null_funcref_val = Value::FuncRef(None);
        let externref_val = Value::ExternRef(Some(ExternRef { index: 1 }));
        let null_externref_val = Value::ExternRef(None);

        assert_eq!(funcref_val.as_func_ref(), Some(Some(1)));
        assert_eq!(null_funcref_val.as_func_ref(), Some(None));
        assert_eq!(funcref_val.as_extern_ref(), None);

        assert_eq!(externref_val.as_extern_ref(), Some(Some(1)));
        assert_eq!(null_externref_val.as_extern_ref(), Some(None));
        assert_eq!(externref_val.as_func_ref(), None);
    }

    #[test]
    fn test_value_display() {
        let i32_val = Value::I32(42);
        let i64_val = Value::I64(42);
        let f32_val = Value::F32(3.14);
        let f64_val = Value::F64(3.14);
        let _v128_val = Value::V128([0; 16]);
        let funcref_val = Value::FuncRef(Some(FuncRef { index: 1 }));
        let null_funcref_val = Value::FuncRef(None);
        let externref_val = Value::ExternRef(Some(ExternRef { index: 1 }));
        let null_externref_val = Value::ExternRef(None);

        assert_eq!(i32_val.to_string(), "i32(42)");
        assert_eq!(i64_val.to_string(), "i64(42)");
        assert_eq!(f32_val.to_string(), "f32(3.14)");
        assert_eq!(f64_val.to_string(), "f64(3.14)");
        assert_eq!(funcref_val.to_string(), "funcref(1)");
        assert_eq!(null_funcref_val.to_string(), "funcref(null)");
        assert_eq!(externref_val.to_string(), "externref(1)");
        assert_eq!(null_externref_val.to_string(), "externref(null)");
    }
}
