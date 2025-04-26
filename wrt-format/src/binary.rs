//! WebAssembly binary format handling.
//!
//! This module provides utilities for parsing and generating WebAssembly binary format.

#[cfg(feature = "std")]
use std::{boxed::Box, format, str, string::String, vec::Vec};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

#[cfg(not(feature = "std"))]
use core::str;

use crate::module::Module;
use crate::types::{BlockType, ValueType};
use wrt_error::{kinds, Error, Result};

/// Magic bytes for WebAssembly modules: \0asm
pub const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];

/// WebAssembly binary format version
pub const WASM_VERSION: [u8; 4] = [0x01, 0x00, 0x00, 0x00];

/// WebAssembly section IDs
pub const CUSTOM_SECTION_ID: u8 = 0x00;
pub const TYPE_SECTION_ID: u8 = 0x01;
pub const IMPORT_SECTION_ID: u8 = 0x02;
pub const FUNCTION_SECTION_ID: u8 = 0x03;
pub const TABLE_SECTION_ID: u8 = 0x04;
pub const MEMORY_SECTION_ID: u8 = 0x05;
pub const GLOBAL_SECTION_ID: u8 = 0x06;
pub const EXPORT_SECTION_ID: u8 = 0x07;
pub const START_SECTION_ID: u8 = 0x08;
pub const ELEMENT_SECTION_ID: u8 = 0x09;
pub const CODE_SECTION_ID: u8 = 0x0A;
pub const DATA_SECTION_ID: u8 = 0x0B;
pub const DATA_COUNT_SECTION_ID: u8 = 0x0C;

/// WebAssembly value types
pub const I32_TYPE: u8 = 0x7F;
pub const I64_TYPE: u8 = 0x7E;
pub const F32_TYPE: u8 = 0x7D;
pub const F64_TYPE: u8 = 0x7C;
pub const V128_TYPE: u8 = 0x7B; // For SIMD extension
pub const FUNCREF_TYPE: u8 = 0x70;
pub const EXTERNREF_TYPE: u8 = 0x6F;

/// WebAssembly control instructions
pub const UNREACHABLE: u8 = 0x00;
pub const NOP: u8 = 0x01;
pub const BLOCK: u8 = 0x02;
pub const LOOP: u8 = 0x03;
pub const IF: u8 = 0x04;
pub const ELSE: u8 = 0x05;
pub const END: u8 = 0x0B;
pub const BR: u8 = 0x0C;
pub const BR_IF: u8 = 0x0D;
pub const BR_TABLE: u8 = 0x0E;
pub const RETURN: u8 = 0x0F;
pub const CALL: u8 = 0x10;
pub const CALL_INDIRECT: u8 = 0x11;

/// WebAssembly variable instructions
pub const LOCAL_GET: u8 = 0x20;
pub const LOCAL_SET: u8 = 0x21;
pub const LOCAL_TEE: u8 = 0x22;
pub const GLOBAL_GET: u8 = 0x23;
pub const GLOBAL_SET: u8 = 0x24;

/// WebAssembly constant instructions
pub const I32_CONST: u8 = 0x41;
pub const I64_CONST: u8 = 0x42;
pub const F32_CONST: u8 = 0x43;
pub const F64_CONST: u8 = 0x44;

//==========================================================================
// WebAssembly Component Model Binary Format
//==========================================================================

/// Component Model magic bytes (same as core: \0asm)
pub const COMPONENT_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];

/// Component Model binary format version - 2 bytes version, 2 bytes layer
/// Version 1.0, Layer 1
pub const COMPONENT_VERSION: [u8; 4] = [0x01, 0x00, 0x01, 0x00];

/// Component Model version only (first two bytes of version)
pub const COMPONENT_VERSION_ONLY: [u8; 2] = [0x01, 0x00];

/// Component Model layer identifier - distinguishes components from modules
pub const COMPONENT_LAYER: [u8; 2] = [0x01, 0x00];

/// Component Model section IDs
pub const COMPONENT_CUSTOM_SECTION_ID: u8 = 0x00;
pub const COMPONENT_CORE_MODULE_SECTION_ID: u8 = 0x01;
pub const COMPONENT_CORE_INSTANCE_SECTION_ID: u8 = 0x02;
pub const COMPONENT_CORE_TYPE_SECTION_ID: u8 = 0x03;
pub const COMPONENT_COMPONENT_SECTION_ID: u8 = 0x04;
pub const COMPONENT_INSTANCE_SECTION_ID: u8 = 0x05;
pub const COMPONENT_ALIAS_SECTION_ID: u8 = 0x06;
pub const COMPONENT_TYPE_SECTION_ID: u8 = 0x07;
pub const COMPONENT_CANON_SECTION_ID: u8 = 0x08;
pub const COMPONENT_START_SECTION_ID: u8 = 0x09;
pub const COMPONENT_IMPORT_SECTION_ID: u8 = 0x0A;
pub const COMPONENT_EXPORT_SECTION_ID: u8 = 0x0B;
pub const COMPONENT_VALUE_SECTION_ID: u8 = 0x0C;

