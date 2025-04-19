//! Stackless WebAssembly execution engine
//!
//! This module implements a stackless version of the WebAssembly execution engine
//! that doesn't rely on the host language's call stack, making it suitable for
//! environments with limited stack space and for no_std contexts.
//!
//! The stackless engine uses a state machine approach to track execution state
//! and allows for pausing and resuming execution at any point.

use crate::{
    behavior::{ControlFlowBehavior, FrameBehavior, InstructionExecutor, Label, StackBehavior},
    error::kinds::{InvalidInstanceIndexError, PoisonedLockError},
    execution::ExecutionStats,
    global::Global,
    instructions::{instruction_type::Instruction as InstructionType, Instruction},
    interface,
    logging::CloneableFn,
    memory::{DefaultMemory, MemoryBehavior},
    module::{Data, Element, ExportKind, Function, Import, Module, OtherExport},
    module_instance::ModuleInstance,
    resource::ResourceTable,
    stackless_frame::StacklessFrame,
    table::Table,
    types::*,
    values::Value,
    ControlFlow, HostFunctionHandler, LogOperation,
};
// Import directly from wrt_error and wrt_sync
use core::mem;
use log::trace;
use parking_lot::Mutex as ParkingLotMutex;
use std::collections::HashMap;
use std::sync::Arc;
use wrt_error::kinds;
use wrt_error::{Error, Result};
use wrt_sync::EngineMutex;

// --- Conditional imports for Mutex ---
// TODO: Define FuelOutcomes and EngineConfig or import if they exist elsewhere
#[derive(Debug)]
pub struct FuelOutcomes; // Placeholder
#[derive(Debug)]
pub struct EngineConfig; // Placeholder
                         // --- End added imports ---

/// Represents the execution state in a stackless implementation
#[derive(Debug, Clone)]
pub enum StacklessExecutionState {
    /// Executing instructions normally
    Running,
    /// Paused execution (for bounded fuel)
    Paused {
        /// Program counter (instruction index)
        pc: usize,
        /// Instance index
        instance_idx: u32,
        /// Function index
        func_idx: u32,
        /// Expected number of results
        expected_results: usize,
    },
    /// Function call in progress
    Calling {
        /// Instance index
        instance_idx: u32,
        /// Function index
        func_idx: u32,
        /// Arguments
        args: Vec<Value>,
        /// Return address (instruction index to return to)
        return_pc: usize,
    },
    /// Return in progress
    Returning {
        /// Return values
        values: Vec<Value>,
    },
    /// Branch in progress
    Branching {
        /// Branch target (label depth)
        depth: u32,
        /// Values to keep on stack
        values: Vec<Value>,
    },
    /// Completed execution
    Completed,
    /// Execution finished
    Finished,
    /// Error occurred
    Error(Error),
}

/// Represents the execution stack in a stackless implementation
#[derive(Debug)]
pub struct StacklessStack {
    /// Shared module reference
    module: Arc<Module>,
    /// Current instance index
    instance_idx: usize,
    /// The operand stack
    pub values: Vec<Value>,
    /// The label stack
    labels: Vec<Label>,
    /// Function frames
    pub frames: Vec<StacklessFrame>,
    /// Current execution state
    pub state: StacklessExecutionState,
    /// Instruction pointer
    pub pc: usize,
    /// Function index
    pub func_idx: u32,
    /// Capacity of the stack
    pub capacity: usize,
}

/// A callback registry for handling WebAssembly component operations
pub struct StacklessCallbackRegistry {
    /// Names of exports that are known to be callbacks
    pub export_names: HashMap<String, HashMap<String, LogOperation>>,
    /// Registered callback functions
    pub callbacks: HashMap<String, HostFunctionHandler>,
}

impl Default for StacklessCallbackRegistry {
    fn default() -> Self {
        Self {
            export_names: HashMap::new(),
            callbacks: HashMap::new(),
        }
    }
}

impl std::fmt::Debug for StacklessCallbackRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StacklessCallbackRegistry")
            .field("known_export_names", &self.export_names)
            .field("callbacks", &"<function>")
            .finish()
    }
}

/// State of the stackless WebAssembly execution engine
#[derive(Debug)]
pub struct StacklessEngine {
    /// The internal state of the stackless engine.
    /// The actual execution stack (values, labels, frames, state)
    pub(crate) exec_stack: StacklessStack,
    /// Remaining fuel for bounded execution
    fuel: Option<u64>,
    /// Execution statistics
    stats: ExecutionStats,
    /// Callback registry for host functions (logging, etc.)
    callbacks: Arc<ParkingLotMutex<StacklessCallbackRegistry>>,
    /// Maximum call depth for function calls
    max_call_depth: Option<usize>,
    /// Use the alias EngineMutex for the instance map
    pub(crate) instances: Arc<ParkingLotMutex<Vec<Arc<ModuleInstance>>>>,
}

