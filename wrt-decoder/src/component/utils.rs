use crate::prelude::*;
use wrt_error::{kinds, Error, Result};
use wrt_format::binary;
use wrt_format::component::ValType;

/// Add a section to the binary with the given ID and content
pub fn add_section(binary: &mut Vec<u8>, section_id: u8, content: &[u8]) {
    binary.push(section_id);
    binary.extend(binary::write_leb128_u32(content.len() as u32));
    binary.extend_from_slice(content);
}

/// Check if a given string is a valid semantic version
pub fn is_valid_semver(version: &str) -> bool {
    // Simple semver validation (major.minor.patch)
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }

    parts.iter().all(|part| part.parse::<u32>().is_ok())
}

/// Check if a string represents a valid integrity hash
pub fn is_valid_integrity(integrity: &str) -> bool {
    // Simple integrity validation (algo-VALUE)
    let parts: Vec<&str> = integrity.split('-').collect();
    if parts.len() != 2 {
        return false;
    }

    // Check algorithm is supported
    let algorithm = parts[0];
    match algorithm {
        "sha256" | "sha384" | "sha512" => true,
        _ => false,
    }
}

/// Check if the binary is a WebAssembly component
pub fn is_component(bytes: &[u8]) -> Result<bool> {
    if bytes.len() < 8 {
        return Err(Error::new(kinds::ParseError(
            "Binary too short for WebAssembly header".to_string(),
        )));
    }

    if bytes[0..4] != binary::WASM_MAGIC {
        return Err(Error::new(kinds::ParseError(
            "Invalid WebAssembly magic bytes".to_string(),
        )));
    }

    // Check for component layer
    Ok(bytes[6] == binary::COMPONENT_LAYER[0] && bytes[7] == binary::COMPONENT_LAYER[1])
}

/// Parse a ValType from binary format
pub fn parse_val_type(bytes: &[u8], offset: usize) -> Result<(ValType, usize)> {
    if offset >= bytes.len() {
        return Err(Error::new(kinds::ParseError(
            "Unexpected end of binary when parsing ValType".to_string(),
        )));
    }

    let val_type_byte = bytes[offset];
    let val_type = match val_type_byte {
        0x00 => ValType::Bool,
        0x01 => ValType::S8,
        0x02 => ValType::U8,
        0x03 => ValType::S16,
        0x04 => ValType::U16,
        0x05 => ValType::S32,
        0x06 => ValType::U32,
        0x07 => ValType::S64,
        0x08 => ValType::U64,
        0x09 => ValType::F32,
        0x0A => ValType::F64,
        0x0B => ValType::Char,
        0x0C => ValType::String,
        _ => {
            return Err(Error::new(kinds::ParseError(format!(
                "Unknown ValType byte: {:#x}",
                val_type_byte
            ))));
        }
    };

    Ok((val_type, 1))
}
