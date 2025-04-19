//! Stackless function activation frame
//!
//! This module defines the `StacklessFrame` struct and its associated implementations,
//! representing the state of a single function activation in the stackless WRT engine.

use core::fmt::Debug;
use std::sync::Arc;

use crate::{
    behavior::{ControlFlowBehavior, FrameBehavior, Label, StackBehavior},
    error::{kinds, Error, Result},
    global::Global,
    instructions::instruction_type::Instruction,
    memory::MemoryBehavior,
    module::{Data, Element, Function, Module},
    stackless::StacklessEngine,
    table::Table,
    types::{BlockType, FuncType, ValueType},
    values::Value,
};

// Add wrt_sync as an external crate
use wrt_sync;

/// Represents a function activation frame in the stackless engine.
#[derive(Debug, Clone)]
pub struct StacklessFrame {
    /// The module associated with this frame.
    pub module: Arc<Module>,
    /// The index of the function being executed in this frame.
    pub func_idx: u32,
    /// The program counter, indicating the next instruction to execute within the function's code.
    pub pc: usize,
    /// The local variables for this frame, including function arguments.
    /// Note: In some contexts within the stackless engine, this might also temporarily hold operand stack values.
    pub locals: Vec<Value>,
    /// The index of the module instance this frame belongs to.
    pub instance_idx: u32,
    /// The number of return values expected by the caller of this function frame.
    pub arity: usize,
    /// The arity (number of stack values expected) of the current control flow block (block, loop, if).
    pub label_arity: usize,
    /// The stack of active control flow labels (blocks, loops, ifs) within this frame.
    pub label_stack: Vec<Label>,
    /// The program counter in the *caller's* frame to return to after this frame finishes.
    pub return_pc: usize,
}

impl StacklessFrame {
    /// Creates a new stackless frame for a function call (internal helper).
    /// Validates argument count and types against the function signature.
    fn new_internal(
        module: Arc<Module>,
        instance_idx: u32,
        func_idx: u32,
        args: Vec<Value>,
    ) -> Result<Self> {
        // Extract the function type directly without keeping any borrows alive
        let func_type = module
            .get_function_type(func_idx)
            .ok_or_else(|| Error::new(kinds::InvalidFunctionIndexError(func_idx as usize)))?;

        // Store the result counts we need
        let results_len = func_type.results.len();
        let params_len = func_type.params.len();

        // Validate arguments
        if args.len() != params_len {
            return Err(Error::new(kinds::InvalidFunctionType(format!(
                "Function {func_idx}: Expected {} arguments, got {}",
                params_len,
                args.len()
            ))));
        }

        // Check argument types
        for (i, (arg, param_type)) in args.iter().zip(func_type.params.iter()).enumerate() {
            if !arg.matches_type(param_type) {
                return Err(Error::new(kinds::InvalidType(format!(
                    "Function {func_idx}: Argument {} type mismatch: expected {:?}, got {:?}",
                    i,
                    param_type,
                    arg.get_type()
                ))));
            }
        }

        Ok(Self {
            module,
            func_idx,
            pc: 0,        // Start at the beginning of the function code
            locals: args, // Arguments become the initial part of locals
            instance_idx,
            arity: results_len,      // Frame arity is the function's RETURN arity
            label_arity: params_len, // Initial label arity matches function INPUT arity
            label_stack: Vec::new(),
            return_pc: 0, // Will be set by the caller
        })
    }

