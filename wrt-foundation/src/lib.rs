//! Core type definitions, traits, and utilities for the WebAssembly Runtime
//! (WRT).
//!
//! This crate provides foundational data structures and functionalities used
//! across the WRT ecosystem, ensuring type safety, memory safety, and
//! consistent error handling. It supports three configurations:
//! - `std`: Full standard library support
//! - `no_std` + `alloc`: No standard library but with allocation
//! - `no_std` + `no_alloc`: Pure no_std without any allocation
//!
//! # Feature Flags
//!
//! - `std`: Enables standard library support (implies `alloc`)
//! - `alloc`: Enables allocation support for `no_std` environments
//! - Default: Pure `no_std` without allocation

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

// Core library is always available
extern crate core;

#[cfg(feature = "std")]
extern crate std;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// WRT - wrt-foundation
// SW-REQ-ID: REQ_MEM_SAFETY_001
//
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// SW-REQ-ID: REQ_LANG_RUST_PROJECT_SETUP_001
// SW-REQ-ID: REQ_LANG_RUST_EDITION_001

// #![deny(
//     warnings,
//     missing_docs,
//     missing_debug_implementations,
//     missing_copy_implementations,
//     trivial_casts,
//     trivial_numeric_casts,
//     unstable_features,
//     unused_import_braces,
//     unused_qualifications
// )]
#[forbid(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#[warn(clippy::pedantic, clippy::nursery)]
// #![deny(pointer_cast)] // Removed, as it's not a standard lint
// #![deny(alloc_instead_of_core)] // TODO: Verify this lint or implement if
// custom

// Conditionally import log if std feature is enabled
// #[cfg(feature = "std")] // Removed
// extern crate log; // Removed

// Prelude module for consistent imports across std and no_std environments
pub mod prelude;

// Re-export common types from prelude
pub use prelude::*;
// Re-export error related types for convenience
pub use wrt_error::{codes, kinds, Error, ErrorCategory};

/// Result type alias for WRT operations using `wrt_error::Error`
pub type WrtResult<T> = core::result::Result<T, Error>;

// Core modules - always available in all configurations
/// Atomic memory operations with integrated checksumming
pub mod atomic_memory;
/// Bounded collections for memory safety
pub mod bounded;
/// Additional bounded collections for no_std/no_alloc environments
pub mod bounded_collections;
/// Builder patterns for no_std/no_alloc types
pub mod builder;
/// WebAssembly Component Model built-in types
pub mod builtin;
/// WebAssembly Component Model types
pub mod component;
/// Type conversion utilities
pub mod conversion;
/// Float representation utilities
pub mod float_repr;
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

// Modules that require allocation
#[cfg(feature = "alloc")]
/// Builder patterns for Component Model types
pub mod component_builder;
#[cfg(feature = "alloc")]
/// Store for component model types
pub mod component_type_store;
#[cfg(feature = "alloc")]
/// WebAssembly Component Model value types
pub mod component_value;
#[cfg(feature = "alloc")]
pub mod component_value_store;
#[cfg(feature = "alloc")]
/// Builder pattern for component value store
pub mod component_value_store_builder;

// Platform-specific modules
#[cfg(feature = "platform-memory")]
/// Linear memory implementation using PageAllocator.
pub mod linear_memory;
#[cfg(feature = "platform-memory")]
/// Memory builder patterns for platform-backed memory types
pub mod memory_builder;
#[cfg(feature = "platform-memory")]
/// Runtime memory module
pub mod runtime_memory;

// Custom HashMap for pure no_std/no_alloc
#[cfg(not(any(feature = "std", feature = "alloc")))]
/// Custom HashMap implementation for no_std/no_alloc environments
pub mod no_std_hashmap;
// pub mod no_std_compat;

// Re-export the most important types - core types always available
pub use atomic_memory::{AtomicMemoryExt, AtomicMemoryOps};
pub use bounded::{BoundedStack, BoundedString, BoundedVec, CapacityError, WasmName};
// Alloc-dependent re-exports
#[cfg(feature = "alloc")]
pub use bounded_collections::BoundedBitSet;
pub use bounded_collections::{BoundedDeque, BoundedMap, BoundedQueue, BoundedSet};
pub use builder::{
    BoundedBuilder, MemoryBuilder, NoStdProviderBuilder, ResourceBuilder, ResourceItemBuilder,
    ResourceTypeBuilder, StringBuilder,
};
pub use builtin::BuiltinType;
pub use component::{ComponentType, ExternType, InstanceType, Namespace, ResourceType};
#[cfg(feature = "alloc")]
pub use component_builder::{ComponentTypeBuilder, ExportBuilder, ImportBuilder, NamespaceBuilder};
#[cfg(feature = "alloc")]
pub use component_type_store::{ComponentTypeStore, TypeRef};
#[cfg(feature = "alloc")]
pub use component_value::ComponentValue;
#[cfg(feature = "alloc")]
pub use component_value_store::{ComponentValueStore, ValueRef};
#[cfg(feature = "alloc")]
pub use component_value_store_builder::ComponentValueStoreBuilder;
pub use conversion::{ref_type_to_val_type, val_type_to_ref_type};
pub use float_repr::{FloatBits32, FloatBits64};
pub use operations::{
    global_fuel_consumed, global_operation_summary, record_global_operation,
    reset_global_operations, Summary as OperationSummary, Tracking as OperationTracking,
    Type as OperationType,
};
// Platform-specific re-exports
#[cfg(feature = "platform-memory")]
pub use runtime_memory::LinearMemory;
pub use safe_memory::{
    NoStdProvider, Provider as MemoryProvider, SafeMemoryHandler, Slice as SafeSlice,
    SliceMut as SafeSliceMut, Stats as MemoryStats,
};
pub use traits::{BoundedCapacity, Checksummed, FromFormat, ToFormat, Validatable};
pub use types::{
    BlockType, // DataSegment, ElementSegment // Uncommented BlockType
    FuncType,
    GlobalType,
    Limits,
    MemoryType,
    RefType,
    TableType,
    ValueType,
};
// Temporarily disabled validation exports due to circular dependency
// pub use validation::{
//     BoundedCapacity, Checksummed, Validatable, ValidationError, ValidationResult,
// };
pub use values::Value;
pub use verification::{Checksum, VerificationLevel};

/// The WebAssembly binary format magic number: \0asm
pub const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];

/// The WebAssembly Core Specification version this runtime aims to support.
/// Version 2.0 includes features like multi-value returns, reference types,
/// etc.
pub const WASM_VERSION: u32 = 2;

// Component model feature re-exports
#[cfg(feature = "component-model-values")]
pub use component_value::ValType;

#[cfg(test)]
mod tests {
    // TODO: Add comprehensive tests for all public functionality in
    // wrt-foundation, ensuring coverage for different VerificationLevels,
    // std/no_std features, and edge cases for component model types, value
    // conversions, etc. Specific modules like bounded.rs, safe_memory.rs,
    // math_ops.rs, etc., should have their own detailed test suites as
    // well.
}
