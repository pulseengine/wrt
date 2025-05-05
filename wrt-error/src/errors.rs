// Import from std when available
#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::string::ToString;

// Import from alloc when not using std but using alloc
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::string::ToString;

/// Unified error handling system for WRT
///
/// This module provides a comprehensive error handling system that covers all error cases
/// across the WRT codebase. It includes error types, categories, and helper functions.
use core::fmt;

// Use alloc if alloc feature is enabled
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::boxed::Box;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::string::String;

// Use std if std feature is enabled
#[cfg(feature = "std")]
use std::boxed::Box;
#[cfg(feature = "std")]
use std::string::String;

use crate::kinds;
use crate::{FromError, ToErrorCategory};

/// Error categories for WRT operations
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

/// Base trait for all error types - std version with Send+Sync
#[cfg(feature = "std")]
pub trait ErrorSource: fmt::Debug + Send + Sync {
    /// Get the error code
    fn code(&self) -> u16;

    /// Get the error message
    fn message(&self) -> &str;

    /// Get the error category
    fn category(&self) -> ErrorCategory;

    /// Get the source error if any
    fn source(&self) -> Option<&(dyn ErrorSource + 'static)> {
        None
    }
}

/// Base trait for all error types - no_std version
#[cfg(not(feature = "std"))]
pub trait ErrorSource: fmt::Debug {
    /// Get the error code
    fn code(&self) -> u16;

    /// Get the error message
    fn message(&self) -> &str;

    /// Get the error category
    fn category(&self) -> ErrorCategory;

    /// Get the source error if any
    fn source(&self) -> Option<&(dyn ErrorSource + 'static)> {
        None
    }
}

/// WRT Error type
///
/// This is the main error type for the WebAssembly Runtime.
/// It provides categorized errors with error codes and optional messages.
#[derive(Debug)]
pub struct Error {
    /// Error category
    pub category: ErrorCategory,
    /// Error code
    pub code: u16,
    /// Error message
    #[cfg(feature = "alloc")]
    pub message: String,
    /// Optional source error
    #[cfg(all(feature = "alloc", feature = "std"))]
    pub source: Option<Box<dyn ErrorSource + Send + Sync + 'static>>,
    /// Optional source error (no_std version)
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub source: Option<Box<dyn ErrorSource + 'static>>,
}

#[cfg(feature = "alloc")]
impl Clone for Error {
    fn clone(&self) -> Self {
        Self {
            category: self.category,
            code: self.code,
            message: self.message.clone(),
            source: None, // Don't clone the source, as it might not be cloneable
        }
    }
}

impl Error {
    /// Create a new error with allocation support
    #[cfg(feature = "alloc")]
    pub fn new<S: Into<String>>(category: ErrorCategory, code: u16, message: S) -> Self {
        Self {
            category,
            code,
            message: message.into(),
            source: None,
        }
    }

    /// Create a new error in no_std mode (no message, no source)
    #[cfg(not(feature = "alloc"))]
    pub fn new<D: core::fmt::Display>(category: ErrorCategory, code: u16, _message: D) -> Self {
        Self { category, code }
    }

    /// Create a new error from a kinds type
    #[cfg(feature = "alloc")]
    pub fn from_kind<K: core::fmt::Display>(kind: K, code: u16, category: ErrorCategory) -> Self {
        Self {
            category,
            code,
            message: kind.to_string(),
            source: None,
        }
    }

    /// Create a new error from a kinds type in no_std mode (no message storage)
    #[cfg(not(feature = "alloc"))]
    pub fn from_kind<K: core::fmt::Display>(_kind: K, code: u16, category: ErrorCategory) -> Self {
        Self { category, code }
    }

    /// Create a new error with a source
    #[cfg(all(feature = "alloc", feature = "std"))]
    pub fn with_source(
        category: ErrorCategory,
        code: u16,
        message: impl Into<String>,
        source: Box<dyn ErrorSource + Send + Sync>,
    ) -> Self {
        Self {
            category,
            code,
            message: message.into(),
            source: Some(source),
        }
    }

