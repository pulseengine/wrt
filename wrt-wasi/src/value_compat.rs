//! Value compatibility layer for WASI components
//!
//! This module provides a simplified Value enum that matches the interface
//! expected by WASI components while being compatible with wrt-foundation's
//! component value system.

use wrt_foundation::{
    safe_memory::NoStdProvider,
    bounded::BoundedVec,
    prelude::*,
};

/// Simplified Value enum for WASI component interface
/// 
/// This provides the variants that WASI code expects while being
/// compatible with wrt-foundation's type system.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Boolean value
    Bool(bool),
    /// Unsigned 8-bit integer
    U8(u8),
    /// Unsigned 16-bit integer  
    U16(u16),
    /// Unsigned 32-bit integer
    U32(u32),
    /// Unsigned 64-bit integer
    U64(u64),
    /// Signed 8-bit integer
    S8(i8),
    /// Signed 16-bit integer
    S16(i16),
    /// Signed 32-bit integer
    S32(i32),
    /// Signed 64-bit integer
    S64(i64),
    /// 32-bit floating point
    F32(f32),
    /// 64-bit floating point
    F64(f64),
    /// String value
    String(String),
    /// List of values
    List(Vec<Value>),
    /// Record with key-value pairs
    Record(Vec<(String, Value)>),
    /// Optional value
    Option(Option<Box<Value>>),
    /// Result value
    Result(core::result::Result<Box<Value>, Box<Value>>),
    /// Tuple of values
    Tuple(Vec<Value>),
}

impl Default for Value {
    fn default() -> Self {
        Value::U32(0)
    }
}

impl Value {
    /// Extract a u32 from the value, returning 0 if not possible
    pub fn as_u32(&self) -> u32 {
        match self {
            Value::U32(v) => *v,
            Value::U16(v) => *v as u32,
            Value::U8(v) => *v as u32,
            _ => 0,
        }
    }

    /// Extract a u64 from the value, returning 0 if not possible
    pub fn as_u64(&self) -> u64 {
        match self {
            Value::U64(v) => *v,
            Value::U32(v) => *v as u64,
            Value::U16(v) => *v as u64,
            Value::U8(v) => *v as u64,
            _ => 0,
        }
    }

    /// Extract a string from the value, returning empty string if not possible
    pub fn as_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            _ => String::new(),
        }
    }

    /// Extract a boolean from the value, returning false if not possible
    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::U32(v) => *v != 0,
            Value::U8(v) => *v != 0,
            _ => false,
        }
    }
}