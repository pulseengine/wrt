//! Component Model value handling
//!
//! This module provides implementations for Component Model value types, including
//! serialization/deserialization, conversion, and runtime representation.

use wrt_common::component::{ComponentValue, ValType as CommonValType};
use wrt_common::{FromFormat, ToFormat};
use wrt_error::{kinds, Error, Result};
use wrt_format::component::ValType;
use wrt_format::component::ValType as FormatValType;
use wrt_intercept::Value;
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

// Further down in the file, alias CommonValType to avoid changing all occurrences
type CommonValType = TypesValType;

/// Convert from CommonValType to wrt_format::component::ValType
pub fn convert_common_to_format_valtype(common_type: &CommonValType) -> ValType {
    match common_type {
        CommonValType::Bool => ValType::Bool,
        CommonValType::S8 => ValType::S8,
        CommonValType::U8 => ValType::U8,
        CommonValType::S16 => ValType::S16,
        CommonValType::U16 => ValType::U16,
        CommonValType::S32 => ValType::S32,
        CommonValType::U32 => ValType::U32,
        CommonValType::S64 => ValType::S64,
        CommonValType::U64 => ValType::U64,
        CommonValType::F32 => ValType::F32,
        CommonValType::F64 => ValType::F64,
        CommonValType::Char => ValType::Char,
        CommonValType::String => ValType::String,
        CommonValType::Ref(idx) => ValType::Ref(*idx),
        CommonValType::Record(fields) => {
            let converted_fields = fields
                .iter()
                .map(|(name, val_type)| (name.clone(), convert_common_to_format_valtype(val_type)))
                .collect();
            ValType::Record(converted_fields)
        }
        CommonValType::Variant(cases) => {
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
            ValType::Variant(converted_cases)
        }
        CommonValType::List(elem_type) => {
            ValType::List(Box::new(convert_common_to_format_valtype(elem_type)))
        }
        CommonValType::Tuple(types) => {
            let converted_types = types
                .iter()
                .map(|val_type| convert_common_to_format_valtype(val_type))
                .collect();
            ValType::Tuple(converted_types)
        }
        CommonValType::Flags(names) => ValType::Flags(names.clone()),
        CommonValType::Enum(variants) => ValType::Enum(variants.clone()),
        CommonValType::Option(inner_type) => {
            ValType::Option(Box::new(convert_common_to_format_valtype(inner_type)))
        }
        CommonValType::Result(result_type) => {
            ValType::Result(Box::new(convert_common_to_format_valtype(result_type)))
        }
        CommonValType::Own(idx) => ValType::Own(*idx),
        CommonValType::Borrow(idx) => ValType::Borrow(*idx),
    }
}

/// Convert from wrt_format::component::ValType to CommonValType
pub fn convert_format_to_common_valtype(format_type: &ValType) -> CommonValType {
    match format_type {
        ValType::Bool => CommonValType::Bool,
        ValType::S8 => CommonValType::S8,
        ValType::U8 => CommonValType::U8,
        ValType::S16 => CommonValType::S16,
        ValType::U16 => CommonValType::U16,
        ValType::S32 => CommonValType::S32,
        ValType::U32 => CommonValType::U32,
        ValType::S64 => CommonValType::S64,
        ValType::U64 => CommonValType::U64,
        ValType::F32 => CommonValType::F32,
        ValType::F64 => CommonValType::F64,
        ValType::Char => CommonValType::Char,
        ValType::String => CommonValType::String,
        ValType::Ref(idx) => CommonValType::Ref(*idx),
        ValType::Record(fields) => {
            let converted_fields = fields
                .iter()
                .map(|(name, val_type)| (name.clone(), convert_format_to_common_valtype(val_type)))
                .collect();
            CommonValType::Record(converted_fields)
        }
        ValType::Variant(cases) => {
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
            CommonValType::Variant(converted_cases)
        }
        ValType::List(elem_type) => {
            CommonValType::List(Box::new(convert_format_to_common_valtype(elem_type)))
        }
        ValType::Tuple(types) => {
            let converted_types = types
                .iter()
                .map(|val_type| convert_format_to_common_valtype(val_type))
                .collect();
            CommonValType::Tuple(converted_types)
        }
        ValType::Flags(names) => CommonValType::Flags(names.clone()),
        ValType::Enum(variants) => CommonValType::Enum(variants.clone()),
        ValType::Option(inner_type) => {
            CommonValType::Option(Box::new(convert_format_to_common_valtype(inner_type)))
        }
        ValType::Result(result_type) => {
            CommonValType::Result(Box::new(convert_format_to_common_valtype(result_type)))
        }
        ValType::ResultErr(err_type) => {
            // Map to CommonValType::Result with a default inner type
            CommonValType::Result(Box::new(CommonValType::Bool))
        }
        ValType::ResultBoth(ok_type, err_type) => {
            // Map to CommonValType::Result with the ok type
            CommonValType::Result(Box::new(convert_format_to_common_valtype(ok_type)))
        }
        ValType::Own(idx) => CommonValType::Own(*idx),
        ValType::Borrow(idx) => CommonValType::Borrow(*idx),
    }
}

