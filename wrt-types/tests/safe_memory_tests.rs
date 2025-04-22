//! Tests for SafeMemory implementations
//!
//! This file contains comprehensive tests for the SafeMemory module
//! and its providers (StdMemoryProvider and NoStdMemoryProvider).

extern crate wrt_types;

use wrt_error::Error;
use wrt_types::{
    safe_memory::{MemoryProvider, MemorySafety, SafeSlice, StdMemoryProvider},
    verification::VerificationLevel,
};

#[test]
fn test_std_memory_provider() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let provider = StdMemoryProvider::new(data);

    // Test basic properties
    assert_eq!(provider.size(), 8);

    // Test slice borrowing
    let slice = provider.borrow_slice(2, 4).unwrap();
    assert_eq!(slice.len(), 4);
    assert_eq!(slice.data().unwrap(), &[3, 4, 5, 6]);

    // Test access tracking
    let stats = provider.memory_stats();
    assert_eq!(stats.access_count, 1);
}

#[test]
fn test_std_memory_provider_out_of_bounds() {
    let data = vec![1, 2, 3, 4, 5];
    let provider = StdMemoryProvider::new(data);

    // Test out of bounds
    let result = provider.borrow_slice(3, 3);
    assert!(result.is_err());

    // Verify error is correct type
    if let Err(e) = result {
        assert!(e.to_string().contains("out of bounds"));
    }
}

#[test]
fn test_std_memory_provider_operations() {
    let mut provider = StdMemoryProvider::with_capacity(10);

    // Add data
    provider.add_data(&[1, 2, 3, 4]);
    assert_eq!(provider.size(), 4);

    // Borrow slice
    let slice = provider.borrow_slice(1, 2).unwrap();
    assert_eq!(slice.data().unwrap(), &[2, 3]);

    // Resize with more data
    provider.resize(8, 0);
    assert_eq!(provider.size(), 8);

    // Clear
    provider.clear();
    assert_eq!(provider.size(), 0);

    // After clearing, the access count should be reset
    let stats = provider.memory_stats();
    assert_eq!(stats.access_count, 0);
}

#[test]
fn test_std_memory_provider_integrity() {
    let mut provider = StdMemoryProvider::with_capacity(1024);
    provider.add_data(&[1, 2, 3, 4, 5, 6, 7, 8]);

    // Access a few slices
    provider.borrow_slice(0, 4).unwrap();
    provider.borrow_slice(2, 4).unwrap();
    provider.borrow_slice(4, 4).unwrap();

    // Check integrity
    assert!(provider.verify_integrity().is_ok());

    // Get stats
    let stats = provider.memory_stats();
    assert_eq!(stats.access_count, 3);
    assert!(stats.unique_regions > 0);
}

// NoStd memory provider tests are only included when not using std feature
#[cfg(not(feature = "std"))]
mod nostd_tests {
    use super::*;
    use wrt_types::safe_memory::NoStdMemoryProvider;

