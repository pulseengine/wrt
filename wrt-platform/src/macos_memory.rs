#![allow(unsafe_code)]
// Allow unsafe FFI calls for mmap/munmap
// WRT - wrt-platform
// Module: macOS Memory Management
// SW-REQ-ID: REQ_PLATFORM_001, REQ_MEMORY_001
//
// Copyright (c) 2025 The WRT Project Developers
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! macOS-specific `PageAllocator` implementation using `mmap`.

use core::ptr::{self, NonNull};

use libc::{mmap, munmap, MAP_ANON, MAP_FAILED, MAP_PRIVATE, PROT_READ, PROT_WRITE};
use wrt_error::{codes, Error, ErrorCategory, Result};

use crate::memory::{PageAllocator, WASM_PAGE_SIZE};

/// A `PageAllocator` implementation for macOS using `mmap` and `munmap`.
#[derive(Debug)]
pub struct MacOsAllocator {
    base_ptr: Option<NonNull<u8>>,
    total_reserved_bytes: usize, /* Total bytes reserved by mmap (may include uncommitted guard
                                  * regions) */
    current_committed_bytes: usize, // Bytes currently committed with PROT_READ | PROT_WRITE
    max_capacity_bytes: usize,      // Maximum bytes this instance can manage
}

impl MacOsAllocator {
    const DEFAULT_MAX_PAGES: u32 = 65536; // Corresponds to 4GiB, a common Wasm limit

    /// Creates a new `MacOsAllocator`.
    pub fn new(maximum_pages: Option<u32>) -> Self {
        let max_pages_val = maximum_pages.unwrap_or(Self::DEFAULT_MAX_PAGES);
        let max_capacity_bytes = max_pages_val as usize * WASM_PAGE_SIZE;
        Self {
            base_ptr: None,
            total_reserved_bytes: 0,
            current_committed_bytes: 0,
            max_capacity_bytes,
        }
    }

    fn pages_to_bytes(pages: u32) -> Result<usize> {
        pages.checked_mul(WASM_PAGE_SIZE as u32).map(|b| b as usize).ok_or_else(|| {
            Error::new(
                ErrorCategory::Memory,
                codes::CAPACITY_EXCEEDED, // Using this, could be INVALID_ARGUMENT
                "Page count results in byte overflow",
            )
        })
    }
}

/// Builder for `MacOsAllocator` to provide a fluent configuration API.
#[derive(Debug)]
pub struct MacOsAllocatorBuilder {
    maximum_pages: Option<u32>,
    guard_pages: bool,
    memory_tagging: bool,
}

impl Default for MacOsAllocatorBuilder {
    fn default() -> Self {
        Self { maximum_pages: None, guard_pages: false, memory_tagging: false }
    }
}

impl MacOsAllocatorBuilder {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum number of WebAssembly pages (64 KiB) that can be
    /// allocated.
    pub fn with_maximum_pages(mut self, pages: u32) -> Self {
        self.maximum_pages = Some(pages);
        self
    }

    /// Enables guard pages for detecting out-of-bounds memory access.
    ///
    /// This is a no-op in the current implementation, but reserved for future
    /// use.
    pub fn with_guard_pages(mut self, enable: bool) -> Self {
        self.guard_pages = enable;
        self
    }

    /// Enables memory tagging for enhanced memory safety (e.g., MTE on Arm).
    ///
    /// This is a no-op in the current implementation, but reserved for future
    /// use.
    pub fn with_memory_tagging(mut self, enable: bool) -> Self {
        self.memory_tagging = enable;
        self
    }

    /// Builds and returns a configured `MacOsAllocator`.
    pub fn build(self) -> MacOsAllocator {
        // Currently, we only support configuring maximum pages
        // The other options are reserved for future implementation
        MacOsAllocator::new(self.maximum_pages)
    }
}

