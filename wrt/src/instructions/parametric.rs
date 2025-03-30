//! WebAssembly parametric instructions
//!
//! This module contains implementations for all WebAssembly parametric instructions,
//! including operations for stack manipulation and control flow.

use crate::{
    behavior::FrameBehavior,
    error::{Error, Result},
    stack::Stack,
    types::ValueType,
    values::Value,
};

/// Execute a drop instruction
///
/// Removes the top value from the stack.
pub fn drop(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    stack.pop()?;
    Ok(())
}

/// Execute a select instruction
///
/// Selects one of two values based on a condition.
pub fn select(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let c = stack.pop()?;
    let val2 = stack.pop()?;
    let val1 = stack.pop()?;
    match c {
        Value::I32(c) => {
            if c != 0 {
                stack.push(val1)?;
            } else {
                stack.push(val2)?;
            }
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute a `select_typed` instruction
///
/// Selects one of two values based on a condition, with type checking.
pub fn select_typed(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    ty: ValueType,
) -> Result<()> {
    let c = stack.pop()?;
    let val2 = stack.pop()?;
    let val1 = stack.pop()?;
    match c {
        Value::I32(c) => {
            if !val1.matches_type(&ty) || !val2.matches_type(&ty) {
                return Err(Error::InvalidType(format!("Expected {ty}")));
            }
            if c != 0 {
                stack.push(val1)?;
            } else {
                stack.push(val2)?;
            }
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute a block instruction
///
/// Creates a new block scope.
pub fn block(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    Ok(())
}

/// Execute an if instruction
///
/// Creates a new conditional block.
pub fn if_instr(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let condition = stack.pop()?;
    match condition {
        Value::I32(0) => Ok(()),
        _ => Ok(()),
    }
}

/// Execute an else instruction
///
/// Ends the "if" part of an if/else and begins the "else" part.
pub fn else_instr(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    Ok(())
}

/// Execute an end instruction
///
/// Ends a block, loop, if, or function.
pub fn end(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    Ok(())
}

/// Execute a br instruction
///
/// Unconditionally branches to a label.
pub fn br(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    label_idx: u32,
) -> Result<()> {
    Ok(())
}

/// Execute a `br_if` instruction
///
/// Conditionally branches to a label.
pub fn br_if(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    label_idx: u32,
) -> Result<()> {
    let condition = stack.pop()?;
    match condition {
        Value::I32(0) => Ok(()),
        _ => Ok(()),
    }
}

/// Execute a `br_table` instruction
///
/// Branches to one of several labels based on an index value.
pub fn br_table(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    label_indices: &[u32],
    default_label: u32,
) -> Result<()> {
    let index = stack.pop()?;
    match index {
        Value::I32(_) => Ok(()),
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute a return instruction
///
/// Returns from the current function.
pub fn return_instr(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    Ok(())
}

/// Execute an unreachable instruction
///
/// Indicates that the current code location should not be reachable.
pub fn unreachable(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    Err(Error::Execution("Reached unreachable instruction".into()))
}

/// Execute a nop instruction
///
/// No operation.
pub fn nop(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    Ok(())
}
