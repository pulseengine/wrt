//! WebAssembly Component Model value types
//!
//! This module defines the runtime value types used in WebAssembly Component Model
//! implementations.

#[cfg(feature = "std")]
use std::{collections::HashMap, string::String, vec::Vec};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{
    boxed::Box,
    collections::BTreeMap as HashMap,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

#[cfg(feature = "std")]
use std::boxed::Box;

use crate::values::Value;
use wrt_error::{kinds, Error, Result};

/// A Component Model value type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValType {
    /// Boolean value
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
}

/// A Component Model value used at runtime
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentValue {
    /// Boolean value
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
    /// List of values
    List(Vec<ComponentValue>),
    /// Record with named fields
    Record(HashMap<String, ComponentValue>),
    /// Variant with case name and optional value
    Variant {
        /// Case/discriminant index
        case: u32,
        /// Optional value associated with this variant case
        value: Option<Box<ComponentValue>>,
    },
    /// Tuple of values
    Tuple(Vec<ComponentValue>),
    /// Flags (set of named boolean flags)
    Flags(HashMap<String, bool>),
    /// Enumeration value
    Enum(u32),
    /// Option type
    Option(Option<Box<ComponentValue>>),
    /// Result type with ok value
    Result(Result<Option<Box<ComponentValue>>, Option<Box<ComponentValue>>>),
    /// Resource handle (owned)
    Own(u32),
    /// Resource handle (borrowed)
    Borrow(u32),
}

// Implement Eq for ComponentValue
// Note: This means we can't use floating point equality comparisons directly
impl Eq for ComponentValue {}

impl ComponentValue {
    /// Get the type of this component value
    pub fn get_type(&self) -> ValType {
        match self {
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
            Self::Record(fields) => {
                let mut field_types = Vec::new();
                for (name, value) in fields {
                    field_types.push((name.clone(), value.get_type()));
                }
                ValType::Record(field_types)
            }
            Self::Variant { case, value } => {
                let cases = vec![(case.to_string(), value.as_ref().map(|v| v.get_type()))];
                ValType::Variant(cases)
            }
            Self::Tuple(items) => {
                let item_types = items.iter().map(|v| v.get_type()).collect();
                ValType::Tuple(item_types)
            }
            Self::Flags(flags) => {
                let names = flags.keys().cloned().collect();
                ValType::Flags(names)
            }
            Self::Enum(variant) => {
                let variants = vec![variant.to_string()];
                ValType::Enum(variants)
            }
            Self::Option(opt) => {
                if let Some(val) = opt {
                    ValType::Option(Box::new(val.get_type()))
                } else {
                    ValType::Option(Box::new(ValType::Bool)) // Placeholder
                }
            }
            Self::Result(val) => ValType::Result(Box::new(if let Ok(Some(v)) = val {
                v.get_type()
            } else {
                ValType::Bool // Placeholder for None or Err
            })),
            Self::Own(idx) => ValType::Own(*idx),
            Self::Borrow(idx) => ValType::Borrow(*idx),
        }
    }

    /// Check if this value matches the specified type
    pub fn matches_type(&self, value_type: &ValType) -> bool {
        match (self, value_type) {
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

            (ComponentValue::Record(fields), ValType::Record(record_types)) => {
                // Check if all fields in the record type are present in the value
                // and that their types match
                if fields.len() != record_types.len() {
                    return false;
                }

                for (field_name, field_type) in record_types {
                    if let Some(field_value) = fields.get(field_name) {
                        if !field_value.matches_type(field_type) {
                            return false;
                        }
                    } else {
                        return false; // Missing field
                    }
                }

                true
            }

            (ComponentValue::Variant { case, value }, ValType::Variant(cases)) => {
                // Check if the case index is valid
                if *case as usize >= cases.len() {
                    return false;
                }

                // Get the case type from the index
                let (_, case_type) = &cases[*case as usize];

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

                flag_names.iter().all(|name| flags.contains_key(name))
            }

            (ComponentValue::Enum(value), ValType::Enum(variants)) => {
                // Check if the enum index is valid
                *value < variants.len() as u32
            }

            (ComponentValue::Option(value), ValType::Option(option_type)) => {
                match value {
                    Some(v) => v.matches_type(option_type),
                    None => true, // None matches any option type
                }
            }

            (ComponentValue::Result(val), ValType::Result(result_type)) => {
                match val {
                    Ok(v) => match v {
                        Some(inner) => inner.matches_type(result_type),
                        None => true, // None matches any result type
                    },
                    Err(_) => true, // We don't check error types for now
                }
            }

            (ComponentValue::Own(handle), ValType::Own(id)) => handle == id,
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
