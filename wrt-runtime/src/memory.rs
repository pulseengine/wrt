//! WebAssembly memory implementation.
//!
//! This module provides a comprehensive implementation of WebAssembly linear memory.
//!
//! # Memory Architecture
//!
//! The `Memory` struct is the core implementation for WebAssembly linear memory. It represents
//! a memory instance as defined in the WebAssembly specification. Key features include:
//!
//! - Thread-safe access with internal synchronization
//! - Performance metrics tracking (access counts, peak usage)
//! - Debug name support for easier debugging
//! - Support for both `std` and `no_std` environments
//! - Memory safety checks and bounds validation
//! - Integrated checksum verification for data integrity
//!
//! # Memory Operations
//!
//! The implementation provides methods for all standard WebAssembly memory operations:
//!
//! - Growing memory (`grow`)
//! - Reading from memory (`read`, `get_byte`)
//! - Writing to memory (`write`, `set_byte`)
//! - Additional safety methods for alignment and bounds checking (`check_alignment`)
//!
//! # Safety Features
//!
//! The implementation includes several safety features:
//!
//! - Checksum verification for data integrity
//! - Bounds checking for all memory operations
//! - Alignment validation
//! - Thread safety guarantees
//! - Memory access tracking
//!
//! # Performance Monitoring
//!
//! Memory instances automatically track usage metrics:
//!
//! - Peak memory usage via `peak_memory()`
//! - Access counts via `access_count()`
//! - Memory access patterns via `memory_stats()`
//!
//! These metrics are updated automatically when memory operations are performed.
//!
//! # Thread Safety
//!
//! Memory operations use appropriate synchronization primitives based on the environment:
//!
//! - In `std` environments, atomic variables are used for metrics
//! - In `no_std` environments, `RwLock` is used for metrics
//!
//! # Usage
//!
//! ```no_run
//! use wrt_runtime::{Memory, MemoryType};
//! use wrt_types::types::Limits;
//!
//! // Create a memory type with initial 1 page (64KB) and max 2 pages
//! let mem_type = MemoryType {
//!     limits: Limits { min: 1, max: Some(2) },
//! };
//!
//! // Create a new memory instance
//! let mut memory = Memory::new(mem_type).unwrap();
//!
//! // Write data to memory
//! memory.write(0, &[1, 2, 3, 4]).unwrap();
//!
//! // Read data from memory
//! let mut buffer = [0; 4];
//! memory.read(0, &mut buffer).unwrap();
//! assert_eq!(buffer, [1, 2, 3, 4]);
//!
//! // Grow memory by 1 page
//! let old_size = memory.grow(1).unwrap();
//! assert_eq!(old_size, 1); // Previous size was 1 page
//! ```

use crate::types::MemoryType;
use crate::{Error, Result};
use wrt_error::kinds;
use wrt_error::{errors::codes, ErrorCategory};
use wrt_types::safe_memory::MemoryStats;
use wrt_types::safe_memory::{MemoryProvider, MemorySafety, SafeSlice};
use wrt_types::verification::VerificationLevel;

#[cfg(feature = "std")]
use std::{
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc,
    },
    vec::Vec,
};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// WebAssembly page size (64KB)
pub const PAGE_SIZE: usize = 65536;

/// Maximum number of memory pages allowed by WebAssembly spec
pub const MAX_PAGES: u32 = 65536;

/// Memory metrics for tracking usage and performance
#[derive(Debug, Default)]
struct MemoryMetrics {
    /// Peak memory usage in bytes
    #[cfg(feature = "std")]
    peak_usage: AtomicUsize,
    /// Memory access counter for profiling
    #[cfg(feature = "std")]
    access_count: AtomicU64,
    /// Maximum size of any access
    #[cfg(feature = "std")]
    max_access_size: AtomicUsize,
    /// Number of unique regions accessed
    #[cfg(feature = "std")]
    unique_regions: AtomicUsize,
    /// Last access offset for validation
    #[cfg(feature = "std")]
    last_access_offset: AtomicUsize,
    /// Last access length for validation
    #[cfg(feature = "std")]
    last_access_length: AtomicUsize,

    /// Peak memory usage (no_std version)
    #[cfg(not(feature = "std"))]
    peak_usage: usize,
    /// Memory access counter (no_std version)
    #[cfg(not(feature = "std"))]
    access_count: u64,
    /// Maximum size of any access (no_std version)
    #[cfg(not(feature = "std"))]
    max_access_size: usize,
    /// Number of unique regions accessed (no_std version)
    #[cfg(not(feature = "std"))]
    unique_regions: usize,
    /// Last access offset for validation (no_std version)
    #[cfg(not(feature = "std"))]
    last_access_offset: usize,
    /// Last access length for validation (no_std version)
    #[cfg(not(feature = "std"))]
    last_access_length: usize,
}

impl MemoryMetrics {
    #[cfg(feature = "std")]
    fn new(size: usize) -> Self {
        Self {
            peak_usage: AtomicUsize::new(size),
            access_count: AtomicU64::new(0),
            max_access_size: AtomicUsize::new(0),
            unique_regions: AtomicUsize::new(0),
            last_access_offset: AtomicUsize::new(0),
            last_access_length: AtomicUsize::new(0),
        }
    }

