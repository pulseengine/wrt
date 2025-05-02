//! WebAssembly value representations
//!
//! This module provides datatypes for representing WebAssembly values at runtime.

use crate::types::ValueType;
use wrt_error::{codes, Error, ErrorCategory, Result};

#[cfg(feature = "std")]
use std::thread_local;

// Core imports
#[cfg(feature = "std")]
use std::fmt;

#[cfg(not(feature = "std"))]
use core::fmt;

// RefCell for thread local storage
#[cfg(feature = "std")]
use std::cell::RefCell;

#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use core::cell::RefCell;

// Box for dynamic allocation
#[cfg(feature = "std")]
use std::boxed::Box;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::boxed::Box;

// Conditional imports for different environments

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::format;

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
    /// Generic reference to an entity
    Ref(u32),
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
            (Value::Ref(a), Value::Ref(b)) => a == b,
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

impl FuncRef {
    /// Creates a new FuncRef from an index
    #[must_use]
    pub fn from_index(index: u32) -> Self {
        Self { index }
    }
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
            ValueType::FuncRef => Value::FuncRef(None), // Default for FuncRef is null
            ValueType::ExternRef => Value::ExternRef(None), // Default for ExternRef is null
                                                         // Add other variants as needed
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
            Self::V128(_) => ValueType::ExternRef, // Map V128 to ExternRef for now
            Self::Ref(_) => ValueType::ExternRef,  // Map Ref to ExternRef type
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
                | (Self::Ref(_), ValueType::ExternRef)
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
            _ => Err(Error::new(
                ErrorCategory::Type,
                codes::INVALID_TYPE,
                "Expected I32 value",
            )),
        }
    }

    /// Attempts to extract a boolean value
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::I32(v) => Some(*v != 0),
            _ => None,
        }
    }

    /// Attempts to extract an i8 value
    #[must_use]
    pub const fn as_i8(&self) -> Option<i8> {
        match self {
            Self::I32(v) if *v >= i8::MIN as i32 && *v <= i8::MAX as i32 => Some(*v as i8),
            _ => None,
        }
    }

    /// Attempts to extract a u8 value
    #[must_use]
    pub const fn as_u8(&self) -> Option<u8> {
        match self {
            Self::I32(v) if *v >= 0 && *v <= u8::MAX as i32 => Some(*v as u8),
            _ => None,
        }
    }

    /// Attempts to extract an i16 value
    #[must_use]
    pub const fn as_i16(&self) -> Option<i16> {
        match self {
            Self::I32(v) if *v >= i16::MIN as i32 && *v <= i16::MAX as i32 => Some(*v as i16),
            _ => None,
        }
    }

    /// Attempts to extract a u16 value
    #[must_use]
    pub const fn as_u16(&self) -> Option<u16> {
        match self {
            Self::I32(v) if *v >= 0 && *v <= u16::MAX as i32 => Some(*v as u16),
            _ => None,
        }
    }

    /// Attempts to extract a character value
    #[must_use]
    pub fn as_char(&self) -> Option<char> {
        match self {
            Self::I32(v) => char::from_u32(*v as u32),
            _ => None,
        }
    }

    /// Attempts to extract a string value
    #[must_use]
    pub fn as_string(&self) -> Option<&str> {
        None // To be implemented based on actual string representation
    }

    /// Attempts to extract a list of values
    #[must_use]
    pub fn as_list(&self) -> Option<&[Value]> {
        None // To be implemented based on actual list representation
    }

    /// Attempts to extract a record (map of field names to values)
    #[must_use]
    #[cfg(feature = "std")]
    pub fn as_record(&self) -> Option<&std::collections::HashMap<std::string::String, Value>> {
        None // To be implemented based on actual record representation
    }

    #[must_use]
    #[cfg(not(feature = "std"))]
    pub fn as_record(&self) -> Option<&crate::HashMap<crate::String, Value>> {
        None // To be implemented based on actual record representation
    }

    /// Attempts to extract a variant (case and optional value)
    #[must_use]
    pub fn as_variant(&self) -> Option<(u32, Option<&Value>)> {
        None // To be implemented based on actual variant representation
    }

    /// Attempts to extract an enum value (index)
    #[must_use]
    pub const fn as_enum(&self) -> Option<u32> {
        None // To be implemented based on actual enum representation
    }

    /// Attempts to extract an option value
    #[must_use]
    pub fn as_option(&self) -> Option<Option<&Value>> {
        None // To be implemented based on actual option representation
    }

    /// Attempts to extract a result value
    #[must_use]
    pub fn as_result(&self) -> Option<&Result<Option<Box<Value>>, Option<Box<Value>>>> {
        None // To be implemented based on actual result representation
    }

    /// Attempts to extract a tuple of values
    #[must_use]
    pub fn as_tuple(&self) -> Option<&[Value]> {
        None // To be implemented based on actual tuple representation
    }

    /// Attempts to extract flags
    #[must_use]
    #[cfg(feature = "std")]
    pub fn as_flags(&self) -> Option<&std::collections::HashMap<std::string::String, bool>> {
        None // To be implemented based on actual flags representation
    }

    #[must_use]
    #[cfg(not(feature = "std"))]
    pub fn as_flags(&self) -> Option<&crate::HashMap<crate::String, bool>> {
        None // To be implemented based on actual flags representation
    }

    /// Attempts to extract an owned resource handle
    #[must_use]
    pub const fn as_own(&self) -> Option<u32> {
        None // To be implemented based on actual resource representation
    }

    /// Attempts to extract a borrowed resource handle
    #[must_use]
    pub const fn as_borrow(&self) -> Option<u32> {
        None // To be implemented based on actual borrowed resource representation
    }

    /// Attempts to extract a v128 value if this Value is a V128.
    pub fn as_v128(&self) -> Result<[u8; 16]> {
        match self {
            Self::V128(v) => Ok(*v),
            _ => Err(Error::new(
                ErrorCategory::Type,
                codes::INVALID_TYPE,
                "Expected V128 value",
            )),
        }
    }

    /// Convert from F32 to I32, returning an error if this is not an F32 value
    pub fn into_i32_from_f32(self) -> Result<i32> {
        match self {
            Self::F32(v) => Ok(v as i32),
            _ => Err(Error::new(
                ErrorCategory::Type,
                codes::INVALID_TYPE,
                "Expected F32 value",
            )),
        }
    }

    /// Convert from F64 to I64, returning an error if this is not an F64 value
    pub fn into_i64_from_f64(self) -> Result<i64> {
        match self {
            Self::F64(v) => Ok(v as i64),
            _ => Err(Error::new(
                ErrorCategory::Type,
                codes::INVALID_TYPE,
                "Expected F64 value",
            )),
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

    /// Attempts to extract a reference value if this Value is a Ref.
    #[must_use]
    pub const fn as_reference(&self) -> Option<u32> {
        match self {
            Self::Ref(v) => Some(*v),
            _ => None,
        }
    }

    /// Convert to a reference value, returning an error if this is not a Ref value
    pub fn into_ref(self) -> Result<u32> {
        match self {
            Self::Ref(v) => Ok(v),
            _ => Err(Error::new(
                ErrorCategory::Type,
                codes::INVALID_TYPE,
                "Expected Ref value",
            )),
        }
    }

    /// Creates a new Ref value
    #[must_use]
    pub fn reference(ref_idx: u32) -> Self {
        Self::Ref(ref_idx)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::I32(v) => write!(f, "i32:{}", v),
            Value::I64(v) => write!(f, "i64:{}", v),
            Value::F32(v) => write!(f, "f32:{}", v),
            Value::F64(v) => write!(f, "f64:{}", v),
            Value::V128(v) => write!(f, "v128:{:?}", v),
            Value::FuncRef(Some(v)) => write!(f, "funcref:{}", v.index),
            Value::FuncRef(None) => write!(f, "funcref:null"),
            Value::ExternRef(Some(v)) => write!(f, "externref:{}", v.index),
            Value::ExternRef(None) => write!(f, "externref:null"),
            Value::Ref(v) => write!(f, "ref:{}", v),
        }
    }
}