impl PageAllocator for MacOsAllocator {
    fn allocate(
        &mut self,
        initial_pages: u32,
        maximum_pages: Option<u32>,
    ) -> Result<(NonNull<u8>, usize)> {
        if self.base_ptr.is_some() {
            return Err(Error::new(
                ErrorCategory::System,
                codes::INITIALIZATION_ERROR,
                "Allocator has already allocated memory",
            ));
        }

        if initial_pages == 0 {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::RUNTIME_INVALID_ARGUMENT_ERROR,
                "Initial pages cannot be zero",
            ));
        }

        let initial_bytes = Self::pages_to_bytes(initial_pages)?;
        let max_pages_hint = maximum_pages.unwrap_or(initial_pages).max(initial_pages);
        let reserve_bytes = Self::pages_to_bytes(max_pages_hint)?.max(initial_bytes);

        if reserve_bytes > self.max_capacity_bytes {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::CAPACITY_EXCEEDED,
                "Requested reservation size exceeds allocator's maximum capacity",
            ));
        }

        // Reserve address space for `reserve_bytes` but initially commit only
        // `initial_bytes`. The plan for macOS: `mmap(PROT_READ | PROT_WRITE)`
        // For now, we will mmap the whole `reserve_bytes` with R/W,
        // as true demand paging / partial commits are more complex with mmap
        // without further mprotect calls. Simpler first: mmap all needed.
        // A more advanced version could mmap with PROT_NONE then mprotect parts.
        // SAFETY: This block calls `mmap` via FFI. The arguments are constructed
        // to be valid for `MAP_PRIVATE | MAP_ANON` allocation. `ptr::null_mut()`
        // is a valid address hint. `reserve_bytes` is the calculated length.
        // Protections and flags are standard. `fd` is -1 and `offset` is 0, as
        // required for anonymous mappings. The result is checked for `MAP_FAILED`.
        let ptr = unsafe {
            mmap(
                ptr::null_mut(),        // address hint
                reserve_bytes,          // length
                PROT_READ | PROT_WRITE, // protection
                MAP_PRIVATE | MAP_ANON, // flags
                -1,                     // file descriptor (none for ANON)
                0,                      // offset
            )
        };

        if ptr == MAP_FAILED {
            let error_message = {
                // When std is not available, provide a static error message.
                // The specific errno can be logged separately if needed by a higher-level
                // system or captured in a more structured error type if
                // Error::new could take an optional code.
                "mmap failed due to OS error. Check OS error codes for details."
            };
            return Err(Error::new(
                ErrorCategory::System,
                codes::MEMORY_ALLOCATION_ERROR,
                error_message,
            ));
        }

        self.base_ptr = NonNull::new(ptr as *mut u8);
        self.total_reserved_bytes = reserve_bytes;
        self.current_committed_bytes = initial_bytes;

        // Ensure base_ptr is Some after MAP_FAILED check, otherwise it's an internal
        // logic error.
        let base_ptr = self.base_ptr.ok_or_else(|| {
            Error::new(
                ErrorCategory::System,
                codes::INVALID_STATE,
                "Internal error: mmap succeeded but base_ptr is None",
            )
        })?;

        Ok((
            base_ptr,
            initial_bytes, // Report initially "usable" bytes as per trait
        ))
    }

    fn grow(&mut self, current_pages: u32, additional_pages: u32) -> Result<()> {
        let _base_ptr = self.base_ptr.ok_or_else(|| {
            Error::new(
                ErrorCategory::System,
                codes::MEMORY_ACCESS_ERROR,
                "No memory allocated to grow",
            )
        })?;

        if additional_pages == 0 {
            return Ok(());
        }

        let current_bytes_from_arg = Self::pages_to_bytes(current_pages)?;
        if current_bytes_from_arg != self.current_committed_bytes {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::RUNTIME_INVALID_ARGUMENT_ERROR,
                "Inconsistent current_pages argument for grow operation: byte counts do not match",
            ));
        }

        let new_total_pages = current_pages
            .checked_add(additional_pages)
            .ok_or_else(|| Error::memory_error("Page count overflow during grow"))?;

        let new_committed_bytes = Self::pages_to_bytes(new_total_pages)?;

        if new_committed_bytes > self.total_reserved_bytes {
            // This means we need to truly grow the mapping, which mmap doesn't directly
            // support. `mremap` is Linux-specific. On macOS, you'd typically
            // mmap a new larger region, copy data, and munmap the old one, or
            // rely on initial over-reservation. The plan implies
            // `LinearMemory<P: PageAllocator>` and then platform backends.
            // For `memory.grow`, it should "Ensures that the memory region managed by this
            // allocator is at least..." The `FallbackAllocator` uses
            // `Vec::resize` which can reallocate. For mmap, if we didn't
            // reserve enough, this is tricky. The `maximum_pages` hint in
            // `allocate` is for this. Our current mmap in `allocate` reserves
            // `reserve_bytes` (based on maximum_pages).
            // So, if `new_committed_bytes` is within `total_reserved_bytes`, we just need
            // to ensure it's accessible, which it already is since we mapped it
            // all R/W.
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::CAPACITY_EXCEEDED,
                "Grow request exceeds total reserved memory space for this allocator instance",
            ));
        }

        // If the memory is already mmap'd with PROT_READ | PROT_WRITE up to
        // total_reserved_bytes, and new_committed_bytes <=
        // total_reserved_bytes, then the pages are already usable. We just need
        // to update our internal accounting.
        self.current_committed_bytes = new_committed_bytes;
        Ok(())
    }

    /// # Safety
    /// This function fulfills the safety contract of
    /// `PageAllocator::deallocate`.
    /// - `ptr` must be the non-null pointer returned by a previous `allocate`
    ///   call on this instance. This is checked by comparing `ptr` with the
    ///   stored `self.base_ptr`.
    /// - The `size` parameter (initial allocation size) is ignored by this
    ///   implementation; `munmap` is called with `self.total_reserved_bytes`,
    ///   which is the total size of the mapping created by `allocate`. This is
    ///   crucial for correctly deallocating the entire mmap'd region.
    /// - The caller must ensure no other references to the memory region exist
    ///   after this call, as per the trait's contract.
    unsafe fn deallocate(&mut self, ptr: NonNull<u8>, _size: usize) -> Result<()> {
        let base_ptr_val = self.base_ptr.ok_or_else(|| {
            Error::new(
                ErrorCategory::System,
                codes::MEMORY_ACCESS_ERROR,
                "Attempted to deallocate when no memory is allocated",
            )
        })?;

        // The `size` parameter in `deallocate` for PageAllocator is the *original* size
        // allocated. We must check if `ptr` matches `base_ptr_val`.
        if ptr != base_ptr_val {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::RUNTIME_INVALID_ARGUMENT_ERROR,
                "Pointer mismatch in deallocate. Invalid use of API.",
            ));
        }

        // SAFETY: `base_ptr_val.as_ptr()` is the valid pointer from mmap.
        // `total_reserved_bytes` is the size of the mmap'd region.
        // The caller guarantees no other references exist.
        let result = unsafe { munmap(base_ptr_val.as_ptr().cast(), self.total_reserved_bytes) };

        if result == -1 {
            // Error handling for munmap
            let error_message =
                { "munmap failed due to OS error. Check OS error codes for details." };
            return Err(Error::new(
                ErrorCategory::System,
                codes::MEMORY_DEALLOCATION_ERROR,
                error_message,
            ));
        }

        // Reset state after successful deallocation
        self.base_ptr = None;
        self.total_reserved_bytes = 0;
        self.current_committed_bytes = 0;

        Ok(())
    }
}

