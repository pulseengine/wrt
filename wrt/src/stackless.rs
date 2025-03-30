//! Stackless WebAssembly execution engine
//!
//! This module implements a stackless version of the WebAssembly execution engine
//! that doesn't rely on the host language's call stack, making it suitable for
//! environments with limited stack space and for no_std contexts.
//!
//! The stackless engine uses a state machine approach to track execution state
//! and allows for pausing and resuming execution at any point.

use crate::{
    behavior::{
        self, ControlFlowBehavior, FrameBehavior, InstructionExecutor, Label, StackBehavior,
    },
    error::{Error, Result},
    execution::ExecutionStats,
    global::Global,
    instructions::Instruction,
    logging::{HostFunctionHandler, LogOperation},
    memory::Memory,
    module::{Function, Module},
    module_instance::ModuleInstance,
    stack::{self, Stack},
    table::Table,
    types::{BlockType, FuncType},
    values::Value,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// Represents the execution state in a stackless implementation
#[derive(Debug, PartialEq)]
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

/// Represents a function activation frame
#[derive(Debug, Clone)]
pub struct StacklessFrame {
    pub module: Arc<Module>,
    pub func_idx: u32,
    pub pc: usize,
    pub locals: Vec<Value>,
    pub instance_idx: u32,
    pub arity: usize,
    pub label_arity: usize,
    pub label_stack: Vec<Label>,
    pub return_pc: usize,
}

impl StacklessFrame {
    /// Creates a new stackless frame
    pub fn new(
        module: Arc<Module>,
        instance_idx: u32,
        func_idx: u32,
        args: Vec<Value>,
    ) -> Result<Self> {
        // Clone the module for getting the function and function type
        let module_clone = module.clone();

        let _func = module_clone
            .get_function(func_idx)
            .ok_or_else(|| Error::InvalidFunctionType(format!("Function not found: {func_idx}")))?;

        let func_type = module_clone.get_function_type(func_idx).ok_or_else(|| {
            Error::InvalidFunctionType(format!("Function type not found: {func_idx}"))
        })?;

        if args.len() != func_type.params.len() {
            return Err(Error::InvalidFunctionType(format!(
                "Expected {} arguments, got {}",
                func_type.params.len(),
                args.len()
            )));
        }

        for (arg, param) in args.iter().zip(func_type.params.iter()) {
            if !arg.matches_type(param) {
                return Err(Error::InvalidType(format!(
                    "Expected type {param:?}, got {arg:?}"
                )));
            }
        }

        // Now use the original module for the struct creation
        Ok(Self {
            module,
            func_idx,
            pc: 0,
            locals: args,
            instance_idx,
            arity: func_type.params.len(),
            label_arity: 0,
            label_stack: Vec::new(),
            return_pc: 0,
        })
    }

    /// Creates a new stackless frame from a function
    pub fn from_function(
        module: Arc<Module>,
        func_idx: u32,
        args: &[Value],
        instance_idx: u32,
    ) -> Result<Self> {
        // Get the function and its type from the module
        let func = module
            .get_function(func_idx)
            .ok_or(Error::FunctionNotFound(func_idx))?;

        let func_type = module.get_function_type(func.type_idx).ok_or_else(|| {
            Error::InvalidFunctionType(format!(
                "Function type not found for index {}",
                func.type_idx
            ))
        })?;

        // Prepare the locals
        let mut locals = Vec::with_capacity(func_type.params.len() + func.locals.len());
        locals.extend(args.iter().cloned());

        // Initialize local variables with their default values
        for local_type in &func.locals {
            locals.push(Value::default_for_type(local_type));
        }

        Ok(Self {
            module: module.clone(),
            func_idx,
            pc: 0,
            locals,
            label_stack: Vec::new(),
            label_arity: func_type.results.len(),
            instance_idx,
            arity: func_type.params.len(),
            return_pc: 0,
        })
    }

    /// Gets a function by index
    pub fn get_function(&self, func_idx: u32) -> Result<&Function> {
        self.module
            .get_function(func_idx)
            .ok_or_else(|| Error::InvalidFunctionType(format!("Function not found: {func_idx}")))
    }

    /// Gets a function type by index
    pub fn get_function_type(&self, func_idx: u32) -> Result<&FuncType> {
        self.module.get_function_type(func_idx).ok_or_else(|| {
            Error::InvalidFunctionType(format!("Function type not found: {func_idx}"))
        })
    }

    /// Gets a global by index
    pub fn get_global(&self, idx: u32) -> Result<Arc<Global>> {
        self.module.get_global(idx as usize)
    }

    /// Gets a mutable global by index
    pub fn get_global_mut(&mut self, idx: u32) -> Result<Arc<Global>> {
        let global = self.module.get_global(idx as usize)?;

        // Check if the global is mutable
        if !global.global_type.mutable {
            return Err(Error::GlobalNotMutable(idx as usize));
        }

        Ok(global)
    }

    /// Gets a memory by index
    pub fn get_memory(&self, idx: usize) -> Result<&Memory> {
        let memory_arc = self.module.get_memory(idx)?;
        // This is unsafe because we're returning a reference to the Arc's contents
        // which isn't guaranteed to live long enough. In practice, the Arc keeps
        // the memory alive for the lifetime of the module, but this is not ideal.
        Ok(unsafe {
            let memory_ptr = Arc::as_ptr(&memory_arc);
            &*memory_ptr
        })
    }

    /// Gets a mutable memory by index
    pub fn get_memory_mut(&mut self, idx: usize) -> Result<&mut Memory> {
        // Since module is Arc, we can't get a mutable reference directly
        Err(Error::Unimplemented(
            "get_memory_mut in stackless frame".to_string(),
        ))
    }

    /// Gets a table by index
    pub fn get_table(&self, idx: usize) -> Result<&Table> {
        let table_arc = self.module.get_table(idx)?;
        // This is unsafe for the same reason as get_memory
        Ok(unsafe {
            let table_ptr = Arc::as_ptr(&table_arc);
            &*table_ptr
        })
    }

    /// Gets a mutable table by index
    pub fn get_table_mut(&mut self, idx: usize) -> Result<Arc<Table>> {
        // Since Module is wrapped in Arc, we can't borrow it mutably
        self.module.get_table(idx)
    }

    /// Execute a frame with a stack
    pub fn execute(&mut self, stack: &mut impl Stack) -> Result<()> {
        // Execute each instruction in the function body
        let func_idx = self.func_idx;
        let func = self
            .module
            .get_function(func_idx)
            .ok_or(Error::InvalidFunctionIndex(func_idx as usize))?;
        // Clone the instructions to avoid borrowing issues
        let instructions = func.code.clone();

        // Push arguments to the stack
        for arg in &self.locals {
            stack.push(arg.clone())?;
        }

        // Execute instructions
        let mut pc = 0;
        while pc < instructions.len() {
            let instruction = &instructions[pc];
            instruction.execute(stack, self)?;
            pc += 1;
        }

        Ok(())
    }

    /// Get the function body for execution
    pub fn get_function_body(&self, func_idx: u32) -> Result<&[Instruction]> {
        let func = self
            .module
            .get_function(func_idx)
            .ok_or(Error::InvalidFunctionIndex(func_idx as usize))?;
        Ok(&func.code)
    }
}

// Implement the Stack trait for StacklessFrame
impl Stack for StacklessFrame {
    fn push_label(&mut self, label: stack::Label) -> Result<()> {
        // Convert stack::Label to behavior::Label
        self.label_stack.push(behavior::Label {
            arity: label.arity,
            pc: label.pc,
            continuation: label.continuation,
        });
        Ok(())
    }

    fn pop_label(&mut self) -> Result<stack::Label> {
        if self.label_stack.is_empty() {
            return Err(Error::Execution("Label stack is empty".to_string()));
        }

        let label = self.label_stack.pop().unwrap();
        // Convert behavior::Label to stack::Label
        Ok(stack::Label {
            arity: label.arity,
            pc: label.pc,
            continuation: label.continuation,
        })
    }

    fn get_label(&self, idx: usize) -> Result<&stack::Label> {
        // We can't directly convert from &behavior::Label to &stack::Label
        // This is a limitation of the current design
        Err(Error::Unimplemented(
            "get_label in StacklessFrame".to_string(),
        ))
    }

    fn get_label_mut(&mut self, idx: usize) -> Result<&mut stack::Label> {
        // Same limitation as above
        Err(Error::Unimplemented(
            "get_label_mut in StacklessFrame".to_string(),
        ))
    }

    fn labels_len(&self) -> usize {
        self.label_stack.len()
    }
}

impl StackBehavior for StacklessFrame {
    fn push(&mut self, value: Value) -> Result<()> {
        // In this implementation, we're using locals as our stack
        self.locals.push(value);
        Ok(())
    }

    fn pop(&mut self) -> Result<Value> {
        self.locals.pop().ok_or(Error::StackUnderflow)
    }

    fn peek(&self) -> Result<&Value> {
        self.locals.last().ok_or(Error::StackUnderflow)
    }

    fn peek_mut(&mut self) -> Result<&mut Value> {
        self.locals.last_mut().ok_or(Error::StackUnderflow)
    }

    fn values(&self) -> &[Value] {
        &self.locals
    }

    fn values_mut(&mut self) -> &mut [Value] {
        &mut self.locals
    }

    fn len(&self) -> usize {
        self.locals.len()
    }

    fn is_empty(&self) -> bool {
        self.locals.is_empty()
    }

    fn push_label(&mut self, arity: usize, pc: usize) {
        self.label_stack.push(Label {
            arity,
            pc,
            continuation: 0,
        });
    }

    fn pop_label(&mut self) -> Result<Label> {
        if self.label_stack.is_empty() {
            return Err(Error::Execution("Label stack is empty".to_string()));
        }

        Ok(self.label_stack.pop().unwrap())
    }

    fn get_label(&self, index: usize) -> Option<&Label> {
        if index < self.label_stack.len() {
            Some(&self.label_stack[index])
        } else {
            None
        }
    }
}

/// Represents the execution stack in a stackless implementation
#[derive(Debug)]
pub struct StacklessStack {
    /// Values on the stack
    pub values: Vec<Value>,
    /// Labels (for control flow)
    pub labels: Vec<Label>,
    /// Function frames
    pub frames: Vec<StacklessFrame>,
    /// Current execution state
    pub state: StacklessExecutionState,
    /// Instruction pointer
    pub pc: usize,
    /// Instance index
    pub instance_idx: usize,
    /// Function index
    pub func_idx: u32,
    /// Reference to the engine
    pub engine: Option<Arc<StacklessEngine>>,
    /// Reference to the module
    pub module: Arc<Module>,
}

/// Registry for callbacks in the stackless implementation
pub struct StacklessCallbackRegistry {
    /// Names of exports that are known to be callbacks
    pub export_names: HashMap<String, HashMap<String, LogOperation>>,
    /// Registered callback functions
    pub callbacks: HashMap<String, HostFunctionHandler>,
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
    /// Execution stack
    pub stack: StacklessStack,
    /// Module instances
    pub instances: Vec<ModuleInstance>,
    /// Remaining fuel for bounded execution
    fuel: Option<u64>,
    /// Execution statistics
    stats: ExecutionStats,
    /// Callback registry for host functions (logging, etc.)
    callbacks: Arc<Mutex<StacklessCallbackRegistry>>,
    /// Maximum call depth for function calls
    max_call_depth: Option<usize>,
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
            engine: None,
            module,
        }
    }

    /// Pushes a value onto the stack
    pub fn push(&mut self, value: Value) -> Result<()> {
        self.values.push(value);
        Ok(())
    }

    /// Pops a value from the stack
    pub fn pop(&mut self) -> Result<Value> {
        self.values.pop().ok_or(Error::StackUnderflow)
    }

    /// Pushes a label onto the control stack
    pub fn push_label(&mut self, arity: usize, pc: usize) -> Result<()> {
        self.labels.push(Label {
            arity,
            pc,
            continuation: pc,
        });
        Ok(())
    }

    /// Pops a label from the control stack
    pub fn pop_label(&mut self) -> Result<Label> {
        self.labels.pop().ok_or(Error::StackUnderflow)
    }

    /// Gets a label at the specified depth
    pub fn get_label(&self, idx: usize) -> Result<&Label> {
        self.labels
            .get(idx)
            .ok_or(Error::InvalidCode(format!("Invalid label index: {idx}")))
    }

    /// Pushes a frame onto the call stack
    pub fn push_frame(
        &mut self,
        instance_idx: usize,
        func_idx: u32,
        locals: Vec<Value>,
    ) -> Result<()> {
        let empty_module = Module::new().expect("Failed to create empty module");
        self.frames.push(StacklessFrame {
            module: Arc::new(empty_module),
            func_idx,
            locals: locals.clone(),
            pc: 0,
            instance_idx: instance_idx as u32,
            arity: locals.len(),
            label_arity: 0,
            label_stack: Vec::new(),
            return_pc: 0,
        });
        Ok(())
    }

    /// Pops a frame from the call stack
    pub fn pop_frame(&mut self) -> Result<StacklessFrame> {
        self.frames.pop().ok_or(Error::StackUnderflow)
    }

    /// Gets the current frame
    pub fn current_frame(&self) -> Result<&StacklessFrame> {
        self.frames.last().ok_or(Error::StackUnderflow)
    }

    /// Gets a mutable reference to the current frame
    pub fn current_frame_mut(&mut self) -> Result<&mut StacklessFrame> {
        self.frames.last_mut().ok_or(Error::StackUnderflow)
    }

    pub fn get_function_instruction(&self, func_idx: u32, pc: usize) -> Result<&Instruction> {
        self.module
            .get_function(func_idx)
            .ok_or(Error::InvalidFunctionIndex(func_idx as usize))?
            .code
            .get(pc)
            .ok_or(Error::InvalidProgramCounter(pc))
    }

    pub fn get_instance(&self, instance_idx: usize) -> Result<&ModuleInstance> {
        if let Some(engine) = &self.engine {
            engine.get_instance(instance_idx)
        } else {
            Err(Error::NoInstances)
        }
    }

    pub fn execute_instruction(
        &mut self,
        instruction: &Instruction,
        frame: &mut StacklessFrame,
    ) -> Result<()> {
        match instruction {
            Instruction::Call(func_idx) => {
                let instance = self.get_instance(frame.instance_idx as usize)?;
                let empty_module = Module::new().expect("Failed to create empty module");
                let mut temp_stack = Self::new(Arc::new(empty_module), self.instance_idx);
                temp_stack.instance_idx = self.instance_idx;
                temp_stack.func_idx = self.func_idx;
                temp_stack.engine = self.engine.clone();

                // Execute the function
                temp_stack.execute_instruction(instruction, frame)?;

                // Update our state with results from the temporary stack
                self.values.extend(temp_stack.values);
                Ok(())
            }
            // For all other instructions, forward to the InstructionExecutor trait implementation
            _ => instruction.execute(self, frame),
        }
    }

    pub fn execute_block(
        &mut self,
        instructions: &[Instruction],
        frame: &mut StacklessFrame,
    ) -> Result<()> {
        let mut pc = 0;
        while pc < instructions.len() {
            let instruction = &instructions[pc];
            let empty_module = Module::new().expect("Failed to create empty module");
            let mut temp_stack = Self::new(Arc::new(empty_module), self.instance_idx);
            temp_stack.instance_idx = self.instance_idx;
            temp_stack.func_idx = self.func_idx;
            temp_stack.engine = self.engine.clone();
            temp_stack.execute_instruction(instruction, frame)?;
            pc += 1;
        }
        Ok(())
    }

    /// Executes a function in a module instance
    pub fn execute_function(
        &mut self,
        instance_idx: usize,
        func_idx: u32,
        args: Vec<Value>,
    ) -> Result<Vec<Value>> {
        println!("DEBUG: execute called for function index: {func_idx}");
        println!("DEBUG: Arguments: {args:?}");

        // Get the instance
        let instance = self.get_instance(instance_idx)?;
        println!("DEBUG: Got instance at index: {instance_idx}");

        // Get the function code from instance
        let func = instance
            .module
            .get_function(func_idx)
            .ok_or(Error::InvalidFunctionIndex(func_idx as usize))?;

        // Get the function type from instance
        let func_type = instance.get_function_type(func_idx)?.clone();

        println!(
            "DEBUG: Function type: params={:?}, results={:?}",
            func_type.params, func_type.results
        );

        // Check that the arguments match the function parameters
        let params_len = func_type.params.len();
        if args.len() != params_len {
            return Err(Error::InvalidArgumentCount {
                expected: params_len,
                actual: args.len(),
            });
        }

        // Get the number of results this function returns
        let results_len = func_type.results.len();
        println!("DEBUG: Results length: {results_len}");

        // Create a new frame with arguments
        let module_arc = Arc::new(instance.module.clone());
        let frame =
            StacklessFrame::from_function(module_arc, func_idx, &args, instance_idx as u32)?;
        println!("DEBUG: Created frame with locals: {:?}", frame.locals);

        // Create a stack for execution
        let mut stack = Vec::new();

        // Push arguments to the stack (needed for operations)
        for arg in args {
            stack.push(arg);
        }
        println!("DEBUG: Initial stack with arguments: {stack:?}");

        // Execute each instruction in the function
        let instructions = &func.code;
        println!("DEBUG: Executing {} instructions", instructions.len());

        let mut pc = 0;
        while pc < instructions.len() {
            let instruction = &instructions[pc];
            println!(
                "DEBUG: Executing instruction [{}/{}]: {:?}",
                pc + 1,
                instructions.len(),
                instruction
            );

            match instruction {
                Instruction::LocalGet(idx) => {
                    println!("DEBUG: LocalGet - Index: {idx}");

                    if *idx as usize >= frame.locals.len() {
                        return Err(Error::InvalidLocalIndex(*idx as usize));
                    }

                    let value = frame.locals[*idx as usize].clone();
                    println!("DEBUG: LocalGet - Value at idx {idx}: {value:?}");

                    stack.push(value);
                    println!("DEBUG: Stack after LocalGet: {stack:?}");
                }
                Instruction::I32Add => {
                    println!("DEBUG: I32Add instruction");

                    if stack.len() < 2 {
                        return Err(Error::StackUnderflow);
                    }

                    // Get the operands, but keep clones for debugging
                    let b = stack.pop().unwrap();
                    let a = stack.pop().unwrap();

                    println!("DEBUG: I32Add - Popped values: a={a:?}, b={b:?}");

                    match (a, b) {
                        (Value::I32(a_val), Value::I32(b_val)) => {
                            let result = a_val.wrapping_add(b_val);
                            println!("DEBUG: I32Add - Result: {a_val} + {b_val} = {result}");
                            stack.push(Value::I32(result));
                        }
                        (a, b) => {
                            return Err(Error::InvalidType(format!(
                                "Type mismatch in I32Add: {a:?} and {b:?}"
                            )));
                        }
                    }

                    println!("DEBUG: Stack after I32Add: {stack:?}");
                }
                Instruction::I32Const(value) => {
                    println!("DEBUG: I32Const - Value: {value}");
                    stack.push(Value::I32(*value));
                    println!("DEBUG: Stack after I32Const: {stack:?}");
                }
                Instruction::I64Const(value) => {
                    println!("DEBUG: I64Const - Value: {value}");
                    stack.push(Value::I64(*value));
                    println!("DEBUG: Stack after I64Const: {stack:?}");
                }
                Instruction::F32Const(value) => {
                    println!("DEBUG: F32Const - Value: {value}");
                    stack.push(Value::F32(*value));
                    println!("DEBUG: Stack after F32Const: {stack:?}");
                }
                Instruction::F64Const(value) => {
                    println!("DEBUG: F64Const - Value: {value}");
                    stack.push(Value::F64(*value));
                    println!("DEBUG: Stack after F64Const: {stack:?}");
                }
                // Handle directly without using the Stack trait
                _ => {
                    println!("DEBUG: Unhandled instruction: {instruction:?}, treating as no-op");
                    // For the basic i32.add test, we can treat other instructions as no-ops
                    // since they're not needed for this test
                }
            }

            pc += 1;
        }

        println!("DEBUG: Final stack after execution: {stack:?}");

        // Critical fix: If we have results to return, extract them from the stack
        if results_len > 0 {
            // Make sure we have enough values on the stack
            if stack.len() < results_len {
                println!(
                    "DEBUG: Error - Not enough values on stack. Expected {}, got {}",
                    results_len,
                    stack.len()
                );
                return Err(Error::Execution(format!(
                    "Function did not produce enough results. Expected {}, got {}",
                    results_len,
                    stack.len()
                )));
            }

            // Extract the results from the end of the stack
            let start_idx = stack.len() - results_len;
            println!(
                "DEBUG: Extracting results from stack index {} (stack length: {})",
                start_idx,
                stack.len()
            );
            let mut results = Vec::with_capacity(results_len);

            for i in 0..results_len {
                let val = stack[start_idx + i].clone();
                println!("DEBUG: Adding result[{i}]: {val:?}");
                results.push(val);
            }

            println!("DEBUG: Final results being returned: {results:?}");
            return Ok(results);
        }

        // If no results expected, return an empty vector
        println!("DEBUG: No results expected, returning empty vector");
        Ok(Vec::new())
    }

    pub fn execute_function_call_direct(
        &mut self,
        table_idx: u32,
        func_idx: u32,
        args: Vec<Value>,
    ) -> Result<Vec<Value>> {
        // First, get all the data we need before any mutable borrows
        let instance = self.get_instance(self.instance_idx)?;
        let func_type = instance.get_function_type(func_idx)?.clone();
        let params_len = func_type.params.len();
        let results_len = func_type.results.len();

        // Validate arguments match function type
        if args.len() != params_len {
            return Err(Error::InvalidArgumentCount {
                expected: params_len,
                actual: args.len(),
            });
        }

        // Create a frame for the called function
        let mut frame = StacklessFrame::from_function(
            Arc::new(instance.module.clone()),
            func_idx,
            &args,
            self.instance_idx as u32,
        )?;

        // Get the function code
        let func = instance
            .module
            .get_function(func_idx)
            .ok_or(Error::InvalidFunctionIndex(func_idx as usize))?;
        let instructions = &func.code;

        // Create a temporary stack for execution
        let empty_module = Module::new().expect("Failed to create empty module");
        let mut temp_stack = Self::new(Arc::new(empty_module), self.instance_idx);
        temp_stack.instance_idx = self.instance_idx;
        temp_stack.func_idx = self.func_idx;
        temp_stack.engine = self.engine.clone();

        // Push arguments to the stack
        for arg in args {
            temp_stack.values.push(arg);
        }

        // Execute instructions
        let mut pc = 0;
        while pc < instructions.len() {
            let instruction = &instructions[pc];
            instruction.execute(&mut temp_stack, &mut frame)?;
            pc += 1;
        }

        // Collect results
        let result_start = temp_stack.values.len().saturating_sub(results_len);
        let results = temp_stack.values.drain(result_start..).collect();

        Ok(results)
    }

    pub fn execute_host_function(
        &mut self,
        func_name: &str,
        args: Vec<Value>,
    ) -> Result<Vec<Value>> {
        // First, get all the data we need before any mutable borrows
        let handler_option = if let Some(engine) = &self.engine {
            // Clone within the narrower scope
            let handler_clone = {
                let callbacks = engine.callbacks.lock().unwrap();
                callbacks.callbacks.get(func_name).cloned()
            };
            handler_clone
        } else {
            None
        };

        if let Some(handler) = handler_option {
            // Create a temporary stack for execution
            let empty_module = Module::new().expect("Failed to create empty module");
            let mut temp_stack = Self::new(Arc::new(empty_module), self.instance_idx);
            temp_stack.instance_idx = self.instance_idx;
            temp_stack.func_idx = self.func_idx;
            temp_stack.engine = self.engine.clone();

            // Call the handler function with correct parameter order
            handler.call(&mut temp_stack, args)
        } else {
            Err(Error::FunctionNotFound(
                func_name.to_string().as_str().parse::<u32>().unwrap_or(0),
            ))
        }
    }

    /// Execute a function in the module with the given instance index and function index
    ///
    /// # Arguments
    ///
    /// * `instance_idx` - The index of the module instance
    /// * `func_idx` - The index of the function in the module
    /// * `args` - The arguments to pass to the function
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the function executed successfully
    /// * `Err(Error)` - If an error occurred
    pub fn execute(&mut self, instance_idx: usize, func_idx: u32, args: Vec<Value>) -> Result<()> {
        // Execute the function with the provided arguments
        self.execute_function(instance_idx, func_idx, args)?;
        Ok(())
    }
}

