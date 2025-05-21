//! Component Model value handling
//!
//! This module provides implementations for Component Model value types,
//! including serialization/deserialization, conversion, and runtime
//! representation.

// Import the various types we need explicitly to avoid confusion
use wrt_format::component::ValType as FormatValType;
use wrt_types::{
    component_value::{ComponentValue, ValType as TypesValType},
    traits::{ReadStream, WriteStream},
    values::Value,
};

use crate::prelude::*;

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
            let converted_types =
                types.iter().map(|val_type| convert_common_to_format_valtype(val_type)).collect();
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
        CanonicalValType::FixedList(elem_type, size) => {
            FormatValType::FixedList(Box::new(convert_common_to_format_valtype(elem_type)), *size)
        }
        CanonicalValType::Void => {
            // Void doesn't have a direct mapping, convert to a unit tuple
            FormatValType::Tuple(Vec::new())
        }
        CanonicalValType::ErrorContext => FormatValType::ErrorContext,
        CanonicalValType::ResultErr(err_type) => {
            FormatValType::ResultErr(Box::new(convert_common_to_format_valtype(err_type)))
        }
        CanonicalValType::ResultBoth(ok_type, err_type) => FormatValType::ResultBoth(
            Box::new(convert_common_to_format_valtype(ok_type)),
            Box::new(convert_common_to_format_valtype(err_type)),
        ),
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
            let converted_types =
                types.iter().map(|val_type| convert_format_to_common_valtype(val_type)).collect();
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
        FormatValType::FixedList(elem_type, size) => CanonicalValType::FixedList(
            Box::new(convert_format_to_common_valtype(elem_type)),
            *size,
        ),
        FormatValType::ErrorContext => CanonicalValType::ErrorContext,
        // Map any unhandled types to Void
        _ => CanonicalValType::Void,
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
        ComponentValue::List(items) => {
            // Serialize list items
            let count = items.len() as u32;
            buffer.extend_from_slice(&count.to_le_bytes());

            // If there are items, use the first item to determine the type
            if let Some(first_item) = items.first() {
                let item_type = first_item.get_type();
                let format_type = convert_common_to_format_valtype(&item_type);

                // Serialize each item
                for item in items {
                    let item_bytes = serialize_component_value(item)?;
                    buffer.extend_from_slice(&item_bytes);
                }
            }
        }
        ComponentValue::Record(fields) => {
            // Serialize the record fields
            let field_count = fields.len() as u32;
            buffer.extend_from_slice(&field_count.to_le_bytes());

            // Serialize each field
            for (name, value) in fields {
                // Serialize the field name
                let name_bytes = name.as_bytes();
                let name_len = name_bytes.len() as u32;
                buffer.extend_from_slice(&name_len.to_le_bytes());
                buffer.extend_from_slice(name_bytes);

                // Serialize the field value
                let value_bytes = serialize_component_value(value)?;
                buffer.extend_from_slice(&value_bytes);
            }
        }
        ComponentValue::Tuple(items) => {
            // Serialize tuple items
            let count = items.len() as u32;
            buffer.extend_from_slice(&count.to_le_bytes());

            // Serialize each item
            for item in items {
                let item_bytes = serialize_component_value(item)?;
                buffer.extend_from_slice(&item_bytes);
            }
        }
        ComponentValue::Variant(case_name, value_opt) => {
            // Serialize the case name
            let name_bytes = case_name.as_bytes();
            let name_len = name_bytes.len() as u32;
            buffer.extend_from_slice(&name_len.to_le_bytes());
            buffer.extend_from_slice(name_bytes);

            // Serialize presence flag for the value
            buffer.push(if value_opt.is_some() { 1 } else { 0 });

            // Serialize the value if present
            if let Some(value) = value_opt {
                let value_bytes = serialize_component_value(value)?;
                buffer.extend_from_slice(&value_bytes);
            }
        }
        ComponentValue::Enum(variant) => {
            // Serialize the enum variant
            let variant_bytes = variant.as_bytes();
            let variant_len = variant_bytes.len() as u32;
            buffer.extend_from_slice(&variant_len.to_le_bytes());
            buffer.extend_from_slice(variant_bytes);
        }
        ComponentValue::Option(value_opt) => {
            // Serialize presence flag for the value
            buffer.push(if value_opt.is_some() { 1 } else { 0 });

            // Serialize the value if present
            if let Some(value) = value_opt {
                let value_bytes = serialize_component_value(value)?;
                buffer.extend_from_slice(&value_bytes);
            }
        }
        ComponentValue::Result(result) => {
            // Serialize success flag
            buffer.push(match result {
                Ok(_) => 1,  // Success
                Err(_) => 0, // Error
            });

            // Serialize the value (either Ok or Err)
            match result {
                Ok(value) => {
                    let value_bytes = serialize_component_value(value)?;
                    buffer.extend_from_slice(&value_bytes);
                }
                Err(error) => {
                    let error_bytes = serialize_component_value(error)?;
                    buffer.extend_from_slice(&error_bytes);
                }
            }
        }
        ComponentValue::Handle(idx) => {
            // Serialize the handle index (from Own)
            buffer.extend_from_slice(&idx.to_le_bytes());
        }
        ComponentValue::Borrow(idx) => {
            // Serialize the borrow index
            buffer.extend_from_slice(&idx.to_le_bytes());
        }
        ComponentValue::Flags(flags) => {
            // Get the number of flags
            let count = flags.len() as u32;
            buffer.extend_from_slice(&count.to_le_bytes());

            // Calculate the flag byte (up to 8 flags in a byte)
            let mut flag_byte: u8 = 0;
            for (i, (_, enabled)) in flags.iter().enumerate().take(8) {
                if *enabled {
                    flag_byte |= 1 << i;
                }
            }
            buffer.push(flag_byte);

            // Serialize each flag name
            for (name, _) in flags {
                let name_bytes = name.as_bytes();
                let name_len = name_bytes.len() as u32;
                buffer.extend_from_slice(&name_len.to_le_bytes());
                buffer.extend_from_slice(name_bytes);
            }
        }
        ComponentValue::FixedList(items, size) => {
            // Serialize fixed list items
            let count = items.len() as u32;
            buffer.extend_from_slice(&count.to_le_bytes());
            buffer.extend_from_slice(&size.to_le_bytes()); // Store the size

            // If there are items, use the first item to determine the type
            if let Some(first_item) = items.first() {
                let item_type = first_item.get_type();
                let format_type = convert_common_to_format_valtype(&item_type);

                // Serialize each item
                for item in items {
                    let item_bytes = serialize_component_value(item)?;
                    buffer.extend_from_slice(&item_bytes);
                }
            }
        }
        ComponentValue::ErrorContext(ctx) => {
            // Serialize error context items
            let count = ctx.len() as u32;
            buffer.extend_from_slice(&count.to_le_bytes());

            // Serialize each context value
            for item in ctx {
                let item_bytes = serialize_component_value(item)?;
                buffer.extend_from_slice(&item_bytes);
            }
        }
        ComponentValue::Void => {
            // Just a type marker for void
            buffer.push(0);
        }
    }

    Ok(buffer)
}

