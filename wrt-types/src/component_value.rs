//! WebAssembly Component Model value types
//!
//! This module defines the runtime value types used in WebAssembly Component Model
//! implementations.

#![allow(clippy::derive_partial_eq_without_eq)]

use core::fmt;
use wrt_error::kinds;
use wrt_error::{Error, Result};

#[cfg(feature = "std")]
use std::{boxed::Box, format, string::String, vec, vec::Vec};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{boxed::Box, format, string::String, string::ToString, vec, vec::Vec};

use crate::Value;

/// A Component Model value type
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ValType {
    /// Boolean value
    #[default]
    Bool,
    /// Signed 8-bit integer
    S8,
    /// Unsigned 8-bit integer
    U8,
    /// Signed 16-bit integer
    S16,
    /// Unsigned 16-bit integer
    U16,
    /// Signed 32-bit integer
    S32,
    /// Unsigned 32-bit integer
    U32,
    /// Signed 64-bit integer
    S64,
    /// Unsigned 64-bit integer
    U64,
    /// 32-bit floating point
    F32,
    /// 64-bit floating point
    F64,
    /// Unicode character
    Char,
    /// UTF-8 string
    String,
    /// Reference to another entity
    Ref(u32),
    /// Record with named fields
    Record(Vec<(String, ValType)>),
    /// Variant with cases
    Variant(Vec<(String, Option<ValType>)>),
    /// List of elements
    List(Box<ValType>),
    /// Fixed-length list of elements with a known length
    FixedList(Box<ValType>, u32),
    /// Tuple of elements
    Tuple(Vec<ValType>),
    /// Flags (set of named boolean flags)
    Flags(Vec<String>),
    /// Enumeration of variants
    Enum(Vec<String>),
    /// Option type
    Option(Box<ValType>),
    /// Result type
    Result(Box<ValType>),
    /// Resource handle (owned)
    Own(u32),
    /// Resource handle (borrowed)
    Borrow(u32),
    /// Void type
    Void,
    /// Error context type
    ErrorContext,
}

/// WebAssembly component value types
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentValue {
    /// Invalid/uninitialized value
    Void,
    /// Boolean value (true/false)
    Bool(bool),
    /// Signed 8-bit integer
    S8(i8),
    /// Unsigned 8-bit integer
    U8(u8),
    /// Signed 16-bit integer
    S16(i16),
    /// Unsigned 16-bit integer
    U16(u16),
    /// Signed 32-bit integer
    S32(i32),
    /// Unsigned 32-bit integer
    U32(u32),
    /// Signed 64-bit integer
    S64(i64),
    /// Unsigned 64-bit integer
    U64(u64),
    /// 32-bit floating point
    F32(f32),
    /// 64-bit floating point
    F64(f64),
    /// Unicode character
    Char(char),
    /// UTF-8 string
    String(String),
    /// List of component values
    List(Vec<ComponentValue>),
    /// Fixed-length list of component values with a known length
    FixedList(Vec<ComponentValue>, u32),
    /// Record with named fields
    Record(Vec<(String, ComponentValue)>),
    /// Variant with case name and optional value
    Variant(String, Option<Box<ComponentValue>>),
    /// Tuple of component values
    Tuple(Vec<ComponentValue>),
    /// Flags with boolean fields
    Flags(Vec<(String, bool)>),
    /// Enumeration with case name
    Enum(String),
    /// Optional value (Some/None)
    Option(Option<Box<ComponentValue>>),
    /// Result value (Ok/Err)
    Result(Result<Box<ComponentValue>, Box<ComponentValue>>),
    /// Handle to a resource (u32 representation)
    Handle(u32),
    /// Reference to a borrowed resource (u32 representation)
    Borrow(u32),
    /// Error context information
    ErrorContext(Vec<ComponentValue>),
}

// Implement Eq for ComponentValue
// Note: This means we can't use floating point equality comparisons directly
impl Eq for ComponentValue {}

impl ComponentValue {
    /// Create a new void value
    pub fn void() -> Self {
        Self::Void
    }

