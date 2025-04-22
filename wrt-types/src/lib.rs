//! Shared WebAssembly type definitions for wrt with functional safety
//!
//! This crate provides type definitions and memory management utilities
//! that are shared between wrt-decoder and wrt, with a focus on
//! functional safety for ASIL-B compliance.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "kani", feature(allocator_api))]
#![deny(missing_docs)]

// Import std when available
#[cfg(feature = "std")]
extern crate std;

// Import alloc for no_std
#[cfg(not(feature = "std"))]
extern crate alloc;

// Core re-exports based on environment
#[cfg(feature = "std")]
pub use std::{collections::HashMap, string::String, vec::Vec};

#[cfg(not(feature = "std"))]
pub use alloc::{collections::BTreeMap as HashMap, string::String, vec::Vec};

// Core modules
pub mod bounded;
pub mod operations;
pub mod safe_memory;
pub mod sections;
pub mod types;
pub mod validation;
pub mod values;
pub mod verification;

// Re-export ComponentValue from wrt-common
pub use wrt_common::component_value::ComponentValue;

// Re-export the most important types
pub use bounded::{BoundedHashMap, BoundedStack, BoundedVec, CapacityError};
pub use operations::{
    global_fuel_consumed, global_operation_summary, record_global_operation,
    reset_global_operations, OperationSummary, OperationTracking, OperationType,
};
pub use safe_memory::{MemoryProvider, SafeSlice};
pub use types::{FuncType, ValueType};
pub use validation::{BoundedCapacity, Checksummed, Validatable};
pub use values::Value;
pub use verification::{Checksum, VerificationLevel};

/// Definitions for WebAssembly Component Model types
pub mod component;
/// Value types and operations for WebAssembly values
pub mod value;

// Re-export commonly used types
pub use component::{
    ComponentType, ExternType, GlobalType, InstanceType, Limits, MemoryType, Namespace,
    ResourceType, TableType,
};

/// The WebAssembly binary format magic number: \0asm
pub const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];

/// The WebAssembly binary format version
pub const WASM_VERSION: u32 = 1;

pub use operations::*;
pub use values::*;
pub use verification::*;

// Create a re-export module for component values
/// Module for Component Model value types
pub mod component_value {
    pub use wrt_common::component_value::*;
}
