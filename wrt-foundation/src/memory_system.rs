// WRT - wrt-foundation
// Module: Unified Memory Provider Hierarchy
// SW-REQ-ID: REQ_MEM_UNIFIED_001, REQ_MEM_HIERARCHY_001, REQ_MEM_PLATFORM_002
//
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Unified Memory Provider Hierarchy for WRT Foundation
//!
//! This module provides a consistent memory provider architecture that can be
//! configured for different platform requirements. It establishes a hierarchy
//! of memory providers with different capacity and performance characteristics.
//!
//! # Design Principles
//!
//! - **Unified Interface**: All memory providers implement the same trait
//! - **Platform Configurability**: Different provider sizes for different platforms
//! - **Safety**: Memory allocation failures are handled gracefully
//! - **Predictability**: Fixed-size providers with known memory bounds
//! - **Performance**: Zero-allocation providers for no_std environments
//!
//! # Memory Provider Hierarchy
//!
//! ```text
//! UnifiedMemoryProvider (trait)
//! ├── ConfigurableProvider<SIZE> (generic fixed-size provider)
//! │   ├── SmallProvider (8KB)
//! │   ├── MediumProvider (64KB)
//! │   └── LargeProvider (1MB)
//! ├── NoStdProvider<SIZE> (existing provider, integrated)
//! └── StdMemoryProvider (std-only, delegating to system allocator)
//! ```
//!
//! # Usage
//!
//! ```rust
//! use wrt_foundation::memory_system::{UnifiedMemoryProvider, SmallProvider};
//!
//! let mut provider = SmallProvider::new();
//! let memory = provider.allocate(1024)?;
//! // Use memory...
//! provider.deallocate(memory)?;
//! ```

use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{Error, ErrorCategory, WrtResult, codes};

#[cfg(feature = "std")]
use std::vec::Vec;

/// Unified memory provider trait for all memory allocation strategies
///
/// This trait provides a consistent interface for memory allocation across
/// different platforms and configurations. All memory providers must implement
/// this trait to ensure compatibility.
///
/// # Safety Requirements
///
/// - `allocate` must return valid, properly aligned memory
/// - `deallocate` must only be called with memory previously returned by `allocate`
/// - Memory returned by `allocate` must remain valid until `deallocate` is called
/// - Providers must be thread-safe (`Send + Sync`)
pub trait UnifiedMemoryProvider: Send + Sync {
    /// Allocate a block of memory of the specified size
    ///
    /// # Arguments
    ///
    /// * `size` - Number of bytes to allocate
    ///
    /// # Returns
    ///
    /// Returns a mutable slice to the allocated memory, or an error if
    /// allocation fails.
    ///
    /// # Errors
    ///
    /// - `ErrorCategory::Capacity` if the provider cannot allocate the requested size
    /// - `ErrorCategory::Memory` if allocation fails for other reasons
    fn allocate(&mut self, size: usize) -> WrtResult<&mut [u8]>;

    /// Deallocate a previously allocated block of memory
    ///
    /// # Arguments
    ///
    /// * `ptr` - Mutable slice to the memory to deallocate
    ///
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - `ptr` was previously returned by a call to `allocate` on this provider
    /// - `ptr` has not been deallocated before
    /// - No references to the memory in `ptr` exist after this call
    ///
    /// # Errors
    ///
    /// - `ErrorCategory::Memory` if deallocation fails
    fn deallocate(&mut self, ptr: &mut [u8]) -> WrtResult<()>;

    /// Get the amount of available memory in bytes
    ///
    /// # Returns
    ///
    /// The number of bytes available for allocation. This may be approximate
    /// for some providers.
    fn available_memory(&self) -> usize;

    /// Get the total memory capacity in bytes
    ///
    /// # Returns
    ///
    /// The total number of bytes this provider can manage.
    fn total_memory(&self) -> usize;

    /// Get memory usage statistics
    ///
    /// # Returns
    ///
    /// A tuple of (allocated_bytes, peak_allocated_bytes)
    fn memory_stats(&self) -> (usize, usize) {
        let allocated = self.total_memory() - self.available_memory();
        (allocated, allocated) // Default implementation assumes current = peak
    }

    /// Check if the provider can allocate a specific size
    ///
    /// # Arguments
    ///
    /// * `size` - Number of bytes to check
    ///
    /// # Returns
    ///
    /// `true` if the provider can allocate the requested size, `false` otherwise.
    fn can_allocate(&self, size: usize) -> bool {
        size <= self.available_memory()
    }

    /// Get the alignment requirements for this provider
    ///
    /// # Returns
    ///
    /// The byte alignment required for allocations from this provider.
    /// Default is 8 bytes for most platforms.
    fn alignment(&self) -> usize {
        8 // Default to 8-byte alignment
    }
}

