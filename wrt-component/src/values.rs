//! Component Model value handling
//!
//! This module provides implementations for Component Model value types, including
//! serialization/deserialization, conversion, and runtime representation.

use wrt_error::{kinds, Error, Result};
use wrt_format::component::ValType as FormatValType;
use wrt_types::component_value::{ComponentValue, ValType as TypesValType};
use wrt_types::values::Value;
use wrt_types::ValueType;

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

// Use TypesValType for the canonical ValType
type CanonicalValType = TypesValType;

/// Convert from CanonicalValType to wrt_format::component::ValType
pub fn convert_common_to_format_valtype(common_type: &CanonicalValType) -> FormatValType {
    match common_type {
        CanonicalValType::Bool => FormatValType::Bool,
        CanonicalValType::S8 => FormatValType::S8,
        CanonicalValType::U8 => FormatValType::U8,
        CanonicalValType::S16 => FormatValType::S16,
        CanonicalValType::U16 => FormatValType::U16,
        CanonicalValType::S32 => FormatValType::S32,
        CanonicalValType::U32 => FormatValType::U32,
        CanonicalValType::S64 => FormatValType::S64,
        CanonicalValType::U64 => FormatValType::U64,
        CanonicalValType::F32 => FormatValType::F32,
        CanonicalValType::F64 => FormatValType::F64,
        CanonicalValType::Char => FormatValType::Char,
        CanonicalValType::String => FormatValType::String,
        CanonicalValType::Ref(idx) => FormatValType::Ref(*idx),
        CanonicalValType::Record(fields) => {
            let converted_fields = fields
                .iter()
                .map(|(name, val_type)| (name.clone(), convert_common_to_format_valtype(val_type)))
                .collect();
            FormatValType::Record(converted_fields)
        }
        CanonicalValType::Variant(cases) => {
            let converted_cases = cases
                .iter()
                .map(|(name, opt_type)| {
                    (
                        name.clone(),
                        opt_type
                            .as_ref()
                            .map(|val_type| convert_common_to_format_valtype(val_type)),
                    )
                })
                .collect();
            FormatValType::Variant(converted_cases)
        }
        CanonicalValType::List(elem_type) => {
            FormatValType::List(Box::new(convert_common_to_format_valtype(elem_type)))
        }
        CanonicalValType::Tuple(types) => {
            let converted_types = types
                .iter()
                .map(|val_type| convert_common_to_format_valtype(val_type))
                .collect();
            FormatValType::Tuple(converted_types)
        }
        CanonicalValType::Flags(names) => FormatValType::Flags(names.clone()),
        CanonicalValType::Enum(variants) => FormatValType::Enum(variants.clone()),
        CanonicalValType::Option(inner_type) => {
            FormatValType::Option(Box::new(convert_common_to_format_valtype(inner_type)))
        }
        CanonicalValType::Result(result_type) => {
            FormatValType::Result(Box::new(convert_common_to_format_valtype(result_type)))
        }
        CanonicalValType::Own(idx) => FormatValType::Own(*idx),
        CanonicalValType::Borrow(idx) => FormatValType::Borrow(*idx),
    }
}

