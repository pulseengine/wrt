//! WebAssembly type definitions.
//!
//! This module provides type definitions for WebAssembly types.

use crate::{format, Vec};
use wrt_error::{kinds, Error, Result};

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

/// WebAssembly value types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
    /// 32-bit integer
    I32,
    /// 64-bit integer
    I64,
    /// 32-bit floating point
    F32,
    /// 64-bit floating point
    F64,
    /// 128-bit vector
    V128,
    /// Function reference
    FuncRef,
    /// External reference
    ExternRef,
}

/// WebAssembly function type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncType {
    /// Parameter types
    pub params: Vec<ValueType>,
    /// Result types
    pub results: Vec<ValueType>,
}

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

/// Represents a WebAssembly block type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockType {
    /// No values are returned
    Empty,
    /// A single value of the specified type is returned
    Value(ValueType),
    /// Multiple values are returned according to the function type
    FuncType(u32), // Function type index
}

/// Parse a value type from a byte
pub fn parse_value_type(byte: u8) -> Result<ValueType> {
    match byte {
        0x7F => Ok(ValueType::I32),
        0x7E => Ok(ValueType::I64),
        0x7D => Ok(ValueType::F32),
        0x7C => Ok(ValueType::F64),
        0x7B => Ok(ValueType::V128),
        0x70 => Ok(ValueType::FuncRef),
        0x6F => Ok(ValueType::ExternRef),
        _ => Err(Error::new(kinds::ParseError(format!(
            "Invalid value type: 0x{:02x}",
            byte
        )))),
    }
}

/// Convert a value type to its binary representation
pub fn value_type_to_byte(value_type: ValueType) -> u8 {
    match value_type {
        ValueType::I32 => 0x7F,
        ValueType::I64 => 0x7E,
        ValueType::F32 => 0x7D,
        ValueType::F64 => 0x7C,
        ValueType::V128 => 0x7B,
        ValueType::FuncRef => 0x70,
        ValueType::ExternRef => 0x6F,
    }
}
