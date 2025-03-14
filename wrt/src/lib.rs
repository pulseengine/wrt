//! WebAssembly Runtime (WRT)
//!
//! A pure Rust implementation of the WebAssembly runtime, supporting the WebAssembly Core
//! and Component Model specifications.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
// Disable because it's unstable
// #![warn(rustdoc::missing_doc_code_examples)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "std"))]
extern crate core as std;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "std")]
pub use std::boxed::Box;
#[cfg(feature = "std")]
pub use std::format;
#[cfg(feature = "std")]
pub use std::string::{String, ToString};
#[cfg(feature = "std")]
pub use std::sync::Mutex;
#[cfg(feature = "std")]
pub use std::vec::{self, Vec};

#[cfg(not(feature = "std"))]
pub use crate::sync::Mutex;
#[cfg(not(feature = "std"))]
pub use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
pub use alloc::format;
#[cfg(not(feature = "std"))]
pub use alloc::string::{String, ToString};
#[cfg(not(feature = "std"))]
pub use alloc::vec::{self, Vec};

// Core WebAssembly modules

/// Module for WebAssembly component model implementation
mod component;

/// Module for error definitions
mod error;

/// Module for WebAssembly execution environment
mod execution;

/// Module for WebAssembly global variables
mod global;

/// Module for WebAssembly instructions
mod instructions;

/// Module for WebAssembly linear memory
mod memory;

/// Module for WebAssembly module definitions
mod module;

/// Module for WebAssembly table
mod table;

/// Module for WebAssembly type definitions
mod types;

/// Module for WebAssembly runtime values
mod values;

/// Module for WebAssembly logging functionality
mod logging;

/// Module for synchronization primitives in no_std environment
#[cfg(not(feature = "std"))]
mod sync;

// Public exports
pub use component::{Component, Host, InstanceValue};
pub use error::{Error, Result};
pub use execution::{Engine, ExecutionStats, Stack};
pub use global::{Global, Globals};
pub use instructions::{BlockType, Instruction};
pub use logging::{CallbackRegistry, LogLevel, LogOperation};
pub use memory::Memory;
pub use module::Module;
pub use table::Table;
pub use types::{
    ComponentType, ExternType, FuncType, GlobalType, MemoryType, TableType, ValueType,
};
pub use values::Value;

/// Version of the WebAssembly Core specification implemented
pub const CORE_VERSION: &str = "1.0";

/// Version of the WebAssembly Component Model specification implemented
pub const COMPONENT_VERSION: &str = "0.1.0";

/// Creates a new WebAssembly engine
pub fn new_engine() -> Engine {
    Engine::new()
}

/// Creates a new WebAssembly module
pub fn new_module() -> Module {
    Module::new()
}

/// Creates a new WebAssembly memory instance
pub fn new_memory(mem_type: MemoryType) -> Memory {
    Memory::new(mem_type)
}

/// Creates a new WebAssembly table instance
pub fn new_table(table_type: TableType) -> Table {
    Table::new(table_type)
}

/// Creates a new WebAssembly global instance
///
/// # Parameters
///
/// * `global_type` - The type of the global variable
/// * `value` - The initial value of the global variable
///
/// # Returns
///
/// A new global instance with the specified type and initial value
///
/// # Errors
///
/// Returns `Error::Validation` if the value type does not match the global type
pub fn new_global(global_type: GlobalType, value: Value) -> Result<Global> {
    Global::new(global_type, value)
}

/// Creates a new collection of WebAssembly global instances
pub fn new_globals() -> Globals {
    Globals::new()
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    /// This test demonstrates how panics are reported
    ///
    /// ```
    /// // This shows usage of the library and how to handle errors
    /// use wrt::*;
    /// let module = new_module();
    /// // Always use error handling rather than unwrap to avoid panics
    /// if let Err(e) = module.validate() {
    ///     println!("Validation error: {}", e);
    /// }
    /// ```
    #[test]
    fn test_panic_documentation() {
        // This test doesn't actually do anything, it's just for the doc comment
    }
}
