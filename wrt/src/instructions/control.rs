//! WebAssembly control flow instructions
//!
//! This module contains implementations for all WebAssembly control flow instructions,
//! including blocks, branches, calls, and returns.

use crate::{
    behavior::{ControlFlowBehavior, FrameBehavior, InstructionExecutor, Label},
    error::{Error, Result},
    stack::Stack,
    types::{BlockType, FuncType},
    values::Value,
};

#[cfg(feature = "std")]
use std::vec;

#[cfg(not(feature = "std"))]
use alloc::vec;

/// Label type for control flow
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabelType {
    /// Block instruction label
    Block,
    /// Loop instruction label
    Loop,
    /// If instruction label
    If,
}

/// Create and push a new label onto the label stack
///
/// This is used internally by block, loop, and if instructions.
pub fn push_label<S: Stack>(
    pc: usize,
    stack: &mut S,
    _label_type: LabelType,
    block_type: BlockType,
    function_types: Option<&[FuncType]>,
) -> Result<()> {
    // Determine the function type for this block
    let func_type = match block_type {
        BlockType::Empty => FuncType {
            params: vec![],
            results: vec![],
        },
        BlockType::Type(value_type) => FuncType {
            params: vec![],
            results: vec![value_type],
        },
        BlockType::Value(value_type) => FuncType {
            params: vec![],
            results: vec![value_type],
        },
        BlockType::FuncType(func_type) => func_type,
        BlockType::TypeIndex(type_idx) => {
            if let Some(types) = function_types {
                if let Some(ty) = types.get(type_idx as usize) {
                    ty.clone()
                } else {
                    return Err(Error::Execution(format!("Invalid type index: {type_idx}")));
                }
            } else {
                return Err(Error::Execution(format!("Invalid type index: {type_idx}")));
            }
        }
    };

    // Create a Label with all required fields
    let label = crate::stack::Label {
        arity: func_type.results.len(),
        pc,
        continuation: 0, // Default value, should be set by caller if needed
    };

    // Push the new label using the Stack trait's method
    <S as Stack>::push_label(stack, label)?;

    Ok(())
}

#[derive(Debug)]
pub struct Block {
    pub block_type: BlockType,
    pub instructions: Vec<Box<dyn InstructionExecutor>>,
}

impl InstructionExecutor for Block {
    fn execute(&self, stack: &mut dyn Stack, frame: &mut dyn FrameBehavior) -> Result<()> {
        // Save current state
        let current_arity = frame.arity();
        let current_label_arity = frame.label_arity();

        // Set up new block
        let new_arity = match &self.block_type {
            BlockType::Empty => 0,
            BlockType::Value(_val_type) => 1,
            BlockType::Type(_val_type) => 1,
            BlockType::TypeIndex(_) => 1, // Assuming type index always results in a single value
            BlockType::FuncType(func_type) => {
                if func_type.results.is_empty() {
                    0
                } else {
                    func_type.results.len()
                }
            }
        };
        frame.set_arity(new_arity);
        frame.set_label_arity(new_arity);

        // Pre-compute values for the label
        let arity = frame.label_arity();
        let pc = frame.pc();
        let continuation = frame.return_pc() + 1;

        // Push label with the pre-computed values
        frame.label_stack().push(Label {
            arity,
            pc,
            continuation,
        });

        // Execute instructions
        for instruction in &self.instructions {
            instruction.execute(stack, frame)?;
        }

        // Restore state
        frame.label_stack().pop();
        frame.set_arity(current_arity);
        frame.set_label_arity(current_label_arity);

        Ok(())
    }
}

#[derive(Debug)]
pub struct Loop {
    pub block_type: BlockType,
    pub instructions: Vec<Box<dyn InstructionExecutor>>,
}

