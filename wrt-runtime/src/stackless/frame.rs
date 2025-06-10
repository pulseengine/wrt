// Stackless frame implementation without unsafe code
//! Stackless function activation frame

extern crate alloc;

use core::fmt::Debug;
#[cfg(feature = "std")]
use std::vec;

// Imports from wrt crates
// Instructions are now in wrt-foundation
use wrt_foundation::types::Instruction;
use crate::types::{ValueStackVec, LocalsVec};
use wrt_error::{codes, Error, ErrorCategory};
use wrt_foundation::values::FuncRef;
use wrt_foundation::{
    safe_memory::SafeSlice, // Added SafeSlice
    values::{FloatBits32, FloatBits64}, // Added for floating-point values
    BlockType,
    BoundedCapacity,
    FuncType,
    Validatable,
    Value,
    ValueType,
    VerificationLevel,
}; // Added FuncRef
// Re-export Label for convenience if it's commonly used with frames
pub use wrt_instructions::control_ops::BranchTarget as Label;

// Internal imports
use super::engine::StacklessEngine;
use crate::prelude::*;
use crate::memory_adapter::StdMemoryProvider;
use crate::{
    global::Global,
    memory::Memory,
    module::{Data, Element, Function, Module}, // Module is already in prelude
    module_instance::ModuleInstance,
    stackless::StacklessStack, // Added StacklessStack
    table::Table,
};

// Import format! macro for string formatting
#[cfg(feature = "std")]
use std::format;
#[cfg(not(feature = "std"))]
use alloc::format;

/// Defines the behavior of a function activation frame in the stackless engine.
pub trait FrameBehavior {
    /// Returns the current program counter (instruction offset within the
    /// function body).
    fn pc(&self) -> usize;

    /// Returns a mutable reference to the program counter.
    fn pc_mut(&mut self) -> &mut usize;

    /// Returns a slice of the local variables for the current frame.
    /// This includes function arguments followed by declared local variables.
    fn locals(&self) -> &[Value];

    /// Returns a mutable slice of the local variables.
    fn locals_mut(&mut self) -> &mut [Value];

    /// Returns a reference to the module instance this frame belongs to.
    fn module_instance(&self) -> &Arc<ModuleInstance>;

    /// Returns the index of the function this frame represents.
    fn function_index(&self) -> u32;

    /// Returns the type (signature) of the function this frame represents.
    fn function_type(&self) -> &FuncType<StdMemoryProvider>;

    /// Returns the arity (number of return values) of the function.
    fn arity(&self) -> usize;

    /// Advances execution by one instruction.
    ///
    /// # Arguments
    ///
    /// * `engine`: The stackless engine, providing access to global state like
    ///   the value stack and call stack.
    ///
    /// # Returns
    ///
    /// * `Ok(ControlFlow)`: Indicates the outcome of the instruction execution,
    ///   e.g., proceed to next instruction, call, return.
    /// * `Err(Error)`: If an error occurs during execution.
    fn step(&mut self, engine: &mut StacklessEngine) -> Result<ControlFlow>; // ControlFlow to be defined
}

/// Represents the control flow outcome of an instruction's execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlFlow {
    /// Continue to the next instruction in the current frame.
    Next,
    /// A function call has been made. A new frame will be pushed.
    Call { func_idx: u32, inputs: ValueStackVec }, // Simplified for now
    /// The current function is returning. The current frame will be popped.
    Return { values: ValueStackVec },
    /// A branch to a given PC offset within the current function.
    Branch(usize),
    /// Trap / Unreachable instruction.
    Trap(Error),
    /// A tail call that replaces the current frame (WebAssembly 2.0).
    TailCall(u32), // function index
}

/// Stackless function activation frame.
#[derive(Debug, Clone)]
pub struct StacklessFrame {
    /// Program counter: offset into the function's instruction stream.
    pc: usize,
    /// Local variables (includes arguments).
    locals: LocalsVec, // Simplified from SafeSlice to avoid lifetime issues
    /// Reference to the module instance.
    module_instance: Arc<ModuleInstance>,
    /// Index of the function in the module.
    func_idx: u32,
    /// Type of the function.
    func_type: FuncType<StdMemoryProvider>,
    /// Arity of the function (number of result values).
    arity: usize,
    /// Block depths for control flow.
    #[cfg(feature = "std")]
    block_depths: Vec<BlockContext>, // Use standard Vec for internal state
    #[cfg(all(not(feature = "std"), not(feature = "std")))]
    block_depths: [Option<BlockContext>; 16], // Fixed array for no_std
}

/// Context for a control flow block (block, loop, if).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct BlockContext {
    /// The type of the block.
    block_type: BlockType,
    /// Program counter to jump to when exiting this block (e.g., `end` of
    /// block/if/loop).
    end_pc: usize,
    /// Program counter for the `else` branch of an `if` block.
    else_pc: Option<usize>,
    /// Stack depth before entering the block, to know how many values to
    /// pop/truncate.
    stack_depth_before: usize,
    /// Value stack depth before parameters were pushed (for block/loop results)
    exec_stack_values_depth_before_params: usize,
    /// Arity of the block (number of result values it's expected to push).
    arity: usize,
}

/// Helper functions for stack operations
impl StacklessFrame {
    /// Helper function to pop a value from the execution stack and handle the Result<Option<T>, E> return type
    fn pop_value(engine: &mut StacklessEngine) -> Result<Value> {
        match engine.exec_stack.values.pop() {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Err(Error::new(
                ErrorCategory::Runtime,
                codes::STACK_UNDERFLOW,
                "Stack underflow"
            )),
            Err(_) => Err(Error::new(
                ErrorCategory::Runtime,
                codes::STACK_UNDERFLOW,
                "Stack operation error"
            )),
        }
    }

    /// Helper function to pop an i32 value from the execution stack
    fn pop_i32(engine: &mut StacklessEngine) -> Result<i32> {
        let value = Self::pop_value(engine)?;
        match value {
            Value::I32(i) => Ok(i),
            _ => Err(Error::new(
                ErrorCategory::Runtime,
                codes::TYPE_MISMATCH_ERROR,
                "Expected i32 value"
            )),
        }
    }

    /// Helper function to pop an i64 value from the execution stack
    fn pop_i64(engine: &mut StacklessEngine) -> Result<i64> {
        let value = Self::pop_value(engine)?;
        match value {
            Value::I64(i) => Ok(i),
            _ => Err(Error::new(
                ErrorCategory::Runtime,
                codes::TYPE_MISMATCH_ERROR,
                "Expected i64 value"
            )),
        }
    }

    /// Helper function to pop an f32 value from the execution stack
    fn pop_f32(engine: &mut StacklessEngine) -> Result<f32> {
        let value = Self::pop_value(engine)?;
        match value {
            Value::F32(f) => Ok(f.value()),
            _ => Err(Error::new(
                ErrorCategory::Runtime,
                codes::TYPE_MISMATCH_ERROR,
                "Expected f32 value"
            )),
        }
    }

    /// Helper function to pop an f64 value from the execution stack
    fn pop_f64(engine: &mut StacklessEngine) -> Result<f64> {
        let value = Self::pop_value(engine)?;
        match value {
            Value::F64(f) => Ok(f.value()),
            _ => Err(Error::new(
                ErrorCategory::Runtime,
                codes::TYPE_MISMATCH_ERROR,
                "Expected f64 value"
            )),
        }
    }
}

impl StacklessFrame {
    /// Creates a new stackless function frame.
    ///
    /// # Arguments
    ///
    /// * `func_ref`: A reference to the function to be called.
    /// * `module_instance`: The module instance this function belongs to.
    /// * `invocation_inputs`: Values passed as arguments to this function call.
    /// * `max_locals`: Maximum number of locals expected (for SafeSlice
    /// Binary std/no_std choice
    pub fn new(
        func_ref: FuncRef,
        module_instance: Arc<ModuleInstance>,
        invocation_inputs: &[Value], // Changed to slice
        max_locals: usize,           // Example: pass from engine config or calculate
    ) -> Result<Self> {
        let func_idx = func_ref.index;
        let func_type = module_instance.function_type(func_idx)?;

        let mut locals_vec = wrt_foundation::bounded::BoundedVec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default())?;
        for value in invocation_inputs.iter() {
            locals_vec.push(value.clone())?;
        }

        // Append default values for declared locals
        if let Some(function_body) = module_instance.module().functions.get(func_idx as usize) {
            for local_entry in &function_body.locals {
                // local_entry is (count, ValueType) in the Module's Function struct
                // Assuming Function struct in module.rs has: pub locals: Vec<(u32, ValueType)>,
                let count = local_entry.0;
                let val_type = local_entry.1;
                for _ in 0..count {
                    locals_vec.push(Value::default_for_type(&val_type));
                }
            }
        } else {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::FUNCTION_NOT_FOUND,
                "Function body not found",
            ));
        }

        let locals = locals_vec;

        if locals.len() > max_locals {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::INVALID_STATE,
                "Too many locals for configured max_locals",
            ));
        }

        Ok(Self {
            pc: 0,
            locals,
            module_instance,
            func_idx,
            arity: func_type.results.len(),
            func_type,
            #[cfg(feature = "std")]
            block_depths: Vec::new(),
            #[cfg(all(not(feature = "std"), not(feature = "std")))]
            block_depths: [None; 16],
        })
    }

    // Helper to get the actual function body from the module instance
    fn function_body(&self) -> Result<&crate::module::Function> {
        self.module_instance.module().functions.get(self.func_idx as usize).map_err(|_| {
            Error::new(
                ErrorCategory::Runtime,
                codes::FUNCTION_NOT_FOUND,
                "Function body not found for index",
            )
        })
    }
}

impl FrameBehavior for StacklessFrame {
    fn pc(&self) -> usize {
        self.pc
    }

    fn pc_mut(&mut self) -> &mut usize {
        &mut self.pc
    }

    fn locals(&self) -> &[Value] {
        &self.locals
    }

    fn locals_mut(&mut self) -> &mut [Value] {
        &mut self.locals
    }

    fn module_instance(&self) -> &Arc<ModuleInstance> {
        &self.module_instance
    }

    fn function_index(&self) -> u32 {
        self.func_idx
    }

    fn function_type(&self) -> &FuncType<StdMemoryProvider> {
        &self.func_type
    }

    fn arity(&self) -> usize {
        self.arity
    }