// SAFETY: MacOsAllocator manages raw pointers from mmap but ensures exclusive
// access conceptually through the PageAllocator trait. It does not share
// mutable state in a way that violates Send/Sync if the PageAllocator contract
// (one owner/user of an instance) is upheld.
// Implementations must be careful if internal state relies on thread-locals or
// other non-Send/Sync primitives.
// This implementation with mmap directly should be Send+Sync as the state is
// self-contained and operations are on OS resources identified by the pointer.
unsafe impl Send for MacOsAllocator {}
unsafe impl Sync for MacOsAllocator {}

impl Drop for MacOsAllocator {
    fn drop(&mut self) {
        if let Some(ptr) = self.base_ptr {
            // SAFETY: This is safe because `drop` is called when the `MacOsAllocator`
            // is no longer accessible. `ptr` and `total_reserved_bytes` reflect a valid
            // mmap'd region if `base_ptr` is Some.
            // The `_size` parameter of the trait deallocate is not used here, we use
            // our own `total_reserved_bytes`.
            // We ignore the result of deallocate in drop, as panicking in drop is bad.
            // Errors during deallocation in drop are typically logged or ignored.
            let _ = unsafe { self.deallocate(ptr, self.current_committed_bytes) };
            // Even if deallocate failed, we set base_ptr to None to prevent double free.
            self.base_ptr = None; // Ensure it won't be deallocated again if
                                  // deallocate errors.
        }
    }
}

// To get std::io::Error::last_os_error(), we need std.
// For no_std, we'd need a different way to get error details or just return a
// generic error. The `wrt-error` crate might provide utilities for this.
// For now, this implementation implicitly requires `std` for good error
// messages. We should make this explicit or provide a no_std friendly error
// reporting.

// Correcting error messages in allocate and deallocate:
// Use a function that can be conditionally compiled or use a generic message.
// For now, will assume 'std' is available for error formatting.
// This should be revisited when focusing on no_std compatibility for this
// specific module.

#[cfg(test)]
#[allow(clippy::panic, clippy::unwrap_used)] // Allow panic/unwrap in tests
mod tests {
    use wrt_error::ErrorSource;

    use super::*;
    use crate::memory::PageAllocator; // Ensure PageAllocator trait is in scope // Import ErrorSource trait for error
                                      // method access

