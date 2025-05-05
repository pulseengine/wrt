//! Memory adapter for wrt
//!
//! This adapter provides safe, bounded memory access
//! with integrated memory safety features for WebAssembly memory instances.

// Use our prelude for consistent imports
use crate::prelude::*;

// Import specific memory-related types
use wrt_runtime::Memory as RuntimeMemory;
use wrt_runtime::MemoryType as RuntimeMemoryType;
use wrt_types::safe_memory::{MemoryProvider, MemorySafety, MemoryStats, StdMemoryProvider};

use core::ops::Range;

/// Memory adapter interface for working with memory
pub trait MemoryAdapter {
    /// Get the memory backing this adapter
    fn memory(&self) -> Arc<RuntimeMemory>;

    /// Read bytes from memory at the given offset
    fn read_bytes(&self, offset: u32, len: u32) -> WrtResult<Vec<u8>>;

    /// Write bytes to memory at the given offset
    fn write_bytes(&self, offset: u32, bytes: &[u8]) -> WrtResult<()>;

    /// Get the size of the memory in pages
    fn size(&self) -> WrtResult<u32>;

    /// Grow the memory by the given number of pages
    fn grow(&self, pages: u32) -> WrtResult<u32>;

    /// Get the number of bytes in the memory
    fn byte_size(&self) -> WrtResult<usize>;

    /// Check if a range is valid for the memory
    fn check_range(&self, offset: u32, size: u32) -> WrtResult<()>;
}

/// Safe memory adapter implementation
pub struct SafeMemoryAdapter {
    /// The underlying memory
    memory: Arc<RuntimeMemory>,
    /// The memory provider for safety checks
    provider: StdMemoryProvider,
}

impl SafeMemoryAdapter {
    /// Create a new memory adapter with the given memory type
    pub fn new(memory_type: RuntimeMemoryType) -> WrtResult<crate::memory::Memory> {
        let memory = RuntimeMemory::new(memory_type)
            .map_err(|e| WrtError::new(crate::error::kinds::MemoryCreationError(format!("{e}"))))?;

        // Create a new adapter with the memory
        let arc_memory = Arc::new(memory);
        let data = arc_memory.buffer()?;
        let provider = StdMemoryProvider::new(data);

        Ok(crate::memory::Memory::Adapter(SafeMemoryAdapter {
            memory: arc_memory,
            provider,
        }))
    }

    /// Create a new adapter with the given memory
    pub fn from_memory(memory: Arc<RuntimeMemory>) -> WrtResult<Self> {
        let data = memory.buffer()?;
        let provider = StdMemoryProvider::new(data);
        Ok(Self { memory, provider })
    }

    /// Create a new adapter with the given memory and verification level
    pub fn with_verification_level(
        memory: Arc<RuntimeMemory>,
        level: VerificationLevel,
    ) -> WrtResult<Self> {
        let data = memory.buffer()?;
        let mut provider = StdMemoryProvider::new(data);
        provider.set_verification_level(level);
        Ok(Self { memory, provider })
    }

    /// Get the memory provider for this adapter
    pub fn memory_provider(&self) -> &StdMemoryProvider {
        &self.provider
    }

    /// Synchronize the provider with memory
    fn sync_provider(&mut self) -> WrtResult<()> {
        let data = self.memory.buffer()?;
        // Create a new provider with updated data
        self.provider = StdMemoryProvider::new(data);
        Ok(())
    }
}

// Implement the MemorySafety trait for SafeMemoryAdapter
impl MemorySafety for SafeMemoryAdapter {
    fn verify_integrity(&self) -> WrtResult<()> {
        self.memory.verify_integrity()
    }

    fn set_verification_level(&mut self, level: VerificationLevel) {
        self.provider.set_verification_level(level);
    }

    fn verification_level(&self) -> VerificationLevel {
        self.provider.verification_level()
    }

    fn memory_stats(&self) -> MemoryStats {
        let size = self.memory.size().unwrap_or(0);
        let access_count = self.memory.access_count();
        let peak_usage = self.memory.peak_usage();

        MemoryStats {
            size,
            access_count,
            peak_usage,
            bounds_checks: self.provider.bounds_check_count(),
            pointer_checks: self.provider.pointer_check_count(),
        }
    }
}

// Implement the MemoryAdapter trait for SafeMemoryAdapter
impl MemoryAdapter for SafeMemoryAdapter {
    fn memory(&self) -> Arc<RuntimeMemory> {
        self.memory.clone()
    }

    fn read_bytes(&self, offset: u32, len: u32) -> WrtResult<Vec<u8>> {
        // Check that the range is valid
        self.check_range(offset, len)?;

        // Read the bytes from memory
        let data = self.memory.read_bytes(offset, len)?;
        Ok(data)
    }

    fn write_bytes(&self, offset: u32, bytes: &[u8]) -> WrtResult<()> {
        // Check that the range is valid
        self.check_range(offset, bytes.len() as u32)?;

        // Write the bytes to memory
        self.memory.write_bytes(offset, bytes)
    }

    fn size(&self) -> WrtResult<u32> {
        Ok(self.memory.size())
    }

    fn grow(&self, pages: u32) -> WrtResult<u32> {
        // Since the memory is wrapped in an Arc, we need to handle mutability differently.
        // This is a simplified implementation - in a real implementation, you would need
        // proper synchronization.
        let result = self.memory.size();

        // Make a mutable clone of the memory for the growth operation
        // In a real implementation, this would use proper locking or atomics
        // But for now we'll use this simplified approach
        let memory_clone = self.memory.clone();
        let mut_memory = Arc::get_mut(&mut memory_clone.clone()).ok_or_else(|| {
            WrtError::new(crate::error::kinds::MemoryAccessError(
                "Failed to get mutable access to memory for grow operation".to_string(),
            ))
        })?;

        // Grow the memory
        mut_memory.grow(pages)?;

        // Return the previous size
        Ok(result)
    }