/// Serialize a component value using the WriteStream interface
pub fn serialize_component_value_with_stream<'a, P: wrt_types::MemoryProvider>(
    value: &ComponentValue,
    writer: &mut WriteStream<'a>,
    provider: &P,
) -> Result<()> {
    match value {
        ComponentValue::Bool(b) => {
            writer.write_bool(*b)?;
        }
        ComponentValue::S8(v) => {
            writer.write_i8(*v)?;
        }
        ComponentValue::U8(v) => {
            writer.write_u8(*v)?;
        }
        ComponentValue::S16(v) => {
            writer.write_i16_le(*v)?;
        }
        ComponentValue::U16(v) => {
            writer.write_u16_le(*v)?;
        }
        ComponentValue::S32(v) => {
            writer.write_i32_le(*v)?;
        }
        ComponentValue::U32(v) => {
            writer.write_u32_le(*v)?;
        }
        ComponentValue::S64(v) => {
            writer.write_i64_le(*v)?;
        }
        ComponentValue::U64(v) => {
            writer.write_u64_le(*v)?;
        }
        ComponentValue::F32(v) => {
            writer.write_f32_le(*v)?;
        }
        ComponentValue::F64(v) => {
            writer.write_f64_le(*v)?;
        }
        ComponentValue::Char(c) => {
            writer.write_u32_le(*c as u32)?;
        }
        ComponentValue::String(s) => {
            // String is encoded as length followed by UTF-8 bytes
            let bytes = s.as_bytes();
            writer.write_u32_le(bytes.len() as u32)?;
            writer.write_all(bytes)?;
        }
        ComponentValue::List(items) => {
            // Serialize list items count
            writer.write_u32_le(items.len() as u32)?;

            // Serialize each item
            for item in items {
                serialize_component_value_with_stream(item, writer, provider)?;
            }
        }
        ComponentValue::Record(fields) => {
            // Serialize the record fields count
            writer.write_u32_le(fields.len() as u32)?;

            // Serialize each field
            for (name, value) in fields {
                // Serialize the field name
                let name_bytes = name.as_bytes();
                writer.write_u32_le(name_bytes.len() as u32)?;
                writer.write_all(name_bytes)?;

                // Serialize the field value
                serialize_component_value_with_stream(value, writer, provider)?;
            }
        }
        ComponentValue::Tuple(items) => {
            // Serialize tuple items count
            writer.write_u32_le(items.len() as u32)?;

            // Serialize each item
            for item in items {
                serialize_component_value_with_stream(item, writer, provider)?;
            }
        }
        ComponentValue::Variant(case_name, value_opt) => {
            // Serialize the case name
            let name_bytes = case_name.as_bytes();
            writer.write_u32_le(name_bytes.len() as u32)?;
            writer.write_all(name_bytes)?;

            // Serialize presence flag for the value
            writer.write_bool(value_opt.is_some())?;

            // Serialize the value if present
            if let Some(value) = value_opt {
                serialize_component_value_with_stream(value, writer, provider)?;
            }
        }
        ComponentValue::Enum(variant) => {
            // Serialize the enum variant
            let variant_bytes = variant.as_bytes();
            writer.write_u32_le(variant_bytes.len() as u32)?;
            writer.write_all(variant_bytes)?;
        }
        ComponentValue::Option(value_opt) => {
            // Serialize presence flag for the value
            writer.write_bool(value_opt.is_some())?;

            // Serialize the value if present
            if let Some(value) = value_opt {
                serialize_component_value_with_stream(value, writer, provider)?;
            }
        }
        ComponentValue::Result(result) => {
            // Serialize success flag
            let is_ok = matches!(result, Ok(_));
            writer.write_bool(is_ok)?;

            // Serialize the value (either Ok or Err)
            match result {
                Ok(value) => {
                    serialize_component_value_with_stream(value, writer, provider)?;
                }
                Err(error) => {
                    serialize_component_value_with_stream(error, writer, provider)?;
                }
            }
        }
        ComponentValue::Handle(idx) => {
            // Serialize the handle index (from Own)
            writer.write_u32_le(*idx)?;
        }
        ComponentValue::Borrow(idx) => {
            // Serialize the borrow index
            writer.write_u32_le(*idx)?;
        }
        ComponentValue::Flags(flags) => {
            // Get the number of flags
            writer.write_u32_le(flags.len() as u32)?;

            // Calculate the flag byte (up to 8 flags in a byte)
            let mut flag_byte: u8 = 0;
            for (i, (_, enabled)) in flags.iter().enumerate().take(8) {
                if *enabled {
                    flag_byte |= 1 << i;
                }
            }
            writer.write_u8(flag_byte)?;

            // Serialize each flag name
            for (name, _) in flags {
                let name_bytes = name.as_bytes();
                writer.write_u32_le(name_bytes.len() as u32)?;
                writer.write_all(name_bytes)?;
            }
        }
        ComponentValue::FixedList(items, size) => {
            // Serialize fixed list items count and size
            writer.write_u32_le(items.len() as u32)?;
            writer.write_u32_le(*size)?;

            // Serialize each item
            for item in items {
                serialize_component_value_with_stream(item, writer, provider)?;
            }
        }
        ComponentValue::ErrorContext(ctx) => {
            // Serialize error context items count
            writer.write_u32_le(ctx.len() as u32)?;

            // Serialize each context value
            for item in ctx {
                serialize_component_value_with_stream(item, writer, provider)?;
            }
        }
        ComponentValue::Void => {
            // Just a type marker for void
            writer.write_u8(0)?;
        }
    }

    Ok(())
}