impl StacklessStack {
    /// Creates a new `StacklessStack` with the given module.
    #[must_use]
    pub const fn new(module: Arc<Module>, instance_idx: usize) -> Self {
        Self {
            values: Vec::new(),
            labels: Vec::new(),
            frames: Vec::new(),
            state: StacklessExecutionState::Running,
            pc: 0,
            instance_idx,
            func_idx: 0,
            module,
            capacity: 1024, // Default capacity
        }
    }

    /// Pushes a value onto the stack
    pub fn push(&mut self, value: Value) -> Result<(), Error> {
        if self.values.len() >= self.capacity {
            return Err(Error::new(kinds::ExecutionError(
                "Stack overflow".to_string(),
            )));
        }
        self.values.push(value);
        Ok(())
    }

    /// Pops a value from the stack
    pub fn pop(&mut self) -> Result<Value, Error> {
        self.values
            .pop()
            .ok_or_else(|| Error::new(kinds::StackUnderflowError))
    }

    /// Pushes a label onto the control stack
    pub fn push_label(&mut self, arity: usize, pc: usize) -> Result<(), Error> {
        self.labels.push(Label {
            arity,
            pc,
            continuation: pc,
            stack_depth: self.values.len(), // Assuming stack_depth is current value stack len
            is_loop: false,                 // Default to false
            is_if: false,                   // Default to false
        });
        Ok(())
    }

    /// Pops a label from the control stack
    pub fn pop_label(&mut self) -> Result<Label, Error> {
        self.labels
            .pop()
            .ok_or_else(|| Error::new(kinds::StackUnderflowError))
    }

    /// Gets a label at the specified depth
    pub fn get_label(&self, idx: usize) -> Option<&Label> {
        self.labels.get(self.labels.len().checked_sub(1 + idx)?)
    }

    /// Returns the number of labels currently on the control stack.
    pub fn labels_len(&self) -> usize {
        self.labels.len()
    }

    /// Checks if the value stack is empty.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns the number of values on the value stack.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns a slice containing all values on the stack.
    pub fn values(&self) -> &[Value] {
        &self.values
    }

    /// Returns a mutable slice containing all values on the stack.
    pub fn values_mut(&mut self) -> &mut [Value] {
        &mut self.values
    }

    /// Returns a reference to the top value on the stack without removing it.
    pub fn peek(&self) -> Result<&Value, Error> {
        self.values
            .last()
            .ok_or(Error::new(kinds::StackUnderflowError))
    }

    /// Returns a mutable reference to the top value on the stack without removing it.
    pub fn peek_mut(&mut self) -> Result<&mut Value, Error> {
        self.values
            .last_mut()
            .ok_or_else(|| Error::new(kinds::StackUnderflowError))
    }

    // Note: Implementations of the `Stack` and `StackBehavior` traits for StacklessStack
    // are added below to maintain compatibility where the engine expects these traits.
}

impl Default for StacklessEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl StacklessEngine {
    /// Creates a new stackless WebAssembly engine
    #[must_use]
    pub fn new() -> Self {
        let empty_module = Arc::new(Module::empty()); // Use Module::empty()
        Self {
            exec_stack: StacklessStack::new(empty_module, 0), // Initialize exec_stack
            // Use the EngineMutex alias for initialization
            instances: Arc::new(ParkingLotMutex::new(Vec::new())),
            fuel: None,
            stats: ExecutionStats::default(),
            callbacks: Arc::new(ParkingLotMutex::new(StacklessCallbackRegistry {
                export_names: HashMap::new(),
                callbacks: HashMap::new(),
            })),
            max_call_depth: None,
        }
    }

    /// Sets the fuel limit for bounded execution
    pub fn set_fuel(&mut self, fuel: Option<u64>) {
        self.fuel = fuel;
    }

    /// Gets the remaining fuel
    #[must_use]
    pub const fn remaining_fuel(&self) -> Option<u64> {
        self.fuel
    }

    /// Gets the execution statistics
    #[must_use]
    pub const fn stats(&self) -> &ExecutionStats {
        &self.stats
    }

    /// Resets the execution statistics
    pub fn reset_stats(&mut self) {
        self.stats = ExecutionStats::default();
    }

