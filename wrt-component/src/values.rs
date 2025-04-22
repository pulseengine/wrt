//! Component Model value handling
//!
//! This module provides implementations for Component Model value types, including
//! serialization/deserialization, conversion, and runtime representation.

use wrt_common::component::ComponentValue;
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
        case: u32,
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
            Self::Result(val) => ValType::Result(Box::new(if let Ok(ok) = val {
                if let Some(v) = ok {
                    v.get_type()
                } else {
                    ValType::Bool // Placeholder for None
                }
            } else {
                ValType::Bool // Placeholder for Err
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
                // Find the case in the variant type
                if let Some((_, case_type)) =
                    cases.iter().find(|(name, _)| name == &case.to_string())
                {
                    // Check if the value matches the case type
                    match (value, case_type) {
                        (Some(value), Some(ty)) => value.matches_type(ty),
                        (None, None) => true,
                        _ => false,
                    }
                } else {
                    false // Case not found
                }
            }

            (ComponentValue::Tuple(items), ValType::Tuple(item_types)) => {
                if items.len() != item_types.len() {
                    return false;
                }

                items
                    .iter()
                    .zip(item_types.iter())
                    .all(|(value, ty)| value.matches_type(ty))
            }

            (ComponentValue::Flags(flags), ValType::Flags(flag_names)) => {
                // Check if all flags in the type are present in the value
                if flags.len() != flag_names.len() {
                    return false;
                }

                flag_names.iter().all(|name| flags.contains_key(name))
            }

            (ComponentValue::Enum(value), ValType::Enum(variants)) => {
                variants.contains(&value.to_string())
            }

            (ComponentValue::Option(value), ValType::Option(option_type)) => {
                match (value, option_type) {
                    (Some(v), ty) => v.matches_type(ty),
                    (None, _) => true, // None matches any option type
                }
            }

            (ComponentValue::Result(val), ValType::Result(result_type)) => {
                match val {
                    Ok(ok) => match ok {
                        Some(v) => v.matches_type(result_type),
                        None => true, // Empty Ok matches any result type
                    },
                    Err(err) => match err {
                        Some(v) => v.matches_type(result_type),
                        None => true, // Empty Err matches any result type
                    },
                }
            }

            (ComponentValue::Own(handle), ValType::Own(id)) => *handle == *id,
            (ComponentValue::Borrow(handle), ValType::Borrow(id)) => *handle == *id,

            // Any other combination doesn't match
            _ => false,
        }
    }
}