    fn byte_size(&self) -> WrtResult<usize> {
        Ok(self.memory.size() as usize * 65536)
    }

    fn check_range(&self, offset: u32, size: u32) -> WrtResult<()> {
        let mem_size = self.byte_size()?;
        let end_offset = offset as usize + size as usize;

        if end_offset > mem_size {
            Err(WrtError::new(error_kinds::MemoryAccessOutOfBoundsError {
                address: offset as u64,
                length: size as u64,
            }))
        } else {
            Ok(())
        }
    }
}

// Helper methods for reading and writing various types
impl SafeMemoryAdapter {
    /// Read an i8 from memory at the given offset
    pub fn read_i8(&self, offset: u32) -> WrtResult<i8> {
        let bytes = self.read_bytes(offset, 1)?;
        Ok(bytes[0] as i8)
    }

    /// Read a u8 from memory at the given offset
    pub fn read_u8(&self, offset: u32) -> WrtResult<u8> {
        let bytes = self.read_bytes(offset, 1)?;
        Ok(bytes[0])
    }

    /// Read an i16 from memory at the given offset
    pub fn read_i16(&self, offset: u32) -> WrtResult<i16> {
        let bytes = self.read_bytes(offset, 2)?;
        let value = i16::from_le_bytes([bytes[0], bytes[1]]);
        Ok(value)
    }

    /// Read a u16 from memory at the given offset
    pub fn read_u16(&self, offset: u32) -> WrtResult<u16> {
        let bytes = self.read_bytes(offset, 2)?;
        let value = u16::from_le_bytes([bytes[0], bytes[1]]);
        Ok(value)
    }

    /// Read an i32 from memory at the given offset
    pub fn read_i32(&self, offset: u32) -> WrtResult<i32> {
        let bytes = self.read_bytes(offset, 4)?;
        let value = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        Ok(value)
    }

    /// Read a u32 from memory at the given offset
    pub fn read_u32(&self, offset: u32) -> WrtResult<u32> {
        let bytes = self.read_bytes(offset, 4)?;
        let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        Ok(value)
    }

    /// Read an i64 from memory at the given offset
    pub fn read_i64(&self, offset: u32) -> WrtResult<i64> {
        let bytes = self.read_bytes(offset, 8)?;
        let value = i64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        Ok(value)
    }

    /// Read a u64 from memory at the given offset
    pub fn read_u64(&self, offset: u32) -> WrtResult<u64> {
        let bytes = self.read_bytes(offset, 8)?;
        let value = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        Ok(value)
    }

    /// Read an f32 from memory at the given offset
    pub fn read_f32(&self, offset: u32) -> WrtResult<f32> {
        let bytes = self.read_bytes(offset, 4)?;
        let value = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        Ok(value)
    }

    /// Read an f64 from memory at the given offset
    pub fn read_f64(&self, offset: u32) -> WrtResult<f64> {
        let bytes = self.read_bytes(offset, 8)?;
        let value = f64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        Ok(value)
    }

    /// Read a v128 from memory at the given offset
    pub fn read_v128(&self, offset: u32) -> WrtResult<[u8; 16]> {
        let bytes = self.read_bytes(offset, 16)?;
        let mut result = [0u8; 16];
        result.copy_from_slice(&bytes);
        Ok(result)
    }

    /// Write an i8 to memory at the given offset
    pub fn write_i8(&self, offset: u32, value: i8) -> WrtResult<()> {
        self.write_bytes(offset, &[value as u8])
    }

    /// Write a u8 to memory at the given offset
    pub fn write_u8(&self, offset: u32, value: u8) -> WrtResult<()> {
        self.write_bytes(offset, &[value])
    }

    /// Write an i16 to memory at the given offset
    pub fn write_i16(&self, offset: u32, value: i16) -> WrtResult<()> {
        self.write_bytes(offset, &value.to_le_bytes())
    }

    /// Write a u16 to memory at the given offset
    pub fn write_u16(&self, offset: u32, value: u16) -> WrtResult<()> {
        self.write_bytes(offset, &value.to_le_bytes())
    }

    /// Write an i32 to memory at the given offset
    pub fn write_i32(&self, offset: u32, value: i32) -> WrtResult<()> {
        self.write_bytes(offset, &value.to_le_bytes())
    }

    /// Write a u32 to memory at the given offset
    pub fn write_u32(&self, offset: u32, value: u32) -> WrtResult<()> {
        self.write_bytes(offset, &value.to_le_bytes())
    }

    /// Write an i64 to memory at the given offset
    pub fn write_i64(&self, offset: u32, value: i64) -> WrtResult<()> {
        self.write_bytes(offset, &value.to_le_bytes())
    }

    /// Write a u64 to memory at the given offset
    pub fn write_u64(&self, offset: u32, value: u64) -> WrtResult<()> {
        self.write_bytes(offset, &value.to_le_bytes())
    }

    /// Write an f32 to memory at the given offset
    pub fn write_f32(&self, offset: u32, value: f32) -> WrtResult<()> {
        self.write_bytes(offset, &value.to_le_bytes())
    }

    /// Write an f64 to memory at the given offset
    pub fn write_f64(&self, offset: u32, value: f64) -> WrtResult<()> {
        self.write_bytes(offset, &value.to_le_bytes())
    }

    /// Write a v128 to memory at the given offset
    pub fn write_v128(&self, offset: u32, value: [u8; 16]) -> WrtResult<()> {
        self.write_bytes(offset, &value)
    }
}
