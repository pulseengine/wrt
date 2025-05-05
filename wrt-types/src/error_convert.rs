//! Conversion between wrt_error and wrt_types error types
//!
//! This module provides utilities for converting between error types in the wrt-error
//! and wrt-types crates. It allows for consistent error handling across the codebase.

// Use std or alloc based on features
#[cfg(feature = "std")]
use std::fmt::Debug;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::fmt::Debug;
#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::string::String;

use wrt_error::{codes, Error as WrtErrorBase, ErrorCategory as WrtErrorCategory, FromError};

/// Error category for the wrt-types crate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Core WebAssembly errors
    Core,
    /// Component Model errors
    Component,
    /// Resource management errors
    Resource,
    /// Memory management errors
    Memory,
    /// Validation errors
    Validation,
    /// Type system errors
    Type,
    /// Runtime errors
    Runtime,
    /// System errors
    System,
    /// Parse errors
    Parse,
}

impl From<WrtErrorCategory> for ErrorCategory {
    fn from(category: WrtErrorCategory) -> Self {
        match category {
            WrtErrorCategory::Core => ErrorCategory::Core,
            WrtErrorCategory::Component => ErrorCategory::Component,
            WrtErrorCategory::Resource => ErrorCategory::Resource,
            WrtErrorCategory::Memory => ErrorCategory::Memory,
            WrtErrorCategory::Validation => ErrorCategory::Validation,
            WrtErrorCategory::Type => ErrorCategory::Type,
            WrtErrorCategory::Runtime => ErrorCategory::Runtime,
            WrtErrorCategory::System => ErrorCategory::System,
            WrtErrorCategory::Parse => ErrorCategory::Parse,
        }
    }
}

impl From<ErrorCategory> for WrtErrorCategory {
    fn from(category: ErrorCategory) -> Self {
        match category {
            ErrorCategory::Core => WrtErrorCategory::Core,
            ErrorCategory::Component => WrtErrorCategory::Component,
            ErrorCategory::Resource => WrtErrorCategory::Resource,
            ErrorCategory::Memory => WrtErrorCategory::Memory,
            ErrorCategory::Validation => WrtErrorCategory::Validation,
            ErrorCategory::Type => WrtErrorCategory::Type,
            ErrorCategory::Runtime => WrtErrorCategory::Runtime,
            ErrorCategory::System => WrtErrorCategory::System,
            ErrorCategory::Parse => WrtErrorCategory::Parse,
        }
    }
}

/// Error type for the wrt-types crate
#[derive(Debug, Clone)]
pub struct Error {
    /// Error category
    pub category: ErrorCategory,
    /// Error code
    pub code: u16,
    /// Error message
    #[cfg(feature = "alloc")]
    pub message: String,
}

impl Error {
    /// Create a new error
    #[cfg(feature = "alloc")]
    pub fn new<S: Into<String>>(category: ErrorCategory, code: u16, message: S) -> Self {
        Self {
            category,
            code,
            message: message.into(),
        }
    }

    /// Create a new error for no_alloc mode
    #[cfg(not(feature = "alloc"))]
    pub fn new<D: core::fmt::Display>(category: ErrorCategory, code: u16, _message: D) -> Self {
        Self { category, code }
    }

    /// Create an invalid type error with the standard invalid type code
    #[cfg(feature = "alloc")]
    pub fn invalid_type<S: Into<String>>(message: S) -> Self {
        Self::new(ErrorCategory::Type, codes::INVALID_TYPE, message)
    }

    /// Create an invalid type error with the standard invalid type code for no_alloc mode
    #[cfg(not(feature = "alloc"))]
    pub fn invalid_type<D: core::fmt::Display>(_message: D) -> Self {
        Self::new(ErrorCategory::Type, codes::INVALID_TYPE, "invalid type")
    }

    /// Create a parse error with the standard parse error code
    #[cfg(feature = "alloc")]
    pub fn parse_error<S: Into<String>>(message: S) -> Self {
        Self::new(ErrorCategory::Parse, codes::PARSE_ERROR, message)
    }

    /// Create a validation error with the standard validation error code
    #[cfg(feature = "alloc")]
    pub fn validation_error<S: Into<String>>(message: S) -> Self {
        Self::new(ErrorCategory::Validation, codes::VALIDATION_ERROR, message)
    }

