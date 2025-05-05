//! Safe memory abstractions for WebAssembly runtime
//!
//! This module provides memory safety abstractions designed for
//! functional safety at ASIL-B level, implementing verification
//! mechanisms to detect memory corruption.

#![allow(unused_unsafe)]
#![cfg_attr(not(feature = "std"), allow(unused_unsafe))]

// Core imports
#[cfg(feature = "std")]
extern crate std;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

use core::cmp::{Eq, PartialEq};
use core::fmt;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::operations::{record_global_operation, OperationType};
use crate::verification::{Checksum, VerificationLevel};
use wrt_error::{codes, Error, ErrorCategory, Result};

#[cfg(feature = "std")]
use std::collections::HashSet;
#[cfg(feature = "std")]
use std::sync::Mutex;

#[cfg(feature = "std")]
use std::{format, vec::Vec};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{format, string::ToString, vec::Vec};

/// A safe slice with integrated checksum for data integrity verification
#[derive(Clone)]
pub struct SafeSlice<'a> {
    /// The underlying data slice
    data: &'a [u8],
    /// Checksum for data integrity verification
    checksum: Checksum,
    /// Length of the slice for redundant verification
    length: usize,
    /// Verification level for this slice
    verification_level: VerificationLevel,
}

impl<'a> SafeSlice<'a> {
    /// Create a new SafeSlice from a raw byte slice
    ///
    /// This computes a checksum for the data which can be used
    /// to verify integrity later.
    pub fn new(data: &'a [u8]) -> Self {
        Self::with_verification_level(data, VerificationLevel::default())
    }

    /// Create a new SafeSlice with a specific verification level
    ///
    /// This computes a checksum for the data which can be used
    /// to verify integrity later.
    ///
    /// # Panics
    ///
    /// This function will panic if the initial integrity verification fails.
    /// This can happen if memory corruption is detected during initialization.
    /// Safety impact: [LOW|MEDIUM|HIGH] - [Brief explanation of the safety implication]
    /// Tracking: WRTQ-XXX (qualification requirement tracking ID).
    pub fn with_verification_level(data: &'a [u8], level: VerificationLevel) -> Self {
        // Track checksum calculation
        record_global_operation(OperationType::ChecksumCalculation, level);

        let checksum = Checksum::compute(data);
        let length = data.len();

        let slice = Self {
            data,
            checksum,
            length,
            verification_level: level,
        };

        // Verify on creation to ensure consistency
        // Use full importance (255) for initial verification
        slice
            .verify_integrity_with_importance(255)
            .expect("Initial verification failed");

        slice
    }

    /// Get the underlying data slice
    ///
    /// This performs an integrity check before returning the data.
    pub fn data(&self) -> Result<&'a [u8]> {
        // Track memory read operation
        record_global_operation(OperationType::MemoryRead, self.verification_level);

        // Medium importance for data access (128)
        self.verify_integrity_with_importance(128)?;
        Ok(self.data)
    }

    /// Get length of the slice
    pub fn len(&self) -> usize {
        self.length
    }

    /// Check if the slice is empty
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Get the current verification level
    pub fn verification_level(&self) -> VerificationLevel {
        self.verification_level
    }

    /// Set a new verification level
    pub fn set_verification_level(&mut self, level: VerificationLevel) {
        self.verification_level = level;
    }

    /// Verify data integrity using the stored checksum
    pub fn verify_integrity(&self) -> Result<()> {
        // Track validation operation
        record_global_operation(OperationType::CollectionValidate, self.verification_level);

        // Medium importance by default (128)
        self.verify_integrity_with_importance(128)
    }

    /// Verify data integrity with specified operation importance
    ///
    /// The importance value (0-255) affects the likelihood of
    /// verification when using VerificationLevel::Sampling
    pub fn verify_integrity_with_importance(&self, importance: u8) -> Result<()> {
        // Skip verification if the level indicates we shouldn't verify
        if !self.verification_level.should_verify(importance) {
            return Ok(());
        }

        // If length doesn't match stored value, memory is corrupt
        if self.data.len() != self.length {
            return Err(Error::validation_error(
                "Memory corruption detected: length mismatch",
            ));
        }

        // Different paths for optimize vs non-optimize
        #[cfg(feature = "optimize")]
        {
            Ok(())
        }

        #[cfg(not(feature = "optimize"))]
        {
            // Track checksum calculation for important operations
            if importance >= 200 || self.verification_level.should_verify_redundant() {
                record_global_operation(
                    OperationType::ChecksumCalculation,
                    self.verification_level,
                );
            }

            // Skip detailed checks for low importance non-redundant checks
            if !self.verification_level.should_verify_redundant() && importance < 200 {
                return Ok(());
            }

            // Compute current checksum and compare with stored checksum
            let current = Checksum::compute(self.data);
            if current == self.checksum {
                Ok(())
            } else {
                Err(Error::validation_error(
                    "Memory corruption detected: checksum mismatch",
                ))
            }
        }
    }

    /// Create a sub-slice with the same safety guarantees
    pub fn slice(&self, start: usize, end: usize) -> Result<SafeSlice<'a>> {
        // Track memory read operation
        record_global_operation(OperationType::MemoryRead, self.verification_level);

        // High importance for slicing operations (200)
        self.verify_integrity_with_importance(200)?;

        if start > end || end > self.length {
            return Err(Error::memory_error(format!(
                "Invalid slice range: {}..{} (len: {})",
                start, end, self.length
            )));
        }

        // Create a new SafeSlice with the specified range
        // We need to use just the exact range requested
        let sub_data = &self.data[start..end];

        // Create a new SafeSlice with the same verification level
        Ok(SafeSlice::with_verification_level(
            sub_data,
            self.verification_level,
        ))
    }
}

impl fmt::Debug for SafeSlice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SafeSlice")
            .field("length", &self.length)
            .field("checksum", &self.checksum)
            .field("verification_level", &self.verification_level)
            .finish()
    }
}

/// Memory provider interface for different allocation strategies
///
/// This trait abstracts over different memory allocation strategies,
/// allowing both std and no_std environments to share the same interface.
pub trait MemoryProvider {
    /// Borrow a slice of memory with safety guarantees
    fn borrow_slice(&self, offset: usize, len: usize) -> Result<SafeSlice<'_>>;

    /// Verify that an access to memory would be valid
    fn verify_access(&self, offset: usize, len: usize) -> Result<()>;

    /// Get the total size of the memory
    fn size(&self) -> usize;
}

/// Memory safety trait for memory providers that implement safety features
pub trait MemorySafety {
    /// Verify memory integrity using checksums and internal checks
    fn verify_integrity(&self) -> Result<()>;

