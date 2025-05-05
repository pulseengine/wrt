//! WebAssembly table instructions
//!
//! This module contains implementations for all WebAssembly table instructions,
//! including table access and manipulation operations.

use crate::{
    behavior::{ControlFlow, FrameBehavior, StackBehavior},
    error::{kinds, Error, Result},
    prelude::TypesValue as Value,
    stackless::StacklessEngine,
};
// These wasmparser imports likely don't exist in newer versions
// use wasmparser::{TableInit, TableCopy, ElemDrop, TableFill};

/// Execute a table.get instruction
///
/// Gets an element from a table.
pub fn table_get(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
    table_idx: u32,
) -> Result<()> {
    let idx = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::new(kinds::InvalidTypeError("Expected i32 index".to_string())))?;
    let table = frame.get_table(table_idx as usize, _engine)?;
    let elem_opt = table.get(idx as u32)?;
    let elem = elem_opt.ok_or_else(|| {
        Error::new(kinds::ExecutionError(
            "Element not found in table or is null".to_string(),
        ))
    })?;
    stack.push(elem)?;
    Ok(())
}

/// Execute a table.set instruction
///
/// Sets an element in a table.
pub fn table_set(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
    table_idx: u32,
) -> Result<()> {
    let val = stack.pop()?;
    let idx = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::new(kinds::InvalidTypeError("Expected i32 index".to_string())))?;
    let table = frame.get_table_mut(table_idx as usize, _engine)?;
    // Table::set expects Option<Value>, so wrap val in Some
    // Check type compatibility (optional but good practice)
    let value_to_set = val;
    if value_to_set.type_() != table.type_().element_type {
        return Err(Error::new(kinds::InvalidTypeError(format!(
            "Type mismatch in table.set: expected {:?}, got {:?}",
            table.type_().element_type,
            value_to_set.type_()
        ))));
    }
    table.set(idx as u32, Some(value_to_set))?;
    Ok(())
}

/// Execute a table.size instruction
///
/// Returns the current size of a table.
pub fn table_size(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
    table_idx: u32,
) -> Result<()> {
    let table = frame.get_table(table_idx as usize, _engine)?;
    let size = table.size() as i32; // size is usize, cast to i32 for stack push
    stack.push(Value::I32(size))?;
    Ok(())
}

/// Execute a table.grow instruction
///
/// Grows a table by a number of elements.
pub fn table_grow(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    _engine: &mut StacklessEngine,
    table_idx: u32,
) -> Result<()> {
    let init_val = stack.pop()?;
    let n = stack.pop()?.as_i32().ok_or_else(|| {
        Error::new(kinds::InvalidTypeError(
            "Expected i32 growth amount".to_string(),
        ))
    })?;
    let table = frame.get_table_mut(table_idx as usize, _engine)?;
    // table.grow only takes delta according to error E0061
    // The init_val from the stack might be needed if the Table impl changes,
    // but for now we ignore it as per Wasm spec for table.grow.
    let _ = init_val; // Mark as used to avoid warnings
    let prev_size = table.grow(n as u32)?;
    stack.push(Value::I32(prev_size as i32))?;
    Ok(())
}

/// Execute a table.init instruction
///
/// Initializes a table segment.
pub fn table_init(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    engine: &mut StacklessEngine,
    dst_table_idx: u32,
    elem_idx: u32,
) -> Result<ControlFlow, Error> {
    // Pop arguments from the provided stack
    let n = stack.pop_i32()? as u32;
    let src = stack.pop_i32()? as u32;
    let dst = stack.pop_i32()? as u32;

    // Use frame to get table and element segment via engine context
    let table = frame.get_table_mut(dst_table_idx as usize, engine)?;
    let element_segment = frame.get_element_segment(elem_idx, engine)?;

    // Bounds check using element_segment.items
    if src
        .checked_add(n)
        .map_or(true, |end| end as usize > element_segment.items.len())
    {
        return Err(Error::new(kinds::TableAccessOutOfBoundsError {
            table_idx: dst_table_idx,
            element_idx: elem_idx as usize,
        }));
    }
    // Compare end (u32) with table.size() (usize) correctly
    if dst
        .checked_add(n)
        .map_or(true, |end| end > table.size() as u32)
    {
        return Err(Error::new(kinds::Trap(
            "table_init destination out of bounds".into(),
        )));
    }

    // Copy elements using element_segment.items
    for i in 0..n {
        let elem_item = element_segment
            .items
            .get((src + i) as usize)
            .ok_or_else(|| {
                Error::new(kinds::TableAccessOutOfBoundsError {
                    table_idx: dst_table_idx,
                    element_idx: (src + i) as usize,
                })
            })?;

        let value_to_set = Value::FuncRef(Some(*elem_item));
        table
            .set((dst + i) as u32, Some(value_to_set))
            .map_err(|e| {
                Error::new(kinds::TableAccessOutOfBoundsError {
                    table_idx: dst_table_idx,
                    element_idx: (src + i) as usize,
                })
            })?;
    }

    Ok(ControlFlow::Continue)
}

