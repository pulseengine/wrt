//! Types for the WebAssembly Component Model
//!
//! This module provides component model type definitions.

#[cfg(not(feature = "std"))]
use core::fmt;
#[cfg(feature = "std")]
use std::fmt;

#[cfg(any(feature = "std", feature = "alloc"))]
use alloc::{string::String, vec::Vec};

use wrt_foundation::{bounded::BoundedVec, prelude::*};

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
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub imports: Vec<ResolvedImport>,
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub imports: BoundedVec<ResolvedImport, 256>,
    /// Resolved exports from this instance
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub exports: Vec<ResolvedExport>,
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub exports: BoundedVec<ResolvedExport, 256>,
    /// Resource tables for this instance
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub resource_tables: Vec<ResourceTable>,
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub resource_tables: BoundedVec<ResourceTable, 16>,
    /// Module instances embedded in this component
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub module_instances: Vec<ModuleInstance>,
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub module_instances: BoundedVec<ModuleInstance, 64>,
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
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fields: Vec<Field>,
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub fields: BoundedVec<Field, 64>,
}

/// Field in a record
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub name: String,
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub name: BoundedString<64>,
    pub ty: ValType,
}

/// Tuple type definition
#[derive(Debug, Clone, PartialEq)]
pub struct Tuple {
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub types: Vec<ValType>,
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub types: BoundedVec<ValType, 32>,
}

/// Variant type definition
#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub cases: Vec<Case>,
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub cases: BoundedVec<Case, 64>,
}

/// Case in a variant
#[derive(Debug, Clone, PartialEq)]
pub struct Case {
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub name: String,
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub name: BoundedString<64>,
    pub ty: Option<ValType>,
    pub refines: Option<u32>,
}

/// Enum type definition
#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub cases: Vec<String>,
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub cases: BoundedVec<BoundedString<64>, 64>,
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
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub labels: Vec<String>,
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub labels: BoundedVec<BoundedString<64>, 64>,
}

/// Component model value
#[derive(Debug, Clone, PartialEq)]
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
    String(BoundedString<1024>),
    /// List value
    #[cfg(any(feature = "std", feature = "alloc"))]
    List(Vec<Value>),
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    List(BoundedVec<Value, 256>),
    /// Record value
    #[cfg(any(feature = "std", feature = "alloc"))]
    Record(Vec<Value>),
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    Record(BoundedVec<Value, 64>),
    /// Tuple value
    #[cfg(any(feature = "std", feature = "alloc"))]
    Tuple(Vec<Value>),
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    Tuple(BoundedVec<Value, 32>),
    /// Variant value
    Variant { discriminant: u32, value: Option<Box<Value>> },
    /// Enum value
    Enum(u32),
    /// Option value
    Option(Option<Box<Value>>),
    /// Result value
    Result(WrtResult<Option<Box<Value>>>),
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