/// Component Model sort kinds
pub const COMPONENT_CORE_SORT_FUNC: u8 = 0x00;
pub const COMPONENT_CORE_SORT_TABLE: u8 = 0x01;
pub const COMPONENT_CORE_SORT_MEMORY: u8 = 0x02;
pub const COMPONENT_CORE_SORT_GLOBAL: u8 = 0x03;
pub const COMPONENT_CORE_SORT_TYPE: u8 = 0x10;
pub const COMPONENT_CORE_SORT_MODULE: u8 = 0x11;
pub const COMPONENT_CORE_SORT_INSTANCE: u8 = 0x12;

pub const COMPONENT_SORT_CORE: u8 = 0x00;
pub const COMPONENT_SORT_FUNC: u8 = 0x01;
pub const COMPONENT_SORT_VALUE: u8 = 0x02;
pub const COMPONENT_SORT_TYPE: u8 = 0x03;
pub const COMPONENT_SORT_COMPONENT: u8 = 0x04;
pub const COMPONENT_SORT_INSTANCE: u8 = 0x05;

/// Component Model value type codes
pub const COMPONENT_VALTYPE_BOOL: u8 = 0x7F;
pub const COMPONENT_VALTYPE_S8: u8 = 0x7E;
pub const COMPONENT_VALTYPE_U8: u8 = 0x7D;
pub const COMPONENT_VALTYPE_S16: u8 = 0x7C;
pub const COMPONENT_VALTYPE_U16: u8 = 0x7B;
pub const COMPONENT_VALTYPE_S32: u8 = 0x7A;
pub const COMPONENT_VALTYPE_U32: u8 = 0x79;
pub const COMPONENT_VALTYPE_S64: u8 = 0x78;
pub const COMPONENT_VALTYPE_U64: u8 = 0x77;
pub const COMPONENT_VALTYPE_F32: u8 = 0x76;
pub const COMPONENT_VALTYPE_F64: u8 = 0x75;
pub const COMPONENT_VALTYPE_CHAR: u8 = 0x74;
pub const COMPONENT_VALTYPE_STRING: u8 = 0x73;
pub const COMPONENT_VALTYPE_REF: u8 = 0x72;
pub const COMPONENT_VALTYPE_RECORD: u8 = 0x71;
pub const COMPONENT_VALTYPE_VARIANT: u8 = 0x70;
pub const COMPONENT_VALTYPE_LIST: u8 = 0x6F;
pub const COMPONENT_VALTYPE_FIXED_LIST: u8 = 0x6E;
pub const COMPONENT_VALTYPE_TUPLE: u8 = 0x6D;
pub const COMPONENT_VALTYPE_FLAGS: u8 = 0x6C;
pub const COMPONENT_VALTYPE_ENUM: u8 = 0x6B;
pub const COMPONENT_VALTYPE_OPTION: u8 = 0x6A;
pub const COMPONENT_VALTYPE_RESULT: u8 = 0x69;
pub const COMPONENT_VALTYPE_RESULT_ERR: u8 = 0x68;
pub const COMPONENT_VALTYPE_RESULT_BOTH: u8 = 0x67;
pub const COMPONENT_VALTYPE_OWN: u8 = 0x66;
pub const COMPONENT_VALTYPE_BORROW: u8 = 0x65;
pub const COMPONENT_VALTYPE_ERROR_CONTEXT: u8 = 0x64;

/// Parse a WebAssembly binary into a module
///
/// This is a placeholder that will be implemented fully in Phase 1.
pub fn parse_binary(bytes: &[u8]) -> Result<Module> {
    // Verify magic bytes
    if bytes.len() < 8 {
        return Err(Error::new(kinds::ParseError(
            "WebAssembly binary too short".to_string(),
        )));
    }

    if bytes[0..4] != WASM_MAGIC {
        return Err(Error::new(kinds::ParseError(
            "Invalid WebAssembly magic bytes".to_string(),
        )));
    }

    if bytes[4..8] != WASM_VERSION {
        return Err(Error::new(kinds::ParseError(
            "Unsupported WebAssembly version".to_string(),
        )));
    }

    // Create an empty module with the binary stored
    let mut module = Module::new();
    module.binary = Some(bytes.to_vec());

    // For now, we don't actually parse the module
    // This will be implemented in Phase 1

    Ok(module)
}

/// Generate a WebAssembly binary from a module
///
/// This is a placeholder that will be implemented fully in Phase 1.
pub fn generate_binary(module: &Module) -> Result<Vec<u8>> {
    // If we have the original binary and haven't modified the module,
    // we can just return it
    if let Some(binary) = &module.binary {
        return Ok(binary.clone());
    }

    // Create a minimal valid module
    let mut binary = Vec::with_capacity(8);

    // Magic bytes
    binary.extend_from_slice(&WASM_MAGIC);

    // Version
    binary.extend_from_slice(&WASM_VERSION);

    // Generate sections (placeholder)
    // This will be implemented in Phase 1

    Ok(binary)
}