    /// Create a new error with a source (no_std version)
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    pub fn with_source(
        category: ErrorCategory,
        code: u16,
        message: impl Into<String>,
        source: Box<dyn ErrorSource>,
    ) -> Self {
        Self {
            category,
            code,
            message: message.into(),
            source: Some(source),
        }
    }

    /// Create a new error with a source in no_std mode (ignores source)
    #[cfg(not(feature = "alloc"))]
    pub fn with_source(
        category: ErrorCategory,
        code: u16,
        _message: impl core::fmt::Display,
        _source: impl core::fmt::Debug,
    ) -> Self {
        Self { category, code }
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

    /// Check if this is a system error
    pub fn is_system_error(&self) -> bool {
        self.category == ErrorCategory::System
    }

    /// Check if this is a core error
    pub fn is_core_error(&self) -> bool {
        self.category == ErrorCategory::Core
    }

    /// Check if this is a component error
    pub fn is_component_error(&self) -> bool {
        self.category == ErrorCategory::Component
    }

    // Factory methods for std/alloc environments

    /// Create a resource error - std/alloc version
    #[cfg(feature = "alloc")]
    pub fn resource_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Resource, codes::RESOURCE_ERROR, message)
    }

    /// Create a memory error - std/alloc version
    #[cfg(feature = "alloc")]
    pub fn memory_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Memory, codes::MEMORY_OUT_OF_BOUNDS, message)
    }

    /// Create a validation error - std/alloc version
    #[cfg(feature = "alloc")]
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Validation, codes::VALIDATION_ERROR, message)
    }

    /// Create a type error - std/alloc version
    #[cfg(feature = "alloc")]
    pub fn type_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, message)
    }

    /// Create a runtime error - std/alloc version
    #[cfg(feature = "alloc")]
    pub fn runtime_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Runtime, codes::RUNTIME_ERROR, message)
    }

    /// Create a system error - std/alloc version
    #[cfg(feature = "alloc")]
    pub fn system_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::System, codes::SYSTEM_ERROR, message)
    }

    /// Create a core error - std/alloc version
    #[cfg(feature = "alloc")]
    pub fn core_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Core, codes::EXECUTION_ERROR, message)
    }

    /// Create a component error - std/alloc version
    #[cfg(feature = "alloc")]
    pub fn component_error(message: impl Into<String>) -> Self {
        Self::new(
            ErrorCategory::Component,
            codes::COMPONENT_TYPE_MISMATCH,
            message,
        )
    }

    /// Create a parse error
    #[cfg(feature = "alloc")]
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Parse, codes::PARSE_ERROR, message)
    }

    /// Create an invalid type error
    #[cfg(feature = "alloc")]
    pub fn invalid_type(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Type, codes::INVALID_TYPE, message)
    }

    /// Create an invalid type error - no_std version
    #[cfg(not(feature = "alloc"))]
    pub fn invalid_type(message: impl core::fmt::Display) -> Self {
        Self::new(ErrorCategory::Type, codes::INVALID_TYPE, message)
    }

    // Factory methods for no_std mode

    /// Create a resource error - no_std version
    #[cfg(not(feature = "alloc"))]
    pub fn resource_error(message: impl core::fmt::Display) -> Self {
        Self::new(ErrorCategory::Resource, codes::RESOURCE_ERROR, message)
    }

    /// Create a memory error - no_std version
    #[cfg(not(feature = "alloc"))]
    pub fn memory_error(message: impl core::fmt::Display) -> Self {
        Self::new(ErrorCategory::Memory, codes::MEMORY_OUT_OF_BOUNDS, message)
    }

    /// Create a validation error - no_std version
    #[cfg(not(feature = "alloc"))]
    pub fn validation_error(message: impl core::fmt::Display) -> Self {
        Self::new(ErrorCategory::Validation, codes::VALIDATION_ERROR, message)
    }

    /// Create a type error - no_std version
    #[cfg(not(feature = "alloc"))]
    pub fn type_error(message: impl core::fmt::Display) -> Self {
        Self::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, message)
    }

    /// Create a runtime error - no_std version
    #[cfg(not(feature = "alloc"))]
    pub fn runtime_error(message: impl core::fmt::Display) -> Self {
        Self::new(ErrorCategory::Runtime, codes::RUNTIME_ERROR, message)
    }

    /// Create a system error - no_std version
    #[cfg(not(feature = "alloc"))]
    pub fn system_error(message: impl core::fmt::Display) -> Self {
        Self::new(ErrorCategory::System, codes::SYSTEM_ERROR, message)
    }

    /// Create a core error - no_std version
    #[cfg(not(feature = "alloc"))]
    pub fn core_error(message: impl core::fmt::Display) -> Self {
        Self::new(ErrorCategory::Core, codes::EXECUTION_ERROR, message)
    }

    /// No allocation version of component_error
    #[cfg(not(feature = "alloc"))]
    pub fn component_error(message: impl core::fmt::Display) -> Self {
        Self::new(
            ErrorCategory::Component,
            codes::COMPONENT_TYPE_MISMATCH,
            message,
        )
    }

    /// No allocation version of parse_error
    #[cfg(not(feature = "alloc"))]
    pub fn parse_error(message: impl core::fmt::Display) -> Self {
        Self::new(ErrorCategory::Parse, codes::PARSE_ERROR, message)
    }

    #[cfg(feature = "alloc")]
    pub fn with_message(message: impl Into<String>) -> Self {
        // Default to runtime error category and unknown code
        Self::new(ErrorCategory::Runtime, codes::UNKNOWN, message)
    }

    #[cfg(not(feature = "alloc"))]
    pub fn with_message(message: impl core::fmt::Display) -> Self {
        // Default to runtime error category and unknown code for no_std
        Self::new(ErrorCategory::Runtime, codes::UNKNOWN, message)
    }

    // Legacy constructor that accepts only a message - forwards to with_message
    // This is only for backward compatibility during migration
    pub fn new_legacy(message: impl Into<String>) -> Self {
        #[cfg(feature = "alloc")]
        return Self::with_message(message);

        #[cfg(not(feature = "alloc"))]
        return Self::with_message(format_args!("{}", message.into()));
    }

    // Quick factory methods for different error categories with default code
    pub fn resource_error_with_code(code: u16, message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Resource, code, message)
    }

    pub fn validation_error_with_code(code: u16, message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Validation, code, message)
    }

    pub fn type_error_with_code(code: u16, message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Type, code, message)
    }

    pub fn runtime_error_with_code(code: u16, message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Runtime, code, message)
    }

    pub fn system_error_with_code(code: u16, message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::System, code, message)
    }

    pub fn core_error_with_code(code: u16, message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Core, code, message)
    }

    pub fn component_error_with_code(code: u16, message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Component, code, message)
    }

    /// Create a parse error from a ParseError kind
    pub fn parse_error_from_kind(kind: crate::kinds::ParseError) -> Self {
        Self::from(kind)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "alloc")]
        {
            write!(f, "{} (code: {})", self.message, self.code)
        }
        #[cfg(not(feature = "alloc"))]
        {
            write!(
                f,
                "Error category {:?} (code: {})",
                self.category, self.code
            )
        }
    }
}

