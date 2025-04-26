//! Shared WebAssembly type definitions for wrt with functional safety
//!
//! This crate provides type definitions and memory management utilities
//! that are shared between wrt-decoder and wrt, with a focus on
//! functional safety for ASIL-B compliance.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "kani", feature(kani))]
#![warn(unsafe_code)]
#![warn(missing_docs)]

// Import std when available
#[cfg(feature = "std")]
extern crate std;

// Import alloc for no_std
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Core re-exports based on environment
#[cfg(feature = "std")]
pub use std::{
    boxed::Box, collections::HashMap, fmt, format, string::String, string::ToString, vec, vec::Vec,
};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use alloc::{
    boxed::Box, collections::BTreeMap as HashMap, format, string::String, string::ToString, vec,
    vec::Vec,
};

// Make sure the necessary types are available for no_std builds
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use alloc::{borrow, rc, sync};

// Re-export error related types for convenience
pub use wrt_error::{kinds, Error, Result};

// Core modules
/// Bounded collections for memory safety
pub mod bounded;
/// WebAssembly Component Model built-in types
pub mod builtin;
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
/// Common traits for type conversions
pub mod traits;
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
pub use builtin::BuiltinType;
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
pub use traits::{FromFormat, ToFormat};
pub use types::ValueType;
pub use validation::{BoundedCapacity, Checksummed, Validatable};
pub use values::Value;
pub use verification::{Checksum, VerificationLevel};

/// The WebAssembly binary format magic number: \0asm
pub const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];

/// The WebAssembly binary format version
pub const WASM_VERSION: u32 = 1;

// Core feature re-exports
// Note: These are feature-gated re-exports that shouldn't conflict with the main ones
// #[cfg(feature = "component-model-core")]
// pub use component::ComponentType;

#[cfg(feature = "component-model-values")]
pub use component_value::ValType;
