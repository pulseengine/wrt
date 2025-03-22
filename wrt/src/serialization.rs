//! Provides serialization and deserialization functionality for WebAssembly runtime state.
//!
//! This module contains structures and traits for serializing and deserializing
//! WebAssembly runtime state, allowing for migration of execution state between
//! different machines or for checkpointing and restoring execution.

// Required imports for serialization
use bincode;
use serde::{Deserialize, Serialize};
use serde_json;

// WRT imports
use crate::error::{Error, Result};
use crate::execution::{Engine, ExecutionState, ModuleInstance};
use crate::execution::{FunctionAddr, GlobalAddr, MemoryAddr, TableAddr};
use crate::global::Global;
use crate::memory::{Memory, PAGE_SIZE};
use crate::table::Table;
use crate::types::{GlobalType, MemoryType, TableType, ValueType};
use crate::values::Value;
use crate::Module;

/// Represents a serializable execution state.
#[derive(Serialize, Deserialize)]
pub enum SerializableExecutionState {
    /// Ready state (equivalent to Idle)
    Ready,
    /// Paused state
    Paused {
        /// Instance index
        instance_idx: usize,
        /// Function index
        func_idx: u32,
        /// Program counter
        pc: usize,
        /// Expected results
        expected_results: usize,
    },
    /// Completed state (equivalent to Finished)
    Completed,
    /// Error state
    Error(String),
}

/// Represents a serializable table.
#[derive(Serialize, Deserialize)]
pub struct SerializableTable {
    /// Elements in the table
    pub elements: Vec<Option<Value>>,
    /// Maximum size of the table
    pub max: Option<u32>,
}

/// Represents a serializable memory.
#[derive(Serialize, Deserialize)]
pub struct SerializableMemory {
    /// Memory data
    pub data: Vec<u8>,
    /// Current size in pages
    pub size: u32,
    /// Maximum size in pages
    pub max: Option<u32>,
}

/// Represents a serializable global.
#[derive(Serialize, Deserialize)]
pub struct SerializableGlobal {
    /// Global value
    pub value: Value,
    /// Whether the global is mutable
    pub mutable: bool,
}

/// Represents a serializable stack.
#[derive(Serialize, Deserialize)]
pub struct SerializableStack {
    /// Values on the stack
    pub values: Vec<Value>,
}

/// Represents a serializable module instance.
#[derive(Serialize, Deserialize)]
pub struct SerializableModuleInstance {
    /// Module index in the engine instances array
    pub module_idx: u32,
    /// Tables defined in the module
    pub tables: Vec<SerializableTable>,
    /// Memories defined in the module
    pub memories: Vec<SerializableMemory>,
    /// Globals defined in the module
    pub globals: Vec<SerializableGlobal>,
    /// Function addresses
    pub func_addrs: Vec<u32>,
    /// Table addresses
    pub table_addrs: Vec<u32>,
    /// Memory addresses
    pub memory_addrs: Vec<u32>,
    /// Global addresses
    pub global_addrs: Vec<u32>,
}

/// Represents a serializable engine state.
#[derive(Serialize, Deserialize)]
pub struct SerializableState {
    /// Current execution state
    state: SerializableExecutionState,
    /// Module instances
    instances: Vec<SerializableModuleInstance>,
    /// Stacks
    stacks: Vec<SerializableStack>,
    /// Function addresses
    functions: Vec<u32>,
    /// Table addresses
    tables: Vec<u32>,
    /// Memory addresses
    memories: Vec<u32>,
    /// Global addresses
    globals: Vec<u32>,
    /// Remaining fuel
    fuel: Option<u64>,
}

/// Trait defining serialization methods.
pub trait Serializable: Sized {
    /// Converts the object to a serializable state.
    fn to_serializable(&self) -> Result<SerializableState>;

    /// Creates an object from a serializable state.
    fn from_serializable(serialized: SerializableState) -> Result<Self>;

    /// Converts the object to a JSON string.
    fn to_json(&self) -> Result<String> {
        let serializable = self.to_serializable()?;
        match serde_json::to_string_pretty(&serializable) {
            Ok(json) => Ok(json),
            Err(e) => Err(Error::Serialization(e.to_string())),
        }
    }