// Serialization and deserialization functions for ComponentValue
pub fn serialize_component_value(value: &ComponentValue) -> Result<Vec<u8>> {
    let common_type = value.get_type();
    let format_type = convert_common_to_format_valtype(&common_type);

    // Serialize the value
    let mut buffer = Vec::new();

    // Implementation of serialization based on the type
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
            buffer.extend_from_slice(&(*c as u32).to_le_bytes());
        }
        ComponentValue::String(s) => {
            let bytes = s.as_bytes();
            buffer.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            buffer.extend_from_slice(bytes);
        }
        // For complex types, would need recursive serialization
        // Implementation omitted for brevity
        _ => return Err(Error::new("Unsupported value type for serialization")),
    }

    Ok(buffer)
}

/// Internal helper function for deserializing component values with offset tracking
fn deserialize_component_value_with_offset(
    data: &[u8],
    offset: &mut usize,
    ty: &ValType,
) -> Result<ComponentValue> {
    match ty {
        ValType::Bool => {
            if *offset >= data.len() {
                return Err(Error::new("Not enough data to deserialize bool"));
            }
            let value = data[*offset] != 0;
            *offset += 1;
            Ok(ComponentValue::Bool(value))
        }
        ValType::S8 => {
            if *offset >= data.len() {
                return Err(Error::new("Not enough data to deserialize S8"));
            }
            let value = data[*offset] as i8;
            *offset += 1;
            Ok(ComponentValue::S8(value))
        }
        ValType::U8 => {
            if *offset >= data.len() {
                return Err(Error::new("Not enough data to deserialize U8"));
            }
            let value = data[*offset];
            *offset += 1;
            Ok(ComponentValue::U8(value))
        }
        ValType::S16 => {
            if *offset + 2 > data.len() {
                return Err(Error::new("Not enough data to deserialize S16"));
            }
            let mut bytes = [0u8; 2];
            bytes.copy_from_slice(&data[*offset..*offset + 2]);
            *offset += 2;
            Ok(ComponentValue::S16(i16::from_le_bytes(bytes)))
        }
        ValType::U16 => {
            if *offset + 2 > data.len() {
                return Err(Error::new("Not enough data to deserialize U16"));
            }
            let mut bytes = [0u8; 2];
            bytes.copy_from_slice(&data[*offset..*offset + 2]);
            *offset += 2;
            Ok(ComponentValue::U16(u16::from_le_bytes(bytes)))
        }
        ValType::S32 => {
            if *offset + 4 > data.len() {
                return Err(Error::new("Not enough data to deserialize S32"));
            }
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[*offset..*offset + 4]);
            *offset += 4;
            Ok(ComponentValue::S32(i32::from_le_bytes(bytes)))
        }
        ValType::U32 => {
            if *offset + 4 > data.len() {
                return Err(Error::new("Not enough data to deserialize U32"));
            }
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[*offset..*offset + 4]);
            *offset += 4;
            Ok(ComponentValue::U32(u32::from_le_bytes(bytes)))
        }
        ValType::S64 => {
            if *offset + 8 > data.len() {
                return Err(Error::new("Not enough data to deserialize S64"));
            }
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&data[*offset..*offset + 8]);
            *offset += 8;
            Ok(ComponentValue::S64(i64::from_le_bytes(bytes)))
        }
        ValType::U64 => {
            if *offset + 8 > data.len() {
                return Err(Error::new("Not enough data to deserialize U64"));
            }
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&data[*offset..*offset + 8]);
            *offset += 8;
            Ok(ComponentValue::U64(u64::from_le_bytes(bytes)))
        }
        ValType::F32 => {
            if *offset + 4 > data.len() {
                return Err(Error::new("Not enough data to deserialize F32"));
            }
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[*offset..*offset + 4]);
            *offset += 4;
            Ok(ComponentValue::F32(f32::from_le_bytes(bytes)))
        }
        ValType::F64 => {
            if *offset + 8 > data.len() {
                return Err(Error::new("Not enough data to deserialize F64"));
            }
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&data[*offset..*offset + 8]);
            *offset += 8;
            Ok(ComponentValue::F64(f64::from_le_bytes(bytes)))
        }
        ValType::Char => {
            if *offset >= data.len() {
                return Err(Error::new("Not enough data to deserialize char"));
            }
            let s = String::from_utf8_lossy(&data[*offset..*offset + 4]);
            *offset += 4;
            let c = s
                .chars()
                .next()
                .ok_or_else(|| Error::invalid_data("Invalid UTF-8 sequence"))?;
            Ok(ComponentValue::Char(c))
        }
        ValType::String => {
            if *offset >= data.len() {
                return Err(Error::new("Not enough data to deserialize string"));
            }
            let len = u32::from_le_bytes(data[*offset..*offset + 4].try_into().unwrap());
            *offset += 4;
            if *offset + len as usize > data.len() {
                return Err(Error::new("Not enough data to deserialize string"));
            }
            let s = String::from_utf8_lossy(&data[*offset..*offset + len as usize]);
            *offset += len as usize;
            Ok(ComponentValue::String(s.to_string()))
        }
        ValType::List(elem_type) => {
            if *offset >= data.len() {
                return Err(Error::new("Not enough data to deserialize list length"));
            }
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[*offset..*offset + 4]);
            *offset += 4;
            let count = u32::from_le_bytes(bytes) as usize;
            if *offset + count * size_in_bytes(elem_type) > data.len() {
                return Err(Error::new("Not enough data to deserialize list"));
            }
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                let item = deserialize_component_value_with_offset(data, offset, elem_type)?;
                items.push(item);
            }
            Ok(ComponentValue::List(items))
        }
        ValType::Record(field_types) => {
            let mut fields = HashMap::new();
            for (name, field_type) in field_types {
                let value = deserialize_component_value_with_offset(data, offset, field_type)?;
                fields.insert(name.clone(), value);
            }
            Ok(ComponentValue::Record(fields))
        }
        ValType::Variant(cases) => {
            if *offset + 4 > data.len() {
                return Err(Error::new("Not enough data to deserialize variant case"));
            }

            // Read the case index as a u32 (4 bytes)
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[*offset..*offset + 4]);
            *offset += 4;
            let case_index = u32::from_le_bytes(bytes);

            // Find the corresponding case
            if case_index as usize >= cases.len() {
                return Err(Error::new(format!(
                    "Invalid variant case index: {}",
                    case_index
                )));
            }

            let (case_name, case_type) = &cases[case_index as usize];

            match case_type {
                Some(val_type) => {
                    // Deserialize the nested value
                    let value = deserialize_component_value_with_offset(data, offset, val_type)?;
                    Ok(ComponentValue::Variant {
                        case: case_index,
                        value: Some(Box::new(value)),
                    })
                }
                None => {
                    // Case without a value
                    Ok(ComponentValue::Variant {
                        case: case_index,
                        value: None,
                    })
                }
            }
        }
        ValType::Tuple(types) => {
            let mut items = Vec::with_capacity(types.len());
            for item_type in types {
                let item = deserialize_component_value_with_offset(data, offset, item_type)?;
                items.push(item);
            }
            Ok(ComponentValue::Tuple(items))
        }
        ValType::Flags(names) => {
            let total_bytes = (names.len() + 7) / 8;
            if *offset + total_bytes > data.len() {
                return Err(Error::new("Not enough data to deserialize flags"));
            }
            let mut flags = HashMap::new();
            for (i, name) in names.iter().enumerate() {
                let byte_index = i / 8;
                let bit_index = i % 8;
                let flag_value = (data[*offset + byte_index] & (1 << bit_index)) != 0;
                flags.insert(name.clone(), flag_value);
            }
            *offset += total_bytes;
            Ok(ComponentValue::Flags(flags))
        }
        ValType::Enum(variants) => {
            if *offset >= data.len() {
                return Err(Error::new("Not enough data to deserialize enum value"));
            }
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[*offset..*offset + 4]);
            *offset += 4;
            let case_name = u32::from_le_bytes(bytes);
            if case_name as usize >= variants.len() {
                return Err(Error::new(format!("Invalid enum variant: {}", case_name)));
            }
            Ok(ComponentValue::Enum(case_name))
        }
        ValType::Option(inner_type) => {
            if *offset >= data.len() {
                return Err(Error::new("Not enough data to deserialize option tag"));
            }
            let tag = data[*offset];
            *offset += 1;
            let value = if tag == 0 {
                None
            } else {
                if *offset >= data.len() {
                    return Err(Error::new("Insufficient data to deserialize option value"));
                }
                let value = deserialize_component_value_with_offset(data, offset, inner_type)?;
                Some(Box::new(value))
            };
            Ok(ComponentValue::Option(value))
        }
        ValType::Result(result_type) => {
            if *offset >= data.len() {
                return Err(Error::new("Not enough data to deserialize result tag"));
            }
            let tag = data[*offset];
            *offset += 1;
            let value = if tag == 0 {
                Err(if *offset < data.len() {
                    let error = deserialize_component_value_with_offset(data, offset, result_type)?;
                    Some(Box::new(error))
                } else {
                    None
                })
            } else {
                Ok(if *offset < data.len() {
                    let ok = deserialize_component_value_with_offset(data, offset, result_type)?;
                    Some(Box::new(ok))
                } else {
                    None
                })
            };
            Ok(ComponentValue::Result(value))
        }
        ValType::Own(_) => {
            if *offset >= data.len() {
                return Err(Error::new("Not enough data to deserialize handle"));
            }
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[*offset..*offset + 4]);
            *offset += 4;
            let handle = u32::from_le_bytes(bytes);
            Ok(ComponentValue::Own(handle))
        }
        ValType::Borrow(_) => {
            if *offset >= data.len() {
                return Err(Error::new("Not enough data to deserialize handle"));
            }
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[*offset..*offset + 4]);
            *offset += 4;
            let handle = u32::from_le_bytes(bytes);
            Ok(ComponentValue::Borrow(handle))
        }
        // For more complex types, would need recursive deserialization
        // Implementation omitted for brevity
        _ => Err(Error::new("Unsupported value type for deserialization")),
    }
}

