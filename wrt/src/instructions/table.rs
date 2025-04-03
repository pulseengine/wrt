//! WebAssembly table instructions
//!
//! This module contains implementations for all WebAssembly table instructions,
//! including table access and manipulation operations.

use crate::{
    behavior::{FrameBehavior, StackBehavior},
    error::{Error, Result},
    instructions::InstructionExecutor,
    stack::Stack,
    values::Value,
    StacklessEngine,
};

/// Execute a table.get instruction
///
/// Gets an element from a table.
pub fn table_get(
    stack: &mut dyn Stack,
    frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
    table_idx: u32,
) -> Result<()> {
    let idx = stack.pop()?.as_i32()?;
    let table = frame.get_table(table_idx)?;
    let elem = table.get(idx as u32)?;
    stack.push(elem.into())
}

/// Execute a table.set instruction
///
/// Sets an element in a table.
pub fn table_set(
    stack: &mut dyn Stack,
    frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
    table_idx: u32,
) -> Result<()> {
    let val = stack.pop()?;
    let idx = stack.pop()?.as_i32()?;
    let table = frame.get_table_mut(table_idx)?;
    table.set(idx as u32, val.try_into()?)?;
    Ok(())
}

/// Execute a table.size instruction
///
/// Returns the current size of a table.
pub fn table_size(
    stack: &mut dyn Stack,
    frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
    table_idx: u32,
) -> Result<()> {
    let table = frame.get_table(table_idx)?;
    stack.push(Value::I32(table.size() as i32))
}

/// Execute a table.grow instruction
///
/// Grows a table by a number of elements.
pub fn table_grow(
    stack: &mut dyn Stack,
    frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
    table_idx: u32,
) -> Result<()> {
    let n = stack.pop()?.as_i32()?;
    let val = stack.pop()?;
    let table = frame.get_table_mut(table_idx)?;
    let prev_size = table.grow(n as u32, val.try_into()?)?;
    stack.push(Value::I32(prev_size as i32))
}

/// Execute a table.init instruction
///
/// Initializes a table segment.
pub fn table_init(
    stack: &mut dyn Stack,
    frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
    table_idx: u32,
    elem_idx: u32,
) -> Result<()> {
    let n = stack.pop()?.as_i32()?;
    let s = stack.pop()?.as_i32()?;
    let d = stack.pop()?.as_i32()?;
    let table = frame.get_table_mut(table_idx)?;
    let elem_segment = frame.get_element_segment(elem_idx)?;
    table.init(d as u32, elem_segment, s as u32, n as u32)?;
    Ok(())
}

/// Execute a table.copy instruction
///
/// Copies elements from one table to another.
pub fn table_copy(
    stack: &mut dyn Stack,
    frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
    dest_table_idx: u32,
    src_table_idx: u32,
) -> Result<()> {
    let n = stack.pop()?.as_i32()?;
    let s = stack.pop()?.as_i32()?;
    let d = stack.pop()?.as_i32()?;

    if dest_table_idx == src_table_idx {
        let table = frame.get_table_mut(dest_table_idx)?;
        table.copy_within(s as u32, d as u32, n as u32)?;
    } else {
        let (src_table, dest_table) = frame.get_two_tables_mut(src_table_idx, dest_table_idx)?;
        dest_table.copy_from(src_table, s as u32, d as u32, n as u32)?;
    }
    Ok(())
}

pub fn elem_drop(
    stack: &mut dyn Stack,
    frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
    elem_idx: u32,
) -> Result<()> {
    frame.drop_element_segment(elem_idx)?;
    Ok(())
}

pub fn table_fill(
    stack: &mut dyn Stack,
    frame: &mut dyn FrameBehavior,
    _engine: &StacklessEngine,
    table_idx: u32,
) -> Result<()> {
    let n = stack.pop()?.as_i32()?;
    let val = stack.pop()?;
    let i = stack.pop()?.as_i32()?;
    let table = frame.get_table_mut(table_idx)?;
    table.fill(i as u32, val.try_into()?, n as u32)?;
    Ok(())
}
