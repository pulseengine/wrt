use crate::error::{Error, Result};
use crate::instructions::Instruction;
use crate::logging::{CallbackRegistry, LogLevel, LogOperation};
use crate::module::Module;
use crate::values::Value;
use crate::{format, String, Vec};

#[cfg(feature = "std")]
use std::sync::{Arc, Mutex};

#[cfg(not(feature = "std"))]
use crate::Mutex;
#[cfg(not(feature = "std"))]
use alloc::sync::Arc;

/// Represents the execution stack
#[derive(Debug)]
pub struct Stack {
    /// Values on the stack
    values: Vec<Value>,
    /// Labels (for control flow)
    labels: Vec<Label>,
    /// Function frames
    frames: Vec<Frame>,
}

/// Represents a label in the control stack
#[derive(Debug)]
pub struct Label {
    /// Number of values on the stack when this label was created
    pub arity: usize,
    /// Instruction to continue from
    pub continuation: usize,
}

/// Represents a function activation frame
#[derive(Debug)]
pub struct Frame {
    /// Function index
    pub func_idx: u32,
    /// Local variables
    pub locals: Vec<Value>,
    /// Module instance
    pub module: ModuleInstance,
}

/// Represents a module instance during execution
#[derive(Debug, Clone)]
pub struct ModuleInstance {
    /// Module index in the engine instances array
    module_idx: u32,
    /// Module definition
    module: Module,
    /// Function addresses
    func_addrs: Vec<FunctionAddr>,
    /// Table addresses
    table_addrs: Vec<TableAddr>,
    /// Memory addresses
    memory_addrs: Vec<MemoryAddr>,
    /// Global addresses
    global_addrs: Vec<GlobalAddr>,
}

/// Represents a function address
#[derive(Debug, Clone)]
struct FunctionAddr {
    /// Module instance index
    #[allow(dead_code)]
    instance_idx: u32,
    /// Function index
    #[allow(dead_code)]
    func_idx: u32,
}

/// Represents a table address
#[derive(Debug, Clone)]
struct TableAddr {
    /// Module instance index
    #[allow(dead_code)]
    instance_idx: u32,
    /// Table index
    #[allow(dead_code)]
    table_idx: u32,
}

/// Represents a memory address
#[derive(Debug, Clone)]
struct MemoryAddr {
    /// Module instance index
    #[allow(dead_code)]
    instance_idx: u32,
    /// Memory index
    #[allow(dead_code)]
    memory_idx: u32,
}

/// Represents a global address
#[derive(Debug, Clone)]
struct GlobalAddr {
    /// Module instance index
    #[allow(dead_code)]
    instance_idx: u32,
    /// Global index
    #[allow(dead_code)]
    global_idx: u32,
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl Stack {
    /// Creates a new empty stack
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            labels: Vec::new(),
            frames: Vec::new(),
        }
    }

    /// Pushes a value onto the stack
    pub fn push(&mut self, value: Value) {
        self.values.push(value);
    }

    /// Pops a value from the stack
    pub fn pop(&mut self) -> Result<Value> {
        self.values
            .pop()
            .ok_or_else(|| Error::Execution("Stack underflow".into()))
    }

    /// Pushes a label onto the control stack
    pub fn push_label(&mut self, arity: usize, continuation: usize) {
        self.labels.push(Label {
            arity,
            continuation,
        });
    }

    /// Pops a label from the control stack
    pub fn pop_label(&mut self) -> Result<Label> {
        self.labels
            .pop()
            .ok_or_else(|| Error::Execution("Label stack underflow".into()))
    }

    /// Gets a label at the specified depth without popping it
    pub fn get_label(&self, depth: u32) -> Result<&Label> {
        let idx = self
            .labels
            .len()
            .checked_sub(1 + depth as usize)
            .ok_or_else(|| Error::Execution(format!("Label depth {} out of bounds", depth)))?;
        self.labels
            .get(idx)
            .ok_or_else(|| Error::Execution(format!("Label at depth {} not found", depth)))
    }

    /// Pushes a frame onto the call stack
    pub fn push_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
    }

    /// Pops a frame from the call stack
    pub fn pop_frame(&mut self) -> Result<Frame> {
        self.frames
            .pop()
            .ok_or_else(|| Error::Execution("Call stack underflow".into()))
    }

    /// Returns the current frame
    pub fn current_frame(&self) -> Result<&Frame> {
        self.frames
            .last()
            .ok_or_else(|| Error::Execution("No active frame".into()))
    }
}