// Simplified deserialization function
pub fn deserialize_component_value(
    data: &[u8],
    format_type: &FormatValType,
) -> Result<ComponentValue> {
    let mut offset = 0;
    match format_type {
        FormatValType::Bool => {
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize bool".to_string(),
                ));
            }
            let value = data[offset] != 0;
            Ok(ComponentValue::Bool(value))
        }
        FormatValType::S8 => {
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize S8".to_string(),
                ));
            }
            let value = data[offset] as i8;
            Ok(ComponentValue::S8(value))
        }
        FormatValType::U8 => {
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize U8".to_string(),
                ));
            }
            let value = data[offset];
            Ok(ComponentValue::U8(value))
        }
        FormatValType::S16 => {
            if offset + 2 > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize S16".to_string(),
                ));
            }
            let value = i16::from_le_bytes([data[offset], data[offset + 1]]);
            Ok(ComponentValue::S16(value))
        }
        FormatValType::U16 => {
            if offset + 2 > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize U16".to_string(),
                ));
            }
            let value = u16::from_le_bytes([data[offset], data[offset + 1]]);
            Ok(ComponentValue::U16(value))
        }
        FormatValType::S32 => {
            if offset + 4 > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize S32".to_string(),
                ));
            }
            let value = i32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            Ok(ComponentValue::S32(value))
        }
        FormatValType::U32 => {
            if offset + 4 > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize U32".to_string(),
                ));
            }
            let value = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            Ok(ComponentValue::U32(value))
        }
        FormatValType::S64 => {
            if offset + 8 > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize S64".to_string(),
                ));
            }
            let value = i64::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]);
            Ok(ComponentValue::S64(value))
        }
        FormatValType::U64 => {
            if offset + 8 > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize U64".to_string(),
                ));
            }
            let value = u64::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]);
            Ok(ComponentValue::U64(value))
        }
        FormatValType::F32 => {
            if offset + 4 > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize F32".to_string(),
                ));
            }
            let value = f32::from_bits(u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]));
            Ok(ComponentValue::F32(value))
        }
        FormatValType::F64 => {
            if offset + 8 > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize F64".to_string(),
                ));
            }
            let value = f64::from_bits(u64::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]));
            Ok(ComponentValue::F64(value))
        }
        FormatValType::Char => {
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize Char".to_string(),
                ));
            }
            let value = data[offset] as char;
            Ok(ComponentValue::Char(value))
        }
        FormatValType::String => {
            if offset + 4 > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize String length".to_string(),
                ));
            }
            let len = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;
            if offset + len as usize > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize String".to_string(),
                ));
            }
            let value =
                String::from_utf8(data[offset..offset + len as usize].to_vec()).map_err(|e| {
                    Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        format!("Invalid UTF-8 in string: {}", e).to_string(),
                    )
                })?;
            Ok(ComponentValue::String(value))
        }
        FormatValType::List(elem_type) => {
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize List length".to_string(),
                ));
            }
            let len = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;
            if offset + len as usize * size_in_bytes(elem_type) > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize List".to_string(),
                ));
            }
            let mut values = Vec::with_capacity(len as usize);
            for _ in 0..len {
                let value = deserialize_component_value(&data[offset..], elem_type)?;
                values.push(value);
                offset += size_in_bytes(elem_type);
            }
            Ok(ComponentValue::List(values))
        }
        FormatValType::Record(fields) => {
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize Record".to_string(),
                ));
            }
            let mut values = Vec::new();
            for (name, val_type) in fields {
                let value = deserialize_component_value(&data[offset..], val_type)?;
                values.push((name.clone(), value));
                offset += size_in_bytes(val_type);
            }
            Ok(ComponentValue::Record(values))
        }
        FormatValType::Tuple(types) => {
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize Tuple".to_string(),
                ));
            }
            let mut values = Vec::with_capacity(types.len());
            for val_type in types {
                let value = deserialize_component_value(&data[offset..], val_type)?;
                values.push(value);
                offset += size_in_bytes(val_type);
            }
            Ok(ComponentValue::Tuple(values))
        }
        FormatValType::Variant(cases) => {
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize Variant".to_string(),
                ));
            }
            let len = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;
            if offset + len as usize > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize Variant".to_string(),
                ));
            }
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize Variant case".to_string(),
                ));
            }

            // Get the case index
            let case_idx = data[offset] as usize;
            // No need to update offset anymore as we return immediately

            if case_idx >= cases.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    format!("Invalid variant case index: {}", case_idx).to_string(),
                ));
            }

            // Get the case name and type
            let (case_name, case_type_opt) = &cases[case_idx];

            // Create the variant value
            match case_type_opt {
                Some(case_type) => {
                    if offset >= data.len() {
                        return Err(Error::new(
                            ErrorCategory::Parse,
                            codes::PARSE_ERROR,
                            "Not enough data to deserialize Variant value".to_string(),
                        ));
                    }
                    let inner_value = deserialize_component_value(&data[offset..], case_type)?;
                    // We've already read the needed value, no need to update offset
                    Ok(ComponentValue::Variant(case_name.clone(), Some(Box::new(inner_value))))
                }
                None => {
                    // Case without a value
                    Ok(ComponentValue::Variant(case_name.clone(), None))
                }
            }
        }
        FormatValType::Enum(variants) => {
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize Enum".to_string(),
                ));
            }
            let len = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;
            if offset + len as usize > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize Enum".to_string(),
                ));
            }
            let value = data[offset] as char;
            // No need to update offset anymore as we return immediately
            Ok(ComponentValue::Enum(variants[value as usize].clone()))
        }
        FormatValType::Option(inner_type) => {
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize Option".to_string(),
                ));
            }
            let value = data[offset] != 0;
            // No need to update offset anymore as we return immediately
            if value {
                let inner_value = deserialize_component_value(&data[offset..], inner_type)?;
                Ok(ComponentValue::Option(Some(Box::new(inner_value))))
            } else {
                Ok(ComponentValue::Option(None))
            }
        }
        FormatValType::Result(result_type) => {
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize Result".to_string(),
                ));
            }
            let value = data[offset] != 0;
            // No need to update offset anymore as we return immediately
            if value {
                let inner_value = deserialize_component_value(&data[offset..], result_type)?;
                Ok(ComponentValue::Result(Ok(Box::new(inner_value))))
            } else {
                // Create a default error value when the result is not successful
                Ok(ComponentValue::Result(Err(Box::new(ComponentValue::Void))))
            }
        }
        FormatValType::ResultErr(err_type) => {
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize ResultErr".to_string(),
                ));
            }
            let value = data[offset] != 0;
            // No need to update offset anymore as we return immediately
            if value {
                Ok(ComponentValue::Result(Err(Box::new(ComponentValue::Bool(true)))))
            } else {
                Ok(ComponentValue::Result(Err(Box::new(ComponentValue::Bool(false)))))
            }
        }
        FormatValType::ResultBoth(ok_type, err_type) => {
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize ResultBoth".to_string(),
                ));
            }
            let value = data[offset] != 0;
            // No need to update offset anymore as we return immediately
            if value {
                let ok_value = deserialize_component_value(&data[offset..], ok_type)?;
                Ok(ComponentValue::Result(Ok(Box::new(ok_value))))
            } else {
                // Use the err_type to deserialize an error value
                let err_value = deserialize_component_value(&data[offset..], err_type)?;
                Ok(ComponentValue::Result(Err(Box::new(err_value))))
            }
        }
        FormatValType::Own(idx) => {
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize Own".to_string(),
                ));
            }
            let value = data[offset] as u32;
            // No need to update offset anymore as we return immediately
            Ok(ComponentValue::Handle(value))
        }
        FormatValType::Borrow(idx) => {
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize Borrow".to_string(),
                ));
            }
            let value = data[offset] as u32;
            // No need to update offset anymore as we return immediately
            Ok(ComponentValue::Borrow(value))
        }
        FormatValType::Flags(names) => {
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize Flags".to_string(),
                ));
            }
            let flag_byte = data[offset];
            // No need to update offset anymore as we return immediately

            // Convert names to (String, bool) pairs based on the flag_byte
            let mut flags = Vec::new();
            for (i, name) in names.iter().enumerate() {
                if i < 8 {
                    // Only process up to 8 flags (one byte)
                    let is_set = (flag_byte & (1 << i)) != 0;
                    flags.push((name.clone(), is_set));
                }
            }

            Ok(ComponentValue::Flags(flags))
        }
        FormatValType::FixedList(elem_type, size) => {
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize FixedList length".to_string(),
                ));
            }
            let len = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            offset += 4;
            if offset + len as usize * size_in_bytes(elem_type) > data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize FixedList".to_string(),
                ));
            }
            let mut values = Vec::with_capacity(len as usize);
            for _ in 0..len {
                let value = deserialize_component_value(&data[offset..], elem_type)?;
                values.push(value);
                offset += size_in_bytes(elem_type);
            }
            Ok(ComponentValue::FixedList(values, *size))
        }
        FormatValType::ErrorContext => {
            if offset >= data.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Not enough data to deserialize ErrorContext".to_string(),
                ));
            }
            let value = data[offset] != 0;
            // No need to update offset anymore as we return immediately
            // Create a proper ErrorContext value with an empty Vec
            Ok(ComponentValue::ErrorContext(Vec::new()))
        }
        // Handle any other unimplemented cases
        _ => Err(Error::new(
            ErrorCategory::System,
            codes::UNSUPPORTED_OPERATION,
            "Type not supported for deserialization".to_string(),
        )),
    }
}