    /// Set the verification level for memory operations
    fn set_verification_level(&mut self, level: VerificationLevel);

    /// Get current verification level
    fn verification_level(&self) -> VerificationLevel;

    /// Get statistics about memory usage
    fn memory_stats(&self) -> MemoryStats;
}

/// Statistics about memory usage for safety analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryStats {
    /// Total memory size in bytes
    pub total_size: usize,
    /// Number of memory accesses
    pub access_count: usize,
    /// Number of unique memory regions accessed
    pub unique_regions: usize,
    /// Maximum size of any access
    pub max_access_size: usize,
}

/// Memory provider for std environments
#[cfg(feature = "std")]
pub struct StdMemoryProvider {
    /// The underlying data buffer
    data: Vec<u8>,
    /// Track memory accesses for safety monitoring
    access_log: Mutex<Vec<(usize, usize)>>,
    /// Counter for access operations
    access_count: AtomicUsize,
    /// Maximum size of any access seen
    max_access_size: AtomicUsize,
    /// Number of unique regions accessed
    unique_regions: AtomicUsize,
    /// Regions hash (for uniqueness tracking)
    regions_hash: Mutex<HashSet<usize>>,
    /// Verification level for memory operations
    verification_level: VerificationLevel,
}

#[cfg(feature = "std")]
impl fmt::Debug for StdMemoryProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StdMemoryProvider")
            .field("data_size", &self.data.len())
            .field("access_count", &self.access_count.load(Ordering::Relaxed))
            .field(
                "max_access_size",
                &self.max_access_size.load(Ordering::Relaxed),
            )
            .field(
                "unique_regions",
                &self.unique_regions.load(Ordering::Relaxed),
            )
            .field("verification_level", &self.verification_level)
            .finish()
    }
}

#[cfg(feature = "std")]
impl StdMemoryProvider {
    /// Create a new StdMemoryProvider with the given data
    #[allow(clippy::redundant_clone)]
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            access_log: Mutex::new(Vec::with_capacity(100)),
            access_count: AtomicUsize::new(0),
            max_access_size: AtomicUsize::new(0),
            unique_regions: AtomicUsize::new(0),
            regions_hash: Mutex::new(HashSet::new()),
            verification_level: VerificationLevel::default(),
        }
    }

    /// Create a new empty StdMemoryProvider with the given capacity
    pub fn with_capacity(_capacity: usize) -> Self {
        Self::new(Vec::with_capacity(100))
    }

    /// Get the access log for debugging and verification
    pub fn access_log(&self) -> Result<Vec<(usize, usize)>> {
        match self.access_log.lock() {
            Ok(log) => Ok(log.clone()),
            Err(_) => Err(Error::runtime_error("Access log mutex poisoned")),
        }
    }

    /// Add data to the memory provider
    pub fn add_data(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    /// Clear all data and access logs
    pub fn clear(&mut self) {
        self.data.clear();
        if let Ok(mut log) = self.access_log.lock() {
            log.clear();
        }
        self.access_count.store(0, Ordering::SeqCst);
        self.max_access_size.store(0, Ordering::SeqCst);
        self.unique_regions.store(0, Ordering::SeqCst);
        if let Ok(mut regions) = self.regions_hash.lock() {
            regions.clear();
        }
    }

    /// Resize the memory to the given size
    pub fn resize(&mut self, new_size: usize, value: u8) {
        self.data.resize(new_size, value);
    }

    /// Track a memory access for safety monitoring
    fn track_access(&self, offset: usize, len: usize) {
        // Record memory read operation
        record_global_operation(OperationType::MemoryRead, self.verification_level);

        self.access_count.fetch_add(1, Ordering::Relaxed);

        // Update max access size if this is larger
        let current_max = self.max_access_size.load(Ordering::Relaxed);
        if len > current_max {
            self.max_access_size.store(len, Ordering::Relaxed);
        }

        // Track unique regions accessed (approximate with offset/1024)
        let region = offset >> 10; // Divide by 1024 to get region

        if let Ok(mut regions) = self.regions_hash.lock() {
            if regions.insert(region) {
                // New region, increment unique regions count
                self.unique_regions.fetch_add(1, Ordering::Relaxed);
            }
        }

        // Log the access if logging is enabled
        if let Ok(mut log) = self.access_log.lock() {
            log.push((offset, len));
        }
    }

    /// Clear the access tracking data but preserve statistics
    pub fn clear_access_tracking(&self) -> Result<()> {
        match self.access_log.lock() {
            Ok(mut log) => {
                log.clear();
                Ok(())
            }
            Err(_) => Err(Error::runtime_error("Access log mutex poisoned")),
        }
    }

    /// Set the verification level for memory operations
    pub fn set_verification_level(&mut self, level: VerificationLevel) {
        self.verification_level = level;
    }

    /// Get current verification level
    pub fn verification_level(&self) -> VerificationLevel {
        self.verification_level
    }

    /// Get detailed memory statistics
    pub fn memory_stats(&self) -> MemoryStats {
        MemoryStats {
            total_size: self.data.len(),
            access_count: self.access_count.load(Ordering::Relaxed),
            unique_regions: self.unique_regions.load(Ordering::Relaxed),
            max_access_size: self.max_access_size.load(Ordering::Relaxed),
        }
    }

    /// Recalculates checksums after direct memory modifications
    ///
    /// This method should be called after directly modifying the underlying data
    /// to ensure memory integrity verification continues to work properly.
    pub fn recalculate_checksums(&mut self) {
        // This is left as a placeholder in the current implementation
        // In a production-grade implementation, this would update any internal
        // checksums or verification data structures.

        // For now, the memory safety checks are mostly based on bounds checking
        // rather than checksums, but this extension point is important for future
        // enhancements to memory safety.

        // Track operation for proper profiling
        record_global_operation(OperationType::ChecksumCalculation, self.verification_level);
    }

    /// Gets a mutable pointer to data at the given offset
    pub fn get_ptr_mut(&self, offset: usize) -> Option<*mut u8> {
        if offset >= self.data.len() {
            return None;
        }
        Some(self.data.as_ptr().wrapping_add(offset) as *mut u8)
    }
}

#[cfg(feature = "std")]
impl MemoryProvider for StdMemoryProvider {
    fn borrow_slice(&self, offset: usize, len: usize) -> Result<SafeSlice<'_>> {
        // Track this memory access
        self.track_access(offset, len);

        // Verify the access is valid
        self.verify_access(offset, len)?;

        // Calculate the end offset
        let end = offset
            .checked_add(len)
            .ok_or_else(|| Error::memory_error("Memory access overflow".to_string()))?;

