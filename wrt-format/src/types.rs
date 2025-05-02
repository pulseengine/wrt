//! WebAssembly type definitions.
//!
//! This module provides type definitions for WebAssembly types.
//! Most core types are re-exported from wrt-types.

use crate::format;
use wrt_error::{codes, Error, ErrorCategory, Result};
// Import types from wrt-types
pub use wrt_types::{BlockType, FuncType, RefType, ValueType};

/// WebAssembly memory index type (standard or 64-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryIndexType {
    /// Standard WebAssembly 1.0 memory (i32 addressing)
    /// Limited to 4GiB (65536 pages × 64KiB)
    I32,
    /// Memory64 extension (i64 addressing)
    /// Allows for memories larger than 4GiB
    I64,
}

/// WebAssembly limits
///
/// Limits represent the minimum and optional maximum sizes for
/// memories and tables as defined in the WebAssembly Core Specification.
///
/// For memories, limits are specified in units of pages (64KiB each).
/// For tables, limits are specified in number of elements.
///
/// The WebAssembly 1.0 specification has the following constraints:
/// - For memories, the maximum number of pages is 65536 (4GiB)
/// - Shared memories must have a maximum size specified
/// - The maximum size must be greater than or equal to the minimum size
#[derive(Debug, Clone)]
pub struct Limits {
    /// Minimum size (pages for memory, elements for table)
    pub min: u64,
    /// Maximum size (optional, required for shared memories)
    pub max: Option<u64>,
    /// Shared memory flag, used for memory types
    /// When true, memory can be shared between threads and requires max to be set
    pub shared: bool,
    /// Memory index type (i32 or i64)
    /// Standard WebAssembly 1.0 uses i32 addressing (up to 4GiB)
    /// The Memory64 extension uses i64 addressing (beyond 4GiB)
    pub memory_index_type: MemoryIndexType,
}

/// Parser-specific block type for binary format
#[derive(Debug, Clone, PartialEq)]
pub enum FormatBlockType {
    /// No return value (void)
    None,
    /// Single return value
    Value(ValueType),
    /// Function type reference
    FuncType(u32),
    /// I32 value type (used in binary format)
    I32,
    /// I64 value type (used in binary format)
    I64,
    /// F32 value type (used in binary format)
    F32,
    /// F64 value type (used in binary format)
    F64,
}

impl From<FormatBlockType> for BlockType {
    fn from(bt: FormatBlockType) -> Self {
        match bt {
            FormatBlockType::None => BlockType::Empty,
            FormatBlockType::Value(vt) => BlockType::Value(vt),
            FormatBlockType::FuncType(idx) => BlockType::TypeIndex(idx),
            FormatBlockType::I32 => BlockType::Value(ValueType::I32),
            FormatBlockType::I64 => BlockType::Value(ValueType::I64),
            FormatBlockType::F32 => BlockType::Value(ValueType::F32),
            FormatBlockType::F64 => BlockType::Value(ValueType::F64),
        }
    }
}

/// Parse a value type byte to a ValueType enum
pub fn parse_value_type(byte: u8) -> Result<ValueType> {
    match byte {
        0x7F => Ok(ValueType::I32),
        0x7E => Ok(ValueType::I64),
        0x7D => Ok(ValueType::F32),
        0x7C => Ok(ValueType::F64),
        0x70 => Ok(ValueType::FuncRef),
        0x6F => Ok(ValueType::ExternRef),
        _ => Err(Error::new(
            ErrorCategory::Validation,
            codes::PARSE_ERROR,
            format!("Invalid value type byte: 0x{:02x}", byte),
        )),
    }
}

/// Convert a ValueType to its binary representation
pub fn value_type_to_byte(value_type: ValueType) -> u8 {
    match value_type {
        ValueType::I32 => 0x7F,
        ValueType::I64 => 0x7E,
        ValueType::F32 => 0x7D,
        ValueType::F64 => 0x7C,
        ValueType::FuncRef => 0x70,
        ValueType::ExternRef => 0x6F,
    }
}