    /// Create a new boolean value
    pub fn bool(v: bool) -> Self {
        Self::Bool(v)
    }

    /// Create a new signed 8-bit integer value
    pub fn s8(v: i8) -> Self {
        Self::S8(v)
    }

    /// Create a new unsigned 8-bit integer value
    pub fn u8(v: u8) -> Self {
        Self::U8(v)
    }

    /// Create a new signed 16-bit integer value
    pub fn s16(v: i16) -> Self {
        Self::S16(v)
    }

    /// Create a new unsigned 16-bit integer value
    pub fn u16(v: u16) -> Self {
        Self::U16(v)
    }

    /// Create a new signed 32-bit integer value
    pub fn s32(v: i32) -> Self {
        Self::S32(v)
    }

    /// Create a new unsigned 32-bit integer value
    pub fn u32(v: u32) -> Self {
        Self::U32(v)
    }

    /// Create a new signed 64-bit integer value
    pub fn s64(v: i64) -> Self {
        Self::S64(v)
    }

    /// Create a new unsigned 64-bit integer value
    pub fn u64(v: u64) -> Self {
        Self::U64(v)
    }

    /// Create a new 32-bit float value
    pub fn f32(v: f32) -> Self {
        Self::F32(v)
    }

    /// Create a new 64-bit float value
    pub fn f64(v: f64) -> Self {
        Self::F64(v)
    }

    /// Create a new character value
    pub fn char(v: char) -> Self {
        Self::Char(v)
    }

    /// Create a new string value
    pub fn string<S: Into<String>>(v: S) -> Self {
        Self::String(v.into())
    }

    /// Create a new list value
    pub fn list(v: Vec<ComponentValue>) -> Self {
        Self::List(v)
    }

    /// Create a new fixed-length list value
    pub fn fixed_list(v: Vec<ComponentValue>, len: u32) -> Result<Self> {
        if v.len() != len as usize {
            return Err(Error::new(kinds::ValidationError(format!(
                "Fixed list length mismatch: expected {}, got {}",
                len,
                v.len()
            ))));
        }
        Ok(Self::FixedList(v, len))
    }

    /// Create a new record value
    pub fn record(v: Vec<(String, ComponentValue)>) -> Self {
        Self::Record(v)
    }

    /// Create a new variant value
    pub fn variant<S: Into<String>>(case: S, value: Option<ComponentValue>) -> Self {
        Self::Variant(case.into(), value.map(Box::new))
    }

    /// Create a new tuple value
    pub fn tuple(v: Vec<ComponentValue>) -> Self {
        Self::Tuple(v)
    }

    /// Create a new flags value
    pub fn flags(v: Vec<(String, bool)>) -> Self {
        Self::Flags(v)
    }

    /// Create a new enum value
    pub fn enum_value<S: Into<String>>(case: S) -> Self {
        Self::Enum(case.into())
    }

    /// Create a new option value (some)
    pub fn some(v: ComponentValue) -> Self {
        Self::Option(Some(Box::new(v)))
    }

    /// Create a new option value (none)
    pub fn none() -> Self {
        Self::Option(None)
    }

    /// Create a new result value (ok)
    pub fn ok(v: ComponentValue) -> Self {
        Self::Result(Ok(Box::new(v)))
    }

    /// Create a new result value (err)
    pub fn err(v: ComponentValue) -> Self {
        Self::Result(Err(Box::new(v)))
    }

    /// Create a new handle value
    pub fn handle(v: u32) -> Self {
        Self::Handle(v)
    }

    /// Create a new borrow value
    pub fn borrow(v: u32) -> Self {
        Self::Borrow(v)
    }

    /// Create a new error context value
    pub fn error_context(v: Vec<ComponentValue>) -> Self {
        Self::ErrorContext(v)
    }

    /// Check if this value is of the void type
    pub fn is_void(&self) -> bool {
        matches!(self, Self::Void)
    }

