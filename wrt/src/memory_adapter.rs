//! Memory adapter for wrt
//!
//! This adapter provides safe, bounded memory access
//! with integrated memory safety features for WebAssembly memory instances.

use wrt_runtime::Memory as RuntimeMemory;
use wrt_types::{
    safe_memory::{MemoryProvider, MemorySafety, SafeSlice, StdMemoryProvider},
    verification::VerificationLevel,
};

use core::ops::Range;
use std::sync::Arc;
use wrt_error::{kinds, Error, Result};

/// Interface for memory implementations used by the engine
pub trait MemoryAdapter {
    /// Load data from memory
    fn load(&self, offset: usize, len: usize) -> Result<Box<[u8]>>;

    /// Store data to memory
    fn store(&self, offset: usize, data: &[u8]) -> Result<()>;

    /// Get the memory size in pages
    fn size(&self) -> Result<u32>;

    /// Get the memory size in bytes
    fn byte_size(&self) -> Result<usize>;

    /// Grow memory by a number of pages
    fn grow(&self, pages: u32) -> Result<u32>;

    /// Verify memory integrity and safety
    fn verify_integrity(&self) -> Result<()>;
}

/// A memory adapter with SafeMemory features
#[derive(Clone, Debug)]
pub struct SafeMemoryAdapter {
    /// The underlying memory implementation
    memory: Arc<RuntimeMemory>,
    /// The SafeMemory provider for memory safety
    provider: Arc<StdMemoryProvider>,
    /// Verification level for memory operations
    verification_level: VerificationLevel,
}

impl SafeMemoryAdapter {
    /// Create a new safety-enhanced memory adapter
    pub fn new(memory: Arc<RuntimeMemory>) -> Self {
        // Create a provider with the current memory data
        let data = match memory.buffer() {
            Ok(data) => data.to_vec(),
            Err(_) => Vec::new(),
        };

        Self {
            memory,
            provider: Arc::new(StdMemoryProvider::new(data)),
            verification_level: VerificationLevel::Standard,
        }
    }
    
    /// Create a new memory adapter with a specific verification level
    pub fn with_verification_level(memory: Arc<RuntimeMemory>, level: VerificationLevel) -> Self {
        let mut adapter = Self::new(memory);
        adapter.verification_level = level;
        adapter
    }
    
    /// Set the verification level for memory operations
    pub fn set_verification_level(&mut self, level: VerificationLevel) {
        self.verification_level = level;
    }
    
    /// Get the current verification level
    pub fn verification_level(&self) -> VerificationLevel {
        self.verification_level
    }
    
    /// Synchronize the provider with the underlying memory
    fn sync_provider(&self) -> Result<Arc<StdMemoryProvider>> {
        // Get the current memory buffer
        let buffer = self.memory.buffer()?;
        
        // Create a new provider with the current data
        let provider = StdMemoryProvider::new(buffer.to_vec());
        
        Ok(Arc::new(provider))
    }
    
    /// Get memory safety statistics
    pub fn memory_stats(&self) -> Result<wrt_types::safe_memory::MemoryStats> {
        let provider = self.sync_provider()?;
        Ok(provider.memory_stats())
    }
}

impl MemoryAdapter for SafeMemoryAdapter {
    fn load(&self, offset: usize, len: usize) -> Result<Box<[u8]>> {
        // For high verification levels, always sync provider for fresh checks
        let provider = if matches!(self.verification_level, VerificationLevel::Full) {
            self.sync_provider()?
        } else {
            self.provider.clone()
        };
        
        // Use the provider to get a validated safe slice
        let safe_slice = provider.borrow_slice(offset, len)?;
        
        // Get the data with integrity check
        let data = safe_slice.data()?;
        
        Ok(data.to_vec().into_boxed_slice())
    }

    fn store(&self, offset: usize, data: &[u8]) -> Result<()> {
        // Verify bounds before storing
        let byte_size = self.byte_size()?;
        if offset + data.len() > byte_size {
            return Err(Error::new(kinds::OutOfBoundsError(format!(
                "Memory access out of bounds: offset={}, len={}, memory_size={}",
                offset, data.len(), byte_size
            ))));
        }
        
        // Store via the runtime memory
        self.memory.store(offset, data)?;
        
        // For Full verification, sync provider after every write
        if matches!(self.verification_level, VerificationLevel::Full) {
            self.sync_provider()?;
        }
        
        Ok(())
    }

    fn size(&self) -> Result<u32> {
        self.memory.size()
    }

    fn byte_size(&self) -> Result<usize> {
        let pages = self.size()?;
        Ok(pages as usize * 65536)
    }

    fn grow(&self, pages: u32) -> Result<u32> {
        let result = self.memory.grow(pages)?;
        
        // After growing, sync the provider
        if !matches!(self.verification_level, VerificationLevel::None) {
            self.sync_provider()?;
        }
        
        Ok(result)
    }
    
    fn verify_integrity(&self) -> Result<()> {
        // Sync provider to get the latest state
        let provider = self.sync_provider()?;
        
        // Verify memory integrity
        provider.verify_integrity()
    }
}

/// A default memory adapter (without safety features)
#[derive(Clone)]
pub struct DefaultMemoryAdapter {
    /// The underlying memory implementation
    memory: Arc<RuntimeMemory>,
}

impl DefaultMemoryAdapter {
    /// Create a new memory adapter
    pub fn new(memory: Arc<RuntimeMemory>) -> Self {
        Self { memory }
    }
}

impl MemoryAdapter for DefaultMemoryAdapter {
    fn load(&self, offset: usize, len: usize) -> Result<Box<[u8]>> {
        let buffer = self.memory.buffer()?;
        
        // Bounds check
        if offset + len > buffer.len() {
            return Err(Error::new(kinds::OutOfBoundsError(format!(
                "Memory access out of bounds: offset={}, len={}, buffer_size={}",
                offset, len, buffer.len()
            ))));
        }
        
        let data = buffer[offset..offset + len].to_vec();
        Ok(data.into_boxed_slice())
    }

    fn store(&self, offset: usize, data: &[u8]) -> Result<()> {
        self.memory.store(offset, data)
    }

    fn size(&self) -> Result<u32> {
        self.memory.size()
    }

    fn byte_size(&self) -> Result<usize> {
        let pages = self.size()?;
        Ok(pages as usize * 65536)
    }

    fn grow(&self, pages: u32) -> Result<u32> {
        self.memory.grow(pages)
    }
    
    fn verify_integrity(&self) -> Result<()> {
        // Basic memory adapter does not have integrity checks
        Ok(())
    }
} 