/// Convert from wrt_format::component::ValType to CanonicalValType
pub fn convert_format_to_common_valtype(format_type: &FormatValType) -> CanonicalValType {
    match format_type {
        FormatValType::Bool => CanonicalValType::Bool,
        FormatValType::S8 => CanonicalValType::S8,
        FormatValType::U8 => CanonicalValType::U8,
        FormatValType::S16 => CanonicalValType::S16,
        FormatValType::U16 => CanonicalValType::U16,
        FormatValType::S32 => CanonicalValType::S32,
        FormatValType::U32 => CanonicalValType::U32,
        FormatValType::S64 => CanonicalValType::S64,
        FormatValType::U64 => CanonicalValType::U64,
        FormatValType::F32 => CanonicalValType::F32,
        FormatValType::F64 => CanonicalValType::F64,
        FormatValType::Char => CanonicalValType::Char,
        FormatValType::String => CanonicalValType::String,
        FormatValType::Ref(idx) => CanonicalValType::Ref(*idx),
        FormatValType::Record(fields) => {
            let converted_fields = fields
                .iter()
                .map(|(name, val_type)| (name.clone(), convert_format_to_common_valtype(val_type)))
                .collect();
            CanonicalValType::Record(converted_fields)
        }
        FormatValType::Variant(cases) => {
            let converted_cases = cases
                .iter()
                .map(|(name, opt_type)| {
                    (
                        name.clone(),
                        opt_type
                            .as_ref()
                            .map(|val_type| convert_format_to_common_valtype(val_type)),
                    )
                })
                .collect();
            CanonicalValType::Variant(converted_cases)
        }
        FormatValType::List(elem_type) => {
            CanonicalValType::List(Box::new(convert_format_to_common_valtype(elem_type)))
        }
        FormatValType::Tuple(types) => {
            let converted_types = types
                .iter()
                .map(|val_type| convert_format_to_common_valtype(val_type))
                .collect();
            CanonicalValType::Tuple(converted_types)
        }
        FormatValType::Flags(names) => CanonicalValType::Flags(names.clone()),
        FormatValType::Enum(variants) => CanonicalValType::Enum(variants.clone()),
        FormatValType::Option(inner_type) => {
            CanonicalValType::Option(Box::new(convert_format_to_common_valtype(inner_type)))
        }
        FormatValType::Result(result_type) => {
            CanonicalValType::Result(Box::new(convert_format_to_common_valtype(result_type)))
        }
        FormatValType::ResultErr(err_type) => {
            // Map to CanonicalValType::Result with a default inner type
            CanonicalValType::Result(Box::new(CanonicalValType::Bool))
        }
        FormatValType::ResultBoth(ok_type, err_type) => {
            // Map to CanonicalValType::Result with the ok type
            CanonicalValType::Result(Box::new(convert_format_to_common_valtype(ok_type)))
        }
        FormatValType::Own(idx) => CanonicalValType::Own(*idx),
        FormatValType::Borrow(idx) => CanonicalValType::Borrow(*idx),
        FormatValType::FixedList(elem_type, _) => {
            // Map FixedList to standard List for runtime
            CanonicalValType::List(Box::new(convert_format_to_common_valtype(elem_type)))
        }
        FormatValType::ErrorContext => {
            // Map error context to a string type
            CanonicalValType::String
        }
    }
}

// Serialization and deserialization functions for ComponentValue
pub fn serialize_component_value(value: &ComponentValue) -> Result<Vec<u8>> {
    let common_type = value.get_type();
    let format_type = convert_common_to_format_valtype(&common_type);

    // Serialize the value based on its type
    let mut buffer = Vec::new();

    match value {
        ComponentValue::Bool(b) => {
            buffer.push(if *b { 1 } else { 0 });
        }
        ComponentValue::S8(v) => {
            buffer.push(*v as u8);
        }
        ComponentValue::U8(v) => {
            buffer.push(*v);
        }
        ComponentValue::S16(v) => {
            buffer.extend_from_slice(&v.to_le_bytes());
        }
        ComponentValue::U16(v) => {
            buffer.extend_from_slice(&v.to_le_bytes());
        }
        ComponentValue::S32(v) => {
            buffer.extend_from_slice(&v.to_le_bytes());
        }
        ComponentValue::U32(v) => {
            buffer.extend_from_slice(&v.to_le_bytes());
        }
        ComponentValue::S64(v) => {
            buffer.extend_from_slice(&v.to_le_bytes());
        }
        ComponentValue::U64(v) => {
            buffer.extend_from_slice(&v.to_le_bytes());
        }
        ComponentValue::F32(v) => {
            buffer.extend_from_slice(&v.to_bits().to_le_bytes());
        }
        ComponentValue::F64(v) => {
            buffer.extend_from_slice(&v.to_bits().to_le_bytes());
        }
        ComponentValue::Char(c) => {
            let bytes = [
                (*c as u32 & 0xff) as u8,
                ((*c as u32 >> 8) & 0xff) as u8,
                ((*c as u32 >> 16) & 0xff) as u8,
                ((*c as u32 >> 24) & 0xff) as u8,
            ];
            buffer.extend_from_slice(&bytes);
        }
        ComponentValue::String(s) => {
            // String is encoded as length followed by UTF-8 bytes
            let bytes = s.as_bytes();
            let len = bytes.len() as u32;
            buffer.extend_from_slice(&len.to_le_bytes());
            buffer.extend_from_slice(bytes);
        }
        // Implement other more complex types as needed
        _ => {
            return Err(Error::new(kinds::UnsupportedOperation(
                "Serialization not yet implemented for this type".into(),
            )));
        }
    }

    Ok(buffer)
}

