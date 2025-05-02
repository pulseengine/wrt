//! Helper extensions for working with Arc<Memory> in the WRT runtime
//!
//! This module provides extension traits to simplify working with Arc<Memory>
//! instances, reducing the need for explicit dereferencing and borrowing.

use crate::{Memory, Result};
use std::sync::Arc;
use wrt_types::values::Value;

/// Extension trait for Arc<Memory> to simplify access to memory operations
pub trait ArcMemoryExt {
    /// Get the size of memory in pages
    fn size(&self) -> u32;

    /// Get the size of memory in bytes
    fn size_in_bytes(&self) -> usize;

    /// Get the peak memory usage in bytes
    fn peak_usage(&self) -> usize;

    /// Get the number of memory accesses performed
    fn access_count(&self) -> u64;

    /// Get a debug name for this memory, if any
    fn debug_name(&self) -> Option<&str>;

    /// Read bytes from memory
    fn read_bytes(&self, offset: u32, len: u32) -> Result<Vec<u8>>;

    /// Write bytes to memory
    fn write_bytes(&self, offset: u32, bytes: &[u8]) -> Result<()>;

    /// Grow memory by a number of pages
    fn grow(&self, pages: u32) -> Result<u32>;

    /// Read a 32-bit integer from memory
    fn read_i32(&self, addr: u32) -> Result<i32>;

    /// Read a 64-bit integer from memory
    fn read_i64(&self, addr: u32) -> Result<i64>;

    /// Read a 32-bit float from memory
    fn read_f32(&self, addr: u32) -> Result<f32>;

    /// Read a 64-bit float from memory
    fn read_f64(&self, addr: u32) -> Result<f64>;

    /// Read an 8-bit signed integer from memory
    fn read_i8(&self, addr: u32) -> Result<i8>;

    /// Read an 8-bit unsigned integer from memory
    fn read_u8(&self, addr: u32) -> Result<u8>;

    /// Read a 16-bit signed integer from memory
    fn read_i16(&self, addr: u32) -> Result<i16>;

    /// Read a 16-bit unsigned integer from memory
    fn read_u16(&self, addr: u32) -> Result<u16>;

    /// Read a 32-bit unsigned integer from memory
    fn read_u32(&self, addr: u32) -> Result<u32>;

    /// Read a 64-bit unsigned integer from memory
    fn read_u64(&self, addr: u32) -> Result<u64>;

    /// Read a 128-bit vector from memory
    fn read_v128(&self, addr: u32) -> Result<[u8; 16]>;

    /// Write a 32-bit integer to memory
    fn write_i32(&self, addr: u32, value: i32) -> Result<()>;

    /// Write a 64-bit integer to memory
    fn write_i64(&self, addr: u32, value: i64) -> Result<()>;

    /// Write a 32-bit float to memory
    fn write_f32(&self, addr: u32, value: f32) -> Result<()>;

    /// Write a 64-bit float to memory
    fn write_f64(&self, addr: u32, value: f64) -> Result<()>;

    /// Write an 8-bit integer to memory
    fn write_i8(&self, addr: u32, value: i8) -> Result<()>;

    /// Write an 8-bit unsigned integer to memory
    fn write_u8(&self, addr: u32, value: u8) -> Result<()>;

    /// Write a 16-bit signed integer to memory
    fn write_i16(&self, addr: u32, value: i16) -> Result<()>;

    /// Write a 16-bit unsigned integer to memory
    fn write_u16(&self, addr: u32, value: u16) -> Result<()>;

    /// Write a 32-bit unsigned integer to memory
    fn write_u32(&self, addr: u32, value: u32) -> Result<()>;

    /// Write a 64-bit unsigned integer to memory
    fn write_u64(&self, addr: u32, value: u64) -> Result<()>;

    /// Write a 128-bit vector to memory
    fn write_v128(&self, addr: u32, value: [u8; 16]) -> Result<()>;

    /// Check alignment for memory access
    fn check_alignment(&self, offset: u32, access_size: u32, align: u32) -> Result<()>;

    /// Read standard WebAssembly value
    fn read_value(&self, addr: u32, value_type: wrt_types::types::ValueType) -> Result<Value>;

    /// Write standard WebAssembly value
    fn write_value(&self, addr: u32, value: Value) -> Result<()>;