    /// Gets the current execution state
    #[must_use]
    pub fn state(&self) -> &StacklessExecutionState {
        &self.exec_stack.state // Access via exec_stack
    }

    /// Sets the execution state
    pub fn set_state(&mut self, state: StacklessExecutionState) {
        self.exec_stack.state = state; // Access via exec_stack
    }

    /// Gets the number of module instances
    #[must_use]
    pub fn instance_count(&self) -> usize {
        // Restore locking logic
        match self.instances.try_lock() {
            Some(guard) => guard.len(),
            None => {
                // Handle poisoned lock or contention if necessary
                // Keep previous logic or adjust as needed
                0 // Assuming 0 on contention for now
            }
        }
    }

    /// Provides temporary access to a module instance by index via a closure.
    pub fn with_instance<F, R>(&self, instance_idx: usize, f: F) -> Result<R, Error>
    where
        F: FnOnce(&ModuleInstance) -> Result<R, Error>,
    {
        let instances_guard = self.instances.lock();
        let instance = instances_guard
            .get(instance_idx)
            .ok_or_else(|| Error::new(kinds::InvalidInstanceIndexError(instance_idx)))?;
        f(instance)
    }

    /// Provides temporary mutable access to a module instance by index via a closure.
    pub fn with_instance_mut<F, R>(&self, instance_idx: usize, f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut ModuleInstance) -> Result<R, Error>,
    {
        let mut instances_guard = self.instances.lock();
        let instance = instances_guard
            .get_mut(instance_idx)
            .ok_or_else(|| Error::new(kinds::InvalidInstanceIndexError(instance_idx)))?;
        // Attempt to get a mutable reference from Arc, might fail if Arc is shared
        if let Some(instance_mut) = Arc::get_mut(instance) {
            f(instance_mut)
        } else {
            Err(Error::new(kinds::ExecutionError(
                "Cannot get mutable access to shared ModuleInstance".into(),
            ))) // Corrected error
        }
    }

    /// Instantiates a module
    pub fn instantiate(&mut self, module: Module) -> Result<usize, Error> {
        // Module needs to be owned by the instance, so we clone it.
        // TODO: Consider if cloning the whole module is necessary or if Arc is sufficient.
        let module_arc = Arc::new(module);
        // FIX: Pass the Arc directly, removing the clone, as ModuleInstance::new now accepts Arc<Module>
        // Fix: Clone the Arc here to keep ownership for later use
        let mut instance = ModuleInstance::new(module_arc.clone())?; // Clone Arc here

        // Initialize memories
        // Need read access to module's memory definitions
        let module_memories = module_arc.memories.read().map_err(|_| {
            Error::new(kinds::PoisonedLockError(
                "Module memories lock poisoned".to_string(),
            ))
        })?;
        for memory_arc in module_memories.iter() {
            // Assuming DefaultMemory implements MemoryBehavior and can be cloned
            // or re-instantiated based on type/descriptor if needed.
            // Here, we clone the Arc and cast it. Adjust if DefaultMemory isn't Arc<T>.
            instance
                .memories
                .push(memory_arc.clone() as Arc<dyn MemoryBehavior>);
        }
        drop(module_memories); // Release read lock

        // TODO: Initialize tables similarly
        // let module_tables = module_arc.tables.read().map_err(|_| Error::new(kinds::PoisonedLock))?;
        // for table_arc in module_tables.iter() {
        //     instance.tables.push(table_arc.clone()); // Assuming Table can be cloned or needs Arc::new
        // }
        // drop(module_tables);

        // TODO: Initialize globals similarly
        // let module_globals = module_arc.globals.read().map_err(|_| Error::new(kinds::PoisonedLock))?;
        // for global_arc in module_globals.iter() {
        //     instance.globals.push(global_arc.clone()); // Assuming Global can be cloned or needs Arc::new
        // }
        // drop(module_globals);

        let instance_arc = Arc::new(instance); // Wrap the initialized instance in Arc

        // Lock the instances vector to push the new instance
        let mut instances_guard = self.instances.lock();
        let instance_idx = instances_guard.len();
        instances_guard.push(instance_arc); // Push first
        if let Some(inst_mut_arc) = instances_guard.get_mut(instance_idx) {
            if let Some(inst_mut) = Arc::get_mut(inst_mut_arc) {
                inst_mut.module_idx = instance_idx as u32; // Assign via mutable reference
            } else {
                return Err(Error::new(kinds::ExecutionError(
                    "Failed to get mutable access to newly added instance Arc".into(),
                )));
            }
        } else {
            return Err(Error::new(kinds::ExecutionError(
                "Failed to find newly added instance after push".into(),
            )));
        }

        // Drop the lock before potentially running the start function
        drop(instances_guard);

        // Execute start function if present
        if let Some(start_func_idx) = module_arc.start {
            // Need mutable access to the newly added instance
            let mut instances_guard_mut = self.instances.lock();
            if let Some(instance_mut_arc) = instances_guard_mut.get_mut(instance_idx) {
                if let Some(instance_mut) = Arc::get_mut(instance_mut_arc) {
                    // TODO: Implement start function execution properly.
                    // This might involve calling self.call_function or a dedicated method.
                    // instance_mut.execute_start_function(self, start_func_idx)?;
                    println!("Warning: Start function execution is not yet fully implemented in instantiate.");
                } else {
                    return Err(Error::new(kinds::ExecutionError(
                        "Failed to get mutable access to newly added instance for start function"
                            .into(),
                    )));
                }
            } else {
                return Err(Error::new(kinds::ExecutionError(
                    "Failed to find newly added instance for start function".into(),
                )));
            }
            // Drop the mutable lock
            drop(instances_guard_mut);
        }

        Ok(instance_idx)
    }

