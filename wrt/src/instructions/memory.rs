//! WebAssembly memory instructions
//!
//! This module contains implementations for all WebAssembly memory instructions,
//! including loads, stores, and memory management operations.

use crate::{
    behavior::{FrameBehavior, StackBehavior},
    error::{Error, Result},
    instructions::InstructionExecutor,
    stack::Stack,
    values::Value,
    StacklessEngine,
};
use std::sync::Arc;

/// Execute an i32 load instruction
///
/// Loads a 32-bit integer from memory at the specified address.
pub fn i32_load(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let addr = stack.pop()?.as_i32()?;
    let memory = frame.get_memory(0)?;
    let effective_addr = (addr as u32).wrapping_add(offset);
    memory.check_alignment(effective_addr, 4, align)?;
    let value = memory.read_i32(effective_addr)?;
    stack.push(Value::I32(value))
}

/// Execute an i64 load instruction
///
/// Loads a 64-bit integer from memory at the specified address.
pub fn i64_load(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let addr = stack.pop()?.as_i32()?;
    let memory = frame.get_memory(0)?;
    let effective_addr = (addr as u32).wrapping_add(offset);
    memory.check_alignment(effective_addr, 8, align)?;
    let value = memory.read_i64(effective_addr)?;
    stack.push(Value::I64(value))
}

/// Execute an f32 load instruction
///
/// Loads a 32-bit float from memory at the specified address.
pub fn f32_load(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let addr = stack.pop()?.as_i32()?;
    let memory = frame.get_memory(0)?;
    let effective_addr = (addr as u32).wrapping_add(offset);
    memory.check_alignment(effective_addr, 4, align)?;
    let value = memory.read_f32(effective_addr)?;
    stack.push(Value::F32(value))
}

/// Execute an f64 load instruction
///
/// Loads a 64-bit float from memory at the specified address.
pub fn f64_load(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let addr = stack.pop()?.as_i32()?;
    let memory = frame.get_memory(0)?;
    let effective_addr = (addr as u32).wrapping_add(offset);
    memory.check_alignment(effective_addr, 8, align)?;
    let value = memory.read_f64(effective_addr)?;
    stack.push(Value::F64(value))
}

/// Execute an i32 `load8_s` instruction
///
/// Loads an 8-bit signed integer from memory and sign-extends it to 32 bits.
pub fn i32_load8_s(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let addr = stack.pop()?.as_i32()?;
    let memory = frame.get_memory(0)?;
    let effective_addr = (addr as u32).wrapping_add(offset);
    memory.check_alignment(effective_addr, 1, align)?;
    let value = memory.read_i8(effective_addr)? as i32;
    stack.push(Value::I32(value))
}

/// Execute an i32 `load8_u` instruction
///
/// Loads an 8-bit unsigned integer from memory and zero-extends it to 32 bits.
pub fn i32_load8_u(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let addr = stack.pop()?.as_i32()?;
    let memory = frame.get_memory(0)?;
    let effective_addr = (addr as u32).wrapping_add(offset);
    memory.check_alignment(effective_addr, 1, align)?;
    let value = memory.read_u8(effective_addr)? as i32;
    stack.push(Value::I32(value))
}

/// Execute an i32 `load16_s` instruction
///
/// Loads a 16-bit signed integer from memory and sign-extends it to 32 bits.
pub fn i32_load16_s(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let addr = stack.pop()?.as_i32()?;
    let memory = frame.get_memory(0)?;
    let effective_addr = (addr as u32).wrapping_add(offset);
    memory.check_alignment(effective_addr, 2, align)?;
    let value = memory.read_i16(effective_addr)? as i32;
    stack.push(Value::I32(value))
}

/// Execute an i32 `load16_u` instruction
///
/// Loads a 16-bit unsigned integer from memory and zero-extends it to 32 bits.
pub fn i32_load16_u(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let addr = stack.pop()?.as_i32()?;
    let memory = frame.get_memory(0)?;
    let effective_addr = (addr as u32).wrapping_add(offset);
    memory.check_alignment(effective_addr, 2, align)?;
    let value = memory.read_u16(effective_addr)? as i32;
    stack.push(Value::I32(value))
}

/// Execute an i32 store instruction
///
/// Stores a 32-bit integer to memory at the specified address.
pub fn i32_store(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let value = stack.pop()?.as_i32()?;
    let addr = stack.pop()?.as_i32()?;
    let memory = frame.get_memory(0)?;
    let effective_addr = (addr as u32).wrapping_add(offset);
    memory.check_alignment(effective_addr, 4, align)?;
    memory.write_i32(effective_addr, value)
}

/// Execute an i64 store instruction
///
/// Stores a 64-bit integer to memory at the specified address.
pub fn i64_store(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let value = stack.pop()?.as_i64()?;
    let addr = stack.pop()?.as_i32()?;
    let memory = frame.get_memory(0)?;
    let effective_addr = (addr as u32).wrapping_add(offset);
    memory.check_alignment(effective_addr, 8, align)?;
    memory.write_i64(effective_addr, value)
}

/// Execute memory size instruction
///
/// Returns the current size of memory in pages (64KB per page).
pub fn memory_size(stack: &mut dyn Stack, frame: &dyn FrameBehavior, mem_idx: u32, engine: &StacklessEngine) -> Result<()> {
    let memory = frame.get_memory(mem_idx as usize)?;
    let size_pages = memory.size(); // Size in pages
    stack.push(Value::I32(size_pages as i32))
}

