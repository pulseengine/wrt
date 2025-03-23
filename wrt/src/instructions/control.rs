//! WebAssembly control flow instructions
//!
//! This module contains implementations for all WebAssembly control flow instructions,
//! including blocks, branches, calls, and returns.

use crate::error::{Error, Result};
use crate::execution::Stack;
use crate::format;
use crate::instructions::BlockType;
use crate::stackless::Frame as StacklessFrame;
use crate::types::FuncType;
use crate::Value;
use crate::Vec;

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
pub fn push_label(
    pc: usize,
    stack: &mut Stack,
    label_type: LabelType,
    block_type: BlockType,
    function_types: Option<&[FuncType]>,
) -> Result<()> {
    // Determine the function type for this block
    let func_type = match block_type {
        BlockType::Empty => FuncType {
            params: Vec::new(),
            results: Vec::new(),
        },
        BlockType::Type(value_type) => FuncType {
            params: Vec::new(),
            results: vec![value_type],
        },
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

    // Push the new label
    stack.push_label(func_type.results.len(), pc);

    Ok(())
}

/// Executes a block instruction
pub fn block(
    frame: &mut StacklessFrame,
    stack: &mut Stack,
    block_type: BlockType,
    continuation_pc: usize,
) -> Result<()> {
    let arity = match block_type {
        BlockType::Empty => 0,
        BlockType::Type(_) => 1,
        BlockType::TypeIndex(_) => 1, // Assuming type index refers to a type with one result
    };
    stack.push_label(arity, continuation_pc);
    Ok(())
}

/// Execute a loop instruction
///
/// Creates a new loop scope with the given block type.
pub fn loop_(
    frame: &mut StacklessFrame,
    stack: &mut Stack,
    block_type: BlockType,
    loop_pc: usize,
) -> Result<()> {
    let arity = match block_type {
        BlockType::Empty => 0,
        BlockType::Type(_) => 1,
        BlockType::TypeIndex(_) => 1, // Assuming type index refers to a type with one result
    };
    stack.push_label(arity, loop_pc);
    Ok(())
}

/// Executes an if instruction
pub fn if_(
    frame: &mut StacklessFrame,
    stack: &mut Stack,
    block_type: BlockType,
    continuation_pc: usize,
    else_pc: usize,
) -> Result<()> {
    let condition = match stack.pop()? {
        Value::I32(v) => v != 0,
        _ => {
            return Err(Error::Execution(
                "Expected i32 value for if condition".into(),
            ))
        }
    };

    let arity = match block_type {
        BlockType::Empty => 0,
        BlockType::Type(_) => 1,
        BlockType::TypeIndex(_) => 1, // Assuming type index refers to a type with one result
    };

    if condition {
        stack.push_label(arity, continuation_pc);
    } else {
        frame.return_pc = else_pc;
    }
    Ok(())
}

/// Executes a br instruction
pub fn br(frame: &mut StacklessFrame, stack: &mut Stack, label_idx: u32) -> Result<()> {
    let label = stack.get_label(label_idx)?;
    frame.return_pc = label.continuation;
    Ok(())
}

/// Executes a `br_if` instruction
pub fn br_if(frame: &mut StacklessFrame, stack: &mut Stack, label_idx: u32) -> Result<()> {
    let condition = match stack.pop()? {
        Value::I32(v) => v != 0,
        _ => {
            return Err(Error::Execution(
                "Expected i32 value for br_if condition".into(),
            ))
        }
    };

    if condition {
        let label = stack.get_label(label_idx)?;
        frame.return_pc = label.continuation;
    }
    Ok(())
}

/// Executes a `br_table` instruction
pub fn br_table(
    frame: &mut StacklessFrame,
    stack: &mut Stack,
    table: &[u32],
    default_idx: u32,
) -> Result<()> {
    let index = match stack.pop()? {
        Value::I32(v) => v as usize,
        _ => {
            return Err(Error::Execution(
                "Expected i32 value for br_table index".into(),
            ))
        }
    };

    let label_idx = table.get(index).copied().unwrap_or(default_idx);
    let label = stack.get_label(label_idx)?;
    frame.return_pc = label.continuation;
    Ok(())
}

/// Executes a return instruction
pub fn return_(frame: &mut StacklessFrame, stack: &mut Stack) -> Result<()> {
    frame.return_pc = usize::MAX;
    Ok(())
}

/// Executes a call instruction
pub const fn call(_func_idx: u32) -> Result<()> {
    // We handle this in the execution loop
    Ok(())
}

/// Executes a `call_indirect` instruction
pub const fn call_indirect(_table_idx: u32, _type_idx: u32) -> Result<()> {
    // We handle this in the execution loop
    Ok(())
}

/// Executes an unreachable instruction
pub fn unreachable(_stack: &mut Stack) -> Result<()> {
    // We handle this in the execution loop
    Ok(())
}

/// Executes a nop instruction
pub fn nop(_stack: &mut Stack) -> Result<()> {
    // We handle this in the execution loop
    Ok(())
}

/// Executes an end instruction
pub fn end(_stack: &mut Stack) -> Result<()> {
    // We handle this in the execution loop
    Ok(())
}

/// Executes an else instruction
pub fn else_instr(_stack: &mut Stack) -> Result<()> {
    // We handle this in the execution loop
    Ok(())
}
