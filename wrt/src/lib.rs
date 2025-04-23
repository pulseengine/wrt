//! WebAssembly Runtime (WRT)
//!
//! A pure Rust implementation of the WebAssembly runtime, supporting the WebAssembly Core
//! and Component Model specifications.

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

#[cfg(not(feature = "std"))]
extern crate alloc;

// Import and re-export types from std when available
#[cfg(feature = "std")]
pub use std::{
    boxed::Box,
    collections::HashMap,
    format,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};

// Import and re-export types for no_std environment
#[cfg(not(feature = "std"))]
pub use alloc::{
    boxed::Box,
    collections::BTreeMap as HashMap,
    format,
    string::{String, ToString},
    vec::Vec,
};

/// Re-export needed traits and types at crate level
pub use wrt_error::{kinds, Error, Result};

// Import and re-export from wrt-runtime
pub use wrt_runtime::{FuncType, GlobalType, Memory, MemoryType, Table, TableType, PAGE_SIZE};

// Use runtime Global directly
pub use wrt_runtime::global::Global;

// Core WebAssembly modules

/// Macro for debugging print statements that only compile with the "std" feature
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        #[cfg(feature = "std")]
        eprintln!($($arg)*);
    }
}

/// Module for WebAssembly component model implementation
pub mod component;

/// Module for WebAssembly error handling
pub mod error;

/// Module for WebAssembly execution
pub mod execution;

/// Module for WebAssembly global variables
pub mod global;

/// Module for WebAssembly instructions
pub mod instructions;

/// Module for WebAssembly Component Model interface types
pub mod interface;

/// Module for WebAssembly linear memory
pub mod memory;

/// Module for WebAssembly module definitions
pub mod module;

/// Module for WebAssembly Component Model resource handling
pub mod resource;

/// Module for WebAssembly serialization
pub mod serialization;

/// Adapter for WebAssembly format handling
pub mod format_adapter;

/// Integration layer for wrt-decoder with functional safety
pub mod decoder_integration;

/// Module for WebAssembly table
pub mod table;

/// Module for WebAssembly type definitions
pub mod types;

/// Module for WebAssembly runtime values
pub mod values;

/// Module for WebAssembly synchronization primitives in no_std environment
#[cfg(not(feature = "std"))]
pub mod sync;

/// Shared instruction implementations for all engines
pub mod shared_instructions;

/// Module for WebAssembly behavior
pub mod behavior;

/// Module for WebAssembly module instance
pub mod module_instance;

/// Module for WebAssembly stackless frame
pub mod stackless_frame;

/// Module for WebAssembly stackless execution engine
pub mod stackless;

/// Module for WebAssembly stackless memory adapter
pub mod memory_adapter;

// Public exports
pub use crate::{stackless::StacklessEngine, stackless_frame::StacklessFrame};
pub use behavior::InstructionExecutor;
pub use component::{Component, Host, InstanceValue};
pub use execution::ExecutionStats;
pub use instructions::{types::BlockType, Instruction};
pub use module::{ExportKind, Function, Import, Module, OtherExport};
pub use types::{ComponentType, ExternType};

// Use wrt_types values
pub use wrt_types::values::Value;

// Reexport wrt_types types to avoid duplicate imports in user code
pub use wrt_types::types::Limits;

/// Version of the WebAssembly Core specification implemented
pub const CORE_VERSION: &str = "1.0";

/// Version of the WebAssembly Component Model specification implemented
pub const COMPONENT_VERSION: &str = "0.1.0";

/// Uses execution engine implementation instead of redefining
pub use execution::Engine as ExecutionEngine;

/// Creates a new WebAssembly engine
#[must_use]
pub fn new_engine() -> ExecutionEngine {
    let module = Module::new().expect("Failed to create new empty module");
    ExecutionEngine::new(module)
}

/// Creates a new stackless WebAssembly engine
///
/// The stackless engine uses a state machine approach instead of recursion,
/// making it suitable for environments with limited stack space and for `no_std` contexts.
/// It also supports fuel-bounded execution for controlled resource usage.
#[must_use]
pub fn new_stackless_engine() -> stackless::StacklessEngine {
    stackless::StacklessEngine::new()
}

/// Creates a new WebAssembly module
#[must_use]
pub fn new_module() -> Result<Module> {
    Module::new()
}

/// Creates a new WebAssembly memory instance
///
/// This now uses the wrt-runtime Memory implementation
#[must_use]
pub fn new_memory(mem_type: MemoryType) -> Memory {
    Memory::new(mem_type).expect("Failed to create new memory instance")
}

/// Creates a new WebAssembly memory adapter
///
/// For backward compatibility
#[must_use]
pub fn new_memory_adapter(mem_type: MemoryType) -> Memory {
    Memory::new(mem_type).expect("Failed to create new memory adapter")
}

/// Create a new table with the specified type
///
/// # Parameters
///
/// * `table_type` - The type of the table to create
///
/// # Returns
///
/// A new table instance with the specified type
pub fn new_table(table_type: TableType) -> Table {
    Table::new(
        table_type.clone(),
        Value::default_for_type(&table_type.element_type),
    )
    .unwrap()
}

/// Create a new table with the specified type
///
/// # Parameters
///
/// * `table_type` - The type of the table to create
///
/// # Returns
///
/// A new table instance with the specified type
pub fn new_table_adapter(table_type: TableType) -> Table {
    Table::new(
        table_type.clone(),
        Value::default_for_type(&table_type.element_type),
    )
    .unwrap()
}

/// Create a new global with the specified type and value
pub fn new_global(global_type: GlobalType, value: Value) -> Result<Global> {
    Ok(Global::new(global_type, value))
}

/// Creates a new global array
pub fn new_globals() -> Vec<std::sync::Arc<Global>> {
    Vec::new()
}

// Explicit type re-exports to avoid ambiguity
#[cfg(feature = "std")]
pub use {
    behavior::{ControlFlow, ControlFlowBehavior, FrameBehavior, Label, StackBehavior},
    interface::Interface,
    module_instance::ModuleInstance,
};

// Re-export types from wrt-logging
pub use wrt_logging::{LogLevel, LogOperation};
// Re-export CallbackRegistry only if std feature is enabled
#[cfg(feature = "std")]
pub use wrt_logging::CallbackRegistry;

// List of module re-exports
pub use crate::{
    // error::Error, // Already imported above
    // error::Result, // Already imported above
    // execution::ExecutionStats, // Already imported in mod declarations
    // global::Global, // Already imported above
    // interface::Interface, // Already imported above
    module::ExportValue,
    // module::Function, // Already imported in mod declarations
    // module::Import, // Already imported in mod declarations
    // module::ImportType, // Commented out to fix compilation error
    // module::Module, // Already imported in mod declarations
    // module::OtherExport, // Already imported in mod declarations
    resource::ResourceTable,
    // stackless::StacklessEngine, // Already imported in mod declarations
    // values::Value, // Already imported in mod declarations
};