    /// Creates an object from a JSON string.
    fn from_json(json: &str) -> Result<Self> {
        match serde_json::from_str(json) {
            Ok(serializable) => Self::from_serializable(serializable),
            Err(e) => Err(Error::Serialization(e.to_string())),
        }
    }

    /// Converts the object to a binary format.
    fn to_binary(&self) -> Result<Vec<u8>> {
        let serializable = self.to_serializable()?;
        match bincode::serialize(&serializable) {
            Ok(binary) => Ok(binary),
            Err(e) => Err(Error::Serialization(e.to_string())),
        }
    }

    /// Creates an object from binary data.
    fn from_binary(data: &[u8]) -> Result<Self> {
        match bincode::deserialize(data) {
            Ok(serializable) => Self::from_serializable(serializable),
            Err(e) => Err(Error::Serialization(e.to_string())),
        }
    }
}

/// Implementation of the Serializable trait for Engine
#[cfg(feature = "serialization")]
impl Serializable for Engine {
    fn to_serializable(&self) -> Result<SerializableState> {
        // Create serializable state from engine
        let mut state = SerializableState {
            state: match self.get_execution_state() {
                ExecutionState::Idle => SerializableExecutionState::Ready,
                ExecutionState::Running => SerializableExecutionState::Ready, // Running becomes ready when serialized
                ExecutionState::Paused {
                    instance_idx,
                    func_idx,
                    pc,
                    expected_results,
                } => SerializableExecutionState::Paused {
                    instance_idx: *instance_idx as usize,
                    func_idx: *func_idx,
                    pc: *pc,
                    expected_results: *expected_results,
                },
                ExecutionState::Finished => SerializableExecutionState::Completed,
            },
            instances: Vec::new(),
            stacks: Vec::new(),
            functions: Vec::new(),
            tables: Vec::new(),
            memories: Vec::new(),
            globals: Vec::new(),
            fuel: self.remaining_fuel(),
        };

        // Serialize instances
        for (instance_idx, instance) in self.get_instances().iter().enumerate() {
            let serializable_instance = SerializableModuleInstance {
                module_idx: instance.module_idx,
                tables: instance
                    .tables
                    .iter()
                    .map(|&table_idx| {
                        let table = &self.get_tables()[table_idx as usize];
                        SerializableTable {
                            elements: Vec::new(), // We don't have access to table elements directly
                            max: table.type_.max,
                        }
                    })
                    .collect(),
                memories: instance
                    .memories
                    .iter()
                    .map(|&memory_idx| {
                        let memory = &self.get_memories()[memory_idx as usize];
                        SerializableMemory {
                            data: memory.data.clone(),
                            size: memory.data.len() as u32 / (64 * 1024), // Convert bytes to pages
                            max: None, // Memory doesn't have a max field
                        }
                    })
                    .collect(),
                globals: instance
                    .globals
                    .iter()
                    .map(|&global_idx| {
                        let global = &self.get_globals()[global_idx as usize];
                        SerializableGlobal {
                            value: global.value.clone(),
                            mutable: global.mutable,
                        }
                    })
                    .collect(),
                func_addrs: instance
                    .func_addrs
                    .iter()
                    .map(|addr| addr.func_idx)
                    .collect(),
                table_addrs: instance
                    .table_addrs
                    .iter()
                    .map(|addr| addr.table_idx)
                    .collect(),
                memory_addrs: instance
                    .memory_addrs
                    .iter()
                    .map(|addr| addr.memory_idx)
                    .collect(),
                global_addrs: instance
                    .global_addrs
                    .iter()
                    .map(|addr| addr.global_idx)
                    .collect(),
            };
            state.instances.push(serializable_instance);
        }

        // Serialize stack
        if let Some(ref stack) = self.stack {
            let serializable_stack = SerializableStack {
                values: stack.values.clone(),
            };
            state.stacks.push(serializable_stack);
        }

        // Collect all function addresses
        for inst in self.get_instances() {
            for addr in &inst.func_addrs {
                state.functions.push(addr.func_idx);
            }
        }

        // Collect all table addresses
        for inst in self.get_instances() {
            for addr in &inst.table_addrs {
                state.tables.push(addr.table_idx);
            }
        }

        // Collect all memory addresses
        for inst in self.get_instances() {
            for addr in &inst.memory_addrs {
                state.memories.push(addr.memory_idx);
            }
        }

        // Collect all global addresses
        for inst in self.get_instances() {
            for addr in &inst.global_addrs {
                state.globals.push(addr.global_idx);
            }
        }

        Ok(state)
    }

