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
//! # Performance Monitoring
//!
//! Memory instances automatically track usage metrics:
//!
//! - Peak memory usage via `peak_memory()`
//! - Access counts via `access_count()`
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
use wrt_types::types::Limits;

#[cfg(feature = "std")]
use std::{
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        Arc, RwLock,
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

    /// Peak memory usage (no_std version)
    #[cfg(not(feature = "std"))]
    peak_usage: usize,
    /// Memory access counter (no_std version)
    #[cfg(not(feature = "std"))]
    access_count: u64,
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
            metrics: MemoryMetrics {
                peak_usage: AtomicUsize::new(size),
                access_count: AtomicU64::new(0),
            },
            #[cfg(not(feature = "std"))]
            metrics: RwLock::new(MemoryMetrics {
                peak_usage: size,
                access_count: 0,
            }),
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
        self.data.len()
    }

    /// Returns the peak memory usage during execution
    ///
    /// # Panics
    ///
    /// In `no_std` environments, this method will panic if the read lock for the metrics
    /// cannot be acquired. This would typically only happen in case of a deadlock or
    /// if the lock is poisoned due to a panic in another thread holding the lock.
    #[must_use]
    pub fn peak_memory(&self) -> usize {
        #[cfg(feature = "std")]
        {
            self.metrics.peak_usage.load(Ordering::Relaxed)
        }
        #[cfg(not(feature = "std"))]
        {
            let metrics = self.metrics.read().expect("Failed to acquire read lock");
            metrics.peak_usage
        }
    }

    /// Returns the memory access count for profiling
    ///
    /// # Panics
    ///
    /// In `no_std` environments, this method will panic if the read lock for the metrics
    /// cannot be acquired. This would typically only happen in case of a deadlock or
    /// if the lock is poisoned due to a panic in another thread holding the lock.
    #[must_use]
    pub fn access_count(&self) -> u64 {
        #[cfg(feature = "std")]
        {
            self.metrics.access_count.load(Ordering::Relaxed)
        }
        #[cfg(not(feature = "std"))]
        {
            let metrics = self.metrics.read().expect("Failed to acquire read lock");
            metrics.access_count
        }
    }

    /// Increments the access count for profiling
    ///
    /// # Panics
    ///
    /// In `no_std` environments, this method will panic if the write lock for the metrics
    /// cannot be acquired. This would typically only happen in case of a deadlock or
    /// if the lock is poisoned due to a panic in another thread holding the lock.
    fn increment_access_count(&self) {
        #[cfg(feature = "std")]
        {
            self.metrics.access_count.fetch_add(1, Ordering::Relaxed);
        }
        #[cfg(not(feature = "std"))]
        {
            let mut metrics = self.metrics.write().expect("Failed to acquire write lock");
            metrics.access_count += 1;
        }
    }

    /// Update peak memory usage metric
    ///
    /// # Panics
    ///
    /// In `no_std` environments, this method will panic if the write lock for the metrics
    /// cannot be acquired. This would typically only happen in case of a deadlock or
    /// if the lock is poisoned due to a panic in another thread holding the lock.
    fn update_peak_memory(&self) {
        let current_size = self.size_in_bytes();

        #[cfg(feature = "std")]
        {
            let mut peak = self.metrics.peak_usage.load(Ordering::Relaxed);
            while current_size > peak {
                match self.metrics.peak_usage.compare_exchange(
                    peak,
                    current_size,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(actual) => peak = actual,
                }
            }
        }

        #[cfg(not(feature = "std"))]
        {
            let mut metrics = self.metrics.write().expect("Failed to acquire write lock");
            metrics.peak_usage = metrics.peak_usage.max(current_size);
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
        let new_page_count = self.current_pages.checked_add(pages).ok_or_else(|| {
            Error::new(kinds::ValidationError("Memory size overflow".to_string()))
        })?;

        // Check if we exceed the maximum
        if let Some(max) = self.ty.limits.max {
            if new_page_count > max {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Cannot grow memory beyond maximum size: {} > {}",
                    new_page_count, max
                ))));
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
        self.increment_access_count();

        let offset = offset as usize;
        if offset + buffer.len() > self.data.len() {
            return Err(Error::new(kinds::ValidationError(
                "Memory access out of bounds".to_string(),
            )));
        }

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
        self.increment_access_count();

        let offset = offset as usize;
        if offset + buffer.len() > self.data.len() {
            return Err(Error::new(kinds::ValidationError(
                "Memory access out of bounds".to_string(),
            )));
        }

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
        self.increment_access_count();

        let offset = offset as usize;
        if offset >= self.data.len() {
            return Err(Error::new(kinds::ValidationError(
                "Memory access out of bounds".to_string(),
            )));
        }

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
        self.increment_access_count();

        let offset = offset as usize;
        if offset >= self.data.len() {
            return Err(Error::new(kinds::ValidationError(
                "Memory access out of bounds".to_string(),
            )));
        }

        self.data[offset] = value;
        Ok(())
    }

    /// Checks if a memory access at the given address with the specified alignment is valid
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to check
    /// * `access_size` - The size of the access in bytes
    /// * `align` - The required alignment in bytes
    ///
    /// # Returns
    ///
    /// Ok(()) if the access is valid
    ///
    /// # Errors
    ///
    /// Returns an error if the access is invalid
    pub fn check_alignment(&self, addr: u32, access_size: u32, align: u32) -> Result<()> {
        if addr % align != 0 {
            return Err(Error::new(kinds::ValidationError(format!(
                "Unaligned memory access: address {addr} is not aligned to {align} bytes"
            ))));
        }

        if (addr as usize) + (access_size as usize) > self.data.len() {
            return Err(Error::new(kinds::ValidationError(format!(
                "Memory access out of bounds: address {addr} + size {access_size} exceeds memory size {}",
                self.data.len()
            ))));
        }

        Ok(())
    }

    /// Gets a safe slice of memory for verified access
    ///
    /// This method creates a SafeSlice that includes checksumming and
    /// integrity verification for ASIL-B level safety.
    ///
    /// # Arguments
    ///
    /// * `addr` - The memory address
    /// * `len` - The length of the slice
    ///
    /// # Returns
    ///
    /// A SafeSlice with integrity verification
    ///
    /// # Errors
    ///
    /// Returns an error if the memory access is invalid
    pub fn get_safe_slice(
        &self,
        addr: u32,
        len: usize,
    ) -> Result<wrt_types::safe_memory::SafeSlice> {
        // Check bounds
        let addr = addr as usize;
        if addr + len > self.data.len() {
            return Err(Error::new(kinds::OutOfBoundsError(format!(
                "Memory access out of bounds: addr={}, len={}, size={}",
                addr,
                len,
                self.data.len()
            ))));
        }

        // Create a new SafeSlice with standard verification
        Ok(wrt_types::safe_memory::SafeSlice::with_verification_level(
            &self.data[addr..addr + len],
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

    /// Check memory integrity using verification
    #[cfg(feature = "std")]
    pub fn verify_integrity(&self) -> Result<()> {
        // Basic integrity check - verify that memory size is consistent
        // with the current page count
        let expected_size = self.current_pages as usize * PAGE_SIZE;
        if self.data.len() != expected_size {
            return Err(Error::new(kinds::ValidationError(format!(
                "Memory integrity check failed: expected size {} (pages: {}), got {}",
                expected_size,
                self.current_pages,
                self.data.len()
            ))));
        }

        // Here we would add more sophisticated checks like pattern verification
        // or checksum validation if we were storing checksums

        Ok(())
    }

    /// Copy memory contents from one memory to another or within the same memory
    ///
    /// # Arguments
    ///
    /// * `src_mem` - Source memory (can be the same as self)
    /// * `src_addr` - Source address
    /// * `dst_addr` - Destination address
    /// * `size` - Number of bytes to copy
    ///
    /// # Returns
    ///
    /// Ok(()) if the operation succeeds
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn copy_within_or_between(
        &mut self,
        src_mem: Arc<Memory>,
        src_addr: usize,
        dst_addr: usize,
        size: usize,
    ) -> Result<()> {
        // Bounds check for source
        if src_addr
            .checked_add(size)
            .is_none_or(|end| end > src_mem.data.len())
        {
            return Err(Error::new(kinds::MemoryAccessOutOfBoundsError {
                address: src_addr as u64,
                length: size as u64,
            }));
        }

        // Bounds check for destination
        if dst_addr
            .checked_add(size)
            .is_none_or(|end| end > self.data.len())
        {
            return Err(Error::new(kinds::MemoryAccessOutOfBoundsError {
                address: dst_addr as u64,
                length: size as u64,
            }));
        }

        // Increment the access count for both memories
        self.increment_access_count();
        src_mem.increment_access_count();

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

    /// Fill a region of memory with a byte value
    ///
    /// # Arguments
    ///
    /// * `dst` - Destination address
    /// * `val` - Byte value to fill with
    /// * `size` - Number of bytes to fill
    ///
    /// # Returns
    ///
    /// Ok(()) if the operation succeeds
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn fill(&mut self, dst: usize, val: u8, size: usize) -> Result<()> {
        // Bounds check for destination
        if dst
            .checked_add(size)
            .is_none_or(|end| end > self.data.len())
        {
            return Err(Error::new(kinds::MemoryAccessOutOfBoundsError {
                address: dst as u64,
                length: size as u64,
            }));
        }

        // Increment the access count
        self.increment_access_count();

        // Fill the memory
        self.data[dst..dst + size].fill(val);

        // Update peak memory usage
        self.update_peak_memory();

        Ok(())
    }

    /// Initialize a region of memory from a data segment
    ///
    /// # Arguments
    ///
    /// * `dst` - Destination address in memory
    /// * `data` - Source data to copy from
    /// * `src` - Source offset in the data
    /// * `size` - Number of bytes to copy
    ///
    /// # Returns
    ///
    /// Ok(()) if the initialization succeeds
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails
    pub fn init(&mut self, dst: usize, data: &[u8], src: usize, size: usize) -> Result<()> {
        // Bounds check for source
        if src.checked_add(size).is_none_or(|end| end > data.len()) {
            return Err(Error::new(kinds::MemoryAccessOutOfBoundsError {
                address: src as u64,
                length: size as u64,
            }));
        }

        // Bounds check for destination
        if dst
            .checked_add(size)
            .is_none_or(|end| end > self.data.len())
        {
            return Err(Error::new(kinds::MemoryAccessOutOfBoundsError {
                address: dst as u64,
                length: size as u64,
            }));
        }

        // Increment the access count
        self.increment_access_count();

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
            metrics: MemoryMetrics::default(), // Reset metrics on clone
            #[cfg(not(feature = "std"))]
            metrics: RwLock::new(MemoryMetrics::default()), // Reset metrics on clone
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(2),
            },
        };
        let memory = Memory::new(mem_type.clone()).unwrap();
        assert_eq!(memory.size(), 1);
        assert_eq!(memory.size_in_bytes(), PAGE_SIZE);
        assert_eq!(memory.debug_name(), None);

        let named_memory = Memory::new_with_name(mem_type, "test_memory").unwrap();
        assert_eq!(named_memory.debug_name(), Some("test_memory"));
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

        // Should fail because we've reached the max
        assert!(memory.grow(1).is_err());
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

        // Test write and read
        let data = [1, 2, 3, 4];
        memory.write(0, &data).unwrap();

        let mut buffer = [0; 4];
        memory.read(0, &mut buffer).unwrap();
        assert_eq!(buffer, data);

        // Test out of bounds
        assert!(memory.write(PAGE_SIZE as u32, &data).is_err());
        assert!(memory.read(PAGE_SIZE as u32, &mut buffer).is_err());

        // Test access count
        assert!(memory.access_count() > 0);
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

        // Test set and get
        memory.set_byte(0, 42).unwrap();
        assert_eq!(memory.get_byte(0).unwrap(), 42);

        // Test out of bounds
        assert!(memory.set_byte(PAGE_SIZE as u32, 42).is_err());
        assert!(memory.get_byte(PAGE_SIZE as u32).is_err());
    }

    #[test]
    fn test_memory_peak_usage() {
        let mem_type = MemoryType {
            limits: Limits {
                min: 1,
                max: Some(3),
            },
        };
        let mut memory = Memory::new(mem_type).unwrap();

        // Initial peak should be the initial size
        assert_eq!(memory.peak_memory(), PAGE_SIZE);

        // Grow and check peak
        memory.grow(1).unwrap();
        assert_eq!(memory.peak_memory(), 2 * PAGE_SIZE);

        // Grow more and check peak
        memory.grow(1).unwrap();
        assert_eq!(memory.peak_memory(), 3 * PAGE_SIZE);
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

        // Aligned access should succeed
        assert!(memory.check_alignment(0, 4, 4).is_ok());
        assert!(memory.check_alignment(4, 4, 4).is_ok());

        // Unaligned access should fail
        assert!(memory.check_alignment(1, 4, 4).is_err());
        assert!(memory.check_alignment(2, 4, 4).is_err());

        // Out of bounds access should fail
        assert!(memory.check_alignment(PAGE_SIZE as u32 - 2, 4, 4).is_err());
    }
}