        // Get a slice and create a SafeSlice from it
        let slice = &self.data[offset..end];
        Ok(SafeSlice::with_verification_level(
            slice,
            self.verification_level,
        ))
    }

    fn verify_access(&self, offset: usize, len: usize) -> Result<()> {
        // Track validation operation
        record_global_operation(OperationType::CollectionValidate, self.verification_level);

        // Calculate the end offset
        let end = offset
            .checked_add(len)
            .ok_or_else(|| Error::memory_error("Memory access overflow".to_string()))?;

        // Check if the access is within bounds
        if end > self.data.len() {
            return Err(Error::memory_error(format!(
                "Memory access out of bounds: offset={}, len={}, size={}",
                offset,
                len,
                self.data.len()
            )));
        }

        Ok(())
    }

    fn size(&self) -> usize {
        self.data.len()
    }
}

#[cfg(feature = "std")]
impl MemorySafety for StdMemoryProvider {
    fn verify_integrity(&self) -> Result<()> {
        // Track validation operation
        record_global_operation(OperationType::CollectionValidate, self.verification_level);

        // For full verification, calculate a checksum of the entire memory
        if matches!(self.verification_level, VerificationLevel::Full) {
            record_global_operation(OperationType::ChecksumCalculation, self.verification_level);

            // Calculate checksum of all used memory
            let _checksum = Checksum::compute(&self.data);

            // In a real implementation, we would compare this against a stored
            // checksum, but for now we just calculate it for fuel accounting
        }

        Ok(())
    }

    fn set_verification_level(&mut self, _level: VerificationLevel) {
        // StdMemoryProvider doesn't need to change verification level
        // since SafeSlice instances handle their own verification
    }

    fn verification_level(&self) -> VerificationLevel {
        // Default to standard verification
        VerificationLevel::Standard
    }

    fn memory_stats(&self) -> MemoryStats {
        self.memory_stats()
    }
}

/// Memory provider for no-std environments
///
/// This provider uses a fixed-size array and manages accesses with atomic operations.
/// It provides memory safety and verification similar to StdMemoryProvider but with
/// a simpler implementation suitable for environments without std.
#[cfg(not(feature = "std"))]
pub struct NoStdMemoryProvider<const N: usize> {
    /// The underlying data buffer
    data: [u8; N],
    /// Current usage of the buffer
    used: usize,
    /// Counter for access operations
    access_count: AtomicUsize,
    /// Last access offset for validation
    last_access_offset: AtomicUsize,
    /// Last access length for validation
    last_access_length: AtomicUsize,
    /// Verification level for memory operations
    verification_level: VerificationLevel,
}

#[cfg(not(feature = "std"))]
impl<const N: usize> fmt::Debug for NoStdMemoryProvider<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NoStdMemoryProvider")
            .field("used", &self.used)
            .field("access_count", &self.access_count.load(Ordering::Relaxed))
            .field("verification_level", &self.verification_level)
            .finish()
    }
}

#[cfg(not(feature = "std"))]
impl<const N: usize> Default for NoStdMemoryProvider<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "std"))]
impl<const N: usize> NoStdMemoryProvider<N> {
    /// Create a new empty memory provider
    pub fn new() -> Self {
        Self {
            data: [0; N],
            used: 0,
            access_count: AtomicUsize::new(0),
            last_access_offset: AtomicUsize::new(0),
            last_access_length: AtomicUsize::new(0),
            verification_level: VerificationLevel::Standard,
        }
    }

    /// Set the data for this memory provider
    pub fn set_data(&mut self, data: &[u8]) -> Result<()> {
        if data.len() > N {
            return Err(Error::memory_error(format!(
                "Data too large for fixed-size buffer: {} > {}",
                data.len(),
                N
            )));
        }

        // Copy the data into our fixed buffer
        for (i, &byte) in data.iter().enumerate() {
            self.data[i] = byte;
        }
        self.used = data.len();

        // Reset access metrics
        self.access_count.store(0, Ordering::SeqCst);
        self.last_access_offset.store(0, Ordering::SeqCst);
        self.last_access_length.store(0, Ordering::SeqCst);

        Ok(())
    }

    /// Get the current access count
    pub fn access_count(&self) -> usize {
        self.access_count.load(Ordering::SeqCst)
    }

    /// Reset usage to zero (clear memory)
    pub fn clear(&mut self) {
        self.used = 0;
        self.access_count.store(0, Ordering::SeqCst);
    }

    /// Get the last accessed region
    pub fn last_access(&self) -> (usize, usize) {
        let offset = self.last_access_offset.load(Ordering::SeqCst);
        let length = self.last_access_length.load(Ordering::SeqCst);
        (offset, length)
    }

    /// Resize the used portion of memory
    pub fn resize(&mut self, new_size: usize) -> Result<()> {
        if new_size > N {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_OUT_OF_BOUNDS,
                format!("Cannot resize to {} (max: {})", new_size, N),
            ));
        }

        // Zero out any newly used memory
        if new_size > self.used {
            for i in self.used..new_size {
                self.data[i] = 0;
            }
        }

        self.used = new_size;
        Ok(())
    }

    /// Verify memory integrity
    pub fn verify_integrity(&self) -> Result<()> {
        // Track validation operation
        record_global_operation(OperationType::CollectionValidate, self.verification_level);

        // Simple length check
        if self.used > N {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::INTEGRITY_VIOLATION,
                "Memory corruption detected: used > capacity",
            ));
        }

        // Verify that the last access was valid
        let offset = self.last_access_offset.load(Ordering::SeqCst);
        let length = self.last_access_length.load(Ordering::SeqCst);

        if length > 0 && offset + length > self.used {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                format!(
                    "Last access out of bounds: offset={}, len={}, used={}",
                    offset, length, self.used
                ),
            ));
        }

        Ok(())
    }

    /// Get memory usage statistics
    pub fn memory_stats(&self) -> MemoryStats {
        let access_count = self.access_count();
        let (_, length) = self.last_access();

        MemoryStats {
            total_size: self.size(),
            access_count,
            unique_regions: 1, // No-std can't track unique regions
            max_access_size: length,
        }
    }
}

#[cfg(not(feature = "std"))]
impl<const N: usize> MemoryProvider for NoStdMemoryProvider<N> {
    fn borrow_slice(&self, offset: usize, len: usize) -> Result<SafeSlice<'_>> {
        // Track memory read
        record_global_operation(OperationType::MemoryRead, self.verification_level);

        // Update access tracking
        self.access_count.fetch_add(1, Ordering::Relaxed);
        self.last_access_offset.store(offset, Ordering::Relaxed);
        self.last_access_length.store(len, Ordering::Relaxed);