    /// Create a resource error with the standard resource error code
    #[cfg(feature = "alloc")]
    pub fn resource_error<S: Into<String>>(message: S) -> Self {
        Self::new(ErrorCategory::Resource, codes::RESOURCE_ERROR, message)
    }

    /// Create a memory error with the standard memory access error code
    #[cfg(feature = "alloc")]
    pub fn memory_error<S: Into<String>>(message: S) -> Self {
        Self::new(ErrorCategory::Memory, codes::MEMORY_ACCESS_ERROR, message)
    }

    /// Create a component error with the standard component error code
    #[cfg(feature = "alloc")]
    pub fn component_error<S: Into<String>>(message: S) -> Self {
        Self::new(
            ErrorCategory::Component,
            codes::COMPONENT_TYPE_MISMATCH,
            message,
        )
    }

    /// Create a runtime error with the standard runtime error code
    #[cfg(feature = "alloc")]
    pub fn runtime_error<S: Into<String>>(message: S) -> Self {
        Self::new(ErrorCategory::Runtime, codes::RUNTIME_ERROR, message)
    }

    /// Create a system error with the standard system error code
    #[cfg(feature = "alloc")]
    pub fn system_error<S: Into<String>>(message: S) -> Self {
        Self::new(ErrorCategory::System, codes::SYSTEM_ERROR, message)
    }

    /// Create a type error with the standard type error code
    #[cfg(feature = "alloc")]
    pub fn type_error<S: Into<String>>(message: S) -> Self {
        Self::new(ErrorCategory::Type, codes::TYPE_MISMATCH, message)
    }

    /// Create a parse error with the standard parse error code for no_alloc mode
    #[cfg(not(feature = "alloc"))]
    pub fn parse_error<D: core::fmt::Display>(_message: D) -> Self {
        Self::new(ErrorCategory::Parse, codes::PARSE_ERROR, "parse error")
    }

    /// Create a validation error with the standard validation error code for no_alloc mode
    #[cfg(not(feature = "alloc"))]
    pub fn validation_error<D: core::fmt::Display>(_message: D) -> Self {
        Self::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            "validation error",
        )
    }

    /// Create a resource error with the standard resource error code for no_alloc mode
    #[cfg(not(feature = "alloc"))]
    pub fn resource_error<D: core::fmt::Display>(_message: D) -> Self {
        Self::new(
            ErrorCategory::Resource,
            codes::RESOURCE_ERROR,
            "resource error",
        )
    }

    /// Create a memory error with the standard memory access error code for no_alloc mode
    #[cfg(not(feature = "alloc"))]
    pub fn memory_error<D: core::fmt::Display>(_message: D) -> Self {
        Self::new(
            ErrorCategory::Memory,
            codes::MEMORY_ACCESS_ERROR,
            "memory error",
        )
    }

    /// Create a component error with the standard component error code for no_alloc mode
    #[cfg(not(feature = "alloc"))]
    pub fn component_error<D: core::fmt::Display>(_message: D) -> Self {
        Self::new(
            ErrorCategory::Component,
            codes::COMPONENT_TYPE_MISMATCH,
            "component error",
        )
    }

    /// Create a runtime error with the standard runtime error code for no_alloc mode
    #[cfg(not(feature = "alloc"))]
    pub fn runtime_error<D: core::fmt::Display>(_message: D) -> Self {
        Self::new(
            ErrorCategory::Runtime,
            codes::RUNTIME_ERROR,
            "runtime error",
        )
    }

    /// Create a system error with the standard system error code for no_alloc mode
    #[cfg(not(feature = "alloc"))]
    pub fn system_error<D: core::fmt::Display>(_message: D) -> Self {
        Self::new(ErrorCategory::System, codes::SYSTEM_ERROR, "system error")
    }

    /// Create a type error with the standard type error code for no_alloc mode
    #[cfg(not(feature = "alloc"))]
    pub fn type_error<D: core::fmt::Display>(_message: D) -> Self {
        Self::new(ErrorCategory::Type, codes::TYPE_MISMATCH, "type error")
    }

    /// Check if this is a resource error
    pub fn is_resource_error(&self) -> bool {
        self.category == ErrorCategory::Resource
    }

    /// Check if this is a memory error
    pub fn is_memory_error(&self) -> bool {
        self.category == ErrorCategory::Memory
    }

    /// Check if this is a validation error
    pub fn is_validation_error(&self) -> bool {
        self.category == ErrorCategory::Validation
    }

    /// Check if this is a type error
    pub fn is_type_error(&self) -> bool {
        self.category == ErrorCategory::Type
    }

    /// Check if this is a runtime error
    pub fn is_runtime_error(&self) -> bool {
        self.category == ErrorCategory::Runtime
    }

    /// Check if this is a parse error
    pub fn is_parse_error(&self) -> bool {
        self.category == ErrorCategory::Parse
    }
}

