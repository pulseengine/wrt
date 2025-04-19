//! WebAssembly section types and parsing utilities
//!
//! This module provides types and functions for parsing WebAssembly section contents.
//! It builds on the foundational types defined in wrt-format.

// Re-export relevant types from wrt-format
pub use wrt_format::module::{
    Data, Element, Export, ExportKind, Function, Global, Import, ImportDesc, Memory, Module, Table,
};
pub use wrt_format::section::{CustomSection, Section, SectionId};
pub use wrt_format::types::{FuncType, Limits, ValueType};

// Local imports
use crate::{String, Vec};
use wrt_error::{kinds, Error, Result};
use wrt_format::binary;
use wrt_format::types::parse_value_type;

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
    pub fn parse_memory_type(bytes: &[u8]) -> Result<(Memory, usize)> {
        let mut offset = 0;

        // Read limits
        let (limits, bytes_read) = parse_limits(&bytes[offset..])?;
        offset += bytes_read;

        Ok((Memory { limits }, offset))
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

        // Read min limit (LEB128)
        let (min, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        // Read max limit if flags indicate it's present
        let max = if flags & 0x01 != 0 {
            let (max_val, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;
            Some(max_val)
        } else {
            None
        };

        Ok((Limits { min, max }, offset))
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