    #[cfg(not(feature = "std"))]
    fn new(size: usize) -> Self {
        Self {
            peak_usage: size,
            access_count: 0,
            max_access_size: 0,
            unique_regions: 0,
            last_access_offset: 0,
            last_access_length: 0,
        }
    }

    #[cfg(feature = "std")]
    fn track_access(&self, offset: usize, len: usize) {
        self.access_count.fetch_add(1, Ordering::Relaxed);
        self.max_access_size.fetch_max(len, Ordering::Relaxed);
        self.last_access_offset.store(offset, Ordering::Relaxed);
        self.last_access_length.store(len, Ordering::Relaxed);
    }

    #[cfg(not(feature = "std"))]
    fn track_access(&mut self, offset: usize, len: usize) {
        self.access_count += 1;
        self.max_access_size = self.max_access_size.max(len);
        self.last_access_offset = offset;
        self.last_access_length = len;
    }

    #[cfg(feature = "std")]
    fn update_peak_usage(&self, size: usize) {
        let mut current = self.peak_usage.load(Ordering::Relaxed);
        while size > current {
            match self.peak_usage.compare_exchange(
                current,
                size,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
    }

    #[cfg(not(feature = "std"))]
    fn update_peak_usage(&mut self, size: usize) {
        self.peak_usage = self.peak_usage.max(size);
    }
}

/// Represents a WebAssembly memory instance
#[derive(Debug)]
pub struct Memory {
    /// Memory type
    pub ty: MemoryType,
    /// Memory data
    data: Vec<u8>,
    /// Current size in pages
    current_pages: u32,
    /// Debug name for this memory instance (optional)
    debug_name: Option<String>,
    /// Performance metrics
    #[cfg(feature = "std")]
    metrics: MemoryMetrics,
    /// Performance metrics (wrapped in RwLock for no_std)
    #[cfg(not(feature = "std"))]
    metrics: RwLock<MemoryMetrics>,
    /// Verification level for memory operations
    verification_level: VerificationLevel,
}

impl Memory {
    /// Creates a new memory instance
    ///
    /// # Arguments
    ///
    /// * `ty` - The memory type
    ///
    /// # Returns
    ///
    /// A new Memory instance with the given type
    ///
    /// # Errors
    ///
    /// Returns an error if the memory cannot be created
    pub fn new(ty: MemoryType) -> Result<Self> {
        // Calculate memory size based on page size (64KB)
        let size = ty.limits.min as usize * PAGE_SIZE;
        let min_pages = ty.limits.min;

        let memory = Self {
            ty,
            data: vec![0; size],
            current_pages: min_pages,
            debug_name: None,
            #[cfg(feature = "std")]
            metrics: MemoryMetrics::new(size),
            #[cfg(not(feature = "std"))]
            metrics: RwLock::new(MemoryMetrics::new(size)),
            verification_level: VerificationLevel::default(),
        };

        Ok(memory)
    }

    /// Creates a new memory instance with a debug name
    ///
    /// # Arguments
    ///
    /// * `ty` - The memory type
    /// * `name` - The debug name for the memory
    ///
    /// # Returns
    ///
    /// A new Memory instance with the given type and name
    ///
    /// # Errors
    ///
    /// Returns an error if the memory cannot be created
    pub fn new_with_name(ty: MemoryType, name: &str) -> Result<Self> {
        let mut memory = Self::new(ty)?;
        memory.debug_name = Some(name.to_string());
        Ok(memory)
    }

    /// Sets a debug name for this memory instance
    pub fn set_debug_name(&mut self, name: &str) {
        self.debug_name = Some(name.to_string());
    }

    /// Returns the debug name of this memory instance, if any
    #[must_use]
    pub fn debug_name(&self) -> Option<&str> {
        self.debug_name.as_deref()
    }

    /// Gets the current size of the memory in pages
    ///
    /// # Returns
    ///
    /// The current size in pages
    #[must_use]
    pub fn size(&self) -> u32 {
        self.current_pages
    }

    /// Gets the current size of the memory in bytes
    ///
    /// # Returns
    ///
    /// The current size in bytes
    #[must_use]
    pub fn size_in_bytes(&self) -> usize {
        self.current_pages as usize * PAGE_SIZE
    }

    /// Returns a reference to the memory buffer
    ///
    /// # Returns
    ///
    /// A reference to the memory data
    pub fn buffer(&self) -> &[u8] {
        &self.data
    }

    /// Returns the peak memory usage during execution
    ///
    /// # Panics
    ///
    /// In `no_std` environments, this method will panic if the read lock for the metrics
    /// cannot be acquired. This would typically only happen in case of a deadlock or
    /// if the lock is poisoned due to a panic in another thread holding the lock.
    /// Safety impact: [LOW|MEDIUM|HIGH] - [Brief explanation of the safety implication]
    /// Tracking: WRTQ-XXX (qualification requirement tracking ID).
    #[must_use]
    pub fn peak_memory(&self) -> usize {
        #[cfg(feature = "std")]
        {
            self.metrics.peak_usage.load(Ordering::Relaxed)
        }
        #[cfg(not(feature = "std"))]
        {
            self.metrics.peak_usage
        }
    }

    /// Returns the memory access count for profiling
    ///
    /// # Panics
    ///
    /// In `no_std` environments, this method will panic if the read lock for the metrics
    /// cannot be acquired. This would typically only happen in case of a deadlock or
    /// Safety impact: [LOW|MEDIUM|HIGH] - [Brief explanation of the safety implication]
    /// Tracking: WRTQ-XXX (qualification requirement tracking ID).
    /// if the lock is poisoned due to a panic in another thread holding the lock.
    #[must_use]
    pub fn access_count(&self) -> u64 {
        #[cfg(feature = "std")]
        {
            self.metrics.access_count.load(Ordering::Relaxed)
        }
        #[cfg(not(feature = "std"))]
        {
            self.metrics.access_count
        }
    }

    /// Increments the access counter and tracks access patterns
    fn increment_access_count(&self, offset: usize, len: usize) {
        #[cfg(feature = "std")]
        {
            self.metrics.track_access(offset, len);
        }
        #[cfg(not(feature = "std"))]
        {
            let mut metrics = self.metrics.write().expect("Failed to acquire write lock");
            metrics.track_access(offset, len);
        }
    }

    /// Updates the peak memory usage if the current usage is higher
    fn update_peak_memory(&self) {
        let current_size = self.size_in_bytes();
        #[cfg(feature = "std")]
        {
            self.metrics.update_peak_usage(current_size);
        }
        #[cfg(not(feature = "std"))]
        {
            let mut metrics = self.metrics.write().expect("Failed to acquire write lock");
            metrics.update_peak_usage(current_size);
        }
    }

    /// Grows the memory by the given number of pages
    ///
    /// # Arguments
    ///
    /// * `pages` - The number of pages to grow by
    ///
    /// # Returns
    ///
    /// The previous size in pages
    ///
    /// # Errors
    ///
    /// Returns an error if the memory cannot be grown (e.g., exceeds max size)
    pub fn grow(&mut self, pages: u32) -> Result<u32> {
        // Check if growing would overflow
        let new_page_count = self.current_pages.checked_add(pages).ok_or_else(|| {
            Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                "Memory size overflow",
            )
        })?;

        // Check if the new size exceeds the maximum size
        if let Some(max) = self.ty.limits.max {
            if new_page_count > max {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    codes::VALIDATION_ERROR,
                    format!(
                        "Cannot grow memory beyond maximum size: {} > {}",
                        new_page_count, max
                    ),
                ));
            }
        }