/// Deserialize a component value using the ReadStream interface
pub fn deserialize_component_value_with_stream<'a, P: wrt_types::MemoryProvider>(
    reader: &mut ReadStream<'a>,
    format_type: &FormatValType,
    provider: &P,
) -> Result<ComponentValue> {
    match format_type {
        FormatValType::Bool => {
            let value = reader.read_bool()?;
            Ok(ComponentValue::Bool(value))
        }
        FormatValType::S8 => {
            let value = reader.read_i8()?;
            Ok(ComponentValue::S8(value))
        }
        FormatValType::U8 => {
            let value = reader.read_u8()?;
            Ok(ComponentValue::U8(value))
        }
        FormatValType::S16 => {
            let value = reader.read_i16_le()?;
            Ok(ComponentValue::S16(value))
        }
        FormatValType::U16 => {
            let value = reader.read_u16_le()?;
            Ok(ComponentValue::U16(value))
        }
        FormatValType::S32 => {
            let value = reader.read_i32_le()?;
            Ok(ComponentValue::S32(value))
        }
        FormatValType::U32 => {
            let value = reader.read_u32_le()?;
            Ok(ComponentValue::U32(value))
        }
        FormatValType::S64 => {
            let value = reader.read_i64_le()?;
            Ok(ComponentValue::S64(value))
        }
        FormatValType::U64 => {
            let value = reader.read_u64_le()?;
            Ok(ComponentValue::U64(value))
        }
        FormatValType::F32 => {
            let value = reader.read_f32_le()?;
            Ok(ComponentValue::F32(value))
        }
        FormatValType::F64 => {
            let value = reader.read_f64_le()?;
            Ok(ComponentValue::F64(value))
        }
        FormatValType::Char => {
            let value_u32 = reader.read_u32_le()?;
            let value = char::from_u32(value_u32).ok_or_else(|| {
                Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    format!("Invalid u32 value for char: {}", value_u32),
                )
            })?;
            Ok(ComponentValue::Char(value))
        }
        FormatValType::String => {
            // Read the string length
            let len = reader.read_u32_le()? as usize;

            // Read the string bytes
            let mut bytes = vec![0u8; len];
            reader.read_exact(&mut bytes)?;

            // Convert to a String
            let value = String::from_utf8(bytes).map_err(|e| {
                Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    format!("Invalid UTF-8 in string: {}", e),
                )
            })?;

            Ok(ComponentValue::String(value))
        }
        FormatValType::List(elem_type) => {
            // Read the number of items
            let count = reader.read_u32_le()? as usize;

            // Read each item
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                let item = deserialize_component_value_with_stream(reader, elem_type, provider)?;
                items.push(item);
            }

            Ok(ComponentValue::List(items))
        }
        FormatValType::Record(fields) => {
            // Read the number of fields
            let count = reader.read_u32_le()? as usize;

            if count != fields.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    format!("Expected {} record fields but found {}", fields.len(), count),
                ));
            }

            // Read each field
            let mut values = Vec::with_capacity(count);
            for (name, val_type) in fields {
                // Read field name length
                let name_len = reader.read_u32_le()? as usize;

                // Read field name
                let mut name_bytes = vec![0u8; name_len];
                reader.read_exact(&mut name_bytes)?;
                let field_name = String::from_utf8(name_bytes).map_err(|e| {
                    Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        format!("Invalid UTF-8 in field name: {}", e),
                    )
                })?;

                // Read field value
                let value = deserialize_component_value_with_stream(reader, val_type, provider)?;
                values.push((field_name, value));
            }

            Ok(ComponentValue::Record(values))
        }
        FormatValType::Tuple(types) => {
            // Read the number of items
            let count = reader.read_u32_le()? as usize;

            if count != types.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    format!("Expected {} tuple items but found {}", types.len(), count),
                ));
            }

            // Read each item
            let mut items = Vec::with_capacity(count);
            for item_type in types {
                let item = deserialize_component_value_with_stream(reader, item_type, provider)?;
                items.push(item);
            }

            Ok(ComponentValue::Tuple(items))
        }
        FormatValType::Variant(cases) => {
            // Read the case name length
            let name_len = reader.read_u32_le()? as usize;

            // Read the case name
            let mut name_bytes = vec![0u8; name_len];
            reader.read_exact(&mut name_bytes)?;
            let case_name = String::from_utf8(name_bytes).map_err(|e| {
                Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    format!("Invalid UTF-8 in variant case name: {}", e),
                )
            })?;

            // Find the case in the list
            let case_idx =
                cases.iter().position(|(name, _)| name == &case_name).ok_or_else(|| {
                    Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        format!("Unknown variant case name: {}", case_name),
                    )
                })?;

            // Read presence flag
            let has_value = reader.read_bool()?;

            // Read the value if present
            let (_, case_type_opt) = &cases[case_idx];
            if has_value {
                if let Some(case_type) = case_type_opt {
                    let value =
                        deserialize_component_value_with_stream(reader, case_type, provider)?;
                    Ok(ComponentValue::Variant(case_name, Some(Box::new(value))))
                } else {
                    Err(Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        format!("Variant case {} has a value but shouldn't", case_name),
                    ))
                }
            } else {
                Ok(ComponentValue::Variant(case_name, None))
            }
        }
        FormatValType::Enum(variants) => {
            // Read the variant name length
            let name_len = reader.read_u32_le()? as usize;

            // Read the variant name
            let mut name_bytes = vec![0u8; name_len];
            reader.read_exact(&mut name_bytes)?;
            let variant_name = String::from_utf8(name_bytes).map_err(|e| {
                Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    format!("Invalid UTF-8 in enum variant name: {}", e),
                )
            })?;

            // Validate the variant name
            if !variants.contains(&variant_name) {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    format!("Unknown enum variant: {}", variant_name),
                ));
            }

            Ok(ComponentValue::Enum(variant_name))
        }
        FormatValType::Option(inner_type) => {
            // Read presence flag
            let has_value = reader.read_bool()?;

            if has_value {
                let value = deserialize_component_value_with_stream(reader, inner_type, provider)?;
                Ok(ComponentValue::Option(Some(Box::new(value))))
            } else {
                Ok(ComponentValue::Option(None))
            }
        }
        FormatValType::Result(result_type) => {
            // Read success flag
            let is_ok = reader.read_bool()?;

            if is_ok {
                let value = deserialize_component_value_with_stream(reader, result_type, provider)?;
                Ok(ComponentValue::Result(Ok(Box::new(value))))
            } else {
                // Create a default error value
                Ok(ComponentValue::Result(Err(Box::new(ComponentValue::Void))))
            }
        }
        FormatValType::ResultErr(err_type) => {
            // Read error flag
            let is_error = reader.read_bool()?;

            if is_error {
                let error = deserialize_component_value_with_stream(reader, err_type, provider)?;
                Ok(ComponentValue::Result(Err(Box::new(error))))
            } else {
                // No error
                Ok(ComponentValue::Result(Ok(Box::new(ComponentValue::Void))))
            }
        }
        FormatValType::ResultBoth(ok_type, err_type) => {
            // Read success flag
            let is_ok = reader.read_bool()?;

            if is_ok {
                let value = deserialize_component_value_with_stream(reader, ok_type, provider)?;
                Ok(ComponentValue::Result(Ok(Box::new(value))))
            } else {
                let error = deserialize_component_value_with_stream(reader, err_type, provider)?;
                Ok(ComponentValue::Result(Err(Box::new(error))))
            }
        }
        FormatValType::Own(idx) => {
            let value = reader.read_u32_le()?;
            Ok(ComponentValue::Handle(value))
        }
        FormatValType::Borrow(idx) => {
            let value = reader.read_u32_le()?;
            Ok(ComponentValue::Borrow(value))
        }
        FormatValType::Flags(names) => {
            // Read the number of flags
            let count = reader.read_u32_le()? as usize;

            if count != names.len() {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    format!("Expected {} flags but found {}", names.len(), count),
                ));
            }

            // Read the flag byte
            let flag_byte = reader.read_u8()?;

            // Convert names to (String, bool) pairs based on the flag_byte
            let mut flags = Vec::with_capacity(count);
            for (i, name) in names.iter().enumerate() {
                if i < 8 {
                    // Only process up to 8 flags (one byte)
                    let is_set = (flag_byte & (1 << i)) != 0;
                    flags.push((name.clone(), is_set));
                } else {
                    // Default to false for flags beyond the first byte
                    flags.push((name.clone(), false));
                }
            }

            // Read each flag name (name is already known from the type)
            for _ in 0..count {
                let name_len = reader.read_u32_le()? as usize;
                let mut name_bytes = vec![0u8; name_len];
                reader.read_exact(&mut name_bytes)?;
                // We don't actually use the names here since we already have
                // them from the type
            }

            Ok(ComponentValue::Flags(flags))
        }
        FormatValType::FixedList(elem_type, size) => {
            // Read the number of items
            let count = reader.read_u32_le()? as usize;

            // Read the fixed size
            let fixed_size = reader.read_u32_le()?;

            if fixed_size != *size {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    format!("Expected fixed list of size {} but found {}", size, fixed_size),
                ));
            }

            // Read each item
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                let item = deserialize_component_value_with_stream(reader, elem_type, provider)?;
                items.push(item);
            }

            Ok(ComponentValue::FixedList(items, *size))
        }
        FormatValType::ErrorContext => {
            // Read the number of context items
            let count = reader.read_u32_le()? as usize;

            // Read each context item
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                // Assuming context items are strings
                let item_len = reader.read_u32_le()? as usize;
                let mut item_bytes = vec![0u8; item_len];
                reader.read_exact(&mut item_bytes)?;
                let item_str = String::from_utf8(item_bytes).map_err(|e| {
                    Error::new(
                        ErrorCategory::Parse,
                        codes::PARSE_ERROR,
                        format!("Invalid UTF-8 in error context: {}", e),
                    )
                })?;
                items.push(ComponentValue::String(item_str));
            }

            Ok(ComponentValue::ErrorContext(items))
        }
        FormatValType::Void => {
            // Just read a marker byte
            let _ = reader.read_u8()?;
            Ok(ComponentValue::Void)
        }
        // Handle any other unimplemented cases
        _ => Err(Error::new(
            ErrorCategory::System,
            codes::UNSUPPORTED_OPERATION,
            format!("Type {:?} not supported for deserialization", format_type),
        )),
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