/// Encode a component value to bytes
pub fn encode_component_value(value: &ComponentValue, ty: &ValType) -> Result<Vec<u8>> {
    match (value, ty) {
        (ComponentValue::Bool(b), ValType::Bool) => Ok(vec![if *b { 1 } else { 0 }]),
        (ComponentValue::S8(n), ValType::S8) => Ok(vec![*n as u8]),
        (ComponentValue::U8(n), ValType::U8) => Ok(vec![*n]),
        (ComponentValue::S16(n), ValType::S16) => Ok(n.to_le_bytes().to_vec()),
        (ComponentValue::U16(n), ValType::U16) => Ok(n.to_le_bytes().to_vec()),
        (ComponentValue::S32(n), ValType::S32) => Ok(n.to_le_bytes().to_vec()),
        (ComponentValue::U32(n), ValType::U32) => Ok(n.to_le_bytes().to_vec()),
        (ComponentValue::S64(n), ValType::S64) => Ok(n.to_le_bytes().to_vec()),
        (ComponentValue::U64(n), ValType::U64) => Ok(n.to_le_bytes().to_vec()),
        (ComponentValue::F32(n), ValType::F32) => Ok(n.to_le_bytes().to_vec()),
        (ComponentValue::F64(n), ValType::F64) => Ok(n.to_le_bytes().to_vec()),
        (ComponentValue::Char(c), ValType::Char) => {
            let mut buffer = Vec::new();
            buffer.extend_from_slice(&(*c as u32).to_le_bytes());
            Ok(buffer)
        }
        (ComponentValue::String(s), ValType::String) => {
            let mut buffer = Vec::new();
            buffer.extend_from_slice(s.as_bytes());
            buffer.push(0); // Null terminator
            Ok(buffer)
        }
        (ComponentValue::Record(fields), ValType::Record(field_types)) => {
            let mut buffer = Vec::new();
            for (name, field_type) in field_types {
                if let Some(field_value) = fields.get(name) {
                    buffer.extend_from_slice(&encode_component_value(field_value, &field_type)?);
                } else {
                    return Err(Error::invalid_data(format!(
                        "Missing field '{}' in record",
                        name
                    )));
                }
            }
            Ok(buffer)
        }
        (ComponentValue::Variant { case, value }, ValType::Variant(cases)) => {
            let mut buffer = Vec::new();
            if *case as usize >= cases.len() {
                return Err(Error::invalid_data("Invalid variant case"));
            }

            buffer.extend_from_slice(&case.to_le_bytes()); // Case discriminant

            if let Some(case_value) = value {
                if let Some(case_type) = &cases[*case as usize].1 {
                    buffer.extend_from_slice(&encode_component_value(case_value, &case_type)?);
                }
            }

            Ok(buffer)
        }
        (ComponentValue::List(elements), ValType::List(elem_type)) => {
            let mut buffer = Vec::new();

            // Write element count
            buffer.extend_from_slice(&(elements.len() as u32).to_le_bytes());

            // Write each element
            for element in elements {
                buffer.extend_from_slice(&encode_component_value(element, &*elem_type)?);
            }

            Ok(buffer)
        }
        (ComponentValue::Tuple(elements), ValType::Tuple(types)) => {
            let mut buffer = Vec::new();

            if elements.len() != types.len() {
                return Err(Error::invalid_data(format!(
                    "Tuple size mismatch: expected {}, got {}",
                    types.len(),
                    elements.len()
                )));
            }

            for (element, ty) in elements.iter().zip(types.iter()) {
                buffer.extend_from_slice(&encode_component_value(element, ty)?);
            }

            Ok(buffer)
        }
        (ComponentValue::Flags(flags), ValType::Flags(names)) => {
            // Represent flags as a bit vector
            let mut bits = vec![0u8; (names.len() + 7) / 8]; // Ceiling division to determine byte count

            for (i, name) in names.iter().enumerate() {
                if let Some(value) = flags.get(name) {
                    if *value {
                        bits[i / 8] |= 1 << (i % 8);
                    }
                }
            }

            Ok(bits)
        }
        (ComponentValue::Enum(variant), ValType::Enum(variants)) => {
            if *variant >= variants.len() as u32 {
                return Err(Error::invalid_data("Invalid enum variant"));
            }
            Ok(variant.to_le_bytes().to_vec())
        }
        (ComponentValue::Option(opt), ValType::Option(inner_type)) => {
            let mut buffer = vec![];
            match opt {
                Some(v) => {
                    buffer.push(1); // Some tag
                    buffer.extend_from_slice(&encode_component_value(v, &*inner_type)?);
                }
                None => {
                    buffer.push(0); // None tag
                }
            }
            Ok(buffer)
        }
        (ComponentValue::Result(val), ValType::Result(result_type)) => {
            let mut buffer = vec![];
            match val {
                Ok(v) => {
                    buffer.push(1); // Ok tag
                    if let Some(value) = v {
                        buffer.extend_from_slice(&encode_component_value(value, &*result_type)?);
                    }
                }
                Err(_) => {
                    buffer.push(0); // Err tag
                }
            }
            Ok(buffer)
        }
        (ComponentValue::Result(val), ValType::ResultErr(err_type)) => {
            let mut buffer = vec![];
            match val {
                Ok(_) => {
                    buffer.push(1); // Ok tag with no value
                }
                Err(v) => {
                    buffer.push(0); // Err tag
                    if let Some(value) = v {
                        buffer.extend_from_slice(&encode_component_value(value, &*err_type)?);
                    }
                }
            }
            Ok(buffer)
        }
        (ComponentValue::Result(val), ValType::ResultBoth(ok_type, err_type)) => {
            let mut buffer = vec![];
            match val {
                Ok(v) => {
                    buffer.push(1); // Ok tag
                    if let Some(value) = v {
                        buffer.extend_from_slice(&encode_component_value(value, &*ok_type)?);
                    }
                }
                Err(v) => {
                    buffer.push(0); // Err tag
                    if let Some(value) = v {
                        buffer.extend_from_slice(&encode_component_value(value, &*err_type)?);
                    }
                }
            }
            Ok(buffer)
        }
        (ComponentValue::Own(handle), ValType::Own(_)) => {
            let buffer = handle.to_le_bytes().to_vec();
            Ok(buffer)
        }
        (ComponentValue::Borrow(handle), ValType::Borrow(_)) => {
            let buffer = handle.to_le_bytes().to_vec();
            Ok(buffer)
        }
        _ => Err(Error::type_mismatch_error(format!(
            "Cannot encode value {:?} as type {:?}",
            value, ty
        ))),
    }
}