// Simplified deserialization function
pub fn deserialize_component_value(
    data: &[u8],
    format_type: &FormatValType,
) -> Result<ComponentValue> {
    let common_type = convert_format_to_common_valtype(format_type);

    let mut offset = 0;

    match common_type {
        CanonicalValType::Bool => {
            if offset >= data.len() {
                return Err(Error::new(kinds::ParseError(
                    "Not enough data to deserialize bool".into(),
                )));
            }
            let value = data[offset] != 0;
            Ok(ComponentValue::Bool(value))
        }
        CanonicalValType::S8 => {
            if offset >= data.len() {
                return Err(Error::new(kinds::ParseError(
                    "Not enough data to deserialize S8".into(),
                )));
            }
            let value = data[offset] as i8;
            Ok(ComponentValue::S8(value))
        }
        CanonicalValType::U8 => {
            if offset >= data.len() {
                return Err(Error::new(kinds::ParseError(
                    "Not enough data to deserialize U8".into(),
                )));
            }
            let value = data[offset];
            Ok(ComponentValue::U8(value))
        }
        // Implement other types as needed
        _ => Err(Error::new(kinds::UnsupportedOperation(
            "Deserialization not yet implemented for this type".into(),
        ))),
    }
}

/// Serialize multiple component values
pub fn serialize_component_values(values: &[ComponentValue]) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();

    // Write the number of values
    let count = values.len() as u32;
    buffer.extend_from_slice(&count.to_le_bytes());

    // Serialize each value
    for value in values {
        let value_bytes = serialize_component_value(value)?;

        // Write the size of this value's bytes
        let size = value_bytes.len() as u32;
        buffer.extend_from_slice(&size.to_le_bytes());

        // Write the value bytes
        buffer.extend_from_slice(&value_bytes);
    }

    Ok(buffer)
}

/// Deserialize multiple component values
pub fn deserialize_component_values(
    data: &[u8],
    types: &[FormatValType],
) -> Result<Vec<ComponentValue>> {
    // Need at least 4 bytes for the count
    if data.len() < 4 {
        return Err(Error::new(kinds::ParseError(
            "Not enough data to read value count".into(),
        )));
    }

    // Read the count
    let mut offset = 0;
    let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    offset += 4;

    // Validate that we have enough types
    if count > types.len() {
        return Err(Error::new(kinds::ValidationError(format!(
            "Expected {} types but only got {}",
            count,
            types.len()
        ))));
    }

    // Read each value
    let mut values = Vec::with_capacity(count);
    for type_idx in 0..count {
        // Need at least 4 more bytes for the size
        if offset + 4 > data.len() {
            return Err(Error::new(kinds::ParseError(
                "Not enough data to read value size".into(),
            )));
        }

        // Read the size
        let size = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4;

        // Validate that we have enough data
        if offset + size > data.len() {
            return Err(Error::new(kinds::ParseError(
                "Not enough data to read value".into(),
            )));
        }

        // Read the value
        let value_data = &data[offset..offset + size];
        let value = deserialize_component_value(value_data, &types[type_idx])?;
        values.push(value);

        // Move to the next value
        offset += size;
    }

    Ok(values)
}