/// Encode a component value to bytes
pub fn encode_component_value(value: &ComponentValue, ty: &ValType) -> Result<Vec<u8>> {
    match (value, ty) {
        (ComponentValue::Bool(v), ValType::Bool) => Ok(vec![if *v { 1 } else { 0 }]),
        (ComponentValue::S8(v), ValType::S8) => Ok(vec![*v as u8]),
        (ComponentValue::U8(v), ValType::U8) => Ok(vec![*v]),
        (ComponentValue::S16(v), ValType::S16) => Ok(v.to_le_bytes().to_vec()),
        (ComponentValue::U16(v), ValType::U16) => Ok(v.to_le_bytes().to_vec()),
        (ComponentValue::S32(v), ValType::S32) => Ok(v.to_le_bytes().to_vec()),
        (ComponentValue::U32(v), ValType::U32) => Ok(v.to_le_bytes().to_vec()),
        (ComponentValue::S64(v), ValType::S64) => Ok(v.to_le_bytes().to_vec()),
        (ComponentValue::U64(v), ValType::U64) => Ok(v.to_le_bytes().to_vec()),
        (ComponentValue::F32(v), ValType::F32) => Ok(v.to_le_bytes().to_vec()),
        (ComponentValue::F64(v), ValType::F64) => Ok(v.to_le_bytes().to_vec()),
        (ComponentValue::Char(v), ValType::Char) => {
            let mut bytes = [0u8; 4];
            let len = v.encode_utf8(&mut bytes).len();
            Ok(bytes[..len].to_vec())
        }
        (ComponentValue::String(v), ValType::String) => Ok(v.as_bytes().to_vec()),
        (ComponentValue::List(items), ValType::List(elem_type)) => {
            let mut buffer = vec![];
            let count = items.len() as u32;
            buffer.extend_from_slice(&count.to_le_bytes());
            for item in items {
                buffer.extend_from_slice(&encode_component_value(item, elem_type)?);
            }
            Ok(buffer)
        }
        (ComponentValue::Record(fields), ValType::Record(field_types)) => {
            let mut buffer = vec![];
            for (name, field_type) in field_types {
                if let Some(field_value) = fields.get(name) {
                    buffer.extend_from_slice(&encode_component_value(field_value, field_type)?);
                } else {
                    return Err(Error::invalid_data(format!(
                        "Field {} not found in record",
                        name
                    )));
                }
            }
            Ok(buffer)
        }
        (ComponentValue::Variant { case, value }, ValType::Variant(cases)) => {
            let mut buffer = vec![];
            buffer.extend_from_slice(&case.to_le_bytes());
            if let Some(case_val) = value {
                if let Some((_, Some(case_type))) = cases.get(*case as usize) {
                    buffer.extend_from_slice(&encode_component_value(case_val, case_type)?);
                }
            }
            Ok(buffer)
        }
        (ComponentValue::Tuple(items), ValType::Tuple(types)) => {
            let mut buffer = vec![];
            if items.len() != types.len() {
                return Err(Error::invalid_data("Tuple length mismatch"));
            }
            for (item, item_type) in items.iter().zip(types.iter()) {
                buffer.extend_from_slice(&encode_component_value(item, item_type)?);
            }
            Ok(buffer)
        }
        (ComponentValue::Flags(flags), ValType::Flags(names)) => {
            let total_bytes = (names.len() + 7) / 8; // Ceiling division
            let mut buffer = vec![0u8; total_bytes];
            for (i, name) in names.iter().enumerate() {
                if let Some(flag) = flags.get(name) {
                    if *flag {
                        buffer[i / 8] |= 1 << (i % 8);
                    }
                }
            }
            Ok(buffer)
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
                    buffer.extend_from_slice(&encode_component_value(v, inner_type)?);
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
                        buffer.extend_from_slice(&encode_component_value(value, result_type)?);
                    }
                }
                Err(v) => {
                    buffer.push(0); // Err tag
                    if let Some(value) = v {
                        buffer.extend_from_slice(&encode_component_value(value, result_type)?);
                    }
                }
            }
            Ok(buffer)
        }
        (ComponentValue::Own(handle), ValType::Own(_)) => {
            let mut buffer = handle.to_le_bytes().to_vec();
            Ok(buffer)
        }
        (ComponentValue::Borrow(handle), ValType::Borrow(_)) => {
            let mut buffer = handle.to_le_bytes().to_vec();
            Ok(buffer)
        }
        _ => Err(Error::type_mismatch_error(format!(
            "Cannot encode value {:?} as type {:?}",
            value, ty
        ))),
    }
}

