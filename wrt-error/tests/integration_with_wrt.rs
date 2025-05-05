//! Tests the integration of wrt-error with the main wrt crate.
//!
//! This file tests the error handling functionality in the context of how it would be used
//! in the main wrt crate.

#[cfg(test)]
mod tests {
    use wrt_error::kinds;
    use wrt_error::{Error, Result};

    #[test]
    fn test_error_conversion() {
        // Create an error from a kind
        let div_error = kinds::division_by_zero_error();
        let error = Error::from(div_error);
        // Assert that the error message contains the expected text
        assert!(format!("{}", error).contains("Division by zero"));

        // Create an error from a memory access error
        let mem_error = kinds::MemoryAccessOutOfBoundsError {
            address: 0x1000,
            length: 8,
        };
        let error = Error::from(mem_error);
        assert!(error.is_memory_error());

        // Create an error using a factory method
        let error = Error::memory_error("Memory access failed");
        assert!(error.is_memory_error());
    }

    #[test]
    fn test_result_operations() {
        // Create a success result
        let success: Result<i32> = Ok(42);
        assert_eq!(success.unwrap(), 42);

        // Create an error result
        let failure: Result<i32> = Err(Error::validation_error("Validation failed"));
        assert!(failure.is_err());
        assert!(failure.err().unwrap().is_validation_error());
    }
}
