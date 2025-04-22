// Type compatibility layer between wrt-format and wrt-types
//
// This module provides conversion traits and utilities to ensure
// compatible types across the different crates in the WebAssembly
// Component Model implementation.

use std::sync::Arc;

use wrt_error::{Error, Result};
use wrt_format::component::ValType;
use wrt_types::component_value::ComponentValue;
use wrt_types::value::ValueType;
use wrt_types::Value;

use crate::values::{decode_component_value, encode_component_value};

/// Trait for converting between wrt-format and wrt-types type systems
pub trait TypeCompatibility {
    /// Convert from wrt-format::component::ValType to wrt-types::value::ValueType
    fn to_value_type(&self) -> Result<ValueType>;

    /// Convert from wrt-types::value::ValueType to wrt-format::component::ValType
    fn to_val_type(&self) -> Result<ValType>;

    /// Get the size in bytes of this type in memory
    fn size_in_bytes(&self) -> usize;
}

/// Implement TypeCompatibility for ValType
impl TypeCompatibility for ValType {
    fn to_value_type(&self) -> Result<ValueType> {
        match self {
            // Direct mappings
            ValType::S32 | ValType::U32 => Ok(ValueType::I32),
            ValType::S64 | ValType::U64 => Ok(ValueType::I64),
            ValType::F32 => Ok(ValueType::F32),
            ValType::F64 => Ok(ValueType::F64),

            // Component model types that map to reference types
            ValType::String
            | ValType::Ref(_)
            | ValType::Record(_)
            | ValType::Variant(_)
            | ValType::List(_)
            | ValType::Tuple(_)
            | ValType::Flags(_)
            | ValType::Enum(_)
            | ValType::Option(_)
            | ValType::Result(_) => Ok(ValueType::ExternRef),

            // Resources become function references
            ValType::Own(_) | ValType::Borrow(_) => Ok(ValueType::FuncRef),

            // Smaller integer types map to I32
            ValType::Bool
            | ValType::S8
            | ValType::U8
            | ValType::S16
            | ValType::U16
            | ValType::Char => Ok(ValueType::I32),
        }
    }

    fn to_val_type(&self) -> Result<ValType> {
        // Identity function - already a ValType
        Ok(self.clone())
    }

    fn size_in_bytes(&self) -> usize {
        match self {
            ValType::Bool => 1,
            ValType::S8 | ValType::U8 => 1,
            ValType::S16 | ValType::U16 => 2,
            ValType::S32 | ValType::U32 | ValType::F32 => 4,
            ValType::S64 | ValType::U64 | ValType::F64 => 8,
            ValType::Char => 4,
            ValType::String => 8, // String is represented as offset + length (4 bytes each)
            ValType::Ref(_) => 4, // References are 32-bit indices
            ValType::Record(_) => 8, // Record is represented as offset + size (4 bytes each)
            ValType::Variant(_) => 8, // Variant is represented as tag + offset (4 bytes each)
            ValType::List(_) => 8, // List is represented as offset + length (4 bytes each)
            ValType::Tuple(_) => 8, // Tuple is represented as offset + size (4 bytes each)
            ValType::Flags(_) => 8, // Flags is represented as a bitmap of 8 bytes maximum
            ValType::Enum(_) => 4, // Enum is represented as a 32-bit index
            ValType::Option(_) => 8, // Option is represented as a tag + payload (4 bytes each)
            ValType::Result(_) => 8, // Result with OK value is represented as a tag + payload
            ValType::Own(_) => 4, // Resource handle is 32 bits
            ValType::Borrow(_) => 4, // Resource handle is 32 bits
        }
    }
}

/// Implement TypeCompatibility for ValueType
impl TypeCompatibility for ValueType {
    fn to_value_type(&self) -> Result<ValueType> {
        // Identity function - already a ValueType
        Ok(self.clone())
    }

    fn to_val_type(&self) -> Result<ValType> {
        match self {
            ValueType::I32 => Ok(ValType::S32),
            ValueType::I64 => Ok(ValType::S64),
            ValueType::F32 => Ok(ValType::F32),
            ValueType::F64 => Ok(ValType::F64),
            ValueType::V128 => Err(Error::type_mismatch_error(
                "V128 not supported in Component Model",
            )),
            ValueType::FuncRef => Err(Error::type_mismatch_error(
                "FuncRef cannot be directly converted to ValType",
            )),
            ValueType::ExternRef => Err(Error::type_mismatch_error(
                "ExternRef cannot be directly converted to ValType",
            )),
        }
    }