/// Deserialize a single ComponentValue
pub fn deserialize_component_value(data: &[u8], format_type: &ValType) -> Result<ComponentValue> {
    let mut offset = 0;
    deserialize_component_value_with_offset(data, &mut offset, format_type)
}

/// Convert a core WebAssembly value to a component value
pub fn core_to_component_value(value: &Value, ty: &ValType) -> Result<ComponentValue> {
    match (value, ty) {
        (Value::I32(v), ValType::Bool) => Ok(ComponentValue::Bool(*v != 0)),
        (Value::I32(v), ValType::S8) => Ok(ComponentValue::S8(*v as i8)),
        (Value::I32(v), ValType::U8) => Ok(ComponentValue::U8(*v as u8)),
        (Value::I32(v), ValType::S16) => Ok(ComponentValue::S16(*v as i16)),
        (Value::I32(v), ValType::U16) => Ok(ComponentValue::U16(*v as u16)),
        (Value::I32(v), ValType::S32) => Ok(ComponentValue::S32(*v)),
        (Value::I32(v), ValType::U32) => Ok(ComponentValue::U32(*v as u32)),
        (Value::I64(v), ValType::S64) => Ok(ComponentValue::S64(*v)),
        (Value::I64(v), ValType::U64) => Ok(ComponentValue::U64(*v as u64)),
        (Value::F32(v), ValType::F32) => Ok(ComponentValue::F32(*v)),
        (Value::F64(v), ValType::F64) => Ok(ComponentValue::F64(*v)),
        _ => Err(Error::conversion_error(format!(
            "Cannot convert core value {:?} to component value of type {:?}",
            value, ty
        ))),
    }
}