    /// Creates a new stackless frame prepared for executing a specific function.
    /// Initializes locals with arguments and default values for declared local variables.
    pub fn new(
        module: Arc<Module>,
        func_idx: u32,
        args: &[Value],
        instance_idx: u32,
    ) -> Result<Self> {
        // First, look up the function to get its type index and locals
        let function = match module.functions.get(func_idx as usize) {
            Some(f) => f,
            None => return Err(Error::new(kinds::FunctionNotFound(func_idx))),
        };

        // Get the function type
        let func_type = match module.types.get(function.type_idx as usize) {
            Some(t) => t,
            None => {
                return Err(Error::new(kinds::InvalidFunctionType(format!(
                    "Function type index not found: {}",
                    func_idx
                ))))
            }
        };

        // Store the result counts we need
        let results_len = func_type.results.len();
        let params_len = func_type.params.len();

        // Validate arguments
        if args.len() != params_len {
            return Err(Error::new(kinds::InvalidType(format!(
                "Function expects {} arguments, but {} provided",
                params_len,
                args.len()
            ))));
        }

        // Create locals with arguments and default values
        let mut locals = Vec::with_capacity(args.len() + function.locals.len());
        locals.extend_from_slice(args);

        // Initialize local variables based on their types
        for &local_type in &function.locals {
            match local_type {
                ValueType::I32 => locals.push(Value::I32(0)),
                ValueType::I64 => locals.push(Value::I64(0)),
                ValueType::F32 => locals.push(Value::F32(0.0)),
                ValueType::F64 => locals.push(Value::F64(0.0)),
                ValueType::V128 => locals.push(Value::V128([0; 16])),
                ValueType::FuncRef => locals.push(Value::FuncRef(None)),
                ValueType::ExternRef => locals.push(Value::ExternRef(None)),
                ValueType::AnyRef => locals.push(Value::AnyRef(None)),
            }
        }

        Ok(Self {
            module,
            func_idx,
            pc: 0,
            locals,
            instance_idx,
            arity: results_len,      // Frame arity is the function's RETURN arity
            label_arity: params_len, // Initial label arity matches function INPUT arity
            label_stack: Vec::new(),
            return_pc: 0, // Will be set by the caller
        })
    }

    /// Gets the function definition associated with this frame.
    pub fn get_function(&self) -> Result<&Function> {
        self.module
            .get_function(self.func_idx)
            .ok_or_else(|| Error::new(kinds::FunctionNotFound(self.func_idx)))
    }

    /// Retrieves the instruction at the specified program counter for the current function.
    pub fn get_instruction_at(&self, pc: usize) -> Result<&Instruction> {
        let func = self.get_function()?;
        func.code.get(pc).ok_or_else(|| {
            Error::new(kinds::ExecutionError(format!(
                "Instruction not found at PC {} in function {}",
                pc, self.func_idx
            )))
        })
    }

    /// Gets the function type associated with this frame.
    pub fn get_function_type(&self) -> Result<&FuncType> {
        self.module.get_function_type(self.func_idx).ok_or_else(|| {
            Error::new(kinds::InvalidFunctionType(format!(
                "Function type not found for index: {}",
                self.func_idx
            )))
        })
    }

    /// Finds the program counter (PC) of the matching `Else` or `End` instruction
    /// for the `If` block starting *after* the current frame PC.
    /// Used when the `If` condition is false.
    pub fn find_matching_else_or_end(&self) -> Result<usize> {
        let func = self.get_function()?;
        let code = &func.code;
        let mut depth = 1; // Start inside the If block
        let mut pc = self.pc + 1; // Start searching after the If instruction

        while pc < code.len() {
            match &code[pc] {
                Instruction::If(..) => depth += 1,
                Instruction::Else if depth == 1 => {
                    // Found the matching Else for the initial If
                    return Ok(pc);
                }
                Instruction::End => {
                    depth -= 1;
                    if depth == 0 {
                        // Found the matching End for the initial If (no Else)
                        return Ok(pc);
                    }
                }
                _ => {}
            }
            pc += 1;
        }

        Err(Error::new(kinds::Execution(format!(
            "Unmatched If at PC {} in function {}",
            self.pc, self.func_idx
        ))))
    }

    /// Finds the program counter (PC) of the matching `End` instruction
    /// for the block (`Block`, `Loop`, `If`) starting *after* the current frame PC.
    /// Used primarily for skipping the `Else` block.
    pub fn find_matching_end(&self) -> Result<usize> {
        let func = self.get_function()?;
        let code = &func.code;
        let mut depth = 1; // Start inside the block needing an End
        let mut pc = self.pc + 1; // Start searching after the instruction starting the block (e.g., Else)

        while pc < code.len() {
            match &code[pc] {
                Instruction::Block(..) | Instruction::Loop(..) | Instruction::If(..) => depth += 1,
                Instruction::End => {
                    depth -= 1;
                    if depth == 0 {
                        // Found the matching End
                        return Ok(pc);
                    }
                }
                _ => {}
            }
            pc += 1;
        }
        Err(Error::new(kinds::Execution(format!(
            "Unmatched block starting near PC {} in function {}",
            self.pc, self.func_idx
        ))))
    }