/// Execution state for resumable execution
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionState {
    /// Initial state, not executing
    Idle,
    /// Currently executing
    Running,
    /// Execution paused due to fuel exhaustion
    Paused {
        /// Instance index
        instance_idx: u32,
        /// Function index
        func_idx: u32,
        /// Program counter
        pc: usize,
        /// Expected return values count
        expected_results: usize,
    },
    /// Execution complete
    Finished,
}

/// Execution statistics for monitoring and reporting
#[derive(Debug, Clone, Default)]
pub struct ExecutionStats {
    /// Total number of instructions executed
    pub instructions_executed: u64,
    /// Total amount of fuel consumed
    pub fuel_consumed: u64,
    /// Peak memory usage in bytes
    pub peak_memory_bytes: usize,
    /// Current memory usage in bytes
    pub current_memory_bytes: usize,
    /// Number of function calls
    pub function_calls: u64,
    /// Number of memory operations
    pub memory_operations: u64,
}

/// The WebAssembly execution engine
#[derive(Debug)]
pub struct Engine {
    /// Execution stack
    stack: Stack,
    /// Module instances
    instances: Vec<ModuleInstance>,
    /// Remaining fuel for bounded execution
    fuel: Option<u64>,
    /// Current execution state
    state: ExecutionState,
    /// Execution statistics
    stats: ExecutionStats,
    /// Callback registry for host functions (logging, etc.)
    callbacks: Arc<Mutex<CallbackRegistry>>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::Instruction;
    use crate::module::Module;
    use crate::types::{FuncType, ValueType};
    use crate::values::Value;
    use crate::Vec;

    #[cfg(not(feature = "std"))]
    use alloc::vec;
    #[cfg(feature = "std")]
    use std::vec;

    #[test]
    fn test_fuel_bounded_execution() {
        // Create a simple module with a single function
        let mut module = Module::new();

        // Add a simple function type (no params, returns an i32)
        module.types.push(FuncType {
            params: vec![],
            results: vec![ValueType::I32],
        });

        // Add a function that executes a large number of instructions
        let mut instructions = Vec::new();
        for _ in 0..100 {
            instructions.push(Instruction::Nop);
        }
        // At the end, push a constant value as the result
        instructions.push(Instruction::I32Const(42));

        // Add the function to the module
        module.functions.push(crate::module::Function {
            type_idx: 0,
            locals: vec![],
            body: instructions,
        });

        // Create an engine with a fuel limit
        let mut engine = Engine::new();
        engine.instantiate(module).unwrap();

        // Test with unlimited fuel
        let result = engine.execute(0, 0, vec![]).unwrap();
        assert_eq!(result, vec![Value::I32(42)]);

        // Create a new module for the limited fuel test
        let mut limited_module = Module::new();

        // Add the same function type and instructions
        limited_module.types.push(FuncType {
            params: vec![],
            results: vec![ValueType::I32],
        });

        // Add a function that executes a large number of instructions
        let mut instructions = Vec::new();
        for _ in 0..100 {
            instructions.push(Instruction::Nop);
        }
        // At the end, push a constant value as the result
        instructions.push(Instruction::I32Const(42));

        // Add the function to the module
        limited_module.functions.push(crate::module::Function {
            type_idx: 0,
            locals: vec![],
            body: instructions,
        });

        // Reset the engine
        let mut engine = Engine::new();
        engine.instantiate(limited_module).unwrap();

        // Test with limited fuel
        engine.set_fuel(Some(10)); // Only enough for 10 instructions
        let result = engine.execute(0, 0, vec![]);

        // Should fail with FuelExhausted error
        assert!(matches!(result, Err(Error::FuelExhausted)));

        // Check the state
        assert!(matches!(engine.state(), ExecutionState::Paused { .. }));

        // Add more fuel and resume
        engine.set_fuel(Some(200)); // Plenty of fuel to finish
        let result = engine.resume().unwrap();

        // Should complete execution
        assert_eq!(result, vec![Value::I32(42)]);

        // Check the state
        assert_eq!(*engine.state(), ExecutionState::Finished);
    }
}

