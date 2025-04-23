//! Memory adapter for wrt
//!
//! This adapter provides safe, bounded memory access
//! with integrated memory safety features for WebAssembly memory instances.

use std::sync::Arc;
use wrt_error::{Error, Result, kinds};
use wrt_runtime::Memory;
use wrt_runtime::memory::MemoryArcExt;
use wrt_types::safe_memory::{MemoryProvider, StdMemoryProvider};
use wrt_types::BoundedCapacity;

use core::ops::Range;

/// Memory adapter interface for working with memory
pub trait MemoryAdapter {
    /// Get the memory backing this adapter
    fn memory(&self) -> Arc<Memory>;

    /// Get the size of the memory in bytes
    fn size(&self) -> Result<usize>;

    /// Load data from memory
    fn load(&self, offset: usize, len: usize) -> Result<Vec<u8>>;

    /// Store data into memory
    fn store(&self, offset: usize, data: &[u8]) -> Result<()>;

    /// Grow memory by number of pages
    fn grow(&self, pages: u32) -> Result<usize>;
}

/// Memory adapter with safety guarantees
pub struct SafeMemoryAdapter {
    /// Underlying memory
    memory: Arc<Memory>,
    /// Memory provider
    provider: StdMemoryProvider,
}

impl SafeMemoryAdapter {
    /// Create a new adapter with the given memory
    pub fn new(memory: Arc<Memory>) -> Self {
        let data = memory.buffer().to_vec();
        let provider = StdMemoryProvider::new(data);
        Self {
            memory,
            provider,
        }
    }

    /// Get the memory provider for this adapter
    pub fn memory_provider(&self) -> &StdMemoryProvider {
        &self.provider
    }

    /// Synchronize the provider with memory
    fn sync_provider(&mut self) -> Result<()> {
        let data = self.memory.buffer().to_vec();
        // Create a new provider with updated data
        self.provider = StdMemoryProvider::new(data);
        Ok(())
    }
}

impl MemoryAdapter for SafeMemoryAdapter {
    fn memory(&self) -> Arc<Memory> {
        self.memory.clone()
    }

    fn size(&self) -> Result<usize> {
        Ok(self.memory.size() as usize * wrt_runtime::PAGE_SIZE)
    }

    fn load(&self, offset: usize, len: usize) -> Result<Vec<u8>> {
        // Use the provider to verify and get data
        let slice = self.provider.borrow_slice(offset, len)?;
        
        // Convert safe slice to Vec<u8>
        Ok(slice.data()?.to_vec())
    }

    fn store(&self, offset: usize, data: &[u8]) -> Result<()> {
        // Check bounds first
        let end_offset = offset
            .checked_add(data.len())
            .ok_or_else(|| Error::new(kinds::MemoryAccessOutOfBoundsError))?;
        
        if end_offset > self.memory.buffer().len() {
            return Err(Error::new(kinds::MemoryAccessOutOfBoundsError));
        }

        // Use the arc_write method from MemoryArcExt
        self.memory.arc_write(offset as u32, data)
    }

    fn grow(&self, pages: u32) -> Result<usize> {
        // Use the arc_grow method from MemoryArcExt
        let old_size = self.memory.arc_grow(pages)?;
        
        Ok(old_size as usize)
    }
}

/// Default memory adapter without additional safety checks
pub struct DefaultMemoryAdapter {
    /// Underlying memory
    memory: Arc<Memory>,
}

impl DefaultMemoryAdapter {
    /// Create a new adapter with the given memory
    pub fn new(memory: Arc<Memory>) -> Self {
        Self { memory }
    }
}

impl MemoryAdapter for DefaultMemoryAdapter {
    fn memory(&self) -> Arc<Memory> {
        self.memory.clone()
    }

    fn size(&self) -> Result<usize> {
        Ok(self.memory.size() as usize * wrt_runtime::PAGE_SIZE)
    }

    fn load(&self, offset: usize, len: usize) -> Result<Vec<u8>> {
        // Bounds check
        let mem_data = self.memory.buffer();
        let end_offset = offset
            .checked_add(len)
            .ok_or_else(|| Error::new(kinds::MemoryAccessOutOfBoundsError))?;
            
        if end_offset > mem_data.len() {
            return Err(Error::new(kinds::MemoryAccessOutOfBoundsError));
        }
        
        // Return a slice of memory
        Ok(mem_data[offset..end_offset].to_vec())
    }

    fn store(&self, offset: usize, data: &[u8]) -> Result<()> {
        // Bounds check
        let mem_size = self.memory.buffer().len();
        let end_offset = offset
            .checked_add(data.len())
            .ok_or_else(|| Error::new(kinds::MemoryAccessOutOfBoundsError))?;
            
        if end_offset > mem_size {
            return Err(Error::new(kinds::MemoryAccessOutOfBoundsError));
        }
        
        // Use the arc_write method from MemoryArcExt
        self.memory.arc_write(offset as u32, data)
    }

    fn grow(&self, pages: u32) -> Result<usize> {
        // Use the arc_grow method from MemoryArcExt
        let old_size = self.memory.arc_grow(pages)?;
        Ok(old_size as usize)
    }
} 