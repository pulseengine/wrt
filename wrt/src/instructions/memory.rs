//! WebAssembly memory instructions
//!
//! This module contains implementations for all WebAssembly memory instructions,
//! including loads, stores, and memory management operations.

use crate::{
    behavior::FrameBehavior,
    error::{Error, Result},
    stack::Stack,
    values::Value,
};

/// Execute an i32 load instruction
///
/// Loads a 32-bit integer from memory at the specified address.
pub fn i32_load(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    let addr = stack.pop()?;
    match addr {
        Value::I32(addr) => {
            let addr = addr as u32;
            let effective_addr = (addr + offset) as usize;
            let value = frame.load_i32(effective_addr, align)?;
            stack.push(Value::I32(value))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an i64 load instruction
///
/// Loads a 64-bit integer from memory at the specified address.
pub fn i64_load(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    let addr = stack.pop()?;
    match addr {
        Value::I32(addr) => {
            let addr = addr as u32;
            let effective_addr = (addr + offset) as usize;
            let value = frame.load_i64(effective_addr, align)?;
            stack.push(Value::I64(value))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an f32 load instruction
///
/// Loads a 32-bit float from memory at the specified address.
pub fn f32_load(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    let addr = stack.pop()?;
    match addr {
        Value::I32(addr) => {
            let addr = addr as u32;
            let effective_addr = (addr + offset) as usize;
            let value = frame.load_f32(effective_addr, align)?;
            stack.push(Value::F32(value))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an f64 load instruction
///
/// Loads a 64-bit float from memory at the specified address.
pub fn f64_load(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    let addr = stack.pop()?;
    match addr {
        Value::I32(addr) => {
            let addr = addr as u32;
            let effective_addr = (addr + offset) as usize;
            let value = frame.load_f64(effective_addr, align)?;
            stack.push(Value::F64(value))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an i32 `load8_s` instruction
///
/// Loads an 8-bit signed integer from memory and sign-extends it to 32 bits.
pub fn i32_load8_s(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    let addr = stack.pop()?;
    match addr {
        Value::I32(addr) => {
            let addr = addr as u32;
            let effective_addr = (addr + offset) as usize;
            let value = frame.load_i8(effective_addr, align)?;
            stack.push(Value::I32(i32::from(value)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an i32 `load8_u` instruction
///
/// Loads an 8-bit unsigned integer from memory and zero-extends it to 32 bits.
pub fn i32_load8_u(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    let addr = stack.pop()?;
    match addr {
        Value::I32(addr) => {
            let addr = addr as u32;
            let effective_addr = (addr + offset) as usize;
            let value = frame.load_u8(effective_addr, align)?;
            stack.push(Value::I32(i32::from(value)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an i32 `load16_s` instruction
///
/// Loads a 16-bit signed integer from memory and sign-extends it to 32 bits.
pub fn i32_load16_s(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    let addr = stack.pop()?;
    match addr {
        Value::I32(addr) => {
            let addr = addr as u32;
            let effective_addr = (addr + offset) as usize;
            let value = frame.load_i16(effective_addr, align)?;
            stack.push(Value::I32(i32::from(value)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an i32 `load16_u` instruction
///
/// Loads a 16-bit unsigned integer from memory and zero-extends it to 32 bits.
pub fn i32_load16_u(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    let addr = stack.pop()?;
    match addr {
        Value::I32(addr) => {
            let addr = addr as u32;
            let effective_addr = (addr + offset) as usize;
            let value = frame.load_u16(effective_addr, align)?;
            stack.push(Value::I32(i32::from(value)))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an i32 store instruction
///
/// Stores a 32-bit integer to memory at the specified address.
pub fn i32_store(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    let value = stack.pop()?;
    let addr = stack.pop()?;
    match (addr, value) {
        (Value::I32(addr), Value::I32(value)) => {
            let addr = addr as u32;
            let effective_addr = (addr + offset) as usize;
            frame.store_i32(effective_addr, align, value)?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Execute an i64 store instruction
///
/// Stores a 64-bit integer to memory at the specified address.
pub fn i64_store(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    let value = stack.pop()?;
    let addr = stack.pop()?;
    match (addr, value) {
        (Value::I32(addr), Value::I64(value)) => {
            let addr = addr as u32;
            let effective_addr = (addr + offset) as usize;
            frame.store_i64(effective_addr, align, value)?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32 and i64".to_string())),
    }
}

/// Execute memory size instruction
///
/// Returns the current size of memory in pages (64KB per page).
pub fn memory_size(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let size = frame.memory_size()?;
    stack.push(Value::I32(size as i32))?;
    Ok(())
}

/// Execute memory grow instruction
///
/// Grows memory by the specified number of pages, returns previous size or -1 on failure.
pub fn memory_grow(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    let pages = stack.pop()?;
    match pages {
        Value::I32(pages) => {
            let old_size = frame.memory_grow(pages as u32)?;
            stack.push(Value::I32(old_size as i32))?;
            Ok(())
        }
        _ => Err(Error::InvalidType("Expected i32".to_string())),
    }
}

/// Pop a memory address from the stack
///
/// Helper function to get a memory address from the stack.
fn pop_memory_address(stack: &mut (impl Stack + ?Sized)) -> Result<u32> {
    match stack.pop()? {
        Value::I32(addr) => Ok(addr as u32),
        _ => Err(Error::TypeMismatch("Expected i32 address".to_string())),
    }
}

/// Execute memory fill instruction
///
/// Fills a region of memory with a given byte value.
pub fn memory_fill(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    // Pop values in reverse order: value, size, destination
    let value = match stack.pop()? {
        Value::I32(val) => val as u8,
        _ => return Err(Error::InvalidType("Expected i32 value".to_string())),
    };

    let size = match stack.pop()? {
        Value::I32(size) => size as u32,
        _ => return Err(Error::InvalidType("Expected i32 size".to_string())),
    };

    let dst = match stack.pop()? {
        Value::I32(dst) => dst as u32,
        _ => {
            return Err(Error::InvalidType(
                "Expected i32 destination address".to_string(),
            ))
        }
    };

    // Fill memory with the byte value
    for i in 0..size {
        let addr = dst + i;
        frame.store_i32(addr as usize, 0, i32::from(value) & 0xFF)?;
    }

    Ok(())
}

/// Execute memory copy instruction
///
/// Copies data from one region of memory to another, possibly overlapping.
pub fn memory_copy(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
) -> Result<()> {
    // Pop values in reverse order: size, source, destination
    let size = match stack.pop()? {
        Value::I32(size) => size as u32,
        _ => return Err(Error::InvalidType("Expected i32 size".to_string())),
    };

    let src = match stack.pop()? {
        Value::I32(src) => src as u32,
        _ => {
            return Err(Error::InvalidType(
                "Expected i32 source address".to_string(),
            ))
        }
    };

    let dst = match stack.pop()? {
        Value::I32(dst) => dst as u32,
        _ => {
            return Err(Error::InvalidType(
                "Expected i32 destination address".to_string(),
            ))
        }
    };

    // Handle overlapping regions by copying in the right direction
    if dst <= src || src + size <= dst {
        // Non-overlapping regions or dst before src, copy forward
        for i in 0..size {
            let val = frame.load_i32((src + i) as usize, 0)?;
            frame.store_i32((dst + i) as usize, 0, val & 0xFF)?;
        }
    } else {
        // Overlapping regions with dst > src, copy backward
        let mut i = size;
        while i > 0 {
            i -= 1;
            let val = frame.load_i32((src + i) as usize, 0)?;
            frame.store_i32((dst + i) as usize, 0, val & 0xFF)?;
        }
    }

    Ok(())
}

/// Execute memory init instruction
///
/// Copies data from a passive data segment to memory.
pub fn memory_init(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    idx: u32,
) -> Result<()> {
    // Pop values in reverse order: size, source offset, destination
    let size = match stack.pop()? {
        Value::I32(size) => size as u32,
        _ => return Err(Error::InvalidType("Expected i32 size".to_string())),
    };

    let src = match stack.pop()? {
        Value::I32(src) => src as u32,
        _ => return Err(Error::InvalidType("Expected i32 source offset".to_string())),
    };

    let dst = match stack.pop()? {
        Value::I32(dst) => dst as u32,
        _ => {
            return Err(Error::InvalidType(
                "Expected i32 destination address".to_string(),
            ))
        }
    };

    // This is a placeholder implementation
    // In a real implementation, this would access data segment idx and copy data to memory
    // For now, we'll just simulate by writing zeros
    for i in 0..size {
        frame.store_i32((dst + i) as usize, 0, 0)?;
    }

    Ok(())
}

/// Execute data drop instruction
///
/// Drops a passive data segment, freeing its resources.
pub fn data_drop(
    _stack: &mut (impl Stack + ?Sized),
    _frame: &mut (impl FrameBehavior + ?Sized),
    idx: u32,
) -> Result<()> {
    // This is a placeholder implementation
    // In a real implementation, this would mark data segment idx as dropped
    // For now, we'll just return Ok as if the operation succeeded
    Ok(())
}

/// Load signed 8-bit value and extend to i64
pub fn i64_load8_s(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    // Get effective address
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };

    // Load i8 and sign-extend to i64
    let value = i64::from(frame.load_i8(addr, align)?);
    stack.push(Value::I64(value))
}

/// Load unsigned 8-bit value and extend to i64
pub fn i64_load8_u(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    // Get effective address
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };

    // Load u8 and zero-extend to i64
    let value = i64::from(frame.load_u8(addr, align)?);
    stack.push(Value::I64(value))
}

/// Load signed 16-bit value and extend to i64
pub fn i64_load16_s(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    // Get effective address
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };

    // Load i16 and sign-extend to i64
    let value = i64::from(frame.load_i16(addr, align)?);
    stack.push(Value::I64(value))
}

/// Load unsigned 16-bit value and extend to i64
pub fn i64_load16_u(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    // Get effective address
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };

    // Load u16 and zero-extend to i64
    let value = i64::from(frame.load_u16(addr, align)?);
    stack.push(Value::I64(value))
}

/// Load signed 32-bit value and extend to i64
pub fn i64_load32_s(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    // Get effective address
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };

    // Load i32 and sign-extend to i64
    let value = i64::from(frame.load_i32(addr, align)?);
    stack.push(Value::I64(value))
}

/// Load unsigned 32-bit value and extend to i64
pub fn i64_load32_u(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    // Get effective address
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };

    // Load i32 (as unsigned) and zero-extend to i64
    let value = i64::from(frame.load_i32(addr, align)? as u32);
    stack.push(Value::I64(value))
}

/// Store f32 value
pub fn f32_store(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    // Get value to store
    let value = match stack.pop()? {
        Value::F32(v) => v,
        _ => return Err(Error::TypeMismatch("Expected f32 value".to_string())),
    };

    // Get effective address
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };

    // Store as i32 value (reinterpreted as f32)
    let bits = f32::to_bits(value);
    frame.store_i32(addr, align, bits as i32)
}

/// Store f64 value
pub fn f64_store(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    // Get value to store
    let value = match stack.pop()? {
        Value::F64(v) => v,
        _ => return Err(Error::TypeMismatch("Expected f64 value".to_string())),
    };

    // Get effective address
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };

    // Store as i64 value (reinterpreted as f64)
    let bits = f64::to_bits(value);
    frame.store_i64(addr, align, bits as i64)
}

/// Store low 8 bits of i32
pub fn i32_store8(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    // Get value to store
    let value = match stack.pop()? {
        Value::I32(v) => v,
        _ => return Err(Error::TypeMismatch("Expected i32 value".to_string())),
    };

    // Get effective address
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };

    // Store the lowest 8 bits
    frame.store_i32(addr, align, value & 0xFF)
}

/// Store low 16 bits of i32
pub fn i32_store16(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    // Get value to store
    let value = match stack.pop()? {
        Value::I32(v) => v,
        _ => return Err(Error::TypeMismatch("Expected i32 value".to_string())),
    };

    // Get effective address
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };

    // Store the lowest 16 bits
    frame.store_i32(addr, align, value & 0xFFFF)
}

/// Store low 8 bits of i64
pub fn i64_store8(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    // Get value to store
    let value = match stack.pop()? {
        Value::I64(v) => v,
        _ => return Err(Error::TypeMismatch("Expected i64 value".to_string())),
    };

    // Get effective address
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };

    // Store the lowest 8 bits
    frame.store_i32(addr, align, (value & 0xFF) as i32)
}

/// Store low 16 bits of i64
pub fn i64_store16(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    // Get value to store
    let value = match stack.pop()? {
        Value::I64(v) => v,
        _ => return Err(Error::TypeMismatch("Expected i64 value".to_string())),
    };

    // Get effective address
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };

    // Store the lowest 16 bits
    frame.store_i32(addr, align, (value & 0xFFFF) as i32)
}

/// Store low 32 bits of i64
pub fn i64_store32(
    stack: &mut (impl Stack + ?Sized),
    frame: &mut (impl FrameBehavior + ?Sized),
    offset: u32,
    align: u32,
) -> Result<()> {
    // Get value to store
    let value = match stack.pop()? {
        Value::I64(v) => v,
        _ => return Err(Error::TypeMismatch("Expected i64 value".to_string())),
    };

    // Get effective address
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };

    // Store the lowest 32 bits
    frame.store_i32(addr, align, (value & 0xFFFFFFFF) as i32)
}