impl InstructionExecutor for Loop {
    fn execute(&self, stack: &mut dyn Stack, frame: &mut dyn FrameBehavior) -> Result<()> {
        // Save current state
        let current_arity = frame.arity();
        let current_label_arity = frame.label_arity();

        // Set up new loop
        let new_arity = match &self.block_type {
            BlockType::Empty => 0,
            BlockType::Value(_val_type) => 1,
            BlockType::Type(_val_type) => 1,
            BlockType::TypeIndex(_) => 1, // Assuming type index always results in a single value
            BlockType::FuncType(func_type) => {
                if func_type.results.is_empty() {
                    0
                } else {
                    func_type.results.len()
                }
            }
        };
        frame.set_arity(new_arity);
        frame.set_label_arity(0); // Loops branch to the start

        // Pre-compute values for the label
        let arity = frame.label_arity();
        let pc = frame.pc();
        let continuation = frame.return_pc();

        // Push label with the pre-computed values
        frame.label_stack().push(Label {
            arity,
            pc,
            continuation,
        });

        // Execute instructions
        for instruction in &self.instructions {
            instruction.execute(stack, frame)?;
        }

        // Restore state
        frame.label_stack().pop();
        frame.set_arity(current_arity);
        frame.set_label_arity(current_label_arity);

        Ok(())
    }
}

#[derive(Debug)]
pub struct If {
    pub block_type: BlockType,
    pub if_instructions: Vec<Box<dyn InstructionExecutor>>,
    pub else_instructions: Vec<Box<dyn InstructionExecutor>>,
}

impl InstructionExecutor for If {
    fn execute(&self, stack: &mut dyn Stack, frame: &mut dyn FrameBehavior) -> Result<()> {
        let condition = stack.pop()?;
        let condition_value = match condition {
            Value::I32(0) => false,
            Value::I32(_) => true,
            _ => return Err(Error::Execution("If condition must be i32".into())),
        };

        // Save current state
        let current_arity = frame.arity();
        let current_label_arity = frame.label_arity();

        // Set up new if block
        let new_arity = match &self.block_type {
            BlockType::Empty => 0,
            BlockType::Value(_val_type) => 1,
            BlockType::Type(_val_type) => 1,
            BlockType::TypeIndex(_) => 1, // Assuming type index always results in a single value
            BlockType::FuncType(func_type) => {
                if func_type.results.is_empty() {
                    0
                } else {
                    func_type.results.len()
                }
            }
        };
        frame.set_arity(new_arity);
        frame.set_label_arity(new_arity);

        // Pre-compute values for the label
        let arity = frame.label_arity();
        let pc = frame.pc();
        let continuation = frame.return_pc() + 1;

        // Push label with the pre-computed values
        frame.label_stack().push(Label {
            arity,
            pc,
            continuation,
        });

        // Execute the appropriate branch
        let instructions = if condition_value {
            &self.if_instructions
        } else {
            &self.else_instructions
        };

        for instruction in instructions {
            instruction.execute(stack, frame)?;
        }

        // Restore state
        frame.label_stack().pop();
        frame.set_arity(current_arity);
        frame.set_label_arity(current_label_arity);

        Ok(())
    }
}

#[derive(Debug)]
pub struct Br {
    pub label_idx: u32,
}

impl InstructionExecutor for Br {
    fn execute(&self, _stack: &mut dyn Stack, frame: &mut dyn FrameBehavior) -> Result<()> {
        // Get the label continuation address
        let label_stack = frame.label_stack();
        let label = label_stack
            .get(self.label_idx as usize)
            .ok_or_else(|| Error::Execution("Branch target out of bounds".into()))?;
        let continuation = label.continuation;

        // Set the return PC
        frame.set_return_pc(continuation);
        Ok(())
    }
}

#[derive(Debug)]
pub struct BrIf {
    pub label_idx: u32,
}

