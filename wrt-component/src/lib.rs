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

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{boxed::Box, string::String, vec::Vec};

#[cfg(feature = "std")]
use std::{boxed::Box, string::String, vec::Vec};

#[cfg(feature = "std")]
use std::sync::Arc;

#[cfg(not(feature = "std"))]
use alloc::sync::Arc;

// Reexports for convenience
pub use wrt_error::{Error, Result};
pub use wrt_host::CallbackRegistry;

// Export modules
pub mod builtins;
pub mod canonical;
pub mod component;
pub mod execution;
pub mod export;
pub mod host;
pub mod import;
pub mod instance;
pub mod namespace;
pub mod parser;
pub mod resources;
pub mod runtime;
pub mod strategies;
pub mod type_conversion;
pub mod values;

// Reexport types
pub use builtins::{BuiltinHandler, BuiltinRegistry};
pub use canonical::CanonicalABI;
pub use export::Export;
pub use host::Host;
pub use import::Import;
pub use instance::InstanceValue;
pub use namespace::Namespace;
pub use parser::get_required_builtins;
pub use resources::{BufferPool, MemoryStrategy, Resource, ResourceTable, VerificationLevel};
pub use strategies::memory::{
    BoundedCopyStrategy, FullIsolationStrategy, MemoryOptimizationStrategy, ZeroCopyStrategy,
};
pub use type_conversion::{
    format_to_types_component_type, format_to_types_extern_type, format_val_type_to_value_type,
    runtime_to_format_extern_type as types_to_format_extern_type, types_to_format_component_type,
    types_valtype_to_format_valtype, value_type_to_format_val_type, IntoFormatType,
    IntoRuntimeType,
};
pub use values::{
    component_to_core_value, component_value_to_value, core_to_component_value,
    deserialize_component_value, encode_component_value as serialize_component_value,
    format_valtype_to_common_valtype, value_to_component_value,
};
pub use wrt_types::builtin::BuiltinType;
pub use wrt_types::component::ComponentType;
pub use wrt_types::types::ValueType;
pub use wrt_types::values::Value;

// Include verification module when the kani feature is enabled
#[cfg(feature = "kani")]
pub mod verify;

pub use component::Component;
pub use component::ExternValue;
pub use component::FunctionValue;
pub use component::GlobalValue;
pub use component::MemoryValue;
pub use component::TableValue;

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
