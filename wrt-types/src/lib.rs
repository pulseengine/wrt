//! Shared WebAssembly type definitions for wrt with functional safety
//!
//! This crate provides type definitions and memory management utilities
//! that are shared between wrt-decoder and wrt, with a focus on
//! functional safety for ASIL-B compliance.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(unsafe_code)]
#![warn(missing_docs)]

// Import std when available
#[cfg(feature = "std")]
extern crate std;

// Import alloc for no_std with allocation
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Prelude module for consistent imports across std and no_std environments
pub mod prelude;

// Re-export common types from prelude
pub use prelude::*;

// Re-export error related types for convenience
pub use wrt_error::{kinds, Error};

/// Result type alias for WRT operations using wrt_error::Error
pub type WrtResult<T> = core::result::Result<T, Error>;

// Core modules
/// Bounded collections for memory safety
pub mod bounded;
/// WebAssembly Component Model built-in types
pub mod builtin;
/// WebAssembly Component Model types
pub mod component;
/// WebAssembly Component Model value types
pub mod component_value;
/// Type conversion utilities
pub mod conversion;
/// Conversions between wrt_error and wrt_types
pub mod error_convert;
/// Operation tracking and fuel metering
pub mod operations;
/// Resource management
pub mod resource;
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
    ComponentType, ExternType, GlobalType, InstanceType, Limits, MemoryType, Namespace,
    ResourceType, TableType,
};
pub use component_value::ComponentValue;
// Re-export conversion utilities
pub use conversion::{
    binary_to_val_type, ref_type_to_val_type, val_type_to_binary, val_type_to_ref_type,
};
pub use error_convert::ErrorCategory;
pub use operations::{
    global_fuel_consumed, global_operation_summary, record_global_operation,
    reset_global_operations, OperationSummary, OperationTracking, OperationType,
};
#[cfg(not(feature = "std"))]
pub use safe_memory::NoStdMemoryProvider;
#[cfg(feature = "std")]
pub use safe_memory::StdMemoryProvider;
pub use safe_memory::{MemoryProvider, SafeMemoryHandler, SafeSlice, SafeStack};
pub use traits::{FromFormat, ToFormat};
pub use types::{BlockType, FuncType, RefType, ValueType};
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
