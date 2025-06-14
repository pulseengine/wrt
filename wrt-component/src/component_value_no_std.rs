// WRT - wrt-component
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! No-std compatible Component Model value handling
//!
//! This module provides implementations for Component Model value types
//! in a no_std environment, using bounded collections for safety.

use wrt_format::component::ValType<NoStdProvider<65536>> as FormatValType<NoStdProvider<65536>>;
use wrt_foundation::{
    bounded::{BoundedVec, MAX_COMPONENT_TYPES},
    // component_value::{ComponentValue, ValType<NoStdProvider<65536>> as TypesValType<NoStdProvider<65536>>, ValType<NoStdProvider<65536>>Ref}, // Commented out - std only
    traits::{ReadStream, WriteStream},
    values::Value,
};

// Temporary no_std compatible types until component_value is available in no_std
pub type ComponentValue = Value; // Use Value instead of ComponentValue for no_std
pub type ValType<NoStdProvider<65536>>Ref = u32;
type TypesValType<NoStdProvider<65536>> = crate::types::ValType<NoStdProvider<65536>>;

use crate::prelude::*;

// Maximum size for serialized component values
pub const MAX_SERIALIZED_VALUE_SIZE: usize = 4096;

// Use TypesValType<NoStdProvider<65536>> for the canonical ValType
type CanonicalValType<NoStdProvider<65536>> = TypesValType<NoStdProvider<65536>>;

