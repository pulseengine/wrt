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
#![deny(unsafe_code)] // Changed from forbid to deny to allow specific unsafe blocks when justified
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(hidden_glob_reexports)]
// Allow clippy warnings that would require substantial refactoring
#![allow(clippy::needless_continue)]
#![allow(clippy::if_not_else)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::elidable_lifetime_names)]
#![allow(clippy::unused_self)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::similar_names)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::inline_always)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::comparison_chain)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::panic)]
#![allow(clippy::single_match_else)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::explicit_iter_loop)]
#![allow(clippy::bool_to_int_with_if)]
#![allow(clippy::match_same_arms)]
// Allow all pedantic clippy warnings for now to focus on core functionality
#![allow(clippy::pedantic)]
#![allow(clippy::identity_op)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::map_identity)]
#![allow(clippy::expect_used)]
#![allow(clippy::useless_conversion)]
#![allow(clippy::unnecessary_map_or)]
#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::manual_flatten)]
#![allow(clippy::float_arithmetic)]
#![allow(clippy::unimplemented)]
#![allow(clippy::useless_attribute)]
#![allow(clippy::manual_div_ceil)]
#![allow(clippy::never_loop)]
#![allow(clippy::while_immutable_condition)]
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::empty_line_after_doc_comments)]
#![allow(unused_imports)]
#![allow(clippy::duplicated_attributes)]
#![allow(clippy::multiple_bound_locations)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]

// Core library is always available
extern crate core;

#[cfg(feature = "std")]
extern crate std;

#[cfg(any(feature = "std", feature = "alloc"))]
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
#[allow(clippy::missing_errors_doc)]
#[allow(clippy::missing_panics_doc)]
#[allow(clippy::return_self_not_must_use)]
#[allow(clippy::doc_markdown)]
// #![deny(pointer_cast)] // Removed, as it's not a standard lint
// Binary std/no_std choice
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

/// `Result` type alias for WRT operations using `wrt_error::Error`
pub type WrtResult<T> = core::result::Result<T, Error>;

// Core modules - always available in all configurations
/// Atomic memory operations with integrated checksumming
pub mod atomic_memory;
/// Bounded collections for memory safety
pub mod bounded;
/// Binary std/no_std choice
pub mod bounded_collections;
/// Binary std/no_std choice
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
/// Shared memory support for multi-threading
pub mod shared_memory;
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
/// Formal verification using Kani
#[cfg(any(doc, kani))]
pub mod verify;

// New foundation modules for Agent A deliverables
/// Unified type system with platform-configurable bounded collections (simplified)
pub mod unified_types_simple;
/// Memory provider hierarchy for predictable allocation behavior
pub mod memory_system;
/// Global memory configuration and platform-aware allocation system
pub mod global_memory_config;
/// ASIL-aware safety primitives for safety-critical applications
pub mod safety_system;
/// ASIL-tagged testing framework for safety verification
pub mod asil_testing;

// Binary std/no_std choice
#[cfg(feature = "std")]
/// Builder patterns for Component Model types
pub mod component_builder;
#[cfg(feature = "std")]
/// Store for component model types
pub mod component_type_store;
#[cfg(feature = "std")]
/// WebAssembly Component Model value types
pub mod component_value;
#[cfg(feature = "std")]
pub mod component_value_store;
#[cfg(feature = "std")]
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

// Binary std/no_std choice
#[cfg(not(feature = "std"))]
/// No-std hash map implementation
pub mod no_std_hashmap;
// pub mod no_std_compat;

// Re-export the most important types - core types always available
pub use atomic_memory::{AtomicMemoryExt, AtomicMemoryOps};
pub use bounded::{BoundedStack, BoundedString, BoundedVec, CapacityError, WasmName};
// Alloc-dependent re-exports
#[cfg(feature = "std")]
pub use bounded_collections::BoundedBitSet;
pub use bounded_collections::{BoundedDeque, BoundedMap, BoundedQueue, BoundedSet};
pub use builder::{
    BoundedBuilder, MemoryBuilder, NoStdProviderBuilder, ResourceBuilder, ResourceItemBuilder,
    ResourceTypeBuilder, StringBuilder,
};
pub use builtin::BuiltinType;
pub use component::{ComponentType, ExternType, InstanceType, Namespace, ResourceType};
#[cfg(feature = "std")]
pub use component_builder::{ComponentTypeBuilder, ExportBuilder, ImportBuilder, NamespaceBuilder};
#[cfg(feature = "std")]
pub use component_type_store::{ComponentTypeStore, TypeRef};
#[cfg(feature = "std")]
pub use component_value::ComponentValue;
#[cfg(feature = "std")]
pub use component_value_store::{ComponentValueStore, ValueRef};
#[cfg(feature = "std")]
pub use component_value_store_builder::ComponentValueStoreBuilder;
#[cfg(feature = "std")]
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
#[cfg(feature = "std")]
pub use safe_memory::StdMemoryProvider;
pub use traits::{BoundedCapacity, Checksummed, FromFormat, ToFormat, Validatable};
pub use types::{
    BlockType, DataMode, ElementMode, MemArg,
    FuncType,
    GlobalType,
    Limits,
    MemoryType,
    RefType,
    TableType,
    ValueType,
};

// Data and element segment types are defined in the types module
// DataSegment and ElementSegment types are provided by wrt-format module when needed
// Temporarily disabled validation exports due to circular dependency
// pub use validation::{
//     BoundedCapacity, Checksummed, Validatable, ValidationError, ValidationResult,
// };
pub use values::Value;
pub use verification::{Checksum, VerificationLevel};

