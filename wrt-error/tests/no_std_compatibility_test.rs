//! Test no_std compatibility for wrt-error
//!
//! This file validates that the wrt-error crate works correctly in no_std environments.

// For testing in a no_std environment
#![cfg_attr(not(feature = "std"), no_std)]

// External crate imports
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(test)]
mod tests {
    // Import necessary types for no_std environment
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::{format, string::String};

    #[cfg(feature = "std")]
    use std::string::String;

    // Import from wrt-error
    use wrt_error::{codes, context::ErrorContext, kinds::ErrorKind, Error, ErrorCategory, Result};

    #[test]
    fn test_error_creation() {
        // Create an error
        let error = Error::new(
            ErrorCategory::Core,
            codes::INVALID_MEMORY_ACCESS,
            "Invalid memory access".to_string(),
        );

        // Verify error properties
        assert_eq!(error.category(), ErrorCategory::Core);
        assert_eq!(error.code(), codes::INVALID_MEMORY_ACCESS);
        assert_eq!(error.message(), "Invalid memory access");
    }

    #[test]
    fn test_error_with_context() {
        // Create an error with context
        let mut error = Error::new(
            ErrorCategory::Core,
            codes::INVALID_MEMORY_ACCESS,
            "Invalid memory access".to_string(),
        );

        // Add context
        error.add_context(ErrorContext::new("line", 42));
        error.add_context(ErrorContext::new("column", 10));

        // Verify context
        let contexts = error.contexts();
        assert_eq!(contexts.len(), 2);

        assert_eq!(contexts[0].key(), "line");
        assert_eq!(contexts[0].value_as_i64().unwrap(), 42);

        assert_eq!(contexts[1].key(), "column");
        assert_eq!(contexts[1].value_as_i64().unwrap(), 10);
    }

    #[test]
    fn test_result_operations() {
        // Test successful result
        let ok_result: Result<i32> = Ok(42);
        assert!(ok_result.is_ok());
        assert_eq!(ok_result.unwrap(), 42);

        // Test error result
        let error = Error::new(
            ErrorCategory::Core,
            codes::INVALID_MEMORY_ACCESS,
            "Invalid memory access".to_string(),
        );

        let err_result: Result<i32> = Err(error);
        assert!(err_result.is_err());

        let extracted_error = err_result.unwrap_err();
        assert_eq!(extracted_error.category(), ErrorCategory::Core);
    }

    #[test]
    fn test_error_categories() {
        // Test error categories
        assert_ne!(ErrorCategory::Core, ErrorCategory::Format);
        assert_ne!(ErrorCategory::Format, ErrorCategory::Validation);
        assert_ne!(ErrorCategory::Validation, ErrorCategory::Runtime);
        assert_ne!(ErrorCategory::Runtime, ErrorCategory::Resource);
    }

    #[test]
    fn test_error_kind() {
        // Test error kinds
        let memory_error = ErrorKind::MemoryError;
        let runtime_error = ErrorKind::RuntimeError;

        assert_ne!(memory_error, runtime_error);
    }
}
