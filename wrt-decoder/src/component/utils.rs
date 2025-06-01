// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use wrt_format::{binary, component::FormatValType};

use crate::{prelude::*, Error, Result};

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
    matches!(algorithm, "sha256" | "sha384" | "sha512")
}

/// Check if the binary is a WebAssembly component
pub fn is_component(bytes: &[u8]) -> Result<bool> {
    if bytes.len() < 8 {
        return Err(Error::parse_error("Binary too short for WebAssembly header".to_string()));
    }

    if bytes[0..4] != binary::WASM_MAGIC {
        return Err(Error::parse_error("Invalid WebAssembly magic bytes".to_string()));
    }

    // Check for component layer
    Ok(bytes[6] == binary::COMPONENT_LAYER[0] && bytes[7] == binary::COMPONENT_LAYER[1])
}

/// Parse a ValType from binary format
pub fn parse_val_type(bytes: &[u8], offset: usize) -> Result<(FormatValType, usize)> {
    if offset >= bytes.len() {
        return Err(Error::parse_error(
            "Unexpected end of binary when parsing ValType".to_string(),
        ));
    }

    let val_type_byte = bytes[offset];
    let val_type = match val_type_byte {
        0x00 => FormatValType::Bool,
        0x01 => FormatValType::S8,
        0x02 => FormatValType::U8,
        0x03 => FormatValType::S16,
        0x04 => FormatValType::U16,
        0x05 => FormatValType::S32,
        0x06 => FormatValType::U32,
        0x07 => FormatValType::S64,
        0x08 => FormatValType::U64,
        0x09 => FormatValType::F32,
        0x0A => FormatValType::F64,
        0x0B => FormatValType::Char,
        0x0C => FormatValType::String,
        _ => {
            return Err(Error::parse_error(format!("Unknown ValType byte: {:#x}", val_type_byte)));
        }
    };

    Ok((val_type, 1))
}

pub fn invalid_component_format(message: &str) -> Error {
    Error::validation_error(message.to_string())
}

pub fn invalid_component_data(message: &str) -> Error {
    Error::validation_error(message.to_string())
}

pub fn invalid_component_section(message: &str) -> Error {
    Error::validation_error(message.to_string())
}

pub fn invalid_component_value(message: &str) -> Error {
    Error::validation_error(message.to_string())
}

pub fn parse_error(message: &str) -> Error {
    Error::parse_error(message.to_string())
}

pub fn parse_error_with_context(message: &str, context: &str) -> Error {
    Error::parse_error(format!("{}: {}", message, context))
}

pub fn parse_error_with_position(message: &str, position: usize) -> Error {
    Error::parse_error(format!("{} at position {}", message, position))
}