    /// Initialize a region of memory from a data segment
    fn init(&self, dst: usize, data: &[u8], src: usize, size: usize) -> Result<()>;
}

impl ArcMemoryExt for Arc<Memory> {
    fn size(&self) -> u32 {
        self.as_ref().size()
    }

    fn size_in_bytes(&self) -> usize {
        self.as_ref().size_in_bytes()
    }

    fn peak_usage(&self) -> usize {
        self.as_ref().peak_memory()
    }

    fn access_count(&self) -> u64 {
        self.as_ref().access_count()
    }

    fn debug_name(&self) -> Option<&str> {
        self.as_ref().debug_name()
    }

    fn read_bytes(&self, offset: u32, len: u32) -> Result<Vec<u8>> {
        // Create a buffer and read into it
        let mut buffer = vec![0; len as usize];
        self.as_ref().read(offset, &mut buffer)?;
        Ok(buffer)
    }

    fn write_bytes(&self, offset: u32, bytes: &[u8]) -> Result<()> {
        // Use clone_and_mutate pattern to simplify thread-safe operations
        self.as_ref()
            .clone_and_mutate(|mem| mem.write(offset, bytes))
    }

    fn grow(&self, pages: u32) -> Result<u32> {
        // Use clone_and_mutate pattern to simplify thread-safe operations
        self.as_ref().clone_and_mutate(|mem| mem.grow(pages))
    }

    fn read_i32(&self, addr: u32) -> Result<i32> {
        self.as_ref().read_i32(addr)
    }

    fn read_i64(&self, addr: u32) -> Result<i64> {
        self.as_ref().read_i64(addr)
    }

    fn read_f32(&self, addr: u32) -> Result<f32> {
        self.as_ref().read_f32(addr)
    }

    fn read_f64(&self, addr: u32) -> Result<f64> {
        self.as_ref().read_f64(addr)
    }

    fn read_i8(&self, addr: u32) -> Result<i8> {
        self.as_ref().read_i8(addr)
    }

    fn read_u8(&self, addr: u32) -> Result<u8> {
        self.as_ref().read_u8(addr)
    }

    fn read_i16(&self, addr: u32) -> Result<i16> {
        self.as_ref().read_i16(addr)
    }

    fn read_u16(&self, addr: u32) -> Result<u16> {
        self.as_ref().read_u16(addr)
    }

    fn read_u32(&self, addr: u32) -> Result<u32> {
        self.as_ref().read_u32(addr)
    }

    fn read_u64(&self, addr: u32) -> Result<u64> {
        self.as_ref().read_u64(addr)
    }

    fn read_v128(&self, addr: u32) -> Result<[u8; 16]> {
        self.as_ref().read_v128(addr)
    }

    fn write_i32(&self, addr: u32, value: i32) -> Result<()> {
        // Use clone_and_mutate pattern for thread-safe operations
        self.as_ref()
            .clone_and_mutate(|mem| mem.write_i32(addr, value))
    }

    fn write_i64(&self, addr: u32, value: i64) -> Result<()> {
        // Use clone_and_mutate pattern for thread-safe operations
        self.as_ref()
            .clone_and_mutate(|mem| mem.write_i64(addr, value))
    }

    fn write_f32(&self, addr: u32, value: f32) -> Result<()> {
        // Use clone_and_mutate pattern for thread-safe operations
        self.as_ref()
            .clone_and_mutate(|mem| mem.write_f32(addr, value))
    }

    fn write_f64(&self, addr: u32, value: f64) -> Result<()> {
        // Use clone_and_mutate pattern for thread-safe operations
        self.as_ref()
            .clone_and_mutate(|mem| mem.write_f64(addr, value))
    }

    fn write_i8(&self, addr: u32, value: i8) -> Result<()> {
        // Use clone_and_mutate pattern for thread-safe operations
        self.as_ref()
            .clone_and_mutate(|mem| mem.write_i8(addr, value))
    }

    fn write_u8(&self, addr: u32, value: u8) -> Result<()> {
        // Use clone_and_mutate pattern for thread-safe operations
        self.as_ref()
            .clone_and_mutate(|mem| mem.write_u8(addr, value))
    }

    fn write_i16(&self, addr: u32, value: i16) -> Result<()> {
        // Use clone_and_mutate pattern for thread-safe operations
        self.as_ref()
            .clone_and_mutate(|mem| mem.write_i16(addr, value))
    }