        // Grow the memory
        let old_size = self.current_pages;
        let new_size = new_page_count as usize * PAGE_SIZE;
        self.data.resize(new_size, 0);
        self.current_pages = new_page_count;

        // Update peak memory usage
        self.update_peak_memory();

        Ok(old_size)
    }

    /// Reads memory into a buffer
    ///
    /// # Arguments
    ///
    /// * `offset` - The offset to read from
    /// * `buffer` - The buffer to read into
    ///
    /// # Returns
    ///
    /// Ok(()) if the read was successful
    ///
    /// # Errors
    ///
    /// Returns an error if the read is out of bounds
    pub fn read(&self, offset: u32, buffer: &mut [u8]) -> Result<()> {
        if buffer.is_empty() {
            return Ok(());
        }

        if !self.verify_bounds(offset, buffer.len() as u32) {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                "Memory access out of bounds",
            ));
        }

        self.increment_access_count(offset as usize, buffer.len());

        let offset = offset as usize;
        buffer.copy_from_slice(&self.data[offset..offset + buffer.len()]);
        Ok(())
    }

    /// Writes memory from a buffer
    ///
    /// # Arguments
    ///
    /// * `offset` - The offset to write to
    /// * `buffer` - The buffer to write from
    ///
    /// # Returns
    ///
    /// Ok(()) if the write was successful
    ///
    /// # Errors
    ///
    /// Returns an error if the write is out of bounds
    pub fn write(&mut self, offset: u32, buffer: &[u8]) -> Result<()> {
        if buffer.is_empty() {
            return Ok(());
        }

        if !self.verify_bounds(offset, buffer.len() as u32) {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                "Memory access out of bounds",
            ));
        }

        self.increment_access_count(offset as usize, buffer.len());

        let offset = offset as usize;
        self.data[offset..offset + buffer.len()].copy_from_slice(buffer);
        Ok(())
    }

    /// Gets a byte from memory
    ///
    /// # Arguments
    ///
    /// * `offset` - The offset to read from
    ///
    /// # Returns
    ///
    /// The byte at the given offset
    ///
    /// # Errors
    ///
    /// Returns an error if the offset is out of bounds
    pub fn get_byte(&self, offset: u32) -> Result<u8> {
        if !self.verify_bounds(offset, 1) {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                "Memory access out of bounds",
            ));
        }

        self.increment_access_count(offset as usize, 1);

        let offset = offset as usize;
        Ok(self.data[offset])
    }

    /// Sets a byte in memory
    ///
    /// # Arguments
    ///
    /// * `offset` - The offset to write to
    /// * `value` - The value to write
    ///
    /// # Returns
    ///
    /// Ok(()) if the write was successful
    ///
    /// # Errors
    ///
    /// Returns an error if the offset is out of bounds
    pub fn set_byte(&mut self, offset: u32, value: u8) -> Result<()> {
        if !self.verify_bounds(offset, 1) {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                "Memory access out of bounds",
            ));
        }

        self.increment_access_count(offset as usize, 1);

        let offset = offset as usize;
        self.data[offset] = value;
        Ok(())
    }

    /// Verify that an access is within bounds
    fn verify_bounds(&self, offset: u32, len: u32) -> bool {
        // First check for overflow in the addition
        match offset.checked_add(len) {
            // Then check if the end is within bounds
            Some(end) => (end as usize) <= self.data.len(),
            // If we had overflow, it's definitely out of bounds
            None => false,
        }
    }

    /// Check alignment for memory accesses
    pub fn check_alignment(&self, addr: u32, access_size: u32, align: u32) -> Result<()> {
        if addr % align != 0 {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                format!("Unaligned memory access: address {addr} is not aligned to {align} bytes"),
            ));
        }

        let addr = addr as usize;
        let access_size = access_size as usize;
        if addr + access_size > self.data.len() {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                format!(
                    "Memory access out of bounds: address {addr} + size {access_size} exceeds memory size {}",
                    self.data.len()
                )
            ));
        }

        Ok(())
    }

    /// Get a safe slice of memory with integrity checking
    ///
    /// # Arguments
    ///
    /// * `addr` - The starting address
    /// * `len` - The length of the slice
    ///
    /// # Returns
    ///
    /// A safe slice with integrity checking
    ///
    /// # Errors
    ///
    /// Returns an error if the address is out of bounds
    pub fn get_safe_slice(
        &self,
        addr: u32,
        len: usize,
    ) -> Result<wrt_types::safe_memory::SafeSlice> {
        if (addr as usize) + len > self.data.len() {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::OUT_OF_BOUNDS_ERROR,
                format!(
                    "Memory access out of bounds: addr={}, len={}, size={}",
                    addr,
                    len,
                    self.data.len()
                ),
            ));
        }

        // Create a new SafeSlice with standard verification
        Ok(wrt_types::safe_memory::SafeSlice::with_verification_level(
            &self.data[addr as usize..addr as usize + len],
            wrt_types::verification::VerificationLevel::Standard,
        ))
    }

    /// Creates a copy of this memory instance and applies a mutation function
    ///
    /// This is useful for operations that need to mutate memory without affecting
    /// the original instance, such as in speculative execution or transaction-like
    /// operations.
    ///
    /// # Arguments
    ///
    /// * `mutate_fn` - The function to apply to the cloned memory
    ///
    /// # Returns
    ///
    /// The result of the mutation function
    pub fn clone_and_mutate<F, R>(&self, mutate_fn: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let mut cloned = self.clone();
        mutate_fn(&mut cloned)
    }

    /// Register a pre-grow hook to be executed before memory grows
    ///
    /// This is used by SafeMemory to validate the memory state before growing
    #[cfg(feature = "std")]
    pub fn with_pre_grow_hook<F>(&self, hook: F) -> Result<()>
    where
        F: FnOnce(u32, &[u8]) -> Result<()> + Send + 'static,
    {
        // Execute the hook immediately with current state
        hook(self.current_pages, &self.data)?;

        // In a full implementation, we would store the hook for later use
        // but for now we just run it once to validate the current state

        Ok(())
    }

    /// Register a post-grow hook to be executed after memory grows
    ///
    /// This is used by SafeMemory to update checksums and validate the new memory state
    #[cfg(feature = "std")]
    pub fn with_post_grow_hook<F>(&self, hook: F) -> Result<()>
    where
        F: FnOnce(u32, &[u8]) -> Result<()> + Send + 'static,
    {
        // Execute the hook immediately with current state
        hook(self.current_pages, &self.data)?;

        // In a full implementation, we would store the hook for later use
        // but for now we just run it once to validate the current state

        Ok(())
    }

    /// Verify the integrity of the memory instance
    ///
    /// This checks that the memory size is consistent with the number of pages.
    ///
    /// # Returns
    ///
    /// Ok(()) if the memory is valid
    ///
    /// # Errors
    ///
    /// Returns an error if the memory integrity check fails
    pub fn verify_integrity(&self) -> Result<()> {
        let expected_size = self.current_pages as usize * PAGE_SIZE;
        if self.data.len() != expected_size {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                format!(
                    "Memory integrity check failed: expected size {} (pages: {}), got {}",
                    expected_size,
                    self.current_pages,
                    self.data.len()
                ),
            ));
        }

        Ok(())
    }

    /// Copy memory within the same memory instance or between two memory instances
    ///
    /// This method implements the memory.copy instruction from the WebAssembly bulk memory operations proposal.
    ///
    /// # Arguments
    ///
    /// * `src_mem` - The source memory instance (can be the same as self)
    /// * `src_addr` - The source address
    /// * `dst_addr` - The destination address
    /// * `size` - The number of bytes to copy
    ///
    /// # Errors
    ///
    /// Returns an error if either the source or destination range is out of bounds
    pub fn copy_within_or_between(
        &mut self,
        src_mem: Arc<Memory>,
        src_addr: usize,
        dst_addr: usize,
        size: usize,
    ) -> Result<()> {
        // Bounds check for source
        let src_end = match src_addr.checked_add(size) {
            Some(end) if end <= src_mem.data.len() => end,
            _ => {
                return Err(Error::new(
                    ErrorCategory::Memory,
                    codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                    format!(
                        "Source memory access out of bounds: address={}, size={}",
                        src_addr, size
                    ),
                ))
            }
        };

        // Bounds check for destination
        let dst_end = match dst_addr.checked_add(size) {
            Some(end) if end <= self.data.len() => end,
            _ => {
                return Err(Error::new(
                    ErrorCategory::Memory,
                    codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                    format!(
                        "Destination memory access out of bounds: address={}, size={}",
                        dst_addr, size
                    ),
                ))
            }
        };

        // Increment the access count for both memories
        self.increment_access_count(src_addr, size);
        src_mem.increment_access_count(src_addr, size);

        // Special case for overlapping regions in the same memory
        if Arc::ptr_eq(&src_mem, &Arc::new(self.clone())) {
            // Handle the overlapping case using copy_within
            self.data.copy_within(src_addr..src_addr + size, dst_addr);
        } else {
            // Handle the non-overlapping case
            let src_slice = &src_mem.data[src_addr..src_addr + size];
            self.data[dst_addr..dst_addr + size].copy_from_slice(src_slice);
        }

        // Update peak memory usage
        self.update_peak_memory();

        Ok(())
    }

    /// Fill memory with a repeated value
    ///
    /// This method implements the memory.fill instruction from the WebAssembly bulk memory operations proposal.
    ///
    /// # Arguments
    ///
    /// * `dst` - The destination address
    /// * `val` - The byte value to fill with
    /// * `size` - The number of bytes to fill
    ///
    /// # Errors
    ///
    /// Returns an error if the destination range is out of bounds
    pub fn fill(&mut self, dst: usize, val: u8, size: usize) -> Result<()> {
        // Bounds check
        let dst_end = match dst.checked_add(size) {
            Some(end) if end <= self.data.len() => end,
            _ => {
                return Err(Error::new(
                    ErrorCategory::Memory,
                    codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                    format!(
                        "Memory access out of bounds: address={}, size={}",
                        dst, size
                    ),
                ))
            }
        };

        // Increment the access count
        self.increment_access_count(dst, size);

        // Fill the memory
        self.data[dst..dst + size].fill(val);

        // Update peak memory usage
        self.update_peak_memory();

        Ok(())
    }

    /// Initialize memory from a data segment
    ///
    /// This method implements the memory.init instruction from the WebAssembly bulk memory operations proposal.
    ///
    /// # Arguments
    ///
    /// * `dst` - The destination address in memory
    /// * `data` - The source data segment
    /// * `src` - The offset within the data segment
    /// * `size` - The number of bytes to copy
    ///
    /// # Errors
    ///
    /// Returns an error if the source or destination range is out of bounds
    pub fn init(&mut self, dst: usize, data: &[u8], src: usize, size: usize) -> Result<()> {
        // Source bounds check
        let src_end = match src.checked_add(size) {
            Some(end) if end <= data.len() => end,
            _ => {
                return Err(Error::new(
                    ErrorCategory::Memory,
                    codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                    format!(
                        "Source data access out of bounds: address={}, size={}",
                        src, size
                    ),
                ))
            }
        };

        // Destination bounds check
        let dst_end = match dst.checked_add(size) {
            Some(end) if end <= self.data.len() => end,
            _ => {
                return Err(Error::new(
                    ErrorCategory::Memory,
                    codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                    format!(
                        "Destination memory access out of bounds: address={}, size={}",
                        dst, size
                    ),
                ))
            }
        };

        // Increment the access count
        self.increment_access_count(dst, size);

        // Copy the data
        self.data[dst..dst + size].copy_from_slice(&data[src..src + size]);

        // Update peak memory usage
        self.update_peak_memory();

        Ok(())
    }

    /// Read a 32-bit integer from memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to read from
    ///
    /// # Returns
    ///
    /// The 32-bit integer read from memory
    ///
    /// # Errors
    ///
    /// Returns an error if the read is out of bounds
    pub fn read_i32(&self, addr: u32) -> Result<i32> {
        let mut buffer = [0; 4];
        self.read(addr, &mut buffer)?;
        Ok(i32::from_le_bytes(buffer))
    }

    /// Read a 64-bit integer from memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to read from
    ///
    /// # Returns
    ///
    /// The 64-bit integer read from memory
    ///
    /// # Errors
    ///
    /// Returns an error if the read is out of bounds
    pub fn read_i64(&self, addr: u32) -> Result<i64> {
        let mut buffer = [0; 8];
        self.read(addr, &mut buffer)?;
        Ok(i64::from_le_bytes(buffer))
    }

    /// Read a 32-bit float from memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to read from
    ///
    /// # Returns
    ///
    /// The 32-bit float read from memory
    ///
    /// # Errors
    ///
    /// Returns an error if the read is out of bounds
    pub fn read_f32(&self, addr: u32) -> Result<f32> {
        let value = self.read_i32(addr)?;
        Ok(f32::from_bits(value as u32))
    }

    /// Read a 64-bit float from memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to read from
    ///
    /// # Returns
    ///
    /// The 64-bit float read from memory
    ///
    /// # Errors
    ///
    /// Returns an error if the read is out of bounds
    pub fn read_f64(&self, addr: u32) -> Result<f64> {
        let value = self.read_i64(addr)?;
        Ok(f64::from_bits(value as u64))
    }

    /// Read an 8-bit signed integer from memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to read from
    ///
    /// # Returns
    ///
    /// The 8-bit signed integer read from memory
    ///
    /// # Errors
    ///
    /// Returns an error if the read is out of bounds
    pub fn read_i8(&self, addr: u32) -> Result<i8> {
        let mut buffer = [0; 1];
        self.read(addr, &mut buffer)?;
        Ok(i8::from_le_bytes(buffer))
    }

    /// Read an 8-bit unsigned integer from memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to read from
    ///
    /// # Returns
    ///
    /// The 8-bit unsigned integer read from memory
    ///
    /// # Errors
    ///
    /// Returns an error if the read is out of bounds
    pub fn read_u8(&self, addr: u32) -> Result<u8> {
        let mut buffer = [0; 1];
        self.read(addr, &mut buffer)?;
        Ok(u8::from_le_bytes(buffer))
    }

    /// Read a 16-bit signed integer from memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to read from
    ///
    /// # Returns
    ///
    /// The 16-bit signed integer read from memory
    ///
    /// # Errors
    ///
    /// Returns an error if the read is out of bounds
    pub fn read_i16(&self, addr: u32) -> Result<i16> {
        let mut buffer = [0; 2];
        self.read(addr, &mut buffer)?;
        Ok(i16::from_le_bytes(buffer))
    }

    /// Read a 16-bit unsigned integer from memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to read from
    ///
    /// # Returns
    ///
    /// The 16-bit unsigned integer read from memory
    ///
    /// # Errors
    ///
    /// Returns an error if the read is out of bounds
    pub fn read_u16(&self, addr: u32) -> Result<u16> {
        let mut buffer = [0; 2];
        self.read(addr, &mut buffer)?;
        Ok(u16::from_le_bytes(buffer))
    }

    /// Read a 32-bit unsigned integer from memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to read from
    ///
    /// # Returns
    ///
    /// The 32-bit unsigned integer read from memory
    ///
    /// # Errors
    ///
    /// Returns an error if the read is out of bounds
    pub fn read_u32(&self, addr: u32) -> Result<u32> {
        let mut buffer = [0; 4];
        self.read(addr, &mut buffer)?;
        Ok(u32::from_le_bytes(buffer))
    }

    /// Read a 64-bit unsigned integer from memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to read from
    ///
    /// # Returns
    ///
    /// The 64-bit unsigned integer read from memory
    ///
    /// # Errors
    ///
    /// Returns an error if the read is out of bounds
    pub fn read_u64(&self, addr: u32) -> Result<u64> {
        let mut buffer = [0; 8];
        self.read(addr, &mut buffer)?;
        Ok(u64::from_le_bytes(buffer))
    }

    /// Read a 128-bit vector from memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to read from
    ///
    /// # Returns
    ///
    /// The 128-bit vector read from memory
    ///
    /// # Errors
    ///
    /// Returns an error if the read is out of bounds
    pub fn read_v128(&self, addr: u32) -> Result<[u8; 16]> {
        let mut buffer = [0; 16];
        self.read(addr, &mut buffer)?;
        Ok(buffer)
    }

    /// Write a 32-bit integer to memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to write to
    /// * `value` - The value to write
    ///
    /// # Returns
    ///
    /// Ok(()) if the write succeeds
    ///
    /// # Errors
    ///
    /// Returns an error if the write is out of bounds
    pub fn write_i32(&mut self, addr: u32, value: i32) -> Result<()> {
        self.write(addr, &value.to_le_bytes())
    }

    /// Write a 64-bit integer to memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to write to
    /// * `value` - The value to write
    ///
    /// # Returns
    ///
    /// Ok(()) if the write succeeds
    ///
    /// # Errors
    ///
    /// Returns an error if the write is out of bounds
    pub fn write_i64(&mut self, addr: u32, value: i64) -> Result<()> {
        self.write(addr, &value.to_le_bytes())
    }

    /// Write a 32-bit float to memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to write to
    /// * `value` - The value to write
    ///
    /// # Returns
    ///
    /// Ok(()) if the write succeeds
    ///
    /// # Errors
    ///
    /// Returns an error if the write is out of bounds
    pub fn write_f32(&mut self, addr: u32, value: f32) -> Result<()> {
        self.write_i32(addr, value.to_bits() as i32)
    }

    /// Write a 64-bit float to memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to write to
    /// * `value` - The value to write
    ///
    /// # Returns
    ///
    /// Ok(()) if the write succeeds
    ///
    /// # Errors
    ///
    /// Returns an error if the write is out of bounds
    pub fn write_f64(&mut self, addr: u32, value: f64) -> Result<()> {
        self.write_i64(addr, value.to_bits() as i64)
    }

    /// Write an 8-bit signed integer to memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to write to
    /// * `value` - The value to write
    ///
    /// # Returns
    ///
    /// Ok(()) if the write succeeds
    ///
    /// # Errors
    ///
    /// Returns an error if the write is out of bounds
    pub fn write_i8(&mut self, addr: u32, value: i8) -> Result<()> {
        self.write(addr, &value.to_le_bytes())
    }

    /// Write an 8-bit unsigned integer to memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to write to
    /// * `value` - The value to write
    ///
    /// # Returns
    ///
    /// Ok(()) if the write succeeds
    ///
    /// # Errors
    ///
    /// Returns an error if the write is out of bounds
    pub fn write_u8(&mut self, addr: u32, value: u8) -> Result<()> {
        self.write(addr, &value.to_le_bytes())
    }

    /// Write a 16-bit signed integer to memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to write to
    /// * `value` - The value to write
    ///
    /// # Returns
    ///
    /// Ok(()) if the write succeeds
    ///
    /// # Errors
    ///
    /// Returns an error if the write is out of bounds
    pub fn write_i16(&mut self, addr: u32, value: i16) -> Result<()> {
        self.write(addr, &value.to_le_bytes())
    }

    /// Write a 16-bit unsigned integer to memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to write to
    /// * `value` - The value to write
    ///
    /// # Returns
    ///
    /// Ok(()) if the write succeeds
    ///
    /// # Errors
    ///
    /// Returns an error if the write is out of bounds
    pub fn write_u16(&mut self, addr: u32, value: u16) -> Result<()> {
        self.write(addr, &value.to_le_bytes())
    }

    /// Write a 32-bit unsigned integer to memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to write to
    /// * `value` - The value to write
    ///
    /// # Returns
    ///
    /// Ok(()) if the write succeeds
    ///
    /// # Errors
    ///
    /// Returns an error if the write is out of bounds
    pub fn write_u32(&mut self, addr: u32, value: u32) -> Result<()> {
        self.write(addr, &value.to_le_bytes())
    }

    /// Write a 64-bit unsigned integer to memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to write to
    /// * `value` - The value to write
    ///
    /// # Returns
    ///
    /// Ok(()) if the write succeeds
    ///
    /// # Errors
    ///
    /// Returns an error if the write is out of bounds
    pub fn write_u64(&mut self, addr: u32, value: u64) -> Result<()> {
        self.write(addr, &value.to_le_bytes())
    }

    /// Write a 128-bit vector to memory
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to write to
    /// * `value` - The value to write
    ///
    /// # Returns
    ///
    /// Ok(()) if the write succeeds
    ///
    /// # Errors
    ///
    /// Returns an error if the write is out of bounds
    pub fn write_v128(&mut self, addr: u32, value: [u8; 16]) -> Result<()> {
        self.write(addr, &value)
    }

    /// Sets the verification level for memory operations
    ///
    /// # Arguments
    ///
    /// * `level` - The new verification level
    pub fn set_verification_level(&mut self, level: VerificationLevel) {
        self.verification_level = level;
    }

    /// Gets the current verification level
    ///
    /// # Returns
    ///
    /// The current verification level
    #[must_use]
    pub fn verification_level(&self) -> VerificationLevel {
        self.verification_level
    }

    /// Gets the maximum size of any access
    #[must_use]
    pub fn max_access_size(&self) -> usize {
        #[cfg(feature = "std")]
        {
            self.metrics.max_access_size.load(Ordering::Relaxed)
        }
        #[cfg(not(feature = "std"))]
        {
            self.metrics.max_access_size
        }
    }

    /// Gets the number of unique regions accessed
    #[must_use]
    pub fn unique_regions(&self) -> usize {
        #[cfg(feature = "std")]
        {
            self.metrics.unique_regions.load(Ordering::Relaxed)
        }
        #[cfg(not(feature = "std"))]
        {
            self.metrics.unique_regions
        }
    }

    /// Gets the last access offset
    #[must_use]
    pub fn last_access_offset(&self) -> usize {
        #[cfg(feature = "std")]
        {
            self.metrics.last_access_offset.load(Ordering::Relaxed)
        }
        #[cfg(not(feature = "std"))]
        {
            self.metrics.last_access_offset
        }
    }

    /// Gets the last access length
    #[must_use]
    pub fn last_access_length(&self) -> usize {
        #[cfg(feature = "std")]
        {
            self.metrics.last_access_length.load(Ordering::Relaxed)
        }
        #[cfg(not(feature = "std"))]
        {
            self.metrics.last_access_length
        }
    }

    /// Get statistics about memory usage
    ///
    /// This provides information about memory access patterns and usage.
    fn memory_stats(&self) -> MemoryStats {
        #[cfg(feature = "std")]
        let stats = MemoryStats {
            total_size: self.data.len(),
            access_count: self.metrics.access_count.load(Ordering::Relaxed) as usize,
            unique_regions: self.metrics.unique_regions.load(Ordering::Relaxed),
            max_access_size: self.metrics.max_access_size.load(Ordering::Relaxed),
        };

        #[cfg(not(feature = "std"))]
        let stats = {
            let metrics = self.metrics.read();
            MemoryStats {
                total_size: self.data.len(),
                access_count: metrics.access_count as usize,
                unique_regions: metrics.unique_regions,
                max_access_size: metrics.max_access_size,
            }
        };

        stats
    }
}

