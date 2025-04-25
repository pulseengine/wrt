use crate::prelude::*;
use wrt_error::{kinds, Error, Result};
use wrt_format::binary;
use wrt_format::component::ValType;

/// Helper function to encode a value type to binary format
pub fn encode_val_type(result: &mut Vec<u8>, val_type: &ValType) -> Result<()> {
    match val_type {
        ValType::Bool => result.push(0x07),
        ValType::S8 => result.push(0x08),
        ValType::U8 => result.push(0x09),
        ValType::S16 => result.push(0x0A),
        ValType::U16 => result.push(0x0B),
        ValType::String => result.push(0x0C),
        ValType::List(inner) => {
            result.push(0x0D);
            encode_val_type(result, inner)?;
        }
        ValType::S32 => result.push(0x01),
        ValType::U32 => result.push(0x02),
        ValType::S64 => result.push(0x03),
        ValType::U64 => result.push(0x04),
        ValType::F32 => result.push(0x05),
        ValType::F64 => result.push(0x06),
        ValType::Record(fields) => {
            result.push(0x0E);
            result.extend_from_slice(&binary::write_leb128_u32(fields.len() as u32));
            for (name, field_type) in fields {
                result.extend_from_slice(&binary::write_string(name));
                encode_val_type(result, field_type)?;
            }
        }
        ValType::Variant(cases) => {
            result.push(0x0F);
            result.extend_from_slice(&binary::write_leb128_u32(cases.len() as u32));
            for (case_name, case_type) in cases {
                result.extend_from_slice(&binary::write_string(case_name));
                if let Some(ty) = case_type {
                    result.push(0x01); // has type
                    encode_val_type(result, ty)?;
                } else {
                    result.push(0x00); // no type
                }
            }
        }
        ValType::Tuple(types) => {
            result.push(0x10);
            result.extend_from_slice(&binary::write_leb128_u32(types.len() as u32));
            for ty in types {
                encode_val_type(result, ty)?;
            }
        }
        ValType::Option(inner) => {
            result.push(0x11);
            encode_val_type(result, inner)?;
        }
        ValType::ResultBoth(ok, err) => {
            result.push(0x12);
            result.push(0x03); // both ok and err
            encode_val_type(result, ok)?;
            encode_val_type(result, err)?;
        }
        ValType::Result(ok) => {
            result.push(0x12);
            result.push(0x01); // ok only
            encode_val_type(result, ok)?;
        }
        ValType::ResultErr(err) => {
            result.push(0x12);
            result.push(0x02); // err only
            encode_val_type(result, err)?;
        }
        ValType::Enum(cases) => {
            result.push(0x13);
            result.extend_from_slice(&binary::write_leb128_u32(cases.len() as u32));
            for case_name in cases {
                result.extend_from_slice(&binary::write_string(case_name));
            }
        }
        ValType::Flags(names) => {
            result.push(0x14);
            result.extend_from_slice(&binary::write_leb128_u32(names.len() as u32));
            for name in names {
                result.extend_from_slice(&binary::write_string(name));
            }
        }
        ValType::Ref(idx) => {
            result.push(0x15);
            result.extend_from_slice(&binary::write_leb128_u32(*idx));
        }
        ValType::Own(_) | ValType::Borrow(_) => {
            return Err(Error::new(kinds::ParseError(
                "Resource types are not supported for encoding yet".to_string(),
            )));
        }
        ValType::Char => result.push(0x16),
        &wrt_format::component::ValType::FixedList(ref inner, size) => {
            // Fixed-length lists are encoded as a list tag followed by the element type and size
            result.push(0x17); // Example tag for fixed list
            let mut inner_bytes = Vec::new();
            encode_val_type(&mut inner_bytes, inner.as_ref())?;

            // Encode size without dereferencing
            result.extend_from_slice(&binary::write_leb128_u32(size));
        }
        &wrt_format::component::ValType::ErrorContext => {
            // Error context is a simple type
            result.push(0x18); // Example tag for error context
        }
    }
    Ok(())
}