/// Convert a component value to a core WebAssembly value
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
        _ => Err(Error::conversion_error(format!(
            "Cannot convert component value {:?} to core value",
            value
        ))),
    }
}

/// Serialize multiple component values
pub fn serialize_component_values(values: &[ComponentValue]) -> Result<Vec<u8>> {
    let mut result = Vec::new();

    // Write number of values
    result.extend_from_slice(&(values.len() as u32).to_le_bytes());

    // Write each value (simplified implementation)
    for value in values {
        // Get the value type
        let val_type = value.get_type();

        // Convert to format ValType
        let format_val_type = convert_common_to_format_valtype(&val_type);

        // Serialize the value
        let value_data = encode_component_value(value, &format_val_type)?;

        // Write value length and data
        result.extend_from_slice(&(value_data.len() as u32).to_le_bytes());
        result.extend_from_slice(&value_data);
    }

    Ok(result)
}

/// Deserialize multiple component values
pub fn deserialize_component_values(
    data: &[u8],
    types: &[wrt_format::component::ValType],
) -> Result<Vec<ComponentValue>> {
    let mut offset = 0;
    let mut result = Vec::new();

    // Read each value according to the provided types
    for ty in types {
        if offset >= data.len() {
            return Err(Error::new(kinds::ParseError(
                "Unexpected end of input while deserializing component values".to_string(),
            )));
        }

        let value = deserialize_component_value_with_offset(data, &mut offset, ty)?;
        result.push(value);
    }

    Ok(result)
}

/// Serialize a ValType to bytes
fn serialize_val_type(ty: &ValType) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();

    // Write the type tag
    match ty {
        ValType::Bool => buffer.push(0),
        ValType::S8 => buffer.push(1),
        ValType::U8 => buffer.push(2),
        ValType::S16 => buffer.push(3),
        ValType::U16 => buffer.push(4),
        ValType::S32 => buffer.push(5),
        ValType::U32 => buffer.push(6),
        ValType::S64 => buffer.push(7),
        ValType::U64 => buffer.push(8),
        ValType::F32 => buffer.push(9),
        ValType::F64 => buffer.push(10),
        ValType::Char => buffer.push(11),
        ValType::String => buffer.push(12),
        ValType::List(_) => buffer.push(13),
        ValType::Record(_) => buffer.push(14),
        ValType::Variant(_) => buffer.push(15),
        ValType::Tuple(_) => buffer.push(16),
        ValType::Flags(_) => buffer.push(17),
        ValType::Enum(_) => buffer.push(18),
        ValType::Option(_) => buffer.push(19),
        ValType::Result(_) => buffer.push(20),
        ValType::ResultErr(_) => buffer.push(21),
        ValType::ResultBoth(_, _) => buffer.push(22),
        ValType::Own(_) => buffer.push(23),
        ValType::Borrow(_) => buffer.push(24),
        ValType::Ref(_) => buffer.push(25),
    }

    // Additional information for complex types
    match ty {
        ValType::List(elem_type) => {
            buffer.extend_from_slice(&serialize_val_type(elem_type)?);
        }
        // For other complex types, we would add serialization logic here
        _ => {}
    }

    Ok(buffer)
}