/// A configurable memory provider with fixed capacity
///
/// This provider manages a fixed-size buffer and allocates memory from it
/// using a simple bump allocator strategy. It's designed for predictable
/// memory usage in safety-critical environments.
///
/// # Type Parameters
///
/// * `SIZE` - The total size of the memory buffer in bytes
#[derive(Debug)]
pub struct ConfigurableProvider<const SIZE: usize> {
    /// The fixed-size memory buffer
    buffer: [u8; SIZE],
    /// Current allocation offset (bump pointer)
    allocated: AtomicUsize,
    /// Peak allocation (for statistics)
    peak_allocated: AtomicUsize,
}

impl<const SIZE: usize> ConfigurableProvider<SIZE> {
    /// Create a new configurable provider
    ///
    /// # Returns
    ///
    /// A new provider with zero-initialized memory buffer.
    pub const fn new() -> Self {
        Self {
            buffer: [0; SIZE],
            allocated: AtomicUsize::new(0),
            peak_allocated: AtomicUsize::new(0),
        }
    }

    /// Reset the provider, deallocating all memory
    ///
    /// This resets the bump pointer to the beginning of the buffer,
    /// effectively deallocating all previously allocated memory.
    ///
    /// # Safety
    ///
    /// The caller must ensure that no references to previously allocated
    /// memory exist after calling this method.
    pub fn reset(&mut self) {
        self.allocated.store(0, Ordering::Relaxed);
    }

    /// Get the buffer size at compile time
    pub const fn buffer_size() -> usize {
        SIZE
    }

    /// Check if the provider is empty (no allocations)
    pub fn is_empty(&self) -> bool {
        self.allocated.load(Ordering::Relaxed) == 0
    }

    /// Get current allocation offset
    pub fn current_offset(&self) -> usize {
        self.allocated.load(Ordering::Relaxed)
    }
}

impl<const SIZE: usize> Default for ConfigurableProvider<SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SIZE: usize> UnifiedMemoryProvider for ConfigurableProvider<SIZE> {
    fn allocate(&mut self, size: usize) -> WrtResult<&mut [u8]> {
        if size == 0 {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::INVALID_VALUE,
                "Cannot allocate zero bytes",
            ));
        }

        let current = self.allocated.load(Ordering::Relaxed);
        let aligned_size = (size + self.alignment() - 1) & !(self.alignment() - 1);
        let new_offset = current + aligned_size;

        if new_offset > SIZE {
            return Err(Error::new(
                ErrorCategory::Capacity,
                codes::CAPACITY_EXCEEDED,
                "Memory provider capacity exceeded",
            ));
        }

        // Update allocation pointer
        self.allocated.store(new_offset, Ordering::Relaxed);
        
        // Update peak allocation
        let peak = self.peak_allocated.load(Ordering::Relaxed);
        if new_offset > peak {
            self.peak_allocated.store(new_offset, Ordering::Relaxed);
        }

        // Safety: We've verified that the range is within bounds
        Ok(&mut self.buffer[current..current + size])
    }

    fn deallocate(&mut self, _ptr: &mut [u8]) -> WrtResult<()> {
        // Bump allocator doesn't support individual deallocation
        // This is a design limitation for simplicity and performance
        Ok(())
    }

    fn available_memory(&self) -> usize {
        SIZE - self.allocated.load(Ordering::Relaxed)
    }

    fn total_memory(&self) -> usize {
        SIZE
    }

    fn memory_stats(&self) -> (usize, usize) {
        let allocated = self.allocated.load(Ordering::Relaxed);
        let peak = self.peak_allocated.load(Ordering::Relaxed);
        (allocated, peak)
    }
}

/// Small memory provider with 8KB capacity
///
/// Suitable for small allocations like function parameters, temporary buffers,
/// and small data structures.
pub type SmallProvider = ConfigurableProvider<8192>;

/// Medium memory provider with 64KB capacity
///
/// Suitable for medium-sized allocations like instruction sequences, module
/// metadata, and component interfaces.
pub type MediumProvider = ConfigurableProvider<65536>;

/// Large memory provider with 1MB capacity
///
/// Suitable for large allocations like WebAssembly memory pages, large
/// data buffers, and component instantiation.
pub type LargeProvider = ConfigurableProvider<1048576>;

/// Integration wrapper for existing NoStdProvider
///
/// This wrapper makes the existing NoStdProvider compatible with the
/// unified memory provider interface.
#[derive(Debug)]
pub struct NoStdProviderWrapper<const SIZE: usize> {
    inner: crate::safe_memory::NoStdProvider<SIZE>,
}

impl<const SIZE: usize> NoStdProviderWrapper<SIZE> {
    /// Create a new NoStdProvider wrapper
    pub fn new() -> Self {
        Self {
            inner: crate::safe_memory::NoStdProvider::new(),
        }
    }

