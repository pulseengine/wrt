use crate::error::{Error, Result};
use crate::instructions::Instruction;
use crate::module::Module;
use crate::values::Value;
use crate::{format, Vec};

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
    instance_idx: u32,
    /// Function index
    func_idx: u32,
}

/// Represents a table address
#[derive(Debug, Clone)]
struct TableAddr {
    /// Module instance index
    instance_idx: u32,
    /// Table index
    table_idx: u32,
}

/// Represents a memory address
#[derive(Debug, Clone)]
struct MemoryAddr {
    /// Module instance index
    instance_idx: u32,
    /// Memory index
    memory_idx: u32,
}

/// Represents a global address
#[derive(Debug, Clone)]
struct GlobalAddr {
    /// Module instance index
    instance_idx: u32,
    /// Global index
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

/// The WebAssembly execution engine
#[derive(Debug)]
pub struct Engine {
    /// Execution stack
    stack: Stack,
    /// Module instances
    instances: Vec<ModuleInstance>,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    /// Creates a new execution engine
    pub fn new() -> Self {
        Self {
            stack: Stack::new(),
            instances: Vec::new(),
        }
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

    /// Executes a function
    pub fn execute(
        &mut self,
        instance_idx: u32,
        func_idx: u32,
        args: Vec<Value>,
    ) -> Result<Vec<Value>> {
        // Clone the necessary information to avoid borrow issues
        let instance_clone;
        let func_clone;
        let func_type_clone;

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
            func_clone = func.clone();
            func_type_clone = func_type.clone();
        }

        // Create frame
        let mut frame = Frame {
            func_idx,
            locals: Vec::new(),
            module: instance_clone,
        };

        // Initialize locals with arguments
        frame.locals.extend(args);

        // Push frame
        self.stack.push_frame(frame);

        // Execute function body using the cloned data
        let mut pc = 0;
        while pc < func_clone.body.len() {
            match self.execute_instruction(&func_clone.body[pc], pc) {
                Ok(Some(new_pc)) => pc = new_pc,
                Ok(None) => pc += 1,
                Err(e) => return Err(e),
            }
        }

        // Pop frame
        self.stack.pop_frame()?;

        // Return results
        let mut results = Vec::new();
        for _ in 0..func_type_clone.results.len() {
            results.push(self.stack.pop()?);
        }
        results.reverse();

        Ok(results)
    }

    /// Executes a single instruction
    fn execute_instruction(&mut self, inst: &Instruction, pc: usize) -> Result<Option<usize>> {
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
            // ... implement other instructions ...
            _ => Err(Error::Execution("Instruction not implemented".into())),
        }
    }
}