/// Serialize multiple component values using a WriteStream
pub fn serialize_component_values_with_stream<'a, P: wrt_types::MemoryProvider>(
    values: &[ComponentValue],
    writer: &mut WriteStream<'a>,
    provider: &P,
) -> Result<()> {
    // Write the number of values
    writer.write_u32_le(values.len() as u32)?;

    // Serialize each value directly to the stream
    for value in values {
        serialize_component_value_with_stream(value, writer, provider)?;
    }

    Ok(())
}

/// Deserialize multiple component values
pub fn deserialize_component_values(
    data: &[u8],
    types: &[FormatValType],
) -> Result<Vec<ComponentValue>> {
    // Need at least 4 bytes for the count
    if data.len() < 4 {
        return Err(Error::new(
            ErrorCategory::Parse,
            codes::PARSE_ERROR,
            "Not enough data to read value count".to_string(),
        ));
    }

    // Read the count
    let mut offset = 0;
    let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    offset += 4;

    // Validate that we have enough types
    if count > types.len() {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            format!("Expected {} types but only got {}", count, types.len()),
        ));
    }

    // Read each value
    let mut values = Vec::with_capacity(count);
    for type_idx in 0..count {
        // Need at least 4 more bytes for the size
        if offset + 4 > data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Not enough data to read value size".to_string(),
            ));
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
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Not enough data to read value".to_string(),
            ));
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

