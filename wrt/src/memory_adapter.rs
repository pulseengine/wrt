//! Memory adapter for wrt
//!
//! This adapter provides safe, bounded memory access
//! with integrated memory safety features for WebAssembly memory instances.

use std::sync::Arc;
use wrt_error::{kinds, Error, Result};
use wrt_runtime::memory::MemoryArcExt;
use wrt_runtime::Memory;
use wrt_runtime::MemoryType;
use wrt_types::safe_memory::{MemoryProvider, MemorySafety, MemoryStats, StdMemoryProvider};
use wrt_types::verification::VerificationLevel;
use wrt_types::Value;

#[cfg(not(feature = "std"))]
use alloc::sync::Arc;
#[cfg(feature = "std")]
use std::sync::Arc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

use core::ops::Range;

/// Memory adapter interface for working with memory
pub trait MemoryAdapter {
    /// Get the memory backing this adapter
    fn memory(&self) -> Arc<Memory>;

    /// Get the size of the memory in bytes
    fn size(&self) -> Result<usize>;

    /// Get the size of the memory in bytes - alternative method name
    /// for compatibility
    fn byte_size(&self) -> Result<usize> {
        self.size()
    }

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
        Self { memory, provider }
    }

    /// Create a new adapter with the given memory and verification level
    pub fn with_verification_level(
        memory: Arc<Memory>,
        level: wrt_types::VerificationLevel,
    ) -> Self {
        let data = memory.buffer().to_vec();
        let mut provider = StdMemoryProvider::new(data);
        provider.set_verification_level(level);
        Self { memory, provider }
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

// Implement the MemorySafety trait for SafeMemoryAdapter
impl MemorySafety for SafeMemoryAdapter {
    fn verify_integrity(&self) -> Result<()> {
        self.memory.verify_integrity()
    }

    fn set_verification_level(&mut self, level: wrt_types::VerificationLevel) {
        self.provider.set_verification_level(level);
    }

    fn verification_level(&self) -> wrt_types::VerificationLevel {
        self.provider.verification_level()
    }

    fn memory_stats(&self) -> MemoryStats {
        let size = self.memory.size().unwrap_or(0);
        let access_count = self.memory.access_count();
        let peak_usage = self.memory.peak_usage();

        MemoryStats {
            total_size: size,
            access_count: access_count as usize,
            unique_regions: 1, // For now, we don't track unique regions
            max_access_size: peak_usage,
        }
    }
}

impl MemoryAdapter for SafeMemoryAdapter {
    fn memory(&self) -> Arc<Memory> {
        self.memory.clone()
    }

    fn size(&self) -> Result<usize> {
        Ok(self.memory.size_in_bytes())
    }

    fn load(&self, offset: usize, len: usize) -> Result<Vec<u8>> {
        let mut buffer = vec![0; len];
        self.memory.read(offset as u32, &mut buffer)?;
        Ok(buffer)
    }

    fn store(&self, offset: usize, data: &[u8]) -> Result<()> {
        self.memory.write(offset as u32, data)
    }

    fn grow(&self, pages: u32) -> Result<usize> {
        self.memory.grow(pages)
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
        if offset + len > self.size()? {
            return Err(Error::new(kinds::MemoryOutOfBoundsError));
        }
        self.memory.read(offset as u32, len as u32)
    }

    fn store(&self, offset: usize, data: &[u8]) -> Result<()> {
        if offset + data.len() > self.size()? {
            return Err(Error::new(kinds::MemoryOutOfBoundsError));
        }
        self.memory.write(offset as u32, data)
    }

    fn grow(&self, pages: u32) -> Result<usize> {
        // Use the arc_grow method from MemoryArcExt
        let old_size = self.memory.arc_grow(pages)?;
        Ok(old_size as usize)
    }
}