/// Read a LEB128 unsigned integer from a byte array
///
/// This function will be used when implementing the full binary parser.
pub fn read_leb128_u32(bytes: &[u8], pos: usize) -> Result<(u32, usize)> {
    let mut result = 0u32;
    let mut shift = 0;
    let mut offset = 0;

    loop {
        if pos + offset >= bytes.len() {
            return Err(Error::new(kinds::ParseError(
                "Truncated LEB128 integer".to_string(),
            )));
        }

        let byte = bytes[pos + offset];
        offset += 1;

        // Apply 7 bits from this byte
        result |= ((byte & 0x7F) as u32) << shift;
        shift += 7;

        // Check for continuation bit
        if byte & 0x80 == 0 {
            break;
        }

        // Guard against malformed/malicious LEB128
        if shift >= 32 {
            return Err(Error::new(kinds::ParseError(
                "LEB128 integer too large".to_string(),
            )));
        }
    }

    Ok((result, offset))
}

/// Read a LEB128 signed integer from a byte array
///
/// This function will be used when implementing the full binary parser.
pub fn read_leb128_i32(bytes: &[u8], pos: usize) -> Result<(i32, usize)> {
    let mut result = 0i32;
    let mut shift = 0;
    let mut offset = 0;
    let mut byte;

    loop {
        if pos + offset >= bytes.len() {
            return Err(Error::new(kinds::ParseError(
                "Truncated LEB128 integer".to_string(),
            )));
        }

        byte = bytes[pos + offset];
        offset += 1;

        // Apply 7 bits from this byte
        result |= ((byte & 0x7F) as i32) << shift;
        shift += 7;

        // Check for continuation bit
        if byte & 0x80 == 0 {
            break;
        }

        // Guard against malformed/malicious LEB128
        if shift >= 32 {
            return Err(Error::new(kinds::ParseError(
                "LEB128 integer too large".to_string(),
            )));
        }
    }

    // Sign extend if needed
    if shift < 32 && (byte & 0x40) != 0 {
        // The result is negative, sign extend it
        result |= !0 << shift;
    }

    Ok((result, offset))
}

/// Read a LEB128 signed 64-bit integer from a byte array
///
/// This function will be used when implementing the full binary parser.
pub fn read_leb128_i64(bytes: &[u8], pos: usize) -> Result<(i64, usize)> {
    let mut result = 0i64;
    let mut shift = 0;
    let mut offset = 0;
    let mut byte;

    loop {
        if pos + offset >= bytes.len() {
            return Err(Error::new(kinds::ParseError(
                "Truncated LEB128 integer".to_string(),
            )));
        }

        byte = bytes[pos + offset];
        offset += 1;

        // Apply 7 bits from this byte
        result |= ((byte & 0x7F) as i64) << shift;
        shift += 7;

        // Check for continuation bit
        if byte & 0x80 == 0 {
            break;
        }

        // Guard against malformed/malicious LEB128
        if shift >= 64 {
            return Err(Error::new(kinds::ParseError(
                "LEB128 integer too large".to_string(),
            )));
        }
    }

    // Sign extend if needed
    if shift < 64 && (byte & 0x40) != 0 {
        // The result is negative, sign extend it
        result |= !0 << shift;
    }

    Ok((result, offset))
}

/// Write a LEB128 unsigned integer to a byte array
///
/// This function will be used when implementing the full binary generator.
pub fn write_leb128_u32(value: u32) -> Vec<u8> {
    if value == 0 {
        return vec![0];
    }

    let mut result = Vec::new();
    let mut value = value;

    while value != 0 {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;

        if value != 0 {
            byte |= 0x80;
        }

        result.push(byte);
    }

    result
}

/// Write a LEB128 signed integer to a byte array
///
/// This function will be used when implementing the full binary generator.
pub fn write_leb128_i32(value: i32) -> Vec<u8> {
    let mut result = Vec::new();
    let mut value = value;
    let mut more = true;

    while more {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;

        // If the original value is negative, we need to sign extend
        let is_sign_bit_set = (byte & 0x40) != 0;
        let sign_extended_value = if value == 0 && !is_sign_bit_set {
            0
        } else if value == -1 && is_sign_bit_set {
            -1
        } else {
            value
        };

        more = sign_extended_value != 0 && sign_extended_value != -1;

        if more {
            byte |= 0x80;
        }

        result.push(byte);
    }

    result
}

/// Write a LEB128 signed 64-bit integer to a byte array
///
/// This function will be used when implementing the full binary formatter.
pub fn write_leb128_i64(value: i64) -> Vec<u8> {
    let mut result = Vec::new();
    let mut value = value;
    let mut more = true;

    while more {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;

        // If the original value is negative, we need to sign extend
        let is_sign_bit_set = (byte & 0x40) != 0;
        let sign_extended_value = if value == 0 && !is_sign_bit_set {
            0
        } else if value == -1 && is_sign_bit_set {
            -1
        } else {
            value
        };

        more = sign_extended_value != 0 && sign_extended_value != -1;

        if more {
            byte |= 0x80;
        }

        result.push(byte);
    }

    result
}

/// Check if a binary has a valid WebAssembly header
///
/// This function validates that the binary starts with the WASM_MAGIC and
/// has a supported version.
pub fn is_valid_wasm_header(bytes: &[u8]) -> bool {
    if bytes.len() < 8 {
        return false;
    }

    // Check magic bytes
    if bytes[0..4] != WASM_MAGIC {
        return false;
    }

    // Check version
    if bytes[4..8] != WASM_VERSION && bytes[4..8] != [0x0A, 0x6D, 0x73, 0x63] {
        return false;
    }

    true
}