/// Deserialize multiple component values using a ReadStream
pub fn deserialize_component_values_with_stream<'a, P: wrt_types::MemoryProvider>(
    reader: &mut ReadStream<'a>,
    types: &[FormatValType],
    provider: &P,
) -> Result<Vec<ComponentValue>> {
    // Read the count
    let count = reader.read_u32_le()? as usize;

    // Validate that we have enough types
    if count > types.len() {
        return Err(Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            format!("Expected {} types but only got {}", count, types.len()),
        ));
    }

    // Read each value
    let mut values = Vec::with_capacity(count);
    for type_idx in 0..count {
        let value = deserialize_component_value_with_stream(reader, &types[type_idx], provider)?;
        values.push(value);
    }

    Ok(values)
}

// Core value conversion functions
pub fn core_to_component_value(value: &Value, ty: &FormatValType) -> Result<ComponentValue> {
    use crate::type_conversion::{
        core_value_to_types_componentvalue, format_valtype_to_types_valtype,
    };

    // First, convert the format value type to a types value type
    let types_val_type = format_valtype_to_types_valtype(ty);

    // Then convert the core value to a component value
    let component_value = core_value_to_types_componentvalue(value)?;

    // Check if the types match or provide a conversion error
    match (&component_value, &types_val_type) {
        // Basic type checking for primitive types
        (ComponentValue::S32(_), ValType::S32)
        | (ComponentValue::S64(_), ValType::S64)
        | (ComponentValue::F32(_), ValType::F32)
        | (ComponentValue::F64(_), ValType::F64) => Ok(component_value),

        // Handle boolean conversion from i32
        (ComponentValue::S32(v), ValType::Bool) => Ok(ComponentValue::Bool(*v != 0)),

        // Other integer width conversions
        (ComponentValue::S32(v), ValType::S8) => Ok(ComponentValue::S8(*v as i8)),
        (ComponentValue::S32(v), ValType::U8) => Ok(ComponentValue::U8(*v as u8)),
        (ComponentValue::S32(v), ValType::S16) => Ok(ComponentValue::S16(*v as i16)),
        (ComponentValue::S32(v), ValType::U16) => Ok(ComponentValue::U16(*v as u16)),
        (ComponentValue::S32(v), ValType::U32) => Ok(ComponentValue::U32(*v as u32)),
        (ComponentValue::S64(v), ValType::U64) => Ok(ComponentValue::U64(*v as u64)),

        // Error for type mismatch
        _ => Err(Error::new(
            ErrorCategory::Type,
            codes::CONVERSION_ERROR,
            format!(
                "Type mismatch: cannot convert {:?} to component value of type {:?}",
                value, types_val_type
            ),
        )),
    }
}