impl Engine {
    /// Creates a new execution engine
    pub fn new() -> Self {
        Self {
            stack: Stack::new(),
            instances: Vec::new(),
            fuel: None, // No fuel limit by default
            state: ExecutionState::Idle,
            stats: ExecutionStats::default(),
            callbacks: Arc::new(Mutex::new(CallbackRegistry::new())),
        }
    }

    /// Get the callback registry
    pub fn callbacks(&self) -> Arc<Mutex<CallbackRegistry>> {
        self.callbacks.clone()
    }

    /// Register a log handler
    pub fn register_log_handler<F>(&self, handler: F)
    where
        F: Fn(LogOperation) + Send + Sync + 'static,
    {
        if let Ok(mut callbacks) = self.callbacks.lock() {
            callbacks.register_log_handler(handler);
        }
    }

    /// Handle a log operation from a WebAssembly component
    pub fn handle_log(&self, level: LogLevel, message: String) {
        if let Ok(callbacks) = self.callbacks.lock() {
            if callbacks.has_log_handler() {
                let operation = LogOperation::new(level, message);
                callbacks.handle_log(operation);
            }
        }
    }

    /// Sets the fuel limit for bounded execution
    ///
    /// # Parameters
    ///
    /// * `fuel` - The amount of fuel to set, or None for unbounded execution
    pub fn set_fuel(&mut self, fuel: Option<u64>) {
        self.fuel = fuel;
    }

    /// Returns the current amount of remaining fuel
    ///
    /// # Returns
    ///
    /// The remaining fuel, or None if unbounded
    pub fn remaining_fuel(&self) -> Option<u64> {
        self.fuel
    }

    /// Returns the current execution state
    ///
    /// # Returns
    ///
    /// The current state of the engine
    pub fn state(&self) -> &ExecutionState {
        &self.state
    }

    /// Returns the current execution statistics
    ///
    /// # Returns
    ///
    /// Statistics about the execution including instruction count and memory usage
    pub fn stats(&self) -> &ExecutionStats {
        &self.stats
    }

    /// Resets the execution statistics
    pub fn reset_stats(&mut self) {
        self.stats = ExecutionStats::default();
    }

    /// Updates memory usage statistics for all memory instances
    fn update_memory_stats(&mut self) -> Result<()> {
        let mut total_memory = 0;

        // Sum up memory from all instances
        for instance in &self.instances {
            for _memory_addr in &instance.memory_addrs {
                // In a real implementation, we would get the actual memory size
                // For now, we'll estimate based on what we know

                // Assume each memory has at least 1 page (64 KB) and some may have grown
                let instance_memory = crate::memory::PAGE_SIZE; // Minimum 1 page (64KB)
                total_memory += instance_memory;
            }
        }

        // Track current memory usage and update peak if needed
        self.stats.current_memory_bytes = total_memory;
        if total_memory > self.stats.peak_memory_bytes {
            self.stats.peak_memory_bytes = total_memory;
        }

        Ok(())
    }