// Re-export unified types for backward compatibility and new functionality
pub use unified_types_simple::{
    DefaultTypes, EmbeddedTypes, DesktopTypes, SafetyCriticalTypes,
    PlatformCapacities, UnifiedTypes,
};

// Re-export memory system types
pub use memory_system::{
    UnifiedMemoryProvider, ConfigurableProvider, SmallProvider, MediumProvider, LargeProvider,
    NoStdProviderWrapper, MemoryProviderFactory,
};

#[cfg(feature = "std")]
pub use memory_system::UnifiedStdProvider;

// Re-export safety system types
pub use safety_system::{
    // Traditional ASIL types
    AsilLevel, SafetyContext, SafetyGuard, SafeMemoryAllocation,
    // Universal safety system types
    SafetyStandard, SafetyStandardType, SafetyStandardConversion,
    UniversalSafetyContext, SeverityScore, SafetyError,
    // Additional safety standard levels
    DalLevel, SilLevel, MedicalClass, RailwaySil, AgricultureLevel,
};

/// The WebAssembly binary format magic number: \0asm
pub const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];

/// The WebAssembly Core Specification version this runtime aims to support.
/// Version 2.0 includes features like multi-value returns, reference types,
/// etc.
pub const WASM_VERSION: u32 = 2;

// Component model feature re-exports
#[cfg(feature = "component-model-values")]
pub use component_value::ValType;

// Component Model async types (always available when component-model-async is enabled)
#[cfg(feature = "component-model-async")]
/// Component Model async types (future, stream, error-context)
pub mod async_types;

// Async support modules
#[cfg(feature = "async-api")]
/// Simple async executor support
pub mod async_executor_simple;
#[cfg(feature = "async-api")]
/// Bridge between Component Model async and Rust async
pub mod async_bridge;

// Component Model async re-exports
#[cfg(feature = "component-model-async")]
pub use async_types::{
    ComponentFuture, ComponentFutureStatus, ComponentStream, ErrorContext, FutureHandle,
    StreamHandle, StreamState,
};

// Async API re-exports
#[cfg(feature = "async-api")]
pub use async_executor_simple::{
    is_using_fallback, AsyncRuntime, ExecutorError, with_async,
};
#[cfg(feature = "async-api")]
pub use async_bridge::{with_async as with_async_bridge};
#[cfg(all(feature = "async-api", feature = "component-model-async"))]
pub use async_bridge::{ComponentAsyncExt, ComponentFutureBridge, ComponentStreamBridge};

// Panic handler disabled to avoid conflicts with other crates
// // Provide a panic handler only when wrt-foundation is being tested in isolation
// #[cfg(all(not(feature = "std"), not(test), not(feature = "disable-panic-handler")))]
// #[panic_handler]
// fn panic(_info: &core::panic::PanicInfo) -> ! {
//     loop {}
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bounded::BoundedVec;
    use crate::safe_memory::{SafeMemoryHandler, NoStdProvider};
    use crate::traits::BoundedCapacity;

    #[test]
    fn test_boundedvec_is_empty() {
        let provider = NoStdProvider::new();
        let mut vec = BoundedVec::<u32, 10, _>::new(provider).unwrap();
        
        // Test is_empty
        assert!(vec.is_empty());
        
        // Add an item
        vec.push(42).unwrap();
        
        // Test not empty
        assert!(!vec.is_empty());
        assert_eq!(vec.len(), 1);
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_boundedvec_to_vec_std() {
        let provider = NoStdProvider::new();
        let mut vec = BoundedVec::<u32, 10, _>::new(provider).unwrap();
        
        vec.push(1).unwrap();
        vec.push(2).unwrap();
        vec.push(3).unwrap();
        
        let std_vec = vec.to_vec().unwrap();
        assert_eq!(std_vec, vec![1, 2, 3]);
    }

    #[test]
    #[cfg(not(feature = "std"))]
    fn test_boundedvec_to_vec_no_std() {
        let provider = NoStdProvider::new();
        let mut vec = BoundedVec::<u32, 10, _>::new(provider).unwrap();
        
        vec.push(1).unwrap();
        vec.push(2).unwrap();
        vec.push(3).unwrap();
        
        let cloned_vec = vec.to_vec().unwrap();
        assert_eq!(cloned_vec.len(), 3);
        assert_eq!(cloned_vec.get(0).unwrap(), 1);
        assert_eq!(cloned_vec.get(1).unwrap(), 2);
        assert_eq!(cloned_vec.get(2).unwrap(), 3);
    }

    #[test]
    fn test_safe_memory_handler_to_vec() {
        let provider = NoStdProvider::new();
        let handler = SafeMemoryHandler::new(provider);
        
        // Test to_vec on empty handler
        let data = handler.to_vec().unwrap();
        
        #[cfg(feature = "std")]
        {
            assert!(data.is_empty());
        }
        
        #[cfg(not(feature = "std"))]
        {
            assert!(data.is_empty());
        }
    }

    // TODO: Add comprehensive tests for all public functionality in
    // wrt-foundation, ensuring coverage for different VerificationLevels,
    // std/no_std features, and edge cases for component model types, value
    // conversions, etc. Specific modules like bounded.rs, safe_memory.rs,
    // math_ops.rs, etc., should have their own detailed test suites as
    // well.
}
