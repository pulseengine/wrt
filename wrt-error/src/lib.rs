//! Error handling for the WRT WebAssembly runtime.
//!
//! This crate provides a lightweight, no_std compatible error handling system
//! that supports error chaining, context, and specific error types for WebAssembly operations.

#![warn(clippy::missing_panics_doc)]

// Re-export modules
pub mod context;
pub mod error;
pub mod kinds;
pub mod source;

// Include verification module when the kani feature is enabled
#[cfg(feature = "kani")]
pub mod verify;

// Re-export key types
#[cfg(feature = "alloc")]
pub use error::Error;

#[cfg(feature = "alloc")]
pub use context::ResultExt;

/// A specialized `Result` type for WRT operations.
/// When the `alloc` feature is enabled, this defaults to using `wrt_error::Error` as the error type.
/// When `alloc` is not available, the specific error type must be provided.
#[cfg(feature = "alloc")]
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// A specialized `Result` type for WRT operations when `alloc` is not available.
/// The specific error type must be provided.
#[cfg(not(feature = "alloc"))]
pub type Result<T, E> = core::result::Result<T, E>;

// Add implementations of From for error types used in execution.rs
#[cfg(feature = "alloc")]
impl From<kinds::ParseError> for Error {
    fn from(err: kinds::ParseError) -> Self {
        Error::new(err)
    }
}

#[cfg(feature = "alloc")]
impl From<kinds::ExecutionError> for Error {
    fn from(err: kinds::ExecutionError) -> Self {
        Error::new(err)
    }
}

#[cfg(feature = "alloc")]
impl From<kinds::ExportNotFoundError> for Error {
    fn from(err: kinds::ExportNotFoundError) -> Self {
        Error::new(err)
    }
}

#[cfg(feature = "alloc")]
impl From<kinds::StackUnderflowError> for Error {
    fn from(err: kinds::StackUnderflowError) -> Self {
        Error::new(err)
    }
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::source::ErrorSource;
    use super::{Error, Result, ResultExt};

    use core::fmt::{self, Display};

    // A simple custom error for testing
    #[derive(Debug, Clone)]
    struct MyTestError(&'static str);

    impl Display for MyTestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "MyTestError: {}", self.0)
        }
    }

    // Implement ErrorSource for the test error
    impl ErrorSource for MyTestError {
        #[cfg(feature = "std")]
        fn source(&self) -> Option<&(dyn ErrorSource + 'static)> {
            None
        }
    }

    #[test]
    fn test_error_creation() {
        let e1: Result<()> = Err(Error::new(MyTestError("Something failed ")));
        assert!(e1.is_err());

        fn fallible() -> Result<(), MyTestError> {
            Err(MyTestError("Operation failed "))
        }

        let e2: Result<()> = fallible().map_err(Error::from); // Use Error::from
        assert!(e2.is_err());
        println!("Created error: {}", e2.unwrap_err()); // Test Display
    }

    #[test]
    fn test_context() {
        fn fallible() -> core::result::Result<(), MyTestError> {
            Err(MyTestError("Base error "))
        }

        let res = fallible().context("Failed during high-level operation ");
        assert!(res.is_err());
        let err = res.unwrap_err();

        let display_output = format!("{}", err);
        assert!(display_output.contains("Failed during high-level operation "));
        assert!(display_output.contains("Base error "));
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_debug_chaining_std() {
        use std::io::{Error as IoError, ErrorKind};

        let io_err = IoError::new(ErrorKind::NotFound, "Low level IO error");

        let res: Result<()> = Err(Error::from(io_err))
            .context("Mid level operation failed")
            .context("Top level task failed");

        assert!(res.is_err());
        let err_dbg = format!("{:?}", res.unwrap_err());

        assert!(err_dbg.contains("Top level task failed"));
        assert!(err_dbg.contains("Mid level operation failed"));
        assert!(err_dbg.contains("Low level IO error"));
    }
}
