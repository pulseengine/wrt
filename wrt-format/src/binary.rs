//! WebAssembly binary format handling.
//!
//! This module provides utilities for parsing and generating WebAssembly binary format.

use crate::module::Module;
use crate::types::{BlockType, ValueType};
use crate::{format, String, Vec};
use wrt_error::{kinds, Error, Result};

#[cfg(feature = "std")]
use std::str;

#[cfg(not(feature = "std"))]
use core::str;

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec};

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

/// Parse a block type from binary
pub fn parse_block_type(bytes: &[u8], pos: usize) -> Result<(BlockType, usize)> {
    if pos >= bytes.len() {
        return Err(Error::new(kinds::ParseError(
            "Unexpected end of block type bytes".to_string(),
        )));
    }

    match bytes[pos] {
        0x40 => Ok((BlockType::Empty, 1)), // Empty return type
        0x7F => Ok((BlockType::Value(ValueType::I32), 1)),
        0x7E => Ok((BlockType::Value(ValueType::I64), 1)),
        0x7D => Ok((BlockType::Value(ValueType::F32), 1)),
        0x7C => Ok((BlockType::Value(ValueType::F64), 1)),
        0x7B => Ok((BlockType::Value(ValueType::V128), 1)),
        0x70 => Ok((BlockType::Value(ValueType::FuncRef), 1)),
        0x6F => Ok((BlockType::Value(ValueType::ExternRef), 1)),
        _ => {
            // If not a value type, it's an index to a function type
            // Read as signed LEB128 (negative indices have special meanings)
            let (type_idx, bytes_read) = read_leb128_i32(bytes, pos)?;
            Ok((BlockType::FuncType(type_idx as u32), bytes_read))
        }
    }
}

#[cfg(feature = "kani")]
mod verification {
    use super::*;
    use kani::*;

    #[kani::proof]
    fn verify_leb128_u32_roundtrip() {
        let value: u32 = any();

        // Encode
        let encoded = write_leb128_u32(value);

        // Decode
        let (decoded, _) = read_leb128_u32(&encoded, 0).unwrap();

        // Verify roundtrip
        assert_eq!(value, decoded);
    }

    #[kani::proof]
    fn verify_leb128_i32_roundtrip() {
        let value: i32 = any();

        // Encode
        let encoded = write_leb128_i32(value);

        // Decode
        let (decoded, _) = read_leb128_i32(&encoded, 0).unwrap();

        // Verify roundtrip
        assert_eq!(value, decoded);
    }
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