impl ErrorSource for Error {
    fn code(&self) -> u16 {
        self.code
    }

    #[cfg(feature = "alloc")]
    fn message(&self) -> &str {
        &self.message
    }

    #[cfg(not(feature = "alloc"))]
    fn message(&self) -> &str {
        ""
    }

    fn category(&self) -> ErrorCategory {
        self.category
    }

    #[cfg(all(feature = "alloc", feature = "std"))]
    fn source(&self) -> Option<&(dyn ErrorSource + 'static)> {
        self.source.as_ref().map(|s| {
            let ptr: *const dyn ErrorSource = &**s as *const dyn ErrorSource;
            unsafe { &*ptr }
        })
    }

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    fn source(&self) -> Option<&(dyn ErrorSource + 'static)> {
        self.source.as_ref().map(|s| s.as_ref())
    }

    #[cfg(not(feature = "alloc"))]
    fn source(&self) -> Option<&(dyn ErrorSource + 'static)> {
        None
    }
}

/// Error codes for different categories
pub mod codes {
    // Core WebAssembly errors (1000-1999)
    pub const STACK_UNDERFLOW: u16 = 1000;
    pub const STACK_OVERFLOW: u16 = 1001;
    pub const UNALIGNED_MEMORY_ACCESS: u16 = 1002;
    pub const INVALID_MEMORY_ACCESS: u16 = 1003;
    pub const INVALID_INSTANCE_INDEX: u16 = 1004;
    pub const EXECUTION_ERROR: u16 = 1005;
    pub const NOT_IMPLEMENTED: u16 = 1006;
    pub const MEMORY_ACCESS_ERROR: u16 = 1007;
    pub const INITIALIZATION_ERROR: u16 = 1008;
    pub const TYPE_MISMATCH: u16 = 1009;
    pub const PARSE_ERROR: u16 = 1010;
    pub const INVALID_VERSION: u16 = 1011;
    pub const OUT_OF_BOUNDS_ERROR: u16 = 1012;