impl InstructionExecutor for BrIf {
    fn execute(&self, stack: &mut dyn Stack, frame: &mut dyn FrameBehavior) -> Result<()> {
        let condition = stack.pop()?;
        let condition_value = match condition {
            Value::I32(0) => false,
            Value::I32(_) => true,
            _ => return Err(Error::Execution("BrIf condition must be i32".into())),
        };

        if condition_value {
            // Get the label continuation address
            let label_stack = frame.label_stack();
            let label = label_stack
                .get(self.label_idx as usize)
                .ok_or_else(|| Error::Execution("Branch target out of bounds".into()))?;
            let continuation = label.continuation;

            // Set the return PC
            frame.set_return_pc(continuation);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct BrTable {
    pub labels: Vec<u32>,
    pub default_label: u32,
}

impl InstructionExecutor for BrTable {
    fn execute(&self, stack: &mut dyn Stack, frame: &mut dyn FrameBehavior) -> Result<()> {
        let index = stack.pop()?;
        let index_value = match index {
            Value::I32(i) => i as usize,
            _ => return Err(Error::Execution("BrTable index must be i32".into())),
        };

        let label_idx = self
            .labels
            .get(index_value)
            .copied()
            .unwrap_or(self.default_label);

        // Get the label continuation address
        let label_stack = frame.label_stack();
        let label = label_stack
            .get(label_idx as usize)
            .ok_or_else(|| Error::Execution("Branch target out of bounds".into()))?;
        let continuation = label.continuation;

        // Set the return PC
        frame.set_return_pc(continuation);
        Ok(())
    }
}

#[derive(Debug)]
pub struct Return;

impl InstructionExecutor for Return {
    fn execute(&self, _stack: &mut dyn Stack, frame: &mut dyn FrameBehavior) -> Result<()> {
        frame.set_return_pc(usize::MAX);
        Ok(())
    }
}

#[derive(Debug)]
pub struct Call {
    pub func_idx: u32,
}

impl InstructionExecutor for Call {
    fn execute(&self, stack: &mut dyn Stack, frame: &mut dyn FrameBehavior) -> Result<()> {
        // Call the function
        frame.call(self.func_idx, stack)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct CallIndirect {
    pub type_idx: u32,
    pub table_idx: u32,
}

impl InstructionExecutor for CallIndirect {
    fn execute(&self, stack: &mut dyn Stack, frame: &mut dyn FrameBehavior) -> Result<()> {
        let table_entry = stack.pop()?;
        match table_entry {
            Value::I32(entry) => {
                // Convert i32 to u32
                let entry_u32 = entry as u32;
                frame.call_indirect(self.type_idx, self.table_idx, entry_u32, stack)?;
                Ok(())
            }
            _ => Err(Error::InvalidType("Expected i32".to_string())),
        }
    }
}

/// Executes an unreachable instruction
pub fn unreachable(
    _stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    Err(Error::Execution(
        "unreachable instruction executed".to_string(),
    ))
}

/// Executes a nop instruction
pub fn nop<S: Stack + ?Sized>(
    _stack: &mut S,
    _frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    // No operation
    Ok(())
}

/// Executes an else instruction
pub fn else_instr<S: Stack>(_stack: &mut S) -> Result<()> {
    // We handle this in the execution loop
    Ok(())
}

/// Execute a block instruction
pub fn block(
    stack: &mut impl Stack,
    frame: &mut (impl FrameBehavior + ?Sized),
    ty: FuncType,
) -> Result<()> {
    frame.enter_block(BlockType::FuncType(ty), stack.len())?;
    Ok(())
}

/// Execute a loop instruction
pub fn loop_(
    stack: &mut impl Stack,
    frame: &mut (impl FrameBehavior + ?Sized),
    ty: FuncType,
) -> Result<()> {
    frame.enter_loop(BlockType::FuncType(ty), stack.len())?;
    Ok(())
}

/// Execute an if instruction
pub fn if_(
    stack: &mut impl Stack,
    frame: &mut (impl FrameBehavior + ?Sized),
    ty: FuncType,
) -> Result<()> {
    // Pop the condition from the stack
    let condition = stack.pop()?;
    let condition_bool = match condition {
        Value::I32(0) => false,
        Value::I32(_) => true,
        _ => return Err(Error::InvalidType("Expected i32 for condition".to_string())),
    };

    frame.enter_if(BlockType::FuncType(ty), stack.len(), condition_bool)?;
    Ok(())
}

/// Execute an else instruction
pub fn else_(stack: &mut impl Stack, frame: &mut (impl FrameBehavior + ?Sized)) -> Result<()> {
    frame.enter_else(stack.len())?;
    Ok(())
}

/// Execute an end instruction
pub fn end(stack: &mut impl Stack, frame: &mut (impl FrameBehavior + ?Sized)) -> Result<()> {
    frame.exit_block(stack)?;
    Ok(())
}

pub fn end_dyn(stack: &mut dyn Stack, frame: &mut dyn FrameBehavior) -> Result<()> {
    if let Some(frame_concrete) = frame.as_any().downcast_mut::<crate::module::Module>() {
        frame_concrete.exit_block(stack)?;
        Ok(())
    } else if let Some(frame_concrete) = frame
        .as_any()
        .downcast_mut::<crate::stackless::StacklessFrame>()
    {
        frame_concrete.exit_block(stack)?;
        Ok(())
    } else {
        Err(Error::Execution("Failed to downcast frame".to_string()))
    }
}

pub fn br(
    stack: &mut impl Stack,
    frame: &mut (impl FrameBehavior + ?Sized),
    label_idx: u32,
) -> Result<()> {
    frame.branch(label_idx, stack)?;
    Ok(())
}

pub fn br_if(
    stack: &mut impl Stack,
    frame: &mut (impl FrameBehavior + ?Sized),
    label_idx: u32,
) -> Result<()> {
    let condition = stack.pop()?;
    match condition {
        Value::I32(c) => {
            if c != 0 {
                frame.branch(label_idx, stack)?;
            }
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn br_table(
    stack: &mut impl Stack,
    frame: &mut (impl FrameBehavior + ?Sized),
    labels: Vec<u32>,
    default: u32,
) -> Result<()> {
    let index = stack.pop()?;
    match index {
        Value::I32(i) => {
            let label_idx = if (i as usize) < labels.len() {
                labels[i as usize]
            } else {
                default
            };
            frame.branch(label_idx, stack)?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn return_(stack: &mut impl Stack, frame: &mut (impl FrameBehavior + ?Sized)) -> Result<()> {
    frame.return_(stack)?;
    Ok(())
}

pub fn call(
    stack: &mut impl Stack,
    frame: &mut (impl FrameBehavior + ?Sized),
    func_idx: u32,
) -> Result<()> {
    frame.call(func_idx, stack)?;
    Ok(())
}

pub fn call_indirect(
    stack: &mut impl Stack,
    frame: &mut (impl FrameBehavior + ?Sized),
    type_idx: u32,
    table_idx: u32,
) -> Result<()> {
    let table_entry = stack.pop()?;
    match table_entry {
        Value::I32(entry) => {
            // Convert i32 to u32
            let entry_u32 = entry as u32;
            frame.call_indirect(type_idx, table_idx, entry_u32, stack)?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

// Create adapter functions that accept dyn Stack
pub fn block_dyn(
    _stack: &mut dyn Stack,
    frame: &mut dyn FrameBehavior,
    ty: BlockType,
) -> Result<()> {
    // Save current state
    let _current_arity = frame.arity();
    let _current_label_arity = frame.label_arity();

    // Set up new block
    let new_arity = match &ty {
        BlockType::Empty => 0,
        BlockType::Value(_val_type) => 1,
        BlockType::Type(_val_type) => 1,
        BlockType::TypeIndex(_) => 1, // Assuming type index always results in a single value
        BlockType::FuncType(func_type) => {
            if func_type.results.is_empty() {
                0
            } else {
                func_type.results.len()
            }
        }
    };
    frame.set_arity(new_arity);
    frame.set_label_arity(new_arity);

    // Copy values first to avoid borrow issues
    let pc = frame.pc();
    let continuation = pc + 1;

    // Push label
    frame.label_stack().push(Label {
        arity: new_arity,
        pc,
        continuation,
    });

    Ok(())
}

pub fn loop_dyn(
    _stack: &mut dyn Stack,
    frame: &mut dyn FrameBehavior,
    ty: BlockType,
) -> Result<()> {
    // Save current state
    let _current_arity = frame.arity();
    let _current_label_arity = frame.label_arity();

    // Set up new loop
    let new_arity = match &ty {
        BlockType::Empty => 0,
        BlockType::Value(_val_type) => 1,
        BlockType::Type(_val_type) => 1,
        BlockType::TypeIndex(_) => 1, // Assuming type index always results in a single value
        BlockType::FuncType(func_type) => {
            if func_type.results.is_empty() {
                0
            } else {
                func_type.results.len()
            }
        }
    };
    frame.set_arity(new_arity);
    frame.set_label_arity(0); // Loops branch to the start

    // Copy values first to avoid borrow issues
    let pc = frame.pc();

    // Push label
    frame.label_stack().push(Label {
        arity: 0,
        pc,
        continuation: pc,
    });

    Ok(())
}

pub fn if_dyn(stack: &mut dyn Stack, frame: &mut dyn FrameBehavior, ty: BlockType) -> Result<()> {
    // Pop the condition from the stack
    let condition = stack.pop()?;
    let _condition_bool = match condition {
        Value::I32(0) => false,
        Value::I32(_) => true,
        _ => return Err(Error::InvalidType("Expected i32 for condition".to_string())),
    };

    // Save current state
    let _current_arity = frame.arity();
    let _current_label_arity = frame.label_arity();

    // Set up new if
    let new_arity = match &ty {
        BlockType::Empty => 0,
        BlockType::Value(_val_type) => 1,
        BlockType::Type(_val_type) => 1,
        BlockType::TypeIndex(_) => 1, // Assuming type index always results in a single value
        BlockType::FuncType(func_type) => {
            if func_type.results.is_empty() {
                0
            } else {
                func_type.results.len()
            }
        }
    };
    frame.set_arity(new_arity);
    frame.set_label_arity(new_arity);

    // Copy values first to avoid borrow issues
    let pc = frame.pc();
    let continuation = pc + 1;

    // Push label
    frame.label_stack().push(Label {
        arity: new_arity,
        pc,
        continuation,
    });

    Ok(())
}

pub fn else_dyn(stack: &mut dyn Stack, frame: &mut dyn FrameBehavior) -> Result<()> {
    if let Some(frame_concrete) = frame.as_any().downcast_mut::<crate::module::Module>() {
        frame_concrete.enter_else(stack.len())?;
        Ok(())
    } else if let Some(frame_concrete) = frame
        .as_any()
        .downcast_mut::<crate::stackless::StacklessFrame>()
    {
        frame_concrete.enter_else(stack.len())?;
        Ok(())
    } else {
        Err(Error::Execution("Failed to downcast frame".to_string()))
    }
}

pub fn br_dyn(stack: &mut dyn Stack, frame: &mut dyn FrameBehavior, label_idx: u32) -> Result<()> {
    if let Some(frame_concrete) = frame.as_any().downcast_mut::<crate::module::Module>() {
        frame_concrete.branch(label_idx, stack)?;
        Ok(())
    } else if let Some(frame_concrete) = frame
        .as_any()
        .downcast_mut::<crate::stackless::StacklessFrame>()
    {
        frame_concrete.branch(label_idx, stack)?;
        Ok(())
    } else {
        Err(Error::Execution("Failed to downcast frame".to_string()))
    }
}

pub fn br_if_dyn(
    stack: &mut dyn Stack,
    frame: &mut dyn FrameBehavior,
    label_idx: u32,
) -> Result<()> {
    let condition = stack.pop()?;
    match condition {
        Value::I32(c) => {
            if c != 0 {
                if let Some(frame_concrete) = frame.as_any().downcast_mut::<crate::module::Module>()
                {
                    frame_concrete.branch(label_idx, stack)?;
                    Ok(())
                } else if let Some(frame_concrete) = frame
                    .as_any()
                    .downcast_mut::<crate::stackless::StacklessFrame>()
                {
                    frame_concrete.branch(label_idx, stack)?;
                    Ok(())
                } else {
                    Err(Error::Execution("Failed to downcast frame".to_string()))
                }
            } else {
                Ok(())
            }
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn br_table_dyn(
    stack: &mut dyn Stack,
    frame: &mut dyn FrameBehavior,
    labels: Vec<u32>,
    default: u32,
) -> Result<()> {
    let index = stack.pop()?;
    match index {
        Value::I32(i) => {
            let label_idx = if (i as usize) < labels.len() {
                labels[i as usize]
            } else {
                default
            };
            if let Some(frame_concrete) = frame.as_any().downcast_mut::<crate::module::Module>() {
                frame_concrete.branch(label_idx, stack)?;
                Ok(())
            } else if let Some(frame_concrete) = frame
                .as_any()
                .downcast_mut::<crate::stackless::StacklessFrame>()
            {
                frame_concrete.branch(label_idx, stack)?;
                Ok(())
            } else {
                Err(Error::Execution("Failed to downcast frame".to_string()))
            }
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn return_dyn(stack: &mut dyn Stack, frame: &mut dyn FrameBehavior) -> Result<()> {
    if let Some(frame_concrete) = frame.as_any().downcast_mut::<crate::module::Module>() {
        frame_concrete.return_(stack)?;
        Ok(())
    } else if let Some(frame_concrete) = frame
        .as_any()
        .downcast_mut::<crate::stackless::StacklessFrame>()
    {
        frame_concrete.return_(stack)?;
        Ok(())
    } else {
        Err(Error::Execution("Failed to downcast frame".to_string()))
    }
}

pub fn call_dyn(stack: &mut dyn Stack, frame: &mut dyn FrameBehavior, func_idx: u32) -> Result<()> {
    if let Some(frame_concrete) = frame.as_any().downcast_mut::<crate::module::Module>() {
        frame_concrete.call(func_idx, stack)?;
        Ok(())
    } else if let Some(frame_concrete) = frame
        .as_any()
        .downcast_mut::<crate::stackless::StacklessFrame>()
    {
        frame_concrete.call(func_idx, stack)?;
        Ok(())
    } else {
        Err(Error::Execution("Failed to downcast frame".to_string()))
    }
}

pub fn call_indirect_dyn(
    stack: &mut dyn Stack,
    frame: &mut dyn FrameBehavior,
    type_idx: u32,
    table_idx: u32,
) -> Result<()> {
    let table_entry = stack.pop()?;
    match table_entry {
        Value::I32(entry) => {
            // Convert i32 to u32
            let entry_u32 = entry as u32;
            if let Some(frame_concrete) = frame.as_any().downcast_mut::<crate::module::Module>() {
                frame_concrete.call_indirect(type_idx, table_idx, entry_u32, stack)?;
                Ok(())
            } else if let Some(frame_concrete) = frame
                .as_any()
                .downcast_mut::<crate::stackless::StacklessFrame>()
            {
                frame_concrete.call_indirect(type_idx, table_idx, entry_u32, stack)?;
                Ok(())
            } else {
                Err(Error::Execution("Failed to downcast frame".to_string()))
            }
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

// Adapter functions for the rest
pub fn unreachable_dyn(stack: &mut dyn Stack, frame: &mut dyn FrameBehavior) -> Result<()> {
    unreachable(stack, frame)
}

pub fn nop_dyn(stack: &mut dyn Stack, frame: &mut dyn FrameBehavior) -> Result<()> {
    nop(stack, frame)
}

/// Execute a `return_call` instruction
pub fn return_call(
    stack: &mut impl Stack,
    frame: &mut impl FrameBehavior,
    func_idx: u32,
) -> Result<()> {
    // Return_call is like a tail call - it replaces the current frame with a new one
    frame.call(func_idx, stack)?;
    Ok(())
}

/// Execute a `return_call_indirect` instruction
pub fn return_call_indirect(
    stack: &mut impl Stack,
    frame: &mut impl FrameBehavior,
    type_idx: u32,
    table_idx: u32,
) -> Result<()> {
    let table_entry = stack.pop()?;
    match table_entry {
        Value::I32(entry) => {
            // Convert i32 to u32
            let entry_u32 = entry as u32;
            frame.call_indirect(type_idx, table_idx, entry_u32, stack)?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn return_call_dyn(
    stack: &mut dyn Stack,
    frame: &mut dyn FrameBehavior,
    func_idx: u32,
) -> Result<()> {
    if let Some(frame_concrete) = frame.as_any().downcast_mut::<crate::module::Module>() {
        frame_concrete.call(func_idx, stack)?;
        Ok(())
    } else if let Some(frame_concrete) = frame
        .as_any()
        .downcast_mut::<crate::stackless::StacklessFrame>()
    {
        frame_concrete.call(func_idx, stack)?;
        Ok(())
    } else {
        Err(Error::Execution("Failed to downcast frame".to_string()))
    }
}

pub fn return_call_indirect_dyn(
    stack: &mut dyn Stack,
    frame: &mut dyn FrameBehavior,
    type_idx: u32,
    table_idx: u32,
) -> Result<()> {
    let table_entry = stack.pop()?;
    match table_entry {
        Value::I32(entry) => {
            // Convert i32 to u32
            let entry_u32 = entry as u32;
            if let Some(frame_concrete) = frame.as_any().downcast_mut::<crate::module::Module>() {
                frame_concrete.call_indirect(type_idx, table_idx, entry_u32, stack)?;
                Ok(())
            } else if let Some(frame_concrete) = frame
                .as_any()
                .downcast_mut::<crate::stackless::StacklessFrame>()
            {
                frame_concrete.call_indirect(type_idx, table_idx, entry_u32, stack)?;
                Ok(())
            } else {
                Err(Error::Execution("Failed to downcast frame".to_string()))
            }
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}
