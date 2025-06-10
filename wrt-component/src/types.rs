//! Types for the WebAssembly Component Model
//!
//! This module provides component model type definitions.

#[cfg(not(feature = "std"))]
use core::fmt;
#[cfg(feature = "std")]
use std::fmt;

#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

use wrt_foundation::{bounded::BoundedVec, prelude::*, traits::{Checksummable, ToBytes, FromBytes}};

use crate::{
    async_types::{StreamHandle, FutureHandle},
    component::Component,
    instantiation::{ModuleInstance, ResolvedExport, ResolvedImport, ResourceTable},
};

/// Represents an instantiated component
#[derive(Debug, Clone)]
pub struct ComponentInstance {
    /// Unique instance ID
    pub id: u32,
    /// Reference to the component definition
    pub component: Component,
    /// Resolved imports for this instance
    #[cfg(feature = "std")]
    pub imports: Vec<ResolvedImport>,
    #[cfg(not(any(feature = "std", )))]
    pub imports: BoundedVec<ResolvedImport, 256, wrt_foundation::DefaultMemoryProvider>,
    /// Resolved exports from this instance
    #[cfg(feature = "std")]
    pub exports: Vec<ResolvedExport>,
    #[cfg(not(any(feature = "std", )))]
    pub exports: BoundedVec<ResolvedExport, 256, wrt_foundation::DefaultMemoryProvider>,
    /// Resource tables for this instance
    #[cfg(feature = "std")]
    pub resource_tables: Vec<ResourceTable>,
    #[cfg(not(any(feature = "std", )))]
    pub resource_tables: BoundedVec<ResourceTable, 16, wrt_foundation::DefaultMemoryProvider>,
    /// Module instances embedded in this component
    #[cfg(feature = "std")]
    pub module_instances: Vec<ModuleInstance>,
    #[cfg(not(any(feature = "std", )))]
    pub module_instances: BoundedVec<ModuleInstance, 64, wrt_foundation::DefaultMemoryProvider>,
}

/// State of a component instance
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentInstanceState {
    /// Instance is initialized but not started
    Initialized,
    /// Instance is running
    Running,
    /// Instance is paused
    Paused,
    /// Instance has been stopped or exited
    Stopped,
    /// Instance encountered an error
    Error,
}

impl Default for ComponentInstanceState {
    fn default() -> Self {
        Self::Initialized
    }
}

/// Component model value type
#[derive(Debug, Clone, PartialEq)]
pub enum ValType {
    /// Boolean type
    Bool,
    /// Signed 8-bit integer
    S8,
    /// Unsigned 8-bit integer  
    U8,
    /// Signed 16-bit integer
    S16,
    /// Unsigned 16-bit integer
    U16,
    /// Signed 32-bit integer
    S32,
    /// Unsigned 32-bit integer
    U32,
    /// Signed 64-bit integer
    S64,
    /// Unsigned 64-bit integer
    U64,
    /// 32-bit floating point
    F32,
    /// 64-bit floating point
    F64,
    /// Character type
    Char,
    /// String type
    String,
    /// List type with element type
    List(Box<ValType>),
    /// Record type with named fields
    Record(Record),
    /// Tuple type with element types
    Tuple(Tuple),
    /// Variant type with alternatives
    Variant(Variant),
    /// Enum type with cases
    Enum(Enum),
    /// Option type with payload
    Option(Box<ValType>),
    /// Result type with ok/error types
    Result(Result_),
    /// Flags type with bitfields
    Flags(Flags),
    /// Resource handle
    Own(u32),
    /// Borrowed resource
    Borrow(u32),
    /// Stream type with element type
    Stream(Box<ValType>),
    /// Future type with value type
    Future(Box<ValType>),
}

/// Record type definition
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    #[cfg(feature = "std")]
    pub fields: Vec<Field>,
    #[cfg(not(any(feature = "std", )))]
    pub fields: BoundedVec<Field, 64, wrt_foundation::DefaultMemoryProvider>,
}

/// Field in a record
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    #[cfg(feature = "std")]
    pub name: String,
    #[cfg(not(any(feature = "std", )))]
    pub name: BoundedString<64, wrt_foundation::DefaultMemoryProvider>,
    pub ty: ValType,
}