    // Component Model errors (2000-2999)
    pub const INVALID_FUNCTION_INDEX: u16 = 2000;
    pub const COMPONENT_TYPE_MISMATCH: u16 = 2001;
    pub const ENCODING_ERROR: u16 = 2002;
    pub const EXECUTION_LIMIT_EXCEEDED: u16 = 2003;
    pub const COMPONENT_INSTANTIATION_ERROR: u16 = 2004;
    pub const CANONICAL_ABI_ERROR: u16 = 2005;
    pub const COMPONENT_LINKING_ERROR: u16 = 2006;

    // Resource Management errors (3000-3999)
    pub const RESOURCE_ERROR: u16 = 3000;
    pub const RESOURCE_LIMIT_EXCEEDED: u16 = 3001;
    pub const RESOURCE_ACCESS_ERROR: u16 = 3002;
    pub const RESOURCE_NOT_FOUND: u16 = 3003;
    pub const RESOURCE_INVALID_HANDLE: u16 = 3004;

    // Memory Management errors (4000-4999)
    pub const MEMORY_OUT_OF_BOUNDS: u16 = 4000;
    pub const MEMORY_GROW_ERROR: u16 = 4001;
    pub const MEMORY_ACCESS_OUT_OF_BOUNDS: u16 = 4002;
    pub const MEMORY_ACCESS_UNALIGNED: u16 = 4003;

    // Validation errors (5000-5999)
    pub const VALIDATION_ERROR: u16 = 5000;
    pub const VALIDATION_FAILURE: u16 = 5001;
    pub const CHECKSUM_MISMATCH: u16 = 5002;
    pub const INTEGRITY_VIOLATION: u16 = 5003;
    pub const VERIFICATION_LEVEL_VIOLATION: u16 = 5004;

    // Type System errors (6000-6999)
    pub const INVALID_TYPE: u16 = 6000;
    pub const TYPE_MISMATCH_ERROR: u16 = 6001;
    pub const INVALID_FUNCTION_TYPE: u16 = 6002;
    pub const INVALID_VALUE_TYPE: u16 = 6003;

    // Runtime errors (7000-7999)
    pub const RUNTIME_ERROR: u16 = 7000;
    pub const EXECUTION_TIMEOUT: u16 = 7001;
    pub const FUEL_EXHAUSTED: u16 = 7002;
    pub const POISONED_LOCK: u16 = 7003;