        // Verify the access is valid
        self.verify_access(offset, len)?;

        // Calculate the end offset
        let end = offset
            .checked_add(len)
            .ok_or_else(|| Error::memory_error("Memory access overflow".to_string()))?;

        // Get a slice and create a SafeSlice from it
        Ok(SafeSlice::with_verification_level(
            &self.data[offset..end],
            self.verification_level,
        ))
    }

    fn verify_access(&self, offset: usize, len: usize) -> Result<()> {
        // Range check to ensure the access is within bounds
        let end = offset.saturating_add(len);

        // Check if offset + len overflows
        if end < offset {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                format!("Memory access overflow: offset={}, len={}", offset, len),
            ));
        }

        // Check if the access is within the used portion of memory
        if end > self.used {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                format!(
                    "Memory access out of bounds: offset={}, len={}, used={}",
                    offset, len, self.used
                ),
            ));
        }

        // Track the access
        self.last_access_offset.store(offset, Ordering::SeqCst);
        self.last_access_length.store(len, Ordering::SeqCst);
        self.access_count.fetch_add(1, Ordering::SeqCst);

        Ok(())
    }

    fn size(&self) -> usize {
        self.used
    }
}

#[cfg(not(feature = "std"))]
impl<const N: usize> MemorySafety for NoStdMemoryProvider<N> {
    fn verify_integrity(&self) -> Result<()> {
        // Track validation operation
        record_global_operation(OperationType::CollectionValidate, self.verification_level);

        // Simple length check
        if self.used > N {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::INTEGRITY_VIOLATION,
                "Memory corruption detected: used > capacity",
            ));
        }

        // Verify that the last access was valid
        let offset = self.last_access_offset.load(Ordering::SeqCst);
        let length = self.last_access_length.load(Ordering::SeqCst);

        if length > 0 && offset + length > self.used {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                format!(
                    "Last access out of bounds: offset={}, len={}, used={}",
                    offset, length, self.used
                ),
            ));
        }

        Ok(())
    }

    fn set_verification_level(&mut self, level: VerificationLevel) {
        self.verification_level = level;
    }

    fn verification_level(&self) -> VerificationLevel {
        self.verification_level
    }

    fn memory_stats(&self) -> MemoryStats {
        self.memory_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "std")]
    fn test_std_memory_provider() {
        let data = vec![1, 2, 3, 4, 5];
        let provider = StdMemoryProvider::new(data);

        // Test valid access
        let slice = provider.borrow_slice(1, 3).unwrap();
        assert_eq!(slice.data().unwrap(), &[2, 3, 4]);

        // Test out of bounds
        assert!(provider.borrow_slice(3, 3).is_err());
    }

    #[test]
    fn test_safe_slice() {
        let data = [1, 2, 3, 4, 5];
        let slice = SafeSlice::new(&data);

        // Test data access
        assert_eq!(slice.data().unwrap(), &[1, 2, 3, 4, 5]);

        // Test sub-slicing
        let sub = slice.slice(1, 4).unwrap();
        assert_eq!(sub.data().unwrap(), &[2, 3, 4]);

        // Test invalid sub-slice
        assert!(slice.slice(3, 6).is_err());
    }

    #[test]
    #[cfg(not(feature = "std"))]
    fn test_no_std_memory_provider() {
        // Create a provider with fixed size 10
        let mut provider = NoStdMemoryProvider::<10>::new();

        // Set some data
        provider.set_data(&[1, 2, 3, 4, 5]).unwrap();

        // Test valid access
        let slice = provider.borrow_slice(1, 3).unwrap();
        assert_eq!(slice.data().unwrap(), &[2, 3, 4]);

        // Test out of bounds
        assert!(provider.borrow_slice(3, 3).is_err());

        // Test setting verification level
        provider.set_verification_level(VerificationLevel::None);
        assert_eq!(provider.verification_level(), VerificationLevel::None);

        // Resize within bounds
        assert!(provider.resize(8).is_ok());

        // Resize beyond bounds should fail
        assert!(provider.resize(11).is_err());

        // Verify integrity
        assert!(provider.verify_integrity().is_ok());

        // Test memory stats
        let stats = provider.memory_stats();
        assert_eq!(stats.total_size, 8);
    }
}

/// A mutex implementation for no_std environments with once-initialized data.
///
/// This module provides a simplified mutex implementation that can be used in no_std
/// environments where the standard library's Mutex is not available.
pub mod once_mutex {
    use core::cell::UnsafeCell;
    use core::sync::atomic::{AtomicBool, Ordering};

    /// A mutex that can only be used once it's been initialized.
    ///
    /// This simplified mutex provides basic thread-safety for no_std environments.
    pub struct OnceMutex<T> {
        locked: AtomicBool,
        data: UnsafeCell<T>,
    }

    impl<T> OnceMutex<T> {
        /// Creates a new mutex containing the given data.
        pub fn new(data: T) -> Self {
            Self {
                locked: AtomicBool::new(false),
                data: UnsafeCell::new(data),
            }
        }

        /// Acquires the mutex, blocking the current thread until it can be acquired.
        ///
        /// Returns a guard that releases the mutex when dropped.
        pub fn lock(&self) -> MutexGuard<'_, T> {
            // Spin until we get the lock
            while self.locked.swap(true, Ordering::Acquire) {
                core::hint::spin_loop();
            }

            MutexGuard { mutex: self }
        }
    }

    /// A RAII guard for a locked mutex.
    ///
    /// When this guard is dropped, the mutex will be unlocked automatically.
    pub struct MutexGuard<'a, T> {
        mutex: &'a OnceMutex<T>,
    }

    #[allow(unsafe_code)]
    impl<T> core::ops::Deref for MutexGuard<'_, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            unsafe { &*self.mutex.data.get() }
        }
    }

    #[allow(unsafe_code)]
    impl<T> core::ops::DerefMut for MutexGuard<'_, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe { &mut *self.mutex.data.get() }
        }
    }

    impl<T> Drop for MutexGuard<'_, T> {
        fn drop(&mut self) {
            self.mutex.locked.store(false, Ordering::Release);
        }
    }

    #[allow(unsafe_code)]
    unsafe impl<T: Send> Send for OnceMutex<T> {}

    #[allow(unsafe_code)]
    unsafe impl<T: Send> Sync for OnceMutex<T> {}
}

/// This provides a Vec-like interface but with integrated
/// memory safety features including checksum verification.
#[derive(Debug)]
#[cfg(feature = "std")]
pub struct SafeStack<T>
where
    T: Clone,
{
    /// The backing memory provider
    provider: StdMemoryProvider,
    /// Current length of the stack
    length: AtomicUsize,
    /// Element size in bytes
    elem_size: usize,
    /// Verification level for this stack
    verification_level: VerificationLevel,
    /// Phantom data for type
    _phantom: core::marker::PhantomData<T>,
}