/// Read a LEB128 unsigned 64-bit integer from a byte array
pub fn read_leb128_u64(bytes: &[u8], pos: usize) -> Result<(u64, usize)> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    let mut offset = pos;
    let mut byte;

    loop {
        if offset >= bytes.len() {
            return Err(Error::new(kinds::ParseError(
                "Unexpected end of LEB128 sequence".to_string(),
            )));
        }

        byte = bytes[offset];
        offset += 1;

        // Apply the 7 bits from the current byte
        result |= ((byte & 0x7F) as u64) << shift;

        // If the high bit is not set, we're done
        if byte & 0x80 == 0 {
            break;
        }

        // Otherwise, shift for the next 7 bits
        shift += 7;

        // Ensure we don't exceed 64 bits (10 bytes)
        if shift >= 64 {
            if byte & 0x7F != 0 {
                return Err(Error::new(kinds::ParseError(
                    "LEB128 sequence exceeds maximum u64 value".to_string(),
                )));
            }
            break;
        }
    }

    Ok((result, offset - pos))
}

/// Write a LEB128 unsigned 64-bit integer to a byte array
pub fn write_leb128_u64(value: u64) -> Vec<u8> {
    let mut result = Vec::new();
    let mut value = value;

    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;

        // If there are more bits to write, set the high bit
        if value != 0 {
            byte |= 0x80;
        }

        result.push(byte);

        // If no more bits, we're done
        if value == 0 {
            break;
        }
    }

    result
}

/// IEEE 754 floating point handling
///
/// Read a 32-bit IEEE 754 float from a byte array
pub fn read_f32(bytes: &[u8], pos: usize) -> Result<(f32, usize)> {
    if pos + 4 > bytes.len() {
        return Err(Error::new(kinds::ParseError(
            "Not enough bytes to read f32".to_string(),
        )));
    }

    let mut buf = [0u8; 4];
    buf.copy_from_slice(&bytes[pos..pos + 4]);

    // Convert the bytes to f32 (little-endian)
    let value = f32::from_le_bytes(buf);

    Ok((value, 4))
}

/// Read a 64-bit IEEE 754 float from a byte array
pub fn read_f64(bytes: &[u8], pos: usize) -> Result<(f64, usize)> {
    if pos + 8 > bytes.len() {
        return Err(Error::new(kinds::ParseError(
            "Not enough bytes to read f64".to_string(),
        )));
    }

    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes[pos..pos + 8]);

    // Convert the bytes to f64 (little-endian)
    let value = f64::from_le_bytes(buf);

    Ok((value, 8))
}

/// Write a 32-bit IEEE 754 float to a byte array
pub fn write_f32(value: f32) -> Vec<u8> {
    let bytes = value.to_le_bytes();
    bytes.to_vec()
}

/// Write a 64-bit IEEE 754 float to a byte array
pub fn write_f64(value: f64) -> Vec<u8> {
    let bytes = value.to_le_bytes();
    bytes.to_vec()
}

/// UTF-8 string validation and parsing
///
/// Validate that a byte slice contains valid UTF-8
pub fn validate_utf8(bytes: &[u8]) -> Result<()> {
    match str::from_utf8(bytes) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new(kinds::ParseError(format!(
            "Invalid UTF-8 sequence: {}",
            e
        )))),
    }
}

/// Read a string from a byte array
///
/// This reads a length-prefixed string (used in WebAssembly names).
pub fn read_string(bytes: &[u8], pos: usize) -> Result<(String, usize)> {
    if pos >= bytes.len() {
        return Err(Error::new(kinds::ParseError(
            "String exceeds buffer bounds".to_string(),
        )));
    }

    // Read the string length
    let (str_len, len_size) = read_leb128_u32(bytes, pos)?;
    let str_start = pos + len_size;
    let str_end = str_start + str_len as usize;

    // Ensure the string fits in the buffer
    if str_end > bytes.len() {
        return Err(Error::new(kinds::ParseError(
            "String exceeds buffer bounds".to_string(),
        )));
    }

    // Extract the string bytes
    let string_bytes = &bytes[str_start..str_end];

    // Convert to a Rust string
    match str::from_utf8(string_bytes) {
        Ok(s) => Ok((s.to_string(), len_size + str_len as usize)),
        Err(e) => Err(Error::new(kinds::ParseError(format!(
            "Invalid UTF-8 in string: {}",
            e
        )))),
    }
}

/// Write a WebAssembly UTF-8 string (length prefixed)
pub fn write_string(value: &str) -> Vec<u8> {
    let mut result = Vec::new();

    // Write the length as LEB128
    let length = value.len() as u32;
    result.extend_from_slice(&write_leb128_u32(length));

    // Write the string bytes
    result.extend_from_slice(value.as_bytes());

    result
}