    fn size_in_bytes(&self) -> usize {
        match self {
            ValueType::I32 => 4,
            ValueType::I64 => 8,
            ValueType::F32 => 4,
            ValueType::F64 => 8,
            ValueType::V128 => 16,
            ValueType::FuncRef => 4,
            ValueType::ExternRef => 4,
        }
    }
}

/// Convert from wrt-types Value to ComponentValue
pub fn value_to_component_value(value: &Value, ty: &ValType) -> Result<ComponentValue> {
    match (value, ty) {
        // Integer values
        (Value::I32(i), ValType::S32) => Ok(ComponentValue::S32(*i)),
        (Value::I32(i), ValType::U32) => {
            if *i < 0 {
                return Err(Error::type_mismatch_error(format!(
                    "Cannot convert negative I32 {} to U32",
                    i
                )));
            }
            Ok(ComponentValue::U32(*i as u32))
        }
        (Value::I32(i), ValType::Bool) => Ok(ComponentValue::Bool(*i != 0)),
        (Value::I32(i), ValType::S8) => Ok(ComponentValue::S8(*i as i8)),
        (Value::I32(i), ValType::U8) => {
            if *i < 0 || *i > 255 {
                return Err(Error::type_mismatch_error(format!(
                    "I32 value {} out of range for U8",
                    i
                )));
            }
            Ok(ComponentValue::U8(*i as u8))
        }
        (Value::I32(i), ValType::S16) => Ok(ComponentValue::S16(*i as i16)),
        (Value::I32(i), ValType::U16) => {
            if *i < 0 || *i > 65535 {
                return Err(Error::type_mismatch_error(format!(
                    "I32 value {} out of range for U16",
                    i
                )));
            }
            Ok(ComponentValue::U16(*i as u16))
        }
        (Value::I32(i), ValType::Char) => match char::from_u32(*i as u32) {
            Some(c) => Ok(ComponentValue::Char(c)),
            None => Err(Error::type_mismatch_error(format!(
                "Invalid character code: {}",
                i
            ))),
        },

        // 64-bit integer values
        (Value::I64(i), ValType::S64) => Ok(ComponentValue::S64(*i)),
        (Value::I64(i), ValType::U64) => {
            if *i < 0 {
                return Err(Error::type_mismatch_error(format!(
                    "Cannot convert negative I64 {} to U64",
                    i
                )));
            }
            Ok(ComponentValue::U64(*i as u64))
        }

        // Floating point values
        (Value::F32(f), ValType::F32) => Ok(ComponentValue::F32(*f)),
        (Value::F64(f), ValType::F64) => Ok(ComponentValue::F64(*f)),

        // Resource handles
        (Value::FuncRef(Some(h)), ValType::Own(_)) => Ok(ComponentValue::Own(*h)),
        (Value::FuncRef(Some(h)), ValType::Borrow(_)) => Ok(ComponentValue::Borrow(*h)),
        (Value::FuncRef(None), ValType::Own(_)) => Err(Error::type_mismatch_error(
            "Cannot convert null reference to owned resource",
        )),
        (Value::FuncRef(None), ValType::Borrow(_)) => Err(Error::type_mismatch_error(
            "Cannot convert null reference to borrowed resource",
        )),

        // Invalid combinations
        _ => Err(Error::type_mismatch_error(format!(
            "Type mismatch: cannot convert {:?} to {:?}",
            value, ty
        ))),
    }
}

