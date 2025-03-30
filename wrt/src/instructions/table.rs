//! WebAssembly table instructions
//!
//! This module contains implementations for all WebAssembly table instructions,
//! including table access and manipulation operations.

use crate::{
    behavior::FrameBehavior,
    error::{Error, Result},
    stack::Stack,
    values::Value,
};

/// Execute a table.get instruction
///
/// Gets an element from a table.
pub fn table_get(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    table_idx: u32,
) -> Result<()> {
    let idx = stack.pop()?;
    match idx {
        Value::I32(idx) => {
            let value = frame.table_get(table_idx, idx as u32)?;
            stack.push(value)?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute a table.set instruction
///
/// Sets an element in a table.
pub fn table_set(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    table_idx: u32,
) -> Result<()> {
    let value = stack.pop()?;
    let idx = stack.pop()?;
    match idx {
        Value::I32(idx) => {
            frame.table_set(table_idx, idx as u32, value)?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute a table.size instruction
///
/// Returns the current size of a table.
pub fn table_size(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    table_idx: u32,
) -> Result<()> {
    let size = frame.table_size(table_idx)?;
    stack.push(Value::I32(size as i32))?;
    Ok(())
}

/// Execute a table.grow instruction
///
/// Grows a table by a number of elements.
pub fn table_grow(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    table_idx: u32,
) -> Result<()> {
    let value = stack.pop()?;
    let delta = stack.pop()?;
    match delta {
        Value::I32(delta) => {
            let old_size = frame.table_grow(table_idx, delta as u32, value)?;
            stack.push(Value::I32(old_size as i32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute a table.init instruction
///
/// Initializes a table segment.
pub fn table_init(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    table_idx: u32,
    elem_idx: u32,
) -> Result<()> {
    let n = stack.pop()?;
    let src = stack.pop()?;
    let dst = stack.pop()?;
    match (dst, src, n) {
        (Value::I32(dst), Value::I32(src), Value::I32(n)) => {
            frame.table_init(table_idx, elem_idx, dst as u32, src as u32, n as u32)?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute a table.copy instruction
///
/// Copies elements from one table to another.
pub fn table_copy(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    dst_table: u32,
    src_table: u32,
) -> Result<()> {
    let n = stack.pop()?;
    let src = stack.pop()?;
    let dst = stack.pop()?;
    match (dst, src, n) {
        (Value::I32(dst), Value::I32(src), Value::I32(n)) => {
            frame.table_copy(dst_table, src_table, dst as u32, src as u32, n as u32)?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

pub fn elem_drop(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    elem_idx: u32,
) -> Result<()> {
    frame.elem_drop(elem_idx)?;
    Ok(())
}

pub fn table_fill(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    table_idx: u32,
) -> Result<()> {
    let n = stack.pop()?;
    let val = stack.pop()?;
    let dst = stack.pop()?;
    match (dst, n) {
        (Value::I32(dst), Value::I32(n)) => {
            frame.table_fill(table_idx, dst as u32, val, n as u32)?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}