    /// Instantiates a module
    pub fn instantiate(&mut self, module: Module) -> Result<()> {
        // Validate the module
        module.validate()?;

        // Determine instance index
        let instance_idx = self.instances.len() as u32;

        // Create module instance
        let instance = ModuleInstance {
            module_idx: instance_idx,
            module,
            func_addrs: Vec::new(),
            table_addrs: Vec::new(),
            memory_addrs: Vec::new(),
            global_addrs: Vec::new(),
        };

        // Add instance to engine
        self.instances.push(instance);

        // Collect necessary data before modifying self.instances
        let function_count = self.instances[instance_idx as usize].module.functions.len();
        let table_count = self.instances[instance_idx as usize].module.tables.len();
        let memory_count = self.instances[instance_idx as usize].module.memories.len();
        let global_count = self.instances[instance_idx as usize].module.globals.len();

        // Initialize function addresses
        for idx in 0..function_count {
            self.instances[instance_idx as usize]
                .func_addrs
                .push(FunctionAddr {
                    instance_idx,
                    func_idx: idx as u32,
                });
        }

        // Initialize table addresses
        for idx in 0..table_count {
            self.instances[instance_idx as usize]
                .table_addrs
                .push(TableAddr {
                    instance_idx,
                    table_idx: idx as u32,
                });
        }

        // Initialize memory addresses
        for idx in 0..memory_count {
            self.instances[instance_idx as usize]
                .memory_addrs
                .push(MemoryAddr {
                    instance_idx,
                    memory_idx: idx as u32,
                });
        }

        // Initialize global addresses
        for idx in 0..global_count {
            self.instances[instance_idx as usize]
                .global_addrs
                .push(GlobalAddr {
                    instance_idx,
                    global_idx: idx as u32,
                });
        }

        Ok(())
    }

    /// Executes a function with fuel-bounded execution
    pub fn execute(
        &mut self,
        instance_idx: u32,
        func_idx: u32,
        args: Vec<Value>,
    ) -> Result<Vec<Value>> {
        // If we're starting a new execution, reset statistics
        if !matches!(self.state, ExecutionState::Paused { .. }) {
            self.reset_stats();
        }

        // Check if we're resuming a paused execution
        let start_pc = if let ExecutionState::Paused { pc, .. } = self.state {
            // We're resuming from a paused state
            pc
        } else {
            // We're starting a new execution
            self.state = ExecutionState::Running;

            // Clone the necessary information to avoid borrow issues
            let instance_clone;

            {
                // Scope to limit the borrow of self.instances
                let instance = &self.instances[instance_idx as usize];
                let func = &instance.module.functions[func_idx as usize];
                let func_type = &instance.module.types[func.type_idx as usize];

                // Check argument count
                if args.len() != func_type.params.len() {
                    return Err(Error::Execution(format!(
                        "Expected {} arguments, got {}",
                        func_type.params.len(),
                        args.len()
                    )));
                }

                // Clone the data we'll need outside this scope
                instance_clone = instance.clone();
            }

            // Create frame
            let mut frame = Frame {
                func_idx,
                locals: Vec::new(),
                module: instance_clone,
            };

            // Initialize locals with arguments
            frame.locals.extend(args);

            // Update function call count statistics
            self.stats.function_calls += 1;

            // Push frame
            self.stack.push_frame(frame);

            // Start from the beginning
            0
        };

        // Get the function clone
        let func_clone = {
            let instance = &self.instances[instance_idx as usize];
            instance.module.functions[func_idx as usize].clone()
        };

        // Get expected results count
        let expected_results = {
            let instance = &self.instances[instance_idx as usize];
            let func = &instance.module.functions[func_idx as usize];
            let func_type = &instance.module.types[func.type_idx as usize];
            func_type.results.len()
        };

        // Execute function body with fuel limitation
        let mut pc = start_pc;
        while pc < func_clone.body.len() {
            // Check if we have fuel
            if let Some(fuel) = self.fuel {
                if fuel == 0 {
                    // Out of fuel, pause execution
                    self.state = ExecutionState::Paused {
                        instance_idx,
                        func_idx,
                        pc,
                        expected_results,
                    };
                    return Err(Error::FuelExhausted);
                }

                // Fuel is consumed in execute_instruction based on instruction type
            }

            // Execute the instruction
            match self.execute_instruction(&func_clone.body[pc], pc) {
                Ok(Some(new_pc)) => pc = new_pc,
                Ok(None) => pc += 1,
                Err(e) => {
                    self.state = ExecutionState::Idle;
                    return Err(e);
                }
            }
        }

        // Pop frame
        self.stack.pop_frame()?;

        // Return results
        let mut results = Vec::new();
        for _ in 0..expected_results {
            results.push(self.stack.pop()?);
        }
        results.reverse();

        // Mark execution as finished
        self.state = ExecutionState::Finished;

        // Update memory usage statistics
        self.update_memory_stats()?;

        Ok(results)
    }