    // Helper to resolve block type RESULT arity (using engine context)
    fn resolve_block_type_results_len(
        &self,
        ty: &BlockType,
        engine: &StacklessEngine,
    ) -> Result<usize> {
        match ty {
            BlockType::Empty => Ok(0),
            BlockType::Value(_) | BlockType::Type(_) => Ok(1), // Result arity is 1
            BlockType::FuncType(func_type) => Ok(func_type.results.len()),
            BlockType::TypeIndex(type_idx) => {
                // Need instance context to resolve function type index
                engine
                    .with_instance(self.instance_idx as usize, |instance| {
                        let func_type = instance.get_function_type(*type_idx)?;
                        Ok(func_type.results.len())
                    })
                    // Flatten the nested Result
                    .and_then(|inner_result| Ok(inner_result))
            }
        }
    }

    // Helper to resolve block type PARAMETER arity (using engine context)
    fn resolve_block_type_params_len(
        &self,
        ty: &BlockType,
        engine: &StacklessEngine,
    ) -> Result<usize> {
        match ty {
            BlockType::Empty => Ok(0),
            BlockType::Value(_) | BlockType::Type(_) => Ok(0), // Value block type has 0 params
            BlockType::FuncType(func_type) => Ok(func_type.params.len()),
            BlockType::TypeIndex(type_idx) => {
                // Need instance context to resolve function type index
                engine
                    .with_instance(self.instance_idx as usize, |instance| {
                        let func_type = instance.get_function_type(*type_idx)?;
                        Ok(func_type.params.len())
                    })
                    // Flatten the nested Result
                    .and_then(|inner_result| Ok(inner_result))
            }
        }
    }
}

// Implement the behavior traits

impl StackBehavior for StacklessFrame {
    // NOTE: StackBehavior for StacklessFrame often manipulates `locals` directly
    // when used within the engine's step function, as there isn't a separate operand stack.
    // Be cautious when interpreting these methods outside that context.

    fn push(&mut self, value: Value) -> Result<()> {
        self.locals.push(value);
        Ok(())
    }

    fn pop(&mut self) -> Result<Value> {
        self.locals
            .pop()
            .ok_or_else(|| Error::new(kinds::StackUnderflow))
    }

    fn peek(&self) -> Result<&Value> {
        self.locals
            .last()
            .ok_or_else(|| Error::new(kinds::StackUnderflow))
    }

