//! Shared WebAssembly type definitions for wrt with functional safety
//!
//! This crate provides type definitions and memory management utilities
//! that are shared between wrt-decoder and wrt, with a focus on
//! functional safety for ASIL-B compliance.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs)]
#![warn(clippy::missing_panics_doc)]

// Import std when available
#[cfg(feature = "std")]
extern crate std;

// Import alloc for no_std
#[cfg(not(feature = "std"))]
extern crate alloc;

// Core re-exports based on environment
#[cfg(feature = "std")]
pub use std::{collections::HashMap, fmt, string::String, vec::Vec};

#[cfg(not(feature = "std"))]
pub use alloc::{collections::BTreeMap as HashMap, string::String, vec::Vec};

// Core modules
/// Bounded collections for memory safety
pub mod bounded;
/// WebAssembly Component Model types
pub mod component;
/// WebAssembly Component Model value types
pub mod component_value;
/// Operation tracking and fuel metering
pub mod operations;
/// Safe memory access primitives
pub mod safe_memory;
/// WebAssembly section definitions
pub mod sections;
/// Core WebAssembly types
pub mod types;
/// Validation utilities
pub mod validation;
/// WebAssembly value representations
pub mod values;
/// Verification and integrity checking
pub mod verification;

// Re-export the most important types
pub use bounded::{BoundedHashMap, BoundedStack, BoundedVec, CapacityError};
pub use component::{
    ComponentType, ExternType, FuncType, GlobalType, InstanceType, Limits, MemoryType, Namespace,
    ResourceType, TableType,
};
pub use component_value::ComponentValue;
pub use operations::{
    global_fuel_consumed, global_operation_summary, record_global_operation,
    reset_global_operations, OperationSummary, OperationTracking, OperationType,
};
pub use safe_memory::{MemoryProvider, SafeSlice};
pub use types::ValueType;
pub use validation::{BoundedCapacity, Checksummed, Validatable};
pub use values::Value;
pub use verification::{Checksum, VerificationLevel};

/// The WebAssembly binary format magic number: \0asm
pub const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];

/// The WebAssembly binary format version
pub const WASM_VERSION: u32 = 1;

// Conversion traits for standardized conversions
/// Trait for converting from a format type to a runtime type
pub trait FromFormat<T> {
    /// Convert from format value to runtime type
    fn from_format(format_value: T) -> wrt_error::Result<Self>
    where
        Self: Sized;
}

/// Trait for converting from a runtime type to a format type
pub trait ToFormat<T> {
    /// Convert from runtime type to format value
    fn to_format(&self) -> wrt_error::Result<T>;
}

// Re-export Result for convenience
pub use wrt_error::Result;