    /// Get the type of this component value
    pub fn get_type(&self) -> ValType {
        match self {
            Self::Void => ValType::Void,
            Self::Bool(_) => ValType::Bool,
            Self::S8(_) => ValType::S8,
            Self::U8(_) => ValType::U8,
            Self::S16(_) => ValType::S16,
            Self::U16(_) => ValType::U16,
            Self::S32(_) => ValType::S32,
            Self::U32(_) => ValType::U32,
            Self::S64(_) => ValType::S64,
            Self::U64(_) => ValType::U64,
            Self::F32(_) => ValType::F32,
            Self::F64(_) => ValType::F64,
            Self::Char(_) => ValType::Char,
            Self::String(_) => ValType::String,
            Self::List(items) => {
                if let Some(first) = items.first() {
                    ValType::List(Box::new(first.get_type()))
                } else {
                    // Empty list, use a placeholder type
                    ValType::List(Box::new(ValType::Bool))
                }
            }
            Self::FixedList(items, _len) => {
                if let Some(first) = items.first() {
                    ValType::FixedList(Box::new(first.get_type()), *_len)
                } else {
                    // Empty list, use a placeholder type
                    ValType::FixedList(Box::new(ValType::Bool), *_len)
                }
            }
            Self::Record(fields) => {
                let mut field_types = Vec::new();
                for (name, value) in fields {
                    field_types.push((name.clone(), value.get_type()));
                }
                ValType::Record(field_types)
            }
            Self::Variant(case, value) => {
                let cases = vec![(case.clone(), value.as_ref().map(|v| v.get_type()))];
                ValType::Variant(cases)
            }
            Self::Tuple(items) => {
                let item_types = items.iter().map(|v| v.get_type()).collect();
                ValType::Tuple(item_types)
            }
            Self::Flags(flags) => {
                let names = flags.iter().map(|(name, _)| name.clone()).collect();
                ValType::Flags(names)
            }
            Self::Enum(case) => {
                let variants = vec![case.clone()];
                ValType::Enum(variants)
            }
            Self::Option(opt) => {
                if let Some(val) = opt {
                    ValType::Option(Box::new(val.get_type()))
                } else {
                    ValType::Option(Box::new(ValType::Bool)) // Placeholder
                }
            }
            Self::Result(val) => {
                match val {
                    Ok(v) => ValType::Result(Box::new(v.get_type())),
                    Err(_) => ValType::Result(Box::new(ValType::Bool)), // Placeholder for error
                }
            }
            Self::Handle(idx) => ValType::Own(*idx),
            Self::Borrow(idx) => ValType::Borrow(*idx),
            Self::ErrorContext(_) => ValType::ErrorContext,
        }
    }