    /// Checks if the engine currently has any module instances loaded.
    ///
    /// # Returns
    ///
    /// `true` if there are no instances, `false` otherwise.
    pub fn has_no_instances(&self) -> bool {
        self.instances.lock().is_empty()
    }

    /// Registers a callback function for a specific export name.
    ///
    /// This allows host functions to be called from WebAssembly.
    pub fn register_callback(
        &mut self,
        export_name: &str,
        callback: HostFunctionHandler,
    ) -> Result<(), Error> {
        let mut registry = self.callbacks.lock();
        if registry.callbacks.contains_key(export_name) {
            return Err(Error::new(kinds::ExecutionError(
                format!("Callback already registered for export: {}", export_name).into(),
            )));
        }
        registry.callbacks.insert(export_name.to_string(), callback);
        Ok(())
    }

    /// Registers known exports that should trigger logging or other callbacks.
    ///
    /// # Arguments
    ///
    /// * `export_names`: A map where the key is the export name (e.g., "wasi:logging/logging.log")
    ///   and the value is another map specifying the log operation (e.g., {"log": LogOperation::Log}).
    pub fn register_known_exports(
        &mut self,
        export_names: HashMap<String, HashMap<String, LogOperation>>,
    ) -> Result<(), Error> {
        let mut registry = self.callbacks.lock();
        registry.export_names = export_names;
        Ok(())
    }

    /// Finds a callback function by export name.
    ///
    /// Requires a lock on the callback registry.
    fn find_callback_locked(
        registry: &StacklessCallbackRegistry,
        export_name: &str,
    ) -> Option<HostFunctionHandler> {
        registry.callbacks.get(export_name).cloned()
    }

    /// Calls an exported function by name
    pub fn call_export(&mut self, export_name: &str, args: &[Value]) -> Result<Vec<Value>, Error> {
        let instance_idx = self.exec_stack.instance_idx;
        let instances_guard = self.instances.lock();
        let instance_arc = instances_guard
            .get(instance_idx)
            .cloned()
            .ok_or_else(|| Error::new(kinds::InvalidInstanceIndexError(instance_idx)))?; // Cast to usize
        drop(instances_guard); // Release lock early

        let export = instance_arc
            .module
            .exports
            .iter()
            .find(|e| e.name == export_name)
            .ok_or_else(|| Error::new(kinds::ExportNotFoundError(export_name.to_string())))?;

        match export.kind {
            ExportKind::Function => {
                let func_idx = export.index;
                self.call_function(instance_idx as u32, func_idx, args)
            }
            _ => {
                Err(Error::new(kinds::ExportNotFoundError(format!(
                    "Export '{export_name}' is not a function (kind: {:?})",
                    export.kind
                )))) // Use tuple struct syntax
            }
        }
    }