/// Read a vector from a byte array
///
/// This is a generic function that reads a length-prefixed vector from a byte array,
/// using the provided function to read each element.
pub fn read_vector<T, F>(bytes: &[u8], pos: usize, read_elem: F) -> Result<(Vec<T>, usize)>
where
    F: Fn(&[u8], usize) -> Result<(T, usize)>,
{
    // Read the vector length
    let (count, mut offset) = read_leb128_u32(bytes, pos)?;
    let mut result = Vec::with_capacity(count as usize);

    // Read each element
    for _ in 0..count {
        let (elem, elem_size) = read_elem(bytes, pos + offset)?;
        result.push(elem);
        offset += elem_size;
    }

    Ok((result, offset))
}

/// Write a vector to a byte array
///
/// This is a generic function that writes a length-prefixed vector to a byte array,
/// using the provided function to write each element.
pub fn write_vector<T, F>(elements: &[T], write_elem: F) -> Vec<u8>
where
    F: Fn(&T) -> Vec<u8>,
{
    let mut result = Vec::new();

    // Write the vector length
    result.extend_from_slice(&write_leb128_u32(elements.len() as u32));

    // Write each element
    for elem in elements {
        result.extend_from_slice(&write_elem(elem));
    }

    result
}

/// Read a section header from a byte array
///
/// Returns a tuple containing the section ID, size, and new position after the header.
/// The position should point to the start of the section content.
pub fn read_section_header(bytes: &[u8], pos: usize) -> Result<(u8, u32, usize)> {
    if pos >= bytes.len() {
        return Err(Error::new(kinds::ParseError(
            "Attempted to read past end of binary".to_string(),
        )));
    }

    // Read section ID
    let id = bytes[pos];

    // Read section size
    let (size, size_len) = read_leb128_u32(bytes, pos + 1)?;

    // Calculate new position (after section ID and size)
    let new_pos = pos + 1 + size_len;

    Ok((id, size, new_pos))
}

/// Write a section header to a byte array
///
/// Writes the section ID and content size as a LEB128 unsigned integer.
pub fn write_section_header(id: u8, content_size: u32) -> Vec<u8> {
    let mut result = Vec::new();

    // Write section ID
    result.push(id);

    // Write section size
    result.extend_from_slice(&write_leb128_u32(content_size));

    result
}

/// Parse a block type value
pub fn parse_block_type(bytes: &[u8], pos: usize) -> Result<(BlockType, usize)> {
    if pos >= bytes.len() {
        return Err(Error::new(kinds::ParseError(
            "Unexpected end of input when parsing block type".to_string(),
        )));
    }

    match bytes[pos] {
        0x40 => Ok((BlockType::None, 1)), // Empty block type
        0x7F => Ok((BlockType::Value(ValueType::I32), 1)),
        0x7E => Ok((BlockType::Value(ValueType::I64), 1)),
        0x7D => Ok((BlockType::Value(ValueType::F32), 1)),
        0x7C => Ok((BlockType::Value(ValueType::F64), 1)),
        0x70 => Ok((BlockType::Value(ValueType::FuncRef), 1)),
        0x6F => Ok((BlockType::Value(ValueType::ExternRef), 1)),
        byte => {
            if (byte & 0x80) != 0 {
                // It's a function type index (negative LEB128)
                let (value, size) = read_leb128_i32(bytes, pos)?;
                if value >= 0 {
                    return Err(Error::new(kinds::ParseError(format!(
                        "Invalid block type index: expected negative value, got {}",
                        value
                    ))));
                }
                Ok((BlockType::FuncType((-value - 1) as u32), size))
            } else {
                Err(Error::new(kinds::ParseError(format!(
                    "Invalid block type byte: 0x{:02x}",
                    byte
                ))))
            }
        }
    }
}

