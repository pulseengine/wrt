//! Component Model value handling
//! 
//! This module provides implementations for Component Model value types, including
//! serialization/deserialization, conversion, and runtime representation.

use wrt_error::{kinds, Error, Result};
use wrt_format::component::ValType;
use wrt_types::values::Value;

#[cfg(feature = "std")]
use std::{collections::HashMap, string::String, sync::Arc, vec, vec::Vec};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{
    collections::BTreeMap as HashMap,
    string::{String, ToString},
    sync::Arc,
    vec,
    vec::Vec,
};

/// A Component Model value used at runtime
#[derive(Debug, Clone)]
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
    Variant(String, Option<Box<ComponentValue>>),
    /// Tuple of values
    Tuple(Vec<ComponentValue>),
    /// Flags (set of named boolean flags)
    Flags(HashMap<String, bool>),
    /// Enumeration value
    Enum(String),
    /// Option type
    Option(Option<Box<ComponentValue>>),
    /// Result type with ok value
    Result(Box<ComponentValue>),
    /// Result type with error value
    ResultErr(Box<ComponentValue>),
    /// Result type with both ok and error values
    ResultBoth(Option<Box<ComponentValue>>, Option<Box<ComponentValue>>),
    /// Resource handle (owned)
    Own(u32),
    /// Resource handle (borrowed)
    Borrow(u32),
}

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
                    let element_type = first.get_type();
                    ValType::List(Box::new(element_type))
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
            Self::Variant(case, _) => {
                let cases = vec![(case.clone(), None)];
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
                let variants = vec![variant.clone()];
                ValType::Enum(variants)
            }
            Self::Option(opt) => {
                if let Some(val) = opt {
                    ValType::Option(Box::new(val.get_type()))
                } else {
                    ValType::Option(Box::new(ValType::Bool)) // Placeholder
                }
            }
            Self::Result(val) => ValType::Result(Box::new(val.get_type())),
            Self::ResultErr(err) => ValType::ResultErr(Box::new(err.get_type())),
            Self::ResultBoth(ok, err) => {
                let ok_type = ok
                    .as_ref()
                    .map_or(ValType::Bool, |v| v.get_type());
                let err_type = err
                    .as_ref()
                    .map_or(ValType::Bool, |v| v.get_type());
                ValType::ResultBoth(Box::new(ok_type), Box::new(err_type))
            }
            Self::Own(idx) => ValType::Own(*idx),
            Self::Borrow(idx) => ValType::Borrow(*idx),
        }
    }

    /// Check if this value matches the specified type
    pub fn matches_type(&self, ty: &ValType) -> bool {
        match (self, ty) {
            (Self::Bool(_), ValType::Bool) => true,
            (Self::S8(_), ValType::S8) => true,
            (Self::U8(_), ValType::U8) => true,
            (Self::S16(_), ValType::S16) => true,
            (Self::U16(_), ValType::U16) => true,
            (Self::S32(_), ValType::S32) => true,
            (Self::U32(_), ValType::U32) => true,
            (Self::S64(_), ValType::S64) => true,
            (Self::U64(_), ValType::U64) => true,
            (Self::F32(_), ValType::F32) => true,
            (Self::F64(_), ValType::F64) => true,
            (Self::Char(_), ValType::Char) => true,
            (Self::String(_), ValType::String) => true,
            (Self::List(items), ValType::List(elem_type)) => {
                items.iter().all(|item| item.matches_type(elem_type))
            }
            (Self::Record(fields), ValType::Record(field_types)) => {
                if fields.len() != field_types.len() {
                    return false;
                }
                
                for (name, field_type) in field_types {
                    if let Some(field_value) = fields.get(name) {
                        if !field_value.matches_type(field_type) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            }
            (Self::Variant(case, value), ValType::Variant(cases)) => {
                for (case_name, case_type) in cases {
                    if case == case_name {
                        return match (value, case_type) {
                            (Some(val), Some(ty)) => val.matches_type(ty),
                            (None, None) => true,
                            _ => false,
                        };
                    }
                }
                false
            }
            (Self::Tuple(items), ValType::Tuple(types)) => {
                if items.len() != types.len() {
                    return false;
                }
                items.iter().zip(types.iter())
                    .all(|(item, item_type)| item.matches_type(item_type))
            }
            (Self::Flags(flags), ValType::Flags(names)) => {
                names.iter().all(|name| flags.contains_key(name))
            }
            (Self::Enum(variant), ValType::Enum(variants)) => {
                variants.contains(variant)
            }
            (Self::Option(opt), ValType::Option(inner_type)) => {
                match opt {
                    Some(val) => val.matches_type(inner_type),
                    None => true,
                }
            }
            (Self::Result(val), ValType::Result(ok_type)) => {
                val.matches_type(ok_type)
            }
            (Self::ResultErr(err), ValType::ResultErr(err_type)) => {
                err.matches_type(err_type)
            }
            (Self::ResultBoth(ok, err), ValType::ResultBoth(ok_type, err_type)) => {
                match (ok, err) {
                    (Some(ok_val), _) if ok_val.matches_type(ok_type) => true,
                    (_, Some(err_val)) if err_val.matches_type(err_type) => true,
                    _ => false,
                }
            }
            (Self::Own(idx1), ValType::Own(idx2)) => idx1 == idx2,
            (Self::Borrow(idx1), ValType::Borrow(idx2)) => idx1 == idx2,
            _ => false,
        }
    }
}

/// Convert a component value to binary format
pub fn encode_component_value(value: &ComponentValue, ty: &ValType) -> Result<Vec<u8>> {
    match (value, ty) {
        (ComponentValue::Bool(v), ValType::Bool) => {
            Ok(vec![if *v { 1 } else { 0 }])
        }
        (ComponentValue::S8(v), ValType::S8) => {
            Ok(vec![*v as u8])
        }
        (ComponentValue::U8(v), ValType::U8) => {
            Ok(vec![*v])
        }
        (ComponentValue::S16(v), ValType::S16) => {
            Ok(v.to_le_bytes().to_vec())
        }
        (ComponentValue::U16(v), ValType::U16) => {
            Ok(v.to_le_bytes().to_vec())
        }
        (ComponentValue::S32(v), ValType::S32) => {
            Ok(v.to_le_bytes().to_vec())
        }
        (ComponentValue::U32(v), ValType::U32) => {
            Ok(v.to_le_bytes().to_vec())
        }
        (ComponentValue::S64(v), ValType::S64) => {
            Ok(v.to_le_bytes().to_vec())
        }
        (ComponentValue::U64(v), ValType::U64) => {
            Ok(v.to_le_bytes().to_vec())
        }
        (ComponentValue::F32(v), ValType::F32) => {
            Ok(v.to_le_bytes().to_vec())
        }
        (ComponentValue::F64(v), ValType::F64) => {
            Ok(v.to_le_bytes().to_vec())
        }
        (ComponentValue::Char(v), ValType::Char) => {
            let bytes = v.encode_utf8(&mut [0; 4]);
            let len = bytes.len() as u32;
            let mut result = len.to_le_bytes().to_vec();
            result.extend_from_slice(bytes.as_bytes());
            Ok(result)
        }
        (ComponentValue::String(v), ValType::String) => {
            let bytes = v.as_bytes();
            let len = bytes.len() as u32;
            let mut result = len.to_le_bytes().to_vec();
            result.extend_from_slice(bytes);
            Ok(result)
        }
        // More complex types will need more complete implementations
        _ => Err(Error::new(kinds::NotImplementedError(
            format!("Encoding for {:?} to {:?} not yet implemented", value, ty)
        )))
    }
}

/// Convert binary data to a component value
pub fn decode_component_value(data: &[u8], ty: &ValType) -> Result<ComponentValue> {
    match ty {
        ValType::Bool => {
            if data.len() != 1 {
                return Err(Error::new(kinds::DecodingError(
                    format!("Expected 1 byte for bool, got {}", data.len())
                )));
            }
            Ok(ComponentValue::Bool(data[0] != 0))
        }
        ValType::S8 => {
            if data.len() != 1 {
                return Err(Error::new(kinds::DecodingError(
                    format!("Expected 1 byte for s8, got {}", data.len())
                )));
            }
            Ok(ComponentValue::S8(data[0] as i8))
        }
        ValType::U8 => {
            if data.len() != 1 {
                return Err(Error::new(kinds::DecodingError(
                    format!("Expected 1 byte for u8, got {}", data.len())
                )));
            }
            Ok(ComponentValue::U8(data[0]))
        }
        ValType::S16 => {
            if data.len() != 2 {
                return Err(Error::new(kinds::DecodingError(
                    format!("Expected 2 bytes for s16, got {}", data.len())
                )));
            }
            let mut bytes = [0; 2];
            bytes.copy_from_slice(&data[0..2]);
            Ok(ComponentValue::S16(i16::from_le_bytes(bytes)))
        }
        ValType::U16 => {
            if data.len() != 2 {
                return Err(Error::new(kinds::DecodingError(
                    format!("Expected 2 bytes for u16, got {}", data.len())
                )));
            }
            let mut bytes = [0; 2];
            bytes.copy_from_slice(&data[0..2]);
            Ok(ComponentValue::U16(u16::from_le_bytes(bytes)))
        }
        ValType::S32 => {
            if data.len() != 4 {
                return Err(Error::new(kinds::DecodingError(
                    format!("Expected 4 bytes for s32, got {}", data.len())
                )));
            }
            let mut bytes = [0; 4];
            bytes.copy_from_slice(&data[0..4]);
            Ok(ComponentValue::S32(i32::from_le_bytes(bytes)))
        }
        ValType::U32 => {
            if data.len() != 4 {
                return Err(Error::new(kinds::DecodingError(
                    format!("Expected 4 bytes for u32, got {}", data.len())
                )));
            }
            let mut bytes = [0; 4];
            bytes.copy_from_slice(&data[0..4]);
            Ok(ComponentValue::U32(u32::from_le_bytes(bytes)))
        }
        ValType::S64 => {
            if data.len() != 8 {
                return Err(Error::new(kinds::DecodingError(
                    format!("Expected 8 bytes for s64, got {}", data.len())
                )));
            }
            let mut bytes = [0; 8];
            bytes.copy_from_slice(&data[0..8]);
            Ok(ComponentValue::S64(i64::from_le_bytes(bytes)))
        }
        ValType::U64 => {
            if data.len() != 8 {
                return Err(Error::new(kinds::DecodingError(
                    format!("Expected 8 bytes for u64, got {}", data.len())
                )));
            }
            let mut bytes = [0; 8];
            bytes.copy_from_slice(&data[0..8]);
            Ok(ComponentValue::U64(u64::from_le_bytes(bytes)))
        }
        ValType::F32 => {
            if data.len() != 4 {
                return Err(Error::new(kinds::DecodingError(
                    format!("Expected 4 bytes for f32, got {}", data.len())
                )));
            }
            let mut bytes = [0; 4];
            bytes.copy_from_slice(&data[0..4]);
            Ok(ComponentValue::F32(f32::from_le_bytes(bytes)))
        }
        ValType::F64 => {
            if data.len() != 8 {
                return Err(Error::new(kinds::DecodingError(
                    format!("Expected 8 bytes for f64, got {}", data.len())
                )));
            }
            let mut bytes = [0; 8];
            bytes.copy_from_slice(&data[0..8]);
            Ok(ComponentValue::F64(f64::from_le_bytes(bytes)))
        }
        // More complex types will need more complete implementations
        _ => Err(Error::new(kinds::NotImplementedError(
            format!("Decoding for {:?} not yet implemented", ty)
        )))
    }
}

/// Convert a core WebAssembly value to a Component Model value
pub fn core_to_component_value(value: &Value, ty: &ValType) -> Result<ComponentValue> {
    match (value, ty) {
        (Value::I32(v), ValType::Bool) => {
            Ok(ComponentValue::Bool(*v != 0))
        }
        (Value::I32(v), ValType::S8) => {
            Ok(ComponentValue::S8(*v as i8))
        }
        (Value::I32(v), ValType::U8) => {
            Ok(ComponentValue::U8(*v as u8))
        }
        (Value::I32(v), ValType::S16) => {
            Ok(ComponentValue::S16(*v as i16))
        }
        (Value::I32(v), ValType::U16) => {
            Ok(ComponentValue::U16(*v as u16))
        }
        (Value::I32(v), ValType::S32) => {
            Ok(ComponentValue::S32(*v))
        }
        (Value::I32(v), ValType::U32) => {
            Ok(ComponentValue::U32(*v as u32))
        }
        (Value::I64(v), ValType::S64) => {
            Ok(ComponentValue::S64(*v))
        }
        (Value::I64(v), ValType::U64) => {
            Ok(ComponentValue::U64(*v as u64))
        }
        (Value::F32(v), ValType::F32) => {
            Ok(ComponentValue::F32(*v))
        }
        (Value::F64(v), ValType::F64) => {
            Ok(ComponentValue::F64(*v))
        }
        // String and other complex types require memory access and more context
        _ => Err(Error::new(kinds::ConversionError(
            format!("Cannot convert {:?} to component type {:?}", value, ty)
        )))
    }
}

/// Convert a Component Model value to a core WebAssembly value
pub fn component_to_core_value(value: &ComponentValue) -> Result<Value> {
    match value {
        ComponentValue::Bool(v) => {
            Ok(Value::I32(if *v { 1 } else { 0 }))
        }
        ComponentValue::S8(v) => {
            Ok(Value::I32(*v as i32))
        }
        ComponentValue::U8(v) => {
            Ok(Value::I32(*v as i32))
        }
        ComponentValue::S16(v) => {
            Ok(Value::I32(*v as i32))
        }
        ComponentValue::U16(v) => {
            Ok(Value::I32(*v as i32))
        }
        ComponentValue::S32(v) => {
            Ok(Value::I32(*v))
        }
        ComponentValue::U32(v) => {
            Ok(Value::I32(*v as i32))
        }
        ComponentValue::S64(v) => {
            Ok(Value::I64(*v))
        }
        ComponentValue::U64(v) => {
            Ok(Value::I64(*v as i64))
        }
        ComponentValue::F32(v) => {
            Ok(Value::F32(*v))
        }
        ComponentValue::F64(v) => {
            Ok(Value::F64(*v))
        }
        // String and other complex types cannot be directly represented
        _ => Err(Error::new(kinds::ConversionError(
            format!("Cannot convert component value {:?} to core value", value)
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_value_encoding_decoding() {
        // Test boolean
        let value = ComponentValue::Bool(true);
        let encoded = encode_component_value(&value, &ValType::Bool).unwrap();
        let decoded = decode_component_value(&encoded, &ValType::Bool).unwrap();
        assert!(matches!(decoded, ComponentValue::Bool(true)));

        // Test integers
        let value = ComponentValue::S32(42);
        let encoded = encode_component_value(&value, &ValType::S32).unwrap();
        let decoded = decode_component_value(&encoded, &ValType::S32).unwrap();
        assert!(matches!(decoded, ComponentValue::S32(42)));

        // Test floats
        let value = ComponentValue::F64(3.14159);
        let encoded = encode_component_value(&value, &ValType::F64).unwrap();
        let decoded = decode_component_value(&encoded, &ValType::F64).unwrap();
        if let ComponentValue::F64(v) = decoded {
            assert!((v - 3.14159).abs() < f64::EPSILON);
        } else {
            panic!("Expected F64 value");
        }
    }

    #[test]
    fn test_value_type_matching() {
        let bool_value = ComponentValue::Bool(true);
        assert!(bool_value.matches_type(&ValType::Bool));
        assert!(!bool_value.matches_type(&ValType::S32));

        let int_value = ComponentValue::S32(42);
        assert!(int_value.matches_type(&ValType::S32));
        assert!(!int_value.matches_type(&ValType::U32));

        let string_value = ComponentValue::String("hello".to_string());
        assert!(string_value.matches_type(&ValType::String));
        assert!(!string_value.matches_type(&ValType::Char));
    }

    #[test]
    fn test_conversion_between_core_and_component() {
        // Test i32 to bool
        let core_value = Value::I32(1);
        let component_value = core_to_component_value(&core_value, &ValType::Bool).unwrap();
        assert!(matches!(component_value, ComponentValue::Bool(true)));

        // Test i64 to u64
        let core_value = Value::I64(42);
        let component_value = core_to_component_value(&core_value, &ValType::U64).unwrap();
        assert!(matches!(component_value, ComponentValue::U64(42)));

        // Test component to core
        let component_value = ComponentValue::S32(-123);
        let core_value = component_to_core_value(&component_value).unwrap();
        assert!(matches!(core_value, Value::I32(-123)));
    }
} 