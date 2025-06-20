//! Simplified WebAssembly type definitions for initial implementation
//!
//! This module contains basic type definitions for WebAssembly parsing
//! without the complex trait requirements of BoundedVec.

use core::fmt;
use wrt_error::{Error, ErrorCategory, Result, codes};

/// WebAssembly value types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueType {
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

/// Simplified WebAssembly module structure
#[derive(Debug, Clone)]
pub struct WasmModule {
    pub function_count: u32,
    pub type_count: u32,
}

impl WasmModule {
    /// Create a new empty module
    pub fn new() -> Self {
        WasmModule {
            function_count: 0,
            type_count: 0,
        }
    }
}

impl Default for WasmModule {
    fn default() -> Self {
        Self::new()
    }
}