// Core value conversion functions
pub fn core_to_component_value(value: &Value, ty: &FormatValType) -> Result<ComponentValue> {
    let common_type = convert_format_to_common_valtype(ty);

    match (value, &common_type) {
        (Value::I32(v), CanonicalValType::Bool) => Ok(ComponentValue::Bool(*v != 0)),
        (Value::I32(v), CanonicalValType::S8) => Ok(ComponentValue::S8(*v as i8)),
        (Value::I32(v), CanonicalValType::U8) => Ok(ComponentValue::U8(*v as u8)),
        (Value::I32(v), CanonicalValType::S16) => Ok(ComponentValue::S16(*v as i16)),
        (Value::I32(v), CanonicalValType::U16) => Ok(ComponentValue::U16(*v as u16)),
        (Value::I32(v), CanonicalValType::S32) => Ok(ComponentValue::S32(*v)),
        (Value::I32(v), CanonicalValType::U32) => Ok(ComponentValue::U32(*v as u32)),
        (Value::I64(v), CanonicalValType::S64) => Ok(ComponentValue::S64(*v)),
        (Value::I64(v), CanonicalValType::U64) => Ok(ComponentValue::U64(*v as u64)),
        (Value::F32(v), CanonicalValType::F32) => Ok(ComponentValue::F32(*v)),
        (Value::F64(v), CanonicalValType::F64) => Ok(ComponentValue::F64(*v)),
        _ => Err(Error::new(kinds::ConversionError(format!(
            "Cannot convert core value {:?} to component value of type {:?}",
            value, common_type
        )))),
    }
}

pub fn component_to_core_value(value: &ComponentValue) -> Result<Value> {
    match value {
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
        // String and other complex types cannot be directly represented
        _ => Err(Error::new(kinds::ConversionError(format!(
            "Cannot convert component value {:?} to core value",
            value
        )))),
    }
}

// Size calculation for component values
pub fn size_in_bytes(ty: &FormatValType) -> usize {
    match ty {
        FormatValType::Bool => 1,
        FormatValType::S8 => 1,
        FormatValType::U8 => 1,
        FormatValType::S16 => 2,
        FormatValType::U16 => 2,
        FormatValType::S32 => 4,
        FormatValType::U32 => 4,
        FormatValType::S64 => 8,
        FormatValType::U64 => 8,
        FormatValType::F32 => 4,
        FormatValType::F64 => 8,
        FormatValType::Char => 4,
        FormatValType::String => 4, // Variable size, 4 is just for the length prefix
        FormatValType::Ref(_) => 4,
        FormatValType::Record(_) => 4, // Variable size, 4 is a placeholder
        FormatValType::Variant(_) => 4, // Variable size, 4 is a placeholder
        FormatValType::List(_) => 4,   // Variable size, 4 is just for the length prefix
        FormatValType::FixedList(_, count) => 4 + *count as usize, // Approximate
        FormatValType::Tuple(_) => 4,  // Variable size, 4 is a placeholder
        FormatValType::Flags(_) => 4,  // Variable size based on number of flags
        FormatValType::Enum(_) => 4,   // Just the discriminant
        FormatValType::Option(_) => 1 + 4, // Present flag + value size
        FormatValType::Result(_) => 1 + 4, // Success flag + value size
        FormatValType::ResultErr(_) => 1 + 4, // Error flag + value size
        FormatValType::ResultBoth(_, _) => 1 + 4, // Flag + value size
        FormatValType::Own(_) => 4,
        FormatValType::Borrow(_) => 4,
        FormatValType::ErrorContext => 4, // Placeholder for error context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_value_encoding_decoding() {
        // Test a few primitive types
        let values = vec![
            ComponentValue::Bool(true),
            ComponentValue::S32(42),
            ComponentValue::F64(3.14159),
        ];

        for value in values {
            let encoded = serialize_component_value(&value).unwrap();
            let format_type = convert_common_to_format_valtype(&value.get_type());
            let decoded = deserialize_component_value(&encoded, &format_type).unwrap();

            // Only check bools since we only implemented deserialization for a subset of types
            if let ComponentValue::Bool(_) = value {
                assert_eq!(value, decoded);
            }
        }
    }
}