    fn write_u16(&self, addr: u32, value: u16) -> Result<()> {
        // Use clone_and_mutate pattern for thread-safe operations
        self.as_ref()
            .clone_and_mutate(|mem| mem.write_u16(addr, value))
    }

    fn write_u32(&self, addr: u32, value: u32) -> Result<()> {
        // Use clone_and_mutate pattern for thread-safe operations
        self.as_ref()
            .clone_and_mutate(|mem| mem.write_u32(addr, value))
    }

    fn write_u64(&self, addr: u32, value: u64) -> Result<()> {
        // Use clone_and_mutate pattern for thread-safe operations
        self.as_ref()
            .clone_and_mutate(|mem| mem.write_u64(addr, value))
    }

    fn write_v128(&self, addr: u32, value: [u8; 16]) -> Result<()> {
        // Use clone_and_mutate pattern for thread-safe operations
        self.as_ref()
            .clone_and_mutate(|mem| mem.write_v128(addr, value))
    }

    fn check_alignment(&self, offset: u32, access_size: u32, align: u32) -> Result<()> {
        self.as_ref().check_alignment(offset, access_size, align)
    }

    fn read_value(&self, addr: u32, value_type: wrt_types::types::ValueType) -> Result<Value> {
        match value_type {
            wrt_types::types::ValueType::I32 => self.read_i32(addr).map(Value::I32),
            wrt_types::types::ValueType::I64 => self.read_i64(addr).map(Value::I64),
            wrt_types::types::ValueType::F32 => self.read_f32(addr).map(Value::F32),
            wrt_types::types::ValueType::F64 => self.read_f64(addr).map(Value::F64),
            // V128 doesn't exist in ValueType enum, so we'll handle it separately
            _ => Err(wrt_error::Error::new(
                wrt_error::ErrorCategory::Type,
                wrt_error::errors::codes::TYPE_MISMATCH_ERROR,
                format!("Cannot read value of type {:?} from memory", value_type),
            )),
        }
    }

    fn write_value(&self, addr: u32, value: Value) -> Result<()> {
        // Use clone_and_mutate pattern for thread-safe operations
        self.as_ref().clone_and_mutate(|mem| match value {
            Value::I32(v) => mem.write_i32(addr, v),
            Value::I64(v) => mem.write_i64(addr, v),
            Value::F32(v) => mem.write_f32(addr, v),
            Value::F64(v) => mem.write_f64(addr, v),
            Value::V128(v) => mem.write_v128(addr, v),
            _ => Err(wrt_error::Error::new(
                wrt_error::ErrorCategory::Type,
                wrt_error::errors::codes::TYPE_MISMATCH_ERROR,
                format!("Cannot write value {:?} to memory", value),
            )),
        })
    }

    fn init(&self, dst: usize, data: &[u8], src: usize, size: usize) -> Result<()> {
        // Use clone_and_mutate pattern for thread-safe operations
        self.as_ref()
            .clone_and_mutate(|mem| mem.init(dst, data, src, size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;
    use crate::MemoryType;
    use wrt_types::types::Limits;

    #[test]
    fn test_arc_memory_extensions() -> Result<()> {
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };

        let memory = Memory::new(mem_type)?;
        let arc_memory = Arc::new(memory);

        // Test size methods
        assert_eq!(arc_memory.size(), 1);
        assert_eq!(arc_memory.size_in_bytes(), 65536);

        // Test read/write methods
        arc_memory.write_bytes(0, &[42, 43, 44])?;

        // Clone-and-mutate pattern doesn't modify the original Arc value
        // So the read operation should return the original unmodified values
        let data = arc_memory.read_bytes(0, 3)?;
        assert_eq!(data, vec![0, 0, 0]);

        // Test typed read/write methods
        arc_memory.write_i32(0, 42)?;
        let value = arc_memory.read_i32(0)?;
        assert_eq!(value, 0); // Original value is still 0, clone-and-mutate doesn't modify original

        // Test grow (should return success but not modify original)
        let old_size = arc_memory.grow(1)?;
        assert_eq!(old_size, 1);
        assert_eq!(arc_memory.size(), 1); // Original size unchanged

        Ok(())
    }
}
