//! WebAssembly Runtime (WRT)
//!
//! A pure Rust implementation of the WebAssembly runtime, supporting the WebAssembly Core
//! and Component Model specifications.

#![deny(clippy::all)]
#![deny(clippy::perf)]
#![deny(clippy::nursery)]
#![deny(clippy::cargo)]
#![warn(clippy::pedantic)]
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
pub use error::{kinds, Error, Result};

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

/// Module for stackless WebAssembly execution
pub mod stackless;

/// Extensions for the stackless WebAssembly execution engine
pub mod stackless_extensions;

/// Module for WebAssembly table
pub mod table;

/// Module for WebAssembly type definitions
pub mod types;

/// Module for WebAssembly runtime values
pub mod values;

/// Module for WebAssembly logging functionality
pub mod logging;

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

// Public exports
pub use crate::{stackless::StacklessEngine, stackless_frame::StacklessFrame};
pub use behavior::InstructionExecutor;
pub use component::{Component, Host, InstanceValue};
pub use execution::ExecutionStats;
pub use global::Global;
pub use instructions::{types::BlockType, Instruction};
pub use memory::PAGE_SIZE;
pub use module::{ExportKind, Function, Import, Module, OtherExport};
pub use table::Table;
pub use types::{
    ComponentType, ExternType, FuncType, GlobalType, MemoryType, TableType, ValueType,
};
pub use values::Value;

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
#[must_use]
pub fn new_memory(mem_type: MemoryType) -> DefaultMemory {
    DefaultMemory::new(mem_type)
}

/// Creates a new WebAssembly table instance
#[must_use]
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

/// Creates a new global array
pub fn new_globals() -> Vec<std::sync::Arc<Global>> {
    use crate::global::Global;
    Vec::new()
}