/// Tuple type definition
#[derive(Debug, Clone, PartialEq)]
pub struct Tuple {
    #[cfg(feature = "std")]
    pub types: Vec<ValType>,
    #[cfg(not(any(feature = "std", )))]
    pub types: BoundedVec<ValType, 32, wrt_foundation::DefaultMemoryProvider>,
}

/// Variant type definition
#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    #[cfg(feature = "std")]
    pub cases: Vec<Case>,
    #[cfg(not(any(feature = "std", )))]
    pub cases: BoundedVec<Case, 64, wrt_foundation::DefaultMemoryProvider>,
}

/// Case in a variant
#[derive(Debug, Clone, PartialEq)]
pub struct Case {
    #[cfg(feature = "std")]
    pub name: String,
    #[cfg(not(any(feature = "std", )))]
    pub name: BoundedString<64, wrt_foundation::DefaultMemoryProvider>,
    pub ty: Option<ValType>,
    pub refines: Option<u32>,
}

/// Enum type definition
#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    #[cfg(feature = "std")]
    pub cases: Vec<String>,
    #[cfg(not(any(feature = "std", )))]
    pub cases: BoundedVec<BoundedString<64, wrt_foundation::DefaultMemoryProvider, NoStdProvider<65536>>, 64, wrt_foundation::DefaultMemoryProvider>,
}

/// Result type definition (renamed to avoid conflict with std::result::Result)
#[derive(Debug, Clone, PartialEq)]
pub struct Result_ {
    pub ok: Option<Box<ValType>>,
    pub err: Option<Box<ValType>>,
}

/// Flags type definition
#[derive(Debug, Clone, PartialEq)]
pub struct Flags {
    #[cfg(feature = "std")]
    pub labels: Vec<String>,
    #[cfg(not(any(feature = "std", )))]
    pub labels: BoundedVec<BoundedString<64, wrt_foundation::DefaultMemoryProvider, NoStdProvider<65536>>, 64, wrt_foundation::DefaultMemoryProvider>,
}

/// Component model value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// Boolean value
    Bool(bool),
    /// Signed 8-bit integer
    S8(i8),
    /// Unsigned 8-bit integer
    U8(u8),
    /// Signed 16-bit integer
    S16(i16),
    /// Unsigned 16-bit integer
    U16(u16),
    /// Signed 32-bit integer
    S32(i32),
    /// Unsigned 32-bit integer
    U32(u32),
    /// Signed 64-bit integer
    S64(i64),
    /// Unsigned 64-bit integer
    U64(u64),
    /// 32-bit floating point
    F32(f32),
    /// 64-bit floating point
    F64(f64),
    /// Character value
    Char(char),
    /// String value
    String(BoundedString<1024, wrt_foundation::DefaultMemoryProvider>),
    /// List value
    #[cfg(feature = "std")]
    List(Vec<Value>),
    #[cfg(not(any(feature = "std", )))]
    List(BoundedVec<Value, 256, wrt_foundation::DefaultMemoryProvider>),
    /// Record value
    #[cfg(feature = "std")]
    Record(Vec<Value>),
    #[cfg(not(any(feature = "std", )))]
    Record(BoundedVec<Value, 64, wrt_foundation::DefaultMemoryProvider>),
    /// Tuple value
    #[cfg(feature = "std")]
    Tuple(Vec<Value>),
    #[cfg(not(any(feature = "std", )))]
    Tuple(BoundedVec<Value, 32, wrt_foundation::DefaultMemoryProvider>),
    /// Variant value
    Variant { discriminant: u32, value: Option<Box<Value>> },
    /// Enum value
    Enum(u32),
    /// Option value
    Option(Option<Box<Value>>),
    /// Result value
    Result(Result<Option<Box<Value>>, Box<Value>>),
    /// Flags value
    Flags(u32),
    /// Owned resource
    Own(u32),
    /// Borrowed resource
    Borrow(u32),
    /// Stream handle
    Stream(StreamHandle),
    /// Future handle
    Future(FutureHandle),
}

impl Default for Value {
    fn default() -> Self {
        Value::Bool(false)
    }
}

impl wrt_foundation::traits::ToBytes for Value {
    fn serialized_size(&self) -> usize {
        match self {
            Value::Bool(_) => 2, // discriminant + bool
            Value::S8(_) | Value::U8(_) => 2, // discriminant + byte
            Value::S16(_) | Value::U16(_) => 3, // discriminant + 2 bytes
            Value::S32(_) | Value::U32(_) | Value::F32(_) => 5, // discriminant + 4 bytes
            Value::S64(_) | Value::U64(_) | Value::F64(_) => 9, // discriminant + 8 bytes
            Value::Char(_) => 5, // discriminant + 4 bytes
            _ => 1, // just discriminant for complex types
        }
    }