/// Execute memory grow instruction
///
/// Grows memory by the specified number of pages, returns previous size or -1 on failure.
pub fn memory_grow(stack: &mut dyn Stack, frame: &mut dyn FrameBehavior, mem_idx: u32, _engine: &StacklessEngine) -> Result<()> {
    let delta_pages = stack.pop()?.as_i32()?;
    let memory = frame.get_memory_mut(mem_idx as usize)?;
    let old_size = memory.grow(delta_pages as u32)?;
    stack.push(Value::I32(old_size as i32))
}

/// Pop a memory address from the stack
///
/// Helper function to get a memory address from the stack.
fn pop_memory_address(stack: &mut dyn Stack) -> Result<u32> {
    match stack.pop()?.as_i32() {
        Some(addr) => Ok(addr),
        None => Err(Error::TypeMismatch("Expected i32 address".to_string())),
    }
}

/// Execute memory fill instruction
///
/// Fills a region of memory with a given byte value.
pub fn memory_fill(stack: &mut dyn Stack, frame: &mut dyn FrameBehavior, mem_idx: u32, _engine: &StacklessEngine) -> Result<()> {
    let n = stack.pop()?.as_i32()? as usize;
    let val = stack.pop()?.as_i32()? as u8;
    let d = stack.pop()?.as_i32()? as usize;
    let memory = frame.get_memory_mut(mem_idx as usize)?;
    memory.fill(d, val, n)
}

/// Execute memory copy instruction
///
/// Copies data from one region of memory to another, possibly overlapping.
pub fn memory_copy(
    stack: &mut dyn Stack,
    frame: &mut dyn FrameBehavior,
    dst_mem: u32,
    src_mem: u32,
    _engine: &StacklessEngine,
) -> Result<()> {
    let n = stack.pop()?.as_i32()? as usize;
    let s = stack.pop()?.as_i32()? as usize;
    let d = stack.pop()?.as_i32()? as usize;
    let memory_src = frame.get_memory(src_mem as usize)?;
    let memory_dst = frame.get_memory_mut(dst_mem as usize)?;
    memory_dst.copy_within_or_between(memory_src, s, d, n)
}

/// Execute memory init instruction
///
/// Copies data from a passive data segment to memory.
pub fn memory_init(
    stack: &mut dyn Stack,
    frame: &mut dyn FrameBehavior,
    data_idx: u32,
    mem_idx: u32,
    _engine: &StacklessEngine,
) -> Result<()> {
    let n = stack.pop()?.as_i32()? as usize;
    let s = stack.pop()?.as_i32()? as usize;
    let d = stack.pop()?.as_i32()? as usize;
    let data_segment = frame.get_data_segment(data_idx)?;
    let memory = frame.get_memory_mut(mem_idx as usize)?;
    memory.init(d, &data_segment.init, s, n)
}

/// Execute data drop instruction
///
/// Drops a passive data segment, freeing its resources.
pub fn data_drop(_stack: &mut dyn Stack, frame: &mut dyn FrameBehavior, data_idx: u32, _engine: &StacklessEngine) -> Result<()> {
    frame.drop_data_segment(data_idx)
}

/// Load signed 8-bit value and extend to i64
pub fn i64_load8_s(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };
    let value = i64::from(frame.load_i8(addr, align)?);
    stack.push(Value::I64(value))
}

/// Load unsigned 8-bit value and extend to i64
pub fn i64_load8_u(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };
    let value = i64::from(frame.load_u8(addr, align)?);
    stack.push(Value::I64(value))
}

/// Load signed 16-bit value and extend to i64
pub fn i64_load16_s(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };
    let value = i64::from(frame.load_i16(addr, align)?);
    stack.push(Value::I64(value))
}

/// Load unsigned 16-bit value and extend to i64
pub fn i64_load16_u(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };
    let value = i64::from(frame.load_u16(addr, align)?);
    stack.push(Value::I64(value))
}

/// Load signed 32-bit value and extend to i64
pub fn i64_load32_s(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };
    let value = i64::from(frame.load_i32(addr, align)?);
    stack.push(Value::I64(value))
}

/// Load unsigned 32-bit value and extend to i64
pub fn i64_load32_u(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };
    let value = i64::from(frame.load_i32(addr, align)? as u32);
    stack.push(Value::I64(value))
}

/// Store f32 value
pub fn f32_store(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let value = match stack.pop()? {
        Value::F32(v) => v,
        _ => return Err(Error::TypeMismatch("Expected f32 value".to_string())),
    };
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };
    let memory = frame.get_memory(0)?;
    memory.check_alignment(addr as u32, 4, align)?;
    memory.write_f32(addr as u32, value)
}

/// Store f64 value
pub fn f64_store(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let value = match stack.pop()? {
        Value::F64(v) => v,
        _ => return Err(Error::TypeMismatch("Expected f64 value".to_string())),
    };
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };
    let memory = frame.get_memory(0)?;
    memory.check_alignment(addr as u32, 8, align)?;
    memory.write_f64(addr as u32, value)
}

/// Store low 8 bits of i32
pub fn i32_store8(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let value = match stack.pop()? {
        Value::I32(v) => v,
        _ => return Err(Error::TypeMismatch("Expected i32 value".to_string())),
    };
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };
    let memory = frame.get_memory(0)?;
    memory.check_alignment(addr as u32, 1, align)?;
    memory.write_u8(addr as u32, (value & 0xFF) as u8)
}

/// Store low 16 bits of i32
pub fn i32_store16(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let value = match stack.pop()? {
        Value::I32(v) => v,
        _ => return Err(Error::TypeMismatch("Expected i32 value".to_string())),
    };
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };
    let memory = frame.get_memory(0)?;
    memory.check_alignment(addr as u32, 2, align)?;
    memory.write_u16(addr as u32, (value & 0xFFFF) as u16)
}

