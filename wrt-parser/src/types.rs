//! WebAssembly type definitions for the parser
//!
//! This module contains type definitions used throughout the parser,
//! including value types, function types, and other WebAssembly types.

use core::fmt;
use wrt_error::{Error, ErrorCategory, Result, codes};

/// WebAssembly value types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ValueType {
    #[default]
    I32,
    I64,
    F32,
    F64,
    V128,
    FuncRef,
    ExternRef,
}

impl ValueType {
    /// Parse a value type from a byte
    pub fn from_byte(byte: u8) -> Result<Self> {
        match byte {
            0x7F => Ok(ValueType::I32),
            0x7E => Ok(ValueType::I64),
            0x7D => Ok(ValueType::F32),
            0x7C => Ok(ValueType::F64),
            0x7B => Ok(ValueType::V128),
            0x70 => Ok(ValueType::FuncRef),
            0x6F => Ok(ValueType::ExternRef),
            _ => Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Invalid value type"
            )),
        }
    }
    
    /// Convert a value type to its byte representation
    pub fn to_byte(self) -> u8 {
        match self {
            ValueType::I32 => 0x7F,
            ValueType::I64 => 0x7E,
            ValueType::F32 => 0x7D,
            ValueType::F64 => 0x7C,
            ValueType::V128 => 0x7B,
            ValueType::FuncRef => 0x70,
            ValueType::ExternRef => 0x6F,
        }
    }
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueType::I32 => write!(f, "i32"),
            ValueType::I64 => write!(f, "i64"),
            ValueType::F32 => write!(f, "f32"),
            ValueType::F64 => write!(f, "f64"),
            ValueType::V128 => write!(f, "v128"),
            ValueType::FuncRef => write!(f, "funcref"),
            ValueType::ExternRef => write!(f, "externref"),
        }
    }
}


/// WebAssembly block types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockType {
    Empty,
    Value(ValueType),
    Type(u32), // Type index
}

impl BlockType {
    /// Parse a block type from LEB128 encoded data
    pub fn from_leb128(data: &[u8], offset: usize) -> Result<(Self, usize)> {
        let (value, bytes_read) = crate::leb128::read_leb128_i32(data, offset)?;
        
        let block_type = match value {
            -64 => BlockType::Empty, // 0x40 as signed LEB128
            -1 => BlockType::Value(ValueType::I32),
            -2 => BlockType::Value(ValueType::I64),
            -3 => BlockType::Value(ValueType::F32),
            -4 => BlockType::Value(ValueType::F64),
            -5 => BlockType::Value(ValueType::V128),
            -16 => BlockType::Value(ValueType::FuncRef),
            -17 => BlockType::Value(ValueType::ExternRef),
            x if x >= 0 => BlockType::Type(x as u32),
            _ => return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Invalid block type"
            )),
        };
        
        Ok((block_type, bytes_read))
    }
}

/// WebAssembly function type
#[derive(Debug, Clone, Default)]
pub struct FuncType {
    pub params: crate::bounded_types::SimpleBoundedVec<ValueType, 32>,
    pub results: crate::bounded_types::SimpleBoundedVec<ValueType, 32>,
}


/// WebAssembly global type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalType {
    pub value_type: ValueType,
    pub mutable: bool,
}

/// WebAssembly memory type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryType {
    pub limits: Limits,
}

/// WebAssembly table type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableType {
    pub element_type: ValueType,
    pub limits: Limits,
}

/// WebAssembly limits
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

impl Limits {
    /// Parse limits from binary data
    pub fn parse(data: &[u8], offset: usize) -> Result<(Self, usize)> {
        let mut current_offset = offset;
        
        if current_offset >= data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Unexpected end of data while reading limits"
            ));
        }
        
        let flags = data[current_offset];
        current_offset += 1;
        
        let (min, bytes_read) = crate::leb128::read_leb128_u32(data, current_offset)?;
        current_offset += bytes_read;
        
        let max = if flags & 0x01 != 0 {
            let (max_val, bytes_read) = crate::leb128::read_leb128_u32(data, current_offset)?;
            current_offset += bytes_read;
            Some(max_val)
        } else {
            None
        };
        
        Ok((Limits { min, max }, current_offset - offset))
    }
}