    fn peek_mut(&mut self) -> Result<&mut Value> {
        self.locals
            .last_mut()
            .ok_or_else(|| Error::new(kinds::StackUnderflow))
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

    // Label stack operations are delegated to the frame's label_stack
    fn push_label(&mut self, label: Label) -> Result<(), Error> {
        // Push the label onto the label stack
        self.label_stack.push(label);
        Ok(())
    }

    fn pop_label(&mut self) -> Result<Label, Error> {
        self.label_stack
            .pop()
            .ok_or_else(|| Error::new(kinds::LabelStackUnderflowError))
    }

    fn get_label(&self, index: usize) -> Option<&Label> {
        None // Not supported on frames directly
    }

    fn push_n(&mut self, values: &[Value]) {
        // Not supported by frames, we don't store the operand stack
    }

    fn pop_n(&mut self, n: usize) -> Vec<Value> {
        // Not supported by frames, we don't store the operand stack
        Vec::new()
    }

    fn pop_frame_label(&mut self) -> Result<Label, Error> {
        // Not supported by frames, should be handled by stack
        Err(Error::new(kinds::UnimplementedError(
            "pop_frame_label not supported on frames".to_string(),
        )))
    }

    fn execute_function_call_direct(
        &mut self,
        engine: &mut StacklessEngine,
        caller_instance_idx: u32,
        func_idx: u32,
        args: Vec<Value>,
    ) -> Result<Vec<Value>, Error> {
        // Delegate to engine
        engine.call_function(caller_instance_idx, func_idx, &args)
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl ControlFlowBehavior for StacklessFrame {
    fn enter_block(&mut self, ty: BlockType, stack_len: usize) -> Result<()> {
        // Get the result number from the block type
        let block_results_len =
            self.resolve_block_type_results_len(&ty, &StacklessEngine::default())?;

        // Create a new label
        let label = Label {
            arity: block_results_len,
            pc: 0,           // Will be set later when we find the matching end
            continuation: 0, // not used for blocks
            stack_depth: stack_len,
            is_loop: false,
            is_if: false,
        };

        // Add to label stack
        self.label_stack.push(label);

        // Set the current label arity
        self.label_arity = block_results_len;

        Ok(())
    }

    fn enter_loop(&mut self, ty: BlockType, stack_len: usize) -> Result<()> {
        // Get the result number from the block type
        let block_results_len =
            self.resolve_block_type_results_len(&ty, &StacklessEngine::default())?;

        // Create a new label - note is_loop = true
        let label = Label {
            arity: block_results_len,
            pc: 0,                 // Will be set later
            continuation: self.pc, // For loops, continuation is the beginning
            stack_depth: stack_len,
            is_loop: true,
            is_if: false,
        };

        // Add to label stack
        self.label_stack.push(label);

        // Set the current label arity
        self.label_arity = block_results_len;

        Ok(())
    }

    fn enter_if(&mut self, ty: BlockType, stack_len: usize, condition: bool) -> Result<()> {
        // Get the result number from the block type
        let block_results_len =
            self.resolve_block_type_results_len(&ty, &StacklessEngine::default())?;

        // Create a new label - note is_if = true
        let label = Label {
            arity: block_results_len,
            pc: 0,           // Will be set later when we find the matching end
            continuation: 0, // Will be set if we find an else
            stack_depth: stack_len,
            is_loop: false,
            is_if: true,
        };

        // Add to label stack
        self.label_stack.push(label);

        // Set the current label arity
        self.label_arity = block_results_len;

        // If condition is false, jump to the else or end
        if !condition {
            // Find the matching else or end
            let target_pc = self.find_matching_else_or_end()?;
            self.pc = target_pc;
        }

        Ok(())
    }

    fn enter_else(&mut self, stack_len: usize) -> Result<()> {
        // Find the matching 'if' label
        let label_idx =
            self.label_stack
                .len()
                .checked_sub(1)
                .ok_or(Error::new(kinds::ExecutionError(
                    "Cannot enter else without a matching if".to_string(),
                )))?;

        // Check that it's actually an if label
        if !self.label_stack[label_idx].is_if {
            return Err(Error::new(kinds::ExecutionError(
                "Attempting to enter 'else' for a non-if block".to_string(),
            )));
        }

        // Find the end of the else block
        let end_pc = self.find_matching_end()?;

        // Jump to the end, skipping the else branch
        self.pc = end_pc;

        Ok(())
    }

    fn exit_block(&mut self, stack: &mut dyn StackBehavior) -> Result<()> {
        // Pop the label
        let label = self
            .label_stack
            .pop()
            .ok_or(Error::new(kinds::ExecutionError(
                "Attempt to exit a non-existent block".to_string(),
            )))?;

        // Set the next label arity based on the remaining label stack
        if let Some(parent_label) = self.label_stack.last() {
            self.label_arity = parent_label.arity;
        } else {
            // If no more labels, set to function arity
            self.label_arity = self.arity;
        }

        Ok(())
    }

    fn branch(&mut self, depth: u32) -> Result<(usize, usize), Error> {
        if depth as usize >= self.label_stack.len() {
            Err(Error::new(kinds::InvalidBranchTargetError { depth }))
        } else {
            // Get the label at the specified depth
            let stack_len = self.label_stack.len();
            let label = &self.label_stack[stack_len - 1 - depth as usize];

            // Return the label's pc and arity
            Ok((label.pc, label.arity))
        }
    }

    // `call` and `call_indirect` are handled by the engine, not directly by frame behavior.
    // The engine pushes a new frame.
    fn call(&mut self, _func_idx: u32, _stack: &mut dyn StackBehavior) -> Result<()> {
        Err(Error::new(kinds::UnimplementedError(
            "call handled by Engine".into(),
        )))
    }

    fn call_indirect(
        &mut self,
        type_idx: u32,
        table_idx: u32,
        entry_idx: u32,
        _stack: &mut dyn StackBehavior,
    ) -> Result<()> {
        // For StacklessFrame, the call_indirect operation is handled by the engine
        // We'll just pass on relevant information to the engine

        // Instead of returning unimplemented error, provide a better error that indicates
        // this is meant to be handled by the StacklessEngine
        Err(Error::new(kinds::ExecutionError(
            "call_indirect in StacklessFrame should be handled by the StacklessEngine".to_string(),
        )))
    }

    // This is primarily managed by enter/exit block/loop/if
    fn set_label_arity(&mut self, arity: usize) {
        self.label_arity = arity;
    }

    fn return_(&mut self) -> Result<(usize, usize), Error> {
        // Return from the current function
        // Return PC should be where to continue execution in the caller
        let return_pc = self.return_pc;

        // Return arity is how many values to transfer to the caller's stack
        let return_arity = self.arity();

        Ok((return_pc, return_arity))
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
            .ok_or_else(|| Error::new(kinds::InvalidLocalIndexError(idx as u32)))
    }

    fn set_local(&mut self, idx: usize, value: Value) -> Result<()> {
        if idx < self.locals.len() {
            self.locals[idx] = value;
            Ok(())
        } else {
            Err(Error::new(kinds::InvalidLocalIndexError(idx as u32)))
        }
    }

    fn get_global(&self, idx: usize, engine: &StacklessEngine) -> Result<Arc<Global>> {
        engine
            .with_instance(self.instance_idx as usize, |instance| {
                instance.get_global(idx)
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn get_global_mut(&mut self, idx: usize) -> Result<wrt_sync::WrtMutexGuard<Value>, Error> {
        // Return an error as this needs to be handled by the engine
        Err(Error::new(kinds::ExecutionError(
            "get_global_mut in StacklessFrame should be handled by the StacklessEngine".to_string(),
        )))
    }

    fn locals_mut(&mut self) -> &mut [Value] {
        self.locals.as_mut_slice()
    }

    fn set_global(&mut self, idx: usize, value: Value, engine: &StacklessEngine) -> Result<()> {
        engine
            .with_instance_mut(self.instance_idx as usize, |instance| {
                // Use instance.set_global directly instead of get_global_mut
                instance.set_global(idx, value)
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn get_memory(&self, idx: usize, engine: &StacklessEngine) -> Result<Arc<dyn MemoryBehavior>> {
        // Use .clone() on the Arc returned from within the closure to ensure proper ownership
        engine
            .with_instance(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory(idx)?;
                Ok(memory.clone())
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn get_memory_mut(
        &mut self,
        idx: usize,
        engine: &StacklessEngine,
    ) -> Result<Arc<dyn MemoryBehavior>> {
        // Use .clone() on the Arc returned from within the closure to ensure proper ownership
        engine
            .with_instance_mut(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory(idx)?;
                Ok(memory.clone())
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn get_table(&self, idx: usize, engine: &StacklessEngine) -> Result<Arc<Table>> {
        engine
            .with_instance(self.instance_idx as usize, |instance| {
                instance.get_table(idx)
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn get_table_mut(&mut self, idx: usize, engine: &StacklessEngine) -> Result<Arc<Table>> {
        // Need mutable engine access to get mutable instance
        engine
            .with_instance_mut(self.instance_idx as usize, |instance| {
                instance.get_table(idx)
            })
            .and_then(|inner_result| Ok(inner_result.clone()))
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

    fn instance_idx(&self) -> u32 {
        self.instance_idx
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

    fn set_return_pc(&mut self, pc: Option<usize>) {
        // If pc is Some, unwrap it, otherwise set to 0 or another default
        self.return_pc = pc.unwrap_or(0);
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    // Memory access methods implementation...
    fn load_i32(&self, addr: usize, _align: u32, engine: &StacklessEngine) -> Result<i32> {
        let mem_idx = 0; // Assuming memory index 0
        engine
            .with_instance(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory(mem_idx)?;
                memory.read_i32(addr as u32)
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn load_i64(&self, addr: usize, _align: u32, engine: &StacklessEngine) -> Result<i64> {
        let mem_idx = 0; // Assuming memory index 0
        engine
            .with_instance(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory(mem_idx)?;
                memory.read_i64(addr as u32)
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn load_f32(&self, addr: usize, _align: u32, engine: &StacklessEngine) -> Result<f32> {
        let mem_idx = 0; // Assuming memory index 0
        engine
            .with_instance(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory(mem_idx)?;
                memory.read_f32(addr as u32)
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn load_f64(&self, addr: usize, _align: u32, engine: &StacklessEngine) -> Result<f64> {
        let mem_idx = 0; // Assuming memory index 0
        engine
            .with_instance(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory(mem_idx)?;
                memory.read_f64(addr as u32)
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn load_i8(&self, addr: usize, _align: u32, engine: &StacklessEngine) -> Result<i8> {
        let mem_idx = 0; // Assuming memory index 0
        engine
            .with_instance(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory(mem_idx)?;
                memory.read_i8(addr as u32)
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn load_u8(&self, addr: usize, _align: u32, engine: &StacklessEngine) -> Result<u8> {
        let mem_idx = 0; // Assuming memory index 0
        engine
            .with_instance(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory(mem_idx)?;
                memory.read_u8(addr as u32)
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn load_i16(&self, addr: usize, _align: u32, engine: &StacklessEngine) -> Result<i16> {
        let mem_idx = 0; // Assuming memory index 0
        engine
            .with_instance(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory(mem_idx)?;
                memory.read_i16(addr as u32)
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn load_u16(&self, addr: usize, _align: u32, engine: &StacklessEngine) -> Result<u16> {
        let mem_idx = 0; // Assuming memory index 0
        engine
            .with_instance(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory(mem_idx)?;
                memory.read_u16(addr as u32)
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn load_v128(&self, addr: usize, _align: u32, engine: &StacklessEngine) -> Result<[u8; 16]> {
        let mem_idx = 0; // Assuming memory index 0
        engine
            .with_instance(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory(mem_idx)?;
                memory.read_v128(addr as u32)
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn store_i32(
        &mut self,
        addr: usize,
        _align: u32,
        value: i32,
        engine: &StacklessEngine,
    ) -> Result<()> {
        engine
            .with_instance_mut(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory_mut(0)?; // Assuming memory index 0
                memory.write_bytes(addr as u32, &value.to_le_bytes())
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn store_i64(
        &mut self,
        addr: usize,
        _align: u32,
        value: i64,
        engine: &StacklessEngine,
    ) -> Result<()> {
        engine
            .with_instance_mut(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory_mut(0)?; // Assuming memory index 0
                memory.write_bytes(addr as u32, &value.to_le_bytes())
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn store_f32(
        &mut self,
        addr: usize,
        _align: u32,
        value: f32,
        engine: &StacklessEngine,
    ) -> Result<()> {
        engine
            .with_instance_mut(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory_mut(0)?; // Assuming memory index 0
                memory.write_bytes(addr as u32, &value.to_le_bytes())
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn store_f64(
        &mut self,
        addr: usize,
        _align: u32,
        value: f64,
        engine: &StacklessEngine,
    ) -> Result<()> {
        engine
            .with_instance_mut(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory_mut(0)?; // Assuming memory index 0
                memory.write_bytes(addr as u32, &value.to_le_bytes())
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn store_i8(
        &mut self,
        addr: usize,
        _align: u32,
        value: i8,
        engine: &StacklessEngine,
    ) -> Result<()> {
        engine
            .with_instance_mut(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory_mut(0)?; // Assuming memory index 0
                memory.write_bytes(addr as u32, &[value as u8])
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn store_u8(
        &mut self,
        addr: usize,
        _align: u32,
        value: u8,
        engine: &StacklessEngine,
    ) -> Result<()> {
        engine
            .with_instance_mut(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory_mut(0)?; // Assuming memory index 0
                memory.write_bytes(addr as u32, &[value])
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn store_i16(
        &mut self,
        addr: usize,
        _align: u32,
        value: i16,
        engine: &StacklessEngine,
    ) -> Result<()> {
        engine
            .with_instance_mut(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory_mut(0)?; // Assuming memory index 0
                memory.write_bytes(addr as u32, &value.to_le_bytes())
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn store_u16(
        &mut self,
        addr: usize,
        _align: u32,
        value: u16,
        engine: &StacklessEngine,
    ) -> Result<()> {
        engine
            .with_instance_mut(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory_mut(0)?;
                memory.write_bytes(addr as u32, &value.to_le_bytes())
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn store_v128(
        &mut self,
        addr: usize,
        _align: u32,
        value: [u8; 16],
        engine: &StacklessEngine,
    ) -> Result<()> {
        engine
            .with_instance_mut(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory_mut(0)?;
                memory.write_bytes(addr as u32, &value)
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn get_function_type(&self, func_idx: u32) -> Result<FuncType> {
        self.module
            .get_function_type(func_idx)
            .cloned()
            .ok_or_else(|| Error::new(kinds::InvalidFunctionIndexError(func_idx as usize)))
    }

    fn memory_size(&self, engine: &StacklessEngine) -> Result<u32> {
        // Delegate to the first memory in the instance
        engine
            .with_instance(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory(0)?;
                Ok(memory.size())
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn memory_grow(&mut self, pages: u32, engine: &StacklessEngine) -> Result<u32> {
        // Delegate to the first memory in the instance
        engine
            .with_instance_mut(self.instance_idx as usize, |instance| {
                let memory = instance.get_memory_mut(0)?;
                memory.grow(pages)
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn table_get(&self, table_idx: u32, idx: u32, engine: &StacklessEngine) -> Result<Value> {
        engine
            .with_instance(self.instance_idx as usize, |instance| {
                let table = instance.get_table(table_idx as usize)?;
                table.get(idx)?.ok_or_else(|| {
                    Error::new(kinds::TableAccessOutOfBoundsError {
                        table_idx: table_idx,
                        element_idx: idx as usize,
                    })
                })
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn table_set(
        &mut self,
        table_idx: u32,
        idx: u32,
        value: Value,
        engine: &StacklessEngine,
    ) -> Result<()> {
        engine
            .with_instance_mut(self.instance_idx as usize, |instance| {
                let table = instance.get_table_mut(table_idx as usize)?;
                table.set(idx, Some(value))
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn table_size(&self, table_idx: u32, engine: &StacklessEngine) -> Result<u32> {
        engine
            .with_instance(self.instance_idx as usize, |instance| {
                let table = instance.get_table(table_idx as usize)?;
                Ok(table.size() as u32) // Convert usize to u32
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn table_grow(
        &mut self,
        table_idx: u32,
        delta: u32,
        value: Value,
        engine: &StacklessEngine,
    ) -> Result<u32> {
        engine
            .with_instance_mut(self.instance_idx as usize, |instance| {
                let table = instance.get_table_mut(table_idx as usize)?;
                table.grow(delta) // Assuming grow takes only delta
                                  // table.grow(delta, Some(value)) // Original incorrect call
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn table_init(
        &mut self,
        table_idx: u32,
        elem_idx: u32,
        dst: u32,
        src: u32,
        n: u32,
        engine: &StacklessEngine,
    ) -> Result<(), Error> {
        // Access the element segment
        let elem_segment = self.get_element_segment(elem_idx, engine)?;

        // Access the table
        let table = self.get_table_mut(table_idx as usize, engine)?;

        // Create a vector of Option<Value> from the function indices in the element segment
        let mut init_values: Vec<Option<Value>> = Vec::with_capacity(n as usize);

        // Get the range of values to initialize
        for i in 0..n {
            let index = (src + i) as usize;
            if index < elem_segment.items.len() {
                let func_idx = elem_segment.items[index];
                init_values.push(Some(Value::FuncRef(Some(func_idx))));
            } else {
                return Err(Error::new(kinds::ElementSegmentOutOfBoundsError(elem_idx)));
            }
        }

        // Initialize the table with the prepared Option<Value> values
        table.init(dst, &init_values)
    }

    fn table_copy(
        &mut self,
        dst_table_idx: u32,
        src_table_idx: u32,
        dst: u32,
        src: u32,
        n: u32,
        engine: &StacklessEngine,
    ) -> Result<()> {
        // Need to implement this without relying on engine.table_copy
        engine
            .with_instance_mut(self.instance_idx as usize, |instance| {
                let dst_table = instance.get_table_mut(dst_table_idx as usize)?;
                let src_table = instance.get_table(src_table_idx as usize)?;

                // Implement table copy logic here
                // For now, just return an unimplemented error
                Err(Error::new(kinds::UnimplementedError(
                    "Table copy not yet implemented".into(),
                )))
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn elem_drop(&mut self, elem_idx: u32, engine: &StacklessEngine) -> Result<(), Error> {
        // Delegate to engine/instance
        engine
            .with_instance_mut(self.instance_idx as usize, |instance| {
                instance.elem_drop(elem_idx)
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn table_fill(
        &mut self,
        table_idx: u32,
        dst: u32,
        val: Value,
        n: u32,
        engine: &StacklessEngine,
    ) -> Result<(), Error> {
        engine
            .with_instance_mut(self.instance_idx as usize, |instance| {
                let table = instance.get_table_mut(table_idx as usize)?;
                table.fill(dst, n, Some(val)) // Corrected order: offset, len, value
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn pop_bool(&mut self, stack: &mut dyn StackBehavior) -> Result<bool, Error> {
        stack.pop_bool()
    }

    fn pop_i32(&mut self, stack: &mut dyn StackBehavior) -> Result<i32, Error> {
        stack.pop_i32()
    }

    fn get_two_tables_mut(
        &mut self,
        idx1: u32,
        idx2: u32,
        engine: &StacklessEngine,
    ) -> Result<(Arc<Table>, Arc<Table>), Error> {
        // Clone the Arc<Table> so they own their data and aren't tied to any closure lifetime
        let table1 = self.get_table_mut(idx1 as usize, engine)?;
        let table2 = self.get_table_mut(idx2 as usize, engine)?;

        // Return the owned Arc<Table> values
        Ok((table1, table2))
    }

    fn set_data_segment(&mut self, _idx: u32, _segment: Arc<Data>) -> Result<(), Error> {
        // Not implemented for StacklessFrame
        Err(Error::new(kinds::UnimplementedError(
            "set_data_segment not implemented for StacklessFrame".to_string(),
        )))
    }

    fn get_element_segment(
        &self,
        elem_idx: u32,
        engine: &StacklessEngine,
    ) -> Result<Arc<Element>, Error> {
        engine
            .with_instance(self.instance_idx as usize, |instance| {
                // Access the element segment using ModuleInstance's method
                let element = instance.get_element_segment(elem_idx)?;
                // Create an Arc of the Element
                Ok(Arc::new(element.clone()))
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn get_data_segment(
        &self,
        data_idx: u32,
        engine: &StacklessEngine,
    ) -> Result<Arc<Data>, Error> {
        // Use the ModuleInstance's get_data method
        engine
            .with_instance(self.instance_idx as usize, |instance| {
                let data = instance.get_data(data_idx)?;
                // Create an Arc of the Data
                Ok(Arc::new(data.clone()))
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn drop_data_segment(&mut self, data_idx: u32, engine: &StacklessEngine) -> Result<(), Error> {
        engine
            .with_instance_mut(self.instance_idx as usize, |instance| {
                instance.drop_data_segment(data_idx)
            })
            .and_then(|inner_result| Ok(inner_result))
    }

    fn push_label(&mut self, label: Label) -> Result<(), Error> {
        // Push the label onto the label stack
        self.label_stack.push(label);
        Ok(())
    }

    fn pop_label(&mut self) -> Result<Label, Error> {
        self.label_stack
            .pop()
            .ok_or_else(|| Error::new(kinds::LabelStackUnderflowError))
    }

    fn get_label(&self, depth: usize) -> Option<&Label> {
        if depth < self.label_stack.len() {
            let idx = self.label_stack.len() - 1 - depth;
            self.label_stack.get(idx)
        } else {
            None
        }
    }
}