// Implementation of Clone for Memory
impl Clone for Memory {
    fn clone(&self) -> Self {
        Self {
            ty: self.ty.clone(),
            data: self.data.clone(),
            current_pages: self.current_pages,
            debug_name: self.debug_name.clone(),
            #[cfg(feature = "std")]
            metrics: MemoryMetrics::new(self.size_in_bytes()),
            #[cfg(not(feature = "std"))]
            metrics: RwLock::new(MemoryMetrics::new(self.size_in_bytes())),
            verification_level: self.verification_level,
        }
    }
}

impl MemoryProvider for Memory {
    fn borrow_slice(&self, offset: usize, len: usize) -> Result<SafeSlice<'_>> {
        self.get_safe_slice(offset as u32, len)
    }

    fn verify_access(&self, offset: usize, len: usize) -> Result<()> {
        if offset + len > self.data.len() {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                format!(
                    "Memory access out of bounds: addr={}, len={}, size={}",
                    offset,
                    len,
                    self.data.len()
                ),
            ));
        }
        Ok(())
    }

    fn size(&self) -> usize {
        self.data.len()
    }
}

impl MemorySafety for Memory {
    fn verify_integrity(&self) -> Result<()> {
        self.verify_integrity()
    }

    fn set_verification_level(&mut self, level: VerificationLevel) {
        self.set_verification_level(level)
    }