    // System errors (8000-8999)
    pub const SYSTEM_ERROR: u16 = 8000;
    pub const UNSUPPORTED_OPERATION: u16 = 8001;
    pub const CONVERSION_ERROR: u16 = 8002;
    pub const DECODING_ERROR: u16 = 8003;

    pub const UNKNOWN: u16 = 9999;
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::new(
            ErrorCategory::Runtime,
            codes::RUNTIME_ERROR,
            "Test runtime error",
        );
        assert_eq!(err.category, ErrorCategory::Runtime);
        assert_eq!(err.code, codes::RUNTIME_ERROR);
        assert_eq!(err.message, "Test runtime error");
    }

    #[test]
    fn test_error_with_source() {
        let source_err = Error::new(
            ErrorCategory::Memory,
            codes::MEMORY_OUT_OF_BOUNDS,
            "Memory out of bounds",
        );

        let err = Error::with_source(
            ErrorCategory::Runtime,
            codes::RUNTIME_ERROR,
            "Runtime error with source",
            Box::new(source_err),
        );

        assert_eq!(err.category, ErrorCategory::Runtime);
        assert_eq!(err.code, codes::RUNTIME_ERROR);
        assert_eq!(err.message, "Runtime error with source");
        assert!(err.source.is_some());
    }

    #[test]
    fn test_error_category_checks() {
        let err = Error::memory_error("Test memory error");
        assert!(err.is_memory_error());
        assert!(!err.is_resource_error());
    }

    #[test]
    fn test_error_display() {
        let err = Error::new(
            ErrorCategory::Runtime,
            codes::RUNTIME_ERROR,
            "Test runtime error",
        );
        assert_eq!(format!("{}", err), "Test runtime error (code: 7000)");
    }

    #[test]
    fn test_factory_methods() {
        let resource_err = Error::resource_error("Resource error");
        assert_eq!(resource_err.category, ErrorCategory::Resource);
        assert_eq!(resource_err.code, codes::RESOURCE_ERROR);

        let memory_err = Error::memory_error("Memory error");
        assert_eq!(memory_err.category, ErrorCategory::Memory);
        assert_eq!(memory_err.code, codes::MEMORY_OUT_OF_BOUNDS);
    }
}

#[cfg(feature = "alloc")]
impl From<String> for Error {
    fn from(message: String) -> Self {
        Self::new(ErrorCategory::Runtime, codes::UNKNOWN, message)
    }
}

#[cfg(feature = "alloc")]
impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Self::new(ErrorCategory::Runtime, codes::UNKNOWN, message)
    }
}

#[cfg(all(feature = "alloc", feature = "std"))]
impl<T: ErrorSource + Send + Sync + 'static> From<Box<T>> for Error {
    fn from(source: Box<T>) -> Self {
        let category = source.category();
        let code = source.code();
        let message = source.message().to_string();
        let source_box = unsafe {
            let raw_ptr = Box::into_raw(source);
            Box::from_raw(raw_ptr as *mut (dyn ErrorSource + Send + Sync + 'static))
        };

        Self {
            category,
            code,
            message,
            source: Some(source_box),
        }
    }
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
impl<T: ErrorSource + 'static> From<Box<T>> for Error {
    fn from(source: Box<T>) -> Self {
        let category = source.category();
        let code = source.code();
        let message = source.message().to_string();

        Self {
            category,
            code,
            message,
            source: Some(source),
        }
    }
}

// Implement From for standard error types
impl From<core::fmt::Error> for Error {
    fn from(_: core::fmt::Error) -> Self {
        #[cfg(feature = "alloc")]
        return Self::new(
            ErrorCategory::System,
            codes::SYSTEM_ERROR,
            "Formatting error",
        );

        #[cfg(not(feature = "alloc"))]
        return Self::new(ErrorCategory::System, codes::SYSTEM_ERROR, "");
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::new(
            ErrorCategory::System,
            codes::SYSTEM_ERROR,
            format!("IO error: {}", e),
        )
    }
}

// Conversion helpers for error kinds
#[cfg(feature = "alloc")]
impl From<kinds::ValidationError> for Error {
    fn from(e: kinds::ValidationError) -> Self {
        Self::new(ErrorCategory::Validation, codes::VALIDATION_ERROR, e.0)
    }
}

#[cfg(feature = "alloc")]
impl From<kinds::ParseError> for Error {
    fn from(e: kinds::ParseError) -> Self {
        Self::new(ErrorCategory::Runtime, codes::PARSE_ERROR, e.0)
    }
}

#[cfg(feature = "alloc")]
impl From<kinds::OutOfBoundsError> for Error {
    fn from(e: kinds::OutOfBoundsError) -> Self {
        Self::new(ErrorCategory::Memory, codes::MEMORY_OUT_OF_BOUNDS, e.0)
    }
}

#[cfg(feature = "alloc")]
impl From<kinds::InvalidType> for Error {
    fn from(e: kinds::InvalidType) -> Self {
        Self::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, e.0)
    }
}