/// Convert ValueType to TypesValType
pub fn value_type_to_common_valtype(
    value_type: &wrt_types::types::ValueType,
) -> wrt_types::component_value::ValType {
    match value_type {
        wrt_types::types::ValueType::I32 => wrt_types::component_value::ValType::S32,
        wrt_types::types::ValueType::I64 => wrt_types::component_value::ValType::S64,
        wrt_types::types::ValueType::F32 => wrt_types::component_value::ValType::F32,
        wrt_types::types::ValueType::F64 => wrt_types::component_value::ValType::F64,
        wrt_types::types::ValueType::V128 => wrt_types::component_value::ValType::Tuple(vec![
            wrt_types::component_value::ValType::S64,
            wrt_types::component_value::ValType::S64,
        ]),
        wrt_types::types::ValueType::FuncRef => wrt_types::component_value::ValType::Own(0), // Default to resource type 0
        wrt_types::types::ValueType::ExternRef => wrt_types::component_value::ValType::Ref(0), // Default to type index 0
    }
}

/// Convert FormatValType to CommonValType
pub fn format_valtype_to_common_valtype(
    format_val_type: &wrt_format::component::ValType,
) -> CommonValType {
    match format_val_type {
        ValType::Bool => CommonValType::Bool,
        ValType::S8 => CommonValType::S8,
        ValType::U8 => CommonValType::U8,
        ValType::S16 => CommonValType::S16,
        ValType::U16 => CommonValType::U16,
        ValType::S32 => CommonValType::S32,
        ValType::U32 => CommonValType::U32,
        ValType::S64 => CommonValType::S64,
        ValType::U64 => CommonValType::U64,
        ValType::F32 => CommonValType::F32,
        ValType::F64 => CommonValType::F64,
        ValType::Char => CommonValType::Char,
        ValType::String => CommonValType::String,
        ValType::Ref(idx) => CommonValType::Ref(*idx),
        ValType::Record(fields) => {
            let converted_fields = fields
                .iter()
                .map(|(name, val_type)| (name.clone(), format_valtype_to_common_valtype(val_type)))
                .collect();
            CommonValType::Record(converted_fields)
        }
        ValType::Variant(cases) => {
            let converted_cases = cases
                .iter()
                .map(|(name, opt_type)| {
                    (
                        name.clone(),
                        opt_type
                            .as_ref()
                            .map(|val_type| Box::new(format_valtype_to_common_valtype(val_type))),
                    )
                })
                .collect();
            CommonValType::Variant(converted_cases)
        }
        ValType::List(elem_type) => {
            CommonValType::List(Box::new(format_valtype_to_common_valtype(elem_type)))
        }
        ValType::Tuple(types) => {
            let converted_types = types
                .iter()
                .map(|val_type| format_valtype_to_common_valtype(val_type))
                .collect();
            CommonValType::Tuple(converted_types)
        }
        ValType::Flags(names) => CommonValType::Flags(names.clone()),
        ValType::Enum(variants) => CommonValType::Enum(variants.clone()),
        ValType::Option(inner_type) => {
            CommonValType::Option(Box::new(format_valtype_to_common_valtype(inner_type)))
        }
        ValType::Result(result_type) => {
            // Use the tuple variant
            CommonValType::Result(Box::new(format_valtype_to_common_valtype(result_type)))
        }
        ValType::ResultErr(err_type) => {
            // For error-only results, convert to a Result with the error type
            CommonValType::Result(Box::new(format_valtype_to_common_valtype(err_type)))
        }
        ValType::ResultBoth(ok_type, _err_type) => {
            // For results with both ok and error, prioritize the ok type
            CommonValType::Result(Box::new(format_valtype_to_common_valtype(ok_type)))
        }
        ValType::Own(idx) => CommonValType::Own(*idx),
        ValType::Borrow(idx) => CommonValType::Borrow(*idx),
    }
}