/// Serialize a ComponentValue to a bounded buffer in a no_std environment
pub fn serialize_component_value_no_std(
    value: &ComponentValue,
) -> core::result::Result<BoundedVec<u8, MAX_SERIALIZED_VALUE_SIZE, NoStdProvider<65536>>, NoStdProvider<65536>> {
    let mut buffer = BoundedVec::new(NoStdProvider::<65536>::default()).unwrap();

    match value {
        ComponentValue::Bool(b) => {
            buffer.push(if *b { 1 } else { 0 }).map_err(|_| {
                Error::new(
                    ErrorCategory::Capacity,
                    codes::CAPACITY_EXCEEDED,
                    "Buffer capacity exceeded when serializing Bool",
                )
            })?;
        }
        ComponentValue::S8(v) => {
            buffer.push(*v as u8).map_err(|_| {
                Error::new(
                    ErrorCategory::Capacity,
                    codes::CAPACITY_EXCEEDED,
                    "Buffer capacity exceeded when serializing S8",
                )
            })?;
        }
        ComponentValue::U8(v) => {
            buffer.push(*v).map_err(|_| {
                Error::new(
                    ErrorCategory::Capacity,
                    codes::CAPACITY_EXCEEDED,
                    "Buffer capacity exceeded when serializing U8",
                )
            })?;
        }
        ComponentValue::S16(v) => {
            for byte in v.to_le_bytes() {
                buffer.push(byte).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Buffer capacity exceeded when serializing S16",
                    )
                })?;
            }
        }
        ComponentValue::U16(v) => {
            for byte in v.to_le_bytes() {
                buffer.push(byte).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Buffer capacity exceeded when serializing U16",
                    )
                })?;
            }
        }
        ComponentValue::S32(v) => {
            for byte in v.to_le_bytes() {
                buffer.push(byte).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Buffer capacity exceeded when serializing S32",
                    )
                })?;
            }
        }
        ComponentValue::U32(v) => {
            for byte in v.to_le_bytes() {
                buffer.push(byte).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Buffer capacity exceeded when serializing U32",
                    )
                })?;
            }
        }
        ComponentValue::S64(v) => {
            for byte in v.to_le_bytes() {
                buffer.push(byte).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Buffer capacity exceeded when serializing S64",
                    )
                })?;
            }
        }
        ComponentValue::U64(v) => {
            for byte in v.to_le_bytes() {
                buffer.push(byte).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Buffer capacity exceeded when serializing U64",
                    )
                })?;
            }
        }
        ComponentValue::F32(v) => {
            for byte in v.to_bits().to_le_bytes() {
                buffer.push(byte).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Buffer capacity exceeded when serializing F32",
                    )
                })?;
            }
        }
        ComponentValue::F64(v) => {
            for byte in v.to_bits().to_le_bytes() {
                buffer.push(byte).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Buffer capacity exceeded when serializing F64",
                    )
                })?;
            }
        }
        ComponentValue::Char(c) => {
            let bytes = [*c as u8];
            for byte in bytes {
                buffer.push(byte).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Buffer capacity exceeded when serializing Char",
                    )
                })?;
            }
        }
        ComponentValue::String(s) => {
            // Push string length as u32
            let len = s.len() as u32;
            for byte in len.to_le_bytes() {
                buffer.push(byte).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Buffer capacity exceeded when serializing String length",
                    )
                })?;
            }

            // Push string bytes
            for byte in s.as_bytes() {
                buffer.push(*byte).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Buffer capacity exceeded when serializing String content",
                    )
                })?;
            }
        }
        ComponentValue::List(items) => {
            // Push list length as u32
            let len = items.len() as u32;
            for byte in len.to_le_bytes() {
                buffer.push(byte).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Buffer capacity exceeded when serializing List length",
                    )
                })?;
            }

            // Push each item
            for item in items {
                let item_data = serialize_component_value_no_std(item)?;
                // Push item size first
                let item_size = item_data.len() as u32;
                for byte in item_size.to_le_bytes() {
                    buffer.push(byte).map_err(|_| {
                        Error::new(
                            ErrorCategory::Capacity,
                            codes::CAPACITY_EXCEEDED,
                            "Buffer capacity exceeded when serializing List item size",
                        )
                    })?;
                }

                // Push item data
                for byte in item_data.iter() {
                    buffer.push(*byte).map_err(|_| {
                        Error::new(
                            ErrorCategory::Capacity,
                            codes::CAPACITY_EXCEEDED,
                            "Buffer capacity exceeded when serializing List item data",
                        )
                    })?;
                }
            }
        }
        ComponentValue::Record(fields) => {
            // Push record field count as u32
            let field_count = fields.len() as u32;
            for byte in field_count.to_le_bytes() {
                buffer.push(byte).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Buffer capacity exceeded when serializing Record field count",
                    )
                })?;
            }

            // Push each field
            for (name, value) in fields {
                // Push field name length as u16
                let name_len = name.len() as u16;
                for byte in name_len.to_le_bytes() {
                    buffer.push(byte).map_err(|_| {
                        Error::new(
                            ErrorCategory::Capacity,
                            codes::CAPACITY_EXCEEDED,
                            "Buffer capacity exceeded when serializing Record field name length",
                        )
                    })?;
                }

                // Push field name bytes
                for byte in name.as_bytes() {
                    buffer.push(*byte).map_err(|_| {
                        Error::new(
                            ErrorCategory::Capacity,
                            codes::CAPACITY_EXCEEDED,
                            "Buffer capacity exceeded when serializing Record field name content",
                        )
                    })?;
                }

                // Serialize and push field value
                let value_data = serialize_component_value_no_std(value)?;
                for byte in value_data.iter() {
                    buffer.push(*byte).map_err(|_| {
                        Error::new(
                            ErrorCategory::Capacity,
                            codes::CAPACITY_EXCEEDED,
                            "Buffer capacity exceeded when serializing Record field value",
                        )
                    })?;
                }
            }
        }
        ComponentValue::Tuple(items) => {
            // Push tuple length as u32
            let len = items.len() as u32;
            for byte in len.to_le_bytes() {
                buffer.push(byte).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Buffer capacity exceeded when serializing Tuple length",
                    )
                })?;
            }

            // Push each item
            for item in items {
                let item_data = serialize_component_value_no_std(item)?;
                // Push item data directly (size is known from type)
                for byte in item_data.iter() {
                    buffer.push(*byte).map_err(|_| {
                        Error::new(
                            ErrorCategory::Capacity,
                            codes::CAPACITY_EXCEEDED,
                            "Buffer capacity exceeded when serializing Tuple item data",
                        )
                    })?;
                }
            }
        }
        ComponentValue::Variant(case, value) => {
            // Push discriminant as u32
            for byte in (*case as u32).to_le_bytes() {
                buffer.push(byte).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Buffer capacity exceeded when serializing Variant discriminant",
                    )
                })?;
            }

            // If there's a value, serialize it
            if let Some(val) = value {
                let value_data = serialize_component_value_no_std(val)?;
                for byte in value_data.iter() {
                    buffer.push(*byte).map_err(|_| {
                        Error::new(
                            ErrorCategory::Capacity,
                            codes::CAPACITY_EXCEEDED,
                            "Buffer capacity exceeded when serializing Variant value",
                        )
                    })?;
                }
            }
        }
        ComponentValue::Enum(case) => {
            // Push discriminant as u32
            for byte in (*case as u32).to_le_bytes() {
                buffer.push(byte).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Buffer capacity exceeded when serializing Enum discriminant",
                    )
                })?;
            }
        }
        ComponentValue::Option(value) => {
            // Push presence flag as u8
            if let Some(val) = value {
                buffer.push(1).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Buffer capacity exceeded when serializing Option presence flag",
                    )
                })?;

                // Serialize the contained value
                let value_data = serialize_component_value_no_std(val)?;
                for byte in value_data.iter() {
                    buffer.push(*byte).map_err(|_| {
                        Error::new(
                            ErrorCategory::Capacity,
                            codes::CAPACITY_EXCEEDED,
                            "Buffer capacity exceeded when serializing Option value",
                        )
                    })?;
                }
            } else {
                buffer.push(0).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Buffer capacity exceeded when serializing Option presence flag",
                    )
                })?;
            }
        }
        ComponentValue::Result(result) => {
            match result {
                Ok(val) => {
                    // Push success flag as u8
                    buffer.push(1).map_err(|_| {
                        Error::new(
                            ErrorCategory::Capacity,
                            codes::CAPACITY_EXCEEDED,
                            "Buffer capacity exceeded when serializing Result success flag",
                        )
                    })?;

                    // If there's a value, serialize it
                    if let Some(v) = val {
                        // Push presence flag as u8
                        buffer.push(1).map_err(|_| {
                            Error::new(
                                ErrorCategory::Capacity,
                                codes::CAPACITY_EXCEEDED,
                                "Buffer capacity exceeded when serializing Result ok presence flag",
                            )
                        })?;

                        // Serialize the contained value
                        let value_data = serialize_component_value_no_std(v)?;
                        for byte in value_data.iter() {
                            buffer.push(*byte).map_err(|_| {
                                Error::new(
                                    ErrorCategory::Capacity,
                                    codes::CAPACITY_EXCEEDED,
                                    "Buffer capacity exceeded when serializing Result ok value",
                                )
                            })?;
                        }
                    } else {
                        // Push absence flag as u8
                        buffer.push(0).map_err(|_| {
                            Error::new(
                                ErrorCategory::Capacity,
                                codes::CAPACITY_EXCEEDED,
                                "Buffer capacity exceeded when serializing Result ok presence flag",
                            )
                        })?;
                    }
                }
                Err(val) => {
                    // Push error flag as u8
                    buffer.push(0).map_err(|_| {
                        Error::new(
                            ErrorCategory::Capacity,
                            codes::CAPACITY_EXCEEDED,
                            "Buffer capacity exceeded when serializing Result error flag",
                        )
                    })?;

                    // If there's a value, serialize it
                    if let Some(v) = val {
                        // Push presence flag as u8
                        buffer.push(1).map_err(|_| {
                            Error::new(
                                ErrorCategory::Capacity,
                                codes::CAPACITY_EXCEEDED,
                                "Buffer capacity exceeded when serializing Result err presence \
                                 flag",
                            )
                        })?;

                        // Serialize the contained value
                        let value_data = serialize_component_value_no_std(v)?;
                        for byte in value_data.iter() {
                            buffer.push(*byte).map_err(|_| {
                                Error::new(
                                    ErrorCategory::Capacity,
                                    codes::CAPACITY_EXCEEDED,
                                    "Buffer capacity exceeded when serializing Result err value",
                                )
                            })?;
                        }
                    } else {
                        // Push absence flag as u8
                        buffer.push(0).map_err(|_| {
                            Error::new(
                                ErrorCategory::Capacity,
                                codes::CAPACITY_EXCEEDED,
                                "Buffer capacity exceeded when serializing Result err presence \
                                 flag",
                            )
                        })?;
                    }
                }
            }
        }
        ComponentValue::Flags(flags) => {
            // Determine how many bytes we need for the flags
            let num_bytes = (flags.len() + 7) / 8;

            // Push the number of bytes as u32
            for byte in (num_bytes as u32).to_le_bytes() {
                buffer.push(byte).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Buffer capacity exceeded when serializing Flags byte count",
                    )
                })?;
            }

            // Push the flag bits as bytes
            for byte_idx in 0..num_bytes {
                let mut byte = 0u8;
                for bit_idx in 0..8 {
                    let flag_idx = byte_idx * 8 + bit_idx;
                    if flag_idx < flags.len() && flags[flag_idx] {
                        byte |= 1 << bit_idx;
                    }
                }
                buffer.push(byte).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Buffer capacity exceeded when serializing Flags bits",
                    )
                })?;
            }
        }
        ComponentValue::U32(v) => {
            for byte in v.to_le_bytes() {
                buffer.push(byte).map_err(|_| {
                    Error::new(
                        ErrorCategory::Capacity,
                        codes::CAPACITY_EXCEEDED,
                        "Buffer capacity exceeded when serializing U32",
                    )
                })?;
            }
        }
        ComponentValue::Void => {
            // Void needs no serialization, it has no data
        }
        _ => {
            return Err(Error::new(
                ErrorCategory::Serialization,
                codes::SERIALIZATION_ERROR,
                "Component not found",
            ));
        }
    }

    Ok(buffer)
}