    #[test]
    fn test_nostd_memory_provider() {
        let mut provider = NoStdMemoryProvider::<16>::new();

        // Set data
        provider.set_data(&[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
        assert_eq!(provider.size(), 8);

        // Borrow slice
        let slice = provider.borrow_slice(2, 4).unwrap();
        assert_eq!(slice.data().unwrap(), &[3, 4, 5, 6]);

        // Check access count
        assert_eq!(provider.access_count(), 1);

        // Check last access
        let (offset, length) = provider.last_access();
        assert_eq!(offset, 2);
        assert_eq!(length, 4);
    }

    #[test]
    fn test_nostd_memory_provider_out_of_bounds() {
        let mut provider = NoStdMemoryProvider::<8>::new();
        provider.set_data(&[1, 2, 3, 4, 5]).unwrap();

        // Test out of bounds
        let result = provider.borrow_slice(3, 3);
        assert!(result.is_err());

        // Verify error is correct type
        if let Err(e) = result {
            assert!(e.to_string().contains("out of bounds"));
        }
    }

    #[test]
    fn test_nostd_memory_provider_operations() {
        let mut provider = NoStdMemoryProvider::<16>::new();

        // Set data
        provider.set_data(&[1, 2, 3, 4]).unwrap();
        assert_eq!(provider.size(), 4);

        // Resize
        provider.resize(6).unwrap();
        assert_eq!(provider.size(), 6);

        // Check that new memory is zeroed
        let slice = provider.borrow_slice(4, 2).unwrap();
        assert_eq!(slice.data().unwrap(), &[0, 0]);

        // Clear
        provider.clear();
        assert_eq!(provider.size(), 0);
    }

    #[test]
    fn test_nostd_memory_provider_integrity() {
        let mut provider = NoStdMemoryProvider::<16>::new();
        provider.set_data(&[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();

        // Access a slice
        provider.borrow_slice(2, 4).unwrap();

        // Check integrity
        assert!(provider.verify_integrity().is_ok());
    }

    #[test]
    fn test_nostd_memory_safety_trait() {
        let mut provider = NoStdMemoryProvider::<16>::new();
        provider.set_data(&[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();

        // Get statistics
        let stats = provider.memory_stats();
        assert_eq!(stats.total_size, 8);
        assert_eq!(stats.access_count, 0); // No accesses yet

        // Check verification level
        assert_eq!(provider.verification_level(), VerificationLevel::Standard);

        // Verify integrity
        assert!(provider.verify_integrity().is_ok());

        // Make some accesses
        provider.borrow_slice(0, 2).unwrap();
        provider.borrow_slice(4, 4).unwrap();

        // Check stats again
        let stats = provider.memory_stats();
        assert_eq!(stats.access_count, 2);
        assert_eq!(stats.max_access_size, 4);
    }
}

#[test]
fn test_safe_slice_verification_levels() {
    let data = &[1, 2, 3, 4, 5, 6, 7, 8];

    // Create safe slices with different verification levels
    let slice_none = SafeSlice::with_verification_level(data, VerificationLevel::None);
    let slice_sampling = SafeSlice::with_verification_level(data, VerificationLevel::Sampling);
    let slice_standard = SafeSlice::with_verification_level(data, VerificationLevel::Standard);
    let slice_full = SafeSlice::with_verification_level(data, VerificationLevel::Full);

    // All should provide correct data access
    assert_eq!(slice_none.data().unwrap(), data);
    assert_eq!(slice_sampling.data().unwrap(), data);
    assert_eq!(slice_standard.data().unwrap(), data);
    assert_eq!(slice_full.data().unwrap(), data);

    // All should have correct length
    assert_eq!(slice_none.len(), 8);
    assert_eq!(slice_sampling.len(), 8);
    assert_eq!(slice_standard.len(), 8);
    assert_eq!(slice_full.len(), 8);
}

#[test]
fn test_memory_safety_trait() {
    // Create providers implementing MemorySafety
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let provider = StdMemoryProvider::new(data);

    // Get statistics
    let stats = provider.memory_stats();
    assert_eq!(stats.total_size, 8);
    assert_eq!(stats.access_count, 0); // No accesses yet

    // Check verification level
    assert_eq!(provider.verification_level(), VerificationLevel::Standard);

    // Verify integrity
    assert!(provider.verify_integrity().is_ok());

    // Make some accesses
    provider.borrow_slice(0, 4).unwrap();
    provider.borrow_slice(4, 4).unwrap();

    // Check stats again
    let stats = provider.memory_stats();
    assert_eq!(stats.access_count, 2);
    assert!(stats.unique_regions > 0);
    assert_eq!(stats.max_access_size, 4);
}

#[test]
fn test_safe_slice_sub_slicing() {
    let data = &[1, 2, 3, 4, 5, 6, 7, 8];
    let slice = SafeSlice::new(data);

    // Create a sub-slice
    let sub_slice = slice.slice(2, 6).unwrap();
    assert_eq!(sub_slice.data().unwrap(), &[3, 4, 5, 6]);
    assert_eq!(sub_slice.len(), 4);

    // Try invalid sub-slice
    let invalid_slice = slice.slice(6, 10);
    assert!(invalid_slice.is_err());
}

#[cfg(feature = "std")]
#[test]
fn test_generic_memory_provider() {
    // Function that works with any memory provider
    fn access_memory<P: MemoryProvider>(provider: &P) -> Result<Vec<u8>, Error> {
        let slice = provider.borrow_slice(2, 4)?;
        let data = slice.data()?;
        Ok(data.to_vec())
    }

    // Test with StdMemoryProvider
    let provider = StdMemoryProvider::new(vec![1, 2, 3, 4, 5, 6, 7, 8]);
    let result = access_memory(&provider).unwrap();
    assert_eq!(result, vec![3, 4, 5, 6]);
}

#[cfg(feature = "std")]
#[test]
fn test_generic_memory_safety() {
    // Generic function that works with any MemorySafety implementation
    fn verify_and_access<M: MemoryProvider + MemorySafety>(provider: &M) -> Result<(), Error> {
        // Verify integrity
        provider.verify_integrity()?;

        // Get memory stats before
        let before = provider.memory_stats();

        // Make an access
        let slice = provider.borrow_slice(2, 4)?;
        assert_eq!(slice.len(), 4);

        // Get memory stats after
        let after = provider.memory_stats();

        // Verify counts increased
        assert!(after.access_count > before.access_count);

        Ok(())
    }

    // Test with StdMemoryProvider
    let provider = StdMemoryProvider::new(vec![1, 2, 3, 4, 5, 6, 7, 8]);
    assert!(verify_and_access(&provider).is_ok());
}

#[cfg(test)]
mod tests {
    use std::vec;

    use wrt_types::safe_memory::{MemoryProvider, MemorySafety, SafeSlice, StdMemoryProvider};
    use wrt_types::verification::VerificationLevel;

    #[test]
    fn test_safe_slice_creation() {
        let data = vec![1, 2, 3, 4, 5];
        let slice = SafeSlice::new(&data);

        assert_eq!(slice.len(), 5);
        assert!(!slice.is_empty());

        // Access the data and verify it matches
        let accessed_data = slice.data().unwrap();
        assert_eq!(accessed_data, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_std_memory_provider() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let provider = StdMemoryProvider::new(data);

        // Borrow a slice and verify its contents
        let slice = provider.borrow_slice(2, 3).unwrap();
        assert_eq!(slice.len(), 3);
        assert_eq!(slice.data().unwrap(), &[3, 4, 5]);

        // Check access tracking
        let stats = provider.memory_stats();
        assert_eq!(stats.access_count, 1);
    }

    #[test]
    fn test_safe_slice_with_verification_levels() {
        let data = vec![1, 2, 3, 4, 5];

        // Test with standard verification level (default)
        let slice = SafeSlice::new(&data);
        assert_eq!(slice.verification_level(), VerificationLevel::Standard);

        // Test with full verification level
        let full_slice = SafeSlice::with_verification_level(&data, VerificationLevel::Full);
        assert_eq!(full_slice.verification_level(), VerificationLevel::Full);

        // Test with no verification
        let no_verify_slice = SafeSlice::with_verification_level(&data, VerificationLevel::None);
        assert_eq!(
            no_verify_slice.verification_level(),
            VerificationLevel::None
        );

        // All should still return correct data
        assert_eq!(slice.data().unwrap(), &[1, 2, 3, 4, 5]);
        assert_eq!(full_slice.data().unwrap(), &[1, 2, 3, 4, 5]);
        assert_eq!(no_verify_slice.data().unwrap(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_out_of_bounds_access() {
        let data = vec![1, 2, 3, 4, 5];
        let provider = StdMemoryProvider::new(data);

        // Try to access beyond the end of data
        let result = provider.borrow_slice(3, 10);
        assert!(result.is_err());

        // Check access tracking still recorded the attempt
        let stats = provider.memory_stats();
        assert_eq!(stats.access_count, 1);
    }

    #[test]
    fn test_slice_sub_slicing() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let provider = StdMemoryProvider::new(data);

        // Borrow a slice
        let slice = provider.borrow_slice(1, 6).unwrap();
        assert_eq!(slice.data().unwrap(), &[2, 3, 4, 5, 6, 7]);

        // Create a sub-slice
        let sub_slice = slice.slice(2, 4).unwrap();
        assert_eq!(sub_slice.data().unwrap(), &[4, 5]);

        // Check access tracking
        let stats = provider.memory_stats();
        assert!(stats.access_count >= 2);
        assert!(stats.unique_regions >= 1);
    }

    #[test]
    fn test_safe_slice_integrity_verification() {
        let data = vec![1, 2, 3, 4, 5];
        let slice = SafeSlice::new(&data);

        // Verify integrity works
        assert!(slice.verify_integrity().is_ok());

        // We can't easily test corruption detection since SafeSlice
        // has immutable access to the data, but we can at least
        // ensure the verification functions don't panic
        let _ = slice.verify_integrity_with_importance(200);
    }

    #[test]
    fn test_memory_provider_safety() {
        let data = vec![1, 2, 3, 4, 5];
        let mut provider = StdMemoryProvider::new(data);

        // Test verification level setting
        provider.set_verification_level(VerificationLevel::Full);
        assert_eq!(provider.verification_level(), VerificationLevel::Full);

        // Test integrity verification
        assert!(provider.verify_integrity().is_ok());

        // Test stats
        let stats = provider.memory_stats();
        assert_eq!(stats.total_size, 5);
    }
}