/// Convert a CommonValType to a FormatValType
pub fn common_valtype_to_format_valtype(
    common_val_type: &CommonValType,
) -> wrt_format::component::ValType {
    match common_val_type {
        CommonValType::Bool => wrt_format::component::ValType::Bool,
        CommonValType::S8 => wrt_format::component::ValType::S8,
        CommonValType::U8 => wrt_format::component::ValType::U8,
        CommonValType::S16 => wrt_format::component::ValType::S16,
        CommonValType::U16 => wrt_format::component::ValType::U16,
        CommonValType::S32 => wrt_format::component::ValType::S32,
        CommonValType::U32 => wrt_format::component::ValType::U32,
        CommonValType::S64 => wrt_format::component::ValType::S64,
        CommonValType::U64 => wrt_format::component::ValType::U64,
        CommonValType::F32 => wrt_format::component::ValType::F32,
        CommonValType::F64 => wrt_format::component::ValType::F64,
        CommonValType::Char => wrt_format::component::ValType::Char,
        CommonValType::String => wrt_format::component::ValType::String,
        CommonValType::List(elem_type) => wrt_format::component::ValType::List(Box::new(
            common_valtype_to_format_valtype(elem_type),
        )),
        CommonValType::Record(fields) => {
            let format_fields = fields
                .iter()
                .map(|(name, val_type)| (name.clone(), common_valtype_to_format_valtype(val_type)))
                .collect();
            wrt_format::component::ValType::Record(format_fields)
        }
        CommonValType::Variant(cases) => {
            let format_cases = cases
                .iter()
                .map(|(name, val_type)| {
                    (
                        name.clone(),
                        val_type
                            .as_ref()
                            .map(|vt| common_valtype_to_format_valtype(vt)),
                    )
                })
                .collect();
            wrt_format::component::ValType::Variant(format_cases)
        }
        CommonValType::Enum(cases) => wrt_format::component::ValType::Enum(cases.clone()),
        CommonValType::Option(inner_type) => wrt_format::component::ValType::Option(Box::new(
            common_valtype_to_format_valtype(inner_type),
        )),
        CommonValType::Result(ok_type) => match ok_type.as_ref() {
            // If the inner type is a tuple, handle it as a result with both ok and err
            TypesValType::Tuple(types) if types.len() == 2 => {
                wrt_format::component::ValType::ResultBoth(
                    Box::new(common_valtype_to_format_valtype(&types[0])),
                    Box::new(common_valtype_to_format_valtype(&types[1])),
                )
            }
            // Otherwise treat it as just the ok type
            _ => wrt_format::component::ValType::Result(Box::new(
                common_valtype_to_format_valtype(ok_type),
            )),
        },
        CommonValType::Tuple(types) => {
            let format_types = types
                .iter()
                .map(|val_type| common_valtype_to_format_valtype(val_type))
                .collect();
            wrt_format::component::ValType::Tuple(format_types)
        }
        CommonValType::Flags(names) => wrt_format::component::ValType::Flags(names.clone()),
        CommonValType::Own(idx) => wrt_format::component::ValType::Own(*idx),
        CommonValType::Borrow(idx) => wrt_format::component::ValType::Borrow(*idx),
    }
}

/// Value to ComponentValue conversion
pub fn value_to_component_value(value: &Value) -> Result<ComponentValue> {
    match value {
        Value::I32(val) => Ok(ComponentValue::S32(*val)),
        Value::I64(val) => Ok(ComponentValue::S64(*val)),
        Value::F32(val) => Ok(ComponentValue::F32(*val)),
        Value::F64(val) => Ok(ComponentValue::F64(*val)),
        Value::FuncRef(val) => {
            if val.is_none() {
                Ok(ComponentValue::Option(None)) // Null function reference
            } else {
                // Non-null function reference represented as a handle
                let func_ref = val.as_ref().unwrap();
                Ok(ComponentValue::Own(func_ref.index()))
            }
        }
        Value::ExternRef(val) => {
            if val.is_none() {
                Ok(ComponentValue::Option(None)) // Null external reference
            } else {
                // Represent as a string for compatibility
                let extern_ref = val.as_ref().unwrap();
                Ok(ComponentValue::String(format!(
                    "ref_{}",
                    extern_ref.index()
                )))
            }
        }
        Value::V128(_) => Err(Error::new(kinds::ConversionError(
            "V128 values not supported in Component Model".to_string(),
        ))),
    }
}

/// ComponentValue to Value conversion
pub fn component_value_to_value(cv: &ComponentValue) -> Result<Value> {
    match cv {
        ComponentValue::Bool(val) => Ok(Value::I32(*val as i32)),
        ComponentValue::S8(val) => Ok(Value::I32(*val as i32)),
        ComponentValue::U8(val) => Ok(Value::I32(*val as i32)),
        ComponentValue::S16(val) => Ok(Value::I32(*val as i32)),
        ComponentValue::U16(val) => Ok(Value::I32(*val as i32)),
        ComponentValue::S32(val) => Ok(Value::I32(*val)),
        ComponentValue::U32(val) => Ok(Value::I32(*val as i32)),
        ComponentValue::S64(val) => Ok(Value::I64(*val)),
        ComponentValue::U64(val) => Ok(Value::I64(*val as i64)),
        ComponentValue::F32(val) => Ok(Value::F32(*val)),
        ComponentValue::F64(val) => Ok(Value::F64(*val)),
        ComponentValue::Char(val) => Ok(Value::I32(*val as i32)),
        // For complex types, use ExternRef with the index of 1 (non-null reference)
        ComponentValue::String(_) => {
            Ok(Value::ExternRef(Some(wrt_types::values::ExternRef::new(1))))
        }
        ComponentValue::List(_) => Ok(Value::ExternRef(Some(wrt_types::values::ExternRef::new(1)))),
        ComponentValue::Record(_) => {
            Ok(Value::ExternRef(Some(wrt_types::values::ExternRef::new(1))))
        }
        ComponentValue::Variant { .. } => {
            Ok(Value::ExternRef(Some(wrt_types::values::ExternRef::new(1))))
        }
        ComponentValue::Tuple(_) => {
            Ok(Value::ExternRef(Some(wrt_types::values::ExternRef::new(1))))
        }
        ComponentValue::Flags(_) => Ok(Value::I32(0)), // Default flag value
        ComponentValue::Enum(val) => Ok(Value::I32(*val as i32)),
        ComponentValue::Option(opt) => match opt {
            Some(_) => Ok(Value::ExternRef(Some(wrt_types::values::ExternRef::new(1)))), // Non-null reference
            None => Ok(Value::ExternRef(None)), // Null reference
        },
        ComponentValue::Result(res) => match res {
            Ok(_) => Ok(Value::I32(1)),  // Success value
            Err(_) => Ok(Value::I32(0)), // Error value
        },
        ComponentValue::Own(handle) => Ok(Value::I32(*handle as i32)),
        ComponentValue::Borrow(handle) => Ok(Value::I32(*handle as i32)),
    }
}