/// Decode bytes to a component value
pub fn decode_component_value(data: &[u8], ty: &ValType) -> Result<ComponentValue> {
    match ty {
        ValType::Bool => {
            if data.is_empty() {
                return Err(Error::invalid_data("Empty data for bool value"));
            }
            Ok(ComponentValue::Bool(data[0] != 0))
        }
        ValType::S8 => {
            if data.is_empty() {
                return Err(Error::invalid_data("Empty data for s8 value"));
            }
            Ok(ComponentValue::S8(data[0] as i8))
        }
        ValType::U8 => {
            if data.is_empty() {
                return Err(Error::invalid_data("Empty data for u8 value"));
            }
            Ok(ComponentValue::U8(data[0]))
        }
        ValType::S16 => {
            if data.len() < 2 {
                return Err(Error::invalid_data("Insufficient data for s16 value"));
            }
            let mut bytes = [0u8; 2];
            bytes.copy_from_slice(&data[0..2]);
            Ok(ComponentValue::S16(i16::from_le_bytes(bytes)))
        }
        ValType::U16 => {
            if data.len() < 2 {
                return Err(Error::invalid_data("Insufficient data for u16 value"));
            }
            let mut bytes = [0u8; 2];
            bytes.copy_from_slice(&data[0..2]);
            Ok(ComponentValue::U16(u16::from_le_bytes(bytes)))
        }
        ValType::S32 => {
            if data.len() < 4 {
                return Err(Error::invalid_data("Insufficient data for s32 value"));
            }
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[0..4]);
            Ok(ComponentValue::S32(i32::from_le_bytes(bytes)))
        }
        ValType::U32 => {
            if data.len() < 4 {
                return Err(Error::invalid_data("Insufficient data for u32 value"));
            }
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[0..4]);
            Ok(ComponentValue::U32(u32::from_le_bytes(bytes)))
        }
        ValType::S64 => {
            if data.len() < 8 {
                return Err(Error::invalid_data("Insufficient data for s64 value"));
            }
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&data[0..8]);
            Ok(ComponentValue::S64(i64::from_le_bytes(bytes)))
        }
        ValType::U64 => {
            if data.len() < 8 {
                return Err(Error::invalid_data("Insufficient data for u64 value"));
            }
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&data[0..8]);
            Ok(ComponentValue::U64(u64::from_le_bytes(bytes)))
        }
        ValType::F32 => {
            if data.len() < 4 {
                return Err(Error::invalid_data("Insufficient data for f32 value"));
            }
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[0..4]);
            Ok(ComponentValue::F32(f32::from_le_bytes(bytes)))
        }
        ValType::F64 => {
            if data.len() < 8 {
                return Err(Error::invalid_data("Insufficient data for f64 value"));
            }
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&data[0..8]);
            Ok(ComponentValue::F64(f64::from_le_bytes(bytes)))
        }
        ValType::Char => {
            if data.is_empty() {
                return Err(Error::invalid_data("Empty data for char value"));
            }
            // Handle UTF-8 characters
            let s = String::from_utf8_lossy(data);
            let mut chars = s.chars();
            let c = chars
                .next()
                .ok_or_else(|| Error::invalid_data("Invalid UTF-8 sequence"))?;

            // Check that we only have one character
            if chars.next().is_some() {
                return Err(Error::invalid_data("Multiple characters in char value"));
            }

            Ok(ComponentValue::Char(c))
        }
        ValType::String => {
            let result = String::from_utf8_lossy(data).to_string();
            Ok(ComponentValue::String(result))
        }
        ValType::List(elem_type) => {
            if data.len() < 4 {
                return Err(Error::invalid_data("Insufficient data for list length"));
            }

            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[0..4]);
            let count = u32::from_le_bytes(bytes) as usize;

            let mut items = Vec::with_capacity(count);
            let mut offset = 4;

            for _ in 0..count {
                if offset >= data.len() {
                    return Err(Error::invalid_data("Unexpected end of data"));
                }

                let elem_data = &data[offset..];
                let elem = decode_component_value(elem_data, elem_type)?;
                items.push(elem);

                // Update offset based on the size of the element
                offset += elem_type.size_bytes();
            }

            Ok(ComponentValue::List(items))
        }
        ValType::Record(field_types) => {
            let mut fields = HashMap::new();
            let mut offset = 0;

            for (name, field_type) in field_types {
                if offset >= data.len() {
                    return Err(Error::invalid_data(format!(
                        "Record missing field {}",
                        name
                    )));
                }

                let field_data = &data[offset..];
                let field_value = decode_component_value(field_data, field_type)?;
                fields.insert(name.clone(), field_value);

                // Update offset based on the size of the field
                offset += field_type.size_bytes();
            }

            Ok(ComponentValue::Record(fields))
        }
        ValType::Variant(cases) => {
            if data.len() < 4 {
                return Err(Error::invalid_data("Insufficient data for variant case"));
            }

            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[0..4]);
            let case = u32::from_le_bytes(bytes);

            if case as usize >= cases.len() {
                return Err(Error::invalid_data(format!(
                    "Invalid variant case: {}",
                    case
                )));
            }

            let (case_name, case_type) = &cases[case as usize];
            let value = if let Some(case_type) = case_type {
                if data.len() < 4 + case_type.size_bytes() {
                    return Err(Error::invalid_data(format!(
                        "Insufficient data for variant case {}",
                        case_name
                    )));
                }

                let value_data = &data[4..];
                Some(Box::new(decode_component_value(value_data, case_type)?))
            } else {
                None
            };

            Ok(ComponentValue::Variant { case, value })
        }
        ValType::Tuple(types) => {
            let mut items = Vec::with_capacity(types.len());
            let mut offset = 0;

            for item_type in types {
                if offset >= data.len() {
                    return Err(Error::invalid_data("Tuple missing elements"));
                }

                let item_data = &data[offset..];
                let item = decode_component_value(item_data, item_type)?;
                items.push(item);

                // Update offset based on the size of the item
                offset += item_type.size_bytes();
            }

            Ok(ComponentValue::Tuple(items))
        }
        ValType::Flags(names) => {
            let total_bytes = (names.len() + 7) / 8; // Ceiling division
            if data.len() < total_bytes {
                return Err(Error::invalid_data("Insufficient data for flags"));
            }

            let mut flags = HashMap::new();
            for (i, name) in names.iter().enumerate() {
                let byte_index = i / 8;
                let bit_index = i % 8;
                let flag_value = (data[byte_index] & (1 << bit_index)) != 0;
                flags.insert(name.clone(), flag_value);
            }

            Ok(ComponentValue::Flags(flags))
        }
        ValType::Enum(variants) => {
            if data.len() < 4 {
                return Err(Error::invalid_data("Insufficient data for enum value"));
            }

            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[0..4]);
            let case_name = u32::from_le_bytes(bytes);

            if case_name as usize >= variants.len() {
                return Err(Error::invalid_data(format!(
                    "Invalid enum variant: {}",
                    case_name
                )));
            }

            Ok(ComponentValue::Enum(case_name))
        }
        ValType::Option(inner_type) => {
            if data.is_empty() {
                return Err(Error::invalid_data("Empty data for option value"));
            }

            let tag = data[0];
            let value = if tag == 0 {
                // None
                None
            } else {
                // Some
                if data.len() < 1 + inner_type.size_bytes() {
                    return Err(Error::invalid_data("Insufficient data for option value"));
                }

                let value_data = &data[1..];
                Some(Box::new(decode_component_value(value_data, inner_type)?))
            };

            Ok(ComponentValue::Option(value))
        }
        ValType::Result(result_type) => {
            if data.is_empty() {
                return Err(Error::invalid_data("Empty data for result value"));
            }

            let tag = data[0];
            let value = if tag == 0 {
                // Err
                let error = if data.len() > 1 {
                    let error_data = &data[1..];
                    Some(Box::new(decode_component_value(error_data, result_type)?))
                } else {
                    None
                };
                Err(error)
            } else {
                // Ok
                let ok = if data.len() > 1 {
                    let ok_data = &data[1..];
                    Some(Box::new(decode_component_value(ok_data, result_type)?))
                } else {
                    None
                };
                Ok(ok)
            };

            Ok(ComponentValue::Result(value))
        }
        ValType::Own(_) => {
            if data.len() < 4 {
                return Err(Error::invalid_data("Insufficient data for handle"));
            }

            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[0..4]);
            let handle = u32::from_le_bytes(bytes);

            Ok(ComponentValue::Own(handle))
        }
        ValType::Borrow(_) => {
            if data.len() < 4 {
                return Err(Error::invalid_data("Insufficient data for handle"));
            }

            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[0..4]);
            let handle = u32::from_le_bytes(bytes);

            Ok(ComponentValue::Borrow(handle))
        }
    }
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

