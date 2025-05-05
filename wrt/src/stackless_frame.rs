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
    module::{Data, Element, Function, Module},
    prelude::TypesValue as Value,
    stackless::StacklessEngine,
    types::{BlockType, FuncType, ValueType},
};
use wrt_error::invalid_type;

// Import from helper crates
use wrt_runtime::{Memory, Table};
// Import from prelude for sync primitives
use crate::prelude::MutexGuard;
// Import the bounded collections and the capacity trait
use wrt_types::{BoundedCapacity, BoundedVec, Validatable, VerificationLevel};

/// The maximum number of local variables in a function
const MAX_LOCALS: usize = 1024;

/// The maximum depth of control flow nesting (blocks, loops, if)
const MAX_LABELS: usize = 64;

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
    pub locals: BoundedVec<Value, MAX_LOCALS>,
    /// The index of the module instance this frame belongs to.
    pub instance_idx: u32,
    /// The number of return values expected by the caller of this function.
    pub arity: usize,
    /// The arity (number of stack values expected) of the current control flow block (block, loop, if).
    pub label_arity: usize,
    /// The stack of active control flow labels (blocks, loops, ifs) within this frame.
    pub label_stack: BoundedVec<Label, MAX_LABELS>,
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
                return Err(Error::new(invalid_type(format!(
                    "Function {func_idx}: Argument {} type mismatch: expected {:?}, got {:?}",
                    i,
                    param_type,
                    arg.get_type()
                ))));
            }
        }

        // Create the frame with bounded collections
        let mut locals = BoundedVec::with_verification_level(VerificationLevel::Standard);
        let label_stack = BoundedVec::with_verification_level(VerificationLevel::Standard);

        // Add args to locals
        for arg in args {
            locals.push(arg).map_err(|_| {
                Error::new(kinds::ExecutionError(format!(
                    "Too many local variables (limit: {})",
                    MAX_LOCALS
                )))
            })?;
        }

        Ok(Self {
            module,
            func_idx,
            pc: 0,  // Start at the beginning of the function code
            locals, // Arguments become the initial part of locals
            instance_idx,
            arity: results_len,      // Frame arity is the function's RETURN arity
            label_arity: params_len, // Initial label arity matches function INPUT arity
            label_stack,
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
            None => {
                return Err(Error::new(kinds::FunctionNotFoundError(
                    func_idx.to_string(),
                )))
            }
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
            return Err(Error::new(invalid_type(format!(
                "Function expects {} arguments, but {} provided",
                params_len,
                args.len()
            ))));
        }

        // Create bounded locals collection with the arguments
        let mut locals = BoundedVec::with_verification_level(VerificationLevel::Standard);

        // Add arguments first
        for arg in args {
            locals.push(arg.clone()).map_err(|_| {
                Error::new(kinds::ExecutionError(format!(
                    "Too many local variables (limit: {})",
                    MAX_LOCALS
                )))
            })?;
        }

        // Initialize local variables based on their types
        for &local_type in &function.locals {
            let default_value = match local_type {
                ValueType::I32 => Value::I32(0),
                ValueType::I64 => Value::I64(0),
                ValueType::F32 => Value::F32(0.0),
                ValueType::F64 => Value::F64(0.0),
                ValueType::V128 => Value::V128([0; 16]),
                ValueType::FuncRef => Value::FuncRef(None),
                ValueType::ExternRef => Value::ExternRef(None),
            };

            locals.push(default_value).map_err(|_| {
                Error::new(kinds::ExecutionError(format!(
                    "Too many local variables (limit: {})",
                    MAX_LOCALS
                )))
            })?;
        }

        // Create bounded label stack
        let label_stack = BoundedVec::with_verification_level(VerificationLevel::Standard);

        Ok(Self {
            module,
            func_idx,
            pc: 0,
            locals,
            instance_idx,
            arity: results_len,      // Frame arity is the function's RETURN arity
            label_arity: params_len, // Initial label arity matches function INPUT arity
            label_stack,
            return_pc: 0, // Will be set by the caller
        })
    }

    /// Validates the frame's integrity, checking all bounded collections.
    pub fn validate(&self) -> Result<()> {
        // Validate the local variables collection
        self.locals.validate()?;

        // Validate the label stack collection
        self.label_stack.validate()?;

        // Validate function index doesn't exceed module's function count
        if (self.func_idx as usize) >= self.module.functions.len() {
            return Err(Error::new(kinds::InvalidFunctionIndexError(
                self.func_idx as usize,
            )));
        }

        Ok(())
    }

    /// Gets the function definition associated with this frame.
    pub fn get_function(&self) -> Result<&Function> {
        self.module
            .get_function(self.func_idx)
            .ok_or_else(|| Error::new(kinds::FunctionNotFoundError(self.func_idx.to_string())))
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

        Err(Error::new(kinds::ExecutionError(format!(
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
        Err(Error::new(kinds::ExecutionError(format!(
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

    /// Get a label at the specified depth (0 = innermost)
    pub fn get_label_at_depth(&self, depth: usize) -> Option<&Label> {
        if depth < self.label_stack.len() {
            let idx = self.label_stack.len() - 1 - depth;
            self.label_stack.get(idx)
        } else {
            None
        }
    }
}

// Implement the behavior traits

impl StackBehavior for StacklessFrame {
    // NOTE: StackBehavior for StacklessFrame often manipulates `locals` directly
    // when used within the engine's step function, as there isn't a separate operand stack.
    // Be cautious when interpreting these methods outside that context.

    fn push(&mut self, value: Value) -> Result<()> {
        self.locals.push(value).map_err(|_| {
            Error::new(kinds::ExecutionError(format!(
                "Stack overflow, maximum locals: {}",
                MAX_LOCALS
            )))
        })
    }

    fn pop(&mut self) -> Result<Value> {
        self.locals
            .pop()
            .ok_or_else(|| Error::new(kinds::StackUnderflowError()))
    }

    fn peek(&self) -> Result<&Value> {
        self.locals
            .get(self.locals.len().checked_sub(1).unwrap_or(0))
            .ok_or_else(|| Error::new(kinds::StackUnderflowError()))
    }

    fn peek_mut(&mut self) -> Result<&mut Value> {
        let len = self.locals.len();
        if len == 0 {
            return Err(Error::new(kinds::StackUnderflowError()));
        }
        Ok(self.locals.get_mut(len - 1).unwrap())
    }

    fn values(&self) -> &[Value] {
        // This is safe because the BoundedVec internally stores a Vec and implements AsRef<[T]>
        self.locals.as_ref()
    }

    fn values_mut(&mut self) -> &mut [Value] {
        // Shortcut to get the entire slice for safety
        self.locals.as_mut()
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
            .ok_or_else(|| Error::new(kinds::StackUnderflowError()))
    }

    fn get_label(&self, depth: usize) -> Option<&Label> {
        self.get_label_at_depth(depth)
    }

    fn push_n(&mut self, _values: &[Value]) {
        // Implementation omitted
    }

    fn pop_n(&mut self, _n: usize) -> Vec<Value> {
        // Implementation omitted
        Vec::new()
    }

    fn pop_frame_label(&mut self) -> Result<Label, Error> {
        // Not supported by frames, should be handled by stack
        Err(Error::new(kinds::NotImplementedError(
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

    fn exit_block(&mut self, _stack: &mut dyn StackBehavior) -> Result<(), Error> {
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
        let depth = depth as usize;

        if depth >= self.label_stack.len() {
            return Err(Error::new(kinds::InvalidLocalIndexError(depth as u32)));
        }

        let label_idx = self.label_stack.len() - 1 - depth;
        let label = self
            .label_stack
            .get(label_idx)
            .ok_or_else(|| Error::new(kinds::InvalidLocalIndexError(depth as u32)))?;

        // The rest of the method remains unchanged
        let target_pc = label.continuation_pc;
        let expected_values = label.arity;

        Ok((target_pc, expected_values))
    }

    // `call` and `call_indirect` are handled by the engine, not directly by frame behavior.
    // The engine pushes a new frame.
    fn call(&mut self, _func_idx: u32, _stack: &mut dyn StackBehavior) -> Result<()> {
        Err(Error::new(kinds::NotImplementedError(
            "call handled by Engine".into(),
        )))
    }

    fn call_indirect(
        &mut self,
        _type_idx: u32,
        _table_idx: u32,
        _entry_idx: u32,
        stack: &mut dyn StackBehavior,
    ) -> Result<(), Error> {
        Err(Error::new(kinds::NotImplementedError(
            "call_indirect not implemented for StacklessFrame yet".to_string(),
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
    fn push_label(&mut self, label: Label) -> Result<(), Error> {
        self.label_stack.push(label).map_err(|e| {
            Error::new(kinds::ExecutionError(format!(
                "Label stack overflow error: {:?}",
                e
            )))
        })?;
        Ok(())
    }

    fn pop_label(&mut self) -> Result<Label, Error> {
        self.label_stack
            .pop()
            .ok_or_else(|| Error::new(kinds::StackUnderflowError()))
    }

    fn get_label(&self, depth: usize) -> Option<&Label> {
        self.get_label_at_depth(depth)
    }

    fn locals(&mut self) -> &mut Vec<Value> {
        // We need to adapt the BoundedVec to work with the existing interface
        // that expects a Vec. This is a compatibility layer.
        self.locals.as_mut_vec()
    }

    fn get_local(&self, idx: usize) -> Result<Value> {
        self.locals
            .get(idx)
            .cloned()
            .ok_or_else(|| Error::new(kinds::InvalidLocalIndexError(idx as u32)))
    }

    fn set_local(&mut self, idx: usize, value: Value) -> Result<()> {
        if let Some(local) = self.locals.get_mut(idx) {
            *local = value;
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

    fn get_global_mut(&mut self, _idx: usize) -> Result<wrt_sync::WrtMutexGuard<Value>, Error> {
        Err(Error::new(kinds::NotImplementedError(
            "get_global_mut not implemented for StacklessFrame".to_string(),
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

    fn get_memory(&self, idx: usize, engine: &StacklessEngine) -> Result<Arc<Memory>> {
        engine
            .with_instance(self.instance_idx as usize, |instance| {
                instance.get_memory(idx).map_err(|e| {
                    Error::new(kinds::ExecutionError(format!(
                        "Error getting memory at index {idx}: {e}"
                    )))
                })
            })
            .map_err(|e| {
                Error::new(kinds::ExecutionError(format!(
                    "Error accessing instance {}: {e}",
                    self.instance_idx
                )))
            })?
    }

    fn get_memory_mut(&mut self, idx: usize, engine: &StacklessEngine) -> Result<Arc<Memory>> {
        let instance_idx = self.instance_idx as usize;
        engine
            .with_instance(instance_idx, |instance| {
                instance.get_memory(idx).map_err(|e| {
                    Error::new(kinds::ExecutionError(format!(
                        "Error getting mutable memory at index {idx}: {e}"
                    )))
                })
            })
            .map_err(|e| {
                Error::new(kinds::ExecutionError(format!(
                    "Error accessing instance {instance_idx}: {e}"
                )))
            })?
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
        // This is not correct, but we need to adapt to the interface
        // In a future PR, we should change this method to return a generic collection
        unimplemented!("Cannot get mutable Vec from BoundedVec in label_stack")
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
        let memory = self.get_memory(mem_idx, engine)?;
        memory.read_i32(addr as u32)
    }

    fn load_i64(&self, addr: usize, _align: u32, engine: &StacklessEngine) -> Result<i64> {
        let mem_idx = 0; // Assuming memory index 0
        let memory = self.get_memory(mem_idx, engine)?;
        memory.read_i64(addr as u32)
    }

    fn load_f32(&self, addr: usize, _align: u32, engine: &StacklessEngine) -> Result<f32> {
        let mem_idx = 0; // Assuming memory index 0
        let memory = self.get_memory(mem_idx, engine)?;
        memory.read_f32(addr as u32)
    }

    fn load_f64(&self, addr: usize, _align: u32, engine: &StacklessEngine) -> Result<f64> {
        let mem_idx = 0; // Assuming memory index 0
        let memory = self.get_memory(mem_idx, engine)?;
        memory.read_f64(addr as u32)
    }

    fn load_i8(&self, addr: usize, _align: u32, engine: &StacklessEngine) -> Result<i8> {
        let mem_idx = 0; // Assuming memory index 0
        let memory = self.get_memory(mem_idx, engine)?;
        memory.read_i8(addr as u32)
    }

    fn load_u8(&self, addr: usize, _align: u32, engine: &StacklessEngine) -> Result<u8> {
        let mem_idx = 0; // Assuming memory index 0
        let memory = self.get_memory(mem_idx, engine)?;
        memory.read_u8(addr as u32)
    }

    fn load_i16(&self, addr: usize, _align: u32, engine: &StacklessEngine) -> Result<i16> {
        let mem_idx = 0; // Assuming memory index 0
        let memory = self.get_memory(mem_idx, engine)?;
        memory.read_i16(addr as u32)
    }

    fn load_u16(&self, addr: usize, _align: u32, engine: &StacklessEngine) -> Result<u16> {
        let mem_idx = 0; // Assuming memory index 0
        let memory = self.get_memory(mem_idx, engine)?;
        memory.read_u16(addr as u32)
    }

    fn load_v128(&self, addr: usize, _align: u32, engine: &StacklessEngine) -> Result<[u8; 16]> {
        let mem_idx = 0; // Assuming memory index 0
        let memory = self.get_memory(mem_idx, engine)?;
        memory.read_v128(addr as u32)
    }

    fn store_i32(
        &mut self,
        addr: usize,
        _align: u32,
        value: i32,
        engine: &StacklessEngine,
    ) -> Result<()> {
        let memory = self.get_memory(0, engine)?;
        let memory_clone = memory.clone();
        if let Some(mut_memory) = Arc::get_mut(&mut memory_clone.clone()) {
            mut_memory.write_i32(addr as u32, value)
        } else {
            Err(Error::new(kinds::ExecutionError(
                "Cannot get mutable access to memory for writing i32".to_string(),
            )))
        }
    }

    fn store_i64(
        &mut self,
        addr: usize,
        _align: u32,
        value: i64,
        engine: &StacklessEngine,
    ) -> Result<()> {
        let memory = self.get_memory(0, engine)?;
        let memory_clone = memory.clone();
        if let Some(mut_memory) = Arc::get_mut(&mut memory_clone.clone()) {
            mut_memory.write_i64(addr as u32, value)
        } else {
            Err(Error::new(kinds::ExecutionError(
                "Cannot get mutable access to memory for writing i64".to_string(),
            )))
        }
    }

    fn store_f32(
        &mut self,
        addr: usize,
        _align: u32,
        value: f32,
        engine: &StacklessEngine,
    ) -> Result<()> {
        let memory = self.get_memory(0, engine)?;
        let memory_clone = memory.clone();
        if let Some(mut_memory) = Arc::get_mut(&mut memory_clone.clone()) {
            mut_memory.write_f32(addr as u32, value)
        } else {
            Err(Error::new(kinds::ExecutionError(
                "Cannot get mutable access to memory for writing f32".to_string(),
            )))
        }
    }

    fn store_f64(
        &mut self,
        addr: usize,
        _align: u32,
        value: f64,
        engine: &StacklessEngine,
    ) -> Result<()> {
        let memory = self.get_memory(0, engine)?;
        let memory_clone = memory.clone();
        if let Some(mut_memory) = Arc::get_mut(&mut memory_clone.clone()) {
            mut_memory.write_f64(addr as u32, value)
        } else {
            Err(Error::new(kinds::ExecutionError(
                "Cannot get mutable access to memory for writing f64".to_string(),
            )))
        }
    }

    fn store_i8(
        &mut self,
        addr: usize,
        _align: u32,
        value: i8,
        engine: &StacklessEngine,
    ) -> Result<()> {
        let memory = self.get_memory(0, engine)?;
        let memory_clone = memory.clone();
        if let Some(mut_memory) = Arc::get_mut(&mut memory_clone.clone()) {
            mut_memory.write_u8(addr as u32, value as u8)
        } else {
            Err(Error::new(kinds::ExecutionError(
                "Cannot get mutable access to memory for writing i8".to_string(),
            )))
        }
    }

    fn store_u8(
        &mut self,
        addr: usize,
        _align: u32,
        value: u8,
        engine: &StacklessEngine,
    ) -> Result<()> {
        let memory = self.get_memory(0, engine)?;
        let memory_clone = memory.clone();
        if let Some(mut_memory) = Arc::get_mut(&mut memory_clone.clone()) {
            mut_memory.write_u8(addr as u32, value)
        } else {
            Err(Error::new(kinds::ExecutionError(
                "Cannot get mutable access to memory for writing u8".to_string(),
            )))
        }
    }

    fn store_i16(
        &mut self,
        addr: usize,
        _align: u32,
        value: i16,
        engine: &StacklessEngine,
    ) -> Result<()> {
        let memory = self.get_memory(0, engine)?;
        let memory_clone = memory.clone();
        if let Some(mut_memory) = Arc::get_mut(&mut memory_clone.clone()) {
            mut_memory.write_u16(addr as u32, value as u16)
        } else {
            Err(Error::new(kinds::ExecutionError(
                "Cannot get mutable access to memory for writing i16".to_string(),
            )))
        }
    }

    fn store_u16(
        &mut self,
        addr: usize,
        _align: u32,
        value: u16,
        engine: &StacklessEngine,
    ) -> Result<()> {
        let memory = self.get_memory(0, engine)?;
        let memory_clone = memory.clone();
        if let Some(mut_memory) = Arc::get_mut(&mut memory_clone.clone()) {
            mut_memory.write_u16(addr as u32, value)
        } else {
            Err(Error::new(kinds::ExecutionError(
                "Cannot get mutable access to memory for writing u16".to_string(),
            )))
        }
    }

    fn store_v128(
        &mut self,
        addr: usize,
        _align: u32,
        value: [u8; 16],
        engine: &StacklessEngine,
    ) -> Result<()> {
        let memory = self.get_memory(0, engine)?;
        let memory_clone = memory.clone();
        if let Some(mut_memory) = Arc::get_mut(&mut memory_clone.clone()) {
            mut_memory.write_v128(addr as u32, value)
        } else {
            Err(Error::new(kinds::ExecutionError(
                "Cannot get mutable access to memory for writing v128".to_string(),
            )))
        }
    }

    fn get_function_type(&self, func_idx: u32) -> Result<FuncType> {
        self.module
            .get_function_type(func_idx)
            .cloned()
            .ok_or_else(|| Error::new(kinds::InvalidFunctionIndexError(func_idx as usize)))
    }

    fn memory_size(&self, engine: &StacklessEngine) -> Result<u32> {
        // Get memory directly and use its size method
        let memory = self.get_memory(0, engine)?;
        Ok(memory.size())
    }

    fn memory_grow(&mut self, pages: u32, engine: &StacklessEngine) -> Result<u32> {
        let memory = self.get_memory(0, engine)?;
        // Store the previous size for return value
        let previous_size = memory.size();

        // Clone the memory to get a mutable version
        let memory_clone = memory.clone();

        // Try to get mutable access to the Arc
        if let Some(mut_memory) = Arc::get_mut(&mut memory_clone.clone()) {
            // Grow the memory
            mut_memory.grow(pages)?;

            // Return the previous size
            Ok(previous_size)
        } else {
            // Return an error if we can't get mutable access
            Err(Error::new(kinds::ExecutionError(
                "Cannot get mutable access to memory for growing".to_string(),
            )))
        }
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
                // Use the new table_set method
                instance.table_set(table_idx as usize, idx, Some(value))
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
                // Use the new table_grow method
                instance.table_grow(table_idx as usize, delta, value)
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
                Err(Error::new(kinds::NotImplementedError(
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

    fn set_data_segment(&mut self, idx: u32, segment: Arc<Data>) -> Result<(), Error> {
        // Not implemented for StacklessFrame
        Err(Error::new(kinds::NotImplementedError(
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
}

// Add AsRef<[u8]> implementation for StacklessFrame to work with BoundedVec
impl AsRef<[u8]> for StacklessFrame {
    fn as_ref(&self) -> &[u8] {
        // Create a static representation of the frame using its critical fields
        // This is a simplification for checksum purposes only
        static mut BUFFER: [u8; 32] = [0; 32];

        unsafe {
            // Pack the critical fields into the buffer
            let func_idx_bytes = self.func_idx.to_le_bytes();
            let pc_bytes = self.pc.to_le_bytes();
            let instance_idx_bytes = self.instance_idx.to_le_bytes();
            let arity_bytes = self.arity.to_le_bytes();

            // Copy bytes into buffer
            BUFFER[0..4].copy_from_slice(&func_idx_bytes);
            BUFFER[4..12].copy_from_slice(&pc_bytes);
            BUFFER[12..16].copy_from_slice(&instance_idx_bytes);
            BUFFER[16..24].copy_from_slice(&arity_bytes);

            // Return a slice to the buffer
            &BUFFER[..]
        }
    }
}

// Implement functions for BoundedVec that mimic Vec functions
impl StacklessFrame {
    fn get_label_at_depth(&self, depth: usize) -> Option<&Label> {
        if depth < self.label_stack.len() {
            let idx = self.label_stack.len() - 1 - depth;
            self.label_stack.get(idx)
        } else {
            None
        }
    }
}