    /// Resumes a paused execution
    ///
    /// # Returns
    ///
    /// The results of the function call if execution completes, or an error if out of fuel again
    pub fn resume(&mut self) -> Result<Vec<Value>> {
        if let ExecutionState::Paused {
            instance_idx,
            func_idx,
            ..
        } = self.state.clone()
        {
            // Resume execution with empty args since we're already set up
            self.execute(instance_idx, func_idx, Vec::new())
        } else {
            Err(Error::Execution(
                "Cannot resume: not in paused state".into(),
            ))
        }
    }

    /// Calculates the fuel cost for a given instruction
    fn instruction_cost(&self, inst: &Instruction) -> u64 {
        match inst {
            // Control instructions - more expensive
            Instruction::Call(_) => 10,
            Instruction::CallIndirect(_, _) => 15,
            Instruction::ReturnCall(_) => 10,
            Instruction::ReturnCallIndirect(_, _) => 15,
            Instruction::Return => 5,
            Instruction::Br(_) | Instruction::BrIf(_) | Instruction::BrTable(_, _) => 4,
            Instruction::If(_) => 3,
            Instruction::Block(_) | Instruction::Loop(_) => 2,

            // Memory instructions - more expensive
            Instruction::I32Load(_, _)
            | Instruction::I64Load(_, _)
            | Instruction::F32Load(_, _)
            | Instruction::F64Load(_, _)
            | Instruction::I32Load8S(_, _)
            | Instruction::I32Load8U(_, _)
            | Instruction::I32Load16S(_, _)
            | Instruction::I32Load16U(_, _)
            | Instruction::I64Load8S(_, _)
            | Instruction::I64Load8U(_, _)
            | Instruction::I64Load16S(_, _)
            | Instruction::I64Load16U(_, _)
            | Instruction::I64Load32S(_, _)
            | Instruction::I64Load32U(_, _) => 8,

            Instruction::I32Store(_, _)
            | Instruction::I64Store(_, _)
            | Instruction::F32Store(_, _)
            | Instruction::F64Store(_, _)
            | Instruction::I32Store8(_, _)
            | Instruction::I32Store16(_, _)
            | Instruction::I64Store8(_, _)
            | Instruction::I64Store16(_, _)
            | Instruction::I64Store32(_, _) => 8,

            Instruction::MemoryGrow => 20,
            Instruction::MemorySize => 3,
            Instruction::MemoryFill => 10,
            Instruction::MemoryCopy => 10,
            Instruction::MemoryInit(_) => 10,
            Instruction::DataDrop(_) => 5,

            // Table instructions
            Instruction::TableGet(_) | Instruction::TableSet(_) => 3,
            Instruction::TableSize(_) => 3,
            Instruction::TableGrow(_) => 10,
            Instruction::TableFill(_) => 8,
            Instruction::TableCopy(_, _) => 8,
            Instruction::TableInit(_, _) => 8,
            Instruction::ElemDrop(_) => 3,

            // Basic instructions - cheaper
            Instruction::I32Const(_)
            | Instruction::I64Const(_)
            | Instruction::F32Const(_)
            | Instruction::F64Const(_) => 1,
            Instruction::Nop => 1,
            Instruction::Drop => 1,
            Instruction::Select | Instruction::SelectTyped(_) => 2,
            Instruction::LocalGet(_) | Instruction::LocalSet(_) | Instruction::LocalTee(_) => 2,
            Instruction::GlobalGet(_) | Instruction::GlobalSet(_) => 3,

            // Numeric instructions - medium cost
            Instruction::I32Eqz | Instruction::I64Eqz => 2,

            // Comparison operations
            Instruction::I32Eq
            | Instruction::I32Ne
            | Instruction::I32LtS
            | Instruction::I32LtU
            | Instruction::I32GtS
            | Instruction::I32GtU
            | Instruction::I32LeS
            | Instruction::I32LeU
            | Instruction::I32GeS
            | Instruction::I32GeU
            | Instruction::I64Eq
            | Instruction::I64Ne
            | Instruction::I64LtS
            | Instruction::I64LtU
            | Instruction::I64GtS
            | Instruction::I64GtU
            | Instruction::I64LeS
            | Instruction::I64LeU
            | Instruction::I64GeS
            | Instruction::I64GeU
            | Instruction::F32Eq
            | Instruction::F32Ne
            | Instruction::F32Lt
            | Instruction::F32Gt
            | Instruction::F32Le
            | Instruction::F32Ge
            | Instruction::F64Eq
            | Instruction::F64Ne
            | Instruction::F64Lt
            | Instruction::F64Gt
            | Instruction::F64Le
            | Instruction::F64Ge => 2,

            // Default for other instructions
            _ => 1,
        }
    }

