// WRT - wrt-foundation
// Module: Linear Memory using Platform Abstraction Layer
// SW-REQ-ID: REQ_MEMORY_001 REQ_PLATFORM_001
//
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

#![allow(unsafe_code)] // This module needs unsafe for memory operations

//! Provides a Wasm linear memory implementation backed by a `PageAllocator`
//! from the `wrt-platform` crate.

// Test comment to try and force recompile / cache clear
use core::{
    fmt::Debug,
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};

use wrt_platform::memory::{PageAllocator, WASM_PAGE_SIZE};

use crate::{
    prelude::*,
    safe_memory::{Provider, Slice, SliceMut, Stats, VerificationLevel},
};

/// A WebAssembly linear memory implementation using a `PageAllocator`.
///
/// This struct manages a region of memory allocated and potentially grown by
/// a platform-specific `PageAllocator`.
#[derive(Debug)]
pub struct PalMemoryProvider<A: PageAllocator + Send + Sync> {
    allocator: A,
    base_ptr: Option<NonNull<u8>>,
    current_pages: u32,
    maximum_pages: Option<u32>,
    initial_allocation_size: usize, // Size returned by the initial allocate call
    verification_level: VerificationLevel,
    // For Provider trait stats, if not derived from allocator directly
    access_count: AtomicUsize,
    max_access_size: AtomicUsize,
}

// SAFETY: The PalMemoryProvider is Send if the PageAllocator A is Send.
// The NonNull<u8> itself is not Send/Sync, but we are managing its lifecycle
// and access. Thread-safety depends on the allocator and how this provider's
// methods are used externally (e.g., if &mut self methods are correctly
// serialized). The raw pointer is only ever accessed through methods that take
// &self or &mut self, and the underlying memory operations via the
// PageAllocator are assumed to be safe or synchronized if A is Send + Sync.
unsafe impl<A: PageAllocator + Send + Sync> Send for PalMemoryProvider<A> {}

// SAFETY: Similar to Send, PalMemoryProvider is Sync if A is Sync.
// Accesses to shared state like AtomicUsize are atomic.
// Accesses to the memory region via &self methods (like borrow_slice) provide
// immutable slices, which is safe. Mutable access is through &mut self.
unsafe impl<A: PageAllocator + Send + Sync> Sync for PalMemoryProvider<A> {}

impl<A: PageAllocator + Send + Sync> PalMemoryProvider<A> {
    /// Creates a new `PalMemoryProvider`.
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
    /// Returns an `Error` if the initial allocation fails.
    pub fn new(
        mut allocator: A,
        initial_pages: u32,
        maximum_pages: Option<u32>,
        verification_level: VerificationLevel,
    ) -> Result<Self> {
        if initial_pages == 0 && maximum_pages.unwrap_or(0) == 0 {
            // Allow zero initial if max is also zero, effectively an empty
            // non-growable memory. Or if allocator can handle
            // initial_pages = 0. For now, let's assume
            // allocator.allocate handles initial_pages = 0 if needed.
            // Wasm spec: min size is required, max is optional.
            // If initial_pages is 0, it will likely allocate 0 bytes as per
            // spec.
        }

        let (ptr, allocated_size) = allocator.allocate(initial_pages, maximum_pages)?;

        Ok(Self {
            allocator,
            base_ptr: Some(ptr),
            current_pages: initial_pages,
            maximum_pages,
            initial_allocation_size: allocated_size, // Store the size from allocate
            verification_level,
            access_count: AtomicUsize::new(0),
            max_access_size: AtomicUsize::new(0),
        })
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
        if additional_pages == 0 {
            return Ok(self.current_pages);
        }
        let Some(_base_ptr) = self.base_ptr else {
            return Err(Error::new(
                ErrorCategory::Core,
                codes::INITIALIZATION_ERROR,
                "Memory not allocated, cannot grow.",
            ));
        };

        let old_pages = self.current_pages;
        let new_total_pages = old_pages.checked_add(additional_pages).ok_or_else(|| {
            Error::new(
                ErrorCategory::Memory,
                codes::CAPACITY_EXCEEDED,
                "Page count overflow during grow attempt.",
            )
        })?;

        if let Some(max) = self.maximum_pages {
            if new_total_pages > max {
                return Err(Error::new(
                    ErrorCategory::Memory,
                    codes::CAPACITY_EXCEEDED,
                    "Grow attempt exceeds maximum pages.",
                ));
            }
        }

        self.allocator.grow(old_pages, additional_pages)?;
        self.current_pages = new_total_pages;
        Ok(old_pages)
    }

    fn track_access(&self, _offset: usize, len: usize) {
        self.access_count.fetch_add(1, Ordering::Relaxed);
        self.max_access_size.fetch_max(len, Ordering::Relaxed);
        // More sophisticated tracking (like unique regions) could be added if
        // needed.
    }

    /// Returns the current number of WebAssembly pages allocated.
    pub fn pages(&self) -> u32 {
        self.current_pages
    }

