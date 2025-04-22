//! Component Model type definitions
//!
//! This module defines the basic types used in WebAssembly Component Model.

#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{string::String, vec::Vec};

/// A Component Model value type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValType {
    /// Boolean value
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
    /// Unicode character
    Char,
    /// UTF-8 string
    String,
    /// Reference to another entity
    Ref(u32),
    /// Record with named fields
    Record(Vec<(String, ValType)>),
    /// Variant with cases
    Variant(Vec<(String, Option<ValType>)>),
    /// List of elements
    List(Box<ValType>),
    /// Tuple of elements
    Tuple(Vec<ValType>),
    /// Flags (set of named boolean flags)
    Flags(Vec<String>),
    /// Enumeration of variants
    Enum(Vec<String>),
    /// Option type
    Option(Box<ValType>),
    /// Result type
    Result(Box<ValType>),
    /// Resource handle (owned)
    Own(u32),
    /// Resource handle (borrowed)
    Borrow(u32),
}

impl ValType {
    /// Get the size in bytes of this type
    pub fn size_bytes(&self) -> usize {
        match self {
            Self::Bool => 1,
            Self::S8 | Self::U8 => 1,
            Self::S16 | Self::U16 => 2,
            Self::S32 | Self::U32 => 4,
            Self::S64 | Self::U64 => 8,
            Self::F32 => 4,
            Self::F64 => 8,
            Self::Char => 4,
            Self::String => 8, // Length prefix + pointer
            Self::Ref(_) => 4,
            Self::Record(_) => 8,  // Length prefix + pointer
            Self::Variant(_) => 8, // Tag + value
            Self::List(_) => 8,    // Length prefix + pointer
            Self::Tuple(_) => 8,   // Length prefix + pointer
            Self::Flags(_) => 4,   // Bitmap
            Self::Enum(_) => 4,    // Tag
            Self::Option(_) => 5,  // Tag + value
            Self::Result(_) => 5,  // Tag + value
            Self::Own(_) => 4,
            Self::Borrow(_) => 4,
        }
    }
}