/// Read a Component Model value type from a byte array
pub fn read_component_valtype(
    bytes: &[u8],
    pos: usize,
) -> Result<(crate::component::ValType, usize)> {
    use crate::component::ValType;

    if pos >= bytes.len() {
        return Err(Error::new(kinds::ParseError(
            "Unexpected end of input when reading component value type".to_string(),
        )));
    }

    let byte = bytes[pos];
    let mut new_pos = pos + 1;

    match byte {
        COMPONENT_VALTYPE_BOOL => Ok((ValType::Bool, new_pos)),
        COMPONENT_VALTYPE_S8 => Ok((ValType::S8, new_pos)),
        COMPONENT_VALTYPE_U8 => Ok((ValType::U8, new_pos)),
        COMPONENT_VALTYPE_S16 => Ok((ValType::S16, new_pos)),
        COMPONENT_VALTYPE_U16 => Ok((ValType::U16, new_pos)),
        COMPONENT_VALTYPE_S32 => Ok((ValType::S32, new_pos)),
        COMPONENT_VALTYPE_U32 => Ok((ValType::U32, new_pos)),
        COMPONENT_VALTYPE_S64 => Ok((ValType::S64, new_pos)),
        COMPONENT_VALTYPE_U64 => Ok((ValType::U64, new_pos)),
        COMPONENT_VALTYPE_F32 => Ok((ValType::F32, new_pos)),
        COMPONENT_VALTYPE_F64 => Ok((ValType::F64, new_pos)),
        COMPONENT_VALTYPE_CHAR => Ok((ValType::Char, new_pos)),
        COMPONENT_VALTYPE_STRING => Ok((ValType::String, new_pos)),
        COMPONENT_VALTYPE_REF => {
            let (idx, next_pos) = read_leb128_u32(bytes, new_pos)?;
            Ok((ValType::Ref(idx), next_pos))
        }
        COMPONENT_VALTYPE_RECORD => {
            let (count, next_pos) = read_leb128_u32(bytes, new_pos)?;
            new_pos = next_pos;

            let mut fields = Vec::with_capacity(count as usize);
            for _ in 0..count {
                let (name, next_pos) = read_string(bytes, new_pos)?;
                new_pos = next_pos;

                let (val_type, next_pos) = read_component_valtype(bytes, new_pos)?;
                new_pos = next_pos;

                fields.push((name, val_type));
            }

            Ok((ValType::Record(fields), new_pos))
        }
        COMPONENT_VALTYPE_VARIANT => {
            let (count, next_pos) = read_leb128_u32(bytes, new_pos)?;
            new_pos = next_pos;

            let mut cases = Vec::with_capacity(count as usize);
            for _ in 0..count {
                let (name, next_pos) = read_string(bytes, new_pos)?;
                new_pos = next_pos;

                let (has_type, next_pos) = read_leb128_u32(bytes, new_pos)?;
                new_pos = next_pos;

                let val_type = if has_type == 1 {
                    let (val_type, next_pos) = read_component_valtype(bytes, new_pos)?;
                    new_pos = next_pos;
                    Some(val_type)
                } else {
                    None
                };

                cases.push((name, val_type));
            }

            Ok((ValType::Variant(cases), new_pos))
        }
        COMPONENT_VALTYPE_LIST => {
            let (val_type, next_pos) = read_component_valtype(bytes, new_pos)?;
            Ok((ValType::List(Box::new(val_type)), next_pos))
        }
        COMPONENT_VALTYPE_FIXED_LIST => {
            let (val_type, next_pos) = read_component_valtype(bytes, new_pos)?;
            new_pos = next_pos;

            let (length, next_pos) = read_leb128_u32(bytes, new_pos)?;
            Ok((ValType::FixedList(Box::new(val_type), length), next_pos))
        }
        COMPONENT_VALTYPE_TUPLE => {
            let (count, next_pos) = read_leb128_u32(bytes, new_pos)?;
            new_pos = next_pos;

            let mut elements = Vec::with_capacity(count as usize);
            for _ in 0..count {
                let (val_type, next_pos) = read_component_valtype(bytes, new_pos)?;
                new_pos = next_pos;
                elements.push(val_type);
            }

            Ok((ValType::Tuple(elements), new_pos))
        }
        COMPONENT_VALTYPE_FLAGS => {
            let (count, next_pos) = read_leb128_u32(bytes, new_pos)?;
            new_pos = next_pos;

            let mut names = Vec::with_capacity(count as usize);
            for _ in 0..count {
                let (name, next_pos) = read_string(bytes, new_pos)?;
                new_pos = next_pos;
                names.push(name);
            }

            Ok((ValType::Flags(names), new_pos))
        }
        COMPONENT_VALTYPE_ENUM => {
            let (count, next_pos) = read_leb128_u32(bytes, new_pos)?;
            new_pos = next_pos;

            let mut names = Vec::with_capacity(count as usize);
            for _ in 0..count {
                let (name, next_pos) = read_string(bytes, new_pos)?;
                new_pos = next_pos;
                names.push(name);
            }

            Ok((ValType::Enum(names), new_pos))
        }
        COMPONENT_VALTYPE_OPTION => {
            let (val_type, next_pos) = read_component_valtype(bytes, new_pos)?;
            Ok((ValType::Option(Box::new(val_type)), next_pos))
        }
        COMPONENT_VALTYPE_RESULT => {
            let (val_type, next_pos) = read_component_valtype(bytes, new_pos)?;
            Ok((ValType::Result(Box::new(val_type)), next_pos))
        }
        COMPONENT_VALTYPE_RESULT_ERR => {
            let (val_type, next_pos) = read_component_valtype(bytes, new_pos)?;
            Ok((ValType::ResultErr(Box::new(val_type)), next_pos))
        }
        COMPONENT_VALTYPE_RESULT_BOTH => {
            let (ok_type, next_pos) = read_component_valtype(bytes, new_pos)?;
            new_pos = next_pos;

            let (err_type, next_pos) = read_component_valtype(bytes, new_pos)?;
            Ok((
                ValType::ResultBoth(Box::new(ok_type), Box::new(err_type)),
                next_pos,
            ))
        }
        COMPONENT_VALTYPE_OWN => {
            let (idx, next_pos) = read_leb128_u32(bytes, new_pos)?;
            Ok((ValType::Own(idx), next_pos))
        }
        COMPONENT_VALTYPE_BORROW => {
            let (idx, next_pos) = read_leb128_u32(bytes, new_pos)?;
            Ok((ValType::Borrow(idx), next_pos))
        }
        COMPONENT_VALTYPE_ERROR_CONTEXT => Ok((ValType::ErrorContext, new_pos)),
        _ => Err(Error::new(kinds::ParseError(format!(
            "Invalid component value type: 0x{:02x}",
            byte
        )))),
    }
}