    /// Calls a function by index within a specific instance
    pub fn call_function(
        &mut self,
        instance_idx: u32,
        func_idx: u32,
        args: &[Value],
    ) -> Result<Vec<Value>, Error> {
        // Fetch module Arc while holding the lock
        let module = {
            let instances_guard = self.instances.lock();
            instances_guard
                .get(instance_idx as usize)
                .cloned() // Clone the Arc<ModuleInstance>
                .ok_or_else(|| Error::new(kinds::InvalidInstanceIndexError(instance_idx as usize)))? // Cast to usize
                .module
                .clone()
        }; // Lock released here

        let export_name = module.exports.iter().find_map(|export| {
            if let ExportKind::Function = export.kind {
                if export.index == func_idx {
                    Some(export.name.clone())
                } else {
                    None
                }
            } else {
                None
            }
        });

        if let Some(name) = export_name.map(|s| s.to_string()) {
            let registry_lock = self.callbacks_lock();
            if let Some(callback) = Self::find_callback_locked(&registry_lock, &name) {
                trace!("DEBUG: Calling host callback: {}", name);
                drop(registry_lock);
                // TODO: Actually call the host function - requires plumbing HostFunc context/env
                // For now, return UnimplementedError correctly
                return Err(Error::new(kinds::UnimplementedError(
                    "Host function callback invocation".to_string(),
                )));
            }
        }

        let initial_frame = StacklessFrame::new(module.into(), func_idx, args, instance_idx)?;
        self.exec_stack.frames.push(initial_frame); // Use exec_stack
        self.exec_stack.state = StacklessExecutionState::Running; // Use exec_stack

        let result = self.run_loop();

        match result {
            Ok(StacklessExecutionState::Completed) => {
                // Access stack via self.exec_stack
                let current_frame = self.exec_stack.frames.last().ok_or_else(|| {
                    Error::new(kinds::ExecutionError(
                        "Frame stack empty after function completion".into(),
                    ))
                })?;
                let func_type = current_frame.get_function_type()?;
                let arity = func_type.results.len();

                if self.exec_stack.values.len() < arity {
                    return Err(Error::new(kinds::StackUnderflowError));
                }
                let results = self
                    .exec_stack
                    .values
                    .split_off(self.exec_stack.values.len() - arity);
                Ok(results)
            }
            Ok(state) => Err(Error::new(kinds::ExecutionError(
                format!("Execution finished in unexpected state: {:?}", state).into(),
            ))),
            Err(e) => Err(e),
        }
    }

    /// Runs the engine until it halts, traps, or requires external interaction.
    pub fn run(&mut self) -> Result<StacklessExecutionState, Error> {
        self.run_loop()
    }