// Implement ToErrorCategory for Error
impl wrt_error::ToErrorCategory for Error {
    fn to_category(&self) -> wrt_error::ErrorCategory {
        self.category.into()
    }
}

// Implement FromError for conversion from Error to WrtErrorBase
impl FromError<Error> for WrtErrorBase {
    fn from_error(error: Error) -> Self {
        #[cfg(feature = "alloc")]
        {
            Self::new(error.category.into(), error.code, error.message.clone())
        }

        #[cfg(not(feature = "alloc"))]
        {
            Self::new(error.category.into(), error.code, "Converted error")
        }
    }
}

// Implement FromError for conversion from WrtErrorBase to Error
impl FromError<WrtErrorBase> for Error {
    fn from_error(error: WrtErrorBase) -> Self {
        #[cfg(feature = "alloc")]
        {
            Self::new(error.category.into(), error.code, error.message.clone())
        }

        #[cfg(not(feature = "alloc"))]
        {
            Self::new(error.category.into(), error.code, "Converted error")
        }
    }
}

/// Converts an Error to a WrtErrorBase
///
/// This function is used to convert between error types across the WRT codebase,
/// preserving the category, code, and message of the original error.
pub fn convert_to_wrt_error(e: Error) -> wrt_error::Error {
    #[cfg(feature = "alloc")]
    {
        wrt_error::Error::new(e.category.into(), e.code, e.message.clone())
    }

    #[cfg(not(feature = "alloc"))]
    {
        wrt_error::Error::new(e.category.into(), e.code, "Converted error")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_category_conversion() {
        let wrt_categories = [
            WrtErrorCategory::Core,
            WrtErrorCategory::Component,
            WrtErrorCategory::Resource,
            WrtErrorCategory::Memory,
            WrtErrorCategory::Validation,
            WrtErrorCategory::Type,
            WrtErrorCategory::Runtime,
            WrtErrorCategory::System,
            WrtErrorCategory::Parse,
        ];

        let types_categories = [
            ErrorCategory::Core,
            ErrorCategory::Component,
            ErrorCategory::Resource,
            ErrorCategory::Memory,
            ErrorCategory::Validation,
            ErrorCategory::Type,
            ErrorCategory::Runtime,
            ErrorCategory::System,
            ErrorCategory::Parse,
        ];

        // Test roundtrip conversions
        for (wrt_cat, types_cat) in wrt_categories.iter().zip(types_categories.iter()) {
            // Convert from wrt to types
            let converted_types_cat: ErrorCategory = (*wrt_cat).into();
            assert_eq!(&converted_types_cat, types_cat);

            // Convert from types to wrt
            let converted_wrt_cat: WrtErrorCategory = (*types_cat).into();
            assert_eq!(&converted_wrt_cat, wrt_cat);
        }
    }

    #[test]
    fn test_error_conversion() {
        // Create a wrt-types error
        let types_error = Error::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            "Test validation error",
        );

        // Convert to wrt_error
        let wrt_error = convert_to_wrt_error(types_error.clone());

        assert_eq!(wrt_error.code, codes::VALIDATION_ERROR);
        assert_eq!(wrt_error.category, WrtErrorCategory::Validation);
    }

    #[test]
    fn test_error_helper_functions() {
        let error = Error::validation_error("Test validation error");
        assert_eq!(error.code, codes::VALIDATION_ERROR);
        assert_eq!(error.category, ErrorCategory::Validation);
        assert!(error.is_validation_error());

        let error = Error::resource_error("Test resource error");
        assert_eq!(error.code, codes::RESOURCE_ERROR);
        assert_eq!(error.category, ErrorCategory::Resource);
        assert!(error.is_resource_error());
    }
}
