//! Integration tests for the wrt-error crate.

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use wrt_error::{Error, Result, ResultExt};
    use wrt_error::source::ErrorSource;
    use wrt_error::kinds;
    use core::fmt::{self, Display};

    // A simple test error type
    #[derive(Debug, Clone)]
    struct TestError(&'static str);

    impl Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "TestError: {}", self.0)
        }
    }

    impl ErrorSource for TestError {
        #[cfg(feature = "std")]
        fn source(&self) -> Option<&(dyn ErrorSource + 'static)> {
            None
        }
    }

    #[test]
    fn test_basic_error_creation() {
        // Create an error from a custom type
        let error = Error::new(TestError("something went wrong"));
        assert_eq!(format!("{}", error), "TestError: something went wrong");
        
        // Create an error using a factory method
        let div_error = Error::division_by_zero();
        assert_eq!(format!("{}", div_error), "Division by zero");
    }

    #[test]
    fn test_error_conversion() {
        // Test From<E> implementation
        let error: Result<()> = Err(TestError("conversion test")).map_err(Error::from);
        assert!(error.is_err());
        assert_eq!(format!("{}", error.unwrap_err()), "TestError: conversion test");
    }

    #[test]
    fn test_context_extension() {
        // Test adding context to an error
        let result: core::result::Result<(), TestError> = Err(TestError("base error"));
        let with_context = result.context("operation failed");
        assert!(with_context.is_err());
        
        let error_message = format!("{}", with_context.unwrap_err());
        assert!(error_message.contains("operation failed"));
        assert!(error_message.contains("base error"));
    }

    #[test]
    fn test_error_kinds() {
        // Test a few error kinds
        let stack_error = kinds::StackUnderflowError;
        assert_eq!(format!("{}", stack_error), "Stack underflow");
        
        let memory_error = kinds::MemoryAccessOutOfBoundsError { address: 1000, length: 8 };
        assert_eq!(
            format!("{}", memory_error),
            "Memory access out of bounds: address 1000, length 8"
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_error_chaining() {
        use std::io::{Error as IoError, ErrorKind};

        // Create an IO error
        let io_error = IoError::new(ErrorKind::NotFound, "file not found");
        
        // Chain contexts
        let result: Result<()> = Err(Error::from(io_error))
            .context("failed to read configuration")
            .context("application initialization failed");
            
        let error = result.unwrap_err();
        let error_message = format!("{}", error);
        
        assert!(error_message.contains("application initialization failed"));
        assert!(error_message.contains("failed to read configuration"));
    }
} 