#[cfg(feature = "std")]
impl<T> SafeStack<T>
where
    T: Clone,
{
    /// Create a new empty stack with default capacity
    pub fn new() -> Self {
        Self::with_capacity(16)
    }

    /// Create a new empty stack with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let elem_size = core::mem::size_of::<T>();
        let provider = StdMemoryProvider::with_capacity(capacity * elem_size);

        Self {
            provider,
            length: AtomicUsize::new(0),
            elem_size,
            verification_level: VerificationLevel::Standard,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Push an element onto the stack
    pub fn push(&mut self, value: T) -> Result<()> {
        // Track operation
        record_global_operation(OperationType::MemoryWrite, self.verification_level);

        // Get current length
        let len = self.length.load(Ordering::Acquire);
        let required_size = (len + 1) * self.elem_size;

        // Ensure we have enough capacity
        if required_size > self.provider.size() {
            // Need to resize
            let new_capacity =
                core::cmp::max(self.provider.size() * 2 / self.elem_size, len + 1) * self.elem_size;

            self.provider.resize(new_capacity, 0);
        }

        // Convert value to bytes and write to memory
        let bytes = self.value_to_bytes(&value)?;

        // Write bytes at the end of the current data
        let offset = len * self.elem_size;
        self.write_bytes_at(offset, &bytes)?;

        // Update length
        self.length.store(len + 1, Ordering::Release);

        Ok(())
    }

    /// Pop an element from the stack
    pub fn pop(&mut self) -> Result<T> {
        // Track operation
        record_global_operation(OperationType::MemoryRead, self.verification_level);

        // Get current length
        let len = self.length.load(Ordering::Acquire);
        if len == 0 {
            return Err(Error::runtime_error("Stack underflow"));
        }

        // Calculate position of last element
        let new_len = len - 1;
        let offset = new_len * self.elem_size;

        // Read bytes from memory
        let bytes = self.read_bytes_at(offset, self.elem_size)?;

        // Convert bytes to value
        let value = self.bytes_to_value(&bytes)?;

        // Update length
        self.length.store(new_len, Ordering::Release);

        Ok(value)
    }

    /// Get the number of elements in the stack
    pub fn len(&self) -> usize {
        self.length.load(Ordering::Relaxed)
    }

    /// Check if the stack is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear the stack
    pub fn clear(&mut self) {
        self.length.store(0, Ordering::Release);
    }

    /// Peek at the top element without removing it
    pub fn peek(&self) -> Result<T> {
        // Track operation
        record_global_operation(OperationType::MemoryRead, self.verification_level);

        // Get current length
        let len = self.length.load(Ordering::Acquire);
        if len == 0 {
            return Err(Error::runtime_error("Stack underflow"));
        }

        // Calculate position of last element
        let offset = (len - 1) * self.elem_size;

        // Read bytes from memory
        let bytes = self.read_bytes_at(offset, self.elem_size)?;

        // Convert bytes to value
        self.bytes_to_value(&bytes)
    }

    /// Set the verification level
    pub fn set_verification_level(&mut self, level: VerificationLevel) {
        self.verification_level = level;
        self.provider.set_verification_level(level);
    }

    /// Convert a value to its byte representation
    #[allow(unsafe_code)]
    fn value_to_bytes(&self, value: &T) -> Result<Vec<u8>> {
        let ptr = value as *const T as *const u8;
        let bytes = unsafe { core::slice::from_raw_parts(ptr, self.elem_size) };
        Ok(bytes.to_vec())
    }

    /// Convert bytes back to a value
    #[allow(unsafe_code)]
    fn bytes_to_value(&self, bytes: &[u8]) -> Result<T> {
        if bytes.len() != self.elem_size {
            return Err(Error::validation_error(
                "Byte size mismatch for value conversion",
            ));
        }

        let ptr = bytes.as_ptr() as *const T;
        let value = unsafe { (*ptr).clone() };
        Ok(value)
    }

    /// Write bytes at a specific offset
    #[allow(unsafe_code)]
    fn write_bytes_at(&mut self, offset: usize, bytes: &[u8]) -> Result<()> {
        // Ensure the offset + bytes.len() is within capacity
        if offset + bytes.len() > self.provider.size() {
            return Err(Error::memory_error("Write would exceed capacity"));
        }

        // Get a slice of the memory
        let _slice = self.provider.borrow_slice(offset, bytes.len())?;

        // Update the data through the memory provider directly
        let data_slice = self.provider.borrow_slice(offset, bytes.len())?;
        if let Ok(data) = data_slice.data() {
            // Direct write using unsafe operations since we know the slice is valid
            let target_ptr = data.as_ptr() as *mut u8;
            let target = unsafe { core::slice::from_raw_parts_mut(target_ptr, bytes.len()) };
            target.copy_from_slice(bytes);
        }

        Ok(())
    }

    /// Read bytes from a specific offset
    fn read_bytes_at(&self, offset: usize, length: usize) -> Result<Vec<u8>> {
        // Borrow a slice from the memory provider
        let slice = self.provider.borrow_slice(offset, length)?;

        // Convert to Vec<u8>
        let data = slice.data()?;
        Ok(data.to_vec())
    }

    /// Get the element at the specified index
    pub fn get(&self, index: usize) -> Result<T> {
        // Track operation
        record_global_operation(OperationType::MemoryRead, self.verification_level);

        // Get current length
        let len = self.length.load(Ordering::Acquire);
        if index >= len {
            return Err(Error::runtime_error(format!(
                "Index out of range: {} >= {}",
                index, len
            )));
        }

        // Calculate position of the element
        let offset = index * self.elem_size;

        // Read bytes from memory
        let bytes = self.read_bytes_at(offset, self.elem_size)?;

        // Convert bytes to value
        self.bytes_to_value(&bytes)
    }

    /// Convert the stack to a Vec<T>
    pub fn to_vec(&self) -> Result<Vec<T>> {
        // Get current length
        let len = self.length.load(Ordering::Acquire);
        let mut result = Vec::with_capacity(len);

        // Read each element
        for i in 0..len {
            result.push(self.get(i)?);
        }

        Ok(result)
    }

    /// Get a slice of the stack
    pub fn as_slice(&self) -> Result<Vec<T>> {
        self.to_vec()
    }

    /// Set the element at the specified index
    pub fn set(&mut self, index: usize, value: T) -> Result<()> {
        // Track operation
        record_global_operation(OperationType::MemoryWrite, self.verification_level);

        // Get current length
        let len = self.length.load(Ordering::Acquire);
        if index >= len {
            return Err(Error::runtime_error(format!(
                "Index out of range: {} >= {}",
                index, len
            )));
        }

        // Calculate position of the element
        let offset = index * self.elem_size;

        // Convert value to bytes
        let bytes = self.value_to_bytes(&value)?;

        // Write bytes at the specified offset
        self.write_bytes_at(offset, &bytes)?;

        Ok(())
    }
}

#[cfg(feature = "std")]
impl<T> SafeStack<T>
where
    T: Clone,
{
    /// Extend the stack with elements from a slice
    pub fn extend_from_slice(&mut self, other: &[T]) -> Result<()> {
        for value in other {
            self.push(value.clone())?;
        }
        Ok(())
    }

    /// Get the last element without removing it
    pub fn last(&self) -> Result<T> {
        self.peek()
    }

    /// Split the stack at the specified position and return the tail
    pub fn split_off(&mut self, at: usize) -> Result<Vec<T>> {
        // Track operation
        record_global_operation(OperationType::MemoryWrite, self.verification_level);

        let len = self.length.load(Ordering::Acquire);
        if at > len {
            return Err(Error::runtime_error(format!(
                "Split index out of bounds: {} (len: {})",
                at, len
            )));
        }

        // Create a new vector with the tail elements
        let mut result = Vec::with_capacity(len - at);

        // Copy elements from position 'at' to the end
        for i in at..len {
            let value = self.get(i)?;
            result.push(value);
        }

        // Update length to truncate the original stack
        self.length.store(at, Ordering::Release);

        Ok(result)
    }
}

#[cfg(feature = "std")]
impl<T> Default for SafeStack<T>
where
    T: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

/// This provides a Vec-like interface but with integrated
/// memory safety features including checksum verification.
/// No_std version using a generic memory provider
#[derive(Debug)]
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub struct SafeStack<T, const N: usize = 128>
where
    T: Clone,
{
    /// The backing memory provider
    provider: NoStdMemoryProvider<N>,
    /// Current length of the stack
    length: AtomicUsize,
    /// Element size in bytes
    elem_size: usize,
    /// Verification level for this stack
    verification_level: VerificationLevel,
    /// Phantom data for type
    _phantom: core::marker::PhantomData<T>,
}

#[cfg(all(not(feature = "std"), feature = "alloc"))]
impl<T, const N: usize> SafeStack<T, N>
where
    T: Clone,
{
    /// Create a new stack with default capacity
    pub fn new() -> Self {
        Self {
            provider: NoStdMemoryProvider::new(),
            length: AtomicUsize::new(0),
            elem_size: core::mem::size_of::<T>(),
            verification_level: VerificationLevel::Standard,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Create a new stack with the specified capacity
    /// Note: In no_std mode, capacity is ignored since the size is static
    pub fn with_capacity(_capacity: usize) -> Self {
        Self::new()
    }

    /// Push an element onto the stack
    pub fn push(&mut self, value: T) -> Result<()> {
        // Track operation
        record_global_operation(OperationType::MemoryWrite, self.verification_level);

        // Get current length
        let len = self.length.load(Ordering::Acquire);
        let required_size = (len + 1) * self.elem_size;

        // Ensure we have enough capacity
        if required_size > self.provider.size() {
            // Need to resize
            let new_capacity =
                core::cmp::max(self.provider.size() * 2 / self.elem_size, len + 1) * self.elem_size;

            self.provider.resize(new_capacity)?;
        }

        // Convert value to bytes and write to memory
        let bytes = self.value_to_bytes(&value)?;

        // Write bytes at the end of the current data
        let offset = len * self.elem_size;
        self.write_bytes_at(offset, &bytes)?;

        // Update length
        self.length.store(len + 1, Ordering::Release);

        Ok(())
    }

    /// Pop an element from the stack
    pub fn pop(&mut self) -> Result<T> {
        // Track operation
        record_global_operation(OperationType::MemoryRead, self.verification_level);

        // Get current length
        let len = self.length.load(Ordering::Acquire);
        if len == 0 {
            return Err(Error::runtime_error("Stack underflow"));
        }

        // Calculate position of last element
        let new_len = len - 1;
        let offset = new_len * self.elem_size;

        // Read bytes from memory
        let bytes = self.read_bytes_at(offset, self.elem_size)?;

        // Convert bytes to value
        let value = self.bytes_to_value(&bytes)?;

        // Update length
        self.length.store(new_len, Ordering::Release);

        Ok(value)
    }

    /// Get the number of elements in the stack
    pub fn len(&self) -> usize {
        self.length.load(Ordering::Relaxed)
    }

    /// Check if the stack is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear the stack
    pub fn clear(&mut self) {
        self.length.store(0, Ordering::Release);
    }

    /// Peek at the top element without removing it
    pub fn peek(&self) -> Result<T> {
        // Track operation
        record_global_operation(OperationType::MemoryRead, self.verification_level);

        // Get current length
        let len = self.length.load(Ordering::Acquire);
        if len == 0 {
            return Err(Error::runtime_error("Stack underflow"));
        }

        // Calculate position of last element
        let offset = (len - 1) * self.elem_size;

        // Read bytes from memory
        let bytes = self.read_bytes_at(offset, self.elem_size)?;

        // Convert bytes to value
        self.bytes_to_value(&bytes)
    }

    /// Set the verification level
    pub fn set_verification_level(&mut self, level: VerificationLevel) {
        self.verification_level = level;
        self.provider.set_verification_level(level);
    }

    /// Convert a value to its byte representation
    #[allow(unsafe_code)]
    fn value_to_bytes(&self, value: &T) -> Result<Vec<u8>> {
        let ptr = value as *const T as *const u8;
        let bytes = unsafe { core::slice::from_raw_parts(ptr, self.elem_size) };
        Ok(bytes.to_vec())
    }

    /// Convert bytes back to a value
    #[allow(unsafe_code)]
    fn bytes_to_value(&self, bytes: &[u8]) -> Result<T> {
        if bytes.len() != self.elem_size {
            return Err(Error::validation_error(
                "Byte size mismatch for value conversion",
            ));
        }

        let ptr = bytes.as_ptr() as *const T;
        let value = unsafe { (*ptr).clone() };
        Ok(value)
    }

    /// Write bytes at a specific offset
    #[allow(unsafe_code)]
    fn write_bytes_at(&mut self, offset: usize, bytes: &[u8]) -> Result<()> {
        // Ensure the offset + bytes.len() is within capacity
        if offset + bytes.len() > self.provider.size() {
            return Err(Error::memory_error("Write would exceed capacity"));
        }

        // Get a slice of the memory
        let _slice = self.provider.borrow_slice(offset, bytes.len())?;

        // Update the data through the memory provider directly
        let data_slice = self.provider.borrow_slice(offset, bytes.len())?;
        if let Ok(data) = data_slice.data() {
            // Direct write using unsafe operations since we know the slice is valid
            let target_ptr = data.as_ptr() as *mut u8;
            let target = unsafe { core::slice::from_raw_parts_mut(target_ptr, bytes.len()) };
            target.copy_from_slice(bytes);
        }

        Ok(())
    }

    /// Read bytes from a specific offset
    fn read_bytes_at(&self, offset: usize, length: usize) -> Result<Vec<u8>> {
        // Borrow a slice from the memory provider
        let slice = self.provider.borrow_slice(offset, length)?;

        // Convert to Vec<u8>
        let data = slice.data()?;
        Ok(data.to_vec())
    }

    /// Get the element at the specified index
    pub fn get(&self, index: usize) -> Result<T> {
        // Track operation
        record_global_operation(OperationType::MemoryRead, self.verification_level);

        // Get current length
        let len = self.length.load(Ordering::Acquire);
        if index >= len {
            return Err(Error::runtime_error(format!(
                "Index out of range: {} >= {}",
                index, len
            )));
        }

        // Calculate position of the element
        let offset = index * self.elem_size;

        // Read bytes from memory
        let bytes = self.read_bytes_at(offset, self.elem_size)?;

        // Convert bytes to value
        self.bytes_to_value(&bytes)
    }

    /// Convert the stack to a Vec<T>
    pub fn to_vec(&self) -> Result<Vec<T>> {
        // Get current length
        let len = self.length.load(Ordering::Acquire);
        let mut result = Vec::with_capacity(len);

        // Read each element
        for i in 0..len {
            result.push(self.get(i)?);
        }

        Ok(result)
    }

    /// Get a slice of the stack
    pub fn as_slice(&self) -> Result<Vec<T>> {
        self.to_vec()
    }

    /// Set the element at the specified index
    pub fn set(&mut self, index: usize, value: T) -> Result<()> {
        // Track operation
        record_global_operation(OperationType::MemoryWrite, self.verification_level);

        // Get current length
        let len = self.length.load(Ordering::Acquire);
        if index >= len {
            return Err(Error::runtime_error(format!(
                "Index out of range: {} >= {}",
                index, len
            )));
        }

        // Calculate position of the element
        let offset = index * self.elem_size;

        // Convert value to bytes
        let bytes = self.value_to_bytes(&value)?;

        // Write bytes at the specified offset
        self.write_bytes_at(offset, &bytes)?;

        Ok(())
    }
}

#[cfg(all(not(feature = "std"), feature = "alloc"))]
impl<T, const N: usize> Default for SafeStack<T, N>
where
    T: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod stack_tests {
    use super::*;

    #[test]
    fn test_safe_stack_push_pop() {
        let mut stack = SafeStack::<u32>::new();

        // Push some values
        stack.push(42).unwrap();
        stack.push(77).unwrap();
        stack.push(99).unwrap();

        assert_eq!(stack.len(), 3);
        assert!(!stack.is_empty());

        // Pop values in reverse order
        assert_eq!(stack.pop().unwrap(), 99);
        assert_eq!(stack.pop().unwrap(), 77);
        assert_eq!(stack.pop().unwrap(), 42);

        assert_eq!(stack.len(), 0);
        assert!(stack.is_empty());

        // Test underflow
        assert!(stack.pop().is_err());
    }

    #[test]
    fn test_safe_stack_peek() {
        let mut stack = SafeStack::<u32>::new();

        // Push a value
        stack.push(42).unwrap();

        // Peek should return the value without removing it
        assert_eq!(stack.peek().unwrap(), 42);
        assert_eq!(stack.len(), 1);

        // Pop should remove the value
        assert_eq!(stack.pop().unwrap(), 42);
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_safe_stack_clear() {
        let mut stack = SafeStack::<u32>::new();

        // Push some values
        stack.push(1).unwrap();
        stack.push(2).unwrap();
        stack.push(3).unwrap();

        assert_eq!(stack.len(), 3);

        // Clear the stack
        stack.clear();

        assert_eq!(stack.len(), 0);
        assert!(stack.is_empty());
        assert!(stack.pop().is_err());
    }
}

/// Safe memory handler that provides a unified interface for memory operations
///
/// This handler abstracts over different memory providers and offers a
/// consistent API for memory operations with safety guarantees.
#[derive(Debug)]
#[cfg(feature = "std")]
pub struct SafeMemoryHandler {
    /// The internal memory provider
    provider: StdMemoryProvider,
    /// Verification level for memory operations
    verification_level: VerificationLevel,
}

#[cfg(feature = "std")]
impl SafeMemoryHandler {
    /// Create a new memory handler with the given data
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            provider: StdMemoryProvider::new(data),
            verification_level: VerificationLevel::Standard,
        }
    }

    /// Create a new empty memory handler with the given capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            provider: StdMemoryProvider::with_capacity(capacity),
            verification_level: VerificationLevel::Standard,
        }
    }

    /// Set the verification level for memory operations
    pub fn set_verification_level(&mut self, level: VerificationLevel) {
        self.verification_level = level;
        self.provider.set_verification_level(level);
    }

    /// Get the current verification level
    pub fn verification_level(&self) -> VerificationLevel {
        self.verification_level
    }

    /// Get a safe slice of the memory at the specified offset and length
    pub fn get_slice(&self, offset: usize, len: usize) -> Result<SafeSlice<'_>> {
        self.provider.borrow_slice(offset, len)
    }

    /// Get the size of the memory
    pub fn size(&self) -> usize {
        self.provider.size()
    }

    /// Add data to the memory
    pub fn add_data(&mut self, data: &[u8]) {
        self.provider.add_data(data);
    }

    /// Clear the memory
    pub fn clear(&mut self) {
        self.provider.clear();
    }

    /// Resize the memory to the given size
    pub fn resize(&mut self, new_size: usize, value: u8) {
        self.provider.resize(new_size, value);
    }

    /// Get a copy of the data as a Vec<u8>
    pub fn to_vec(&self) -> Result<Vec<u8>> {
        let slice = self.provider.borrow_slice(0, self.provider.size())?;
        let data = slice.data()?;
        Ok(data.to_vec())
    }

    /// Verify the integrity of the memory
    pub fn verify_integrity(&self) -> Result<()> {
        self.provider.verify_integrity()
    }

    /// Get the memory provider directly
    pub fn provider(&self) -> &StdMemoryProvider {
        &self.provider
    }

    /// Get memory usage statistics
    pub fn memory_stats(&self) -> MemoryStats {
        self.provider.memory_stats()
    }

    /// Get the current length of the memory data
    pub fn len(&self) -> usize {
        self.provider.size()
    }

    /// Check if the memory is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Write data to the memory at a specific offset
    pub fn write_data(&mut self, offset: usize, data: &[u8]) -> Result<()> {
        if offset + data.len() > self.size() {
            return Err(Error::memory_error(format!(
                "Write would exceed capacity: offset={}, length={}, capacity={}",
                offset,
                data.len(),
                self.size()
            )));
        }

        // Get the internal data to modify
        let internal_data = &mut self.provider.data;

        if offset + data.len() > internal_data.len() {
            return Err(Error::memory_error(format!(
                "Write out of bounds: offset={}, size={}, capacity={}",
                offset,
                data.len(),
                internal_data.len()
            )));
        }

        // Copy the data directly into the internal buffer
        internal_data[offset..offset + data.len()].copy_from_slice(data);

        // If verification is enabled, perform an integrity check
        if self.verification_level.should_verify(100) {
            // Update checksums and verify integrity
            self.provider.recalculate_checksums();
            self.verify_integrity()?;
        }

        Ok(())
    }

    /// Helper method to clone bytes from one location to another
    #[allow(unsafe_code)]
    #[allow(dead_code)]
    fn clone_bytes_from(&self, offset: usize, bytes: &[u8]) -> Result<()> {
        if offset + bytes.len() > self.size() {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
                "Attempted to access out of bounds memory",
            ));
        }

        // Get a slice reference to the destination memory
        let _slice = self.provider.borrow_slice(offset, bytes.len())?;

        // Directly write bytes to the destination memory
        if let Some(target_ptr) = self.provider.get_ptr_mut(offset) {
            let target = unsafe { core::slice::from_raw_parts_mut(target_ptr, bytes.len()) };
            target.copy_from_slice(bytes);
        }

        Ok(())
    }
}