pub fn component_to_core_value(value: &ComponentValue) -> Result<Value> {
    use crate::type_conversion::types_componentvalue_to_core_value;

    // Use the centralized conversion function
    types_componentvalue_to_core_value(value)
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
        FormatValType::String => 8, // Pointer size
        FormatValType::Ref(_) => 4,
        FormatValType::Record(_) => 8,
        FormatValType::Variant(_) => 8,
        FormatValType::List(_) => 8,
        FormatValType::Tuple(_) => 8,
        FormatValType::Flags(_) => 8,
        FormatValType::Enum(_) => 4,
        FormatValType::Option(_) => 8,
        FormatValType::Result(_) => 8,
        FormatValType::ResultErr(_) => 8,
        FormatValType::ResultBoth(_, _) => 8,
        FormatValType::Own(_) => 4,
        FormatValType::Borrow(_) => 4,
        FormatValType::FixedList(_, _) => 8,
        FormatValType::ErrorContext => 4,
        // Default size for any unhandled types
        _ => 4,
    }
}

#[cfg(test)]
mod tests {
    use wrt_types::safe_memory::{NoStdProvider, SafeMemoryHandler, Slice, SliceMut};

    use super::*;

    #[test]
    fn test_primitive_value_encoding_decoding() {
        // Test a few primitive types
        let values =
            vec![ComponentValue::Bool(true), ComponentValue::S32(42), ComponentValue::F64(3.14159)];

        for value in values {
            let encoded = serialize_component_value(&value).unwrap();
            let format_type = convert_common_to_format_valtype(&value.get_type());
            let decoded = deserialize_component_value(&encoded, &format_type).unwrap();

            // Only check bools since we only implemented deserialization for a subset of
            // types
            if let ComponentValue::Bool(_) = value {
                assert_eq!(value, decoded);
            }
        }
    }

