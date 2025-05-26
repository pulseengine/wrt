// WRT - wrt-foundation
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Provides the `LinearMemory` struct, representing an instance of a
//! WebAssembly linear memory, backed by a `PalMemoryProvider`.

use core::fmt::Debug;

use wrt_platform::memory::PageAllocator;

use crate::{
    linear_memory::PalMemoryProvider,
    prelude::*,
    safe_memory::{Provider, Slice, SliceMut, Stats},
    verification::VerificationLevel,
};

/// Represents an instance of a WebAssembly linear memory.
///
/// This struct encapsulates a `PalMemoryProvider` to manage the memory's
/// lifecycle and provide safe access to its contents. It is generic over
/// a `PageAllocator` allowing different backing strategies for memory
/// allocation.
#[derive(Debug)]
pub struct LinearMemory<A: PageAllocator + Send + Sync + Clone + 'static> {
    provider: PalMemoryProvider<A>,
}

impl<A: PageAllocator + Send + Sync + Clone + 'static> LinearMemory<A> {
    /// Creates a new `LinearMemory`.
    ///
    /// # Arguments
    ///
    /// * `allocator`: The `PageAllocator` instance to use for memory
    ///   operations.
    /// * `initial_pages`: The initial number of Wasm pages to allocate.
    /// * `maximum_pages`: An optional maximum number of Wasm pages the memory
    ///   can grow to.
    /// * `verification_level`: The verification level for memory operations.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if the initial allocation via the provider fails.
    pub fn new(
        allocator: A,
        initial_pages: u32,
        maximum_pages: Option<u32>,
        verification_level: VerificationLevel,
    ) -> Result<Self> {
        let provider =
            PalMemoryProvider::new(allocator, initial_pages, maximum_pages, verification_level)?;
        Ok(Self { provider })
    }

    /// Grows the memory by `additional_pages`.
    ///
    /// Returns the previous number of pages on success, as per Wasm
    /// `memory.grow`.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if growing fails (e.g., exceeds maximum, allocator
    /// error).
    pub fn grow(&mut self, additional_pages: u32) -> Result<u32> {
        self.provider.grow(additional_pages)
    }

    /// Returns the current size of the memory in bytes.
    pub fn size(&self) -> usize {
        self.provider.size()
    }

    /// Returns the current capacity of the memory in bytes.
    pub fn capacity(&self) -> usize {
        self.provider.capacity()
    }

    /// Borrows an immutable slice of memory.
    ///
    /// This method delegates to the underlying memory provider, which ensures
    /// appropriate bounds and integrity checks are performed.
    ///
    /// # Arguments
    ///
    /// * `offset`: The byte offset from the start of the memory.
    /// * `len`: The length of the slice in bytes.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if the requested slice is out of bounds or if
    /// integrity checks fail within the provider.
    pub fn borrow_slice(&self, offset: usize, len: usize) -> Result<Slice<'_>> {
        self.provider.borrow_slice(offset, len)
    }

    /// Borrows a mutable slice of memory.
    ///
    /// This method delegates to the underlying memory provider, which ensures
    /// appropriate bounds and integrity checks are performed.
    ///
    /// # Arguments
    ///
    /// * `offset`: The byte offset from the start of the memory.
    /// * `len`: The length of the slice in bytes.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if the requested slice is out of bounds or if
    /// integrity checks fail within the provider.
    pub fn get_slice_mut(&mut self, offset: usize, len: usize) -> Result<SliceMut<'_>> {
        self.provider.get_slice_mut(offset, len)
    }

    /// Writes data to the memory at a given offset.
    ///
    /// This method delegates to the underlying memory provider, which ensures
    /// appropriate bounds and integrity checks are performed.
    ///
    /// # Arguments
    ///
    /// * `offset`: The byte offset from the start of the memory.
    /// * `data`: The byte slice to write.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if the write operation would go out of bounds or
    /// if integrity checks fail within the provider.
    pub fn write_data(&mut self, offset: usize, data: &[u8]) -> Result<()> {
        self.provider.write_data(offset, data)
    }

    /// Verifies that an access to memory (read or write) of `len` at `offset`
    /// would be valid.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if the described access would be invalid.
    pub fn verify_access(&self, offset: usize, len: usize) -> Result<()> {
        self.provider.verify_access(offset, len)
    }

    /// Verifies the overall integrity of the memory.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if an integrity violation is detected.
    pub fn verify_integrity(&self) -> Result<()> {
        self.provider.verify_integrity()
    }

    /// Sets the verification level for memory operations.
    pub fn set_verification_level(&mut self, level: VerificationLevel) {
        self.provider.set_verification_level(level);
    }

    /// Gets the current verification level of this memory.
    pub fn verification_level(&self) -> VerificationLevel {
        self.provider.verification_level()
    }

    /// Gets statistics about memory usage.
    pub fn memory_stats(&self) -> Stats {
        self.provider.memory_stats()
    }

    /// Copies data within the memory from a source offset to a destination
    /// offset.
    ///
    /// This method delegates to the underlying memory provider, which ensures
    /// appropriate bounds and integrity checks are performed.
    ///
    /// # Errors
    ///
    /// Returns an error if the source or destination ranges are invalid or out
    /// of bounds, or if the copy operation fails due to internal provider
    /// issues.
    pub fn copy_within(&mut self, src_offset: usize, dst_offset: usize, len: usize) -> Result<()> {
        self.provider.copy_within(src_offset, dst_offset, len)
    }
}

// The `LinearMemory` itself does not need to implement `
