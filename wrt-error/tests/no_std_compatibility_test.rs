//! Test `no_std` compatibility for wrt-error
//!
//! This file validates that the wrt-error crate works correctly in `no_std`
//! environments.

// For testing in a no_std environment
#![cfg_attr(not(feature = "std"), no_std)]

// External crate imports
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::unnecessary_literal_unwrap, clippy::panic)]
mod tests {
    // Import necessary types for no_std environment
    // #[cfg(all(not(feature = "std"), feature = "alloc"))]
    // use alloc::{format, string::ToString};

    // Import from wrt-error
    use wrt_error::{codes, kinds, Error, ErrorCategory, Result};

    #[test]
    fn test_error_creation() {
        // Create an error
        let error =
            Error::new(ErrorCategory::Core, codes::INVALID_MEMORY_ACCESS, "Invalid memory access");

        // Verify error properties
        assert_eq!(error.category, ErrorCategory::Core);
        assert_eq!(error.code, codes::INVALID_MEMORY_ACCESS);
    }

    #[test]
    fn test_result_operations() {
        // Test successful result
        let ok_result: Result<i32> = Ok(42);
        assert!(ok_result.is_ok());
        assert_eq!(ok_result.unwrap(), 42);

        // Test error result
        let error =
            Error::new(ErrorCategory::Core, codes::INVALID_MEMORY_ACCESS, "Invalid memory access");

        let err_result: Result<i32> = Err(error);
        assert!(err_result.is_err());

        let extracted_error = err_result.unwrap_err();
        assert_eq!(extracted_error.category, ErrorCategory::Core);
    }

    #[test]
    fn test_error_categories() {
        // Test error categories
        assert_ne!(ErrorCategory::Core, ErrorCategory::Resource);
        assert_ne!(ErrorCategory::Memory, ErrorCategory::Validation);
        assert_ne!(ErrorCategory::Validation, ErrorCategory::Runtime);
        assert_ne!(ErrorCategory::Runtime, ErrorCategory::System);
    }

    #[test]
    fn test_error_kind() {
        let validation_error = kinds::validation_error("Validation error");
        let memory_error = kinds::memory_access_error("Memory error");
        let runtime_error = kinds::runtime_error("Runtime error");

        let type_name_validation = core::any::type_name_of_val(&validation_error);
        assert!(type_name_validation.contains("ValidationError"));

        let type_name_memory = core::any::type_name_of_val(&memory_error);
        // Note: kinds::memory_access_error creates a kinds::MemoryAccessError struct
        assert!(type_name_memory.contains("MemoryAccessError"));

        let type_name_runtime = core::any::type_name_of_val(&runtime_error);
        assert!(type_name_runtime.contains("RuntimeError"));
    }

    // Helper to get the concrete type
    #[allow(dead_code)]
    fn types_of<T>(_: T) -> T {
        panic!("This function should never be called")
    }

    // Test basic error creation (no_std)
    #[test]
    fn test_error_creation_no_std() {
        let error = Error::new(
            ErrorCategory::Core,
            codes::COMPONENT_INSTANTIATION_ERROR,
            "Invalid memory access",
        );
        assert_eq!(error.category, ErrorCategory::Core);
        assert_eq!(error.code, codes::COMPONENT_INSTANTIATION_ERROR);

        let result: Result<()> = Err(Error::new(
            ErrorCategory::Core,
            codes::COMPONENT_INSTANTIATION_ERROR,
            "Invalid memory access",
        ));
        match result {
            Ok(()) => panic!("Expected error, got Ok"),
            Err(e) => {
                assert_eq!(e.category, ErrorCategory::Core);
                assert_eq!(e.code, codes::COMPONENT_INSTANTIATION_ERROR);
                assert_eq!(e.message, "Invalid memory access");
            }
        }
    }

    #[test]
    fn test_error_handling_no_std() {
        type Result<T> = core::result::Result<T, Error>;

        let result: Result<()> = Err(Error::new(
            ErrorCategory::Core,
            codes::COMPONENT_INSTANTIATION_ERROR,
            "Invalid memory access",
        ));

        match result {
            Err(e) => {
                assert_eq!(e.category, ErrorCategory::Core);
                assert_eq!(e.code, codes::COMPONENT_INSTANTIATION_ERROR);
                assert_eq!(e.message, "Invalid memory access");
            }
            Ok(()) => panic!("Expected an error"),
        }
    }

    // Test error creation and handling with different error types (no_std)
    #[test]
    fn test_complex_error_no_std() {
        let error = Error::new(
            ErrorCategory::Resource,
            codes::RESOURCE_LIMIT_EXCEEDED,
            "Invalid memory access",
        );

        assert_eq!(error.category, ErrorCategory::Resource);
        assert_eq!(error.code, codes::RESOURCE_LIMIT_EXCEEDED);
    }
}