    #[test]
    fn macos_allocator_new() {
        let allocator = MacOsAllocator::new(Some(100));
        assert!(allocator.base_ptr.is_none());
        assert_eq!(allocator.total_reserved_bytes, 0);
        assert_eq!(allocator.current_committed_bytes, 0);
        assert_eq!(allocator.max_capacity_bytes, 100 * WASM_PAGE_SIZE);

        let allocator_default = MacOsAllocator::new(None);
        assert_eq!(
            allocator_default.max_capacity_bytes,
            MacOsAllocator::DEFAULT_MAX_PAGES as usize * WASM_PAGE_SIZE
        );
    }

    #[test]
    fn macos_allocator_allocate_deallocate_cycle() {
        let mut allocator = MacOsAllocator::new(Some(10));
        let initial_pages = 2;
        let max_pages_alloc = Some(5);

        let (ptr, size) =
            allocator.allocate(initial_pages, max_pages_alloc).expect("allocate failed");

        assert!(!ptr.as_ptr().is_null());
        assert_eq!(size, initial_pages as usize * WASM_PAGE_SIZE);
        assert!(allocator.base_ptr.is_some());
        assert_eq!(allocator.base_ptr.unwrap(), ptr);
        assert_eq!(
            allocator.total_reserved_bytes,
            max_pages_alloc.unwrap() as usize * WASM_PAGE_SIZE
        );
        assert_eq!(allocator.current_committed_bytes, initial_pages as usize * WASM_PAGE_SIZE);

        // Write to the memory to ensure it's accessible
        unsafe {
            ptr.as_ptr().write_bytes(0xAB, size);
            assert_eq!(ptr.as_ptr().read_volatile(), 0xAB);
        }

        unsafe {
            allocator.deallocate(ptr, size).expect("deallocate failed");
        }

        assert!(allocator.base_ptr.is_none());
        assert_eq!(allocator.total_reserved_bytes, 0);
        assert_eq!(allocator.current_committed_bytes, 0);
    }