/// Get the size in bytes of a ValType
fn size_in_bytes(ty: &ValType) -> usize {
    match ty {
        ValType::Bool => 1,
        ValType::S8 | ValType::U8 => 1,
        ValType::S16 | ValType::U16 => 2,
        ValType::S32 | ValType::U32 | ValType::F32 => 4,
        ValType::S64 | ValType::U64 | ValType::F64 => 8,
        ValType::Char => 4,             // Unicode code points
        ValType::String => 8,           // Reference to a string
        ValType::List(_) => 8,          // Reference to a list
        ValType::Record(_) => 8,        // Reference to a record
        ValType::Variant(_) => 8,       // Variant tag + payload
        ValType::Tuple(_) => 8,         // Reference to a tuple
        ValType::Flags(_) => 4,         // Flags are represented as integers
        ValType::Enum(_) => 4,          // Enum discriminants
        ValType::Option(_) => 8,        // Tag + payload
        ValType::Result(_) => 8,        // Tag + payload
        ValType::ResultErr(_) => 8,     // Tag + payload
        ValType::ResultBoth(_, _) => 8, // Tag + payload
        ValType::Own(_) => 4,           // Resource handle
        ValType::Borrow(_) => 4,        // Resource handle
        ValType::Ref(_) => 4,           // Reference
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_value_encoding_decoding() {
        let values = vec![
            (ComponentValue::Bool(true), ValType::Bool),
            (ComponentValue::S8(-42), ValType::S8),
            (ComponentValue::U8(42), ValType::U8),
            (ComponentValue::S16(-1000), ValType::S16),
            (ComponentValue::U16(1000), ValType::U16),
            (ComponentValue::S32(-100000), ValType::S32),
            (ComponentValue::U32(100000), ValType::U32),
            (ComponentValue::S64(-10000000000), ValType::S64),
            (ComponentValue::U64(10000000000), ValType::U64),
            (ComponentValue::F32(3.14159), ValType::F32),
            (ComponentValue::F64(2.71828), ValType::F64),
        ];

        for (value, ty) in values {
            let encoded = encode_component_value(&value, &ty).unwrap();
            let decoded = decode_component_value(&encoded, &ty).unwrap();
            assert_eq!(decoded, value);
        }
    }

    #[test]
    fn test_value_type_matching() {
        let value = ComponentValue::U32(42);
        assert!(value.matches_type(&ValType::U32));
        assert!(!value.matches_type(&ValType::S32));
        assert!(!value.matches_type(&ValType::U64));

        let list_value = ComponentValue::List(vec![
            ComponentValue::S32(1),
            ComponentValue::S32(2),
            ComponentValue::S32(3),
        ]);
        assert!(list_value.matches_type(&ValType::List(Box::new(ValType::S32))));
        assert!(!list_value.matches_type(&ValType::List(Box::new(ValType::U32))));
    }

    #[test]
    fn test_conversion_between_core_and_component() {
        let core_value = Value::I32(42);
        let component_value = core_to_component_value(&core_value, &ValType::S32).unwrap();
        assert_eq!(component_value, ComponentValue::S32(42));

        let core_value2 = component_to_core_value(&component_value).unwrap();
        assert_eq!(core_value2, core_value);
    }

    #[test]
    fn test_resource_handle_encoding_decoding() {
        let values = vec![
            (ComponentValue::Own(123), ValType::Own(456)),
            (ComponentValue::Borrow(789), ValType::Borrow(101112)),
        ];

        for (value, ty) in values {
            let encoded = encode_component_value(&value, &ty).unwrap();
            let decoded = decode_component_value(&encoded, &ty).unwrap();
            assert_eq!(decoded, value);
        }
    }

    #[test]
    fn test_resource_type_matching() {
        let own_value = ComponentValue::Own(42);
        assert!(own_value.matches_type(&ValType::Own(42)));
        assert!(!own_value.matches_type(&ValType::Own(43)));
        assert!(!own_value.matches_type(&ValType::Borrow(42)));

        let borrow_value = ComponentValue::Borrow(42);
        assert!(borrow_value.matches_type(&ValType::Borrow(42)));
        assert!(!borrow_value.matches_type(&ValType::Borrow(43)));
        assert!(!borrow_value.matches_type(&ValType::Own(42)));
    }
}