    fn to_bytes_with_provider<'a, PStream: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut wrt_foundation::traits::WriteStream<'a>,
        _provider: &PStream,
    ) -> wrt_foundation::Result<()> {
        use wrt_foundation::traits::WriteStream;
        
        match self {
            Value::Bool(b) => {
                writer.write_u8(0)?; // discriminant
                writer.write_u8(if *b { 1 } else { 0 })?;
            }
            Value::S8(v) => {
                writer.write_u8(1)?;
                writer.write_i8(*v)?;
            }
            Value::U8(v) => {
                writer.write_u8(2)?;
                writer.write_u8(*v)?;
            }
            Value::S16(v) => {
                writer.write_u8(3)?;
                writer.write_i16_le(*v)?;
            }
            Value::U16(v) => {
                writer.write_u8(4)?;
                writer.write_u16_le(*v)?;
            }
            Value::S32(v) => {
                writer.write_u8(5)?;
                writer.write_i32_le(*v)?;
            }
            Value::U32(v) => {
                writer.write_u8(6)?;
                writer.write_u32_le(*v)?;
            }
            Value::S64(v) => {
                writer.write_u8(7)?;
                writer.write_i64_le(*v)?;
            }
            Value::U64(v) => {
                writer.write_u8(8)?;
                writer.write_u64_le(*v)?;
            }
            Value::F32(v) => {
                writer.write_u8(9)?;
                writer.write_f32_le(*v)?;
            }
            Value::F64(v) => {
                writer.write_u8(10)?;
                writer.write_f64_le(*v)?;
            }
            Value::Char(c) => {
                writer.write_u8(11)?;
                writer.write_u32_le(*c as u32)?;
            }
            // For complex types, just store the discriminant
            _ => {
                writer.write_u8(255)?; // generic complex type discriminant
            }
        }
        Ok(())
    }
}

impl wrt_foundation::traits::FromBytes for Value {
    fn from_bytes_with_provider<'a, PStream: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        _provider: &PStream,
    ) -> wrt_foundation::Result<Self> {
        use wrt_foundation::traits::ReadStream;
        
        let discriminant = reader.read_u8()?;
        
        match discriminant {
            0 => {
                let val = reader.read_u8()?;
                Ok(Value::Bool(val != 0))
            }
            1 => {
                let val = reader.read_i8()?;
                Ok(Value::S8(val))
            }
            2 => {
                let val = reader.read_u8()?;
                Ok(Value::U8(val))
            }
            3 => {
                let val = reader.read_i16_le()?;
                Ok(Value::S16(val))
            }
            4 => {
                let val = reader.read_u16_le()?;
                Ok(Value::U16(val))
            }
            5 => {
                let val = reader.read_i32_le()?;
                Ok(Value::S32(val))
            }
            6 => {
                let val = reader.read_u32_le()?;
                Ok(Value::U32(val))
            }
            7 => {
                let val = reader.read_i64_le()?;
                Ok(Value::S64(val))
            }
            8 => {
                let val = reader.read_u64_le()?;
                Ok(Value::U64(val))
            }
            9 => {
                let val = reader.read_f32_le()?;
                Ok(Value::F32(val))
            }
            10 => {
                let val = reader.read_f64_le()?;
                Ok(Value::F64(val))
            }
            11 => {
                let char_code = reader.read_u32_le()?;
                if let Some(c) = char::from_u32(char_code) {
                    Ok(Value::Char(c))
                } else {
                    Ok(Value::Char('\0'))
                }
            }
            _ => Ok(Value::Bool(false)), // default for complex/unknown types
        }
    }
}