    /// Executes a single step (instruction) in the engine.
    pub fn step(&mut self) -> Result<(), Error> {
        if self.exec_stack.frames.is_empty() {
            return Err(Error::new(kinds::ExecutionError(
                "No frames on the execution stack".to_string(),
            )));
        }

        let top_frame_idx = self.exec_stack.frames.len() - 1;

        // Get the instruction and increment PC
        let (instr, pc) = {
            let frame = &self.exec_stack.frames[top_frame_idx];
            let pc = frame.pc();
            let func = frame.get_function().map_err(|e| {
                Error::new(kinds::ExecutionError(format!(
                    "Failed to get function: {}",
                    e
                )))
            })?;

            // Check if we're at the end of the function
            if pc >= func.code.len() {
                return Err(Error::new(kinds::ExecutionError(
                    "Reached end of function without return".to_string(),
                )));
            }

            // Get the instruction
            let instr = func.code[pc].clone(); // Clone to avoid reference

            // Increment PC in a separate scope
            {
                let frame = &mut self.exec_stack.frames[top_frame_idx];
                frame.set_pc(pc + 1);
            }

            (instr, pc)
        };

        // Execute the instruction
        trace!("Executing instruction: {:?}", &instr);

        // Clone the instruction to avoid lifetime issues
        let instruction = instr.clone();

        // Clone the stack and engine to avoid borrowing self twice
        let mut stack_clone = self.exec_stack.clone();
        let mut engine_clone = self.clone();

        // Execute directly with a cloned frame
        let frame_idx = stack_clone.frames.len() - 1;
        let mut frame_clone = stack_clone.frames[frame_idx].clone();

        // Execute the instruction
        let result = instruction.execute(&mut stack_clone, &mut frame_clone, &mut engine_clone);

        // If successful, update our state
        if result.is_ok() {
            // Update the frame in the stack
            stack_clone.frames[frame_idx] = frame_clone;

            // Update the main stack
            self.exec_stack.frames = stack_clone.frames;
            self.exec_stack.values = stack_clone.values;
            self.exec_stack.labels = stack_clone.labels;

            // Update state if needed based on control flow
            match &result {
                Ok(ControlFlow::Return { values }) => {
                    self.exec_stack.state = StacklessExecutionState::Returning {
                        values: values.clone(),
                    };
                }
                Ok(ControlFlow::Call {
                    func_idx,
                    args,
                    return_pc,
                }) => {
                    // Use the values directly, no need to dereference
                    let func_idx_val = *func_idx;
                    let return_pc_val = *return_pc;

                    self.exec_stack.state = StacklessExecutionState::Calling {
                        instance_idx: stack_clone.instance_idx as u32,
                        func_idx: func_idx_val,
                        args: args.clone(),
                        return_pc: return_pc_val,
                    };
                }
                Ok(ControlFlow::Branch {
                    target_pc,
                    values_to_keep: _,
                }) => {
                    // Use the value directly, no need to dereference
                    let target_pc_val = *target_pc;

                    // Update PC for branches if needed
                    self.exec_stack.pc = target_pc_val;
                }
                _ => {}
            }
        }

        // Convert the result from Result<ControlFlow, Error> to Result<(), Error>
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// The main execution loop that drives the engine forward.
    /// This function is typically called internally by `run`.
    fn run_loop(&mut self) -> Result<StacklessExecutionState, Error> {
        loop {
            match self.exec_stack.state {
                StacklessExecutionState::Running => self.step()?,
                StacklessExecutionState::Completed | StacklessExecutionState::Finished => {
                    // Replace state with Completed and return the original
                    return Ok(mem::replace(
                        &mut self.exec_stack.state,
                        StacklessExecutionState::Completed,
                    ));
                }
                StacklessExecutionState::Paused { .. } => {
                    // Replace state with Completed and return the original (Paused state)
                    return Ok(mem::replace(
                        &mut self.exec_stack.state,
                        StacklessExecutionState::Completed,
                    ));
                }
                StacklessExecutionState::Error(_) => {
                    // Replace state with Completed and return the original (Error state)
                    return Ok(mem::replace(
                        &mut self.exec_stack.state,
                        StacklessExecutionState::Completed,
                    ));
                }
                // Other states like Calling, Returning, Branching are handled internally by step/run_loop
                _ => self.step()?, // Continue stepping if in an intermediate state
            }
        }
    }

    /// Returns an immutable reference to the current (top) execution frame.
    pub fn current_frame(&self) -> Result<&StacklessFrame, Error> {
        self.exec_stack
            .frames
            .last()
            .ok_or_else(|| Error::new(kinds::ExecutionError("Call stack empty".to_string())))
    }

    /// Returns a mutable reference to the current (top) execution frame.
    pub fn current_frame_mut(&mut self) -> Result<&mut StacklessFrame, Error> {
        self.exec_stack
            .frames
            .last_mut()
            .ok_or_else(|| Error::new(kinds::ExecutionError("Call stack empty".to_string())))
    }

    fn pop_n(&mut self, n: usize) -> Vec<Value> {
        if self.exec_stack.values.len() < n {
            // Log the error but return an empty vector
            log::error!("Error popping values from stack: stack underflow");
            Vec::new()
        } else {
            let new_len = self.exec_stack.values.len() - n;
            self.exec_stack.values.split_off(new_len)
        }
    }

    fn pop_frame_label(&self) -> Result<Label, Error> {
        if let Some(frame) = self.exec_stack.frames.last() {
            if let Some(label) = frame.label_stack.last() {
                return Ok(label.clone());
            }
        }
        Err(Error::new(kinds::StackUnderflowError))
    }

    /// Get the current instance being executed
    pub fn get_current_instance(&self) -> Result<Arc<ModuleInstance>, Error> {
        let frame = self.current_frame()?;
        self.with_instance(frame.instance_idx.try_into().unwrap(), |instance| {
            Ok(Arc::new(instance.clone()))
        })
    }

    pub fn callbacks_lock(&self) -> parking_lot::MutexGuard<'_, StacklessCallbackRegistry> {
        self.callbacks.lock()
    }