/// Executes a test with the stackless engine.
///
/// # Errors
///
/// Returns an error if the test execution fails, such as when loading the module,
/// instantiating the module, or executing the test itself.
#[cfg(feature = "wat-parsing")]
pub fn execute_test_with_stackless(path: &str) -> Result<()> {
    // Parse the WAT to WASM
    let wasm = wat::parse_file(path).map_err(|e| Error::Parse(e.to_string()))?;

    // Create a new module
    let mut module = Module::new()?;
    let module = module.load_from_binary(&wasm)?;

    println!(
        "Successfully loaded module with {} memory definitions",
        module.memories_len()
    );
    println!("Memory types: {:?}", module.memories);
    println!(
        "Exports: {}",
        module
            .exports
            .iter()
            .map(|e| format!("{} (kind={:?}, idx={})", e.name, e.kind, e.index))
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Initialize the StacklessVM
    let mut engine = new_stackless_engine();
    let instance_idx = engine.instantiate(module.clone())?;

    // Set a fuel limit to prevent infinite loops
    engine.set_fuel(Some(1000000));

    // Find the 'run' export in the module
    let export = module
        .exports
        .iter()
        .find(|e| e.name == "run" && e.kind == crate::module::ExportKind::Function)
        .ok_or_else(|| Error::Execution("No 'run' export found".into()))?;

    println!("Found 'run' export at index {}", export.index);

    // Set up the engine state for execution
    let instance_idx_usize = instance_idx
        .try_into()
        .map_err(|_| Error::InvalidInstanceIndex(instance_idx))?;
    engine.stack.execute(
        engine,
        instance_idx_usize,
        export.index as usize,
        Vec::new(),
    )?;

    // Check if we have a result
    if let Some(result) = engine.stack.values.last() {
        if *result == Value::I32(1) {
            Ok(())
        } else {
            Err(Error::Execution(format!(
                "Test failed: expected 1, got {result:?}"
            )))
        }
    } else {
        Err(Error::Execution("Expected I32 result".to_string()))
    }
}

/// Executes an exported function by name from a specific instance in the engine.
///
/// This function looks up the export by name and calls it using the engine.
///
/// # Errors
///
/// Returns an error if the export is not found, if the export is not a function,
/// or if there is an error during the function execution.
pub fn execute_export_by_name(
    instance_idx: usize,
    name: &str,
    engine: &mut ExecutionEngine,
) -> Result<Vec<Value>> {
    use std::vec::Vec;

    match find_export_by_name(instance_idx, name, engine) {
        Some(export) => {
            // Check if this is a function export, otherwise return an error
            if export.kind != ExportKind::Function {
                return Err(Error::new(kinds::ExportNotFoundError(format!(
                    "Export {name} is not a function"
                ))));
            }

            // Execute the function with an empty arguments vector
            let mut result = engine.execute(instance_idx, export.index as usize, Vec::new());

            // Keep trying execution in case of fuel exhaustion
            while let Err(e) = &result {
                if e.is_fuel_exhausted() {
                    engine.stats.fuel_exhausted_count += 1;
                    // Try one more time
                    result = engine.execute(instance_idx, export.index as usize, Vec::new());
                } else {
                    // Other error, return it
                    break;
                }
            }

            return result;
        }
        None => Err(Error::new(kinds::ExportNotFoundError(format!(
            "Export {name} not found"
        )))),
    }
}

/// Find an export by name in a module instance
fn find_export_by_name(
    instance_idx: usize,
    name: &str,
    engine: &ExecutionEngine,
) -> Option<OtherExport> {
    if instance_idx >= engine.instances.len() {
        return None;
    }

    // Look for the export in the module's exports
    for export in &engine.module.exports {
        if export.name == name {
            return Some(export.clone());
        }
    }

    None
}

/// Gets the memory count of a module
pub fn memories_len(module: &Module) -> Result<usize> {
    Ok(module.memories_len())
}

/// Gets the number of tables in a module
pub fn tables_len(module: &Module) -> Result<usize> {
    Ok(module.tables_len())
}

/// Gets the number of globals in a module
pub fn globals_len(module: &Module) -> Result<usize> {
    Ok(module.globals_len())
}

pub fn new() -> Result<ExecutionEngine> {
    ExecutionEngine::new_from_result(Module::new())
}

impl StacklessEngine {
    pub fn execute(
        &mut self,
        instance_idx: usize,
        func_idx: u32,
        args: Vec<Value>,
    ) -> Result<Vec<Value>> {
        // Lock the instances mutex to get access to the Vec
        let instances_guard = self.instances.lock();
        // Access the instance using the guard
        let instance = instances_guard
            .get(instance_idx)
            .ok_or_else(|| Error::new(kinds::InvalidInstanceIndexError(instance_idx)))?;

        // Get the function type to determine number of results
        let func = instance
            .module
            .get_function(func_idx)
            .ok_or_else(|| Error::new(kinds::InvalidFunctionIndexError(func_idx as usize)))?;
        let func_type = instance
            .module
            .get_function_type(func.type_idx)
            .ok_or_else(|| {
                Error::new(kinds::InvalidFunctionTypeError(format!(
                    "Function type not found for index {}",
                    func.type_idx
                )))
            })?;

        let result_count = func_type.results.len();

        println!(
            "DEBUG: Function type has {} parameters and {} results",
            func_type.params.len(),
            result_count
        );

        // Use the module from the ModuleInstance
        let mut frame = StacklessFrame::new(
            instance.module.clone(),
            func_idx,
            &args,
            instance_idx as u32,
        )?;

        // Get a concrete stack implementation for execution
        let mut stack = Vec::<Value>::new();

        // Execute the frame with our concrete stack
        // frame.execute(&mut stack)?; // << COMMENTED OUT - Method doesn't exist, needs revisit
        println!(
            "WARN: StacklessFrame::execute commented out in wrt/src/lib.rs - execution logic needs revisit"
        );

        println!(
            "DEBUG: After execution, stack has {} values: {:?}",
            stack.len(),
            stack
        );

        // Take the top 'result_count' values from the stack as our results
        let mut results = Vec::with_capacity(result_count);

        // Make sure we have enough values on the stack
        if stack.len() < result_count {
            return Err(Error::new(kinds::ExecutionError(format!(
                "Function did not produce enough results. Expected {}, got {}",
                result_count,
                stack.len()
            ))));
        }

        // Return the appropriate number of results
        if result_count > 0 {
            // Take values from the end of the stack (most recently pushed)
            let start_index = stack.len() - result_count;
            for i in 0..result_count {
                results.push(stack[start_index + i].clone());
            }
        }

        println!("DEBUG: Returning {} results: {:?}", results.len(), results);

        Ok(results)
    }
}

#[cfg(feature = "wat-parsing")]
pub fn execute_wasm_test<F>(wat_string: &str, test_fn: F) -> Result<()>
where
    F: FnOnce(&mut StacklessEngine) -> Result<()>,
{
    let module = Module::from_wat(wat_string)?;
    let _frame = StacklessFrame::new(
        module.clone(),
        0,   // Assuming function index 0 if needed, adjust as necessary
        &[], // No initial arguments for the frame itself usually
        0,   // Assuming instance index 0
    )?;
    let _stack = Vec::<Value>::new();
    let mut engine = StacklessEngine::new_with_module(module);
    test_fn(&mut engine)
}

/// Retrieves the length of memories for a given module instance
pub fn memories_len_for_instance(instance_idx: usize, engine: &ExecutionEngine) -> Result<usize> {
    if instance_idx >= engine.instances.len() {
        return Err(Error::new(kinds::InvalidInstanceIndexError(instance_idx)));
    }

    // Use the engine's module instead of the instance's module
    Ok(engine.module.memories_len())
}

// Remove duplicate module declarations as they're already declared at the top of the file
// Keep error module only for minimal feature
// pub mod error;

// All other modules depend on std feature - these should be removed
// They're already declared above with different conditional compilation attributes
// The declarations below conflict with the ones above
/*
#[cfg(feature = "std")]
pub mod behavior;
#[cfg(feature = "std")]
pub mod component;
#[cfg(feature = "std")]
pub mod execution;
#[cfg(feature = "std")]
pub mod global;
#[cfg(feature = "std")]
pub mod instructions;
#[cfg(feature = "std")]
pub mod interface;
#[cfg(feature = "std")]
pub mod logging;
#[cfg(feature = "std")]
pub mod memory;
#[cfg(feature = "std")]
pub mod module;
#[cfg(feature = "std")]
pub mod module_instance;
#[cfg(feature = "std")]
pub mod resource;
#[cfg(feature = "std")]
pub mod stack;
#[cfg(feature = "std")]
pub mod stackless;
#[cfg(feature = "std")]
pub mod stackless_extensions;
#[cfg(feature = "std")]
pub mod stackless_frame;
#[cfg(feature = "std")]
pub mod table;
#[cfg(feature = "std")]
pub mod shared_instructions;
#[cfg(feature = "std")]
pub mod types;
#[cfg(feature = "std")]
pub mod values;
*/

// Include only required public exports for minimal feature
#[cfg(not(feature = "std"))]
pub use error::*;

// For full std feature, include all public exports
#[cfg(feature = "std")]
pub use {error::*, execution::*, logging::*, resource::*};

// Explicit type re-exports to avoid ambiguity
#[cfg(feature = "std")]
pub use {
    behavior::{ControlFlow, ControlFlowBehavior, FrameBehavior, Label, StackBehavior},
    interface::Interface,
    memory::DefaultMemory,
    module_instance::ModuleInstance,
};

// Re-export CloneableFn and HostFunctionHandler specifically
pub use logging::{CloneableFn, HostFunctionHandler};

// Synchronization primitives for WRT.
// Re-exports synchronization primitives from the wrt-sync crate.

// Re-export the mutex and rwlock types from wrt-sync
pub use wrt_sync::{
    WrtMutex as Mutex, WrtMutexGuard as MutexGuard, WrtRwLock as RwLock,
    WrtRwLockReadGuard as RwLockReadGuard, WrtRwLockWriteGuard as RwLockWriteGuard,
};

// Remove all the parking lock imports that don't exist
// #[cfg(feature = "std")]
// pub use wrt_sync::WrtParkingRwLock as ParkingRwLock;

// #[cfg(feature = "std")]
// pub use wrt_sync::{
//     WrtParkingRwLockReadGuard as ParkingRwLockReadGuard,
//     WrtParkingRwLockWriteGuard as ParkingRwLockWriteGuard,
// };