#[cfg(feature = "alloc")]
impl From<kinds::ResourceError> for Error {
    fn from(e: kinds::ResourceError) -> Self {
        Self::new(ErrorCategory::Resource, codes::RESOURCE_ERROR, e.0)
    }
}

#[cfg(feature = "alloc")]
impl From<kinds::ComponentError> for Error {
    fn from(e: kinds::ComponentError) -> Self {
        Self::new(
            ErrorCategory::Component,
            codes::COMPONENT_LINKING_ERROR,
            e.0,
        )
    }
}

#[cfg(feature = "alloc")]
impl From<kinds::RuntimeError> for Error {
    fn from(e: kinds::RuntimeError) -> Self {
        Self::new(ErrorCategory::Runtime, codes::RUNTIME_ERROR, e.0)
    }
}

#[cfg(feature = "alloc")]
impl From<kinds::PoisonedLockError> for Error {
    fn from(e: kinds::PoisonedLockError) -> Self {
        Self::from_kind(e, codes::POISONED_LOCK, ErrorCategory::Runtime)
    }
}

#[cfg(feature = "alloc")]
impl From<kinds::TypeMismatchError> for Error {
    fn from(e: kinds::TypeMismatchError) -> Self {
        Self::from_kind(e, codes::TYPE_MISMATCH_ERROR, ErrorCategory::Type)
    }
}

#[cfg(feature = "alloc")]
impl From<kinds::MemoryAccessOutOfBoundsError> for Error {
    fn from(e: kinds::MemoryAccessOutOfBoundsError) -> Self {
        Self::from_kind(e, codes::MEMORY_ACCESS_OUT_OF_BOUNDS, ErrorCategory::Memory)
    }
}

#[cfg(feature = "alloc")]
impl From<kinds::TableAccessOutOfBounds> for Error {
    fn from(e: kinds::TableAccessOutOfBounds) -> Self {
        Self::from_kind(e, codes::OUT_OF_BOUNDS_ERROR, ErrorCategory::Memory)
    }
}

impl From<kinds::ConversionError> for Error {
    fn from(e: kinds::ConversionError) -> Self {
        Self::from_kind(e, codes::CONVERSION_ERROR, ErrorCategory::System)
    }
}

impl From<kinds::DivisionByZeroError> for Error {
    fn from(e: kinds::DivisionByZeroError) -> Self {
        Self::from_kind(e, codes::RUNTIME_ERROR, ErrorCategory::Runtime)
    }
}

impl From<kinds::IntegerOverflowError> for Error {
    fn from(e: kinds::IntegerOverflowError) -> Self {
        Self::from_kind(e, codes::RUNTIME_ERROR, ErrorCategory::Runtime)
    }
}

impl From<kinds::StackUnderflow> for Error {
    fn from(e: kinds::StackUnderflow) -> Self {
        Self::from_kind(e, codes::STACK_UNDERFLOW, ErrorCategory::Runtime)
    }
}