    /// Public accessor for the callbacks lock
    pub fn get_callbacks_lock(&self) -> parking_lot::MutexGuard<'_, StacklessCallbackRegistry> {
        self.callbacks.lock()
    }

    pub fn invoke_host_function(
        &mut self,
        _func_ref: u32,
        _instance_idx: usize,
        _args: Vec<Value>,
    ) -> Result<Vec<Value>> {
        Err(Error::new(kinds::UnimplementedError(
            "invoke_host_function".to_string(),
        )))
    }

    pub fn get_func_ref_from_table(
        &mut self,
        _table_idx: u32,
        _idx: u32,
        _instance_idx: usize,
    ) -> Result<u32> {
        Err(Error::new(kinds::UnimplementedError(
            "get_func_ref_from_table".to_string(),
        )))
    }

    /// Execute a single instruction
    pub fn execute_instruction(
        &mut self,
        stack: &mut StacklessStack,
        instruction: &InstructionType,
    ) -> Result<ControlFlow, Error> {
        if stack.frames.is_empty() {
            return Err(Error::new(kinds::ExecutionError(
                "No frames on stack".to_string(),
            )));
        }

        // Get the frame index
        let frame_idx = stack.frames.len() - 1;

        // Clone the frame and engine to avoid borrow issues
        let mut frame = stack.frames[frame_idx].clone();
        let mut engine_clone = self.clone();

        // Execute directly with the cloned frame
        let result = instruction.execute(stack, &mut frame, &mut engine_clone);

        // If successful, update the frame in the stack
        if result.is_ok() {
            stack.frames[frame_idx] = frame;

            // Update the engine state if needed
            match &result {
                Ok(ControlFlow::Return { values }) => {
                    // Handle return values if needed
                    self.exec_stack.state = StacklessExecutionState::Returning {
                        values: values.clone(),
                    };
                }
                Ok(ControlFlow::Call {
                    func_idx,
                    args,
                    return_pc,
                }) => {
                    // Use the values directly, no need to dereference
                    let func_idx_val = *func_idx;
                    let return_pc_val = *return_pc;

                    self.exec_stack.state = StacklessExecutionState::Calling {
                        instance_idx: stack.instance_idx as u32,
                        func_idx: func_idx_val,
                        args: args.clone(),
                        return_pc: return_pc_val,
                    };
                }
                Ok(ControlFlow::Branch {
                    target_pc,
                    values_to_keep: _,
                }) => {
                    // Use the value directly, no need to dereference
                    let target_pc_val = *target_pc;

                    // Update PC for branches if needed
                    self.exec_stack.pc = target_pc_val;
                }
                _ => {}
            }
        }

        result
    }

    /// Gets a copy of the current module being executed
    pub fn get_module_copy(&self) -> Result<Module> {
        // Get a reference to the module
        let instance_idx = self.exec_stack.instance_idx;

        self.with_instance(instance_idx, |instance| {
            // Clone the module - dereference the Arc to get a Module
            Ok((*instance.module).clone())
        })
    }
}

// Implement StackBehavior for StacklessEngine by delegating to exec_stack
impl StackBehavior for StacklessEngine {
    fn push(&mut self, value: Value) -> Result<(), Error> {
        self.exec_stack.push(value)
    }

    fn pop(&mut self) -> Result<Value, Error> {
        self.exec_stack.pop()
    }

    fn peek(&self) -> Result<&Value, Error> {
        self.exec_stack.peek()
    }

    fn peek_mut(&mut self) -> Result<&mut Value, Error> {
        self.exec_stack.peek_mut()
    }

    fn values(&self) -> &[Value] {
        self.exec_stack.values()
    }

    fn values_mut(&mut self) -> &mut [Value] {
        self.exec_stack.values_mut()
    }

    fn len(&self) -> usize {
        self.exec_stack.len()
    }

    fn is_empty(&self) -> bool {
        self.exec_stack.is_empty()
    }

    fn push_label(&mut self, label: Label) -> Result<(), Error> {
        self.exec_stack.push_label(label.arity, label.pc)
    }

    fn pop_label(&mut self) -> Result<Label, Error> {
        self.exec_stack.pop_label()
    }

    fn get_label(&self, index: usize) -> Option<&Label> {
        self.exec_stack.get_label(index)
    }

    fn push_n(&mut self, values: &[Value]) {
        self.exec_stack.values.extend_from_slice(values);
    }

    fn pop_n(&mut self, n: usize) -> Vec<Value> {
        if self.exec_stack.values.len() < n {
            // Log the error but return an empty vector
            log::error!("Error popping values from stack: stack underflow");
            Vec::new()
        } else {
            let new_len = self.exec_stack.values.len() - n;
            self.exec_stack.values.split_off(new_len)
        }
    }

    fn pop_frame_label(&mut self) -> Result<Label, Error> {
        self.exec_stack.pop_frame_label()
    }