    /// Check if this value matches the specified type
    pub fn matches_type(&self, value_type: &ValType) -> bool {
        match (self, value_type) {
            // Handle Void type
            (ComponentValue::Void, ValType::Void) => true,
            (ComponentValue::Void, _) => false,

            // Handle ErrorContext type
            (ComponentValue::ErrorContext(_), ValType::ErrorContext) => true,

            // Simple primitive type checks
            (ComponentValue::Bool(_), ValType::Bool) => true,
            (ComponentValue::S8(_), ValType::S8) => true,
            (ComponentValue::U8(_), ValType::U8) => true,
            (ComponentValue::S16(_), ValType::S16) => true,
            (ComponentValue::U16(_), ValType::U16) => true,
            (ComponentValue::S32(_), ValType::S32) => true,
            (ComponentValue::U32(_), ValType::U32) => true,
            (ComponentValue::S64(_), ValType::S64) => true,
            (ComponentValue::U64(_), ValType::U64) => true,
            (ComponentValue::F32(_), ValType::F32) => true,
            (ComponentValue::F64(_), ValType::F64) => true,
            (ComponentValue::Char(_), ValType::Char) => true,
            (ComponentValue::String(_), ValType::String) => true,

            // Complex type checks
            (ComponentValue::List(items), ValType::List(list_type)) => {
                items.iter().all(|item| item.matches_type(list_type))
            }

            // Fixed-length list type check
            (
                ComponentValue::FixedList(items, list_len),
                ValType::FixedList(list_type, expected_len),
            ) => {
                *list_len == *expected_len && items.iter().all(|item| item.matches_type(list_type))
            }

            (ComponentValue::Record(fields), ValType::Record(record_types)) => {
                // Check if all fields in the record type are present in the value
                // and that their types match
                if fields.len() != record_types.len() {
                    return false;
                }

                for (field_name, field_type) in record_types {
                    // Find the field by name in the vector
                    let field_value = fields.iter().find(|(name, _)| name == field_name);
                    if let Some((_, value)) = field_value {
                        if !value.matches_type(field_type) {
                            return false;
                        }
                    } else {
                        return false; // Missing field
                    }
                }

                true
            }

            (ComponentValue::Variant(case, value), ValType::Variant(cases)) => {
                // Check if the case index is valid
                if !cases.iter().any(|(c, _)| c == case) {
                    return false;
                }

                // Get the case type from the index
                let (_, case_type) = cases.iter().find(|(c, _)| c == case).unwrap();

                // Check if the value matches the case type
                match (value, case_type) {
                    (Some(value), Some(ty)) => value.matches_type(ty),
                    (None, None) => true,
                    _ => false,
                }
            }

            (ComponentValue::Tuple(items), ValType::Tuple(item_types)) => {
                // Check if the tuple length matches
                if items.len() != item_types.len() {
                    return false;
                }

                // Check if each item matches its corresponding type
                items
                    .iter()
                    .zip(item_types.iter())
                    .all(|(item, item_type)| item.matches_type(item_type))
            }

            (ComponentValue::Flags(flags), ValType::Flags(flag_names)) => {
                // Check if all flag names in the type are present in the value
                if flags.len() != flag_names.len() {
                    return false;
                }

                // Check that all flag names in the type are present in the value
                flag_names
                    .iter()
                    .all(|name| flags.iter().any(|(fname, _)| fname == name))
            }

            (ComponentValue::Enum(value), ValType::Enum(variants)) => {
                // Check if the enum index is valid
                variants.contains(value)
            }

            (ComponentValue::Option(value), ValType::Option(option_type)) => {
                match value {
                    Some(v) => v.matches_type(option_type),
                    None => true, // None matches any option type
                }
            }

            (ComponentValue::Result(val), ValType::Result(result_type)) => {
                match val {
                    Ok(v) => v.matches_type(result_type),
                    Err(_) => false, // Error doesn't match ok type
                }
            }

            (ComponentValue::Handle(handle), ValType::Own(id)) => handle == id,
            (ComponentValue::Borrow(handle), ValType::Borrow(id)) => handle == id,

            // All other combinations don't match
            _ => false,
        }
    }

    /// Convert a WebAssembly core value to a component value
    pub fn from_core_value(value: &Value) -> Result<Self> {
        match value {
            Value::I32(v) => Ok(ComponentValue::S32(*v)),
            Value::I64(v) => Ok(ComponentValue::S64(*v)),
            Value::F32(v) => Ok(ComponentValue::F32(*v)),
            Value::F64(v) => Ok(ComponentValue::F64(*v)),
            _ => Err(Error::new(kinds::ConversionError(
                "Unsupported value type for conversion to component value".to_string(),
            ))),
        }
    }

    /// Convert this component value to a WebAssembly core value
    pub fn to_core_value(&self) -> Result<Value> {
        match self {
            ComponentValue::Bool(v) => Ok(Value::I32(if *v { 1 } else { 0 })),
            ComponentValue::S8(v) => Ok(Value::I32(*v as i32)),
            ComponentValue::U8(v) => Ok(Value::I32(*v as i32)),
            ComponentValue::S16(v) => Ok(Value::I32(*v as i32)),
            ComponentValue::U16(v) => Ok(Value::I32(*v as i32)),
            ComponentValue::S32(v) => Ok(Value::I32(*v)),
            ComponentValue::U32(v) => Ok(Value::I32(*v as i32)),
            ComponentValue::S64(v) => Ok(Value::I64(*v)),
            ComponentValue::U64(v) => Ok(Value::I64(*v as i64)),
            ComponentValue::F32(v) => Ok(Value::F32(*v)),
            ComponentValue::F64(v) => Ok(Value::F64(*v)),
            _ => Err(Error::new(kinds::ConversionError(
                "Unsupported component value type for conversion to core value".to_string(),
            ))),
        }
    }
}

