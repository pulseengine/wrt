use crate::error::{Error, Result};
use crate::types::*;
use crate::Vec;
#[cfg(not(feature = "std"))]
use alloc::vec;
#[cfg(feature = "std")]
use std::vec;

/// Size of a WebAssembly memory page in bytes
pub const PAGE_SIZE: usize = 65536;

/// Represents a WebAssembly memory instance
#[derive(Debug)]
pub struct Memory {
    /// Memory type
    mem_type: MemoryType,
    /// Memory data
    data: Vec<u8>,
}

impl Memory {
    /// Creates a new memory instance
    pub fn new(mem_type: MemoryType) -> Self {
        let initial_size = mem_type.min as usize * PAGE_SIZE;
        Self {
            mem_type,
            data: vec![0; initial_size],
        }
    }

    /// Returns the memory type
    pub fn type_(&self) -> &MemoryType {
        &self.mem_type
    }

    /// Returns the current size in pages
    pub fn size(&self) -> u32 {
        (self.data.len() / PAGE_SIZE) as u32
    }

    /// Grows the memory by the specified number of pages
    pub fn grow(&mut self, delta: u32) -> Result<u32> {
        let old_size = self.size();
        let new_size = old_size
            .checked_add(delta)
            .ok_or_else(|| Error::Execution("Memory size overflow".into()))?;

        if new_size > self.mem_type.max.unwrap_or(u32::MAX) {
            return Err(Error::Execution("Memory size exceeds maximum".into()));
        }

        let new_size_bytes = new_size as usize * PAGE_SIZE;
        self.data.resize(new_size_bytes, 0);
        Ok(old_size)
    }

    /// Reads a byte from memory
    pub fn read_byte(&self, addr: u32) -> Result<u8> {
        self.check_bounds(addr, 1)?;
        Ok(self.data[addr as usize])
    }

    /// Writes a byte to memory
    pub fn write_byte(&mut self, addr: u32, value: u8) -> Result<()> {
        self.check_bounds(addr, 1)?;
        self.data[addr as usize] = value;
        Ok(())
    }

    /// Reads a 16-bit integer from memory
    ///
    /// # Panics
    ///
    /// This function will panic if the underlying slice doesn't have exactly 2 bytes
    /// for conversion to a u16 with `try_into().unwrap()`. This should never happen
    /// as long as the bounds checking is working correctly.
    pub fn read_u16(&self, addr: u32) -> Result<u16> {
        self.check_bounds(addr, 2)?;
        let bytes = &self.data[addr as usize..addr as usize + 2];
        Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
    }

    /// Writes a 16-bit integer to memory
    pub fn write_u16(&mut self, addr: u32, value: u16) -> Result<()> {
        self.check_bounds(addr, 2)?;
        let bytes = value.to_le_bytes();
        self.data[addr as usize..addr as usize + 2].copy_from_slice(&bytes);
        Ok(())
    }

    /// Reads a 32-bit integer from memory
    ///
    /// # Panics
    ///
    /// Panics if the memory slice cannot be converted to a 4-byte array, which should never
    /// happen as long as the bounds check passes.
    pub fn read_u32(&self, addr: u32) -> Result<u32> {
        self.check_bounds(addr, 4)?;
        let bytes = &self.data[addr as usize..addr as usize + 4];
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    /// Writes a 32-bit integer to memory
    pub fn write_u32(&mut self, addr: u32, value: u32) -> Result<()> {
        self.check_bounds(addr, 4)?;
        let bytes = value.to_le_bytes();
        self.data[addr as usize..addr as usize + 4].copy_from_slice(&bytes);
        Ok(())
    }

    /// Reads a 64-bit integer from memory
    ///
    /// # Panics
    ///
    /// Panics if the memory slice cannot be converted to an 8-byte array, which should never
    /// happen as long as the bounds check passes.
    pub fn read_u64(&self, addr: u32) -> Result<u64> {
        self.check_bounds(addr, 8)?;
        let bytes = &self.data[addr as usize..addr as usize + 8];
        Ok(u64::from_le_bytes(bytes.try_into().unwrap()))
    }

    /// Writes a 64-bit integer to memory
    pub fn write_u64(&mut self, addr: u32, value: u64) -> Result<()> {
        self.check_bounds(addr, 8)?;
        let bytes = value.to_le_bytes();
        self.data[addr as usize..addr as usize + 8].copy_from_slice(&bytes);
        Ok(())
    }

    /// Reads a 32-bit float from memory
    ///
    /// # Panics
    ///
    /// Panics if the memory slice cannot be converted to a 4-byte array, which should never
    /// happen as long as the bounds check passes.
    pub fn read_f32(&self, addr: u32) -> Result<f32> {
        self.check_bounds(addr, 4)?;
        let bytes = &self.data[addr as usize..addr as usize + 4];
        Ok(f32::from_le_bytes(bytes.try_into().unwrap()))
    }

    /// Writes a 32-bit float to memory
    pub fn write_f32(&mut self, addr: u32, value: f32) -> Result<()> {
        self.check_bounds(addr, 4)?;
        let bytes = value.to_le_bytes();
        self.data[addr as usize..addr as usize + 4].copy_from_slice(&bytes);
        Ok(())
    }

    /// Reads a 64-bit float from memory
    ///
    /// # Panics
    ///
    /// Panics if the memory slice cannot be converted to an 8-byte array, which should never
    /// happen as long as the bounds check passes.
    pub fn read_f64(&self, addr: u32) -> Result<f64> {
        self.check_bounds(addr, 8)?;
        let bytes = &self.data[addr as usize..addr as usize + 8];
        Ok(f64::from_le_bytes(bytes.try_into().unwrap()))
    }

    /// Writes a 64-bit float to memory
    pub fn write_f64(&mut self, addr: u32, value: f64) -> Result<()> {
        self.check_bounds(addr, 8)?;
        let bytes = value.to_le_bytes();
        self.data[addr as usize..addr as usize + 8].copy_from_slice(&bytes);
        Ok(())
    }

    /// Reads a vector of bytes from memory
    pub fn read_bytes(&self, addr: u32, len: usize) -> Result<&[u8]> {
        // Safely convert len to u32, handling potential overflow
        let len_u32 =
            u32::try_from(len).map_err(|_| Error::Execution("Memory length too large".into()))?;
        self.check_bounds(addr, len_u32)?;
        Ok(&self.data[addr as usize..addr as usize + len])
    }

    /// Writes a vector of bytes to memory
    pub fn write_bytes(&mut self, addr: u32, bytes: &[u8]) -> Result<()> {
        // Safely convert bytes.len() to u32, handling potential overflow
        let len_u32 = u32::try_from(bytes.len())
            .map_err(|_| Error::Execution("Memory length too large".into()))?;
        self.check_bounds(addr, len_u32)?;
        self.data[addr as usize..addr as usize + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    /// Checks if a memory access is within bounds
    fn check_bounds(&self, addr: u32, len: u32) -> Result<()> {
        let end = addr
            .checked_add(len)
            .ok_or_else(|| Error::Execution("Memory access overflow".into()))?;

        // Convert to usize for Rust vec bounds check, carefully handling conversion
        let end_usize = end as usize;

        if end_usize > self.data.len() {
            return Err(Error::Execution("Memory access out of bounds".into()));
        }
        Ok(())
    }
}

// PAGE_SIZE is defined at the top of this file