    /// Get access to the inner NoStdProvider
    pub fn inner(&self) -> &crate::safe_memory::NoStdProvider<SIZE> {
        &self.inner
    }

    /// Get mutable access to the inner NoStdProvider
    pub fn inner_mut(&mut self) -> &mut crate::safe_memory::NoStdProvider<SIZE> {
        &mut self.inner
    }
}

impl<const SIZE: usize> Default for NoStdProviderWrapper<SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SIZE: usize> UnifiedMemoryProvider for NoStdProviderWrapper<SIZE> {
    fn allocate(&mut self, size: usize) -> WrtResult<&mut [u8]> {
        // For now, return an error since the current NoStdProvider
        // doesn't directly support this interface
        // TODO: Implement proper integration with NoStdProvider
        Err(Error::new(
            ErrorCategory::Memory,
            codes::UNIMPLEMENTED,
            "NoStdProvider integration not yet implemented",
        ))
    }

    fn deallocate(&mut self, _ptr: &mut [u8]) -> WrtResult<()> {
        // TODO: Implement proper integration with NoStdProvider
        Ok(())
    }

    fn available_memory(&self) -> usize {
        // TODO: Get actual available memory from NoStdProvider
        SIZE
    }

    fn total_memory(&self) -> usize {
        SIZE
    }
}

/// Standard library memory provider that delegates to the system allocator
///
/// This provider uses the standard library's allocation facilities and
/// is only available when the `std` feature is enabled.
#[cfg(feature = "std")]
#[derive(Debug, Default)]
pub struct UnifiedStdProvider {
    /// Tracking of allocated memory blocks (address -> size)
    allocated_blocks: std::collections::HashMap<usize, usize>,
    /// Total bytes allocated
    total_allocated: AtomicUsize,
    /// Peak bytes allocated
    peak_allocated: AtomicUsize,
}

#[cfg(feature = "std")]
impl UnifiedStdProvider {
    /// Create a new standard library memory provider
    pub fn new() -> Self {
        Self {
            allocated_blocks: std::collections::HashMap::new(),
            total_allocated: AtomicUsize::new(0),
            peak_allocated: AtomicUsize::new(0),
        }
    }
}

#[cfg(feature = "std")]
impl UnifiedMemoryProvider for UnifiedStdProvider {
    fn allocate(&mut self, size: usize) -> WrtResult<&mut [u8]> {
        if size == 0 {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::INVALID_VALUE,
                "Cannot allocate zero bytes",
            ));
        }

        let layout = std::alloc::Layout::from_size_align(size, self.alignment())
            .map_err(|_| Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ALLOCATION_FAILED,
                "Invalid memory layout",
            ))?;

        // Allocate memory using the global allocator
        #[allow(unsafe_code)]
        let ptr = unsafe { std::alloc::alloc(layout) };
        if ptr.is_null() {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ALLOCATION_FAILED,
                "System memory allocation failed",
            ));
        }

        // Track the allocation (convert pointer to address)
        self.allocated_blocks.insert(ptr as usize, size);
        let new_total = self.total_allocated.fetch_add(size, Ordering::Relaxed) + size;
        
        // Update peak allocation
        let peak = self.peak_allocated.load(Ordering::Relaxed);
        if new_total > peak {
            self.peak_allocated.store(new_total, Ordering::Relaxed);
        }

        // Safety: We just allocated this memory and verified it's not null
        #[allow(unsafe_code)]
        Ok(unsafe { std::slice::from_raw_parts_mut(ptr, size) })
    }

    fn deallocate(&mut self, ptr: &mut [u8]) -> WrtResult<()> {
        let ptr_addr = ptr.as_mut_ptr();
        let size = self.allocated_blocks.remove(&(ptr_addr as usize))
            .ok_or_else(|| Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ACCESS_ERROR,
                "Attempt to deallocate untracked memory",
            ))?;

        if size != ptr.len() {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ACCESS_ERROR,
                "Memory block size mismatch",
            ));
        }

        let layout = std::alloc::Layout::from_size_align(size, self.alignment())
            .map_err(|_| Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ACCESS_ERROR,
                "Invalid memory layout for deallocation",
            ))?;

        // Deallocate memory using the global allocator
        #[allow(unsafe_code)]
        unsafe { std::alloc::dealloc(ptr_addr, layout) };

        // Update allocation tracking
        self.total_allocated.fetch_sub(size, Ordering::Relaxed);

        Ok(())
    }

    fn available_memory(&self) -> usize {
        // For std provider, we assume unlimited memory (subject to system limits)
        usize::MAX
    }

    fn total_memory(&self) -> usize {
        // For std provider, we assume unlimited memory
        usize::MAX
    }

    fn memory_stats(&self) -> (usize, usize) {
        let allocated = self.total_allocated.load(Ordering::Relaxed);
        let peak = self.peak_allocated.load(Ordering::Relaxed);
        (allocated, peak)
    }

    fn can_allocate(&self, _size: usize) -> bool {
        // For std provider, we assume we can always allocate (subject to system limits)
        true
    }
}

