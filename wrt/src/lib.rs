//! WebAssembly Runtime (WRT)
//!
//! A pure Rust implementation of the WebAssembly runtime, supporting the WebAssembly Core
//! and Component Model specifications.
//!
//! WRT is designed to be compatible with both std and no_std environments, making it
//! suitable for a wide range of applications, from server-side WebAssembly execution
//! to embedded systems and bare-metal environments.
//!
//! ## Features
//!
//! - Full WebAssembly Core specification support
//! - Component Model implementation
//! - Stackless execution engine for environments with limited stack space
//! - no_std compatibility
//! - Comprehensive error handling
//! - Safe memory implementation

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::all)]
#![deny(clippy::perf)]
#![deny(clippy::nursery)]
#![deny(clippy::cargo)]
#![warn(clippy::pedantic)]
#![warn(clippy::missing_panics_doc)]
#![warn(missing_docs)]
// Disable because it's unstable
// #![warn(rustdoc::missing_doc_code_examples)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Define debug_println macro for conditional debug printing
#[cfg(feature = "std")]
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            println!($($arg)*);
        }
    };
}

#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {{
        // No-op in no_std environments unless we implement a different printing mechanism
    }};
}

// Include prelude module for consistent imports across crates
pub mod prelude;
// Don't glob-import the prelude to avoid name conflicts
// We'll use qualified imports instead

// Core module imports
mod behavior;
mod component;
mod decoder_integration;
mod error;
mod execution;
mod format_adapter;
mod global;
mod instructions;
mod instructions_adapter;
mod interface;
mod memory;
mod memory_adapter;
mod module;
mod module_instance;
mod resource;
mod serialization;
mod shared_instructions;
mod stack;
mod stackless;
mod stackless_extensions;
mod stackless_frame;
mod sync;
mod table;
mod types;
mod validation;
mod values;

// Public exports with careful naming to avoid conflicts
pub use crate::behavior::InstructionExecutor;
pub use crate::component::{Component, Host, InstanceValue};
pub use crate::execution::ExecutionStats;
pub use crate::global::Global;
pub use crate::instructions::Instruction;
pub use crate::memory::Memory;
pub use crate::module::{ExportKind, Function, Import, Module, OtherExport};
pub use crate::stackless::StacklessEngine;
pub use crate::stackless_frame::StacklessFrame;
pub use crate::table::Table;
pub use crate::types::BlockType;

// Import types from prelude with explicit namespace
use prelude::{RuntimeMemoryType, RuntimeTableType, TypesMemoryType, TypesTableType};

/// Version of the WebAssembly Core specification implemented
pub const CORE_VERSION: &str = "1.0";

/// Version of the WebAssembly Component Model specification implemented
pub const COMPONENT_VERSION: &str = "0.1.0";

/// Execution engine for WebAssembly modules
///
/// This type represents an execution engine that can be used to run WebAssembly modules.
#[derive(Debug, Clone, Copy)]
pub struct ExecutionEngine;

/// Create a new execution engine for WebAssembly modules.
///
/// This function creates a new execution engine that can be used to run
/// WebAssembly modules.
///
/// # Returns
///
/// A new execution engine.
pub fn new_engine() -> ExecutionEngine {
    ExecutionEngine
}

/// Create a new stackless execution engine for WebAssembly modules.
///
/// This function creates a new stackless execution engine that can be used to run
/// WebAssembly modules in environments with limited stack space.
///
/// # Returns
///
/// A new stackless execution engine.
pub fn new_stackless_engine() -> stackless::StacklessEngine {
    stackless::StacklessEngine::new()
}

/// Create a new, empty WebAssembly module.
///
/// # Returns
///
/// A `WrtResult` containing the new module, or an error if the module
/// could not be created.
pub fn new_module() -> prelude::WrtResult<Module> {
    Module::new()
}

/// Create a new WebAssembly memory with the given type.
///
/// # Arguments
///
/// * `mem_type` - The type of memory to create.
///
/// # Returns
///
/// A new memory instance.
pub fn new_memory(mem_type: TypesMemoryType) -> Memory {
    // Convert from wrt-types MemoryType to wrt-runtime MemoryType
    let runtime_mem_type = RuntimeMemoryType {
        limits: mem_type.limits.clone(),
    };

    Memory::new(runtime_mem_type).unwrap()
}

/// Create a new WebAssembly memory adapter with the given type.
///
/// # Arguments
///
/// * `mem_type` - The type of memory to create.
///
/// # Returns
///
/// A new memory adapter instance.
pub fn new_memory_adapter(mem_type: TypesMemoryType) -> Memory {
    // Convert from wrt-types MemoryType to wrt-runtime MemoryType
    let runtime_mem_type = RuntimeMemoryType {
        limits: mem_type.limits.clone(),
    };

    memory_adapter::MemoryAdapter::new(runtime_mem_type).unwrap()
}

/// Create a new WebAssembly table with the given type.
///
/// # Arguments
///
/// * `table_type` - The type of table to create.
///
/// # Returns
///
/// A new table instance.
pub fn new_table(table_type: TypesTableType) -> Table {
    // Convert from wrt-types TableType to wrt-runtime TableType
    let runtime_table_type = RuntimeTableType {
        element_type: table_type.element_type,
        limits: table_type.limits.clone(),
    };

    Table::new(runtime_table_type).unwrap()
}
