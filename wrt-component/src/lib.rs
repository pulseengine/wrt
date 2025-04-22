//! Component Model implementation for the WebAssembly Runtime (WRT).
//!
//! This crate provides an implementation of the WebAssembly Component Model,
//! enabling composition and interoperability between WebAssembly modules
//! with shared-nothing linking.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "kani", feature(kani))]

// When no_std but alloc is available
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{boxed::Box, string::String, vec::Vec};

#[cfg(feature = "std")]
use std::{boxed::Box, string::String, vec::Vec};

// Reexports for convenience
pub use wrt_error::{Error, Result};
pub use wrt_host::CallbackRegistry;

// Export modules
pub mod canonical;
pub mod component;
pub mod execution;
pub mod export;
pub mod host;
pub mod import;
pub mod instance;
pub mod namespace;
pub mod resources;
pub mod runtime;
pub mod strategies;
pub mod type_compatibility;
pub mod values;

// Reexport types
pub use canonical::CanonicalABI;
pub use component::{Component, ComponentType};
pub use export::Export;
pub use host::Host;
pub use import::Import;
pub use instance::InstanceValue;
pub use namespace::Namespace;
pub use resources::{BufferPool, MemoryStrategy, Resource, ResourceTable, VerificationLevel};
pub use strategies::memory::{
    BoundedCopyStrategy, FullIsolationStrategy, MemoryOptimizationStrategy, ZeroCopyStrategy,
};
pub use type_compatibility::{
    component_value_to_value, deserialize_component_value, serialize_component_value,
    value_to_component_value, TypeCompatibility,
};
pub use values::ComponentValue;

// Include verification module when the kani feature is enabled
#[cfg(feature = "kani")]
pub mod verify;