// Format implementation
impl fmt::Display for ComponentValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentValue::Void => write!(f, "void"),
            ComponentValue::Bool(b) => write!(f, "{}", b),
            ComponentValue::S8(n) => write!(f, "{}i8", n),
            ComponentValue::U8(n) => write!(f, "{}u8", n),
            ComponentValue::S16(n) => write!(f, "{}i16", n),
            ComponentValue::U16(n) => write!(f, "{}u16", n),
            ComponentValue::S32(n) => write!(f, "{}i32", n),
            ComponentValue::U32(n) => write!(f, "{}u32", n),
            ComponentValue::S64(n) => write!(f, "{}i64", n),
            ComponentValue::U64(n) => write!(f, "{}u64", n),
            ComponentValue::F32(n) => write!(f, "{}f32", n),
            ComponentValue::F64(n) => write!(f, "{}f64", n),
            ComponentValue::Char(c) => write!(f, "'{}'", c),
            ComponentValue::String(s) => write!(f, "\"{}\"", s),
            ComponentValue::List(v) => {
                write!(f, "[")?;
                for (i, val) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            ComponentValue::FixedList(v, len) => {
                write!(f, "[{}: ", len)?;
                for (i, val) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            ComponentValue::Record(fields) => {
                write!(f, "{{")?;
                for (i, (name, val)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", name, val)?;
                }
                write!(f, "}}")
            }
            ComponentValue::Variant(case, val) => {
                write!(f, "{}(", case)?;
                if let Some(v) = val {
                    write!(f, "{}", v)?;
                }
                write!(f, ")")
            }
            ComponentValue::Tuple(v) => {
                write!(f, "(")?;
                for (i, val) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, ")")
            }
            ComponentValue::Flags(flags) => {
                write!(f, "{{")?;
                let mut first = true;
                for (name, enabled) in flags {
                    if *enabled {
                        if !first {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", name)?;
                        first = false;
                    }
                }
                write!(f, "}}")
            }
            ComponentValue::Enum(case) => write!(f, "{}", case),
            ComponentValue::Option(opt) => match opt {
                Some(v) => write!(f, "some({})", v),
                None => write!(f, "none"),
            },
            ComponentValue::Result(res) => match res {
                Ok(v) => write!(f, "ok({})", v),
                Err(e) => write!(f, "err({})", e),
            },
            ComponentValue::Handle(h) => write!(f, "handle({})", h),
            ComponentValue::Borrow(b) => write!(f, "borrow({})", b),
            ComponentValue::ErrorContext(ctx) => {
                write!(f, "error_context(")?;
                for (i, val) in ctx.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, ")")
            }
        }
    }
}

/// Basic serialization/deserialization functions for component values
/// These are simple implementations to handle the basic needs for the
/// intercept crate. Full serialization is in the component crate.
///
/// Simple serialization of component values
pub fn serialize_component_values(values: &[ComponentValue]) -> Result<Vec<u8>> {
    let mut result = Vec::new();

    // Write the number of values
    result.extend_from_slice(&(values.len() as u32).to_le_bytes());

    // Write each value (very basic implementation)
    for value in values {
        match value {
            ComponentValue::Bool(b) => {
                result.push(0); // Type tag for bool
                result.push(if *b { 1 } else { 0 });
            }
            ComponentValue::U32(v) => {
                result.push(1); // Type tag for u32
                result.extend_from_slice(&v.to_le_bytes());
            }
            ComponentValue::S32(v) => {
                result.push(2); // Type tag for s32
                result.extend_from_slice(&v.to_le_bytes());
            }
            // Add more types as needed for intercept functionality
            _ => {
                return Err(Error::new(kinds::EncodingError(format!(
                    "Serialization not implemented for this type: {:?}",
                    value
                ))))
            }
        }
    }

    Ok(result)
}