/// Store low 8 bits of i64
pub fn i64_store8(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let value = match stack.pop()? {
        Value::I64(v) => v,
        _ => return Err(Error::TypeMismatch("Expected i64 value".to_string())),
    };
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };
    let memory = frame.get_memory(0)?;
    memory.check_alignment(addr as u32, 1, align)?;
    memory.write_u8(addr as u32, (value & 0xFF) as u8)
}

/// Store low 16 bits of i64
pub fn i64_store16(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let value = match stack.pop()? {
        Value::I64(v) => v,
        _ => return Err(Error::TypeMismatch("Expected i64 value".to_string())),
    };
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };
    let memory = frame.get_memory(0)?;
    memory.check_alignment(addr as u32, 2, align)?;
    memory.write_u16(addr as u32, (value & 0xFFFF) as u16)
}

/// Store low 32 bits of i64
pub fn i64_store32(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let value = match stack.pop()? {
        Value::I64(v) => v,
        _ => return Err(Error::TypeMismatch("Expected i64 value".to_string())),
    };
    let addr = match stack.pop()? {
        Value::I32(addr) => addr as usize + offset as usize,
        _ => return Err(Error::TypeMismatch("Expected i32 address".to_string())),
    };
    let memory = frame.get_memory(0)?;
    memory.check_alignment(addr as u32, 4, align)?;
    memory.write_u32(addr as u32, (value & 0xFFFFFFFF) as u32)
}

/// Execute a v128 store instruction
///
/// Stores a 128-bit value (16 bytes) to memory at the specified address.
pub fn v128_store(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let value = stack.pop()?.as_v128()?;
    let addr = stack.pop()?.as_i32()?;
    let memory = frame.get_memory(0)?;
    memory.check_alignment(addr as u32, 16, align)?;
    memory.write_v128(addr as u32, value)
}

