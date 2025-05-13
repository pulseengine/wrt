// WRT - wrt-platform
// Module: Platform Memory Management Abstraction
// SW-REQ-ID: REQ_PLATFORM_001, REQ_MEMORY_001
//
// Copyright (c) 2025 The WRT Project Developers
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Provides traits and implementations for platform-specific memory management.

use core::{alloc::Layout, fmt::Debug, ptr::NonNull};

use wrt_error::{codes, Error, ErrorCategory, Result};

/// Represents a single WebAssembly page (64 KiB).
pub const WASM_PAGE_SIZE: usize = 65536; // 64 * 1024

/// Trait for platform-specific memory allocation.
///
/// Implementors handle the allocation, growth, and protection of memory
/// regions suitable for WebAssembly linear memory.
///
/// # Safety
/// Implementations involving `unsafe` (e.g., interacting with OS memory APIs)
/// must ensure they uphold memory safety guarantees, properly handle alignment,
/// and manage memory lifetimes correctly.
// Note: #![forbid(unsafe_code)] applies to the crate, specific unsafe blocks
// would need justification and potentially move to a dedicated `hal` submodule
// if complex.
pub trait PageAllocator: Debug + Send + Sync {
    /// Allocate a region of memory capable of holding `initial_pages`.
    ///
    /// The allocated memory should be suitable for read/write access.
    /// Implementations may reserve address space beyond `initial_pages` up to
    /// `maximum_pages` if applicable.
    ///
    /// # Arguments
    ///
    /// * `initial_pages`: The number of Wasm pages (64 KiB) to make initially
    ///   accessible.
    /// * `maximum_pages`: An optional hint for the maximum number of pages the
    ///   memory might grow to.
    ///
    /// # Returns
    ///
    /// A `Result` containing a pointer to the start of the allocated memory
    /// region and the total committed memory size in bytes, or an `Error`
    /// on failure.
    fn allocate(
        &mut self,
        initial_pages: u32,
        maximum_pages: Option<u32>,
    ) -> Result<(NonNull<u8>, usize)>;

    /// Grow the allocated memory region by `additional_pages`.
    ///
    /// Ensures that the memory region managed by this allocator is at least
    /// `current_pages + additional_pages` in size.
    ///
    /// # Arguments
    ///
    /// * `current_pages`: The current size of the memory region in Wasm pages.
    /// * `additional_pages`: The number of Wasm pages to add.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an `Error` if the memory cannot be grown (e.g.,
    /// exceeds maximum limits or allocation fails).
    fn grow(&mut self, current_pages: u32, additional_pages: u32) -> Result<()>;

    /// Deallocate the memory region previously allocated by `allocate`.
    ///
    /// # Safety
    /// The caller must ensure that the `ptr` and `size` correspond exactly to a
    /// previously successful allocation from *this* allocator instance, and
    /// that no references to the memory region exist after this call.
    unsafe fn deallocate(&mut self, ptr: NonNull<u8>, size: usize) -> Result<()>;

    // Add methods for memory protection (e.g., MTE, guard pages) later if needed.
    // fn protect(...) -> Result<(), Error>;
}

// --- Fallback Allocator (std) ---

#[cfg(feature = "std")]
mod fallback_std {
    use alloc::{boxed::Box, vec::Vec};

    use super::*;

    /// Fallback page allocator using `std::vec::Vec`.
    /// This allocator simulates page-based allocation on the heap.
    #[derive(Debug)]
    pub struct FallbackAllocator {
        memory: Option<Vec<u8>>,
        capacity_pages: u32, // Maximum pages this instance can grow to
    }

    impl Default for FallbackAllocator {
        fn default() -> Self {
            Self {
                memory: None,
                capacity_pages: u32::MAX, // Default to a large capacity
            }
        }
    }

    impl FallbackAllocator {
        /// Creates a new fallback allocator.
        /// Optionally, a maximum capacity in pages can be set.
        pub fn new(maximum_pages: Option<u32>) -> Self {
            Self {
                memory: None,
                capacity_pages: maximum_pages.unwrap_or(u32::MAX / (WASM_PAGE_SIZE as u32)), /* Default to a practical limit if None */
            }
        }
    }