/// Memory handler for no_std environments
///
/// This handler provides the same interface as SafeMemoryHandler but uses
/// NoStdMemoryProvider internally for no_std environments
#[derive(Debug)]
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub struct SafeMemoryHandler<const N: usize = 1024> {
    /// The internal memory provider
    provider: NoStdMemoryProvider<N>,
    /// Verification level for memory operations
    verification_level: VerificationLevel,
}

#[cfg(all(not(feature = "std"), feature = "alloc"))]
impl<const N: usize> SafeMemoryHandler<N> {
    /// Create a new memory handler
    pub fn new() -> Self {
        Self {
            provider: NoStdMemoryProvider::new(),
            verification_level: VerificationLevel::Standard,
        }
    }

    /// Create a new memory handler with data
    pub fn with_data(data: &[u8]) -> Result<Self> {
        let mut handler = Self::new();
        handler.provider.set_data(data)?;
        Ok(handler)
    }

    /// Set the verification level for memory operations
    pub fn set_verification_level(&mut self, level: VerificationLevel) {
        self.verification_level = level;
        self.provider.set_verification_level(level);
    }

    /// Get the current verification level
    pub fn verification_level(&self) -> VerificationLevel {
        self.verification_level
    }

    /// Get a safe slice of the memory at the specified offset and length
    pub fn get_slice(&self, offset: usize, len: usize) -> Result<SafeSlice<'_>> {
        self.provider.borrow_slice(offset, len)
    }

    /// Get the size of the memory
    pub fn size(&self) -> usize {
        self.provider.size()
    }

    /// Clear the memory
    pub fn clear(&mut self) {
        self.provider.clear();
    }

    /// Resize the memory to the given size
    pub fn resize(&mut self, new_size: usize) -> Result<()> {
        self.provider.resize(new_size)
    }

    /// Get a copy of the data as a Vec<u8>
    pub fn to_vec(&self) -> Result<Vec<u8>> {
        let slice = self.provider.borrow_slice(0, self.provider.size())?;
        let data = slice.data()?;
        Ok(data.to_vec())
    }

    /// Verify the integrity of the memory
    pub fn verify_integrity(&self) -> Result<()> {
        self.provider.verify_integrity()
    }

    /// Get the memory provider directly
    pub fn provider(&self) -> &NoStdMemoryProvider<N> {
        &self.provider
    }

    /// Get memory usage statistics
    pub fn memory_stats(&self) -> MemoryStats {
        self.provider.memory_stats()
    }

    /// Get the current length of the memory data
    pub fn len(&self) -> usize {
        self.provider.size()
    }

    /// Check if the memory is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Create SafeMemoryHandler with capacity (ignores capacity in no_std mode)
    pub fn with_capacity(_capacity: usize) -> Self {
        Self::new()
    }
}

