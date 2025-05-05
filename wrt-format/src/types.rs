//! WebAssembly type definitions.
//!
//! This module provides type definitions for WebAssembly types.
//! Most core types are re-exported from wrt-types.

use wrt_error::Result;
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
    /// Whether this limit is for a 64-bit memory
    pub memory64: bool,
}

/// Parser-specific block type for binary format
#[derive(Debug, Clone, PartialEq)]
pub enum FormatBlockType {
    /// No return value (void)
    Empty,
    /// Single return value
    ValueType(ValueType),
    /// Function type reference
    TypeIndex(u32),
    /// Function type (used for complex block types)
    FuncType(FuncType),
}

impl From<FormatBlockType> for BlockType {
    fn from(bt: FormatBlockType) -> Self {
        match bt {
            FormatBlockType::Empty => BlockType::Empty,
            FormatBlockType::ValueType(vt) => BlockType::Value(vt),
            FormatBlockType::TypeIndex(idx) => BlockType::TypeIndex(idx),
            FormatBlockType::FuncType(func_type) => BlockType::FuncType(func_type),
        }
    }
}

/// Parse a value type byte to a ValueType enum using the conversion module
pub fn parse_value_type(byte: u8) -> Result<ValueType> {
    crate::conversion::parse_value_type(byte)
}

/// Convert a ValueType to its binary representation using the conversion module
pub fn value_type_to_byte(value_type: ValueType) -> u8 {
    crate::conversion::format_value_type(value_type)
}