    fn step(&mut self, engine: &mut StacklessEngine) -> Result<ControlFlow> {
        let func_body = self.function_body()?;
        let instructions = &func_body.body; // Function struct has `body` field, not `code`

        if self.pc >= instructions.len() {
            // If PC is at or beyond the end, and it's not a trap/return already handled,
            // it implies a fallthrough return for a void function or a missing explicit
            // return.
            if self.arity == 0 {
                // Implicit return for void function
                #[cfg(feature = "std")]
                return Ok(ControlFlow::Return { values: ValueStackVec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap() });
                #[cfg(not(feature = "std"))]
                return Ok(ControlFlow::Return { 
                    values: wrt_foundation::bounded::BoundedVec::new(
                        wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
                    ).unwrap() 
                });
            } else {
                return Err(Error::new(
                    ErrorCategory::Runtime,
                    codes::RUNTIME_ERROR,
                    "Function ended without returning expected values",
                ));
            }
        }

        let instruction = &instructions[self.pc];
        self.pc += 1;

        // --- Execute Instruction ---
        // This is where the large match statement for all instructions will go.
        // For now, a placeholder.
        match instruction {
            Instruction::Unreachable => Ok(ControlFlow::Trap(Error::new(
                ErrorCategory::Runtime,
                codes::RUNTIME_TRAP_ERROR,
                "Unreachable instruction executed",
            ))),
            Instruction::Nop => Ok(ControlFlow::Next),
            Instruction::Block { block_type_idx } => {
                // Enter a new block scope
                let block_context = BlockContext {
                    block_type: BlockType::Empty, // Simplified for now - should resolve block_type_idx
                    end_pc: 0, // Will be set when we encounter the matching End instruction
                    else_pc: None,
                    stack_depth_before: engine.exec_stack.values.len(),
                    exec_stack_values_depth_before_params: engine.exec_stack.values.len(),
                    arity: 0, // Should be determined from block type
                };
                
                #[cfg(feature = "std")]
                self.block_depths.push(block_context);
                #[cfg(all(not(feature = "std"), not(feature = "std")))]
                {
                    // Find the first available slot in fixed array
                    let mut found = false;
                    for slot in &mut self.block_depths {
                        if slot.is_none() {
                            *slot = Some(block_context);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        return Err(Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Too many nested blocks"));
                    }
                }
                
                Ok(ControlFlow::Next)
            }
            Instruction::Loop { block_type_idx } => {
                // Enter a new loop scope - branches target the loop start (current PC)
                let block_context = BlockContext {
                    block_type: BlockType::Empty, // Simplified for now - should resolve block_type_idx
                    end_pc: 0, // Will be set when we encounter the matching End instruction
                    else_pc: None,
                    stack_depth_before: engine.exec_stack.values.len(),
                    exec_stack_values_depth_before_params: engine.exec_stack.values.len(),
                    arity: 0, // Should be determined from block type
                };
                
                #[cfg(feature = "std")]
                self.block_depths.push(block_context);
                #[cfg(all(not(feature = "std"), not(feature = "std")))]
                {
                    // Find the first available slot in fixed array
                    let mut found = false;
                    for slot in &mut self.block_depths {
                        if slot.is_none() {
                            *slot = Some(block_context);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        return Err(Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Too many nested blocks"));
                    }
                }
                
                Ok(ControlFlow::Next)
            }
            Instruction::If { block_type_idx } => {
                // Pop condition from stack
                let condition_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let condition = match condition_val {
                    Value::I32(val) => val != 0,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "If condition not i32")),
                };
                
                // Enter If block scope
                let block_context = BlockContext {
                    block_type: BlockType::Empty, // Simplified for now - should resolve block_type_idx
                    end_pc: 0, // Will be set when we encounter the matching End instruction
                    else_pc: None, // Will be set when we encounter Else instruction
                    stack_depth_before: engine.exec_stack.values.len(),
                    exec_stack_values_depth_before_params: engine.exec_stack.values.len(),
                    arity: 0, // Should be determined from block type
                };
                
                #[cfg(feature = "std")]
                self.block_depths.push(block_context);
                #[cfg(all(not(feature = "std"), not(feature = "std")))]
                {
                    // Find the first available slot in fixed array
                    let mut found = false;
                    for slot in &mut self.block_depths {
                        if slot.is_none() {
                            *slot = Some(block_context);
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        return Err(Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Too many nested blocks"));
                    }
                }
                
                if condition {
                    // Continue to then branch
                    Ok(ControlFlow::Next)
                } else {
                    // Jump to else or end - for now, we'll need to scan forward to find it
                    // This is a simplified implementation
                    todo!("If false branch - need to implement else/end scanning")
                }
            }
            Instruction::Else => {
                // TODO: Jump to end of current If block's 'then' part.
                // let current_block = self.block_depths.last().ok_or_else(...)?;
                // self.pc = current_block.end_pc;
                todo!("Else instruction")
            }
            Instruction::End => {
                // Check if this is the end of the function itself or a nested block
                let has_blocks = {
                    #[cfg(feature = "std")]
                    { !self.block_depths.is_empty() }
                    #[cfg(all(not(feature = "std"), not(feature = "std")))]
                    { self.block_depths.iter().any(|slot| slot.is_some()) }
                };
                
                if !has_blocks {
                    // This 'end' corresponds to the function body's implicit block.
                    // Values for return should be on the stack matching self.arity.
                    #[cfg(feature = "std")]
                    let mut return_values = ValueStackVec::with_capacity(self.arity);
                    #[cfg(not(feature = "std"))]
                    let mut return_values = wrt_foundation::bounded::BoundedVec::new(
                        wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
                    ).unwrap();
                    for _ in 0..self.arity {
                        return_values.push(engine.exec_stack.values.pop().map_err(|e| {
                            Error::new(
                                ErrorCategory::Runtime,
                                codes::STACK_UNDERFLOW,
                                "Stack operation error",
                            )
                        })?);
                    }
                    return_values.reverse(); // Values are popped in reverse order
                    return Ok(ControlFlow::Return { values: return_values });
                } else {
                    // Pop the most recent block context
                    #[cfg(feature = "std")]
                    {
                        let _block_context = self.block_depths.pop().ok_or_else(|| {
                            Error::new(ErrorCategory::Runtime, codes::INVALID_STATE, "No block to end")
                        })?;
                    }
                    #[cfg(all(not(feature = "std"), not(feature = "std")))]
                    {
                        // Find and clear the last occupied slot
                        let mut found = false;
                        for slot in self.block_depths.iter_mut().rev() {
                            if slot.is_some() {
                                *slot = None;
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            return Err(Error::new(ErrorCategory::Runtime, codes::INVALID_STATE, "No block to end"));
                        }
                    }
                    
                    Ok(ControlFlow::Next) // Continue after ending the block
                }
            }
            Instruction::Br(label_idx) => {
                // Branch to the specified label (relative depth)
                // For now, simplified implementation - need to implement proper label resolution
                Ok(ControlFlow::Branch(label_idx as usize))
            }
            Instruction::BrIf(label_idx) => {
                // Pop condition from stack
                let condition_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let condition = match condition_val {
                    Value::I32(val) => val != 0,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "BrIf condition not i32")),
                };
                
                if condition {
                    // Branch to the specified label
                    Ok(ControlFlow::Branch(label_idx as usize))
                } else {
                    // Continue to next instruction
                    Ok(ControlFlow::Next)
                }
            }
            // ... other control flow instructions ...
            Instruction::Return => {
                #[cfg(feature = "std")]
                let mut return_values = ValueStackVec::with_capacity(self.arity);
                #[cfg(not(feature = "std"))]
                let mut return_values = wrt_foundation::bounded::BoundedVec::new_with_provider(
                    wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
                ).unwrap();
                for _ in 0..self.arity {
                    return_values.push(engine.exec_stack.values.pop().map_err(|e| {
                        Error::new(
                            ErrorCategory::Runtime,
                            codes::STACK_UNDERFLOW,
                            "Stack operation error",
                        )
                    })?);
                }
                return_values.reverse();
                Ok(ControlFlow::Return { values: return_values })
            }
            Instruction::Call(func_idx_val) => {
                // Get the target function type to know how many arguments to pop
                let target_func_type = self.module_instance.function_type(func_idx_val)?;
                #[cfg(feature = "std")]
                let mut args = ValueStackVec::with_capacity(target_func_type.params.len());
                #[cfg(not(feature = "std"))]
                let mut args = wrt_foundation::bounded::BoundedVec::new_with_provider(
                    wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
                ).unwrap();
                
                // Pop arguments from stack in reverse order (last param first)
                for _ in 0..target_func_type.params.len() {
                    args.push(engine.exec_stack.values.pop().map_err(|e| {
                        Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                    })?);
                }
                args.reverse(); // Restore correct argument order
                
                Ok(ControlFlow::Call { func_idx: func_idx_val, inputs: args })
            }
            Instruction::CallIndirect(type_idx, table_idx) => {
                // 1. Pop function index from stack
                let elem_idx_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let elem_idx = match elem_idx_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "CallIndirect index not i32")),
                };
                
                // 2. Get table and validate index
                let table = self.module_instance.table(table_idx)?;
                let func_ref_opt = table.get(elem_idx)?;
                let func_ref = func_ref_opt.ok_or_else(|| {
                    Error::new(ErrorCategory::Runtime, codes::RUNTIME_TRAP_ERROR, "CallIndirect: null function reference")
                })?;
                
                // 3. Extract function index from the function reference
                let actual_func_idx = match func_ref {
                    Value::FuncRef(Some(func_ref)) => func_ref.index,
                    Value::FuncRef(None) => return Err(Error::new(ErrorCategory::Runtime, codes::RUNTIME_TRAP_ERROR, "CallIndirect: null function reference")),
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "CallIndirect: table element not a function reference")),
                };
                
                // 4. Type checking - get expected type and actual type
                let expected_func_type = self.module_instance.module().types.get(type_idx as usize).map_err(|_| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH, "CallIndirect: invalid type index")
                })?;
                let actual_func_type = self.module_instance.function_type(actual_func_idx)?;
                
                // 5. Verify type compatibility (simplified check)
                if expected_func_type.params.len() != actual_func_type.params.len() ||
                   expected_func_type.results.len() != actual_func_type.results.len() {
                    return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH, "CallIndirect: function signature mismatch"));
                }
                
                // 6. Pop arguments from stack
                #[cfg(feature = "std")]
                let mut args = ValueStackVec::with_capacity(actual_func_type.params.len());
                #[cfg(not(feature = "std"))]
                let mut args = wrt_foundation::bounded::BoundedVec::new_with_provider(
                    wrt_foundation::safe_memory::NoStdProvider::<1024>::default()
                ).unwrap();
                for _ in 0..actual_func_type.params.len() {
                    args.push(engine.exec_stack.values.pop().map_err(|e| {
                        Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                    })?);
                }
                args.reverse(); // Restore correct argument order
                
                Ok(ControlFlow::Call { func_idx: actual_func_idx, inputs: args })
            }

            // Local variable instructions
            Instruction::LocalGet(local_idx) => {
                let value = self.locals.get(local_idx as usize).map_err(|_| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::INVALID_VALUE,
                        "Invalid local index for get",
                    )
                })?;
                engine.exec_stack.values.push(value.clone()).map_err(|e| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::STACK_OVERFLOW,
                        "Stack overflow on local.get",
                    )
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::LocalSet(local_idx) => {
                let value = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::STACK_UNDERFLOW,
                        "Stack underflow on local.set",
                    )
                })?;
                self.locals.set(local_idx as usize, value).map_err(|e| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::INVALID_VALUE,
                        "Invalid local index for set",
                    )
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::LocalTee(local_idx) => {
                let value = engine
                    .exec_stack
                    .values
                    .peek()
                    .map_err(|e| {
                        Error::new(
                            ErrorCategory::Runtime,
                            codes::STACK_UNDERFLOW,
                            "Stack underflow on local.tee",
                        )
                    })?
                    .clone();
                self.locals.set(local_idx as usize, value).map_err(|e| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::INVALID_VALUE,
                        "Invalid local index for tee",
                    )
                })?;
                Ok(ControlFlow::Next)
            }

            // Global variable instructions
            Instruction::GlobalGet(global_idx) => {
                let global = self.module_instance.global(global_idx)?;
                engine.exec_stack.values.push(global.get_value()).map_err(|e| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::STACK_OVERFLOW,
                        "Stack overflow on global.get",
                    )
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::GlobalSet(global_idx) => {
                let global = self.module_instance.global(global_idx)?;
                if !global.is_mutable() {
                    return Err(Error::new(
                        ErrorCategory::Validation,
                        codes::VALIDATION_ERROR,
                        "Cannot set immutable global",
                    ));
                }
                let value = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::STACK_UNDERFLOW,
                        "Stack underflow on global.set",
                    )
                })?;
                global.set_value(value)?;
                Ok(ControlFlow::Next)
            }

            // Table instructions
            Instruction::TableGet(table_idx) => {
                let table = self.module_instance.table(table_idx)?;
                let elem_idx_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::STACK_UNDERFLOW,
                        "Stack underflow for TableGet index",
                    )
                })?;
                let elem_idx = match elem_idx_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "TableGet index not i32")),
                };

                match table.get(elem_idx)? {
                    Some(val) => engine.exec_stack.values.push(val).map_err(|e| {
                        Error::new(
                            ErrorCategory::Runtime,
                            codes::STACK_OVERFLOW,
                            "Stack overflow on TableGet",
                        )
                    })?,
                    None => {
                        return Err(Error::new(
                            ErrorCategory::Runtime,
                            codes::OUT_OF_BOUNDS_ERROR,
                            "TableGet returned None (null ref or OOB)",
                        ))
                    } // Or specific error for null if needed
                }
                Ok(ControlFlow::Next)
            }
            Instruction::TableSet(table_idx) => {
                let table = self.module_instance.table(table_idx)?;
                let val_to_set = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::STACK_UNDERFLOW,
                        "Stack underflow for TableSet value",
                    )
                })?;
                let elem_idx_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::STACK_UNDERFLOW,
                        "Stack operation error",
                    )
                })?;
                let elem_idx = elem_idx_val.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "TableSet index not i32")
                })? as u32;

                // TODO: Type check val_to_set against table.element_type()
                table.set(elem_idx, val_to_set)?;
                Ok(ControlFlow::Next)
            }
            Instruction::TableSize(table_idx) => {
                let table = self.module_instance.table(table_idx)?;
                engine.exec_stack.values.push(Value::I32(table.size() as i32)).map_err(|e| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::STACK_OVERFLOW,
                        "Stack operation error",
                    )
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::TableGrow(table_idx) => {
                let table = self.module_instance.table(table_idx)?;
                let init_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::STACK_UNDERFLOW,
                        "Stack operation error",
                    )
                })?;
                let delta_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::STACK_UNDERFLOW,
                        "Stack operation error",
                    )
                })?;
                let delta = delta_val.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "TableGrow delta not i32")
                })? as u32;

                let old_size = table.grow(delta, init_val)?;
                engine.exec_stack.values.push(Value::I32(old_size as i32)).map_err(|e| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::STACK_OVERFLOW,
                        "Stack operation error",
                    )
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::TableFill(table_idx) => {
                self.table_fill(table_idx, engine)?;
                Ok(ControlFlow::Next)
            }
            Instruction::TableCopy(dst_table_idx, src_table_idx) => {
                self.table_copy(dst_table_idx, src_table_idx, engine)?;
                Ok(ControlFlow::Next)
            }
            Instruction::TableInit(elem_seg_idx, table_idx) => {
                self.table_init(elem_seg_idx, table_idx, engine)?;
                Ok(ControlFlow::Next)
            }
            Instruction::ElemDrop(elem_seg_idx) => {
                self.module_instance.module().drop_element_segment(elem_seg_idx);
                Ok(ControlFlow::Next)
            }

            // Memory instructions (Placeholders, many need base address + offset)
            // Common pattern: pop address, calculate effective_address, operate on memory.
            // Example: I32Load needs `addr = pop_i32() + offset_immediate`
            //          `value = memory.read_i32(addr)`
            //          `push(value)`
            Instruction::I32Load(mem_arg) => {
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Load address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I32Load address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?; // Assuming memory index 0
                
                // Check bounds
                if effective_addr.checked_add(4).map_or(true, |end| end as usize > memory.size_in_bytes()) {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I32Load out of bounds"));
                }
                
                // Read 4 bytes as little-endian i32
                let mut bytes = [0u8; 4];
                memory.read(effective_addr as usize, &mut bytes)?;
                let value = i32::from_le_bytes(bytes);
                
                engine.exec_stack.values.push(Value::I32(value)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Load(mem_arg) => {
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Load address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Load address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr.checked_add(8).map_or(true, |end| end as usize > memory.size_in_bytes()) {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Load out of bounds"));
                }
                
                let mut bytes = [0u8; 8];
                memory.read(effective_addr as usize, &mut bytes)?;
                let value = i64::from_le_bytes(bytes);
                
                engine.exec_stack.values.push(Value::I64(value)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32Load(mem_arg) => {
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Load address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "F32Load address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr.checked_add(4).map_or(true, |end| end as usize > memory.size_in_bytes()) {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "F32Load out of bounds"));
                }
                
                let mut bytes = [0u8; 4];
                memory.read(effective_addr as usize, &mut bytes)?;
                let bits = u32::from_le_bytes(bytes);
                let value = f32::from_bits(bits);
                
                engine.exec_stack.values.push(Value::F32(value)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64Load(mem_arg) => {
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Load address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "F64Load address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr.checked_add(8).map_or(true, |end| end as usize > memory.size_in_bytes()) {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "F64Load out of bounds"));
                }
                
                let mut bytes = [0u8; 8];
                memory.read(effective_addr as usize, &mut bytes)?;
                let bits = u64::from_le_bytes(bytes);
                let value = f64::from_bits(bits);
                
                engine.exec_stack.values.push(Value::F64(value)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32Load8S(mem_arg) => {
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Load8S address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I32Load8S address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr as usize >= memory.size_in_bytes() {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I32Load8S out of bounds"));
                }
                
                let mut byte = [0u8; 1];
                memory.read(effective_addr as usize, &mut byte)?;
                // Sign extend 8-bit to 32-bit
                let value = byte[0] as i8 as i32;
                
                engine.exec_stack.values.push(Value::I32(value)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32Load8U(mem_arg) => {
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Load8U address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I32Load8U address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr as usize >= memory.size_in_bytes() {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I32Load8U out of bounds"));
                }
                
                let mut byte = [0u8; 1];
                memory.read(effective_addr as usize, &mut byte)?;
                // Zero extend 8-bit to 32-bit
                let value = byte[0] as i32;
                
                engine.exec_stack.values.push(Value::I32(value)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32Load16S(mem_arg) => {
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Load16S address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I32Load16S address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr.checked_add(2).map_or(true, |end| end as usize > memory.size_in_bytes()) {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I32Load16S out of bounds"));
                }
                
                let mut bytes = [0u8; 2];
                memory.read(effective_addr as usize, &mut bytes)?;
                // Sign extend 16-bit to 32-bit
                let value = i16::from_le_bytes(bytes) as i32;
                
                engine.exec_stack.values.push(Value::I32(value)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32Load16U(mem_arg) => {
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Load16U address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I32Load16U address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr.checked_add(2).map_or(true, |end| end as usize > memory.size_in_bytes()) {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I32Load16U out of bounds"));
                }
                
                let mut bytes = [0u8; 2];
                memory.read(effective_addr as usize, &mut bytes)?;
                // Zero extend 16-bit to 32-bit
                let value = u16::from_le_bytes(bytes) as i32;
                
                engine.exec_stack.values.push(Value::I32(value)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Load8S(mem_arg) => {
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Load8S address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Load8S address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr as usize >= memory.size_in_bytes() {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Load8S out of bounds"));
                }
                
                let mut bytes = [0u8; 1];
                memory.read(effective_addr as usize, &mut bytes)?;
                let value = i8::from_le_bytes(bytes) as i64; // Sign extend
                
                engine.exec_stack.values.push(Value::I64(value)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Load8U(mem_arg) => {
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Load8U address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Load8U address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr as usize >= memory.size_in_bytes() {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Load8U out of bounds"));
                }
                
                let mut bytes = [0u8; 1];
                memory.read(effective_addr as usize, &mut bytes)?;
                let value = u8::from_le_bytes(bytes) as i64; // Zero extend
                
                engine.exec_stack.values.push(Value::I64(value)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Load16S(mem_arg) => {
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Load16S address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Load16S address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr.checked_add(2).map_or(true, |end| end as usize > memory.size_in_bytes()) {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Load16S out of bounds"));
                }
                
                let mut bytes = [0u8; 2];
                memory.read(effective_addr as usize, &mut bytes)?;
                let value = i16::from_le_bytes(bytes) as i64; // Sign extend
                
                engine.exec_stack.values.push(Value::I64(value)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Load16U(mem_arg) => {
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Load16U address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Load16U address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr.checked_add(2).map_or(true, |end| end as usize > memory.size_in_bytes()) {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Load16U out of bounds"));
                }
                
                let mut bytes = [0u8; 2];
                memory.read(effective_addr as usize, &mut bytes)?;
                let value = u16::from_le_bytes(bytes) as i64; // Zero extend
                
                engine.exec_stack.values.push(Value::I64(value)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Load32S(mem_arg) => {
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Load32S address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Load32S address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr.checked_add(4).map_or(true, |end| end as usize > memory.size_in_bytes()) {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Load32S out of bounds"));
                }
                
                let mut bytes = [0u8; 4];
                memory.read(effective_addr as usize, &mut bytes)?;
                let value = i32::from_le_bytes(bytes) as i64; // Sign extend
                
                engine.exec_stack.values.push(Value::I64(value)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Load32U(mem_arg) => {
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Load32U address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Load32U address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr.checked_add(4).map_or(true, |end| end as usize > memory.size_in_bytes()) {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Load32U out of bounds"));
                }
                
                let mut bytes = [0u8; 4];
                memory.read(effective_addr as usize, &mut bytes)?;
                let value = u32::from_le_bytes(bytes) as i64; // Zero extend
                
                engine.exec_stack.values.push(Value::I64(value)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }

            Instruction::I32Store(mem_arg) => {
                let value_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let value = match value_val {
                    Value::I32(val) => val,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Store value not i32")),
                };
                
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Store address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I32Store address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr.checked_add(4).map_or(true, |end| end as usize > memory.size_in_bytes()) {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I32Store out of bounds"));
                }
                
                let bytes = value.to_le_bytes();
                memory.write(effective_addr as usize, &bytes)?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Store(mem_arg) => {
                let value_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let value = match value_val {
                    Value::I64(val) => val,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Store value not i64")),
                };
                
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Store address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Store address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr.checked_add(8).map_or(true, |end| end as usize > memory.size_in_bytes()) {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Store out of bounds"));
                }
                
                let bytes = value.to_le_bytes();
                memory.write(effective_addr as usize, &bytes)?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32Store(mem_arg) => {
                let value_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let value = match value_val {
                    Value::F32(val) => val,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Store value not f32")),
                };
                
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Store address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "F32Store address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr.checked_add(4).map_or(true, |end| end as usize > memory.size_in_bytes()) {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "F32Store out of bounds"));
                }
                
                let bits = value.to_bits();
                let bytes = bits.to_le_bytes();
                memory.write(effective_addr as usize, &bytes)?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64Store(mem_arg) => {
                let value_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let value = match value_val {
                    Value::F64(val) => val,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Store value not f64")),
                };
                
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Store address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "F64Store address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr.checked_add(8).map_or(true, |end| end as usize > memory.size_in_bytes()) {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "F64Store out of bounds"));
                }
                
                let bits = value.to_bits();
                let bytes = bits.to_le_bytes();
                memory.write(effective_addr as usize, &bytes)?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32Store8(mem_arg) => {
                let value_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let value = match value_val {
                    Value::I32(val) => val,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Store8 value not i32")),
                };
                
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Store8 address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I32Store8 address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr as usize >= memory.size_in_bytes() {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I32Store8 out of bounds"));
                }
                
                // Truncate to 8 bits
                let byte = (value & 0xFF) as u8;
                memory.write(effective_addr as usize, &[byte])?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32Store16(mem_arg) => {
                let value_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let value = match value_val {
                    Value::I32(val) => val,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Store16 value not i32")),
                };
                
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Store16 address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I32Store16 address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr.checked_add(2).map_or(true, |end| end as usize > memory.size_in_bytes()) {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I32Store16 out of bounds"));
                }
                
                // Truncate to 16 bits
                let bytes = (value as u16).to_le_bytes();
                memory.write(effective_addr as usize, &bytes)?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Store8(mem_arg) => {
                let value_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                
                let value = match value_val {
                    Value::I64(val) => val,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Store8 value not i64")),
                };
                
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Store8 address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Store8 address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr as usize >= memory.size_in_bytes() {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Store8 out of bounds"));
                }
                
                // Store lower 8 bits
                let bytes = [(value as u8)];
                memory.write(effective_addr as usize, &bytes)?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Store16(mem_arg) => {
                let value_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                
                let value = match value_val {
                    Value::I64(val) => val,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Store16 value not i64")),
                };
                
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Store16 address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Store16 address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr.checked_add(2).map_or(true, |end| end as usize > memory.size_in_bytes()) {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Store16 out of bounds"));
                }
                
                // Store lower 16 bits
                let bytes = (value as u16).to_le_bytes();
                memory.write(effective_addr as usize, &bytes)?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Store32(mem_arg) => {
                let value_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                
                let value = match value_val {
                    Value::I64(val) => val,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Store32 value not i64")),
                };
                
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Store32 address not i32")),
                };
                
                let effective_addr = addr.checked_add(mem_arg.offset).ok_or_else(|| {
                    Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Store32 address overflow")
                })?;
                
                let memory = self.module_instance.memory(0)?;
                
                if effective_addr.checked_add(4).map_or(true, |end| end as usize > memory.size_in_bytes()) {
                    return Err(Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "I64Store32 out of bounds"));
                }
                
                // Store lower 32 bits
                let bytes = (value as u32).to_le_bytes();
                memory.write(effective_addr as usize, &bytes)?;
                Ok(ControlFlow::Next)
            }

            Instruction::MemorySize(_mem_idx) => {
                // mem_idx is always 0 in Wasm MVP
                let mem = self.module_instance.memory(0)?; // Assuming memory index 0
                engine.exec_stack.values.push(Value::I32(mem.size_pages() as i32)).map_err(|e| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::STACK_OVERFLOW,
                        "Stack operation error",
                    )
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::MemoryGrow(_mem_idx) => {
                // mem_idx is always 0 in Wasm MVP
                let mem = self.module_instance.memory(0)?;
                let delta_pages_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::STACK_UNDERFLOW,
                        "Stack operation error",
                    )
                })?;
                let delta_pages = delta_pages_val.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "MemoryGrow delta not i32")
                })? as u32;

                let old_size_pages = mem.grow(delta_pages)?;
                engine.exec_stack.values.push(Value::I32(old_size_pages as i32)).map_err(|e| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::STACK_OVERFLOW,
                        "Stack operation error",
                    )
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::MemoryFill(_mem_idx) => {
                self.memory_fill(0, engine)?; // Assuming memory index 0
                Ok(ControlFlow::Next)
            }
            Instruction::MemoryCopy(_dst_mem_idx, _src_mem_idx) => {
                // both always 0 in MVP
                self.memory_copy(0, 0, engine)?;
                Ok(ControlFlow::Next)
            }
            Instruction::MemoryInit(_data_seg_idx, _mem_idx) => {
                // mem_idx always 0 in MVP
                self.memory_init(_data_seg_idx, 0, engine)?;
                Ok(ControlFlow::Next)
            }
            Instruction::DataDrop(_data_seg_idx) => {
                self.module_instance.module().drop_data_segment(_data_seg_idx);
                Ok(ControlFlow::Next)
            }

            // Numeric Const instructions
            Instruction::I32Const(val) => {
                engine.exec_stack.values.push(Value::I32(val)).map_err(|e| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::STACK_OVERFLOW,
                        "Stack operation error",
                    )
                })?
            }
            Instruction::I64Const(val) => {
                engine.exec_stack.values.push(Value::I64(val)).map_err(|e| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::STACK_OVERFLOW,
                        "Stack operation error",
                    )
                })?
            }
            Instruction::F32Const(val) => engine
                .exec_stack
                .values
                .push(Value::F32(f32::from_bits(val))) // Assuming val is u32 bits
                .map_err(|e| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::STACK_OVERFLOW,
                        "Stack operation error",
                    )
                })?,
            Instruction::F64Const(val) => engine
                .exec_stack
                .values
                .push(Value::F64(f64::from_bits(val))) // Assuming val is u64 bits
                .map_err(|e| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::STACK_OVERFLOW,
                        "Stack operation error",
                    )
                })?,

            // Arithmetic instructions
            Instruction::I32Add => {
                let b = Self::pop_i32(engine)?;
                let a = Self::pop_i32(engine)?;
                engine.exec_stack.values.push(Value::I32(a.wrapping_add(b))).map_err(|_| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack overflow")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32Sub => {
                let b = Self::pop_i32(engine)?;
                let a = Self::pop_i32(engine)?;
                engine.exec_stack.values.push(Value::I32(a.wrapping_sub(b))).map_err(|_| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack overflow")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32Mul => {
                let b = Self::pop_i32(engine)?;
                let a = Self::pop_i32(engine)?;
                engine.exec_stack.values.push(Value::I32(a.wrapping_mul(b))).map_err(|_| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack overflow")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // Missing I32 arithmetic operations
            Instruction::I32RemS => {
                let b = Self::pop_i32(engine)?;
                if b == 0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::RUNTIME_DIVISION_BY_ZERO_ERROR, "I32RemS division by zero"));
                }
                let a = Self::pop_i32(engine)?;
                // Check for overflow: i32::MIN % -1 would panic, but result should be 0
                let result = if a == i32::MIN && b == -1 { 0 } else { a % b };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|_| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack overflow")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32RemU => {
                let b = Self::pop_i32(engine)?;
                if b == 0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::DIVISION_BY_ZERO, "I32RemU division by zero"));
                }
                let a = Self::pop_i32(engine)?;
                // Unsigned remainder - cast to u32
                let result = (a as u32) % (b as u32);
                engine.exec_stack.values.push(Value::I32(result as i32)).map_err(|_| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack overflow")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32And => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32And second operand not i32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32And first operand not i32")
                })?;
                engine.exec_stack.values.push(Value::I32(a & b)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32Or => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Or second operand not i32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Or first operand not i32")
                })?;
                engine.exec_stack.values.push(Value::I32(a | b)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32Xor => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Xor second operand not i32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Xor first operand not i32")
                })?;
                engine.exec_stack.values.push(Value::I32(a ^ b)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32Shl => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Shl second operand not i32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Shl first operand not i32")
                })?;
                // Shift amount is masked to 5 bits (0-31) as per WebAssembly spec
                let shift = (b as u32) & 0x1F;
                engine.exec_stack.values.push(Value::I32(a << shift)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32ShrS => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32ShrS second operand not i32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32ShrS first operand not i32")
                })?;
                // Shift amount is masked to 5 bits (0-31) as per WebAssembly spec
                let shift = (b as u32) & 0x1F;
                engine.exec_stack.values.push(Value::I32(a >> shift)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32ShrU => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32ShrU second operand not i32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32ShrU first operand not i32")
                })?;
                // Shift amount is masked to 5 bits (0-31) as per WebAssembly spec
                let shift = (b as u32) & 0x1F;
                // Unsigned right shift
                let result = (a as u32) >> shift;
                engine.exec_stack.values.push(Value::I32(result as i32)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32Rotl => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Rotl second operand not i32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Rotl first operand not i32")
                })?;
                // Rotate amount is masked to 5 bits (0-31) as per WebAssembly spec
                let rotate = (b as u32) & 0x1F;
                let result = (a as u32).rotate_left(rotate);
                engine.exec_stack.values.push(Value::I32(result as i32)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32Rotr => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Rotr second operand not i32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Rotr first operand not i32")
                })?;
                // Rotate amount is masked to 5 bits (0-31) as per WebAssembly spec
                let rotate = (b as u32) & 0x1F;
                let result = (a as u32).rotate_right(rotate);
                engine.exec_stack.values.push(Value::I32(result as i32)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // Stack manipulation
            Instruction::Drop => {
                engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // Additional I32 arithmetic instructions
            Instruction::I32DivS => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "I32DivS second operand not i32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "I32DivS first operand not i32")
                })?;
                if b == 0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::RUNTIME_ERROR, "Division by zero"));
                }
                if a == i32::MIN && b == -1 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::RUNTIME_ERROR, "Integer overflow"));
                }
                engine.exec_stack.values.push(Value::I32(a / b)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32DivU => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "I32DivU second operand not i32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "I32DivU first operand not i32")
                })?;
                if b == 0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::RUNTIME_ERROR, "Division by zero"));
                }
                let result = (a as u32) / (b as u32);
                engine.exec_stack.values.push(Value::I32(result as i32)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // I32 comparison instructions
            Instruction::I32Eq => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "I32Eq second operand not i32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "I32Eq first operand not i32")
                })?;
                engine.exec_stack.values.push(Value::I32(if a == b { 1 } else { 0 })).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32Ne => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "I32Ne second operand not i32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "I32Ne first operand not i32")
                })?;
                engine.exec_stack.values.push(Value::I32(if a != b { 1 } else { 0 })).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32LtS => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "I32LtS second operand not i32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "I32LtS first operand not i32")
                })?;
                engine.exec_stack.values.push(Value::I32(if a < b { 1 } else { 0 })).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // I64 arithmetic instructions  
            Instruction::I64Add => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "I64Add second operand not i64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "I64Add first operand not i64")
                })?;
                engine.exec_stack.values.push(Value::I64(a.wrapping_add(b))).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Sub => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "I64Sub second operand not i64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "I64Sub first operand not i64")
                })?;
                engine.exec_stack.values.push(Value::I64(a.wrapping_sub(b))).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Mul => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Mul second operand not i64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Mul first operand not i64")
                })?;
                engine.exec_stack.values.push(Value::I64(a.wrapping_mul(b))).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64DivS => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64DivS second operand not i64")
                })?;
                
                if b == 0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::DIVISION_BY_ZERO, "I64DivS division by zero"));
                }
                
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64DivS first operand not i64")
                })?;
                
                // Check for overflow: i64::MIN / -1 would overflow
                if a == i64::MIN && b == -1 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::INTEGER_OVERFLOW, "I64DivS integer overflow"));
                }
                
                engine.exec_stack.values.push(Value::I64(a / b)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64DivU => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64DivU second operand not i64")
                })?;
                
                if b == 0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::DIVISION_BY_ZERO, "I64DivU division by zero"));
                }
                
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64DivU first operand not i64")
                })?;
                
                // Unsigned division - cast to u64
                let result = (a as u64) / (b as u64);
                engine.exec_stack.values.push(Value::I64(result as i64)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64And => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64And second operand not i64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64And first operand not i64")
                })?;
                engine.exec_stack.values.push(Value::I64(a & b)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Or => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Or second operand not i64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Or first operand not i64")
                })?;
                engine.exec_stack.values.push(Value::I64(a | b)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Xor => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Xor second operand not i64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Xor first operand not i64")
                })?;
                engine.exec_stack.values.push(Value::I64(a ^ b)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64RemS => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64RemS second operand not i64")
                })?;
                
                if b == 0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::DIVISION_BY_ZERO, "I64RemS division by zero"));
                }
                
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64RemS first operand not i64")
                })?;
                
                // Check for overflow: i64::MIN % -1 would panic, but result should be 0
                let result = if a == i64::MIN && b == -1 { 0 } else { a % b };
                
                engine.exec_stack.values.push(Value::I64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64RemU => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64RemU second operand not i64")
                })?;
                
                if b == 0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::DIVISION_BY_ZERO, "I64RemU division by zero"));
                }
                
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64RemU first operand not i64")
                })?;
                
                // Unsigned remainder - cast to u64
                let result = (a as u64) % (b as u64);
                engine.exec_stack.values.push(Value::I64(result as i64)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Shl => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Shl second operand not i64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Shl first operand not i64")
                })?;
                // Shift amount is masked to 6 bits (0-63) as per WebAssembly spec for i64
                let shift = (b as u64) & 0x3F;
                engine.exec_stack.values.push(Value::I64(a << shift)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64ShrS => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64ShrS second operand not i64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64ShrS first operand not i64")
                })?;
                // Shift amount is masked to 6 bits (0-63) as per WebAssembly spec for i64
                let shift = (b as u64) & 0x3F;
                engine.exec_stack.values.push(Value::I64(a >> shift)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64ShrU => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64ShrU second operand not i64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64ShrU first operand not i64")
                })?;
                // Shift amount is masked to 6 bits (0-63) as per WebAssembly spec for i64
                let shift = (b as u64) & 0x3F;
                // Unsigned right shift
                let result = (a as u64) >> shift;
                engine.exec_stack.values.push(Value::I64(result as i64)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Rotl => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Rotl second operand not i64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Rotl first operand not i64")
                })?;
                // Rotate amount is masked to 6 bits (0-63) as per WebAssembly spec for i64
                let rotate = (b as u64) & 0x3F;
                let result = (a as u64).rotate_left(rotate as u32);
                engine.exec_stack.values.push(Value::I64(result as i64)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Rotr => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Rotr second operand not i64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Rotr first operand not i64")
                })?;
                // Rotate amount is masked to 6 bits (0-63) as per WebAssembly spec for i64
                let rotate = (b as u64) & 0x3F;
                let result = (a as u64).rotate_right(rotate as u32);
                engine.exec_stack.values.push(Value::I64(result as i64)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // Floating-point arithmetic operations
            Instruction::F32Add => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Add second operand not f32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Add first operand not f32")
                })?;
                engine.exec_stack.values.push(Value::F32(a + b)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32Sub => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Sub second operand not f32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Sub first operand not f32")
                })?;
                engine.exec_stack.values.push(Value::F32(a - b)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32Mul => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Mul second operand not f32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Mul first operand not f32")
                })?;
                engine.exec_stack.values.push(Value::F32(a * b)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32Div => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Div second operand not f32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Div first operand not f32")
                })?;
                engine.exec_stack.values.push(Value::F32(a / b)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // Stack manipulation
            Instruction::Select => {
                let condition_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let condition = match condition_val {
                    Value::I32(val) => val,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "Select condition not i32")),
                };
                let val2 = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let val1 = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                
                let result = if condition != 0 { val1 } else { val2 };
                engine.exec_stack.values.push(result).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // I32 comparison operations
            Instruction::I32LtU => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32LtU second operand not i32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32LtU first operand not i32")
                })?;
                let result = if (a as u32) < (b as u32) { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32GtS => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32GtS second operand not i32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32GtS first operand not i32")
                })?;
                let result = if a > b { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32GtU => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32GtU second operand not i32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32GtU first operand not i32")
                })?;
                let result = if (a as u32) > (b as u32) { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32LeS => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32LeS second operand not i32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32LeS first operand not i32")
                })?;
                let result = if a <= b { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32LeU => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32LeU second operand not i32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32LeU first operand not i32")
                })?;
                let result = if (a as u32) <= (b as u32) { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32GeS => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32GeS second operand not i32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32GeS first operand not i32")
                })?;
                let result = if a >= b { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32GeU => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32GeU second operand not i32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32GeU first operand not i32")
                })?;
                let result = if (a as u32) >= (b as u32) { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // I32 unary operations
            Instruction::I32Eqz => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Eqz operand not i32")
                })?;
                let result = if a == 0 { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // Type conversion operations
            Instruction::I32WrapI64 => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32WrapI64 operand not i64")
                })?;
                // Wrap i64 to i32 by truncating upper 32 bits
                let result = a as i32;
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64ExtendI32S => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64ExtendI32S operand not i32")
                })?;
                // Sign-extend i32 to i64
                let result = a as i64;
                engine.exec_stack.values.push(Value::I64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64ExtendI32U => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64ExtendI32U operand not i32")
                })?;
                // Zero-extend i32 to i64
                let result = (a as u32) as i64;
                engine.exec_stack.values.push(Value::I64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32TruncF32S => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32TruncF32S operand not f32")
                })?;
                
                // Check for NaN or out-of-range values
                if a.is_nan() || a.is_infinite() || a < -2_147_483_649.0 || a >= 2_147_483_648.0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::INTEGER_OVERFLOW, "I32TruncF32S out of range"));
                }
                
                let result = {
                    #[cfg(feature = "std")]
                    { a.trunc() }
                    #[cfg(not(feature = "std"))]
                    { 
                        // Manual truncation: remove fractional part
                        a as i32 as f32
                    }
                } as i32;
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32TruncF32U => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32TruncF32U operand not f32")
                })?;
                
                // Check for NaN or out-of-range values for unsigned
                if a.is_nan() || a.is_infinite() || a < -1.0 || a >= 4_294_967_296.0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::INTEGER_OVERFLOW, "I32TruncF32U out of range"));
                }
                
                let result = ({
                    #[cfg(feature = "std")]
                    { a.trunc() }
                    #[cfg(not(feature = "std"))]
                    { 
                        // Manual truncation: remove fractional part
                        a as i32 as f32
                    }
                } as u32) as i32;
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // I64 comparison operations
            Instruction::I64Eq => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Eq second operand not i64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Eq first operand not i64")
                })?;
                let result = if a == b { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Ne => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Ne second operand not i64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Ne first operand not i64")
                })?;
                let result = if a != b { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64LtS => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64LtS second operand not i64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64LtS first operand not i64")
                })?;
                let result = if a < b { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64LtU => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64LtU second operand not i64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64LtU first operand not i64")
                })?;
                let result = if (a as u64) < (b as u64) { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64GtS => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64GtS second operand not i64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64GtS first operand not i64")
                })?;
                let result = if a > b { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64GtU => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64GtU second operand not i64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64GtU first operand not i64")
                })?;
                let result = if (a as u64) > (b as u64) { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64LeS => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64LeS second operand not i64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64LeS first operand not i64")
                })?;
                let result = if a <= b { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64LeU => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64LeU second operand not i64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64LeU first operand not i64")
                })?;
                let result = if (a as u64) <= (b as u64) { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64GeS => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64GeS second operand not i64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64GeS first operand not i64")
                })?;
                let result = if a >= b { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64GeU => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64GeU second operand not i64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64GeU first operand not i64")
                })?;
                let result = if (a as u64) >= (b as u64) { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // I64 unary operations
            Instruction::I64Eqz => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Eqz operand not i64")
                })?;
                let result = if a == 0 { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // F32 comparison operations
            Instruction::F32Eq => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Eq second operand not f32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Eq first operand not f32")
                })?;
                let result = if a == b { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32Ne => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Ne second operand not f32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Ne first operand not f32")
                })?;
                let result = if a != b { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32Lt => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Lt second operand not f32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Lt first operand not f32")
                })?;
                let result = if a < b { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32Gt => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Gt second operand not f32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Gt first operand not f32")
                })?;
                let result = if a > b { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32Le => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Le second operand not f32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Le first operand not f32")
                })?;
                let result = if a <= b { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32Ge => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Ge second operand not f32")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Ge first operand not f32")
                })?;
                let result = if a >= b { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // F64 comparison operations
            Instruction::F64Eq => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Eq second operand not f64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Eq first operand not f64")
                })?;
                let result = if a == b { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64Ne => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Ne second operand not f64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Ne first operand not f64")
                })?;
                let result = if a != b { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64Lt => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Lt second operand not f64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Lt first operand not f64")
                })?;
                let result = if a < b { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64Gt => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Gt second operand not f64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Gt first operand not f64")
                })?;
                let result = if a > b { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64Le => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Le second operand not f64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Le first operand not f64")
                })?;
                let result = if a <= b { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64Ge => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Ge second operand not f64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Ge first operand not f64")
                })?;
                let result = if a >= b { 1 } else { 0 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // F32 unary operations
            Instruction::F32Abs => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Abs operand not f32")
                })?;
                let result = a.abs();
                engine.exec_stack.values.push(Value::F32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32Neg => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Neg operand not f32")
                })?;
                let result = -a;
                engine.exec_stack.values.push(Value::F32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32Ceil => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Ceil operand not f32")
                })?;
                let result = a.ceil();
                engine.exec_stack.values.push(Value::F32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32Floor => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Floor operand not f32")
                })?;
                let result = a.floor();
                engine.exec_stack.values.push(Value::F32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32Trunc => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Trunc operand not f32")
                })?;
                let result = {
                    #[cfg(feature = "std")]
                    { a.trunc() }
                    #[cfg(not(feature = "std"))]
                    { 
                        // Manual truncation: remove fractional part
                        a as i32 as f32
                    }
                };
                engine.exec_stack.values.push(Value::F32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32Nearest => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Nearest operand not f32")
                })?;
                let result = a.round();
                engine.exec_stack.values.push(Value::F32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32Sqrt => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Sqrt operand not f32")
                })?;
                let result = a.sqrt();
                engine.exec_stack.values.push(Value::F32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // F64 arithmetic operations
            Instruction::F64Add => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Add second operand not f64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Add first operand not f64")
                })?;
                engine.exec_stack.values.push(Value::F64(a + b)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64Sub => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Sub second operand not f64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Sub first operand not f64")
                })?;
                engine.exec_stack.values.push(Value::F64(a - b)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64Mul => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Mul second operand not f64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Mul first operand not f64")
                })?;
                engine.exec_stack.values.push(Value::F64(a * b)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64Div => {
                let b = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Div second operand not f64")
                })?;
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Div first operand not f64")
                })?;
                engine.exec_stack.values.push(Value::F64(a / b)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // More type conversion operations
            Instruction::I32TruncF64S => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32TruncF64S operand not f64")
                })?;
                
                // Check for NaN or out-of-range values
                if a.is_nan() || a.is_infinite() || a < -2_147_483_649.0 || a >= 2_147_483_648.0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::INTEGER_OVERFLOW, "I32TruncF64S out of range"));
                }
                
                let result = {
                    #[cfg(feature = "std")]
                    { a.trunc() }
                    #[cfg(not(feature = "std"))]
                    { 
                        // Manual truncation: remove fractional part
                        a as i32 as f32
                    }
                } as i32;
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32TruncF64U => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32TruncF64U operand not f64")
                })?;
                
                // Check for NaN or out-of-range values for unsigned
                if a.is_nan() || a.is_infinite() || a < -1.0 || a >= 4_294_967_296.0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::INTEGER_OVERFLOW, "I32TruncF64U out of range"));
                }
                
                let result = ({
                    #[cfg(feature = "std")]
                    { a.trunc() }
                    #[cfg(not(feature = "std"))]
                    { 
                        // Manual truncation: remove fractional part
                        a as i32 as f32
                    }
                } as u32) as i32;
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64TruncF32S => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64TruncF32S operand not f32")
                })?;
                
                // Check for NaN or out-of-range values
                if a.is_nan() || a.is_infinite() || a < -9_223_372_036_854_775_808.0 || a >= 9_223_372_036_854_775_808.0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::INTEGER_OVERFLOW, "I64TruncF32S out of range"));
                }
                
                let result = {
                    #[cfg(feature = "std")]
                    { a.trunc() }
                    #[cfg(not(feature = "std"))]
                    { 
                        // Manual truncation: remove fractional part
                        a as i32 as f32
                    }
                } as i64;
                engine.exec_stack.values.push(Value::I64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64TruncF32U => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64TruncF32U operand not f32")
                })?;
                
                // Check for NaN or out-of-range values for unsigned
                if a.is_nan() || a.is_infinite() || a < -1.0 || a >= 18_446_744_073_709_551_616.0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::INTEGER_OVERFLOW, "I64TruncF32U out of range"));
                }
                
                let result = ({
                    #[cfg(feature = "std")]
                    { a.trunc() }
                    #[cfg(not(feature = "std"))]
                    { 
                        // Manual truncation: remove fractional part
                        a as i32 as f32
                    }
                } as u64) as i64;
                engine.exec_stack.values.push(Value::I64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64TruncF64S => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64TruncF64S operand not f64")
                })?;
                
                // Check for NaN or out-of-range values
                if a.is_nan() || a.is_infinite() || a < -9_223_372_036_854_775_808.0 || a >= 9_223_372_036_854_775_808.0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::INTEGER_OVERFLOW, "I64TruncF64S out of range"));
                }
                
                let result = {
                    #[cfg(feature = "std")]
                    { a.trunc() }
                    #[cfg(not(feature = "std"))]
                    { 
                        // Manual truncation: remove fractional part
                        a as i32 as f32
                    }
                } as i64;
                engine.exec_stack.values.push(Value::I64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64TruncF64U => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64TruncF64U operand not f64")
                })?;
                
                // Check for NaN or out-of-range values for unsigned
                if a.is_nan() || a.is_infinite() || a < -1.0 || a >= 18_446_744_073_709_551_616.0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::INTEGER_OVERFLOW, "I64TruncF64U out of range"));
                }
                
                let result = ({
                    #[cfg(feature = "std")]
                    { a.trunc() }
                    #[cfg(not(feature = "std"))]
                    { 
                        // Manual truncation: remove fractional part
                        a as i32 as f32
                    }
                } as u64) as i64;
                engine.exec_stack.values.push(Value::I64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // Float to float conversions
            Instruction::F32ConvertI32S => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32ConvertI32S operand not i32")
                })?;
                let result = a as f32;
                engine.exec_stack.values.push(Value::F32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32ConvertI32U => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32ConvertI32U operand not i32")
                })?;
                let result = (a as u32) as f32;
                engine.exec_stack.values.push(Value::F32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32ConvertI64S => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32ConvertI64S operand not i64")
                })?;
                let result = a as f32;
                engine.exec_stack.values.push(Value::F32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32ConvertI64U => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32ConvertI64U operand not i64")
                })?;
                let result = (a as u64) as f32;
                engine.exec_stack.values.push(Value::F32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32DemoteF64 => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32DemoteF64 operand not f64")
                })?;
                let result = a as f32;
                engine.exec_stack.values.push(Value::F32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // F64 conversion operations
            Instruction::F64ConvertI32S => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64ConvertI32S operand not i32")
                })?;
                let result = a as f64;
                engine.exec_stack.values.push(Value::F64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64ConvertI32U => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64ConvertI32U operand not i32")
                })?;
                let result = (a as u32) as f64;
                engine.exec_stack.values.push(Value::F64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64ConvertI64S => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64ConvertI64S operand not i64")
                })?;
                let result = a as f64;
                engine.exec_stack.values.push(Value::F64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64ConvertI64U => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64ConvertI64U operand not i64")
                })?;
                let result = (a as u64) as f64;
                engine.exec_stack.values.push(Value::F64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64PromoteF32 => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64PromoteF32 operand not f32")
                })?;
                let result = a as f64;
                engine.exec_stack.values.push(Value::F64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // Reinterpret operations
            Instruction::I32ReinterpretF32 => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32ReinterpretF32 operand not f32")
                })?;
                let result = a.to_bits() as i32;
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64ReinterpretF64 => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64ReinterpretF64 operand not f64")
                })?;
                let result = a.to_bits() as i64;
                engine.exec_stack.values.push(Value::I64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32ReinterpretI32 => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32ReinterpretI32 operand not i32")
                })?;
                let result = f32::from_bits(a as u32);
                engine.exec_stack.values.push(Value::F32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64ReinterpretI64 => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64ReinterpretI64 operand not i64")
                })?;
                let result = f64::from_bits(a as u64);
                engine.exec_stack.values.push(Value::F64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // F64 unary operations
            Instruction::F64Abs => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Abs operand not f64")
                })?;
                let result = a.abs();
                engine.exec_stack.values.push(Value::F64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64Neg => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Neg operand not f64")
                })?;
                let result = -a;
                engine.exec_stack.values.push(Value::F64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64Ceil => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Ceil operand not f64")
                })?;
                let result = a.ceil();
                engine.exec_stack.values.push(Value::F64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64Floor => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Floor operand not f64")
                })?;
                let result = a.floor();
                engine.exec_stack.values.push(Value::F64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64Trunc => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Trunc operand not f64")
                })?;
                let result = {
                    #[cfg(feature = "std")]
                    { a.trunc() }
                    #[cfg(not(feature = "std"))]
                    { 
                        // Manual truncation: remove fractional part
                        a as i32 as f32
                    }
                };
                engine.exec_stack.values.push(Value::F64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64Nearest => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Nearest operand not f64")
                })?;
                let result = a.round();
                engine.exec_stack.values.push(Value::F64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64Sqrt => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Sqrt operand not f64")
                })?;
                let result = a.sqrt();
                engine.exec_stack.values.push(Value::F64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // Sign extension operations
            Instruction::I32Extend8S => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Extend8S operand not i32")
                })?;
                // Sign-extend from 8 bits to 32 bits
                let result = (a as i8) as i32;
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32Extend16S => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Extend16S operand not i32")
                })?;
                // Sign-extend from 16 bits to 32 bits
                let result = (a as i16) as i32;
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Extend8S => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Extend8S operand not i64")
                })?;
                // Sign-extend from 8 bits to 64 bits
                let result = (a as i8) as i64;
                engine.exec_stack.values.push(Value::I64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Extend16S => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Extend16S operand not i64")
                })?;
                // Sign-extend from 16 bits to 64 bits
                let result = (a as i16) as i64;
                engine.exec_stack.values.push(Value::I64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Extend32S => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Extend32S operand not i64")
                })?;
                // Sign-extend from 32 bits to 64 bits
                let result = (a as i32) as i64;
                engine.exec_stack.values.push(Value::I64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // Bit counting operations
            Instruction::I32Clz => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Clz operand not i32")
                })?;
                // Count leading zeros
                let result = a.leading_zeros() as i32;
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32Ctz => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Ctz operand not i32")
                })?;
                // Count trailing zeros
                let result = a.trailing_zeros() as i32;
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I32Popcnt => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32Popcnt operand not i32")
                })?;
                // Count number of 1 bits
                let result = a.count_ones() as i32;
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Clz => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Clz operand not i64")
                })?;
                // Count leading zeros
                let result = a.leading_zeros() as i64;
                engine.exec_stack.values.push(Value::I64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Ctz => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Ctz operand not i64")
                })?;
                // Count trailing zeros
                let result = a.trailing_zeros() as i64;
                engine.exec_stack.values.push(Value::I64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::I64Popcnt => {
                let a = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?.and_then(|v| v.as_i64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I64Popcnt operand not i64")
                })?;
                // Count number of 1 bits
                let result = a.count_ones() as i64;
                engine.exec_stack.values.push(Value::I64(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // Floating-point min/max/copysign operations
            Instruction::F32Min => {
                let b_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let b = b_val.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Min second operand not f32")
                })?;
                let a_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let a = a_val.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Min first operand not f32")
                })?;
                // WebAssembly min: NaN if either operand is NaN, otherwise the smaller value
                let result = if a.is_nan() || b.is_nan() {
                    f32::NAN
                } else if a == 0.0 && b == 0.0 {
                    // -0.0 is smaller than +0.0
                    if a.is_sign_negative() || b.is_sign_negative() {
                        -0.0
                    } else {
                        0.0
                    }
                } else {
                    a.min(b)
                };
                engine.exec_stack.values.push(Value::F32(FloatBits32::from_float(result))).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32Max => {
                let b_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let b = b_val.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Max second operand not f32")
                })?;
                let a_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let a = a_val.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Max first operand not f32")
                })?;
                // WebAssembly max: NaN if either operand is NaN, otherwise the larger value
                let result = if a.is_nan() || b.is_nan() {
                    f32::NAN
                } else if a == 0.0 && b == 0.0 {
                    // +0.0 is larger than -0.0
                    if a.is_sign_positive() || b.is_sign_positive() {
                        0.0
                    } else {
                        -0.0
                    }
                } else {
                    a.max(b)
                };
                engine.exec_stack.values.push(Value::F32(FloatBits32::from_float(result))).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F32Copysign => {
                let b_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let b = b_val.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Copysign second operand not f32")
                })?;
                let a_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let a = a_val.and_then(|v| v.as_f32()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F32Copysign first operand not f32")
                })?;
                // Copy sign from b to a
                let result = a.copysign(b);
                engine.exec_stack.values.push(Value::F32(FloatBits32::from_float(result))).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64Min => {
                let b_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let b = b_val.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Min second operand not f64")
                })?;
                let a_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let a = a_val.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Min first operand not f64")
                })?;
                // WebAssembly min: NaN if either operand is NaN, otherwise the smaller value
                let result = if a.is_nan() || b.is_nan() {
                    f64::NAN
                } else if a == 0.0 && b == 0.0 {
                    // -0.0 is smaller than +0.0
                    if a.is_sign_negative() || b.is_sign_negative() {
                        -0.0
                    } else {
                        0.0
                    }
                } else {
                    a.min(b)
                };
                engine.exec_stack.values.push(Value::F64(FloatBits64::from_float(result))).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64Max => {
                let b_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let b = b_val.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Max second operand not f64")
                })?;
                let a_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let a = a_val.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Max first operand not f64")
                })?;
                // WebAssembly max: NaN if either operand is NaN, otherwise the larger value
                let result = if a.is_nan() || b.is_nan() {
                    f64::NAN
                } else if a == 0.0 && b == 0.0 {
                    // +0.0 is larger than -0.0
                    if a.is_sign_positive() || b.is_sign_positive() {
                        0.0
                    } else {
                        -0.0
                    }
                } else {
                    a.max(b)
                };
                engine.exec_stack.values.push(Value::F64(FloatBits64::from_float(result))).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::F64Copysign => {
                let b_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let b = b_val.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Copysign second operand not f64")
                })?;
                let a_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let a = a_val.and_then(|v| v.as_f64()).ok_or_else(|| {
                    Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "F64Copysign first operand not f64")
                })?;
                // Copy sign from b to a
                let result = a.copysign(b);
                engine.exec_stack.values.push(Value::F64(FloatBits64::from_float(result))).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // Reference type instructions
            Instruction::RefNull(ref_type) => {
                let null_value = match ref_type.to_value_type() {
                    ValueType::FuncRef => Value::FuncRef(None),
                    ValueType::ExternRef => Value::ExternRef(None),
                    _ => return Err(Error::new(
                        ErrorCategory::Validation,
                        codes::TYPE_MISMATCH_ERROR,
                        "RefNull with invalid reference type"
                    )),
                };
                engine.exec_stack.values.push(null_value).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::RefIsNull => {
                let ref_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let is_null = match ref_val {
                    Value::FuncRef(opt_ref) => opt_ref.is_none(),
                    Value::ExternRef(opt_ref) => opt_ref.is_none(),
                    _ => return Err(Error::new(
                        ErrorCategory::Validation,
                        codes::TYPE_MISMATCH_ERROR,
                        "RefIsNull operand is not a reference type"
                    )),
                };
                let result = if is_null { 1i32 } else { 0i32 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::RefFunc(func_idx) => {
                // Validate that the function index exists
                let module = self.module_instance.module();
                if func_idx >= module.functions.len() as u32 {
                    return Err(Error::new(
                        ErrorCategory::Validation,
                        codes::INVALID_FUNCTION_INDEX,
                        "Stack operation error"
                    ));
                }
                let func_ref = Value::FuncRef(Some(FuncRef::from_index(func_idx)));
                engine.exec_stack.values.push(func_ref).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // Stack operations
            Instruction::Drop => {
                engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::Select => {
                let condition_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let condition = match condition_val {
                    Value::I32(val) => val,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "Select condition not i32")),
                };
                let val2 = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let val1 = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let result = if condition != 0 { val1 } else { val2 };
                engine.exec_stack.values.push(result).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            Instruction::SelectWithType(_value_types) => {
                // SelectWithType behaves the same as Select for execution, the type information is for validation
                let condition_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let condition = match condition_val {
                    Value::I32(val) => val,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "SelectWithType condition not i32")),
                };
                let val2 = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let val1 = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let result = if condition != 0 { val1 } else { val2 };
                engine.exec_stack.values.push(result).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // Branch table instruction
            Instruction::BrTable { targets, default_target } => {
                let index_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let index = match index_val {
                    Value::I32(val) => val as usize,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "BrTable index not i32")),
                };
                
                // Select the target label: if index is in bounds, use targets[index], otherwise use default_target
                let target_label = if index < targets.len() {
                    targets.get(index).map_err(|e| {
                        Error::new(ErrorCategory::Runtime, codes::MEMORY_OUT_OF_BOUNDS, "Stack operation error")
                    })?
                } else {
                    default_target
                };
                
                // Perform the branch to the selected target
                self.branch_to_label(target_label, engine)?;
                Ok(ControlFlow::Branch(target_label))
            }
            
            // Advanced memory operations
            Instruction::MemoryFill(mem_idx) => {
                let size_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let size = match size_val {
                    Value::I32(val) => val as usize,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "MemoryFill size not i32")),
                };
                
                let value_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let value = match value_val {
                    Value::I32(val) => val as u8,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "MemoryFill value not i32")),
                };
                
                let offset_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let offset = match offset_val {
                    Value::I32(val) => val as usize,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "MemoryFill offset not i32")),
                };
                
                // Get the memory instance
                let memory = self.module_instance.memory(mem_idx)?;
                
                // Perform bounds check
                if offset + size > memory.size_in_bytes() {
                    return Err(Error::new(ErrorCategory::Runtime, codes::MEMORY_OUT_OF_BOUNDS, "MemoryFill operation out of bounds"));
                }
                
                // Fill memory with the specified value
                for i in 0..size {
                    memory.write_byte(offset + i, value).map_err(|e| {
                        Error::new(ErrorCategory::Runtime, codes::MEMORY_ACCESS_ERROR, "Stack operation error")
                    })?;
                }
                
                Ok(ControlFlow::Next)
            }
            
            Instruction::MemoryCopy(dst_mem_idx, src_mem_idx) => {
                let size_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let size = match size_val {
                    Value::I32(val) => val as usize,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "MemoryCopy size not i32")),
                };
                
                let src_offset_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let src_offset = match src_offset_val {
                    Value::I32(val) => val as usize,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "MemoryCopy src_offset not i32")),
                };
                
                let dst_offset_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let dst_offset = match dst_offset_val {
                    Value::I32(val) => val as usize,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "MemoryCopy dst_offset not i32")),
                };
                
                // Get memory instances
                let src_memory = self.module_instance.memory(src_mem_idx)?;
                let dst_memory = self.module_instance.memory(dst_mem_idx)?;
                
                // Perform bounds checks
                if src_offset + size > src_memory.size_in_bytes() {
                    return Err(Error::new(ErrorCategory::Runtime, codes::MEMORY_OUT_OF_BOUNDS, "MemoryCopy source out of bounds"));
                }
                if dst_offset + size > dst_memory.size_in_bytes() {
                    return Err(Error::new(ErrorCategory::Runtime, codes::MEMORY_OUT_OF_BOUNDS, "MemoryCopy destination out of bounds"));
                }
                
                // Copy memory (handle overlapping regions correctly)
                for i in 0..size {
                    let byte = src_memory.read_byte(src_offset + i).map_err(|e| {
                        Error::new(ErrorCategory::Runtime, codes::MEMORY_ACCESS_ERROR, "Stack operation error")
                    })?;
                    dst_memory.write_byte(dst_offset + i, byte).map_err(|e| {
                        Error::new(ErrorCategory::Runtime, codes::MEMORY_ACCESS_ERROR, "Stack operation error")
                    })?;
                }
                
                Ok(ControlFlow::Next)
            }
            
            Instruction::DataDrop(data_seg_idx) => {
                // Data segments are typically handled at module instantiation time
                // DataDrop marks a data segment as "dropped" to prevent further use in memory.init
                // For now, we'll implement this as a no-op since our current implementation
                // doesn't track active data segments at runtime
                
                // Validate that the data segment index is valid
                let module = self.module_instance.module();
                if data_seg_idx >= module.data.len() as u32 {
                    return Err(Error::new(
                        ErrorCategory::Validation,
                        codes::VALIDATION_INVALID_DATA_SEGMENT_INDEX,
                        "Stack operation error"
                    ));
                }
                
                // TODO: In a full implementation, mark the data segment as dropped
                // This would prevent future memory.init operations from using this segment
                
                Ok(ControlFlow::Next)
            }
            
            // Tail call instructions (WebAssembly 2.0)
            Instruction::ReturnCall(func_idx) => {
                // Validate function index
                let module = self.module_instance.module();
                if func_idx >= module.functions.len() as u32 {
                    return Err(Error::new(
                        ErrorCategory::Validation,
                        codes::INVALID_FUNCTION_INDEX,
                        "Stack operation error"
                    ));
                }
                
                // Return TailCall control flow to indicate frame replacement
                Ok(ControlFlow::TailCall(func_idx))
            }
            
            Instruction::ReturnCallIndirect(type_idx, table_idx) => {
                // Pop the function index from stack
                let func_index_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let func_index = match func_index_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "ReturnCallIndirect function index not i32")),
                };
                
                // Get table and validate index
                let table = self.module_instance.get_table(table_idx as usize).map_err(|_| {
                    Error::new(ErrorCategory::Validation, codes::VALIDATION_INVALID_TABLE_INDEX, "Stack operation error")
                })?;
                
                if func_index >= table.size() {
                    return Err(Error::new(ErrorCategory::Runtime, codes::MEMORY_OUT_OF_BOUNDS, "ReturnCallIndirect function index out of table bounds"));
                }
                
                // Get function reference from table
                let func_ref = table.get(func_index).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::MEMORY_ACCESS_ERROR, "Stack operation error")
                })?;
                
                let actual_func_idx = match func_ref {
                    Some(Value::FuncRef(Some(fref))) => fref.index,
                    Some(Value::FuncRef(None)) | None => {
                        return Err(Error::new(ErrorCategory::Runtime, codes::TYPE_MISMATCH_ERROR, "ReturnCallIndirect null function reference"));
                    }
                    _ => {
                        return Err(Error::new(ErrorCategory::Runtime, codes::TYPE_MISMATCH_ERROR, "ReturnCallIndirect invalid table element type"));
                    }
                };
                
                // Validate function type matches expected type
                let module = self.module_instance.module();
                let function = module.functions.get(actual_func_idx as usize).map_err(|_| {
                    Error::new(ErrorCategory::Validation, codes::INVALID_FUNCTION_INDEX, "Stack operation error")
                })?;
                
                if function.type_idx != type_idx {
                    return Err(Error::new(ErrorCategory::Runtime, codes::TYPE_MISMATCH_ERROR, "ReturnCallIndirect function type mismatch"));
                }
                
                // Return TailCall control flow for the resolved function
                Ok(ControlFlow::TailCall(actual_func_idx))
            }
            
            // Branch on null instructions (WebAssembly 2.0 GC)
            Instruction::BrOnNull(label_idx) => {
                let ref_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                
                let is_null = match ref_val {
                    Value::FuncRef(opt_ref) => opt_ref.is_none(),
                    Value::ExternRef(opt_ref) => opt_ref.is_none(),
                    Value::StructRef(opt_ref) => opt_ref.is_none(),
                    Value::ArrayRef(opt_ref) => opt_ref.is_none(),
                    _ => return Err(Error::new(
                        ErrorCategory::Validation,
                        codes::TYPE_MISMATCH_ERROR,
                        "BrOnNull operand is not a reference type"
                    )),
                };
                
                if is_null {
                    // Branch to the label
                    self.branch_to_label(label_idx, engine)?;
                    Ok(ControlFlow::Branch(label_idx as usize))
                } else {
                    // Push the non-null reference back onto stack and continue
                    engine.exec_stack.values.push(ref_val).map_err(|e| {
                        Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                    })?;
                    Ok(ControlFlow::Next)
                }
            }
            
            Instruction::BrOnNonNull(label_idx) => {
                let ref_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                
                let is_null = match ref_val {
                    Value::FuncRef(opt_ref) => opt_ref.is_none(),
                    Value::ExternRef(opt_ref) => opt_ref.is_none(),
                    Value::StructRef(opt_ref) => opt_ref.is_none(),
                    Value::ArrayRef(opt_ref) => opt_ref.is_none(),
                    _ => return Err(Error::new(
                        ErrorCategory::Validation,
                        codes::TYPE_MISMATCH_ERROR,
                        "BrOnNonNull operand is not a reference type"
                    )),
                };
                
                if !is_null {
                    // Push the non-null reference back onto stack and branch
                    engine.exec_stack.values.push(ref_val).map_err(|e| {
                        Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                    })?;
                    self.branch_to_label(label_idx, engine)?;
                    Ok(ControlFlow::Branch(label_idx as usize))
                } else {
                    // Reference is null, continue without branching (don't push null back)
                    Ok(ControlFlow::Next)
                }
            }
            
            // Memory initialization instruction
            Instruction::MemoryInit(data_seg_idx, mem_idx) => {
                let size_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let size = match size_val {
                    Value::I32(val) => val as usize,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "MemoryInit size not i32")),
                };
                
                let src_offset_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let src_offset = match src_offset_val {
                    Value::I32(val) => val as usize,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "MemoryInit src_offset not i32")),
                };
                
                let dst_offset_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let dst_offset = match dst_offset_val {
                    Value::I32(val) => val as usize,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "MemoryInit dst_offset not i32")),
                };
                
                // Validate memory index
                let memory = self.module_instance.memory(mem_idx)?;
                
                // Validate data segment index
                let module = self.module_instance.module();
                let data_segment = module.data.get(data_seg_idx as usize).map_err(|_| {
                    Error::new(ErrorCategory::Validation, codes::VALIDATION_INVALID_DATA_SEGMENT_INDEX, "Stack operation error")
                })?;
                
                // Bounds checks
                if dst_offset + size > memory.size_in_bytes() {
                    return Err(Error::new(ErrorCategory::Runtime, codes::MEMORY_OUT_OF_BOUNDS, "MemoryInit destination out of bounds"));
                }
                
                if src_offset + size > data_segment.data().len() {
                    return Err(Error::new(ErrorCategory::Runtime, codes::MEMORY_OUT_OF_BOUNDS, "MemoryInit source out of bounds"));
                }
                
                // Copy data from segment to memory
                for i in 0..size {
                    let byte = data_segment.data().get(src_offset + i).ok_or_else(|| {
                        Error::new(ErrorCategory::Runtime, codes::MEMORY_OUT_OF_BOUNDS, "MemoryInit data segment access out of bounds")
                    })?;
                    memory.write_byte(dst_offset + i, *byte).map_err(|e| {
                        Error::new(ErrorCategory::Runtime, codes::MEMORY_ACCESS_ERROR, "Stack operation error")
                    })?;
                }
                
                Ok(ControlFlow::Next)
            }
            
            // Additional reference operations (WebAssembly 2.0 GC)
            Instruction::RefAsNonNull => {
                let ref_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                
                let is_null = match ref_val {
                    Value::FuncRef(opt_ref) => opt_ref.is_none(),
                    Value::ExternRef(opt_ref) => opt_ref.is_none(),
                    Value::StructRef(opt_ref) => opt_ref.is_none(),
                    Value::ArrayRef(opt_ref) => opt_ref.is_none(),
                    _ => return Err(Error::new(
                        ErrorCategory::Validation,
                        codes::TYPE_MISMATCH_ERROR,
                        "RefAsNonNull operand is not a reference type"
                    )),
                };
                
                if is_null {
                    // Trap if reference is null
                    return Err(Error::new(
                        ErrorCategory::RuntimeTrap,
                        codes::EXECUTION_ERROR,
                        "RefAsNonNull: null reference"
                    ));
                } else {
                    // Push the non-null reference back onto stack
                    engine.exec_stack.values.push(ref_val).map_err(|e| {
                        Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                    })?;
                    Ok(ControlFlow::Next)
                }
            }
            
            Instruction::RefEq => {
                let ref2_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let ref1_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                
                // Compare references for equality
                let are_equal = match (&ref1_val, &ref2_val) {
                    (Value::FuncRef(opt1), Value::FuncRef(opt2)) => match (opt1, opt2) {
                        (None, None) => true,
                        (Some(ref1), Some(ref2)) => ref1.index == ref2.index,
                        _ => false,
                    },
                    (Value::ExternRef(opt1), Value::ExternRef(opt2)) => match (opt1, opt2) {
                        (None, None) => true,
                        (Some(ref1), Some(ref2)) => ref1.index == ref2.index,
                        _ => false,
                    },
                    (Value::StructRef(opt1), Value::StructRef(opt2)) => match (opt1, opt2) {
                        (None, None) => true,
                        (Some(ref1), Some(ref2)) => {
                            // For struct references, we compare by identity (same reference)
                            // In a full GC implementation, this would be pointer equality
                            ref1.type_index == ref2.type_index && ref1.fields == ref2.fields
                        },
                        _ => false,
                    },
                    (Value::ArrayRef(opt1), Value::ArrayRef(opt2)) => match (opt1, opt2) {
                        (None, None) => true,
                        (Some(ref1), Some(ref2)) => {
                            // For array references, we compare by identity (same reference)
                            ref1.type_index == ref2.type_index && ref1.elements == ref2.elements
                        },
                        _ => false,
                    },
                    _ => return Err(Error::new(
                        ErrorCategory::Validation,
                        codes::TYPE_MISMATCH_ERROR,
                        "RefEq: operands must be compatible reference types"
                    )),
                };
                
                let result = if are_equal { 1i32 } else { 0i32 };
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            // Atomic operations (WebAssembly Threads proposal)
            Instruction::MemoryAtomicNotify { memarg } => {
                let count_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let count = match count_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "MemoryAtomicNotify count not i32")),
                };
                
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "MemoryAtomicNotify addr not i32")),
                };
                
                // Calculate effective address with alignment check
                let effective_addr = addr + memarg.offset;
                if effective_addr % 4 != 0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::UNALIGNED_MEMORY_ACCESS, "MemoryAtomicNotify requires 4-byte alignment"));
                }
                
                // For now, implement as a no-op since we don't have a full threading model
                // In a full implementation, this would notify threads waiting on this memory location
                let woken_count = 0i32; // No threads to wake in current implementation
                
                engine.exec_stack.values.push(Value::I32(woken_count)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            Instruction::MemoryAtomicWait32 { memarg } => {
                let timeout_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let timeout = match timeout_val {
                    Value::I64(val) => val,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "MemoryAtomicWait32 timeout not i64")),
                };
                
                let expected_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let expected = match expected_val {
                    Value::I32(val) => val,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "MemoryAtomicWait32 expected not i32")),
                };
                
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "MemoryAtomicWait32 addr not i32")),
                };
                
                // Calculate effective address with alignment check
                let effective_addr = addr + memarg.offset;
                if effective_addr % 4 != 0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::UNALIGNED_MEMORY_ACCESS, "MemoryAtomicWait32 requires 4-byte alignment"));
                }
                
                // Get memory and read current value
                let memory = self.module_instance.get_memory(0).map_err(|_| {
                    Error::new(ErrorCategory::Validation, codes::VALIDATION_INVALID_MEMORY_INDEX, "No memory instance for atomic operation")
                })?;
                
                let current_val = memory.read_i32(effective_addr as usize).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::MEMORY_ACCESS_ERROR, "Stack operation error")
                })?;
                
                // Compare and return result
                let result = if current_val != expected {
                    1i32 // "not-equal"
                } else {
                    // In a full implementation, this would block the thread until notified or timeout
                    // For now, return "ok" (value was equal but we don't wait)
                    0i32 // "ok"
                };
                
                engine.exec_stack.values.push(Value::I32(result)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            Instruction::I32AtomicLoad { memarg } => {
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32AtomicLoad addr not i32")),
                };
                
                // Calculate effective address with alignment check
                let effective_addr = addr + memarg.offset;
                if effective_addr % 4 != 0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::UNALIGNED_MEMORY_ACCESS, "I32AtomicLoad requires 4-byte alignment"));
                }
                
                // Get memory and perform atomic load
                let memory = self.module_instance.get_memory(0).map_err(|_| {
                    Error::new(ErrorCategory::Validation, codes::VALIDATION_INVALID_MEMORY_INDEX, "No memory instance for atomic operation")
                })?;
                
                let value = memory.read_i32(effective_addr as usize).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::MEMORY_ACCESS_ERROR, "Stack operation error")
                })?;
                
                engine.exec_stack.values.push(Value::I32(value)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            Instruction::I32AtomicStore { memarg } => {
                let value_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let value = match value_val {
                    Value::I32(val) => val,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32AtomicStore value not i32")),
                };
                
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32AtomicStore addr not i32")),
                };
                
                // Calculate effective address with alignment check
                let effective_addr = addr + memarg.offset;
                if effective_addr % 4 != 0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::UNALIGNED_MEMORY_ACCESS, "I32AtomicStore requires 4-byte alignment"));
                }
                
                // Get memory and perform atomic store
                let memory = self.module_instance.get_memory(0).map_err(|_| {
                    Error::new(ErrorCategory::Validation, codes::VALIDATION_INVALID_MEMORY_INDEX, "No memory instance for atomic operation")
                })?;
                
                memory.write_i32(effective_addr as usize, value).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::MEMORY_ACCESS_ERROR, "Stack operation error")
                })?;
                
                Ok(ControlFlow::Next)
            }
            
            Instruction::I32AtomicRmwAdd { memarg } => {
                let value_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let value = match value_val {
                    Value::I32(val) => val,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32AtomicRmwAdd value not i32")),
                };
                
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32AtomicRmwAdd addr not i32")),
                };
                
                // Calculate effective address with alignment check
                let effective_addr = addr + memarg.offset;
                if effective_addr % 4 != 0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::UNALIGNED_MEMORY_ACCESS, "I32AtomicRmwAdd requires 4-byte alignment"));
                }
                
                // Get memory and perform atomic read-modify-write add
                let memory = self.module_instance.get_memory(0).map_err(|_| {
                    Error::new(ErrorCategory::Validation, codes::VALIDATION_INVALID_MEMORY_INDEX, "No memory instance for atomic operation")
                })?;
                
                let old_value = memory.read_i32(effective_addr as usize).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::MEMORY_ACCESS_ERROR, "Stack operation error")
                })?;
                
                let new_value = old_value.wrapping_add(value);
                memory.write_i32(effective_addr as usize, new_value).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::MEMORY_ACCESS_ERROR, "Stack operation error")
                })?;
                
                // Return the old value
                engine.exec_stack.values.push(Value::I32(old_value)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            Instruction::I32AtomicRmwCmpxchg { memarg } => {
                let replacement_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let replacement = match replacement_val {
                    Value::I32(val) => val,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32AtomicRmwCmpxchg replacement not i32")),
                };
                
                let expected_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let expected = match expected_val {
                    Value::I32(val) => val,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32AtomicRmwCmpxchg expected not i32")),
                };
                
                let addr_val = engine.exec_stack.values.pop().map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
                })?;
                let addr = match addr_val {
                    Value::I32(val) => val as u32,
                    _ => return Err(Error::new(ErrorCategory::Validation, codes::TYPE_MISMATCH_ERROR, "I32AtomicRmwCmpxchg addr not i32")),
                };
                
                // Calculate effective address with alignment check
                let effective_addr = addr + memarg.offset;
                if effective_addr % 4 != 0 {
                    return Err(Error::new(ErrorCategory::Runtime, codes::UNALIGNED_MEMORY_ACCESS, "I32AtomicRmwCmpxchg requires 4-byte alignment"));
                }
                
                // Get memory and perform atomic compare-exchange
                let memory = self.module_instance.get_memory(0).map_err(|_| {
                    Error::new(ErrorCategory::Validation, codes::VALIDATION_INVALID_MEMORY_INDEX, "No memory instance for atomic operation")
                })?;
                
                let current_value = memory.read_i32(effective_addr as usize).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::MEMORY_ACCESS_ERROR, "Stack operation error")
                })?;
                
                if current_value == expected {
                    // Values match, perform the exchange
                    memory.write_i32(effective_addr as usize, replacement).map_err(|e| {
                        Error::new(ErrorCategory::Runtime, codes::MEMORY_ACCESS_ERROR, "Stack operation error")
                    })?;
                }
                
                // Return the old value regardless of whether exchange occurred
                engine.exec_stack.values.push(Value::I32(current_value)).map_err(|e| {
                    Error::new(ErrorCategory::Runtime, codes::STACK_OVERFLOW, "Stack operation error")
                })?;
                Ok(ControlFlow::Next)
            }
            
            Instruction::AtomicFence => {
                // Atomic fence ensures memory ordering
                // In a single-threaded implementation, this is effectively a no-op
                // In a multi-threaded implementation, this would provide memory barriers
                Ok(ControlFlow::Next)
            }
            _ => {
                return Err(Error::new(
                    ErrorCategory::Runtime,
                    codes::UNSUPPORTED_OPERATION,
                    &format!(
                        "Instruction {:?} not yet implemented in StacklessFrame::step",
                        instruction
                    ),
                ));
            }
        }
        // If the instruction was handled and didn't return/trap/call/branch:
        if !matches!(
            instruction,
            Instruction::Unreachable | Instruction::Return // | Call | Br...
        ) {
            Ok(ControlFlow::Next)
        } else {
            // This branch should ideally not be hit if all control flow instrs return their
            // specific ControlFlow variant
            Err(Error::new(ErrorCategory::Runtime, codes::RUNTIME_ERROR, "Unhandled instruction outcome in step"))
        }
    }
}