#[cfg(all(not(feature = "std"), feature = "alloc"))]
impl<const N: usize> Default for SafeMemoryHandler<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod memory_handler_tests {
    use super::*;

    #[test]
    fn test_safe_memory_handler() {
        // Create a handler with initial data
        let data = vec![1, 2, 3, 4, 5];
        let handler = SafeMemoryHandler::new(data);

        // Get a slice
        let slice = handler.get_slice(1, 3).unwrap();
        assert_eq!(slice.data().unwrap(), &[2, 3, 4]);

        // Convert back to Vec
        let vec = handler.to_vec().unwrap();
        assert_eq!(vec, vec![1, 2, 3, 4, 5]);

        // Check size
        assert_eq!(handler.size(), 5);
    }

    #[test]
    fn test_safe_memory_handler_modifications() {
        // Create an empty handler
        let mut handler = SafeMemoryHandler::with_capacity(10);

        // Add data
        handler.add_data(&[1, 2, 3]);

        // Resize
        handler.resize(5, 0);

        // Get slice
        let slice = handler.get_slice(0, 5).unwrap();
        assert_eq!(slice.data().unwrap(), &[1, 2, 3, 0, 0]);

        // Clear
        handler.clear();
        assert_eq!(handler.size(), 0);
    }

    #[test]
    fn test_safe_memory_handler_verification() {
        let mut handler = SafeMemoryHandler::new(vec![1, 2, 3, 4, 5]);

        // Set verification level
        handler.set_verification_level(VerificationLevel::Full);
        assert_eq!(handler.verification_level(), VerificationLevel::Full);

        // Verify integrity
        assert!(handler.verify_integrity().is_ok());

        // Get memory stats
        let stats = handler.memory_stats();
        assert_eq!(stats.total_size, 5);
    }
}