/// Serialize a vector of ComponentValue to bytes
pub fn serialize_component_values(values: &[ComponentValue]) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();

    // Write the number of values
    buffer.extend_from_slice(&(values.len() as u32).to_le_bytes());

    // For each value, encode its type and value
    for value in values {
        let value_type = value.get_type();
        let value_buffer = encode_component_value(value, &value_type)?;

        // Write the type
        let type_buffer = serialize_val_type(&value_type)?;
        buffer.extend_from_slice(&type_buffer);

        // Write the value size and data
        buffer.extend_from_slice(&(value_buffer.len() as u32).to_le_bytes());
        buffer.extend_from_slice(&value_buffer);
    }

    Ok(buffer)
}

/// Deserialize bytes to a vector of ComponentValue
pub fn deserialize_component_values(
    data: &[u8],
    types: &[wrt_format::component::ValType],
) -> Result<Vec<ComponentValue>> {
    if types.is_empty() {
        return Ok(Vec::new());
    }

    let mut values = Vec::with_capacity(types.len());
    let mut offset = 0;

    for ty in types {
        let value = deserialize_component_value(data, &mut offset, ty)?;
        values.push(value);
    }

    Ok(values)
}

/// Deserialize a single ComponentValue
fn deserialize_component_value(
    data: &[u8],
    offset: &mut usize,
    ty: &wrt_format::component::ValType,
) -> Result<ComponentValue> {
    if *offset >= data.len() {
        return Err(Error::invalid_data("Unexpected end of data"));
    }

    // Extract a slice with the rest of the data
    let value_data = &data[*offset..];

    // Decode the value
    let value = decode_component_value(value_data, ty)?;

    // Update the offset
    *offset += ty.size_bytes();

    Ok(value)
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
        ValType::Own(_) => buffer.push(21),
        ValType::Borrow(_) => buffer.push(22),
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
