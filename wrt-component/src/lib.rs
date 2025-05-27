// WRT - wrt-component
// Module: WebAssembly Component Model Implementation
// SW-REQ-ID: REQ_019
// SW-REQ-ID: REQ_002
//
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

#![forbid(unsafe_code)] // Rule 2

//! Component Model implementation for the WebAssembly Runtime (WRT).
//!
//! This crate provides an implementation of the WebAssembly Component Model,
//! enabling composition and interoperability between WebAssembly modules
//! with shared-nothing linking.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "kani", feature(kani))]
#![warn(clippy::missing_panics_doc)]

// When no_std but alloc is available
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Note about functionality with different features
// - std: Full functionality
// - no_std + alloc: Full no_std functionality
// - no_std without alloc: Limited to validation and introspection

// Export our prelude module for consistent imports
pub mod prelude;

// Export modules - some are conditionally compiled
pub mod builtins;
pub mod canonical;
#[cfg(feature = "std")]
pub mod component;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub mod component_no_std;
#[cfg(feature = "std")]
pub mod component_registry;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub mod component_registry_no_std;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub mod component_value_no_std;
pub mod error_format;
pub mod execution_engine;
pub mod memory_layout;
pub mod resource_lifecycle;
pub mod string_encoding;
// No-alloc module for pure no_std environments
pub mod execution;
pub mod export;
pub mod export_map;
pub mod factory;
pub mod host;
pub mod import;
pub mod import_map;
#[cfg(feature = "std")]
pub mod instance;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub mod instance_no_std;
pub mod modules;
pub mod namespace;
pub mod no_alloc;
pub mod parser;
pub mod resources;
pub mod runtime;
pub mod strategies;
pub mod type_conversion;
pub mod types;
pub mod values;

// Include verification module when the kani feature is enabled
#[cfg(feature = "kani")]
pub mod verify;

// Re-export core types and functionality for convenience
pub use builtins::{BuiltinHandler, BuiltinRegistry};
pub use canonical::CanonicalABI;
// Re-export component types based on feature flags
#[cfg(feature = "std")]
pub use component::{Component, ExternValue, FunctionValue, GlobalValue, MemoryValue, TableValue};
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use component_no_std::{
    BuiltinRequirements, Component, ComponentBuilder, ExternValue, FunctionValue, GlobalValue,
    MemoryValue, RuntimeInstance, TableValue, WrtComponentType, WrtComponentTypeBuilder,
    MAX_COMPONENT_EXPORTS, MAX_COMPONENT_IMPORTS, MAX_COMPONENT_INSTANCES,
};
// Re-export common constants
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use component_no_std::{
    MAX_BINARY_SIZE, MAX_COMPONENT_EXPORTS, MAX_COMPONENT_IMPORTS, MAX_COMPONENT_INSTANCES,
    MAX_FUNCTION_REF_SIZE, MAX_LINKED_COMPONENTS, MAX_MEMORY_SIZE, MAX_TABLE_SIZE,
};
// Re-export component registry based on feature flags
#[cfg(feature = "std")]
pub use component_registry::ComponentRegistry;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use component_registry_no_std::ComponentRegistry;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use component_value_no_std::deserialize_component_value_no_std as deserialize_component_value;
// Re-export component value utilities for no_std
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use component_value_no_std::{
    convert_format_to_valtype, convert_valtype_to_format, serialize_component_value_no_std,
};
pub use execution_engine::{ComponentExecutionEngine, ExecutionContext, ExecutionState};
pub use export::Export;
pub use factory::ComponentFactory;
pub use host::Host;
pub use import::Import;
#[cfg(feature = "std")]
pub use instance::InstanceValue;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use instance_no_std::{InstanceCollection, InstanceValue, InstanceValueBuilder};
pub use namespace::Namespace;
pub use parser::get_required_builtins;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use resources::{
    BoundedBufferPool, MemoryStrategy, Resource, ResourceArena, ResourceManager,
    ResourceOperationNoStd, ResourceStrategyNoStd, ResourceTable, VerificationLevel,
};
// Re-export resource types based on feature flags
#[cfg(feature = "std")]
pub use resources::{
    BufferPool, MemoryStrategy, Resource, ResourceArena, ResourceManager, ResourceTable,
    VerificationLevel,
};
pub use strategies::memory::{
    BoundedCopyStrategy, FullIsolationStrategy, MemoryOptimizationStrategy, ZeroCopyStrategy,
};
pub use type_conversion::{
    common_to_format_val_type, core_value_to_types_componentvalue, extern_type_to_func_type,
    format_constvalue_to_types_componentvalue, format_to_common_val_type,
    format_to_types_extern_type, format_val_type_to_value_type, format_valtype_to_types_valtype,
    types_componentvalue_to_core_value, types_componentvalue_to_format_constvalue,
    types_to_format_extern_type, types_valtype_to_format_valtype, value_type_to_format_val_type,
    value_type_to_types_valtype, FormatComponentType, FormatInstanceType, IntoFormatComponentType,
    IntoFormatInstanceType, IntoFormatType, IntoRuntimeComponentType, IntoRuntimeInstanceType,
    IntoRuntimeType, RuntimeComponentType, RuntimeInstanceType,
};
pub use types::ComponentInstance;
// Re-export value functions conditionally
#[cfg(feature = "std")]
pub use values::{component_to_core_value, core_to_component_value, deserialize_component_value};
pub use wrt_error::{codes, Error, ErrorCategory, Result};
pub use wrt_foundation::{
    builtin::BuiltinType, component::ComponentType, types::ValueType, values::Value,
};
pub use wrt_host::CallbackRegistry;

/// Debug logging macro - conditionally compiled
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        #[cfg(all(feature = "debug-log", feature = "std"))]
        {
            println!($($arg)*);
        }
        #[cfg(not(all(feature = "debug-log", feature = "std")))]
        {
            // Do nothing when debug logging is disabled or in no_std
        }
    };
}
