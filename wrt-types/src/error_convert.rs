//! Error conversion module
//! This module provides conversion implementations between wrt_error and wrt_types

use wrt_error::codes;
use wrt_error::Error as WrtErrorBase;
use wrt_error::ErrorCategory as WrtErrorCategory;

#[cfg(feature = "std")]
use std::string::String;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{string::String, string::ToString};

/// Error category for the wrt-types crate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Core,
    Component,
    Resource,
    Memory,
    Validation,
    Type,
    Runtime,
    System,
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
    pub fn new(category: ErrorCategory, code: u16, message: impl Into<String>) -> Self {
        Self {
            category,
            code,
            message: message.into(),
        }
    }

    /// Create a new error for no_alloc mode
    #[cfg(not(feature = "alloc"))]
    pub fn new(category: ErrorCategory, code: u16, _message: impl core::fmt::Display) -> Self {
        Self { category, code }
    }

    /// Create an invalid type error with the standard invalid type code
    #[cfg(feature = "alloc")]
    pub fn invalid_type(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Type, codes::INVALID_TYPE, message)
    }

    /// Create an invalid type error with the standard invalid type code for no_alloc mode
    #[cfg(not(feature = "alloc"))]
    pub fn invalid_type(_message: impl core::fmt::Display) -> Self {
        Self::new(ErrorCategory::Type, codes::INVALID_TYPE, "invalid type")
    }

    /// Create a parse error with the standard parse error code
    #[cfg(feature = "alloc")]
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Parse, codes::PARSE_ERROR, message)
    }

    /// Create a validation error with the standard validation error code
    #[cfg(feature = "alloc")]
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Validation, codes::VALIDATION_ERROR, message)
    }

    /// Create a resource error with the standard resource error code
    #[cfg(feature = "alloc")]
    pub fn resource_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Resource, codes::RESOURCE_ERROR, message)
    }

    /// Create a parse error with the standard parse error code for no_alloc mode
    #[cfg(not(feature = "alloc"))]
    pub fn parse_error(_message: impl core::fmt::Display) -> Self {
        Self::new(ErrorCategory::Parse, codes::PARSE_ERROR, "parse error")
    }

    /// Create a validation error with the standard validation error code for no_alloc mode
    #[cfg(not(feature = "alloc"))]
    pub fn validation_error(_message: impl core::fmt::Display) -> Self {
        Self::new(
            ErrorCategory::Validation,
            codes::VALIDATION_ERROR,
            "validation error",
        )
    }

    /// Create a resource error with the standard resource error code for no_alloc mode
    #[cfg(not(feature = "alloc"))]
    pub fn resource_error(_message: impl core::fmt::Display) -> Self {
        Self::new(
            ErrorCategory::Resource,
            codes::RESOURCE_ERROR,
            "resource error",
        )
    }
}

// Add From implementation for converting Error to WrtErrorBase
#[cfg(feature = "alloc")]
impl From<Error> for WrtErrorBase {
    fn from(error: Error) -> Self {
        WrtErrorBase::new(error.category.into(), error.code, error.message.clone())
    }
}

#[cfg(not(feature = "alloc"))]
impl From<Error> for WrtErrorBase {
    fn from(error: Error) -> Self {
        WrtErrorBase::new(
            error.category.into(),
            error.code,
            "error", // Simplified message for no_alloc mode
        )
    }
}

// Add From implementation for converting WrtErrorBase to Error
#[cfg(feature = "alloc")]
impl From<WrtErrorBase> for Error {
    fn from(error: WrtErrorBase) -> Self {
        Self {
            category: error.category.into(),
            code: error.code,
            message: error.message.clone(),
        }
    }
}

#[cfg(not(feature = "alloc"))]
impl From<WrtErrorBase> for Error {
    fn from(error: WrtErrorBase) -> Self {
        Self {
            category: error.category.into(),
            code: error.code,
        }
    }
}

// Keep the convert_to_wrt_error function for backward compatibility
// but mark it as deprecated
#[cfg(feature = "alloc")]
#[deprecated(since = "0.2.0", note = "Use the From trait implementation instead")]
pub fn convert_to_wrt_error(e: Error) -> WrtErrorBase {
    e.into()
}

#[cfg(not(feature = "alloc"))]
#[deprecated(since = "0.2.0", note = "Use the From trait implementation instead")]
pub fn convert_to_wrt_error(e: Error) -> WrtErrorBase {
    e.into()
}