/// Write a Component Model value type to a byte array
pub fn write_component_valtype(val_type: &crate::component::ValType) -> Vec<u8> {
    use crate::component::ValType;

    let mut bytes = Vec::new();

    match val_type {
        ValType::Bool => bytes.push(COMPONENT_VALTYPE_BOOL),
        ValType::S8 => bytes.push(COMPONENT_VALTYPE_S8),
        ValType::U8 => bytes.push(COMPONENT_VALTYPE_U8),
        ValType::S16 => bytes.push(COMPONENT_VALTYPE_S16),
        ValType::U16 => bytes.push(COMPONENT_VALTYPE_U16),
        ValType::S32 => bytes.push(COMPONENT_VALTYPE_S32),
        ValType::U32 => bytes.push(COMPONENT_VALTYPE_U32),
        ValType::S64 => bytes.push(COMPONENT_VALTYPE_S64),
        ValType::U64 => bytes.push(COMPONENT_VALTYPE_U64),
        ValType::F32 => bytes.push(COMPONENT_VALTYPE_F32),
        ValType::F64 => bytes.push(COMPONENT_VALTYPE_F64),
        ValType::Char => bytes.push(COMPONENT_VALTYPE_CHAR),
        ValType::String => bytes.push(COMPONENT_VALTYPE_STRING),
        ValType::Ref(idx) => {
            bytes.push(COMPONENT_VALTYPE_REF);
            bytes.extend_from_slice(&write_leb128_u32(*idx));
        }
        ValType::Record(fields) => {
            bytes.push(COMPONENT_VALTYPE_RECORD);
            bytes.extend_from_slice(&write_leb128_u32(fields.len() as u32));

            for (name, val_type) in fields {
                bytes.extend_from_slice(&write_string(name));
                bytes.extend_from_slice(&write_component_valtype(val_type));
            }
        }
        ValType::Variant(cases) => {
            bytes.push(COMPONENT_VALTYPE_VARIANT);
            bytes.extend_from_slice(&write_leb128_u32(cases.len() as u32));

            for (name, val_type) in cases {
                bytes.extend_from_slice(&write_string(name));

                match val_type {
                    Some(ty) => {
                        bytes.extend_from_slice(&write_leb128_u32(1));
                        bytes.extend_from_slice(&write_component_valtype(ty));
                    }
                    None => {
                        bytes.extend_from_slice(&write_leb128_u32(0));
                    }
                }
            }
        }
        ValType::List(val_type) => {
            bytes.push(COMPONENT_VALTYPE_LIST);
            bytes.extend_from_slice(&write_component_valtype(val_type));
        }
        ValType::FixedList(val_type, length) => {
            bytes.push(COMPONENT_VALTYPE_FIXED_LIST);
            bytes.extend_from_slice(&write_component_valtype(val_type));
            bytes.extend_from_slice(&write_leb128_u32(*length));
        }
        ValType::Tuple(elements) => {
            bytes.push(COMPONENT_VALTYPE_TUPLE);
            bytes.extend_from_slice(&write_leb128_u32(elements.len() as u32));

            for val_type in elements {
                bytes.extend_from_slice(&write_component_valtype(val_type));
            }
        }
        ValType::Flags(names) => {
            bytes.push(COMPONENT_VALTYPE_FLAGS);
            bytes.extend_from_slice(&write_leb128_u32(names.len() as u32));

            for name in names {
                bytes.extend_from_slice(&write_string(name));
            }
        }
        ValType::Enum(names) => {
            bytes.push(COMPONENT_VALTYPE_ENUM);
            bytes.extend_from_slice(&write_leb128_u32(names.len() as u32));

            for name in names {
                bytes.extend_from_slice(&write_string(name));
            }
        }
        ValType::Option(val_type) => {
            bytes.push(COMPONENT_VALTYPE_OPTION);
            bytes.extend_from_slice(&write_component_valtype(val_type));
        }
        ValType::Result(val_type) => {
            bytes.push(COMPONENT_VALTYPE_RESULT);
            bytes.extend_from_slice(&write_component_valtype(val_type));
        }
        ValType::ResultErr(val_type) => {
            bytes.push(COMPONENT_VALTYPE_RESULT_ERR);
            bytes.extend_from_slice(&write_component_valtype(val_type));
        }
        ValType::ResultBoth(ok_type, err_type) => {
            bytes.push(COMPONENT_VALTYPE_RESULT_BOTH);
            bytes.extend_from_slice(&write_component_valtype(ok_type));
            bytes.extend_from_slice(&write_component_valtype(err_type));
        }
        ValType::Own(idx) => {
            bytes.push(COMPONENT_VALTYPE_OWN);
            bytes.extend_from_slice(&write_leb128_u32(*idx));
        }
        ValType::Borrow(idx) => {
            bytes.push(COMPONENT_VALTYPE_BORROW);
            bytes.extend_from_slice(&write_leb128_u32(*idx));
        }
        ValType::ErrorContext => bytes.push(COMPONENT_VALTYPE_ERROR_CONTEXT),
    }

    bytes
}

