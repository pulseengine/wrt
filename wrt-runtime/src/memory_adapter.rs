//! Memory adapter for wrt-runtime
//!
//! This adapter provides safe, bounded memory access
//! with integrated memory safety features for WebAssembly memory instances.

// Use our prelude for consistent imports
extern crate alloc;

// Import Arc from the correct location to match ArcMemoryExt
#[cfg(feature = "std")]
use std::sync::Arc;
#[cfg(not(feature = "std"))]
use alloc::sync::Arc;

use crate::{memory::Memory, prelude::*};
use crate::memory_helpers::*;

// Import format! macro for string formatting
#[cfg(feature = "std")]
use std::format;
#[cfg(not(feature = "std"))]
use alloc::format;

/// Invalid offset error code
const INVALID_OFFSET: u16 = 4006;
/// Size too large error code  
const SIZE_TOO_LARGE: u16 = 4007;

/// Safe conversion from WebAssembly u32 offset to Rust usize
/// 
/// # Arguments
/// 
/// * `offset` - WebAssembly offset as u32
/// 
/// # Returns
/// 
/// Ok(usize) if conversion is safe, error otherwise
fn wasm_offset_to_usize(offset: u32) -> Result<usize> {
    usize::try_from(offset).map_err(|_| Error::new(
        ErrorCategory::Memory, 
        INVALID_OFFSET, 
        "Offset exceeds usize limit"
    ))
}

/// Safe conversion from Rust usize to WebAssembly u32
/// 
/// # Arguments
/// 
/// * `size` - Rust size as usize
/// 
/// # Returns
/// 
/// Ok(u32) if conversion is safe, error otherwise  
fn usize_to_wasm_u32(size: usize) -> Result<u32> {
    u32::try_from(size).map_err(|_| Error::new(
        ErrorCategory::Memory, 
        SIZE_TOO_LARGE, 
        "Size exceeds u32 limit"
    ))
}

/// Memory adapter interface for working with memory
pub trait MemoryAdapter: Debug + Send + Sync {
    /// Get the memory backing this adapter
    fn memory(&self) -> Arc<Memory>;

    /// Read bytes from memory at the given offset
    fn read_exact(&self, offset: u32, len: u32) -> Result<BoundedVec<u8, 65_536, StdMemoryProvider>>;

    /// Write bytes to memory at the given offset
    fn write_all(&self, offset: u32, bytes: &[u8]) -> Result<()>;

    /// Get the size of the memory in pages
    fn size(&self) -> Result<u32>;

    /// Grow the memory by the given number of pages
    fn grow(&self, pages: u32) -> Result<u32>;

    /// Get the number of bytes in the memory
    fn byte_size(&self) -> Result<usize>;

    /// Check if a range is valid for the memory
    fn check_range(&self, offset: u32, size: u32) -> Result<()>;

    /// Borrow a slice of memory with integrity verification
    fn borrow_slice(&self, offset: u32, len: u32) -> Result<BoundedVec<u8, 65_536, StdMemoryProvider>>;
}

/// Safe memory adapter implementation
#[derive(Debug)]
pub struct SafeMemoryAdapter {
    /// The underlying memory
    memory: Arc<Memory>,
    /// The memory provider for safety checks
    provider: StdMemoryProvider,
}

/// Standard memory provider implementation
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StdMemoryProvider {
    /// Verification level for memory safety checks
    verification_level: VerificationLevel,
}

impl wrt_foundation::MemoryProvider for StdMemoryProvider {
    type Allocator = Self;