/// Simple deserialization of component values
pub fn deserialize_component_values(data: &[u8], types: &[ValType]) -> Result<Vec<ComponentValue>> {
    let mut result = Vec::new();
    let mut offset = 0;

    // Read the number of values
    if data.len() < 4 {
        return Err(Error::new(kinds::DecodingError(
            "Data too short to contain value count".to_string(),
        )));
    }

    let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    offset += 4;

    // Sanity check
    if count != types.len() {
        return Err(Error::new(kinds::TypeMismatch(format!(
            "Value count mismatch: data has {} values but types list has {}",
            count,
            types.len()
        ))));
    }

    // Read each value
    for _ in 0..count {
        if offset >= data.len() {
            return Err(Error::new(kinds::DecodingError(
                "Unexpected end of data".to_string(),
            )));
        }

        let type_tag = data[offset];
        offset += 1;

        match type_tag {
            0 => {
                // Bool
                if offset >= data.len() {
                    return Err(Error::new(kinds::DecodingError(
                        "Unexpected end of data".to_string(),
                    )));
                }
                let value = data[offset] != 0;
                offset += 1;
                result.push(ComponentValue::Bool(value));
            }
            1 => {
                // U32
                if offset + 4 > data.len() {
                    return Err(Error::new(kinds::DecodingError(
                        "Unexpected end of data".to_string(),
                    )));
                }
                let value = u32::from_le_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]);
                offset += 4;
                result.push(ComponentValue::U32(value));
            }
            2 => {
                // S32
                if offset + 4 > data.len() {
                    return Err(Error::new(kinds::DecodingError(
                        "Unexpected end of data".to_string(),
                    )));
                }
                let value = i32::from_le_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]);
                offset += 4;
                result.push(ComponentValue::S32(value));
            }
            // Add more types as needed for intercept functionality
            _ => {
                return Err(Error::new(kinds::DecodingError(format!(
                    "Deserialization not implemented for type tag: {}",
                    type_tag
                ))))
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_value_type_matching() {
        let bool_value = ComponentValue::Bool(true);
        let int_value = ComponentValue::S32(42);
        let float_value = ComponentValue::F32(3.14);

        assert!(bool_value.matches_type(&ValType::Bool));
        assert!(!bool_value.matches_type(&ValType::S32));

        assert!(int_value.matches_type(&ValType::S32));
        assert!(!int_value.matches_type(&ValType::Bool));

        assert!(float_value.matches_type(&ValType::F32));
        assert!(!float_value.matches_type(&ValType::F64));
    }

    #[test]
    fn test_conversion_between_core_and_component() {
        let i32_val = Value::I32(42);
        let f64_val = Value::F64(3.14);

        let comp_i32 = ComponentValue::from_core_value(&i32_val).unwrap();
        let comp_f64 = ComponentValue::from_core_value(&f64_val).unwrap();

        assert!(matches!(comp_i32, ComponentValue::S32(42)));
        assert!(matches!(comp_f64, ComponentValue::F64(v) if (v - 3.14).abs() < f64::EPSILON));

        let core_i32 = comp_i32.to_core_value().unwrap();
        let core_f64 = comp_f64.to_core_value().unwrap();

        assert_eq!(core_i32, i32_val);
        assert_eq!(core_f64, f64_val);
    }

    #[test]
    fn test_serialization_deserialization() {
        let values = vec![
            ComponentValue::Bool(true),
            ComponentValue::U32(42),
            ComponentValue::S32(-7),
        ];

        let types = vec![ValType::Bool, ValType::U32, ValType::S32];

        let serialized = serialize_component_values(&values).unwrap();
        let deserialized = deserialize_component_values(&serialized, &types).unwrap();

        assert_eq!(deserialized.len(), values.len());

        if let ComponentValue::Bool(v) = &deserialized[0] {
            assert!(*v);
        } else {
            panic!("Expected Bool value");
        }

        if let ComponentValue::U32(v) = &deserialized[1] {
            assert_eq!(*v, 42);
        } else {
            panic!("Expected U32 value");
        }

        if let ComponentValue::S32(v) = &deserialized[2] {
            assert_eq!(*v, -7);
        } else {
            panic!("Expected S32 value");
        }
    }
}