    fn from_serializable(serialized: SerializableState) -> Result<Self> {
        // Create a new empty engine
        let module = Module::empty();
        let mut engine = Engine::new(module);

        // Set the engine state
        engine.set_execution_state(match serialized.state {
            SerializableExecutionState::Ready => ExecutionState::Idle,
            SerializableExecutionState::Paused {
                instance_idx,
                func_idx,
                pc,
                expected_results,
            } => ExecutionState::Paused {
                instance_idx: instance_idx as u32,
                func_idx,
                pc,
                expected_results,
            },
            SerializableExecutionState::Completed => ExecutionState::Finished,
            SerializableExecutionState::Error(msg) => return Err(Error::Execution(msg)),
        });

        // Set fuel
        engine.set_fuel(serialized.fuel);

        // Create and populate instances
        for instance in serialized.instances {
            let mut module_instance = ModuleInstance {
                module_idx: instance.module_idx,
                module: module.clone(),
                func_addrs: Vec::new(),
                table_addrs: Vec::new(),
                memory_addrs: Vec::new(),
                global_addrs: Vec::new(),
                memories: Vec::new(),
                tables: Vec::new(),
                globals: Vec::new(),
            };

            // Restore tables
            for table in &instance.tables {
                let table_obj = table.clone();
                let table_idx = engine.add_table(table_obj);
                module_instance.table_addrs.push(TableAddr {
                    instance_idx: 0,
                    table_idx,
                });
            }

            // Restore memories
            for memory in &instance.memories {
                let memory_obj = Memory::new(MemoryType {
                    min: (memory.data.len() / PAGE_SIZE) as u32,
                    max: None,
                });
                memory_obj.data = memory.data.clone();
                let memory_idx = engine.add_memory(memory_obj);
                module_instance.memory_addrs.push(MemoryAddr {
                    instance_idx: 0,
                    memory_idx,
                });
            }

            // Restore globals
            for global in &instance.globals {
                let global_obj = Global::new(
                    GlobalType {
                        content_type: global.value.get_type(),
                        mutable: global.mutable,
                    },
                    global.value.clone(),
                )
                .unwrap();
                let global_idx = engine.add_global(global_obj);
                module_instance.global_addrs.push(GlobalAddr {
                    instance_idx: 0,
                    global_idx,
                });
            }

            // Restore function addresses
            for &func_idx in &instance.func_addrs {
                module_instance.func_addrs.push(FunctionAddr {
                    instance_idx: instance.module_idx,
                    func_idx,
                });
            }

            // Restore table addresses
            for &table_idx in &instance.table_addrs {
                module_instance.table_addrs.push(TableAddr {
                    instance_idx: 0,
                    table_idx,
                });
            }

            // Restore memory addresses
            for &memory_idx in &instance.memory_addrs {
                module_instance.memory_addrs.push(MemoryAddr {
                    instance_idx: 0,
                    memory_idx,
                });
            }

            // Restore global addresses
            for &global_idx in &instance.global_addrs {
                module_instance.global_addrs.push(GlobalAddr {
                    instance_idx: 0,
                    global_idx,
                });
            }

            // Add the instance
            engine.add_instance(module_instance);
        }

        // Restore stack
        if !serialized.stacks.is_empty() {
            let stack = crate::execution::Stack {
                values: serialized.stacks[0].values.clone(),
                call_frames: Vec::new(),
                labels: Vec::new(),
            };
            engine.stack = Some(stack);
        }

        Ok(engine)
    }
}

// Implementation of serialization methods for Engine
#[cfg(feature = "serialization")]
impl Engine {
    /// Saves the engine state to a JSON string
    pub fn save_state_json(&self) -> Result<String> {
        self.to_json()
    }

    /// Loads the engine state from a JSON string
    pub fn load_state_json(json: &str) -> Result<Self> {
        Self::from_json(json)
    }