    fn execute_function_call_direct(
        &mut self,
        _engine: &mut StacklessEngine, // Param required by trait, unused when self is engine
        caller_instance_idx: u32,
        func_idx: u32,
        args: Vec<Value>,
    ) -> Result<Vec<Value>, Error> {
        // This is a bit of a hack - we unwrap self since self is already the engine
        self.call_function(caller_instance_idx, func_idx, &args)
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// Replace the StackBehavior implementation for StacklessStack to avoid recursion
impl StackBehavior for StacklessStack {
    fn push(&mut self, value: Value) -> Result<(), Error> {
        if self.values.len() >= self.capacity {
            return Err(Error::new(kinds::ExecutionError(
                "Stack overflow".to_string(),
            )));
        }
        self.values.push(value);
        Ok(())
    }

    fn pop(&mut self) -> Result<Value, Error> {
        self.values
            .pop()
            .ok_or_else(|| Error::new(kinds::StackUnderflowError))
    }

    fn peek(&self) -> Result<&Value, Error> {
        self.values
            .last()
            .ok_or_else(|| Error::new(kinds::StackUnderflowError))
    }

    fn peek_mut(&mut self) -> Result<&mut Value, Error> {
        self.values
            .last_mut()
            .ok_or_else(|| Error::new(kinds::StackUnderflowError))
    }

    fn values(&self) -> &[Value] {
        &self.values
    }

    fn values_mut(&mut self) -> &mut [Value] {
        &mut self.values
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    fn push_label(&mut self, label: Label) -> Result<(), Error> {
        self.labels.push(label);
        Ok(())
    }

    fn pop_label(&mut self) -> Result<Label, Error> {
        self.labels
            .pop()
            .ok_or_else(|| Error::new(kinds::StackUnderflowError))
    }

    fn get_label(&self, index: usize) -> Option<&Label> {
        self.labels.get(index)
    }

    fn push_n(&mut self, values: &[Value]) {
        for value in values {
            let _ = self.push(value.clone());
        }
    }

    fn pop_n(&mut self, n: usize) -> Vec<Value> {
        if self.values.len() < n {
            log::error!("Error popping values from stack: stack underflow");
            Vec::new()
        } else {
            let new_len = self.values.len() - n;
            let mut result = self.values.split_off(new_len);
            result.reverse(); // maintain stack order
            result
        }
    }

    fn pop_frame_label(&mut self) -> Result<Label, Error> {
        if let Some(frame) = self.frames.last() {
            if let Some(label) = frame.label_stack.last() {
                return Ok(label.clone());
            }
        }
        Err(Error::new(kinds::StackUnderflowError))
    }

    fn execute_function_call_direct(
        &mut self,
        engine: &mut StacklessEngine,
        caller_instance_idx: u32,
        func_idx: u32,
        args: Vec<Value>,
    ) -> Result<Vec<Value>, Error> {
        engine.call_function(caller_instance_idx, func_idx, &args)
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// Implement Clone for StacklessEngine
impl Clone for StacklessEngine {
    fn clone(&self) -> Self {
        Self {
            exec_stack: self.exec_stack.clone(),
            fuel: self.fuel,
            stats: self.stats.clone(),
            callbacks: self.callbacks.clone(),
            max_call_depth: self.max_call_depth,
            instances: self.instances.clone(),
        }
    }
}

// Fix the Clone implementation for StacklessExecutionState in StacklessStack
impl Clone for StacklessStack {
    fn clone(&self) -> Self {
        Self {
            module: self.module.clone(),
            instance_idx: self.instance_idx,
            values: self.values.clone(),
            labels: self.labels.clone(),
            frames: self.frames.clone(),
            state: match &self.state {
                StacklessExecutionState::Running => StacklessExecutionState::Running,
                StacklessExecutionState::Paused {
                    pc,
                    instance_idx,
                    func_idx,
                    expected_results,
                } => StacklessExecutionState::Paused {
                    pc: *pc,
                    instance_idx: *instance_idx,
                    func_idx: *func_idx,
                    expected_results: *expected_results,
                },
                StacklessExecutionState::Calling {
                    instance_idx,
                    func_idx,
                    args,
                    return_pc,
                } => StacklessExecutionState::Calling {
                    instance_idx: *instance_idx,
                    func_idx: *func_idx,
                    args: args.clone(),
                    return_pc: *return_pc,
                },
                StacklessExecutionState::Returning { values } => {
                    StacklessExecutionState::Returning {
                        values: values.clone(),
                    }
                }
                StacklessExecutionState::Branching { depth, values } => {
                    StacklessExecutionState::Branching {
                        depth: *depth,
                        values: values.clone(),
                    }
                }
                StacklessExecutionState::Completed => StacklessExecutionState::Completed,
                StacklessExecutionState::Finished => StacklessExecutionState::Finished,
                StacklessExecutionState::Error(err) => StacklessExecutionState::Error(Error::new(
                    kinds::ExecutionError(err.to_string()),
                )),
            },
            pc: self.pc,
            func_idx: self.func_idx,
            capacity: self.capacity,
        }
    }
}