impl From<kinds::TypeMismatch> for Error {
    fn from(e: kinds::TypeMismatch) -> Self {
        Self::from_kind(e, codes::TYPE_MISMATCH_ERROR, ErrorCategory::Type)
    }
}

impl From<kinds::InvalidTableIndexError> for Error {
    fn from(e: kinds::InvalidTableIndexError) -> Self {
        Self::from_kind(e, codes::OUT_OF_BOUNDS_ERROR, ErrorCategory::Memory)
    }
}

// Implement the ToErrorCategory trait for Error
impl ToErrorCategory for Error {
    fn to_category(&self) -> ErrorCategory {
        self.category
    }
}

// Implement FromError for Error (self conversion)
impl FromError<Error> for Error {
    fn from_error(error: Error) -> Self {
        error
    }
}

// Standard implementation of FromError for common error types
#[cfg(feature = "std")]
impl FromError<std::io::Error> for Error {
    fn from_error(error: std::io::Error) -> Self {
        #[cfg(feature = "alloc")]
        {
            Self::system_error(format!("IO error: {}", error))
        }
        #[cfg(not(feature = "alloc"))]
        {
            Self::system_error("IO error")
        }
    }
}

// Implement FromError for string types when alloc is available
#[cfg(feature = "alloc")]
impl FromError<String> for Error {
    fn from_error(error: String) -> Self {
        Self::runtime_error(error)
    }
}

// Implement FromError for &str
impl FromError<&str> for Error {
    fn from_error(error: &str) -> Self {
        #[cfg(feature = "alloc")]
        {
            Self::runtime_error(error)
        }
        #[cfg(not(feature = "alloc"))]
        {
            Self::runtime_error(error)
        }
    }
}

// Add FromError implementations for the kinds error types
#[cfg(feature = "alloc")]
impl FromError<kinds::ValidationError> for Error {
    fn from_error(error: kinds::ValidationError) -> Self {
        Self::new(ErrorCategory::Validation, codes::VALIDATION_ERROR, error.0)
    }
}

#[cfg(feature = "alloc")]
impl FromError<kinds::ParseError> for Error {
    fn from_error(error: kinds::ParseError) -> Self {
        Self::new(ErrorCategory::Runtime, codes::PARSE_ERROR, error.0)
    }
}

#[cfg(feature = "alloc")]
impl FromError<kinds::OutOfBoundsError> for Error {
    fn from_error(error: kinds::OutOfBoundsError) -> Self {
        Self::new(ErrorCategory::Memory, codes::MEMORY_OUT_OF_BOUNDS, error.0)
    }
}

#[cfg(feature = "alloc")]
impl FromError<kinds::InvalidType> for Error {
    fn from_error(error: kinds::InvalidType) -> Self {
        Self::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, error.0)
    }
}

#[cfg(feature = "alloc")]
impl FromError<kinds::ResourceError> for Error {
    fn from_error(error: kinds::ResourceError) -> Self {
        Self::new(ErrorCategory::Resource, codes::RESOURCE_ERROR, error.0)
    }
}

#[cfg(feature = "alloc")]
impl FromError<kinds::ComponentError> for Error {
    fn from_error(error: kinds::ComponentError) -> Self {
        Self::new(
            ErrorCategory::Component,
            codes::COMPONENT_LINKING_ERROR,
            error.0,
        )
    }
}

#[cfg(feature = "alloc")]
impl FromError<kinds::RuntimeError> for Error {
    fn from_error(error: kinds::RuntimeError) -> Self {
        Self::new(ErrorCategory::Runtime, codes::RUNTIME_ERROR, error.0)
    }
}

#[cfg(feature = "alloc")]
impl FromError<kinds::PoisonedLockError> for Error {
    fn from_error(error: kinds::PoisonedLockError) -> Self {
        Self::from_kind(error, codes::POISONED_LOCK, ErrorCategory::Runtime)
    }
}