/// Convert from ComponentValue to wrt-types Value
pub fn component_value_to_value(value: &ComponentValue) -> Result<Value> {
    match value {
        // Integer values
        ComponentValue::Bool(b) => Ok(Value::I32(if *b { 1 } else { 0 })),
        ComponentValue::S8(i) => Ok(Value::I32(*i as i32)),
        ComponentValue::U8(i) => Ok(Value::I32(*i as i32)),
        ComponentValue::S16(i) => Ok(Value::I32(*i as i32)),
        ComponentValue::U16(i) => Ok(Value::I32(*i as i32)),
        ComponentValue::S32(i) => Ok(Value::I32(*i)),
        ComponentValue::U32(i) => Ok(Value::I32(*i as i32)),
        ComponentValue::S64(i) => Ok(Value::I64(*i)),
        ComponentValue::U64(i) => Ok(Value::I64(*i as i64)),

        // Floating point values
        ComponentValue::F32(f) => Ok(Value::F32(*f)),
        ComponentValue::F64(f) => Ok(Value::F64(*f)),

        // Character values
        ComponentValue::Char(c) => Ok(Value::I32(*c as i32)),

        // Resource handles
        ComponentValue::Own(h) => Ok(Value::FuncRef(Some(*h))),
        ComponentValue::Borrow(h) => Ok(Value::FuncRef(Some(*h))),

        // String and complex values - create an ExternRef
        ComponentValue::String(_)
        | ComponentValue::List(_)
        | ComponentValue::Record(_)
        | ComponentValue::Variant { .. }
        | ComponentValue::Tuple(_)
        | ComponentValue::Flags(_)
        | ComponentValue::Enum(_)
        | ComponentValue::Option(_)
        | ComponentValue::Result(_) => Ok(Value::ExternRef(Some(1))), // Use 1 as placeholder
    }
}

/// Serialize component value to binary format
pub fn serialize_component_value(value: &ComponentValue, val_type: &ValType) -> Result<Vec<u8>> {
    encode_component_value(value, val_type)
}

