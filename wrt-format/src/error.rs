//! Error utilities for the format crate.
//!
//! This module provides helper functions for creating various error types
//! related to parsing and validation of WebAssembly formats.

#[cfg(feature = "std")]
use std::fmt;

#[cfg(not(feature = "std"))]
use core::fmt;

use wrt_error::{codes, Error as WrtErrorBase};
use wrt_types::error_convert::{convert_to_wrt_error, Error, ErrorCategory};
use wrt_types::String;

// Create trait for error conversion
pub trait IntoError {
    fn into_error(self, category: ErrorCategory, code: u16) -> Error;
}

// Implement for string types
impl<S> IntoError for S
where
    S: Into<String>,
{
    fn into_error(self, category: ErrorCategory, code: u16) -> Error {
        Error::new(category, code, self.into())
    }
}

// Helper functions for common error types
pub fn parse_error(message: impl Into<String> + fmt::Display) -> Error {
    Error::new(ErrorCategory::Parse, codes::PARSE_ERROR, message.into())
}

// Create a validation error
pub fn validation_error(message: impl Into<String> + fmt::Display) -> Error {
    Error::new(
        ErrorCategory::Validation,
        codes::VALIDATION_ERROR,
        message.into(),
    )
}

// Create a type error
pub fn type_error(message: impl Into<String> + fmt::Display) -> Error {
    Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH, message.into())
}

// Create a runtime error
pub fn runtime_error(message: impl Into<String> + fmt::Display) -> Error {
    Error::new(ErrorCategory::Runtime, codes::RUNTIME_ERROR, message.into())
}

/// Create a new runtime error with the given message using WrtError
pub fn wrt_runtime_error(message: impl Into<String>) -> WrtErrorBase {
    WrtErrorBase::runtime_error(message)
}

/// Create a new parse error with the given message using WrtError
pub fn wrt_parse_error(message: impl Into<String>) -> WrtErrorBase {
    WrtErrorBase::validation_error(message)
}

/// Create a new validation error with the given message using WrtError
pub fn wrt_validation_error(message: impl Into<String>) -> WrtErrorBase {
    WrtErrorBase::validation_error(message)
}

/// Create a new type error with the given message using WrtError
pub fn wrt_type_error(message: impl Into<String>) -> WrtErrorBase {
    WrtErrorBase::type_error(message)
}

/// Convert a wrt-types Error to a WrtErrorBase
///
/// This is a convenience function to convert wrt-types errors to wrt-error
/// to avoid the `.into()` method which may cause issues with other crates.
pub fn to_wrt_error(error: Error) -> WrtErrorBase {
    // Use the built-in conversion function from wrt_types
    convert_to_wrt_error(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = parse_error("Test parse error");
        assert_eq!(err.code, codes::PARSE_ERROR);
        assert_eq!(err.category, ErrorCategory::Parse);

        let err = validation_error("Test validation error");
        assert_eq!(err.code, codes::VALIDATION_ERROR);
        assert_eq!(err.category, ErrorCategory::Validation);

        let err = type_error("Test type error");
        assert_eq!(err.code, codes::TYPE_MISMATCH);
        assert_eq!(err.category, ErrorCategory::Type);

        let err = runtime_error("Test runtime error");
        assert_eq!(err.code, codes::RUNTIME_ERROR);
        assert_eq!(err.category, ErrorCategory::Runtime);
    }

    #[test]
    fn test_into_error_trait() {
        let error = "test error".into_error(ErrorCategory::Validation, codes::PARSE_ERROR);
        assert_eq!(error.category, ErrorCategory::Validation);
        assert_eq!(error.code, codes::PARSE_ERROR);

        // Don't check message content because it may not be accessible or formatted consistently
        // across different modes (std vs no_std)
    }

    #[test]
    fn test_error_conversion() {
        let error = Error::validation_error("Validation error test");
        let wrt_error = to_wrt_error(error.clone());

        assert_eq!(wrt_error.code, error.code);
        assert_eq!(wrt_error.category, wrt_error::ErrorCategory::Validation);
    }
}