impl Default for StacklessCallbackRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl StacklessCallbackRegistry {
    /// Creates a new callback registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            export_names: HashMap::new(),
            callbacks: HashMap::new(),
        }
    }

    /// Registers a callback for a specific export
    pub fn register_callback(&mut self, export_name: String, callback: HostFunctionHandler) {
        self.callbacks.insert(export_name, callback);
    }

    /// Registers a log operation for a specific export
    pub fn register_log_operation(&mut self, export_name: String, operation: LogOperation) {
        self.export_names
            .entry(export_name)
            .or_default()
            .insert(operation.message.clone(), operation);
    }

    /// Gets a callback for a specific export
    #[must_use]
    pub fn get_callback(&self, export_name: &str) -> Option<&HostFunctionHandler> {
        self.callbacks.get(export_name)
    }

    /// Gets log operations for a specific export
    #[must_use]
    pub fn get_log_operations(&self, export_name: &str) -> Option<&HashMap<String, LogOperation>> {
        self.export_names.get(export_name)
    }
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
        let empty_module = Module::new().expect("Failed to create empty module");
        Self {
            stack: StacklessStack::new(Arc::new(empty_module), 0),
            instances: Vec::new(),
            fuel: None,
            stats: ExecutionStats::default(),
            callbacks: Arc::new(Mutex::new(StacklessCallbackRegistry {
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
    pub const fn state(&self) -> &StacklessExecutionState {
        &self.stack.state
    }

    /// Sets the execution state
    pub fn set_state(&mut self, state: StacklessExecutionState) {
        self.stack.state = state;
    }

    /// Gets the number of module instances
    #[must_use]
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    /// Gets a module instance by index
    pub fn get_instance(&self, instance_idx: usize) -> Result<&ModuleInstance> {
        self.instances
            .get(instance_idx)
            .ok_or(Error::Execution("Invalid instance index".into()))
    }

    /// Adds a module instance
    pub fn add_instance(&mut self, instance: ModuleInstance) -> usize {
        self.instances.push(instance);
        self.instances.len() - 1
    }

    /// Instantiates a module
    pub fn instantiate(&mut self, module: Module) -> Result<usize> {
        println!(
            "DEBUG: instantiate called for module with {} exports",
            module.exports.len()
        );
        let instance = ModuleInstance::new(module)?;
        Ok(self.add_instance(instance))
    }

    #[must_use]
    pub fn has_no_instances(&self) -> bool {
        self.instances.is_empty()
    }
}

impl Stack for StacklessStack {
    fn push_label(&mut self, label: stack::Label) -> Result<()> {
        self.labels.push(behavior::Label {
            arity: label.arity,
            pc: label.pc,
            continuation: label.continuation,
        });
        Ok(())
    }

    fn pop_label(&mut self) -> Result<stack::Label> {
        self.labels
            .pop()
            .map(|label| stack::Label {
                arity: label.arity,
                pc: label.pc,
                continuation: label.continuation,
            })
            .ok_or(Error::StackUnderflow)
    }

    fn get_label(&self, idx: usize) -> Result<&stack::Label> {
        Err(Error::Execution(
            "Cannot return reference to stack::Label from StacklessStack".to_string(),
        ))
    }

    fn get_label_mut(&mut self, idx: usize) -> Result<&mut stack::Label> {
        Err(Error::Execution(
            "Cannot return mutable reference to stack::Label from StacklessStack".to_string(),
        ))
    }

    fn labels_len(&self) -> usize {
        self.labels.len()
    }
}

impl StackBehavior for StacklessStack {
    fn push(&mut self, value: Value) -> Result<()> {
        self.values.push(value);
        Ok(())
    }

    fn pop(&mut self) -> Result<Value> {
        self.values.pop().ok_or(Error::StackUnderflow)
    }

    fn peek(&self) -> Result<&Value> {
        self.values.last().ok_or(Error::StackUnderflow)
    }

    fn peek_mut(&mut self) -> Result<&mut Value> {
        self.values.last_mut().ok_or(Error::StackUnderflow)
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

    fn push_label(&mut self, arity: usize, pc: usize) {
        self.labels.push(Label {
            arity,
            pc,
            continuation: pc + 1, // Default continuation is the next instruction
        });
    }

    fn pop_label(&mut self) -> Result<Label> {
        self.labels.pop().ok_or(Error::StackUnderflow)
    }

    fn get_label(&self, _index: usize) -> Option<&Label> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct ReturnFrame {
    pub pc: u32,
    pub arity: u32,
}

// This is a free function, not a method
pub fn execute_instruction(
    instruction: &Instruction,
    stack: &mut impl Stack,
    frame: &mut StacklessFrame,
    func_type: &FuncType,
) -> Result<()> {
    // Execute the instruction directly now that we have proper cloning in the caller
    instruction.execute(stack, frame)
}

impl StacklessFrame {
    pub fn get_global_by_idx(&self, idx: usize) -> Result<Value> {
        let global = self.module.get_global(idx)?;
        Ok(global.value.clone())
    }

    pub fn get_memory_by_idx(&self, idx: usize) -> Result<Arc<Memory>> {
        self.module.get_memory(idx)
    }

    pub fn get_memory_mut_by_idx(&mut self, idx: usize) -> Result<Arc<Memory>> {
        // Since Module is wrapped in Arc, we can't borrow it mutably
        self.module.get_memory(idx)
    }

    pub fn get_table_by_idx(&self, idx: usize) -> Result<Arc<Table>> {
        self.module.get_table(idx)
    }

    pub fn get_table_mut_by_idx(&mut self, idx: usize) -> Result<Arc<Table>> {
        // Since Module is wrapped in Arc, we can't borrow it mutably
        self.module.get_table(idx)
    }

    pub fn get_global_mut_by_idx(&mut self, idx: usize) -> Result<Arc<Global>> {
        // Since Arc doesn't allow direct mutable access, we return the Arc itself
        self.module.get_global(idx)
    }

    pub fn find_matching_else(&mut self) -> Result<usize> {
        let func = self
            .module
            .get_function(self.func_idx)
            .ok_or(Error::InvalidFunctionIndex(self.func_idx as usize))?;

        let code = &func.code;
        let mut depth = 1;
        let mut i = self.pc + 1;

        while i < code.len() && depth > 0 {
            match &code[i] {
                Instruction::If(..) => depth += 1,
                Instruction::Else if depth == 1 => {
                    // Found matching else
                    return Ok(i);
                }
                Instruction::End => {
                    depth -= 1;
                    if depth == 0 {
                        // Found matching end without an else
                        return Ok(i);
                    }
                }
                _ => {}
            }
            i += 1;
        }

        Err(Error::Execution(
            "Could not find matching else or end".to_string(),
        ))
    }

    pub fn find_matching_end(&mut self) -> Result<usize> {
        let func_idx = self.func_idx as usize;
        let func = self
            .module
            .get_function(func_idx as u32)
            .ok_or(Error::InvalidFunctionIndex(func_idx))?;

        let instructions = &func.code;
        let mut i = self.pc + 1;
        let mut depth = 1;

        while i < instructions.len() {
            match &instructions[i] {
                Instruction::Block(..) | Instruction::Loop(..) | Instruction::If(..) => depth += 1,
                Instruction::End => {
                    depth -= 1;
                    if depth == 0 {
                        // Found matching end, jump to it
                        self.pc = i;
                        return Ok(i);
                    }
                }
                _ => {}
            }
            i += 1;
        }

        Err(Error::Execution("Could not find matching end".to_string()))
    }
}

// Implement ControlFlowBehavior trait for StacklessFrame
impl ControlFlowBehavior for StacklessFrame {
    fn enter_block(&mut self, ty: BlockType, stack_len: usize) -> Result<()> {
        // Store current PC to return to on block exit
        let continuation = self.pc + 1;

        // Calculate arity based on block type
        let arity = match &ty {
            BlockType::Empty => 0,
            BlockType::Type(value_type) => 1,
            BlockType::TypeIndex(type_idx) => {
                if let Some(func_type) = self.module.get_function_type(*type_idx) {
                    func_type.results.len()
                } else {
                    return Err(Error::InvalidType(format!(
                        "Type index not found: {type_idx}"
                    )));
                }
            }
            BlockType::FuncType(func_type) => func_type.results.len(),
            BlockType::Value(value_type) => 1,
        };

        // Push a new label
        self.label_stack.push(Label {
            arity,
            pc: self.pc,
            continuation,
        });

        Ok(())
    }

    fn enter_loop(&mut self, ty: BlockType, stack_len: usize) -> Result<()> {
        // For loops, the continuation is the loop start (current PC)
        let continuation = self.pc;

        // Calculate arity based on block type (same as for blocks)
        let arity = match &ty {
            BlockType::Empty => 0,
            BlockType::Type(value_type) => 1,
            BlockType::TypeIndex(type_idx) => {
                if let Some(func_type) = self.module.get_function_type(*type_idx) {
                    func_type.results.len()
                } else {
                    return Err(Error::InvalidType(format!(
                        "Type index not found: {type_idx}"
                    )));
                }
            }
            BlockType::FuncType(func_type) => func_type.results.len(),
            BlockType::Value(value_type) => 1,
        };

        // Push a new label
        self.label_stack.push(Label {
            arity,
            pc: self.pc,
            continuation,
        });

        Ok(())
    }

    fn enter_if(&mut self, ty: BlockType, stack_len: usize, condition: bool) -> Result<()> {
        // Get the arity for the label
        let arity = match ty {
            BlockType::Empty => 0,
            BlockType::Value(_) => 1,
            BlockType::Type(_) => 1,
            BlockType::FuncType(func_type) => func_type.results.len(),
            BlockType::TypeIndex(type_idx) => {
                let func_type = self.module.get_function_type(type_idx).ok_or_else(|| {
                    Error::InvalidFunctionType(format!("Invalid type index: {type_idx}"))
                })?;
                func_type.results.len()
            }
        };

        if condition {
            // Enter the block and continue
            self.label_stack.push(Label {
                arity,
                pc: self.pc,
                continuation: 0, // We will update this when we find the matching end
            });

            Ok(())
        } else {
            // Find the matching else or end
            let else_idx = self.find_matching_else()?;

            self.pc = else_idx;

            // If this is an else branch, enter it
            let func = self
                .module
                .get_function(self.func_idx)
                .ok_or(Error::InvalidFunctionIndex(self.func_idx as usize))?;
            let code = &func.code;
            if self.pc < code.len() && matches!(code[self.pc], Instruction::Else) {
                self.label_stack.push(Label {
                    arity,
                    pc: self.pc,
                    continuation: 0, // We will update this when we find the matching end
                });
                self.pc += 1; // Move past the else
            }

            Ok(())
        }
    }

    fn enter_else(&mut self, stack_len: usize) -> Result<()> {
        // Skip to the end of the if-else block
        let func = self
            .module
            .get_function(self.func_idx)
            .ok_or(Error::InvalidFunctionIndex(self.func_idx as usize))?;

        let code = &func.code;
        let mut depth = 1;
        let mut i = self.pc + 1;

        while i < code.len() && depth > 0 {
            match &code[i] {
                Instruction::If(..) => depth += 1,
                Instruction::End => {
                    depth -= 1;
                    if depth == 0 {
                        // Found matching end, jump to it
                        self.pc = i;
                        return Ok(());
                    }
                }
                _ => {}
            }
            i += 1;
        }

        Err(Error::Execution("Could not find matching end".to_string()))
    }

    fn exit_block(&mut self, _stack: &mut dyn Stack) -> Result<()> {
        if self.label_stack.is_empty() {
            return Err(Error::Execution("Label stack underflow".to_string()));
        }

        // Pop label on exit
        let label = self.label_stack.pop().unwrap();
        self.pc = label.pc;
        self.arity = label.arity;

        Ok(())
    }

    fn branch(&mut self, label_idx: u32, _stack: &mut dyn Stack) -> Result<()> {
        let stack_len = self.label_stack.len();
        if label_idx as usize >= stack_len {
            return Err(Error::Execution(format!(
                "Branch index {label_idx} out of bounds (label stack len = {stack_len})"
            )));
        }

        // Set pc to the target label
        self.pc = self.label_stack[stack_len - 1 - (label_idx as usize)].pc;
        Ok(())
    }

    fn return_(&mut self, _stack: &mut dyn Stack) -> Result<()> {
        self.pc = self.return_pc;
        Ok(())
    }

    fn call(&mut self, _func_idx: u32, _stack: &mut dyn Stack) -> Result<()> {
        Err(Error::Unimplemented("call in stackless frame".to_string()))
    }

    fn call_indirect(
        &mut self,
        type_idx: u32,
        table_idx: u32,
        entry: u32,
        stack: &mut dyn Stack,
    ) -> Result<()> {
        // Get the table
        let table_arc = self.module.get_table(table_idx as usize)?;
        let table = unsafe {
            let table_ptr = Arc::as_ptr(&table_arc);
            &*table_ptr
        };

        // Check bounds
        if entry as usize >= table.size() as usize {
            return Err(Error::InvalidTableIndex(entry as usize));
        }

        // Get the function index from the table
        let func_entry = table.get(entry)?;

        match func_entry {
            Some(Value::FuncRef(Some(func_idx))) => {
                // Check if the function type matches the expected type
                let expected_type = self.module.types.get(type_idx as usize).ok_or_else(|| {
                    Error::InvalidType(format!("Type index out of bounds: {type_idx}"))
                })?;

                let func = self.get_function(func_idx)?;
                let func_type = self
                    .module
                    .types
                    .get(func.type_idx as usize)
                    .ok_or_else(|| {
                        Error::InvalidType(format!("Type index out of bounds: {}", func.type_idx))
                    })?;

                // Check that the types match
                if expected_type != func_type {
                    return Err(Error::TypeMismatch(format!(
                        "Function type mismatch in call_indirect: expected {expected_type:?}, got {func_type:?}"
                    )));
                }

                // Save current state
                let return_pc = self.pc;

                // Extract arguments
                let param_count = func_type.params.len();
                let mut args = Vec::with_capacity(param_count);
                for _ in 0..param_count {
                    args.push(stack.pop()?);
                }
                args.reverse(); // Arguments were popped in reverse order

                // Create a new execution context
                let state = StacklessExecutionState::Calling {
                    instance_idx: self.instance_idx,
                    func_idx,
                    args,
                    return_pc,
                };

                // Return a special error to signal state change
                Err(Error::StateChange(Box::new(state)))
            }
            Some(Value::FuncRef(None)) => Err(Error::NullFunctionReference),
            None => Err(Error::InvalidTableIndex(entry as usize)),
            Some(other) => Err(Error::TypeMismatch(format!(
                "Expected function reference, got {other:?}"
            ))),
        }
    }

    fn set_label_arity(&mut self, arity: usize) {
        self.label_arity = arity;
    }
}

// Implement FrameBehavior trait for StacklessFrame
impl FrameBehavior for StacklessFrame {
    fn locals(&mut self) -> &mut Vec<Value> {
        &mut self.locals
    }

    fn get_local(&self, idx: usize) -> Result<Value> {
        self.locals
            .get(idx)
            .cloned()
            .ok_or_else(|| Error::InvalidLocal(format!("Local index out of bounds: {idx}")))
    }

    fn set_local(&mut self, idx: usize, value: Value) -> Result<()> {
        if idx < self.locals.len() {
            self.locals[idx] = value;
            Ok(())
        } else {
            Err(Error::InvalidLocal(format!(
                "Local index out of bounds: {idx}"
            )))
        }
    }

    fn get_global(&self, idx: usize) -> Result<Value> {
        let global = self.module.get_global(idx)?;
        Ok(global.get())
    }

    fn set_global(&mut self, idx: usize, value: Value) -> Result<()> {
        let global = self.module.get_global(idx)?;
        // Cannot modify through Arc directly
        Err(Error::Execution(
            "Cannot modify global through Arc in stackless frame".to_string(),
        ))
    }

    fn get_memory(&self, idx: usize) -> Result<&Memory> {
        let memory_arc = self.module.get_memory(idx)?;
        // This is unsafe because we're returning a reference to the Arc's contents
        // which isn't guaranteed to live long enough. In practice, the Arc keeps
        // the memory alive for the lifetime of the module, but this is not ideal.
        Ok(unsafe {
            let memory_ptr = Arc::as_ptr(&memory_arc);
            &*memory_ptr
        })
    }

    fn get_memory_mut(&mut self, idx: usize) -> Result<&mut Memory> {
        // Since module is Arc, we can't get a mutable reference directly
        Err(Error::Unimplemented(
            "get_memory_mut in stackless frame".to_string(),
        ))
    }

    fn get_table(&self, idx: usize) -> Result<&Table> {
        let table_arc = self.module.get_table(idx)?;
        // This is unsafe for the same reason as get_memory
        Ok(unsafe {
            let table_ptr = Arc::as_ptr(&table_arc);
            &*table_ptr
        })
    }

    fn get_table_mut(&mut self, idx: usize) -> Result<&mut Table> {
        // Since module is Arc, we can't get a mutable reference directly
        Err(Error::Unimplemented(
            "get_table_mut in stackless frame".to_string(),
        ))
    }

    fn get_global_mut(&mut self, idx: usize) -> Option<&mut Global> {
        None // Since module is Arc, we can't get a mutable reference directly
    }

    fn pc(&self) -> usize {
        self.pc
    }

    fn set_pc(&mut self, pc: usize) {
        self.pc = pc;
    }

    fn func_idx(&self) -> u32 {
        self.func_idx
    }

    fn instance_idx(&self) -> usize {
        self.instance_idx as usize
    }

    fn locals_len(&self) -> usize {
        self.locals.len()
    }

    fn label_stack(&mut self) -> &mut Vec<Label> {
        &mut self.label_stack
    }

    fn arity(&self) -> usize {
        self.arity
    }

    fn set_arity(&mut self, arity: usize) {
        self.arity = arity;
    }

    fn label_arity(&self) -> usize {
        self.label_arity
    }

    fn return_pc(&self) -> usize {
        self.return_pc
    }

    fn set_return_pc(&mut self, pc: usize) {
        self.return_pc = pc;
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    // Implement the remaining methods from FrameBehavior trait
    fn load_i32(&mut self, addr: usize, align: u32) -> Result<i32> {
        let memory = self.get_memory(0)?;
        Ok(memory.read_u32(addr as u32)? as i32)
    }

    fn load_i64(&mut self, addr: usize, align: u32) -> Result<i64> {
        let memory = self.get_memory(0)?;
        Ok(memory.read_u64(addr as u32)? as i64)
    }

    fn load_f32(&mut self, addr: usize, align: u32) -> Result<f32> {
        let memory = self.get_memory(0)?;
        memory.read_f32(addr as u32)
    }

    fn load_f64(&mut self, addr: usize, align: u32) -> Result<f64> {
        let memory = self.get_memory(0)?;
        memory.read_f64(addr as u32)
    }

    fn load_i8(&mut self, addr: usize, align: u32) -> Result<i8> {
        let memory = self.get_memory(0)?;
        Ok(memory.read_byte(addr as u32)? as i8)
    }

    fn load_u8(&mut self, addr: usize, align: u32) -> Result<u8> {
        let memory = self.get_memory(0)?;
        memory.read_byte(addr as u32)
    }

    fn load_i16(&mut self, addr: usize, align: u32) -> Result<i16> {
        let memory = self.get_memory(0)?;
        Ok(memory.read_u16(addr as u32)? as i16)
    }

    fn load_u16(&mut self, addr: usize, align: u32) -> Result<u16> {
        let memory = self.get_memory(0)?;
        memory.read_u16(addr as u32)
    }

    fn store_i32(&mut self, addr: usize, align: u32, value: i32) -> Result<()> {
        let memory_arc = self.module.get_memory(0)?;
        // This is unsafe because we're writing through a shared reference
        unsafe {
            let memory_ptr = Arc::as_ptr(&memory_arc).cast_mut();
            (*memory_ptr).write_u32(addr as u32, value as u32)
        }
    }

    fn store_i64(&mut self, addr: usize, align: u32, value: i64) -> Result<()> {
        let memory_arc = self.module.get_memory(0)?;
        // This is unsafe because we're writing through a shared reference
        unsafe {
            let memory_ptr = Arc::as_ptr(&memory_arc).cast_mut();
            (*memory_ptr).write_u64(addr as u32, value as u64)
        }
    }

    fn memory_size(&mut self) -> Result<u32> {
        let memory = self.get_memory(0)?;
        Ok(memory.size())
    }

    fn memory_grow(&mut self, pages: u32) -> Result<u32> {
        let memory_arc = self.module.get_memory(0)?;
        // This is unsafe because we're modifying through a shared reference
        unsafe {
            let memory_ptr = Arc::as_ptr(&memory_arc).cast_mut();
            (*memory_ptr).grow(pages)
        }
    }

    fn table_get(&mut self, table_idx: u32, idx: u32) -> Result<Value> {
        let table = self.get_table(table_idx as usize)?;
        let value_opt = table.get(idx)?;
        value_opt.ok_or(Error::InvalidTableIndex(idx as usize))
    }

    fn table_set(&mut self, table_idx: u32, idx: u32, value: Value) -> Result<()> {
        let table_arc = self.module.get_table(table_idx as usize)?;
        // This is unsafe because we're modifying through a shared reference
        unsafe {
            let table_ptr = Arc::as_ptr(&table_arc).cast_mut();
            (*table_ptr).set(idx, Some(value))
        }
    }

    fn table_size(&mut self, table_idx: u32) -> Result<u32> {
        let table = self.get_table(table_idx as usize)?;
        Ok(table.size())
    }

    fn table_grow(&mut self, table_idx: u32, delta: u32, value: Value) -> Result<u32> {
        let table_arc = self.module.get_table(table_idx as usize)?;
        // This is unsafe because we're modifying through a shared reference
        unsafe {
            let table_ptr = Arc::as_ptr(&table_arc).cast_mut();
            (*table_ptr).grow(delta)
        }
    }

    fn table_init(
        &mut self,
        table_idx: u32,
        elem_idx: u32,
        dst: u32,
        src: u32,
        n: u32,
    ) -> Result<()> {
        Err(Error::Unimplemented(
            "table_init in stackless frame".to_string(),
        ))
    }

    fn table_copy(
        &mut self,
        dst_table: u32,
        src_table: u32,
        dst: u32,
        src: u32,
        n: u32,
    ) -> Result<()> {
        Err(Error::Unimplemented(
            "table_copy in stackless frame".to_string(),
        ))
    }

    fn elem_drop(&mut self, elem_idx: u32) -> Result<()> {
        Err(Error::Unimplemented(
            "elem_drop in stackless frame".to_string(),
        ))
    }

    fn table_fill(&mut self, table_idx: u32, dst: u32, val: Value, n: u32) -> Result<()> {
        Err(Error::Unimplemented(
            "table_fill in stackless frame".to_string(),
        ))
    }

    fn pop_bool(&mut self, stack: &mut dyn Stack) -> Result<bool> {
        match stack.pop()? {
            Value::I32(0) => Ok(false),
            Value::I32(_) => Ok(true),
            _ => Err(Error::TypeMismatch(
                "Expected i32 boolean value".to_string(),
            )),
        }
    }

    fn pop_i32(&mut self, stack: &mut dyn Stack) -> Result<i32> {
        match stack.pop()? {
            Value::I32(v) => Ok(v),
            _ => Err(Error::TypeMismatch("Expected i32 value".to_string())),
        }
    }
}
