//! WebAssembly section types and parsing utilities
//!
//! This module provides types and functions for parsing WebAssembly section contents.
//! It builds on the foundational types defined in wrt-format.

// Re-export relevant types from wrt-format
pub use wrt_format::module::{
    Data, DataMode, Element, Export, ExportKind, Function, Global, Import, ImportDesc, Memory,
    Module, Table,
};
pub use wrt_format::section::{CustomSection, Section, SectionId};
pub use wrt_format::types::{FuncType, Limits, MemoryIndexType, ValueType};

// Local imports
use crate::{String, Vec};
use wrt_error::{kinds, Error, Result};
use wrt_format::binary;
use wrt_format::types::parse_value_type;
// Use our prelude for common imports
use crate::prelude::*;

// Imports from rest of crate

// Create a module structure to organize section parsing code
pub mod parsers {
    //! Section-specific parsers

    use super::*;

    /// Parse a code entry
    pub fn parse_code(bytes: &[u8]) -> Result<(Code, usize)> {
        let mut offset = 0;

        // Read the size of locals
        let (size, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        if (offset + size as usize) > bytes.len() {
            return Err(Error::new(kinds::ParseError(
                "Code size exceeds available bytes".to_string(),
            )));
        }

        // For now, just store the code as raw bytes
        let code_bytes = bytes[offset..(offset + size as usize)].to_vec();
        offset += size as usize;

        let code = Code {
            size,
            locals: Vec::new(), // Will be parsed later
            body: code_bytes,
        };

        Ok((code, offset))
    }

    /// Parse a function type
    pub fn parse_func_type(bytes: &[u8]) -> Result<(FuncType, usize)> {
        let mut offset = 0;

        // Check for function type marker (0x60)
        if bytes.is_empty() || bytes[0] != 0x60 {
            return Err(Error::new(kinds::ParseError(
                "Invalid function type".to_string(),
            )));
        }
        offset += 1;

        // Read parameter count
        let (param_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        // Read parameter types
        let mut params = Vec::with_capacity(param_count as usize);
        for _ in 0..param_count {
            if offset >= bytes.len() {
                return Err(Error::new(kinds::ParseError(
                    "Unexpected end of parameter types".to_string(),
                )));
            }

            let val_type = parse_value_type(bytes[offset])?;
            params.push(val_type);
            offset += 1;
        }

        // Read result count
        let (result_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        // Read result types
        let mut results = Vec::with_capacity(result_count as usize);
        for _ in 0..result_count {
            if offset >= bytes.len() {
                return Err(Error::new(kinds::ParseError(
                    "Unexpected end of result types".to_string(),
                )));
            }

            let val_type = parse_value_type(bytes[offset])?;
            results.push(val_type);
            offset += 1;
        }

        Ok((FuncType { params, results }, offset))
    }

    /// Parse a table type
    pub fn parse_table_type(bytes: &[u8]) -> Result<(Table, usize)> {
        let mut offset = 0;

        // Read element type
        if offset >= bytes.len() {
            return Err(Error::new(kinds::ParseError(
                "Unexpected end of table type bytes".to_string(),
            )));
        }

        let element_type = parse_value_type(bytes[offset])?;
        offset += 1;

        // Read limits
        let (limits, bytes_read) = parse_limits(&bytes[offset..])?;
        offset += bytes_read;

        Ok((
            Table {
                element_type,
                limits,
            },
            offset,
        ))
    }

    /// Parse a memory type
    ///
    /// According to the WebAssembly Core Specification (Binary Format):
    /// https://webassembly.github.io/spec/core/bikeshed/#binary-memtype
    ///
    /// The memory type has the following format:
    /// - Flags byte:
    ///   - bit 0 = has_max
    ///   - bit 1 = is_shared (shared memory extension)
    ///   - bit 2 = is_memory64 (memory64 extension)
    ///   - bits 3-7 reserved (must be 0)
    /// - Min size: u32 (memory32) or u64 (memory64) in units of pages (64KiB)
    /// - Max size: Optional u32 (memory32) or u64 (memory64) in units of pages (present if has_max)
    ///
    /// Validation rules:
    /// - Shared memories must have a maximum size specified
    /// - For memory32, min and max must not exceed 65536 pages (4GiB)
    /// - The maximum size must be greater than or equal to the minimum size for shared memories
    pub fn parse_memory_type(bytes: &[u8]) -> Result<(Memory, usize)> {
        let mut offset = 0;

        // Read flags
        if offset >= bytes.len() {
            return Err(Error::new(kinds::ParseError(
                "Unexpected end of memory type bytes".to_string(),
            )));
        }
        let flags = bytes[offset];
        offset += 1;

        // Check if flags are valid
        // For memory64, 0x04 bit is used to indicate 64-bit indexing
        // According to the spec, reserved bits must be 0
        if flags & 0xF8 != 0 {
            return Err(Error::new(kinds::ParseError(
                "Invalid memory flags, reserved bits must be 0 (per WebAssembly spec)".to_string(),
            )));
        }

        // Extract flags
        let has_max = (flags & 0x01) != 0;
        let is_shared = (flags & 0x02) != 0;
        let is_memory64 = (flags & 0x04) != 0;

        let memory_index_type = if is_memory64 {
            MemoryIndexType::I64
        } else {
            MemoryIndexType::I32
        };

        // Shared memories must have max specified (shared = 0x03)
        // Per WebAssembly spec, shared memories must specify maximum size
        if is_shared && !has_max {
            return Err(Error::new(kinds::ParseError(
                "Shared memory must have maximum size specified (per WebAssembly spec)".to_string(),
            )));
        }

        // Read min limit (LEB128)
        let min;
        let bytes_read;

        if is_memory64 {
            let (value, read) = binary::read_leb128_u64(bytes, offset)?;
            min = value;
            bytes_read = read;

            // Validate minimum size for memory64 (implementation-defined, but should be reasonable)
            // WebAssembly spec doesn't define specific limits for memory64, but implementations should check
            if min > (1u64 << 48) {
                return Err(Error::new(kinds::ParseError(
                    "Memory64 minimum size exceeds implementation limit (2^48)".to_string(),
                )));
            }
        } else {
            let (value, read) = binary::read_leb128_u32(bytes, offset)?;
            min = value as u64;
            bytes_read = read;

            // Validate minimum size for memory32 (per WebAssembly spec)
            // In WebAssembly 1.0, memories are limited to 4GiB (max pages = 65536)
            if min > 65536 {
                return Err(Error::new(kinds::ParseError(
                    "Memory32 minimum size exceeds WebAssembly limit of 65536 pages".to_string(),
                )));
            }
        }

        offset += bytes_read;

        // Read max limit if flags indicate it's present
        let max = if has_max {
            let max_val;
            let bytes_read;

            if is_memory64 {
                let (value, read) = binary::read_leb128_u64(bytes, offset)?;
                max_val = value;
                bytes_read = read;

                // Validate maximum size for memory64 (implementation-defined, but should be reasonable)
                if max_val > (1u64 << 48) {
                    return Err(Error::new(kinds::ParseError(
                        "Memory64 maximum size exceeds implementation limit (2^48)".to_string(),
                    )));
                }
            } else {
                let (value, read) = binary::read_leb128_u32(bytes, offset)?;
                max_val = value as u64;
                bytes_read = read;

                // Validate maximum size for memory32 (per WebAssembly spec)
                if max_val > 65536 {
                    return Err(Error::new(kinds::ParseError(
                        "Memory32 maximum size exceeds WebAssembly limit of 65536 pages"
                            .to_string(),
                    )));
                }
            }

            offset += bytes_read;

            // Verify max >= min for shared memory
            if is_shared && max_val < min {
                return Err(Error::new(kinds::ParseError(
                    "Shared memory maximum size must be greater than or equal to minimum size (per WebAssembly spec)"
                        .to_string(),
                )));
            }

            Some(max_val)
        } else {
            None
        };

        // Create limits
        let limits = Limits {
            min,
            max,
            shared: is_shared,
            memory_index_type,
        };

        Ok((
            Memory {
                limits,
                shared: is_shared,
            },
            offset,
        ))
    }

    /// Parse limits (used by table and memory)
    pub fn parse_limits(bytes: &[u8]) -> Result<(Limits, usize)> {
        let mut offset = 0;

        // Read flags
        if offset >= bytes.len() {
            return Err(Error::new(kinds::ParseError(
                "Unexpected end of limits bytes".to_string(),
            )));
        }
        let flags = bytes[offset];
        offset += 1;

        // For standard limits with potential memory64
        // Bit 0: has_max
        // Bit 1: is_shared (memory only)
        // Bit 2: is_memory64 (memory only)
        // Remaining bits should be zero
        if flags & 0xF8 != 0 {
            return Err(Error::new(kinds::ParseError(
                "Invalid limits flags, reserved bits must be 0".to_string(),
            )));
        }

        // Extract flags
        let has_max = (flags & 0x01) != 0;
        let is_shared = (flags & 0x02) != 0;
        let is_memory64 = (flags & 0x04) != 0;

        let memory_index_type = if is_memory64 {
            MemoryIndexType::I64
        } else {
            MemoryIndexType::I32
        };

        // Read min limit (LEB128)
        let min;
        let bytes_read;

        if is_memory64 {
            let (value, read) = binary::read_leb128_u64(bytes, offset)?;
            min = value;
            bytes_read = read;
        } else {
            let (value, read) = binary::read_leb128_u32(bytes, offset)?;
            min = value as u64;
            bytes_read = read;
        }

        offset += bytes_read;

        // Read max limit if flags indicate it's present
        let max = if has_max {
            let max_val;
            let bytes_read;

            if is_memory64 {
                let (value, read) = binary::read_leb128_u64(bytes, offset)?;
                max_val = value;
                bytes_read = read;
            } else {
                let (value, read) = binary::read_leb128_u32(bytes, offset)?;
                max_val = value as u64;
                bytes_read = read;
            }

            offset += bytes_read;
            Some(max_val)
        } else {
            None
        };

        Ok((
            Limits {
                min,
                max,
                shared: is_shared,
                memory_index_type,
            },
            offset,
        ))
    }

    /// Parse a string using wrt-format's binary utility
    pub fn parse_string(bytes: &[u8]) -> Result<(String, usize)> {
        binary::read_string(bytes, 0)
    }
}

/// Function code representation
#[derive(Debug, Clone)]
pub struct Code {
    /// Size of the code section entry
    pub size: u32,
    /// Local declarations (count, type pairs)
    pub locals: Vec<(u32, ValueType)>,
    /// Function body as raw bytes, will be parsed into instructions later
    pub body: Vec<u8>,
}