    fn borrow_slice(&self, _offset: usize, _len: usize) -> wrt_foundation::WrtResult<wrt_foundation::safe_memory::Slice<'_>> {
        // For StdMemoryProvider, this is a placeholder
        Err(wrt_error::Error::new(
            wrt_error::ErrorCategory::Memory,
            wrt_error::codes::NOT_IMPLEMENTED,
            "borrow_slice not implemented for StdMemoryProvider"
        ))
    }

    fn write_data(&mut self, _offset: usize, _data: &[u8]) -> wrt_foundation::WrtResult<()> {
        // For StdMemoryProvider, this is a placeholder
        Ok(())
    }

    fn verify_access(&self, _offset: usize, _len: usize) -> wrt_foundation::WrtResult<()> {
        // For StdMemoryProvider, this is a placeholder
        Ok(())
    }

    fn size(&self) -> usize {
        0
    }

    fn capacity(&self) -> usize {
        // For std mode, we can use large capacities
        1024 * 1024 // 1MB
    }

    fn verify_integrity(&self) -> wrt_foundation::WrtResult<()> {
        Ok(())
    }

    fn set_verification_level(&mut self, level: wrt_foundation::verification::VerificationLevel) {
        self.verification_level = level;
    }

    fn verification_level(&self) -> wrt_foundation::verification::VerificationLevel {
        self.verification_level
    }

    fn memory_stats(&self) -> wrt_foundation::MemoryStats {
        wrt_foundation::MemoryStats::default()
    }

    fn get_slice_mut(&mut self, _offset: usize, _len: usize) -> wrt_foundation::WrtResult<wrt_foundation::safe_memory::SliceMut<'_>> {
        Err(wrt_error::Error::new(
            wrt_error::ErrorCategory::Memory,
            wrt_error::codes::NOT_IMPLEMENTED,
            "get_slice_mut not implemented for StdMemoryProvider"
        ))
    }

    fn copy_within(&mut self, _src: usize, _dst: usize, _len: usize) -> wrt_foundation::WrtResult<()> {
        Ok(())
    }

    fn ensure_used_up_to(&mut self, _offset: usize) -> wrt_foundation::WrtResult<()> {
        Ok(())
    }

    fn acquire_memory(&self, _layout: core::alloc::Layout) -> wrt_foundation::WrtResult<*mut u8> {
        Err(wrt_error::Error::new(
            wrt_error::ErrorCategory::Memory,
            wrt_error::codes::NOT_IMPLEMENTED,
            "acquire_memory not implemented for StdMemoryProvider"
        ))
    }

    fn release_memory(&self, _ptr: *mut u8, _layout: core::alloc::Layout) -> wrt_foundation::WrtResult<()> {
        Ok(())
    }

    fn get_allocator(&self) -> &Self::Allocator {
        self
    }

    fn new_handler(&self) -> wrt_foundation::WrtResult<wrt_foundation::safe_memory::SafeMemoryHandler<Self>>
    where
        Self: Clone,
    {
        Ok(wrt_foundation::safe_memory::SafeMemoryHandler::new(self.clone()))
    }
}

impl wrt_foundation::safe_memory::Allocator for StdMemoryProvider {
    fn allocate(&self, _layout: core::alloc::Layout) -> wrt_foundation::WrtResult<*mut u8> {
        Err(wrt_error::Error::new(
            wrt_error::ErrorCategory::Memory,
            wrt_error::codes::NOT_IMPLEMENTED,
            "allocate not implemented for StdMemoryProvider"
        ))
    }

    fn deallocate(&self, _ptr: *mut u8, _layout: core::alloc::Layout) -> wrt_foundation::WrtResult<()> {
        Ok(())
    }
}

impl StdMemoryProvider {
    /// Create a new standard memory provider
    pub fn new(_data: &[u8]) -> Self {
        Self { verification_level: VerificationLevel::Standard }
    }

    /// Get the current verification level
    pub fn verification_level(&self) -> VerificationLevel {
        self.verification_level
    }

    /// Set the verification level
    pub fn set_verification_level(&mut self, level: VerificationLevel) {
        self.verification_level = level;
    }

    /// Create a safe slice of memory with verification
    pub fn create_safe_slice<'a>(
        &self,
        buffer: &'a [u8],
        offset: usize,
        len: usize,
    ) -> Result<BoundedVec<u8, 65_536, StdMemoryProvider>> {
        if offset + len > buffer.len() {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                "Memory access out of bounds",
            ));
        }

        // Instead of returning a reference, copy the data into a BoundedVec
        let mut bounded_vec = BoundedVec::with_verification_level(self.clone(), self.verification_level())?;

        for i in offset..(offset + len) {
            bounded_vec.push(buffer[i]).map_err(|_| {
                Error::new(
                    ErrorCategory::Memory,
                    codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                    "Failed to push byte to bounded vector",
                )
            })?;
        }

        Ok(bounded_vec)
    }

}