/// Parse a WebAssembly component binary into a component
pub fn parse_component_binary(bytes: &[u8]) -> Result<crate::component::Component> {
    // Verify magic bytes
    if bytes.len() < 8 {
        return Err(Error::new(kinds::ParseError(
            "WebAssembly component binary too short".to_string(),
        )));
    }

    if bytes[0..4] != COMPONENT_MAGIC {
        return Err(Error::new(kinds::ParseError(
            "Invalid WebAssembly component magic bytes".to_string(),
        )));
    }

    // Verify component version and layer
    if bytes[4..6] != COMPONENT_VERSION_ONLY {
        return Err(Error::new(kinds::ParseError(
            "Unsupported WebAssembly component version".to_string(),
        )));
    }

    if bytes[6..8] != COMPONENT_LAYER {
        return Err(Error::new(kinds::ParseError(
            "Invalid WebAssembly component layer".to_string(),
        )));
    }

    // Create an empty component with the binary stored
    let mut component = crate::component::Component::new();
    component.binary = Some(bytes.to_vec());

    // In a real implementation, we would parse the sections here
    // This will be fully implemented in the future

    Ok(component)
}

/// Generate a WebAssembly component binary from a component
pub fn generate_component_binary(component: &crate::component::Component) -> Result<Vec<u8>> {
    // If we have the original binary and haven't modified the component,
    // we can just return it
    if let Some(binary) = &component.binary {
        return Ok(binary.clone());
    }

    // Create a minimal valid component
    let mut binary = Vec::with_capacity(8);

    // Magic bytes
    binary.extend_from_slice(&COMPONENT_MAGIC);

    // Version and layer
    binary.extend_from_slice(&COMPONENT_VERSION);

    // Generate sections
    // This is a placeholder - full implementation will be added in the future

    // In a complete implementation, we would encode all sections:
    // - Core module sections
    // - Core instance sections
    // - Core type sections
    // - Component sections
    // - Instance sections
    // - Alias sections
    // - Type sections
    // - Canon sections
    // - Start sections
    // - Import sections
    // - Export sections
    // - Value sections

    Ok(binary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_roundtrip() {
        let values = [
            0.0f32,
            -0.0,
            1.0,
            -1.0,
            3.14159,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NAN,
        ];

        for &value in &values {
            let bytes = write_f32(value);
            let (decoded, size) = read_f32(&bytes, 0).unwrap();

            assert_eq!(size, 4);
            if value.is_nan() {
                assert!(decoded.is_nan());
            } else {
                assert_eq!(decoded, value);
            }
        }
    }

    #[test]
    fn test_f64_roundtrip() {
        let values = [
            0.0f64,
            -0.0,
            1.0,
            -1.0,
            3.14159265358979,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NAN,
        ];

        for &value in &values {
            let bytes = write_f64(value);
            let (decoded, size) = read_f64(&bytes, 0).unwrap();

            assert_eq!(size, 8);
            if value.is_nan() {
                assert!(decoded.is_nan());
            } else {
                assert_eq!(decoded, value);
            }
        }
    }

    #[test]
    fn test_string_roundtrip() {
        let test_strings = [
            "",
            "Hello, World!",
            "UTF-8 test: ñáéíóú",
            "🦀 Rust is awesome!",
        ];

        for &s in &test_strings {
            let bytes = write_string(s);
            let (decoded, _) = read_string(&bytes, 0).unwrap();

            assert_eq!(decoded, s);
        }
    }

    #[test]
    fn test_leb128_u64_roundtrip() {
        let test_values = [
            0u64,
            1,
            127,
            128,
            16384,
            0x7FFFFFFF,
            0xFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
        ];

        for &value in &test_values {
            let bytes = write_leb128_u64(value);
            let (decoded, _) = read_leb128_u64(&bytes, 0).unwrap();

            assert_eq!(decoded, value);
        }
    }

    #[test]
    fn test_utf8_validation() {
        // Valid UTF-8
        assert!(validate_utf8(b"Hello").is_ok());
        assert!(validate_utf8("🦀 Rust".as_bytes()).is_ok());

        // Invalid UTF-8
        let invalid_utf8 = [0xFF, 0xFE, 0xFD];
        assert!(validate_utf8(&invalid_utf8).is_err());
    }

    #[test]
    fn test_read_write_vector() {
        // Create a test vector of u32 values
        let values = vec![1u32, 42, 100, 1000];

        // Write the vector
        let bytes = write_vector(&values, |v| write_leb128_u32(*v));

        // Read the vector back
        let (decoded, _) = read_vector(&bytes, 0, read_leb128_u32).unwrap();

        assert_eq!(values, decoded);
    }

    #[test]
    fn test_section_header() {
        // Create a section header for a type section with 10 bytes of content
        let section_id = TYPE_SECTION_ID;
        let content_size = 10;

        let bytes = write_section_header(section_id, content_size);

        // Read the section header back
        let (decoded_id, decoded_size, _) = read_section_header(&bytes, 0).unwrap();

        assert_eq!(section_id, decoded_id);
        assert_eq!(content_size, decoded_size);
    }
}