/// Deserialize component value from binary format
pub fn deserialize_component_value(data: &[u8], val_type: &ValType) -> Result<ComponentValue> {
    decode_component_value(data, val_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_val_type_to_value_type() {
        let types = vec![
            (ValType::Bool, ValueType::I32),
            (ValType::S8, ValueType::I32),
            (ValType::U8, ValueType::I32),
            (ValType::S16, ValueType::I32),
            (ValType::U16, ValueType::I32),
            (ValType::S32, ValueType::I32),
            (ValType::U32, ValueType::I32),
            (ValType::S64, ValueType::I64),
            (ValType::U64, ValueType::I64),
            (ValType::F32, ValueType::F32),
            (ValType::F64, ValueType::F64),
            (ValType::Char, ValueType::I32),
            (ValType::String, ValueType::ExternRef),
            (ValType::List(Box::new(ValType::S32)), ValueType::ExternRef),
            (
                ValType::Record(vec![("field".to_string(), ValType::S32)]),
                ValueType::ExternRef,
            ),
            (
                ValType::Variant(vec![("case".to_string(), Some(ValType::S32))]),
                ValueType::ExternRef,
            ),
            (
                ValType::Tuple(vec![ValType::S32, ValType::S64]),
                ValueType::ExternRef,
            ),
            (
                ValType::Flags(vec!["flag1".to_string(), "flag2".to_string()]),
                ValueType::ExternRef,
            ),
            (
                ValType::Enum(vec!["variant1".to_string(), "variant2".to_string()]),
                ValueType::ExternRef,
            ),
            (
                ValType::Option(Box::new(ValType::S32)),
                ValueType::ExternRef,
            ),
            (
                ValType::Result(Box::new(ValType::S32)),
                ValueType::ExternRef,
            ),
            (ValType::Own(1), ValueType::FuncRef),
            (ValType::Borrow(1), ValueType::FuncRef),
        ];

        for (val_type, expected) in types {
            let result = val_type.to_value_type().unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_value_type_to_val_type() {
        let types = vec![
            (ValueType::I32, ValType::S32),
            (ValueType::I64, ValType::S64),
            (ValueType::F32, ValType::F32),
            (ValueType::F64, ValType::F64),
        ];

        for (value_type, expected) in types {
            let result = value_type.to_val_type().unwrap();
            assert_eq!(result, expected);
        }

        // These should fail
        assert!(ValueType::V128.to_val_type().is_err());
        assert!(ValueType::FuncRef.to_val_type().is_err());
        assert!(ValueType::ExternRef.to_val_type().is_err());
    }

    #[test]
    fn test_value_to_component_value() {
        let tests = vec![
            (Value::I32(42), ValType::S32, ComponentValue::S32(42)),
            (Value::I32(42), ValType::U32, ComponentValue::U32(42)),
            (Value::I32(1), ValType::Bool, ComponentValue::Bool(true)),
            (Value::I32(0), ValType::Bool, ComponentValue::Bool(false)),
            (Value::I32(-42), ValType::S8, ComponentValue::S8(-42i8)),
            (Value::I32(42), ValType::U8, ComponentValue::U8(42u8)),
            (
                Value::I32(-1000),
                ValType::S16,
                ComponentValue::S16(-1000i16),
            ),
            (Value::I32(1000), ValType::U16, ComponentValue::U16(1000u16)),
            (Value::I32(65), ValType::Char, ComponentValue::Char('A')),
            (Value::I64(-42), ValType::S64, ComponentValue::S64(-42i64)),
            (Value::I64(42), ValType::U64, ComponentValue::U64(42u64)),
            (Value::F32(3.14), ValType::F32, ComponentValue::F32(3.14f32)),
            (
                Value::F64(3.14159),
                ValType::F64,
                ComponentValue::F64(3.14159f64),
            ),
            (
                Value::FuncRef(Some(123)),
                ValType::Own(1),
                ComponentValue::Own(123),
            ),
            (
                Value::FuncRef(Some(456)),
                ValType::Borrow(1),
                ComponentValue::Borrow(456),
            ),
        ];

        for (value, ty, expected) in tests {
            let result = value_to_component_value(&value, &ty).unwrap();
            assert_eq!(result, expected);
        }

        // These should fail
        assert!(value_to_component_value(&Value::I32(-1), &ValType::U32).is_err());
        assert!(value_to_component_value(&Value::I32(256), &ValType::U8).is_err());
        assert!(value_to_component_value(&Value::I32(65536), &ValType::U16).is_err());
        assert!(value_to_component_value(&Value::I64(-1), &ValType::U64).is_err());
        assert!(value_to_component_value(&Value::FuncRef(None), &ValType::Own(1)).is_err());
    }

    #[test]
    fn test_component_value_to_value() {
        let tests = vec![
            (ComponentValue::Bool(true), Value::I32(1)),
            (ComponentValue::Bool(false), Value::I32(0)),
            (ComponentValue::S8(-42), Value::I32(-42)),
            (ComponentValue::U8(42), Value::I32(42)),
            (ComponentValue::S16(-1000), Value::I32(-1000)),
            (ComponentValue::U16(1000), Value::I32(1000)),
            (ComponentValue::S32(-100000), Value::I32(-100000)),
            (ComponentValue::U32(100000), Value::I32(100000)),
            (ComponentValue::S64(-10000000000), Value::I64(-10000000000)),
            (ComponentValue::U64(10000000000), Value::I64(10000000000)),
            (ComponentValue::F32(3.14), Value::F32(3.14)),
            (ComponentValue::F64(3.14159), Value::F64(3.14159)),
            (ComponentValue::Char('A'), Value::I32(65)),
            (ComponentValue::Own(123), Value::FuncRef(Some(123))),
            (ComponentValue::Borrow(456), Value::FuncRef(Some(456))),
        ];

        for (component_value, expected) in tests {
            let result = component_value_to_value(&component_value).unwrap();
            assert_eq!(result, expected);
        }

        // Complex types should convert to ExternRef
        let complex_values = vec![
            ComponentValue::String("test".to_string()),
            ComponentValue::List(vec![ComponentValue::S32(1), ComponentValue::S32(2)]),
            ComponentValue::Record(std::collections::HashMap::from([(
                "field".to_string(),
                ComponentValue::S32(42),
            )])),
            ComponentValue::Variant {
                case: 0,
                value: Some(Box::new(ComponentValue::S32(42))),
            },
            ComponentValue::Tuple(vec![ComponentValue::S32(1), ComponentValue::S64(2)]),
            ComponentValue::Flags(std::collections::HashMap::from([
                ("flag1".to_string(), true),
                ("flag2".to_string(), false),
            ])),
            ComponentValue::Enum(1),
            ComponentValue::Option(Some(Box::new(ComponentValue::S32(42)))),
            ComponentValue::Result(Ok(Some(Box::new(ComponentValue::S32(42))))),
        ];

        for value in complex_values {
            let result = component_value_to_value(&value).unwrap();
            assert_eq!(result, Value::ExternRef(Some(1)));
        }
    }
}