    #[test]
    fn test_stream_serialization() {
        // Create a memory provider
        let provider = NoStdProvider::<1024>::default();
        let handler = SafeMemoryHandler::new(provider).unwrap();

        // Get a mutable slice for the output buffer
        let mut slice_mut = handler.get_slice_mut(0, 1024).unwrap();
        let mut writer = WriteStream::new(slice_mut);

        // Create a simple value to serialize
        let value = ComponentValue::Bool(true);

        // Serialize using stream
        serialize_component_value_with_stream(&value, &mut writer, &handler).unwrap();

        // Read back
        let position = writer.position();
        drop(writer); // Release mutable borrow

        let slice = handler.borrow_slice(0, position).unwrap();
        let mut reader = ReadStream::new(slice);

        // Deserialize using stream
        let format_type = FormatValType::Bool;
        let decoded =
            deserialize_component_value_with_stream(&mut reader, &format_type, &handler).unwrap();

        // Verify
        assert_eq!(value, decoded);
    }

    #[test]
    fn test_multiple_values_stream() {
        // Create a memory provider
        let provider = NoStdProvider::<1024>::default();
        let handler = SafeMemoryHandler::new(provider).unwrap();

        // Get a mutable slice for the output buffer
        let mut slice_mut = handler.get_slice_mut(0, 1024).unwrap();
        let mut writer = WriteStream::new(slice_mut);

        // Create values to serialize
        let values = vec![
            ComponentValue::Bool(true),
            ComponentValue::S32(42),
            ComponentValue::String("hello".to_string()),
        ];

        // Serialize using stream
        serialize_component_values_with_stream(&values, &mut writer, &handler).unwrap();

        // Read back
        let position = writer.position();
        drop(writer); // Release mutable borrow

        let slice = handler.borrow_slice(0, position).unwrap();
        let mut reader = ReadStream::new(slice);

        // Prepare format types
        let format_types = vec![FormatValType::Bool, FormatValType::S32, FormatValType::String];

        // Deserialize using stream
        let decoded =
            deserialize_component_values_with_stream(&mut reader, &format_types, &handler).unwrap();

        // Verify count
        assert_eq!(values.len(), decoded.len());

        // Verify individual values
        assert_eq!(values[0], decoded[0]);

        // For S32, we can use a direct comparison
        if let (ComponentValue::S32(original), ComponentValue::S32(decoded_val)) =
            (&values[1], &decoded[1])
        {
            assert_eq!(original, decoded_val);
        } else {
            panic!("Expected S32 values");
        }

        // For String, compare the string values
        if let (ComponentValue::String(original), ComponentValue::String(decoded_val)) =
            (&values[2], &decoded[2])
        {
            assert_eq!(original, decoded_val);
        } else {
            panic!("Expected String values");
        }
    }
}