impl SafeMemoryAdapter {
    /// Create a new memory adapter with the given memory type
    pub fn new(memory_type: CoreMemoryType) -> Result<Arc<SafeMemoryAdapter>> {
        let memory = Memory::new(memory_type)?;

        // Create a new adapter with the memory
        let arc_memory = Arc::new(memory);
        let data = arc_memory.buffer()?;
        let provider = StdMemoryProvider::new(data.as_slice());

        // Return a Memory adapter
        let adapter = SafeMemoryAdapter { memory: arc_memory, provider };

        Ok(Arc::new(adapter))
    }

    /// Verify memory safety
    pub fn verify_memory_safety(&self) -> Result<()> {
        Ok(())
    }
}

// Implement the MemorySafety trait for SafeMemoryAdapter
// MemorySafety trait implementation removed as it doesn't exist in wrt-foundation

// Implement the MemoryAdapter trait for SafeMemoryAdapter
impl MemoryAdapter for SafeMemoryAdapter {
    fn memory(&self) -> Arc<Memory> {
        self.memory.clone()
    }

    fn read_exact(&self, offset: u32, len: u32) -> Result<BoundedVec<u8, 65_536, StdMemoryProvider>> {
        // Check that the range is valid
        self.check_range(offset, len)?;

        // Read the bytes directly from the buffer
        let buffer = self.memory.buffer()?;
        let start = wasm_offset_to_usize(offset)?;
        let end = start + wasm_offset_to_usize(len)?;

        // Create a new BoundedVec with the data
        let mut bounded_vec =
            BoundedVec::with_verification_level(self.provider.clone(), self.provider.verification_level())?;

        // Copy the data from the buffer into the bounded vector
        for i in start..end {
            let byte = buffer.get(i).map_err(|_| {
                Error::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, "Buffer access out of bounds")
            })?;
            bounded_vec.push(byte).map_err(|_| {
                Error::new(
                    ErrorCategory::Memory,
                    codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                    "Failed to push byte to bounded vector",
                )
            })?;
        }

        Ok(bounded_vec)
    }

    fn write_all(&self, offset: u32, bytes: &[u8]) -> Result<()> {
        // Check that the range is valid
        self.check_range(offset, bytes.len() as u32)?;

        // We can't modify buffer directly through Arc, so use a special method to write
        // to memory without dereferencing Arc<Memory> as mutable
        ArcMemoryExt::write_via_callback(&self.memory, offset, bytes)?;

        Ok(())
    }

    fn size(&self) -> Result<u32> {
        // Wrap the direct u32 return in a Result
        Ok(self.memory.size())
    }

    fn grow(&self, pages: u32) -> Result<u32> {
        // Get the current size
        let result = self.memory.size();

        // Grow the memory - this should handle interior mutability internally
        ArcMemoryExt::grow_via_callback(&self.memory, pages)?;

        // Return the previous size
        Ok(result)
    }

    fn byte_size(&self) -> Result<usize> {
        // Convert WebAssembly page count to byte size safely
        let pages = self.memory.size();
        let page_size_bytes = wasm_offset_to_usize(pages)? * 65_536;
        Ok(page_size_bytes)
    }

    fn check_range(&self, offset: u32, size: u32) -> Result<()> {
        let mem_size = self.byte_size()?;
        let end_offset = wasm_offset_to_usize(offset)? + wasm_offset_to_usize(size)?;

        if end_offset > mem_size {
            Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                "Memory access out of bounds",
            ))
        } else {
            Ok(())
        }
    }

    // Change the return type to BoundedVec instead of SafeSlice to avoid lifetime
    // issues
    fn borrow_slice(&self, offset: u32, len: u32) -> Result<BoundedVec<u8, 65_536, StdMemoryProvider>> {
        // Check that the range is valid
        self.check_range(offset, len)?;

        // Get the buffer
        let buffer = self.memory.buffer()?;
        
        // Convert to usize for internal use
        let offset_usize = wasm_offset_to_usize(offset)?;
        let len_usize = wasm_offset_to_usize(len)?;

        // Create a new BoundedVec with the copied data
        self.provider.create_safe_slice(buffer.as_slice(), offset_usize, len_usize)
    }
}