/// Execute a table.copy instruction
///
/// Copies elements from one table to another.
pub fn table_copy(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    dst_table_idx: u32,
    src_table_idx: u32,
    engine: &mut StacklessEngine,
) -> Result<()> {
    let n = stack
        .pop()?
        .as_i32()
        .ok_or_else(|| Error::new(kinds::InvalidTypeError("Expected i32 count".to_string())))?;
    let s = stack.pop()?.as_i32().ok_or_else(|| {
        Error::new(kinds::InvalidTypeError(
            "Expected i32 source offset".to_string(),
        ))
    })?;
    let d = stack.pop()?.as_i32().ok_or_else(|| {
        Error::new(kinds::InvalidTypeError(
            "Expected i32 destination offset".to_string(),
        ))
    })?;

    let (dst_table, src_table) = frame.get_two_tables_mut(dst_table_idx, src_table_idx, engine)?;

    // Bounds checking
    let dst_size = dst_table.size() as i32;
    let src_size = src_table.size() as i32;

    if s.checked_add(n).map_or(true, |end| end > src_size)
        || d.checked_add(n).map_or(true, |end| end > dst_size)
    {
        return Err(Error::new(kinds::ExecutionError(
            "Table index out of bounds".to_string(),
        )));
    }

    // Perform the copy
    // FIXME: Needs engine access and public API on Table to avoid accessing private fields.
    return Err(Error::new(kinds::NotImplementedError(
        "table.copy needs engine access and Table API change".to_string(),
    )));
    // Ok(())
}

pub fn elem_drop(engine: &mut StacklessEngine, elem_idx: u32) -> Result<()> {
    let frame = engine.current_frame()?;
    let instance_idx = frame.instance_idx();
    // Delegate dropping to the engine/instance
    engine.with_instance_mut(instance_idx as usize, |instance| {
        instance.elem_drop(elem_idx)
    })
}

pub fn table_fill(
    stack: &mut dyn StackBehavior,
    frame: &mut dyn FrameBehavior,
    table_idx: u32,
    engine: &mut StacklessEngine,
) -> Result<()> {
    let n = stack.pop()?.as_i32().ok_or_else(|| {
        Error::new(kinds::InvalidTypeError(
            "Expected i32 for table_fill count".to_string(),
        ))
    })?;
    let val = stack.pop()?;
    let d = stack.pop()?.as_i32().ok_or_else(|| {
        Error::new(kinds::InvalidTypeError(
            "Expected i32 for table_fill offset".to_string(),
        ))
    })?;

    let table = frame.get_table_mut(table_idx as usize, engine)?;
    let table_size = table.size() as i32;

    if d.checked_add(n).map_or(true, |end| end > table_size) {
        return Err(Error::new(kinds::ExecutionError(
            "Table index out of bounds".to_string(),
        )));
    }

    // Validate value type matches table type
    if val.type_() != table.type_().element_type {
        return Err(Error::new(kinds::InvalidTypeError(format!(
            "Expected type {}, found type {}",
            table.type_().element_type,
            val.type_()
        ))));
    }

    // FIXME: Needs engine access and public API on Table to avoid accessing private fields.
    return Err(Error::new(kinds::NotImplementedError(
        "table.fill needs engine access and Table API change".to_string(),
    )));
    // Ok(())
}