    #[test]
    fn macos_allocator_allocate_zero_initial_pages_fails() {
        let mut allocator = MacOsAllocator::new(Some(10));
        let result = allocator.allocate(0, Some(5));
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.code(), codes::RUNTIME_INVALID_ARGUMENT_ERROR);
        }
    }

    #[test]
    fn macos_allocator_allocate_exceeds_capacity() {
        let mut allocator = MacOsAllocator::new(Some(1)); // Max 1 page
        let result = allocator.allocate(2, Some(2)); // Request 2 pages
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.code(), codes::CAPACITY_EXCEEDED);
        }
    }

    #[test]
    fn macos_allocator_allocate_already_allocated() {
        let mut allocator = MacOsAllocator::new(Some(10));
        let (ptr, size) = allocator.allocate(1, Some(1)).expect("first allocate failed");

        let result = allocator.allocate(1, Some(1)); // Attempt to allocate again
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.code(), codes::INITIALIZATION_ERROR);
        }

        // Cleanup first allocation
        unsafe {
            allocator.deallocate(ptr, size).expect("deallocate failed");
        }
    }

    #[test]
    fn macos_allocator_grow_simple() {
        let mut allocator = MacOsAllocator::new(Some(10));
        let initial_pages = 1;
        let max_pages_alloc = Some(5);
        let (ptr, size) =
            allocator.allocate(initial_pages, max_pages_alloc).expect("allocate failed");

        let grow_additional_pages = 1;
        let grow_res = allocator.grow(initial_pages, grow_additional_pages);
        assert!(grow_res.is_ok(), "grow failed: {:?}", grow_res.err());

        assert_eq!(
            allocator.current_committed_bytes,
            (initial_pages + grow_additional_pages) as usize * WASM_PAGE_SIZE
        );
        assert_eq!(
            allocator.total_reserved_bytes, // Should remain the same as mmap reserved it all
            max_pages_alloc.unwrap() as usize * WASM_PAGE_SIZE
        );

        // Write to the newly grown part
        let new_total_bytes = allocator.current_committed_bytes;
        unsafe {
            // Write to the end of the original allocation
            ptr.as_ptr().add(initial_pages as usize * WASM_PAGE_SIZE - 1).write_volatile(0xBB);
            assert_eq!(
                ptr.as_ptr().add(initial_pages as usize * WASM_PAGE_SIZE - 1).read_volatile(),
                0xBB
            );

            // Write to the start of the newly grown area
            ptr.as_ptr().add(initial_pages as usize * WASM_PAGE_SIZE).write_volatile(0xCC);
            assert_eq!(
                ptr.as_ptr().add(initial_pages as usize * WASM_PAGE_SIZE).read_volatile(),
                0xCC
            );

            // Write to the end of the newly grown area
            ptr.as_ptr().add(new_total_bytes - 1).write_volatile(0xDD);
            assert_eq!(ptr.as_ptr().add(new_total_bytes - 1).read_volatile(), 0xDD);
        }

        unsafe {
            allocator.deallocate(ptr, size).expect("deallocate failed");
        }
    }

    #[test]
    fn macos_allocator_grow_zero_pages() {
        let mut allocator = MacOsAllocator::new(Some(10));
        let (ptr, size) = allocator.allocate(1, Some(5)).expect("allocate failed");
        let initial_committed_bytes = allocator.current_committed_bytes;

        let grow_res = allocator.grow(1, 0);
        assert!(grow_res.is_ok(), "grow by zero pages failed: {:?}", grow_res.err());
        assert_eq!(allocator.current_committed_bytes, initial_committed_bytes);

        unsafe {
            allocator.deallocate(ptr, size).expect("deallocate failed");
        }
    }

    #[test]
    fn macos_allocator_grow_exceeds_reserved_capacity() {
        let mut allocator = MacOsAllocator::new(Some(2)); // Overall capacity 2 pages
                                                          // Allocate 1, reserve up to 1 (due to mmap strategy)
        let (ptr, size) = allocator.allocate(1, Some(1)).expect("allocate failed");

        // Try to grow by 1 page. current_committed = 1, total_reserved = 1.
        // new_committed_bytes = 2 * PAGE_SIZE. total_reserved = 1 * PAGE_SIZE.
        // This should fail as it exceeds what was mmap'd (which was based on Some(1)).
        let grow_res = allocator.grow(1, 1);
        assert!(grow_res.is_err(), "Expected grow to fail due to exceeding reserved space");
        if let Err(e) = grow_res {
            assert_eq!(e.code(), codes::CAPACITY_EXCEEDED);
        }

        unsafe {
            allocator.deallocate(ptr, size).expect("deallocate failed");
        }
    }

    #[test]
    fn macos_allocator_grow_with_inconsistent_current_pages() {
        let mut allocator = MacOsAllocator::new(Some(10));
        let (ptr, size) = allocator.allocate(2, Some(5)).expect("allocate failed"); // current_committed_bytes is 2*PAGE_SIZE

        let grow_res = allocator.grow(1, 1); // Caller incorrectly states current is 1 page
        assert!(grow_res.is_err());
        if let Err(e) = grow_res {
            assert_eq!(e.code(), codes::RUNTIME_INVALID_ARGUMENT_ERROR);
        }
        unsafe {
            allocator.deallocate(ptr, size).expect("deallocate failed");
        }
    }

    #[test]
    fn macos_allocator_deallocate_unallocated() {
        let mut allocator = MacOsAllocator::new(Some(1));
        let dummy_ptr = NonNull::dangling(); // Dummy ptr
        let result = unsafe { allocator.deallocate(dummy_ptr, 0) };
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.code(), codes::MEMORY_ACCESS_ERROR);
        }
    }

    #[test]
    fn macos_allocator_deallocate_mismatched_pointer() {
        let mut allocator = MacOsAllocator::new(Some(10));
        let (ptr_correct, size_correct) = allocator.allocate(1, Some(1)).expect("allocate failed");

        // Create a different NonNull pointer (e.g., offset or entirely different)
        // This needs to be a plausible pointer value, not necessarily dangling if the
        // system checks for validity. For this test, any non-matching NonNull is fine.
        let ptr_incorrect =
            unsafe { NonNull::new_unchecked(ptr_correct.as_ptr().add(WASM_PAGE_SIZE)) };

        let result = unsafe { allocator.deallocate(ptr_incorrect, size_correct) };
        assert!(result.is_err(), "Deallocate with incorrect pointer should fail");
        if let Err(e) = result {
            assert_eq!(e.code(), codes::RUNTIME_INVALID_ARGUMENT_ERROR);
            assert_eq!(e.category(), ErrorCategory::Memory);
        }

        // Must deallocate the original correct pointer to clean up
        unsafe {
            allocator.deallocate(ptr_correct, size_correct).expect("Cleanup deallocate failed");
        }
    }

    #[test]
    fn macos_pages_to_bytes_overflow() {
        let large_pages = u32::MAX;
        let result = MacOsAllocator::pages_to_bytes(large_pages);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.code(), codes::CAPACITY_EXCEEDED);
        }

        let result_ok = MacOsAllocator::pages_to_bytes(1);
        assert_eq!(result_ok.unwrap(), WASM_PAGE_SIZE);
    }
}