// Helper methods for complex instructions, moved out of FrameBehavior::step
impl StacklessFrame {
    fn table_init(
        &mut self,
        elem_idx: u32,
        table_idx: u32,
        engine: &mut StacklessEngine,
    ) -> Result<()> {
        let module = self.module_instance.module();
        let segment = module.elements.get(elem_idx as usize).map_err(|_| {
            Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_INVALID_ELEMENT_INDEX,
                "Stack operation error",
            )
        })?;

        let len_val = engine.exec_stack.values.pop().map_err(|e| {
            Error::new(
                ErrorCategory::Runtime,
                codes::STACK_UNDERFLOW,
                "Stack operation error",
            )
        })?;
        let src_offset_val = engine.exec_stack.values.pop().map_err(|e| {
            Error::new(
                ErrorCategory::Runtime,
                codes::STACK_UNDERFLOW,
                "Stack operation error",
            )
        })?;
        let dst_offset_val = engine.exec_stack.values.pop().map_err(|e| {
            Error::new(
                ErrorCategory::Runtime,
                codes::STACK_UNDERFLOW,
                "Stack operation error",
            )
        })?;

        let n = len_val
            .and_then(|v| v.as_i32())
            .ok_or_else(|| Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "table.init len not i32"))?
            as u32;
        let src_offset = src_offset_val.and_then(|v| v.as_i32()).ok_or_else(|| {
            Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "table.init src_offset not i32")
        })? as u32;
        let dst_offset = dst_offset_val.and_then(|v| v.as_i32()).ok_or_else(|| {
            Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "table.init dst_offset not i32")
        })? as u32;

        // Bounds checks from Wasm spec:
        // dst_offset + n > table.len()
        // src_offset + n > segment.items.len()
        let table = self.module_instance.table(table_idx)?;
        if dst_offset.checked_add(n).map_or(true, |end| end > table.size())
            || src_offset.checked_add(n).map_or(true, |end| end as usize > segment.items.len())
        {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::OUT_OF_BOUNDS_ERROR,
                "table.init out of bounds",
            ));
        }

        if n == 0 {
            return Ok(());
        } // No-op

        // Assuming segment.items are Vec<u32> (function indices) or similar that can be
        // turned into Value::FuncRef This needs to align with how Element
        // segments store their items. If Element.items are already `Value` or
        // `Option<Value>`, this is simpler. Let's assume Element stores func
        // indices as u32.
        let items_to_init: Vec<Option<Value>> = segment
            .items
            .get(src_offset as usize..(src_offset + n) as usize)
            .ok_or_else(|| {
                Error::new(
                    ErrorCategory::Runtime,
                    codes::OUT_OF_BOUNDS_ERROR,
                    "table.init source slice OOB on segment items",
                )
            })?
            .iter()
            .map(|&func_idx| Some(Value::FuncRef(Some(FuncRef { index: func_idx })))) // Assuming items are u32 func indices
            .collect();

        table.init(dst_offset, &items_to_init)
    }

    fn table_copy(
        &mut self,
        dst_table_idx: u32,
        src_table_idx: u32,
        engine: &mut StacklessEngine,
    ) -> Result<()> {
        let len_val = engine.exec_stack.values.pop().map_err(|e| {
            Error::new(
                ErrorCategory::Runtime,
                codes::STACK_UNDERFLOW,
                "Stack operation error",
            )
        })?;
        let src_offset_val = engine.exec_stack.values.pop().map_err(|e| {
            Error::new(
                ErrorCategory::Runtime,
                codes::STACK_UNDERFLOW,
                "Stack operation error",
            )
        })?;
        let dst_offset_val = engine.exec_stack.values.pop().map_err(|e| {
            Error::new(
                ErrorCategory::Runtime,
                codes::STACK_UNDERFLOW,
                "Stack operation error",
            )
        })?;

        let n = len_val
            .and_then(|v| v.as_i32())
            .ok_or_else(|| Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "table.copy len not i32"))?
            as u32;
        let src_offset = src_offset_val.and_then(|v| v.as_i32()).ok_or_else(|| {
            Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "table.copy src_offset not i32")
        })? as u32;
        let dst_offset = dst_offset_val.and_then(|v| v.as_i32()).ok_or_else(|| {
            Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "table.copy dst_offset not i32")
        })? as u32;

        let dst_table = self.module_instance.table(dst_table_idx)?;
        let src_table = self.module_instance.table(src_table_idx)?;

        // Bounds checks (Wasm spec)
        if dst_offset.checked_add(n).map_or(true, |end| end > dst_table.size())
            || src_offset.checked_add(n).map_or(true, |end| end > src_table.size())
        {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::OUT_OF_BOUNDS_ERROR,
                "table.copy out of bounds",
            ));
        }

        if n == 0 {
            return Ok(());
        }

        // Perform copy: if ranges overlap, copy direction matters.
        // Wasm spec: "if s+n > d and s < d" (copy backwards) or "if d+n > s and d < s"
        // (copy forwards) Simplest is often to read all source elements first
        // if temp storage is okay. Or, copy element by element, checking
        // overlap for direction.
        if dst_offset <= src_offset {
            // Copy forwards
            for i in 0..n {
                let val = src_table.get(src_offset + i)?.ok_or_else(|| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::OUT_OF_BOUNDS_ERROR,
                        "table.copy source element uninitialized/null",
                    )
                })?;
                dst_table.set(dst_offset + i, val)?;
            }
        } else {
            // Copy backwards (dst_offset > src_offset)
            for i in (0..n).rev() {
                let val = src_table.get(src_offset + i)?.ok_or_else(|| {
                    Error::new(
                        ErrorCategory::Runtime,
                        codes::OUT_OF_BOUNDS_ERROR,
                        "table.copy source element uninitialized/null",
                    )
                })?;
                dst_table.set(dst_offset + i, val)?;
            }
        }
        Ok(())
    }

    fn table_fill(&mut self, table_idx: u32, engine: &mut StacklessEngine) -> Result<()> {
        let n_val = engine.exec_stack.values.pop().map_err(|e| {
            Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
        })?;
        let val_to_fill = engine.exec_stack.values.pop().map_err(|e| {
            Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
        })?;
        let offset_val = engine.exec_stack.values.pop().map_err(|e| {
            Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
        })?;

        let n = n_val
            .and_then(|v| v.as_i32())
            .ok_or_else(|| Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "table.fill count not i32"))?
            as u32;
        let offset = offset_val
            .and_then(|v| v.as_i32())
            .ok_or_else(|| Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "table.fill offset not i32"))?
            as u32;

        let table = self.module_instance.table(table_idx)?;
        if offset.checked_add(n).map_or(true, |end| end > table.size()) {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::OUT_OF_BOUNDS_ERROR,
                "table.fill out of bounds",
            ));
        }

        if n == 0 {
            return Ok(());
        }
        // TODO: Type check val_to_fill against table.element_type()
        for i in 0..n {
            table.set(offset + i, val_to_fill.clone())?;
        }
        Ok(())
    }

    // Memory operations
    fn memory_init(
        &mut self,
        data_idx: u32,
        mem_idx: u32,
        engine: &mut StacklessEngine,
    ) -> Result<()> {
        let n_val = engine.exec_stack.values.pop().map_err(|e| {
            Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
        })?;
        let src_offset_val = engine.exec_stack.values.pop().map_err(|e| {
            Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
        })?;
        let dst_offset_val = engine.exec_stack.values.pop().map_err(|e| {
            Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
        })?;

        let n = n_val
            .and_then(|v| v.as_i32())
            .ok_or_else(|| Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "memory.init len not i32"))?
            as usize;
        let src_offset = src_offset_val.and_then(|v| v.as_i32()).ok_or_else(|| {
            Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "memory.init src_offset not i32")
        })? as usize;
        let dst_offset = dst_offset_val.and_then(|v| v.as_i32()).ok_or_else(|| {
            Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "memory.init dst_offset not i32")
        })? as usize;

        let memory = self.module_instance.memory(mem_idx)?;
        let data_segment =
            self.module_instance.module().data.get(data_idx as usize).map_err(|_| {
                    Error::new(
                        ErrorCategory::Validation,
                        codes::VALIDATION_INVALID_DATA_SEGMENT_INDEX,
                        "Stack operation error",
                    )
                },
            )?;

        // Bounds checks (Wasm Spec)
        if dst_offset.checked_add(n).map_or(true, |end| end > memory.size_bytes())
            || src_offset.checked_add(n).map_or(true, |end| end > data_segment.data.len())
        {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                "memory.init out of bounds",
            ));
        }
        if n == 0 {
            return Ok(());
        }

        let data_to_write = data_segment.data.get(src_offset..src_offset + n).ok_or_else(|| {
            Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                "memory.init source data segment OOB",
            )
        })?;

        memory.write(dst_offset, data_to_write)
    }

    fn memory_copy(
        &mut self,
        dst_mem_idx: u32,
        src_mem_idx: u32,
        engine: &mut StacklessEngine,
    ) -> Result<()> {
        // In Wasm MVP, src_mem_idx and dst_mem_idx are always 0.
        let n_val = engine.exec_stack.values.pop().map_err(|e| {
            Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
        })?;
        let src_offset_val = engine.exec_stack.values.pop().map_err(|e| {
            Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
        })?;
        let dst_offset_val = engine.exec_stack.values.pop().map_err(|e| {
            Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
        })?;

        let n = n_val
            .and_then(|v| v.as_i32())
            .ok_or_else(|| Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "memory.copy len not i32"))?
            as usize;
        let src_offset = src_offset_val.and_then(|v| v.as_i32()).ok_or_else(|| {
            Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "memory.copy src_offset not i32")
        })? as usize;
        let dst_offset = dst_offset_val.and_then(|v| v.as_i32()).ok_or_else(|| {
            Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "memory.copy dst_offset not i32")
        })? as usize;

        let dst_memory = self.module_instance.memory(dst_mem_idx)?;
        let src_memory = if dst_mem_idx == src_mem_idx {
            Arc::clone(&dst_memory)
        } else {
            self.module_instance.memory(src_mem_idx)?
        };

        // Bounds checks
        if dst_offset.checked_add(n).map_or(true, |end| end > dst_memory.size_bytes())
            || src_offset.checked_add(n).map_or(true, |end| end > src_memory.size_bytes())
        {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                "memory.copy out of bounds",
            ));
        }
        if n == 0 {
            return Ok(());
        }

        // Wasm spec: if d_m is m and s_m is m, then the copy is performed as if the
        // bytes are copied from m to a temporary buffer of size n and then from the
        // buffer to m. This means we can read all source bytes then write, or
        // handle overlap carefully. For simplicity, if it's the same memory and
        // regions overlap, a temporary buffer is safest. Otherwise (different
        // memories, or same memory but no overlap), direct copy is fine.

        // A simple approach that is correct but might be slower if n is large:
        #[cfg(feature = "std")]
        let mut temp_buffer = vec![0u8; n];
        #[cfg(all(not(feature = "std"), not(feature = "std")))]
        let mut temp_buffer = {
            let mut buf = wrt_foundation::bounded::BoundedVec::new(wrt_foundation::safe_memory::NoStdProvider::<1024>::default()).unwrap();
            for _ in 0..n.min(4096) {
                buf.push(0u8).unwrap();
            }
            buf
        };
        src_memory.read(src_offset, &mut temp_buffer)?;
        dst_memory.write(dst_offset, &temp_buffer)
    }

    fn memory_fill(&mut self, mem_idx: u32, engine: &mut StacklessEngine) -> Result<()> {
        let n_val = engine.exec_stack.values.pop().map_err(|e| {
            Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
        })?;
        let val_to_fill_val = engine.exec_stack.values.pop().map_err(|e| {
            Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
        })?;
        let dst_offset_val = engine.exec_stack.values.pop().map_err(|e| {
            Error::new(ErrorCategory::Runtime, codes::STACK_UNDERFLOW, "Stack operation error")
        })?;

        let n = n_val
            .and_then(|v| v.as_i32())
            .ok_or_else(|| Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "memory.fill len not i32"))?
            as usize;
        let val_to_fill_byte = val_to_fill_val
            .and_then(|v| v.as_i32())
            .ok_or_else(|| Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "memory.fill value not i32"))?
            as u8; // Value must be i32, truncated to u8
        let dst_offset = dst_offset_val.and_then(|v| v.as_i32()).ok_or_else(|| {
            Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, "memory.fill dst_offset not i32")
        })? as usize;

        let memory = self.module_instance.memory(mem_idx)?;
        if dst_offset.checked_add(n).map_or(true, |end| end > memory.size_bytes()) {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                "memory.fill out of bounds",
            ));
        }
        if n == 0 {
            return Ok(());
        }

        memory.fill(dst_offset, val_to_fill_byte, n)
    }

    // TODO: Add methods for enter_block, exit_block, branch_to_label, etc.
    // These will manipulate self.block_depths and self.pc, and interact with
    // engine.exec_stack.values.
}

// Validatable might not be applicable directly to StacklessFrame in the same
// way as Module. If it's for ensuring internal consistency, it might be useful.
impl Validatable for StacklessFrame {
    type Error = Error;
    
    fn validation_level(&self) -> VerificationLevel {
        VerificationLevel::Basic
    }
    
    fn set_validation_level(&mut self, _level: VerificationLevel) {
        // Validation level is fixed for frames
    }
    
    fn validate(&self) -> Result<()> {
        // Example validations:
        // - self.pc should be within bounds of function code
        // - self.locals should match arity + declared locals of self.func_type
        // - self.block_depths should be consistent (e.g. not deeper than allowed)
        if self.pc > self.function_body()?.body.len() {
            return Err(Error::new(ErrorCategory::Runtime, codes::OUT_OF_BOUNDS_ERROR, "PC out of bounds"));
        }
        // More checks can be added here.
        Ok(())
    }
}