    /// Returns the maximum number of WebAssembly pages this memory can grow to.
    pub fn max_pages(&self) -> Option<u32> {
        self.maximum_pages
    }
}

impl<A: PageAllocator + Send + Sync> Provider for PalMemoryProvider<A> {
    fn borrow_slice(&self, offset: usize, len: usize) -> Result<Slice<'_>> {
        self.verify_access(offset, len)?;
        let Some(base_ptr) = self.base_ptr else {
            return Err(Error::new(
                ErrorCategory::Core,
                codes::INITIALIZATION_ERROR,
                "Memory not allocated, cannot borrow slice.",
            ));
        };
        self.track_access(offset, len);
        // SAFETY: `verify_access` ensures that `offset + len` is within the
        // currently allocated and accessible memory bounds (current_pages *
        // WASM_PAGE_SIZE). `base_ptr` is guaranteed to be non-null and valid if
        // Some by the module's invariants (it's set on successful allocation
        // and cleared on deallocation). The lifetime of the returned slice is
        // tied to `&self`, ensuring the data remains valid as long as the
        // `PalMemoryProvider` is borrowed. The underlying memory pointed to by
        // `base_ptr.as_ptr().add(offset)` is valid for reads of `len` bytes
        // because `verify_access` has passed.
        let data_slice = unsafe { core::slice::from_raw_parts(base_ptr.as_ptr().add(offset), len) };
        Slice::with_verification_level(data_slice, self.verification_level)
    }

    fn write_data(&mut self, offset: usize, data: &[u8]) -> Result<()> {
        self.verify_access(offset, data.len())?;
        let Some(base_ptr) = self.base_ptr else {
            return Err(Error::new(
                ErrorCategory::Core,
                codes::INITIALIZATION_ERROR,
                "Memory not allocated, cannot write data.",
            ));
        };
        self.track_access(offset, data.len());
        // SAFETY: `verify_access` ensures that `offset + data.len()` is within the
        // currently allocated and accessible memory bounds.
        // `base_ptr` is guaranteed to be non-null and valid if Some.
        // The method takes `&mut self`, ensuring exclusive access to the
        // `PalMemoryProvider`, and thus to the underlying memory region for the
        // duration of this call. This prevents data races.
        // The memory at `base_ptr.as_ptr().add(offset)` is valid for writes of
        // `data.len()` bytes because `verify_access` has passed.
        let dest_slice =
            unsafe { core::slice::from_raw_parts_mut(base_ptr.as_ptr().add(offset), data.len()) };
        dest_slice.copy_from_slice(data);
        Ok(())
    }

    fn verify_access(&self, offset: usize, len: usize) -> Result<()> {
        let current_byte_size = self.current_pages as usize * WASM_PAGE_SIZE;
        let end_offset = offset.checked_add(len).ok_or_else(|| {
            Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ACCESS_OUT_OF_BOUNDS, // Or overflow error code
                "Access range calculation overflow.",
            )
        })?;

        if end_offset > current_byte_size {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                "Access out of bounds.",
            ));
        }
        Ok(())
    }

    fn size(&self) -> usize {
        self.current_pages as usize * WASM_PAGE_SIZE
    }

    fn capacity(&self) -> usize {
        self.maximum_pages.map_or_else(
            || self.size(), // If no max, capacity is current size (or could be allocator defined)
            |max_pages| max_pages as usize * WASM_PAGE_SIZE,
        )
    }

    fn verify_integrity(&self) -> Result<()> {
        // Integrity for this provider primarily means the allocator itself is sound
        // and our view (pages, ptr) is consistent. Deeper integrity (checksums)
        // is handled by Slice/SliceMut.
        if self.base_ptr.is_none() && self.current_pages > 0 {
            return Err(Error::new(
                ErrorCategory::Core,
                codes::INTEGRITY_VIOLATION,
                "Memory pointer is None but current_pages > 0",
            ));
        }
        // Further checks could involve querying the allocator if it exposes health
        // checks.
        Ok(())
    }

    fn set_verification_level(&mut self, level: VerificationLevel) {
        self.verification_level = level;
    }

    fn verification_level(&self) -> VerificationLevel {
        self.verification_level
    }

    fn memory_stats(&self) -> Stats {
        Stats {
            total_size: self.size(),
            access_count: self.access_count.load(Ordering::Relaxed),
            unique_regions: 0, // Not tracked by this basic provider yet
            max_access_size: self.max_access_size.load(Ordering::Relaxed),
        }
    }

    fn get_slice_mut(&mut self, offset: usize, len: usize) -> Result<SliceMut<'_>> {
        self.verify_access(offset, len)?;
        let Some(base_ptr) = self.base_ptr else {
            return Err(Error::new(
                ErrorCategory::Core,
                codes::INITIALIZATION_ERROR,
                "Memory not allocated, cannot get mutable slice.",
            ));
        };
        self.track_access(offset, len);
        // SAFETY: `verify_access` ensures that `offset + len` is within the
        // currently allocated and accessible memory bounds. `base_ptr` is
        // non-null and valid. `&mut self` ensures exclusive access. The
        // memory region is valid for mutable access.
        let data_slice =
            unsafe { core::slice::from_raw_parts_mut(base_ptr.as_ptr().add(offset), len) };
        SliceMut::with_verification_level(data_slice, self.verification_level)
    }

    fn copy_within(&mut self, src_offset: usize, dst_offset: usize, len: usize) -> Result<()> {
        if len == 0 {
            return Ok(());
        }

        // Verify source and destination ranges independently first.
        self.verify_access(src_offset, len).map_err(|e| {
            Error::new_with_cause(
                ErrorCategory::Memory,
                codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                "Source range out of bounds for copy_within.",
                e,
            )
        })?;
        self.verify_access(dst_offset, len).map_err(|e| {
            Error::new_with_cause(
                ErrorCategory::Memory,
                codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                "Destination range out of bounds for copy_within.",
                e,
            )
        })?;

        let Some(base_ptr) = self.base_ptr else {
            return Err(Error::new(
                ErrorCategory::Core,
                codes::INITIALIZATION_ERROR,
                "Memory not allocated, cannot copy_within.",
            ));
        };

        // Track access before performing the copy
        // We could track two accesses (read from src, write to dst) or one logical
        // copy operation.
        self.track_access(src_offset, len); // Track the read part
        self.track_access(dst_offset, len); // Track the write part

        // SAFETY: `verify_access` for both source and destination ensures that
        // `src_offset + len` and `dst_offset + len` are within bounds.
        // `base_ptr` is non-null and valid. `&mut self` ensures exclusive access
        // suitable for `copy_within` which might have overlapping regions.
        // `as_ptr()` is safe as `base_ptr` is `NonNull`.
        // `add(offset)` is pointer arithmetic, safe here because bounds are checked.
        // `copy_to_nonoverlapping` or `copy` needs to be chosen based on overlap.
        // `core::ptr::copy` handles overlapping regions correctly.
        unsafe {
            let src_ptr = base_ptr.as_ptr().add(src_offset);
            let dst_ptr = base_ptr.as_ptr().add(dst_offset);
            core::ptr::copy(src_ptr, dst_ptr, len);
        }
        Ok(())
    }

    fn ensure_used_up_to(&mut self, byte_offset: usize) -> Result<()> {
        let current_size_bytes = self.size();
        if byte_offset > current_size_bytes {
            if byte_offset > self.capacity() {
                return Err(Error::new(
                    ErrorCategory::Memory,
                    codes::CAPACITY_EXCEEDED,
                    "ensure_used_up_to exceeds capacity",
                ));
            }
            // Calculate additional pages needed. Ceiling division.
            let additional_bytes_needed = byte_offset - current_size_bytes;
            let additional_pages_needed =
                (additional_bytes_needed + WASM_PAGE_SIZE - 1) / WASM_PAGE_SIZE;

            self.grow(additional_pages_needed as u32)?;
            // grow updates current_pages, size() will reflect the new size.
            // We need to ensure the new size is at least byte_offset.
            if self.size() < byte_offset {
                // This case should ideally not be hit if grow succeeded and calculations are
                // correct. It might happen if grow couldn't satisfy the full
                // request but didn't error, or if there are rounding issues.
                return Err(Error::new(
                    ErrorCategory::Memory,
                    codes::INVALID_STATE,
                    "Memory growth did not reach requested byte_offset",
                ));
            }
        }
        Ok(())
    }
}

