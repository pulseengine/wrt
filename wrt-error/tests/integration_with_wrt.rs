//! Tests the integration of wrt-error with the main wrt crate.
//!
//! This file is a placeholder for now since we haven't yet modified the wrt crate
//! to use wrt-error directly.

// These tests require the alloc feature to be enabled since they use Error
#[cfg(all(test, feature = "alloc"))]
mod tests {
    use wrt_error::Error;

    #[test]
    fn test_error_creation() {
        // This is a simple test to verify the wrt-error crate works properly
        // Future tests could use the wrt crate directly once it's updated to use wrt-error
        let error = Error::division_by_zero();
        assert_eq!(format!("{}", error), "Division by zero");
    }

    #[test]
    fn test_memory_error_creation() {
        let error = Error::memory_access_out_of_bounds(0x1000, 8);
        assert_eq!(
            format!("{}", error),
            "Memory access out of bounds: address 4096, length 8"
        );
    }
}