/// AsRef<[u8]> implementation for Value
///
/// This implementation allows a Value to be treated as a byte slice
/// reference. It is primarily used for memory operations.
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
                        // For no_std, use a static buffer
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
                    // Using thread_local for thread safety
                    thread_local! {
                        static BYTES: RefCell<[u8; 8]> = const { RefCell::new([0; 8]) };
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
                    // For no_std, use a static buffer
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
            Self::Ref(ref_idx) => {
                #[cfg(feature = "std")]
                {
                    thread_local! {
                        static BYTES: RefCell<[u8; 4]> = const { RefCell::new([0; 4]) };
                    }

                    BYTES.with(|cell| {
                        let mut bytes = cell.borrow_mut();
                        *bytes = ref_idx.to_le_bytes();
                        let leaked: &'static [u8] = Box::leak(Box::new(*bytes));
                        leaked
                    })
                }
                #[cfg(not(feature = "std"))]
                {
                    static mut BYTES: [u8; 4] = [0; 4];
                    unsafe {
                        BYTES = ref_idx.to_le_bytes();
                        &BYTES
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f32::consts::PI as PI_F32;
    use core::f64::consts::PI as PI_F64;

    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;

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
        assert_eq!(v128_val.type_(), ValueType::ExternRef);
        assert_eq!(funcref_val.type_(), ValueType::FuncRef);
        assert_eq!(externref_val.type_(), ValueType::ExternRef);
    }

    #[test]
    fn test_value_default_creation() {
        let i32_default = Value::default_for_type(&ValueType::I32);
        let i64_default = Value::default_for_type(&ValueType::I64);
        let f32_default = Value::default_for_type(&ValueType::F32);
        let f64_default = Value::default_for_type(&ValueType::F64);
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
        assert!(!i32_val.matches_type(&ValueType::ExternRef));
    }

    #[test]
    fn test_value_conversion() {
        let i32_val = Value::I32(42);
        let i64_val = Value::I64(-7);
        let f32_val = Value::F32(PI_F32);
        let f64_val = Value::F64(PI_F64);

        assert_eq!(i32_val.as_i32(), Some(42));
        assert_eq!(i32_val.as_i64(), None);
        assert_eq!(i32_val.as_f32(), None);
        assert_eq!(i32_val.as_f64(), None);

        assert_eq!(i64_val.as_i32(), None);
        assert_eq!(i64_val.as_i64(), Some(-7));
        assert_eq!(i64_val.as_f32(), None);
        assert_eq!(i64_val.as_f64(), None);

        assert_eq!(f32_val.as_i32(), None);
        assert_eq!(f32_val.as_i64(), None);
        assert_eq!(f32_val.as_f32(), Some(PI_F32));
        assert_eq!(f32_val.as_f64(), None);

        assert_eq!(f64_val.as_i32(), None);
        assert_eq!(f64_val.as_i64(), None);
        assert_eq!(f64_val.as_f32(), None);
        assert_eq!(f64_val.as_f64(), Some(PI_F64));
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

        assert_eq!(i32_val.to_string(), "i32:42");
        assert_eq!(i64_val.to_string(), "i64:42");
        assert_eq!(f32_val.to_string(), "f32:3.14");
        assert_eq!(f64_val.to_string(), "f64:3.14");
        assert_eq!(funcref_val.to_string(), "funcref:1");
        assert_eq!(null_funcref_val.to_string(), "funcref:null");
        assert_eq!(externref_val.to_string(), "externref:1");
        assert_eq!(null_externref_val.to_string(), "externref:null");
    }

    #[test]
    fn test_numeric_value_extraction() {
        let f32_val = Value::F32(PI_F32);
        let f64_val = Value::F64(PI_F64);

        assert_eq!(f32_val.as_f32(), Some(PI_F32));
        assert_eq!(f64_val.as_f64(), Some(PI_F64));
    }

    #[test]
    fn test_value_conversion() {
        let f32_val = Value::F32(PI_F32);
        let f64_val = Value::F64(PI_F64);

        assert_eq!(f32_val.as_f32(), Some(PI_F32));
        assert_eq!(f64_val.as_f64(), Some(PI_F64));
    }
}