impl<A: PageAllocator + Send + Sync> Drop for PalMemoryProvider<A> {
    fn drop(&mut self) {
        if let Some(ptr) = self.base_ptr.take() {
            // The `initial_allocation_size` stores the size returned by the
            // `PageAllocator::allocate` call. This is the size that should be
            // passed to `PageAllocator::deallocate`.
            let size_to_deallocate = self.initial_allocation_size;

            if size_to_deallocate > 0 {
                // SAFETY: `ptr` was obtained from `self.allocator.allocate` and is
                // valid. `size_to_deallocate` is the size of the region allocated
                // by the allocator. This deallocation is performed when
                // `PalMemoryProvider` goes out of scope, ensuring exclusive access
                // for deallocation.
                unsafe {
                    if let Err(_e) = self.allocator.deallocate(ptr, size_to_deallocate) {
                        // In a no_std environment, error reporting in drop is
                        // complex. Panicking in drop is
                        // highly discouraged.
                        // Logging might be done via a specific facade if
                        // available. For now, we
                        // silently ignore deallocation errors here.
                        // The error `_e` could potentially be logged if a
                        // mechanism exists.
                    }
                }
            }
        }
    }
}

// Conditionally compile tests module only when std is enabled
#[cfg(all(test, feature = "std"))]
mod tests {
    // ... existing code ...
}