    fn verification_level(&self) -> VerificationLevel {
        self.verification_level()
    }

    fn memory_stats(&self) -> MemoryStats {
        self.memory_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wrt_types::types::Limits;

    #[test]
    fn test_memory_creation() {
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };
        let memory = Memory::new(mem_type).unwrap();
        assert_eq!(memory.size(), 1);
        assert_eq!(memory.size_in_bytes(), PAGE_SIZE);
    }

    #[test]
    fn test_memory_grow() {
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };
        let mut memory = Memory::new(mem_type).unwrap();
        let old_size = memory.grow(1).unwrap();
        assert_eq!(old_size, 1);
        assert_eq!(memory.size(), 2);
        assert_eq!(memory.size_in_bytes(), 2 * PAGE_SIZE);
    }

    #[test]
    fn test_memory_read_write() {
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };
        let mut memory = Memory::new(mem_type).unwrap();
        let data = [1, 2, 3, 4];
        memory.write(0, &data).unwrap();
        let mut buffer = [0; 4];
        memory.read(0, &mut buffer).unwrap();
        assert_eq!(buffer, data);
    }

    #[test]
    fn test_memory_get_set_byte() {
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };
        let mut memory = Memory::new(mem_type).unwrap();
        memory.set_byte(0, 42).unwrap();
        assert_eq!(memory.get_byte(0).unwrap(), 42);
    }

    #[test]
    fn test_memory_peak_usage() {
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };
        let mut memory = Memory::new(mem_type).unwrap();
        assert_eq!(memory.peak_memory(), PAGE_SIZE);
        memory.grow(1).unwrap();
        assert_eq!(memory.peak_memory(), 2 * PAGE_SIZE);
    }

    #[test]
    fn test_alignment_check() {
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };
        let memory = Memory::new(mem_type).unwrap();
        assert!(memory.check_alignment(0, 4, 4).is_ok());
        assert!(memory.check_alignment(1, 4, 4).is_err());
    }

    #[test]
    fn test_memory_access_tracking() {
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };
        let mut memory = Memory::new(mem_type).unwrap();

        // Test single byte access
        memory.set_byte(0, 42).unwrap();
        assert_eq!(memory.max_access_size(), 1);
        assert_eq!(memory.last_access_offset(), 0);
        assert_eq!(memory.last_access_length(), 1);

        // Test multi-byte access
        let data = [1, 2, 3, 4];
        memory.write(10, &data).unwrap();
        assert_eq!(memory.max_access_size(), 4);
        assert_eq!(memory.last_access_offset(), 10);
        assert_eq!(memory.last_access_length(), 4);

        // Test read access
        let mut buffer = [0; 2];
        memory.read(20, &mut buffer).unwrap();
        assert_eq!(memory.max_access_size(), 4); // Still 4 from previous write
        assert_eq!(memory.last_access_offset(), 20);
        assert_eq!(memory.last_access_length(), 2);
    }

    #[test]
    fn test_memory_copy_tracking() {
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };
        let mut memory1 = Memory::new(mem_type.clone()).unwrap();
        let mut memory2 = Memory::new(mem_type).unwrap();

        // Initialize source memory
        let data = [1, 2, 3, 4];
        memory1.write(0, &data).unwrap();

        // Copy between memories
        memory2
            .copy_within_or_between(Arc::new(memory1.clone()), 0, 0, 4)
            .unwrap();

        // Check access tracking
        assert_eq!(memory1.max_access_size(), 4);
        assert_eq!(memory1.last_access_offset(), 0);
        assert_eq!(memory1.last_access_length(), 4);

        assert_eq!(memory2.max_access_size(), 4);
        assert_eq!(memory2.last_access_offset(), 0);
        assert_eq!(memory2.last_access_length(), 4);
    }

    #[test]
    fn test_memory_fill_tracking() {
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };
        let mut memory = Memory::new(mem_type).unwrap();

        // Fill memory region
        memory.fill(0, 42, 10).unwrap();

        // Check access tracking
        assert_eq!(memory.max_access_size(), 10);
        assert_eq!(memory.last_access_offset(), 0);
        assert_eq!(memory.last_access_length(), 10);
    }

    #[test]
    fn test_memory_init_tracking() {
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };
        let mut memory = Memory::new(mem_type).unwrap();

        // Initialize memory region
        let data = [1, 2, 3, 4, 5];
        memory.init(0, &data, 0, 5).unwrap();

        // Check access tracking
        assert_eq!(memory.max_access_size(), 5);
        assert_eq!(memory.last_access_offset(), 0);
        assert_eq!(memory.last_access_length(), 5);
    }
}