/// Memory provider factory for creating providers based on configuration
pub struct MemoryProviderFactory;

impl MemoryProviderFactory {
    /// Create a small memory provider
    pub fn create_small() -> SmallProvider {
        SmallProvider::new()
    }

    /// Create a medium memory provider
    pub fn create_medium() -> MediumProvider {
        MediumProvider::new()
    }

    /// Create a large memory provider
    pub fn create_large() -> LargeProvider {
        LargeProvider::new()
    }

    /// Create a provider with custom size
    pub fn create_custom<const SIZE: usize>() -> ConfigurableProvider<SIZE> {
        ConfigurableProvider::<SIZE>::new()
    }

    /// Create a std provider (only available with std feature)
    #[cfg(feature = "std")]
    pub fn create_std() -> UnifiedStdProvider {
        UnifiedStdProvider::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configurable_provider_basic() {
        let mut provider = SmallProvider::new();
        
        assert_eq!(provider.total_memory(), 8192);
        assert_eq!(provider.available_memory(), 8192);
        assert!(provider.is_empty());

        let memory = provider.allocate(1024).unwrap();
        assert_eq!(memory.len(), 1024);
        assert_eq!(provider.available_memory(), 8192 - 1024 - (8 - (1024 % 8)) % 8); // Account for alignment
        assert!(!provider.is_empty());

        provider.deallocate(memory).unwrap(); // Should be no-op for bump allocator
    }

    #[test]
    fn test_configurable_provider_capacity_exceeded() {
        let mut provider = SmallProvider::new();
        
        // Try to allocate more than capacity
        let result = provider.allocate(10000);
        assert!(result.is_err());
        
        if let Err(err) = result {
            assert_eq!(err.category, ErrorCategory::Capacity);
        }
    }

    #[test]
    fn test_configurable_provider_zero_allocation() {
        let mut provider = SmallProvider::new();
        
        let result = provider.allocate(0);
        assert!(result.is_err());
        
        if let Err(err) = result {
            assert_eq!(err.category, ErrorCategory::Memory);
        }
    }

    #[test]
    fn test_memory_stats() {
        let mut provider = MediumProvider::new();
        
        let (allocated, peak) = provider.memory_stats();
        assert_eq!(allocated, 0);
        assert_eq!(peak, 0);

        let _memory1 = provider.allocate(1000).unwrap();
        let (allocated, peak) = provider.memory_stats();
        assert!(allocated >= 1000); // May be larger due to alignment
        assert!(peak >= 1000);

        let _memory2 = provider.allocate(2000).unwrap();
        let (allocated, peak) = provider.memory_stats();
        assert!(allocated >= 3000);
        assert!(peak >= 3000);
    }

    #[test]
    fn test_provider_reset() {
        let mut provider = SmallProvider::new();
        
        let _memory = provider.allocate(1000).unwrap();
        assert!(!provider.is_empty());
        
        provider.reset();
        assert!(provider.is_empty());
        assert_eq!(provider.available_memory(), provider.total_memory());
    }

    #[test]
    fn test_memory_provider_factory() {
        let small = MemoryProviderFactory::create_small();
        assert_eq!(small.total_memory(), 8192);

        let medium = MemoryProviderFactory::create_medium();
        assert_eq!(medium.total_memory(), 65536);

        let large = MemoryProviderFactory::create_large();
        assert_eq!(large.total_memory(), 1048576);

        let custom = MemoryProviderFactory::create_custom::<4096>();
        assert_eq!(custom.total_memory(), 4096);
    }

    #[test]
    fn test_alignment() {
        let provider = SmallProvider::new();
        assert_eq!(provider.alignment(), 8);
    }

    #[test]
    fn test_can_allocate() {
        let provider = SmallProvider::new();
        assert!(provider.can_allocate(1000));
        assert!(provider.can_allocate(8192));
        assert!(!provider.can_allocate(10000));
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_std_provider() {
        let mut provider = UnifiedStdProvider::new();
        
        assert_eq!(provider.total_memory(), usize::MAX);
        assert_eq!(provider.available_memory(), usize::MAX);
        assert!(provider.can_allocate(1000));

        let memory = provider.allocate(1024).unwrap();
        assert_eq!(memory.len(), 1024);

        let (allocated, peak) = provider.memory_stats();
        assert_eq!(allocated, 1024);
        assert_eq!(peak, 1024);

        provider.deallocate(memory).unwrap();
        
        let (allocated, _) = provider.memory_stats();
        assert_eq!(allocated, 0);
    }
}