    /// Saves the engine state to a binary format
    pub fn save_state_binary(&self) -> Result<Vec<u8>> {
        self.to_binary()
    }

    /// Loads the engine state from binary data
    pub fn load_state_binary(data: &[u8]) -> Result<Self> {
        Self::from_binary(data)
    }

    /// Creates a binary checkpoint of the current engine state
    pub fn create_checkpoint(&self) -> Result<Vec<u8>> {
        self.save_state_binary()
    }

    /// Restores the engine state from a checkpoint
    pub fn restore_from_checkpoint(data: &[u8]) -> Result<Self> {
        Self::load_state_binary(data)
    }
}

// Tests for serialization
#[cfg(all(test, feature = "serialization"))]
mod tests {
    use super::*;
    use crate::module::Module;
    use crate::values::Value;

    #[test]
    fn test_engine_serialization_json() {
        // Create a simple module that adds two integers
        let wat = r#"
            (module
                (func (export "add") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add
                )
            )
        "#;
        let module = Module::from_wat(wat).unwrap();

        // Create an engine and instantiate the module
        let mut engine = Engine::new(module);

        // Execute the add function with arguments 5 and 7
        let result = engine
            .invoke_export("add", &[Value::I32(5), Value::I32(7)])
            .unwrap();
        assert_eq!(result, Value::I32(12));

        // Serialize the engine state to JSON
        let json = engine.save_state_json().unwrap();

        // Deserialize the JSON back into an engine
        let mut restored_engine = Engine::load_state_json(&json).unwrap();

        // Execute the add function again with the restored engine
        let result = restored_engine
            .invoke_export("add", &[Value::I32(10), Value::I32(20)])
            .unwrap();
        assert_eq!(result, Value::I32(30));
    }

    #[test]
    fn test_engine_serialization_binary() {
        // Create a simple module that multiplies two integers
        let wat = r#"
            (module
                (func (export "multiply") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.mul
                )
            )
        "#;
        let module = Module::from_wat(wat).unwrap();

        // Create an engine and instantiate the module
        let mut engine = Engine::new(module);

        // Execute the multiply function with arguments 6 and 7
        let result = engine
            .invoke_export("multiply", &[Value::I32(6), Value::I32(7)])
            .unwrap();
        assert_eq!(result, Value::I32(42));

        // Serialize the engine state to binary
        let binary = engine.save_state_binary().unwrap();

        // Deserialize the binary back into an engine
        let mut restored_engine = Engine::load_state_binary(&binary).unwrap();

        // Execute the multiply function again with the restored engine
        let result = restored_engine
            .invoke_export("multiply", &[Value::I32(8), Value::I32(8)])
            .unwrap();
        assert_eq!(result, Value::I32(64));
    }

    #[test]
    fn test_checkpoint_restore() {
        // Create a module with a global counter
        let wat = r#"
            (module
                (global $counter (export "counter") (mut i32) (i32.const 0))
                (func (export "increment") 
                    global.get $counter
                    i32.const 1
                    i32.add
                    global.set $counter
                )
                (func (export "get_counter") (result i32)
                    global.get $counter
                )
            )
        "#;
        let module = Module::from_wat(wat).unwrap();

        // Create an engine and instantiate the module
        let mut engine = Engine::new(module);

        // Increment the counter 5 times
        for _ in 0..5 {
            engine.invoke_export("increment", &[]).unwrap();
        }

        // Verify the counter is at 5
        let counter = engine.invoke_export("get_counter", &[]).unwrap();
        assert_eq!(counter, Value::I32(5));

        // Create a checkpoint
        let checkpoint = engine.create_checkpoint().unwrap();

        // Increment the counter 5 more times
        for _ in 0..5 {
            engine.invoke_export("increment", &[]).unwrap();
        }

        // Verify the counter is at 10
        let counter = engine.invoke_export("get_counter", &[]).unwrap();
        assert_eq!(counter, Value::I32(10));

        // Restore from checkpoint
        let mut restored_engine = Engine::restore_from_checkpoint(&checkpoint).unwrap();

        // Verify the counter is back at 5
        let counter = restored_engine.invoke_export("get_counter", &[]).unwrap();
        assert_eq!(counter, Value::I32(5));
    }
}