/// Load a v128 value from memory.
///
/// Pops the base address (i32), calculates the effective address using the offset,
/// checks alignment, reads 16 bytes, and pushes the V128 value onto the stack.
pub fn v128_load(
    stack: &mut dyn Stack,
    frame: &dyn FrameBehavior,
    offset: u32,
    align: u32,
    engine: &StacklessEngine,
) -> Result<()> {
    let addr = stack.pop()?.as_i32()?;
    let memory = frame.get_memory(0)?;
    let effective_addr = (addr as u32).wrapping_add(offset);
    memory.check_alignment(effective_addr, 16, align)?;
    let bytes = memory.read_bytes(effective_addr, 16)?;
    let value: [u8; 16] = bytes.try_into().map_err(|v: Vec<u8>| {
        Error::Execution(format!("Expected 16 bytes for v128 load, got {}", v.len()))
    })?;
    stack.push(Value::V128(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::behavior::{ControlFlowBehavior, FrameBehavior, Label};
    use crate::error::{Error, Result};
    use crate::global::Global;
    use crate::memory::MemoryBehavior;
    use crate::table::Table;
    use crate::types::{BlockType, FuncType};
    use crate::values::Value;
    use std::sync::Arc;
    use std::sync::RwLock;

    // --- Mock Implementations ---\

    // Mock Memory Instance
    #[derive(Debug, Clone)]
    struct MockMemory {
        data: Arc<RwLock<Vec<u8>>>,
        mem_type: crate::types::MemoryType,
        peak_memory_used: Arc<RwLock<usize>>,
    }

    impl MockMemory {
        fn new(size_pages: u32) -> Self {
            let initial_size = size_pages as usize * crate::memory::PAGE_SIZE;
            MockMemory {
                data: Arc::new(RwLock::new(vec![0; initial_size])),
                mem_type: crate::types::MemoryType {
                    min: size_pages,
                    max: Some(size_pages),
                },
                peak_memory_used: Arc::new(RwLock::new(initial_size)),
            }
        }
        fn check_bounds(&self, addr: u32, len: u32) -> Result<()> {
            let data_len = self.data.read().unwrap().len();
            let end = addr.checked_add(len).ok_or_else(|| {
                Error::InvalidMemoryAccess(format!("Address overflow: addr={}, len={}", addr, len))
            })?;
            if (end as usize) > data_len {
                Err(Error::InvalidMemoryAccess(format!(
                    "Access out of bounds: addr={}, len={}, size={}",
                    addr, len, data_len
                )))
            } else {
                Ok(())
            }
        }
    }

    // Implement MemoryBehavior for MockMemory using interior mutability
    impl MemoryBehavior for MockMemory {
        fn type_(&self) -> &crate::types::MemoryType {
            &self.mem_type
        }
        fn size(&self) -> u32 {
            (self.data.read().unwrap().len() / crate::memory::PAGE_SIZE) as u32
        }
        fn size_bytes(&self) -> usize {
            self.data.read().unwrap().len()
        }
        fn grow(&self, pages: u32) -> Result<u32> {
            let old_size_bytes = self.data.read().unwrap().len();
            let old_size_pages = (old_size_bytes / crate::memory::PAGE_SIZE) as u32;

            let new_size_pages = old_size_pages
                .checked_add(pages)
                .ok_or_else(|| Error::MemoryGrowError("Addition overflow".to_string()))?;

            if let Some(max) = self.mem_type.max {
                if new_size_pages > max {
                    return Ok(u32::MAX);
                }
            }

            let new_size_bytes = new_size_pages as usize * crate::memory::PAGE_SIZE;

            let mut data_guard = self.data.write().unwrap();
            data_guard.resize(new_size_bytes, 0);

            let mut peak_guard = self.peak_memory_used.write().unwrap();
            *peak_guard = (*peak_guard).max(new_size_bytes);

            Ok(old_size_pages)
        }
        fn read_byte(&self, addr: u32) -> Result<u8> {
            self.check_bounds(addr, 1)?;
            Ok(self.data.read().unwrap()[addr as usize])
        }
        fn write_byte(&self, addr: u32, value: u8) -> Result<()> {
            self.check_bounds(addr, 1)?;
            self.data.write().unwrap()[addr as usize] = value;
            Ok(())
        }
        fn read_bytes(&self, addr: u32, len: usize) -> Result<Vec<u8>> {
            self.check_bounds(addr, len as u32)?;
            Ok(self.data.read().unwrap()[addr as usize..addr as usize + len].to_vec())
        }
        fn write_bytes(&self, addr: u32, bytes: &[u8]) -> Result<()> {
            self.check_bounds(addr, bytes.len() as u32)?;
            self.data.write().unwrap()[addr as usize..addr as usize + bytes.len()]
                .copy_from_slice(bytes);
            Ok(())
        }
        fn check_alignment(&self, addr: u32, _access_size: u32, align: u32) -> Result<()> {
            if align > 0 {
                let alignment = 1 << align;
                if addr % alignment != 0 {
                    return Err(Error::InvalidAlignment(format!(
                        "Invalid alignment: address {} is not aligned to {}",
                        addr, alignment
                    )));
                }
            }
            Ok(())
        }
        fn read_u16(&self, addr: u32) -> Result<u16> {
            self.check_bounds(addr, 2)?;
            let data_guard = self.data.read().unwrap();
            let bytes: [u8; 2] = data_guard[addr as usize..addr as usize + 2]
                .try_into()
                .map_err(|_| Error::Execution("Failed to read 2 bytes for u16".to_string()))?;
            Ok(u16::from_le_bytes(bytes))
        }
        fn write_u16(&self, addr: u32, value: u16) -> Result<()> {
            self.check_bounds(addr, 2)?;
            self.data.write().unwrap()[addr as usize..addr as usize + 2]
                .copy_from_slice(&value.to_le_bytes());
            Ok(())
        }
        fn read_i32(&self, addr: u32) -> Result<i32> {
            self.check_bounds(addr, 4)?;
            let data_guard = self.data.read().unwrap();
            let bytes: [u8; 4] = data_guard[addr as usize..addr as usize + 4]
                .try_into()
                .map_err(|_| Error::Execution("Failed to read 4 bytes for i32".to_string()))?;
            Ok(i32::from_le_bytes(bytes))
        }
        fn write_i32(&self, addr: u32, value: i32) -> Result<()> {
            self.check_bounds(addr, 4)?;
            self.data.write().unwrap()[addr as usize..addr as usize + 4]
                .copy_from_slice(&value.to_le_bytes());
            Ok(())
        }
        fn read_i64(&self, addr: u32) -> Result<i64> {
            self.check_bounds(addr, 8)?;
            let data_guard = self.data.read().unwrap();
            let bytes: [u8; 8] = data_guard[addr as usize..addr as usize + 8]
                .try_into()
                .map_err(|_| Error::Execution("Failed to read 8 bytes for i64".to_string()))?;
            Ok(i64::from_le_bytes(bytes))
        }
        fn write_i64(&self, addr: u32, value: i64) -> Result<()> {
            self.check_bounds(addr, 8)?;
            self.data.write().unwrap()[addr as usize..addr as usize + 8]
                .copy_from_slice(&value.to_le_bytes());
            Ok(())
        }
        fn read_f32(&self, addr: u32) -> Result<f32> {
            Ok(f32::from_bits(self.read_i32(addr)? as u32))
        }
        fn write_f32(&self, addr: u32, value: f32) -> Result<()> {
            self.write_i32(addr, value.to_bits() as i32)
        }
        fn read_f64(&self, addr: u32) -> Result<f64> {
            Ok(f64::from_bits(self.read_i64(addr)? as u64))
        }
        fn write_f64(&self, addr: u32, value: f64) -> Result<()> {
            self.write_i64(addr, value.to_bits() as i64)
        }
        fn read_v128(&self, addr: u32) -> Result<[u8; 16]> {
            self.check_bounds(addr, 16)?;
            let data_guard = self.data.read().unwrap();
            let bytes: [u8; 16] = data_guard[addr as usize..addr as usize + 16]
                .try_into()
                .map_err(|_| Error::Execution("Failed to read 16 bytes for v128".to_string()))?;
            Ok(bytes)
        }
        fn write_v128(&self, addr: u32, value: [u8; 16]) -> Result<()> {
            self.check_bounds(addr, 16)?;
            self.data.write().unwrap()[addr as usize..addr as usize + 16].copy_from_slice(&value);
            Ok(())
        }
    }

    // Mock Frame that holds a MockMemory
    #[derive(Clone)]
    struct MockFrame {
        memory: Arc<MockMemory>,
        locals: Vec<Value>,
        pc: usize,
        func_idx: u32,
        instance_idx: usize,
        label_stack: Vec<Label>,
        arity: usize,
        return_pc: usize,
    }

    impl MockFrame {
        fn new(memory_pages: u32) -> Self {
            MockFrame {
                memory: Arc::new(MockMemory::new(memory_pages)),
                locals: vec![],
                pc: 0,
                func_idx: 0,
                instance_idx: 0,
                label_stack: vec![],
                arity: 0,
                return_pc: 0,
            }
        }
    }

    // Implement FrameBehavior for MockFrame
    impl FrameBehavior for MockFrame {
        fn locals(&mut self) -> &mut Vec<Value> {
            &mut self.locals
        }
        fn get_local(&self, idx: usize) -> Result<Value> {
            self.locals
                .get(idx)
                .cloned()
                .ok_or(Error::InvalidLocalIndex(idx))
        }
        fn set_local(&mut self, idx: usize, value: Value) -> Result<()> {
            if idx < self.locals.len() {
                self.locals[idx] = value;
                Ok(())
            } else {
                Err(Error::InvalidLocalIndex(idx))
            }
        }
        fn get_global(&self, _idx: usize) -> Result<Arc<Global>> {
            Err(Error::Unimplemented("get_global mock".to_string()))
        }
        fn set_global(&mut self, _idx: usize, _value: Value) -> Result<()> {
            Err(Error::Unimplemented("set_global mock".to_string()))
        }
        fn get_table(&self, _idx: usize) -> Result<Arc<Table>> {
            Err(Error::Unimplemented("get_table mock".to_string()))
        }
        fn get_function_type(&self, _func_idx: u32) -> Result<FuncType> {
            Err(Error::Unimplemented("get_function_type mock".to_string()))
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
            self.instance_idx as u32
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
            self.label_stack.last().map_or(self.arity, |l| l.arity)
        }
        fn return_pc(&self) -> usize {
            self.return_pc
        }
        fn set_return_pc(&mut self, pc: usize) {
            self.return_pc = pc;
        }
        fn as_any(&mut self) -> &mut dyn std::any::Any {
            self
        }
        fn get_memory(&self, _idx: usize) -> Result<Arc<dyn MemoryBehavior>> {
            Ok(self.memory.clone())
        }
        fn get_memory_mut(&mut self, _idx: usize) -> Result<Arc<dyn MemoryBehavior>> {
            Ok(self.memory.clone())
        }
        fn get_table_mut(&mut self, _idx: usize) -> Result<Arc<Table>> {
            Err(Error::Unimplemented("get_table_mut mock".to_string()))
        }
        fn load_i32(&self, addr: usize, align: u32) -> Result<i32> {
            self.memory.check_alignment(addr as u32, 4, align)?;
            self.memory.read_i32(addr as u32)
        }
        fn store_i32(&mut self, addr: usize, align: u32, value: i32) -> Result<()> {
            self.memory.check_alignment(addr as u32, 4, align)?;
            self.memory.write_i32(addr as u32, value)
        }
        fn load_i64(&self, addr: usize, align: u32) -> Result<i64> {
            self.memory.check_alignment(addr as u32, 8, align)?;
            self.memory.read_i64(addr as u32)
        }
        fn store_i64(&mut self, addr: usize, align: u32, value: i64) -> Result<()> {
            self.memory.check_alignment(addr as u32, 8, align)?;
            self.memory.write_i64(addr as u32, value)
        }
        fn load_f32(&self, addr: usize, align: u32) -> Result<f32> {
            self.memory.check_alignment(addr as u32, 4, align)?;
            self.memory.read_f32(addr as u32)
        }
        fn store_f32(&mut self, addr: usize, align: u32, value: f32) -> Result<()> {
            self.memory.check_alignment(addr as u32, 4, align)?;
            self.memory.write_f32(addr as u32, value)
        }
        fn load_f64(&self, addr: usize, align: u32) -> Result<f64> {
            self.memory.check_alignment(addr as u32, 8, align)?;
            self.memory.read_f64(addr as u32)
        }
        fn store_f64(&mut self, addr: usize, align: u32, value: f64) -> Result<()> {
            self.memory.check_alignment(addr as u32, 8, align)?;
            self.memory.write_f64(addr as u32, value)
        }
        fn load_i8(&self, addr: usize, align: u32) -> Result<i8> {
            self.memory.check_alignment(addr as u32, 1, align)?;
            Ok(self.memory.read_byte(addr as u32)? as i8)
        }
        fn load_u8(&self, addr: usize, align: u32) -> Result<u8> {
            self.memory.check_alignment(addr as u32, 1, align)?;
            self.memory.read_byte(addr as u32)
        }
        fn load_i16(&self, addr: usize, align: u32) -> Result<i16> {
            self.memory.check_alignment(addr as u32, 2, align)?;
            Ok(self.memory.read_u16(addr as u32)? as i16)
        }
        fn load_u16(&self, addr: usize, align: u32) -> Result<u16> {
            self.memory.check_alignment(addr as u32, 2, align)?;
            self.memory.read_u16(addr as u32)
        }
        fn store_i8(&mut self, addr: usize, align: u32, value: i8) -> Result<()> {
            self.memory.check_alignment(addr as u32, 1, align)?;
            self.memory.write_byte(addr as u32, value as u8)
        }
        fn store_u8(&mut self, addr: usize, align: u32, value: u8) -> Result<()> {
            self.memory.check_alignment(addr as u32, 1, align)?;
            self.memory.write_byte(addr as u32, value)
        }
        fn store_i16(&mut self, addr: usize, align: u32, value: i16) -> Result<()> {
            self.memory.check_alignment(addr as u32, 2, align)?;
            self.memory.write_u16(addr as u32, value as u16)
        }
        fn store_u16(&mut self, addr: usize, align: u32, value: u16) -> Result<()> {
            self.memory.check_alignment(addr as u32, 2, align)?;
            self.memory.write_u16(addr as u32, value)
        }
        fn load_v128(&self, addr: usize, align: u32) -> Result<[u8; 16]> {
            self.memory.check_alignment(addr as u32, 16, align)?;
            self.memory.read_v128(addr as u32)
        }
        fn store_v128(&mut self, addr: usize, align: u32, value: [u8; 16]) -> Result<()> {
            self.memory.check_alignment(addr as u32, 16, align)?;
            self.memory.write_v128(addr as u32, value)
        }
        fn memory_size(&self) -> Result<u32> {
            Ok(self.memory.size())
        }
        fn memory_grow(&mut self, pages: u32) -> Result<u32> {
            self.memory.grow(pages)
        }
        fn table_get(&self, _table_idx: u32, _idx: u32) -> Result<Value> {
            Err(Error::Unimplemented("table_get mock".to_string()))
        }
        fn table_set(&mut self, _table_idx: u32, _idx: u32, _value: Value) -> Result<()> {
            Err(Error::Unimplemented("table_set mock".to_string()))
        }
        fn table_size(&self, _table_idx: u32) -> Result<u32> {
            Err(Error::Unimplemented("table_size mock".to_string()))
        }
        fn table_grow(&mut self, _table_idx: u32, _delta: u32, _value: Value) -> Result<u32> {
            Err(Error::Unimplemented("table_grow mock".to_string()))
        }
        fn table_init(
            &mut self,
            _table_idx: u32,
            _elem_idx: u32,
            _dst: u32,
            _src: u32,
            _n: u32,
        ) -> Result<()> {
            Err(Error::Unimplemented("table_init mock".to_string()))
        }
        fn table_copy(
            &mut self,
            _dst_table: u32,
            _src_table: u32,
            _dst: u32,
            _src: u32,
            _n: u32,
        ) -> Result<()> {
            Err(Error::Unimplemented("table_copy mock".to_string()))
        }
        fn elem_drop(&mut self, _elem_idx: u32) -> Result<()> {
            Err(Error::Unimplemented("elem_drop mock".to_string()))
        }
        fn table_fill(&mut self, _table_idx: u32, _dst: u32, _val: Value, _n: u32) -> Result<()> {
            Err(Error::Unimplemented("table_fill mock".to_string()))
        }
        fn pop_bool(&mut self, stack: &mut dyn Stack) -> Result<bool> {
            stack.pop_bool()
        }
        fn pop_i32(&mut self, stack: &mut dyn Stack) -> Result<i32> {
            stack.pop_i32()
        }
        fn get_two_tables_mut(&mut self, idx1: u32, idx2: u32) -> Result<(std::sync::MutexGuard<Table>, std::sync::MutexGuard<Table>)> {
            todo!()
        }
    }

    // Implement ControlFlowBehavior for MockFrame (minimal implementation for tests)
    impl ControlFlowBehavior for MockFrame {
        fn enter_block(&mut self, ty: BlockType, _stack_len: usize) -> Result<()> {
            let arity = match ty {
                BlockType::Empty | BlockType::FuncType(_) | BlockType::TypeIndex(_) => 0,
                BlockType::Type(_) | BlockType::Value(_) => 1,
            };
            self.label_stack.push(Label {
                arity,
                pc: self.pc + 1,
                continuation: self.pc + 1,
            });
            Ok(())
        }
        fn enter_loop(&mut self, ty: BlockType, _stack_len: usize) -> Result<()> {
            let arity = match ty {
                BlockType::Empty
                | BlockType::Type(_)
                | BlockType::Value(_)
                | BlockType::FuncType(_)
                | BlockType::TypeIndex(_) => 0,
            };
            self.label_stack.push(Label {
                arity,
                pc: self.pc + 1,
                continuation: self.pc,
            });
            Ok(())
        }
        fn enter_if(&mut self, ty: BlockType, _stack_len: usize, condition: bool) -> Result<()> {
            let arity = match ty {
                BlockType::Empty | BlockType::FuncType(_) | BlockType::TypeIndex(_) => 0,
                BlockType::Type(_) | BlockType::Value(_) => 1,
            };
            let continuation = if condition { self.pc } else { self.pc + 1 };
            let end_pc = self.pc + 2;
            self.label_stack.push(Label {
                arity,
                pc: end_pc,
                continuation,
            });
            self.pc = continuation;
            Ok(())
        }
        fn enter_else(&mut self, _stack_len: usize) -> Result<()> {
            if let Some(label) = self.label_stack.last_mut() {
                self.pc = label.pc;
            }
            Ok(())
        }
        fn exit_block(&mut self, _stack: &mut dyn Stack) -> Result<()> {
            if let Some(label) = self.label_stack.pop() {
                self.pc = label.pc;
            }
            Ok(())
        }
        fn branch(&mut self, label_idx: u32, _stack: &mut dyn Stack) -> Result<()> {
            let target_idx = self
                .label_stack
                .len()
                .checked_sub(1 + label_idx as usize)
                .ok_or(Error::InvalidLabelIndex(label_idx as usize))?;
            let label = &self.label_stack[target_idx];
            self.pc = label.pc;
            self.label_stack.truncate(target_idx + 1);
            Ok(())
        }
        fn return_(&mut self, _stack: &mut dyn Stack) -> Result<()> {
            self.pc = self.return_pc;
            self.label_stack.clear();
            Ok(())
        }
        fn call(&mut self, _func_idx: u32, _stack: &mut dyn Stack) -> Result<()> {
            Ok(())
        }
        fn call_indirect(
            &mut self,
            _type_idx: u32,
            _table_idx: u32,
            _entry: u32,
            _stack: &mut dyn Stack,
        ) -> Result<()> {
            Ok(())
        }
        fn set_label_arity(&mut self, _arity: usize) {
        }
    }

    // --- Tests ---

    #[test]
    fn test_i32_load() -> Result<()> {
        let memory = Arc::new(MockMemory::new(1));
        let mut frame = MockFrame {
            memory: memory.clone(),
            ..MockFrame::new(0)
        };
        let mut stack = Vec::<Value>::new();
        stack.push(Value::I32(0));
        memory.write_i32(0, 12345)?;
        i32_load(&mut stack, &mut frame, 0, 0, &StacklessEngine::new())?;
        assert_eq!(stack.pop().ok_or(Error::StackUnderflow)?, Value::I32(12345));
        Ok(())
    }

    #[test]
    fn test_i64_load() -> Result<()> {
        let memory = Arc::new(MockMemory::new(1));
        let mut frame = MockFrame {
            memory: memory.clone(),
            ..MockFrame::new(0)
        };
        let mut stack = Vec::<Value>::new();
        stack.push(Value::I32(8));
        memory.write_i64(8, 9876543210)?;
        i64_load(&mut stack, &mut frame, 0, 0, &StacklessEngine::new())?;
        assert_eq!(
            stack.pop().ok_or(Error::StackUnderflow)?,
            Value::I64(9876543210)
        );
        Ok(())
    }

    #[test]
    fn test_f32_load() -> Result<()> {
        let memory = Arc::new(MockMemory::new(1));
        let mut frame = MockFrame {
            memory: memory.clone(),
            ..MockFrame::new(0)
        };
        let mut stack = Vec::<Value>::new();
        stack.push(Value::I32(16));
        memory.write_f32(16, 3.14)?;
        f32_load(&mut stack, &mut frame, 0, 0, &StacklessEngine::new())?;
        assert_eq!(stack.pop().ok_or(Error::StackUnderflow)?, Value::F32(3.14));
        Ok(())
    }

    #[test]
    fn test_f64_load() -> Result<()> {
        let memory = Arc::new(MockMemory::new(1));
        let mut frame = MockFrame {
            memory: memory.clone(),
            ..MockFrame::new(0)
        };
        let mut stack = Vec::<Value>::new();
        stack.push(Value::I32(24));
        memory.write_f64(24, 2.71828)?;
        f64_load(&mut stack, &mut frame, 0, 0, &StacklessEngine::new())?;
        assert_eq!(
            stack.pop().ok_or(Error::StackUnderflow)?,
            Value::F64(2.71828)
        );
        Ok(())
    }

    #[test]
    fn test_i32_load8_s() -> Result<()> {
        let memory = Arc::new(MockMemory::new(1));
        let mut frame = MockFrame {
            memory: memory.clone(),
            ..MockFrame::new(0)
        };
        let mut stack = Vec::<Value>::new();
        stack.push(Value::I32(0));
        memory.write_byte(0, 0x80)?;
        i32_load8_s(&mut stack, &mut frame, 0, 0, &StacklessEngine::new())?;
        assert_eq!(stack.pop().ok_or(Error::StackUnderflow)?, Value::I32(-128));
        Ok(())
    }

    #[test]
    fn test_i32_load8_u() -> Result<()> {
        let memory = Arc::new(MockMemory::new(1));
        let mut frame = MockFrame {
            memory: memory.clone(),
            ..MockFrame::new(0)
        };
        let mut stack = Vec::<Value>::new();
        stack.push(Value::I32(0));
        memory.write_byte(0, 0x80)?;
        i32_load8_u(&mut stack, &mut frame, 0, 0, &StacklessEngine::new())?;
        assert_eq!(stack.pop().ok_or(Error::StackUnderflow)?, Value::I32(128));
        Ok(())
    }

    #[test]
    fn test_i32_load16_s() -> Result<()> {
        let memory = Arc::new(MockMemory::new(1));
        let mut frame = MockFrame {
            memory: memory.clone(),
            ..MockFrame::new(0)
        };
        let mut stack = Vec::<Value>::new();
        stack.push(Value::I32(0));
        memory.write_u16(0, 0x8000)?;
        i32_load16_s(&mut stack, &mut frame, 0, 0, &StacklessEngine::new())?;
        assert_eq!(
            stack.pop().ok_or(Error::StackUnderflow)?,
            Value::I32(-32768)
        );
        Ok(())
    }

    #[test]
    fn test_i32_load16_u() -> Result<()> {
        let memory = Arc::new(MockMemory::new(1));
        let mut frame = MockFrame {
            memory: memory.clone(),
            ..MockFrame::new(0)
        };
        let mut stack = Vec::<Value>::new();
        stack.push(Value::I32(0));
        memory.write_u16(0, 0x8000)?;
        i32_load16_u(&mut stack, &mut frame, 0, 0, &StacklessEngine::new())?;
        assert_eq!(stack.pop().ok_or(Error::StackUnderflow)?, Value::I32(32768));
        Ok(())
    }

    #[test]
    fn test_i32_store() -> Result<()> {
        let memory = Arc::new(MockMemory::new(1));
        let mut frame = MockFrame {
            memory: memory.clone(),
            ..MockFrame::new(0)
        };
        let mut stack = Vec::<Value>::new();
        stack.push(Value::I32(0));
        stack.push(Value::I32(54321));
        i32_store(&mut stack, &mut frame, 0, 0, &StacklessEngine::new())?;
        assert_eq!(memory.read_i32(0)?, 54321);
        Ok(())
    }

    #[test]
    fn test_i64_store() -> Result<()> {
        let memory = Arc::new(MockMemory::new(1));
        let mut frame = MockFrame {
            memory: memory.clone(),
            ..MockFrame::new(0)
        };
        let mut stack = Vec::<Value>::new();
        stack.push(Value::I32(8));
        stack.push(Value::I64(123456789012));
        i64_store(&mut stack, &mut frame, 0, 0, &StacklessEngine::new())?;
        assert_eq!(memory.read_i64(8)?, 123456789012);
        Ok(())
    }

    #[test]
    fn test_f32_store() -> Result<()> {
        let memory = Arc::new(MockMemory::new(1));
        let mut frame = MockFrame {
            memory: memory.clone(),
            ..MockFrame::new(0)
        };
        let mut stack = Vec::<Value>::new();
        stack.push(Value::I32(16));
        stack.push(Value::F32(1.618));
        f32_store(&mut stack, &mut frame, 0, 0, &StacklessEngine::new())?;
        assert_eq!(memory.read_f32(16)?, 1.618);
        Ok(())
    }

    #[test]
    fn test_f64_store() -> Result<()> {
        let memory = Arc::new(MockMemory::new(1));
        let mut frame = MockFrame {
            memory: memory.clone(),
            ..MockFrame::new(0)
        };
        let mut stack = Vec::<Value>::new();
        stack.push(Value::I32(24));
        stack.push(Value::F64(0.57721));
        f64_store(&mut stack, &mut frame, 0, 0, &StacklessEngine::new())?;
        assert_eq!(memory.read_f64(24)?, 0.57721);
        Ok(())
    }

    #[test]
    fn test_i32_store8() -> Result<()> {
        let memory = Arc::new(MockMemory::new(1));
        let mut frame = MockFrame {
            memory: memory.clone(),
            ..MockFrame::new(0)
        };
        let mut stack = Vec::<Value>::new();
        stack.push(Value::I32(32));
        stack.push(Value::I32(0x1234_ABCD));
        i32_store8(&mut stack, &mut frame, 0, 0, &StacklessEngine::new())?;
        assert_eq!(memory.read_byte(32)?, 0xCD);
        Ok(())
    }

    #[test]
    fn test_i32_store16() -> Result<()> {
        let memory = Arc::new(MockMemory::new(1));
        let mut frame = MockFrame {
            memory: memory.clone(),
            ..MockFrame::new(0)
        };
        let mut stack = Vec::<Value>::new();
        stack.push(Value::I32(34));
        stack.push(Value::I32(0x1234_ABCD));
        i32_store16(&mut stack, &mut frame, 0, 0, &StacklessEngine::new())?;
        assert_eq!(memory.read_u16(34)?, 0xABCD);
        Ok(())
    }

    #[test]
    fn test_i64_store8() -> Result<()> {
        let memory = Arc::new(MockMemory::new(1));
        let mut frame = MockFrame {
            memory: memory.clone(),
            ..MockFrame::new(0)
        };
        let mut stack = Vec::<Value>::new();
        stack.push(Value::I32(0));
        stack.push(Value::I64(0x1234_5678_9ABC_DEF0));
        i64_store8(&mut stack, &mut frame, 0, 0, &StacklessEngine::new())?;
        assert_eq!(memory.read_byte(0)?, 0xF0);
        Ok(())
    }

    #[test]
    fn test_i64_store16() -> Result<()> {
        let memory = Arc::new(MockMemory::new(1));
        let mut frame = MockFrame {
            memory: memory.clone(),
            ..MockFrame::new(0)
        };
        let mut stack = Vec::<Value>::new();
        stack.push(Value::I32(2));
        stack.push(Value::I64(0x1234_5678_9ABC_DEF0));
        i64_store16(&mut stack, &mut frame, 0, 1, &StacklessEngine::new())?;
        assert_eq!(memory.read_u16(2)?, 0xDEF0);
        Ok(())
    }

    #[test]
    fn test_i64_store32() -> Result<()> {
        let memory = Arc::new(MockMemory::new(1));
        let mut frame = MockFrame {
            memory: memory.clone(),
            ..MockFrame::new(0)
        };
        let mut stack = Vec::<Value>::new();
        stack.push(Value::I32(4));
        stack.push(Value::I64(0x1234_5678_9ABC_DEF0));
        i64_store32(&mut stack, &mut frame, 0, 2, &StacklessEngine::new())?;
        assert_eq!(memory.read_i32(4)?, 0x9ABC_DEF0u32 as i32);
        Ok(())
    }

    #[test]
    fn test_v128_load() -> Result<()> {
        let memory = Arc::new(MockMemory::new(1));
        let mut frame = MockFrame {
            memory: memory.clone(),
            ..MockFrame::new(0)
        };
        let mut stack = Vec::<Value>::new();
        let v128_data: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        memory.write_bytes(16, &v128_data)?;
        stack.push(Value::I32(16));
        v128_load(&mut stack, &mut frame, 0, 4, &StacklessEngine::new())?;
        assert_eq!(
            stack.pop().ok_or(Error::StackUnderflow)?,
            Value::V128(v128_data)
        );
        Ok(())
    }

    #[test]
    fn test_v128_store() -> Result<()> {
        let memory = Arc::new(MockMemory::new(1));
        let mut frame = MockFrame {
            memory: memory.clone(),
            ..MockFrame::new(0)
        };
        let mut stack = Vec::<Value>::new();
        let v128_data: [u8; 16] = [15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0];
        stack.push(Value::I32(32));
        stack.push(Value::V128(v128_data));
        v128_store(&mut stack, &mut frame, 0, 4, &StacklessEngine::new())?;
        let stored_bytes = memory.read_bytes(32, 16)?;
        assert_eq!(stored_bytes, v128_data);
        Ok(())
    }
}
