//! Formal verification for the error handling system using Kani.
//!
//! This module contains proofs that verify core properties of the error handling system.
//! These proofs only run with `cargo kani --features kani`.

#[cfg(feature = "kani")]
pub mod kani_verification {
    use crate::source::ErrorSource;
    use crate::{Error, Result, ResultExt};
    use core::fmt::{self, Debug, Display};

    // A simple test error for verification
    #[derive(Debug, Clone)]
    struct VerifyError(&'static str);

    impl Display for VerifyError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "VerifyError: {}", self.0)
        }
    }

    impl ErrorSource for VerifyError {
        #[cfg(feature = "std")]
        fn source(&self) -> Option<&(dyn ErrorSource + 'static)> {
            None
        }
    }

    /// Verify that creating and displaying an error works correctly
    #[cfg_attr(feature = "kani", kani_verifier::proof)]
    pub fn verify_error_creation() {
        let error = Error::new(VerifyError("verification test"));
        let error_str = format!("{}", error);

        // Verify display formatting
        assert!(error_str.contains("VerifyError: verification test"));
    }

    /// Verify that error context chaining works correctly
    #[cfg_attr(feature = "kani", kani_verifier::proof)]
    pub fn verify_error_context() {
        let result: core::result::Result<(), VerifyError> = Err(VerifyError("base error"));
        let with_context = result.context("operation failed");

        assert!(with_context.is_err());
        let error = with_context.unwrap_err();
        let error_str = format!("{}", error);

        // Verify both context and original error appear in display string
        assert!(error_str.contains("operation failed"));
        assert!(error_str.contains("base error"));
    }

    /// Verify that multiple contexts chain correctly
    #[cfg_attr(feature = "kani", kani_verifier::proof)]
    pub fn verify_multiple_contexts() {
        let result: Result<()> = Err(Error::new(VerifyError("original error")))
            .context("first context")
            .context("second context");

        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_str = format!("{}", error);

        // All context levels should be present
        assert!(error_str.contains("second context"));
        assert!(error_str.contains("first context"));
        assert!(error_str.contains("original error"));
    }

    /// Verify that factory methods create the correct error types
    #[cfg_attr(feature = "kani", kani_verifier::proof)]
    pub fn verify_factory_methods() {
        let div_error = Error::division_by_zero();
        assert!(format!("{}", div_error) == "Division by zero");

        let memory_error = Error::memory_access_out_of_bounds(1000, 8);
        assert!(
            format!("{}", memory_error) == "Memory access out of bounds: address 1000, length 8"
        );
    }

    /// Verify that Error::from works correctly
    #[cfg_attr(feature = "kani", kani_verifier::proof)]
    pub fn verify_error_from() {
        let original_error = VerifyError("source error");
        let error = Error::from(original_error);

        assert!(format!("{}", error) == "VerifyError: source error");
    }

    /// Verify that Result works correctly with different error types
    #[cfg_attr(feature = "kani", kani_verifier::proof)]
    pub fn verify_result_type() {
        // Test with the default error type
        let result1: Result<i32> = Ok(42);
        assert_eq!(result1.unwrap(), 42);

        let result2: Result<i32> = Err(Error::division_by_zero());
        assert!(result2.is_err());

        // Test with a custom error type
        let result3: Result<i32, VerifyError> = Ok(42);
        assert_eq!(result3.unwrap(), 42);

        let result4: Result<i32, VerifyError> = Err(VerifyError("custom error"));
        assert!(result4.is_err());
    }
}

// Include the verification module in the main library when kani feature is enabled
#[cfg(feature = "kani")]
pub use kani_verification::*;