    /// Executes a single instruction
    fn execute_instruction(&mut self, inst: &Instruction, pc: usize) -> Result<Option<usize>> {
        // Increment instruction count
        self.stats.instructions_executed += 1;

        // Track memory operations
        match inst {
            // Memory operations
            Instruction::I32Load(_, _)
            | Instruction::I64Load(_, _)
            | Instruction::F32Load(_, _)
            | Instruction::F64Load(_, _)
            | Instruction::I32Load8S(_, _)
            | Instruction::I32Load8U(_, _)
            | Instruction::I32Load16S(_, _)
            | Instruction::I32Load16U(_, _)
            | Instruction::I64Load8S(_, _)
            | Instruction::I64Load8U(_, _)
            | Instruction::I64Load16S(_, _)
            | Instruction::I64Load16U(_, _)
            | Instruction::I64Load32S(_, _)
            | Instruction::I64Load32U(_, _)
            | Instruction::I32Store(_, _)
            | Instruction::I64Store(_, _)
            | Instruction::F32Store(_, _)
            | Instruction::F64Store(_, _)
            | Instruction::I32Store8(_, _)
            | Instruction::I32Store16(_, _)
            | Instruction::I64Store8(_, _)
            | Instruction::I64Store16(_, _)
            | Instruction::I64Store32(_, _)
            | Instruction::MemoryGrow
            | Instruction::MemorySize
            | Instruction::MemoryFill
            | Instruction::MemoryCopy
            | Instruction::MemoryInit(_)
            | Instruction::DataDrop(_) => {
                self.stats.memory_operations += 1;
            }
            // Function calls
            Instruction::Call(_)
            | Instruction::CallIndirect(_, _)
            | Instruction::ReturnCall(_)
            | Instruction::ReturnCallIndirect(_, _) => {
                self.stats.function_calls += 1;
            }
            _ => {}
        }

        // Consume instruction-specific fuel amount if needed
        if let Some(fuel) = self.fuel {
            let cost = self.instruction_cost(inst);
            if fuel < cost {
                // Not enough fuel for this instruction
                self.fuel = Some(0); // Set to 0 to trigger out-of-fuel error on next check
            } else {
                self.fuel = Some(fuel - cost);
                // Track fuel consumption
                self.stats.fuel_consumed += cost;
            }
        }

        match inst {
            // Control instructions
            Instruction::Unreachable => {
                Err(Error::Execution("Unreachable instruction executed".into()))
            }
            Instruction::Nop => Ok(None),
            Instruction::Block(_block_type) => {
                self.stack.push_label(0, pc + 1);
                Ok(None)
            }
            Instruction::Loop(_block_type) => {
                self.stack.push_label(0, pc);
                Ok(None)
            }
            Instruction::If(_block_type) => {
                let cond = self
                    .stack
                    .pop()?
                    .as_i32()
                    .ok_or_else(|| Error::Execution("Expected i32 condition".into()))?;
                if cond != 0 {
                    self.stack.push_label(0, pc + 1);
                    Ok(None)
                } else {
                    Ok(Some(pc + 2))
                }
            }
            Instruction::Else => {
                let label = self.stack.pop_label()?;
                self.stack.push_label(label.arity, pc + 1);
                Ok(None)
            }
            Instruction::End => {
                let _label = self.stack.pop_label()?;
                Ok(None)
            }
            Instruction::Br(depth) => {
                let label = self.stack.get_label(*depth)?;
                Ok(Some(label.continuation))
            }
            Instruction::BrIf(depth) => {
                let cond = self
                    .stack
                    .pop()?
                    .as_i32()
                    .ok_or_else(|| Error::Execution("Expected i32 condition".into()))?;
                if cond != 0 {
                    let label = self.stack.get_label(*depth)?;
                    Ok(Some(label.continuation))
                } else {
                    Ok(None)
                }
            }
            Instruction::Return => {
                let frame = self.stack.current_frame()?;
                let func = &frame.module.module.functions[frame.func_idx as usize];
                let func_type = &frame.module.module.types[func.type_idx as usize];
                let mut results = Vec::new();
                for _ in 0..func_type.results.len() {
                    results.push(self.stack.pop()?);
                }
                results.reverse();
                self.stack.pop_frame()?;
                for result in results {
                    self.stack.push(result);
                }
                Ok(None)
            }
            Instruction::Call(func_idx) => {
                // Get information we need from the current frame
                let frame = self.stack.current_frame()?;
                let local_func_idx = *func_idx;
                let func = &frame.module.module.functions[local_func_idx as usize];
                let func_type = &frame.module.module.types[func.type_idx as usize];
                let params_len = func_type.params.len();
                let module_idx = frame.module.module_idx;

                // End the immutable borrow of the frame before mutable operations
                let _ = frame;

                // Get function arguments
                let mut args = Vec::new();
                for _ in 0..params_len {
                    args.push(self.stack.pop()?);
                }
                args.reverse();

                // Execute the function and push results
                let results = self.execute(module_idx, local_func_idx, args)?;
                for result in results {
                    self.stack.push(result);
                }

                Ok(None)
            }

            // Numeric constants
            Instruction::I32Const(value) => {
                self.stack.push(Value::I32(*value));
                Ok(None)
            }
            Instruction::I64Const(value) => {
                self.stack.push(Value::I64(*value));
                Ok(None)
            }
            Instruction::F32Const(value) => {
                self.stack.push(Value::F32(*value));
                Ok(None)
            }
            Instruction::F64Const(value) => {
                self.stack.push(Value::F64(*value));
                Ok(None)
            }

            // Variable access
            Instruction::LocalGet(idx) => {
                let frame = self.stack.current_frame()?;
                let local = frame
                    .locals
                    .get(*idx as usize)
                    .ok_or_else(|| Error::Execution(format!("Local {} not found", idx)))?
                    .clone();
                self.stack.push(local);
                Ok(None)
            }
            Instruction::LocalSet(idx) => {
                let value = self.stack.pop()?;
                let frame = self.stack.current_frame()?;
                let idx = *idx as usize;
                if idx >= frame.locals.len() {
                    return Err(Error::Execution(format!("Local {} out of bounds", idx)));
                }
                // Can't borrow mutably while borrowing immutably, so we need to drop the frame ref
                let _ = frame;

                // Now get a mutable reference to the current frame
                if let Some(frame) = self.stack.frames.last_mut() {
                    frame.locals[idx] = value;
                } else {
                    return Err(Error::Execution("No active frame for local set".into()));
                }
                Ok(None)
            }

            // ... implement other instructions ...
            _ => Err(Error::Execution("Instruction not implemented".into())),
        }
    }
}