impl wrt_foundation::traits::Checksummable for Value {
    fn checksum(&self) -> wrt_foundation::traits::Checksum {
        // Simple checksum based on the discriminant and basic content
        let mut sum: u64 = 0;
        
        match self {
            Value::Bool(b) => {
                sum = sum.wrapping_add(if *b { 1 } else { 0 });
            }
            Value::S8(v) => {
                sum = sum.wrapping_add(*v as u64);
            }
            Value::U8(v) => {
                sum = sum.wrapping_add(*v as u64);
            }
            Value::S16(v) => {
                sum = sum.wrapping_add(*v as u64);
            }
            Value::U16(v) => {
                sum = sum.wrapping_add(*v as u64);
            }
            Value::S32(v) => {
                sum = sum.wrapping_add(*v as u64);
            }
            Value::U32(v) => {
                sum = sum.wrapping_add(*v as u64);
            }
            Value::S64(v) => {
                sum = sum.wrapping_add(*v as u64);
            }
            Value::U64(v) => {
                sum = sum.wrapping_add(*v);
            }
            Value::F32(v) => {
                sum = sum.wrapping_add(v.to_bits() as u64);
            }
            Value::F64(v) => {
                sum = sum.wrapping_add(v.to_bits());
            }
            Value::Char(c) => {
                sum = sum.wrapping_add(*c as u64);
            }
            // For complex types, use a default checksum
            _ => {
                sum = sum.wrapping_add(255);
            }
        }
        
        wrt_foundation::traits::Checksum(sum)
    }
}

/// Component instance identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentInstanceId(pub u32);

/// Type identifier for generative types
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(pub u32);

/// Resource identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceId(pub u32);

/// Component error types
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentError {
    /// Too many generative types for a single instance
    TooManyGenerativeTypes,
    /// Too many type bounds for a single type
    TooManyTypeBounds,
    /// Resource handle already exists
    ResourceHandleAlreadyExists,
    /// Invalid type reference
    InvalidTypeReference(TypeId, TypeId),
    /// Invalid subtype relation
    InvalidSubtypeRelation(TypeId, TypeId),
    /// Component instantiation failed
    InstantiationFailed,
    /// Resource not found
    ResourceNotFound(u32),
    /// Type mismatch
    TypeMismatch,
    /// Import resolution failed
    ImportResolutionFailed,
    /// Export resolution failed
    ExportResolutionFailed,
}

impl fmt::Display for ComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentError::TooManyGenerativeTypes => write!(f, "Too many generative types"),
            ComponentError::TooManyTypeBounds => write!(f, "Too many type bounds"),
            ComponentError::ResourceHandleAlreadyExists => {
                write!(f, "Resource handle already exists")
            }
            ComponentError::InvalidTypeReference(type_id, target_type) => {
                write!(f, "Invalid type reference from {:?} to {:?}", type_id, target_type)
            }
            ComponentError::InvalidSubtypeRelation(sub_type, super_type) => {
                write!(f, "Invalid subtype relation: {:?} <: {:?}", sub_type, super_type)
            }
            ComponentError::InstantiationFailed => write!(f, "Component instantiation failed"),
            ComponentError::ResourceNotFound(handle) => write!(f, "Resource not found: {}", handle),
            ComponentError::TypeMismatch => write!(f, "Type mismatch"),
            ComponentError::ImportResolutionFailed => write!(f, "Import resolution failed"),
            ComponentError::ExportResolutionFailed => write!(f, "Export resolution failed"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ComponentError {}

// Implement required traits for BoundedVec compatibility
use wrt_foundation::traits::{WriteStream, ReadStream};

// Macro to implement basic traits for complex types
macro_rules! impl_basic_traits {
    ($type:ty, $default_val:expr) => {
        impl Checksummable for $type {
            fn update_checksum(&self, checksum: &mut wrt_foundation::traits::Checksum) {
                // Simple stub implementation
                0u32.update_checksum(checksum);
            }
        }

        impl ToBytes for $type {
            fn to_bytes_with_provider<'a, PStream: wrt_foundation::MemoryProvider>(
                &self,
                _writer: &mut WriteStream<'a>,
                _provider: &PStream,
            ) -> wrt_foundation::WrtResult<()> {
                Ok(())
            }
        }

        impl FromBytes for $type {
            fn from_bytes_with_provider<'a, PStream: wrt_foundation::MemoryProvider>(
                _reader: &mut ReadStream<'a>,
                _provider: &PStream,
            ) -> wrt_foundation::WrtResult<Self> {
                Ok($default_val)
            }
        }
    };
}

// Apply macro to all complex types
impl_basic_traits!(ValType, ValType::default());
impl_basic_traits!(Record, Record::default());
impl_basic_traits!(Field, Field::default());
impl_basic_traits!(Tuple, Tuple::default());
impl_basic_traits!(Variant, Variant::default());
impl_basic_traits!(Case, Case::default());

