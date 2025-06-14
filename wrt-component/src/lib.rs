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

// Binary std/no_std choice
#[cfg(any(feature = "std", feature = "alloc"))]
extern crate alloc;

// Debug macro for both std and no_std environments
#[cfg(feature = "std")]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        eprintln!($($arg)*);
    };
}

#[cfg(not(feature = "std"))]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        // No-op in no_std environments
    };
}

// Export our prelude module for consistent imports
pub mod prelude;

// Core component modules
pub mod adapter;
pub mod agent_registry;
pub mod builtins;
pub mod call_context;
pub mod canonical_abi;
pub mod components;
pub mod execution_engine;
pub mod export;
pub mod import;
pub mod instance;
#[cfg(not(feature = "std"))]
pub mod instance_no_std;
pub mod memory_layout;
pub mod resource_management;
pub mod resources;
pub mod runtime;
pub mod string_encoding;
pub mod strategies;
pub mod type_conversion;
pub mod types;
pub mod unified_execution_agent;
pub mod values;

// Async support
#[cfg(feature = "component-model-async")]
pub mod async_;

// Threading support
#[cfg(feature = "component-model-threading")]
pub mod threading;

// Include verification module when the kani feature is enabled
#[cfg(feature = "kani")]
pub mod verify;

// Essential re-exports only
pub use builtins::{BuiltinHandler, BuiltinRegistry};
pub use canonical_abi::canonical::CanonicalABI;

// Component types based on feature flags
#[cfg(feature = "std")]
pub use components::component::{Component, ExternValue, FunctionValue, GlobalValue, MemoryValue, TableValue};

#[cfg(not(feature = "std"))]
pub use components::component_no_std::{
    Component, ComponentBuilder, ExternValue, FunctionValue, GlobalValue,
    MemoryValue, RuntimeInstance, TableValue, WrtComponentType, WrtComponentTypeBuilder,
};

// Component registry based on feature flags
#[cfg(feature = "std")]
pub use components::component_registry::ComponentRegistry;

#[cfg(not(feature = "std"))]
pub use components::component_registry_no_std::ComponentRegistry;

// Type alias for convenience
pub type WrtResult<T> = wrt_error::Result<T>;

// Debug printing macro for both std and no_std environments
#[cfg(feature = "std")]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            println!($($arg)*);
        }
        #[cfg(not(debug_assertions))]
        {
            // Do nothing in release mode
        }
    };
}

#[cfg(not(feature = "std"))]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        // In no_std, we'll just discard debug output for now
        // In a real implementation, this might write to a debug buffer
    };
}

// Make the macro available to submodules
pub(crate) use debug_println;