/// Convert ValType<NoStdProvider<65536>> from wrt_foundation to FormatValType<NoStdProvider<65536>> from wrt_format
/// This function is adapted for no_std environments
pub fn convert_valtype_to_format<P: MemoryProvider + Default + Clone + PartialEq + Eq>(
    val_type: &TypesValType<P>,
) -> Result<FormatValType<NoStdProvider<65536>> {
    match val_type {
        TypesValType<NoStdProvider<65536>>::Bool => Ok(FormatValType<NoStdProvider<65536>>::Bool),
        TypesValType<NoStdProvider<65536>>::S8 => Ok(FormatValType<NoStdProvider<65536>>::S8),
        TypesValType<NoStdProvider<65536>>::U8 => Ok(FormatValType<NoStdProvider<65536>>::U8),
        TypesValType<NoStdProvider<65536>>::S16 => Ok(FormatValType<NoStdProvider<65536>>::S16),
        TypesValType<NoStdProvider<65536>>::U16 => Ok(FormatValType<NoStdProvider<65536>>::U16),
        TypesValType<NoStdProvider<65536>>::S32 => Ok(FormatValType<NoStdProvider<65536>>::S32),
        TypesValType<NoStdProvider<65536>>::U32 => Ok(FormatValType<NoStdProvider<65536>>::U32),
        TypesValType<NoStdProvider<65536>>::S64 => Ok(FormatValType<NoStdProvider<65536>>::S64),
        TypesValType<NoStdProvider<65536>>::U64 => Ok(FormatValType<NoStdProvider<65536>>::U64),
        TypesValType<NoStdProvider<65536>>::F32 => Ok(FormatValType<NoStdProvider<65536>>::F32),
        TypesValType<NoStdProvider<65536>>::F64 => Ok(FormatValType<NoStdProvider<65536>>::F64),
        TypesValType<NoStdProvider<65536>>::Char => Ok(FormatValType<NoStdProvider<65536>>::Char),
        TypesValType<NoStdProvider<65536>>::String => Ok(FormatValType<NoStdProvider<65536>>::String),
        TypesValType<NoStdProvider<65536>>::Ref(idx) => Ok(FormatValType<NoStdProvider<65536>>::Ref(*idx)),
        // Complex types like Record, Variant, List, etc. are not fully implemented
        // for no_std but would follow the same pattern, converting each nested type
        TypesValType<NoStdProvider<65536>>::Void => Ok(FormatValType<NoStdProvider<65536>>::Tuple(Vec::new())),
        _ => Err(Error::new(
            ErrorCategory::Type,
            codes::TYPE_CONVERSION_ERROR,
            "Component not found",
        )),
    }
}