    impl PageAllocator for FallbackAllocator {
        fn allocate(
            &mut self,
            initial_pages: u32,
            maximum_pages: Option<u32>,
        ) -> Result<(NonNull<u8>, usize)> {
            if self.memory.is_some() {
                return Err(Error::new(
                    ErrorCategory::System,
                    codes::INITIALIZATION_ERROR, // Or a new platform specific code
                    "Allocator has already allocated memory",
                ));
            }

            if initial_pages == 0 {
                return Err(Error::new(
                    ErrorCategory::Memory,
                    codes::MEMORY_GROW_ERROR, // Or INVALID_ARGUMENT if we had one
                    "Initial pages cannot be zero",
                ));
            }

            let capacity_pages = maximum_pages.unwrap_or(initial_pages).max(initial_pages);
            if capacity_pages > self.capacity_pages {
                return Err(Error::new(
                    ErrorCategory::Memory,
                    codes::CAPACITY_EXCEEDED,
                    "Requested maximum pages exceeds allocator capacity",
                ));
            }
            self.capacity_pages = capacity_pages;

            let initial_bytes = initial_pages as usize * WASM_PAGE_SIZE;

            let mut vec = Vec::new();
            if vec.try_reserve_exact(initial_bytes).is_err() {
                return Err(Error::memory_error("Failed to reserve initial memory for Vec"));
            }
            // Initialize with zeros, similar to how Wasm memory is expected
            vec.resize(initial_bytes, 0);

            let ptr = NonNull::new(vec.as_mut_ptr()).ok_or_else(|| {
                Error::new(
                    ErrorCategory::System,
                    codes::MEMORY_ACCESS_ERROR, // Or a more specific "allocation returned null"
                    "Vec allocation returned null pointer, this should not happen",
                )
            })?;

            self.memory = Some(vec);
            Ok((ptr, initial_bytes))
        }

        fn grow(&mut self, current_pages: u32, additional_pages: u32) -> Result<()> {
            let vec = self.memory.as_mut().ok_or_else(|| {
                Error::new(
                    ErrorCategory::System,
                    codes::MEMORY_ACCESS_ERROR, // Or "not_allocated"
                    "No memory allocated to grow",
                )
            })?;

            if additional_pages == 0 {
                return Ok(()); // No growth requested
            }

            let current_bytes = current_pages as usize * WASM_PAGE_SIZE;
            if vec.len() != current_bytes {
                return Err(Error::new(
                    ErrorCategory::System,
                    codes::MEMORY_ACCESS_ERROR, // Or "state_mismatch"
                    "Current size mismatch during grow operation",
                ));
            }

            let new_total_pages = current_pages
                .checked_add(additional_pages)
                .ok_or_else(|| Error::memory_error("Page count overflow during grow"))?;

            if new_total_pages > self.capacity_pages {
                return Err(Error::memory_error(
                    "Grow operation exceeds maximum configured capacity",
                ));
            }

            let new_total_bytes = new_total_pages as usize * WASM_PAGE_SIZE;

            if vec.try_reserve_exact(new_total_bytes - current_bytes).is_err() {
                return Err(Error::memory_error("Failed to reserve memory for growth"));
            }
            // Initialize new memory with zeros
            vec.resize(new_total_bytes, 0);

            // Pointer might change if Vec reallocated, but PageAllocator API here
            // doesn't return the new pointer for grow, it assumes base stays same.
            // This is a limitation of this simple Vec based fallback.
            // Real mmap-based allocators would keep the base pointer.

            Ok(())
        }

        unsafe fn deallocate(&mut self, ptr: NonNull<u8>, size: usize) -> Result<()> {
            match self.memory.take() {
                // take() consumes self.memory, setting it to None
                Some(vec_mem) => {
                    // Basic check: does the provided ptr match the start of our vec?
                    // And does the size match?
                    // This is not foolproof as the caller *could* pass a ptr into the middle
                    // but it's a basic sanity check.
                    if vec_mem.as_ptr() == ptr.as_ptr() && vec_mem.len() == size {
                        // The Vec is dropped when it goes out of scope here, deallocating the
                        // memory.
                        Ok(())
                    } else {
                        // Put the memory back if it wasn't ours, to prevent losing it
                        // if this deallocate call was erroneous but there's still a valid
                        // allocation. However, if it's a partial match,
                        // it's already UB by caller. For simplicity here,
                        // if it's a mismatch, we assume it's an error.
                        self.memory = Some(vec_mem); // Put it back if checks fail. This is tricky.
                                                     // A better approach if mismatch: error, but memory is now leaked by take().
                                                     // Or, don't take() until sure.
                        Err(Error::new(
                            ErrorCategory::Memory,
                            codes::MEMORY_ACCESS_ERROR, // Or "deallocation_mismatch"
                            "Deallocation ptr or size does not match stored values",
                        ))
                    }
                }
                None => Err(Error::new(
                    ErrorCategory::System,
                    codes::MEMORY_ACCESS_ERROR, // Or "deallocate_not_allocated"
                    "Deallocate attempted on allocator with no active allocation",
                )),
            }
        }
    }
}

#[cfg(feature = "std")]
pub use fallback_std::FallbackAllocator;
