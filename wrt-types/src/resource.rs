//! Resource type definitions for the Component Model
//!
//! This module provides types for working with WebAssembly Component Model resources.
//! Resources in the Component Model are handles to external entities or capabilities.

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::format;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::string::String;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;
use core::fmt;
#[cfg(feature = "std")]
#[cfg(feature = "std")]
#[cfg(not(feature = "std"))]
#[allow(unused_imports)]
use core::fmt::{Debug, Write};
#[cfg(not(feature = "std"))]
use core::str::FromStr;
#[cfg(feature = "std")]
#[allow(unused_imports)]
use std::fmt::{Debug, Write};
#[cfg(feature = "std")]
use std::string::String;

// optional imports
#[cfg(feature = "component-model-resources")]
use crate::bounded::BoundedVec;

use wrt_error::Result;

/// Resource identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(pub u64);

/// Resource New operation data
#[derive(Debug, Clone)]
pub struct ResourceNew {
    /// Type index for resource type
    pub type_idx: u32,
}

/// Resource Drop operation data
#[derive(Debug, Clone)]
pub struct ResourceDrop {
    /// Type index for resource type
    pub type_idx: u32,
}

/// Resource Rep operation data
#[derive(Debug, Clone)]
pub struct ResourceRep {
    /// Type index for resource type
    pub type_idx: u32,
}

/// Operations that can be performed on resources
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourceOperation {
    /// Read access to a resource
    Read,
    /// Write access to a resource
    Write,
    /// Execute a resource as code
    Execute,
    /// Create a new resource
    Create,
    /// Delete an existing resource
    Delete,
    /// Reference a resource (borrow it)
    Reference,
    /// Dereference a resource (access it through a reference)
    Dereference,
}

/// Resource operation in a canonical function
#[derive(Debug, Clone)]
pub enum ResourceCanonicalOperation {
    /// New resource operation
    New(ResourceNew),
    /// Drop a resource
    Drop(ResourceDrop),
    /// Resource representation operation
    Rep(ResourceRep),
}

impl ResourceOperation {
    /// Check if the operation requires read access
    pub fn requires_read(&self) -> bool {
        matches!(
            self,
            ResourceOperation::Read | ResourceOperation::Execute | ResourceOperation::Dereference
        )
    }

    /// Check if the operation requires write access
    pub fn requires_write(&self) -> bool {
        matches!(
            self,
            ResourceOperation::Write
                | ResourceOperation::Create
                | ResourceOperation::Delete
                | ResourceOperation::Reference
        )
    }

    /// Get the string representation of the operation
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceOperation::Read => "read",
            ResourceOperation::Write => "write",
            ResourceOperation::Execute => "execute",
            ResourceOperation::Create => "create",
            ResourceOperation::Delete => "delete",
            ResourceOperation::Reference => "reference",
            ResourceOperation::Dereference => "dereference",
        }
    }
}

impl fmt::Display for ResourceOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl core::str::FromStr for ResourceOperation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "read" => Ok(ResourceOperation::Read),
            "write" => Ok(ResourceOperation::Write),
            "execute" => Ok(ResourceOperation::Execute),
            "create" => Ok(ResourceOperation::Create),
            "delete" => Ok(ResourceOperation::Delete),
            "reference" => Ok(ResourceOperation::Reference),
            "dereference" => Ok(ResourceOperation::Dereference),
            #[cfg(feature = "alloc")]
            _ => Err(format!("Unknown resource operation: {}", s)),
            #[cfg(not(feature = "alloc"))]
            _ => {
                let mut message = String::new();
                let _ = write!(message, "Unknown resource operation: {}", s);
                Err(message)
            }
        }
    }
}

/// Resource representation type
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResourceRepresentation {
    /// 32-bit handle representation
    Handle32,
    /// 64-bit handle representation
    Handle64,
    /// Record representation with field names
    #[cfg(feature = "alloc")]
    Record(Vec<String>),
    /// Aggregate representation with type indices
    #[cfg(feature = "alloc")]
    Aggregate(Vec<u32>),
    /// Record representation (no_alloc version)
    #[cfg(not(feature = "alloc"))]
    Record,
}

impl ResourceRepresentation {
    /// Get the string representation of the representation type
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceRepresentation::Handle32 => "handle32",
            ResourceRepresentation::Handle64 => "handle64",
            #[cfg(feature = "alloc")]
            ResourceRepresentation::Record(_) => "record",
            #[cfg(feature = "alloc")]
            ResourceRepresentation::Aggregate(_) => "aggregate",
            #[cfg(not(feature = "alloc"))]
            ResourceRepresentation::Record => "record",
        }
    }
}

impl fmt::Display for ResourceRepresentation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl core::str::FromStr for ResourceRepresentation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "handle32" => Ok(ResourceRepresentation::Handle32),
            "handle64" => Ok(ResourceRepresentation::Handle64),
            #[cfg(feature = "alloc")]
            "record" => Ok(ResourceRepresentation::Record(Vec::new())),
            #[cfg(feature = "alloc")]
            "aggregate" => Ok(ResourceRepresentation::Aggregate(Vec::new())),
            #[cfg(not(feature = "alloc"))]
            "record" => Ok(ResourceRepresentation::Record),
            #[cfg(feature = "alloc")]
            _ => Err(format!("Unknown resource representation: {}", s)),
            #[cfg(not(feature = "alloc"))]
            _ => Ok(ResourceRepresentation::Record),
        }
    }
}
