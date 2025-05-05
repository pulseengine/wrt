//! WebAssembly type definitions for the WRT runtime.
//!
//! This module provides type definitions and utilities used by the WebAssembly runtime.
//! It builds on the types provided by wrt-types, wrt-runtime, and other crates.

// Import from our prelude
use crate::prelude::*;

// Local imports
use crate::module::Module;
use crate::resource::ResourceType;

// We'll use these types from our prelude with specific names
use crate::prelude::{
    RuntimeGlobalType as GlobalType, RuntimeTableType as TableType, TypesBlockType as BlockType,
    TypesFuncType as FuncType, TypesMemoryType as MemoryType, TypesValueType as ValueType,
    WrtError as Error, WrtResult as Result,
};

/// Represents a WebAssembly external type
#[derive(Debug, Clone, PartialEq)]
pub enum ExternType {
    /// A WebAssembly function type
    Func(u32),
    /// A WebAssembly table type
    Table(TableType),
    /// A WebAssembly memory type
    Memory(MemoryType),
    /// A WebAssembly global type
    Global(GlobalType),
}

/// Represents the mutability of a global variable
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mutability {
    /// The global is not mutable
    Const,
    /// The global is mutable
    Var,
}

/// Represents a reference type for components
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RefType {
    /// A reference to a WebAssembly function
    FuncRef,
    /// A reference to an external object
    ExternRef,
}

/// Represents a WebAssembly value
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A 32-bit integer
    I32(i32),
    /// A 64-bit integer
    I64(i64),
    /// A 32-bit floating-point number
    F32(f32),
    /// A 64-bit floating-point number
    F64(f64),
    /// A 128-bit vector
    V128([u8; 16]),
    /// A function reference
    FuncRef(Option<u32>),
    /// An external reference
    ExternRef(Option<u32>),
}

/// Convert a wrt-types Value to our Value
pub fn from_types_value(value: &TypesValue) -> Value {
    match value {
        TypesValue::I32(v) => Value::I32(*v),
        TypesValue::I64(v) => Value::I64(*v),
        TypesValue::F32(v) => Value::F32(*v),
        TypesValue::F64(v) => Value::F64(*v),
        TypesValue::V128(v) => Value::V128(*v),
        TypesValue::FuncRef(v) => Value::FuncRef(*v),
        TypesValue::ExternRef(v) => Value::ExternRef(*v),
    }
}

/// Convert our Value to a wrt-types Value
pub fn to_types_value(value: &Value) -> TypesValue {
    match value {
        Value::I32(v) => TypesValue::I32(*v),
        Value::I64(v) => TypesValue::I64(*v),
        Value::F32(v) => TypesValue::F32(*v),
        Value::F64(v) => TypesValue::F64(*v),
        Value::V128(v) => TypesValue::V128(*v),
        // Convert between the different FuncRef/ExternRef types
        // For now, create dummy values - this needs to be properly implemented
        Value::FuncRef(_) => TypesValue::I32(0),  // Temporary placeholder 
        Value::ExternRef(_) => TypesValue::I32(0),  // Temporary placeholder
    }
}

impl Value {
    /// Get the value type of a value
    pub fn value_type(&self) -> ValueType {
        match self {
            Value::I32(_) => ValueType::I32,
            Value::I64(_) => ValueType::I64,
            Value::F32(_) => ValueType::F32,
            Value::F64(_) => ValueType::F64,
            // V128 is not in ValueType enum, map to something reasonable
            Value::V128(_) => ValueType::I64, // Best approximation for SIMD value
            Value::FuncRef(_) => ValueType::FuncRef,
            Value::ExternRef(_) => ValueType::ExternRef,
        }
    }

    /// Type name
    pub fn type_(&self) -> &'static str {
        match self {
            Value::I32(_) => "i32",
            Value::I64(_) => "i64",
            Value::F32(_) => "f32",
            Value::F64(_) => "f64",
            Value::V128(_) => "v128",
            Value::FuncRef(_) => "funcref",
            Value::ExternRef(_) => "externref",
        }
    }

    /// Get the i32 value if this is an i32
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Value::I32(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the i64 value if this is an i64
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::I64(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the f32 value if this is an f32
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            Value::F32(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the f64 value if this is an f64
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::F64(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the v128 value if this is a v128
    pub fn as_v128(&self) -> Result<[u8; 16]> {
        match self {
            Value::V128(v) => Ok(*v),
            _ => Err(Error::new(
                wrt_error::ErrorCategory::Validation,
                wrt_error::codes::VALIDATION_ERROR,
                format!("Expected v128, found {}", self.type_())
            )),
        }
    }

    /// Get the funcref value if this is a funcref
    pub fn as_funcref(&self) -> Option<Option<u32>> {
        match self {
            Value::FuncRef(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the externref value if this is an externref
    pub fn as_externref(&self) -> Option<Option<u32>> {
        match self {
            Value::ExternRef(v) => Some(*v),
            _ => None,
        }
    }
}

/// Create a Value from scratch based on a ValueType
pub fn create_value(ty: ValueType) -> Value {
    match ty {
        ValueType::I32 => Value::I32(0),
        ValueType::I64 => Value::I64(0),
        ValueType::F32 => Value::F32(0.0),
        ValueType::F64 => Value::F64(0.0),
        ValueType::FuncRef => Value::FuncRef(None),
        ValueType::ExternRef => Value::ExternRef(None),
    }
}

/// Create a resource type from a wrt-type ExternType
pub fn create_resource_type(ty: &wrt_types::ExternType) -> Option<ResourceType> {
    match ty {
        wrt_types::ExternType::Resource(_) => Some(ResourceType { 
            name: "resource".to_string(),
            representation: wrt_types::resource::ResourceRepresentation::Handle32,
            borrowable: false,
            nullable: true,
        }),
        _ => None,
    }
}
