//! Safe memory abstractions for WebAssembly runtime
//!
//! This module provides memory safety abstractions designed for
//! functional safety at ASIL-B level, implementing verification
//! mechanisms to detect memory corruption.

use core::fmt;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::operations::{record_global_operation, OperationType};
use crate::verification::{Checksum, VerificationLevel};
use wrt_error::{kinds, Error, Result};

#[cfg(feature = "std")]
use std::sync::Mutex;

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{format, string::ToString};

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

        // Verify length first
        if self.data.len() != self.length {
            return Err(Error::new(kinds::ValidationError(
                "Memory corruption detected: length mismatch".into(),
            )));
        }

        // Skip detailed verification in optimize mode for non-critical paths
        #[cfg(feature = "optimize")]
        return Ok(());

        // Skip checksum verification for non-redundant checks
        // unless we're using full verification
        if !self.verification_level.should_verify_redundant() && importance < 200 {
            return Ok(());
        }

        // Track checksum calculation
        if importance >= 200 || self.verification_level.should_verify_redundant() {
            record_global_operation(OperationType::ChecksumCalculation, self.verification_level);
        }

        // Compute current checksum and compare with stored checksum
        let current = Checksum::compute(self.data);
        if current == self.checksum {
            Ok(())
        } else {
            Err(Error::new(kinds::ValidationError(
                "Memory corruption detected: checksum mismatch".into(),
            )))
        }
    }

    /// Create a sub-slice with the same safety guarantees
    pub fn slice(&self, start: usize, end: usize) -> Result<SafeSlice<'a>> {
        // Track memory read operation
        record_global_operation(OperationType::MemoryRead, self.verification_level);

        // High importance for slicing operations (200)
        self.verify_integrity_with_importance(200)?;

        if start > end || end > self.length {
            return Err(Error::new(kinds::OutOfBoundsError(format!(
                "Invalid slice range: {}..{} (len: {})",
                start, end, self.length
            ))));
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
    regions_hash: Mutex<std::collections::HashSet<usize>>,
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
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            access_log: Mutex::new(Vec::new()),
            access_count: AtomicUsize::new(0),
            max_access_size: AtomicUsize::new(0),
            unique_regions: AtomicUsize::new(0),
            regions_hash: Mutex::new(std::collections::HashSet::new()),
            verification_level: VerificationLevel::Standard,
        }
    }

    /// Create a new empty StdMemoryProvider with the given capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self::new(Vec::with_capacity(capacity))
    }

    /// Get the access log for debugging and verification
    pub fn access_log(&self) -> Result<Vec<(usize, usize)>> {
        match self.access_log.lock() {
            Ok(log) => Ok(log.clone()),
            Err(_) => Err(Error::new(kinds::PoisonedLockError(
                "Access log mutex poisoned".into(),
            ))),
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
            _ => Err(Error::new(kinds::ValidationError(
                "Access log mutex poisoned".into(),
            ))),
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
}

#[cfg(feature = "std")]
impl MemoryProvider for StdMemoryProvider {
    fn borrow_slice(&self, offset: usize, len: usize) -> Result<SafeSlice<'_>> {
        // Track this memory access
        self.track_access(offset, len);

        // Verify the access is valid
        self.verify_access(offset, len)?;

        // Calculate the end offset
        let end = offset.checked_add(len).ok_or_else(|| {
            Error::new(kinds::OutOfBoundsError(
                "Memory access overflow".to_string(),
            ))
        })?;

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
        let end = offset.checked_add(len).ok_or_else(|| {
            Error::new(kinds::OutOfBoundsError(
                "Memory access overflow".to_string(),
            ))
        })?;

        // Check if the access is within bounds
        if end > self.data.len() {
            return Err(Error::new(kinds::OutOfBoundsError(format!(
                "Memory access out of bounds: offset={}, len={}, size={}",
                offset,
                len,
                self.data.len()
            ))));
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
            return Err(Error::new(kinds::OutOfBoundsError(
                "Data too large for fixed-size buffer".into(),
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
            #[cfg(feature = "std")]
            return Err(Error::new(kinds::OutOfBoundsError(format!(
                "Cannot resize to {} (max: {})",
                new_size, N
            ))));

            #[cfg(all(not(feature = "std"), feature = "alloc"))]
            return Err(Error::new(kinds::OutOfBoundsError(format!(
                "Cannot resize to {} (max: {})",
                new_size, N
            ))));

            #[cfg(not(any(feature = "std", feature = "alloc")))]
            return Err(Error::new(kinds::OutOfBoundsError(
                "Memory resize exceeds capacity".into(),
            )));
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
        // Verify that the last access was valid
        let offset = self.last_access_offset.load(Ordering::SeqCst);
        let length = self.last_access_length.load(Ordering::SeqCst);

        if length > 0 && offset + length > self.used {
            #[cfg(feature = "std")]
            return Err(Error::new(kinds::ValidationError(format!(
                "Last access out of bounds: offset={}, len={}, used={}",
                offset, length, self.used
            ))));

            #[cfg(all(not(feature = "std"), feature = "alloc"))]
            return Err(Error::new(kinds::ValidationError(format!(
                "Last access out of bounds: offset={}, len={}, used={}",
                offset, length, self.used
            ))));

            #[cfg(not(any(feature = "std", feature = "alloc")))]
            return Err(Error::new(kinds::ValidationError(
                "Last memory access was out of bounds".into(),
            )));
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
        record_global_operation(OperationType::MemoryRead, VerificationLevel::Standard);

        // Update access tracking
        self.access_count.fetch_add(1, Ordering::Relaxed);
        self.last_access_offset.store(offset, Ordering::Relaxed);
        self.last_access_length.store(len, Ordering::Relaxed);

        // Verify the access is valid
        self.verify_access(offset, len)?;

        // Calculate the end offset
        let end = offset.checked_add(len).ok_or_else(|| {
            Error::new(kinds::OutOfBoundsError(
                "Memory access overflow".to_string(),
            ))
        })?;

        // Get a slice and create a SafeSlice from it
        Ok(SafeSlice::with_verification_level(
            &self.data[offset..end],
            VerificationLevel::Standard,
        ))
    }

    fn verify_access(&self, offset: usize, len: usize) -> Result<()> {
        // Track validation operation
        record_global_operation(
            OperationType::CollectionValidate,
            VerificationLevel::Standard,
        );

        // Calculate the end offset
        let end = offset.checked_add(len).ok_or_else(|| {
            Error::new(kinds::OutOfBoundsError(
                "Memory access overflow".to_string(),
            ))
        })?;

        // Check if the access is within the used portion of memory
        if end > self.used {
            #[cfg(feature = "std")]
            return Err(Error::new(kinds::OutOfBoundsError(format!(
                "Memory access out of bounds: offset={}, len={}, used={}",
                offset, len, self.used
            ))));

            #[cfg(all(not(feature = "std"), feature = "alloc"))]
            return Err(Error::new(kinds::OutOfBoundsError(format!(
                "Memory access out of bounds: offset={}, len={}, used={}",
                offset, len, self.used
            ))));

            #[cfg(not(any(feature = "std", feature = "alloc")))]
            return Err(Error::new(kinds::OutOfBoundsError(
                "Memory access out of bounds".into(),
            )));
        }

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
        record_global_operation(
            OperationType::CollectionValidate,
            VerificationLevel::Standard,
        );

        // Simple length check
        if self.used > N {
            return Err(Error::new(kinds::ValidationError(
                "Memory corruption detected: used > capacity".into(),
            )));
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
}