/// Convert FormatValType<NoStdProvider<65536>> from wrt_format to ValType<NoStdProvider<65536>> from wrt_foundation
/// This function is adapted for no_std environments
pub fn convert_format_to_valtype<P: MemoryProvider + Default + Clone + PartialEq + Eq>(
    format_type: &FormatValType<NoStdProvider<65536>>,
) -> Result<TypesValType<P>> {
    match format_type {
        FormatValType<NoStdProvider<65536>>::Bool => Ok(TypesValType<NoStdProvider<65536>>::Bool),
        FormatValType<NoStdProvider<65536>>::S8 => Ok(TypesValType<NoStdProvider<65536>>::S8),
        FormatValType<NoStdProvider<65536>>::U8 => Ok(TypesValType<NoStdProvider<65536>>::U8),
        FormatValType<NoStdProvider<65536>>::S16 => Ok(TypesValType<NoStdProvider<65536>>::S16),
        FormatValType<NoStdProvider<65536>>::U16 => Ok(TypesValType<NoStdProvider<65536>>::U16),
        FormatValType<NoStdProvider<65536>>::S32 => Ok(TypesValType<NoStdProvider<65536>>::S32),
        FormatValType<NoStdProvider<65536>>::U32 => Ok(TypesValType<NoStdProvider<65536>>::U32),
        FormatValType<NoStdProvider<65536>>::S64 => Ok(TypesValType<NoStdProvider<65536>>::S64),
        FormatValType<NoStdProvider<65536>>::U64 => Ok(TypesValType<NoStdProvider<65536>>::U64),
        FormatValType<NoStdProvider<65536>>::F32 => Ok(TypesValType<NoStdProvider<65536>>::F32),
        FormatValType<NoStdProvider<65536>>::F64 => Ok(TypesValType<NoStdProvider<65536>>::F64),
        FormatValType<NoStdProvider<65536>>::Char => Ok(TypesValType<NoStdProvider<65536>>::Char),
        FormatValType<NoStdProvider<65536>>::String => Ok(TypesValType<NoStdProvider<65536>>::String),
        FormatValType<NoStdProvider<65536>>::Ref(idx) => Ok(TypesValType<NoStdProvider<65536>>::Ref(*idx)),
        // Complex types like Record, Variant, List, etc. are not fully implemented
        // for no_std but would follow the same pattern, converting each nested type
        _ => Err(Error::new(
            ErrorCategory::Type,
            codes::TYPE_CONVERSION_ERROR,
            "Component not found",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_simple_types() {
        // Test boolean serialization
        let bool_value = ComponentValue::Bool(true);
        let serialized = serialize_component_value_no_std(&bool_value).unwrap();
        assert_eq!(serialized.as_slice(), &[1]);

        // Test u32 serialization
        let u32_value = ComponentValue::U32(0x12345678);
        let serialized = serialize_component_value_no_std(&u32_value).unwrap();
        assert_eq!(serialized.as_slice(), &[0x78, 0x56, 0x34, 0x12]); // Little endian

        // Test string serialization
        let string_value = ComponentValue::String("test".to_string());
        let serialized = serialize_component_value_no_std(&string_value).unwrap();
        assert_eq!(serialized.as_slice(), &[4, 0, 0, 0, b't', b'e', b's', b't']);
    }

    #[test]
    fn test_valtype_conversion() {
        use wrt_foundation::safe_memory::NoStdProvider<65536>;

        // Test bool conversion
        let bool_type = TypesValType<NoStdProvider<65536>>::<NoStdProvider::<65536>>::Bool;
        let format_type = convert_valtype_to_format(&bool_type).unwrap();
        assert!(matches!(format_type, FormatValType<NoStdProvider<65536>>::Bool));

        let converted_back =
            convert_format_to_valtype::<NoStdProvider::<65536>>(&format_type).unwrap();
        assert!(matches!(converted_back, TypesValType<NoStdProvider<65536>>::Bool));
    }
}
