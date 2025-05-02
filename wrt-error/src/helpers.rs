//! Error helper functions for common error patterns.

#[cfg(feature = "alloc")]
use super::errors::{codes, Error, ErrorCategory};

#[cfg(feature = "std")]
extern crate std;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Use alloc for formatting if std is not enabled
#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::format as alloc_format;

// Use std for formatting if std is enabled
#[cfg(feature = "std")]
use std::format as std_format;

pub use crate::kinds::*;

/// Helper functions for creating index-related errors with consistent formatting
#[cfg(feature = "alloc")]
pub fn create_index_error(index_type: &str, index: usize) -> Error {
    Error::new(
        ErrorCategory::Core,
        codes::INVALID_FUNCTION_INDEX,
        #[cfg(feature = "std")]
        std_format!("Invalid {} index: {}", index_type, index),
        #[cfg(all(not(feature = "std"), feature = "alloc"))]
        alloc_format!("Invalid {} index: {}", index_type, index),
    )
}

/// Helper functions for creating resource limit errors with consistent formatting
#[cfg(feature = "alloc")]
pub fn create_resource_limit_error(resource_type: &str, current: usize, limit: usize) -> Error {
    Error::new(
        ErrorCategory::Resource,
        codes::RESOURCE_LIMIT_EXCEEDED,
        #[cfg(feature = "std")]
        std_format!(
            "{} limit exceeded: current={}, limit={}",
            resource_type,
            current,
            limit
        ),
        #[cfg(all(not(feature = "std"), feature = "alloc"))]
        alloc_format!(
            "{} limit exceeded: current={}, limit={}",
            resource_type,
            current,
            limit
        ),
    )
}

/// Helper for creating memory access errors with detailed context
#[cfg(feature = "alloc")]
pub fn create_memory_access_error(
    address: u64,
    size: u64,
    memory_size: u64,
    operation: &str,
) -> Error {
    Error::new(
        ErrorCategory::Memory,
        codes::MEMORY_ACCESS_OUT_OF_BOUNDS,
        #[cfg(feature = "std")]
        std_format!(
            "Memory access out of bounds during {}: address={}, size={}, memory_size={}",
            operation,
            address,
            size,
            memory_size
        ),
        #[cfg(all(not(feature = "std"), feature = "alloc"))]
        alloc_format!(
            "Memory access out of bounds during {}: address={}, size={}, memory_size={}",
            operation,
            address,
            size,
            memory_size
        ),
    )
}

/// Helper for creating type mismatch errors with expected/actual types
#[cfg(feature = "alloc")]
pub fn create_type_mismatch_error(expected: &str, actual: &str, context: &str) -> Error {
    Error::new(
        ErrorCategory::Type,
        codes::TYPE_MISMATCH_ERROR,
        #[cfg(feature = "std")]
        std_format!(
            "Type mismatch in {}: expected {}, got {}",
            context,
            expected,
            actual
        ),
        #[cfg(all(not(feature = "std"), feature = "alloc"))]
        alloc_format!(
            "Type mismatch in {}: expected {}, got {}",
            context,
            expected,
            actual
        ),
    )
}

/// Helper for creating validation errors with detailed context
#[cfg(feature = "alloc")]
pub fn create_validation_error(validation_type: &str, details: &str) -> Error {
    Error::new(
        ErrorCategory::Validation,
        codes::VALIDATION_ERROR,
        #[cfg(feature = "std")]
        std_format!("Validation failed ({}): {}", validation_type, details),
        #[cfg(all(not(feature = "std"), feature = "alloc"))]
        alloc_format!("Validation failed ({}): {}", validation_type, details),
    )
}

/// Helper for creating resource access errors with context
#[cfg(feature = "alloc")]
pub fn create_resource_access_error(resource_type: &str, operation: &str, reason: &str) -> Error {
    Error::new(
        ErrorCategory::Resource,
        codes::RESOURCE_ACCESS_ERROR,
        #[cfg(feature = "std")]
        std_format!(
            "Failed to {} {} resource: {}",
            operation,
            resource_type,
            reason
        ),
        #[cfg(all(not(feature = "std"), feature = "alloc"))]
        alloc_format!(
            "Failed to {} {} resource: {}",
            operation,
            resource_type,
            reason
        ),
    )
}

/// Helper for creating component errors with context
#[cfg(feature = "alloc")]
pub fn create_component_error(component_name: &str, operation: &str, details: &str) -> Error {
    Error::new(
        ErrorCategory::Component,
        codes::COMPONENT_INSTANTIATION_ERROR,
        #[cfg(feature = "std")]
        std_format!(
            "Component '{}' {} failed: {}",
            component_name,
            operation,
            details
        ),
        #[cfg(all(not(feature = "std"), feature = "alloc"))]
        alloc_format!(
            "Component '{}' {} failed: {}",
            component_name,
            operation,
            details
        ),
    )
}

/// Create a parse error with standard error code
#[cfg(feature = "alloc")]
pub fn create_simple_parse_error(message: impl Into<String>) -> Error {
    Error::new(ErrorCategory::Validation, codes::PARSE_ERROR, message)
}

/// Create a validation error with standard error code
#[cfg(feature = "alloc")]
pub fn create_simple_validation_error(message: impl Into<String>) -> Error {
    Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR, message)
}

/// Create a type error with standard error code
#[cfg(feature = "alloc")]
pub fn create_simple_type_error(message: impl Into<String>) -> Error {
    Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, message)
}

/// Create a memory error with standard error code
#[cfg(feature = "alloc")]
pub fn create_simple_memory_error(message: impl Into<String>) -> Error {
    Error::new(ErrorCategory::Memory, codes::MEMORY_OUT_OF_BOUNDS, message)
}

/// Create a resource error with standard error code
#[cfg(feature = "alloc")]
pub fn create_simple_resource_error(message: impl Into<String>) -> Error {
    Error::new(ErrorCategory::Resource, codes::RESOURCE_ERROR, message)
}

/// Create a component error with standard error code
#[cfg(feature = "alloc")]
pub fn create_simple_component_error(message: impl Into<String>) -> Error {
    Error::new(
        ErrorCategory::Component,
        codes::COMPONENT_LINKING_ERROR,
        message,
    )
}

/// Create a runtime error with standard error code
#[cfg(feature = "alloc")]
pub fn create_simple_runtime_error(message: impl Into<String>) -> Error {
    Error::new(ErrorCategory::Runtime, codes::RUNTIME_ERROR, message)
}

/// Create a system error with standard error code
#[cfg(feature = "alloc")]
pub fn create_simple_system_error(message: impl Into<String>) -> Error {
    Error::new(ErrorCategory::System, codes::SYSTEM_ERROR, message)
}

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;

    #[test]
    fn test_index_error() {
        let error = create_index_error("function", 42);
        assert_eq!(error.to_string(), "Invalid function index: 42 (code: 2000)");
    }

    #[test]
    fn test_resource_limit_error() {
        let error = create_resource_limit_error("memory", 1024, 1000);
        assert_eq!(
            error.to_string(),
            "memory limit exceeded: current=1024, limit=1000 (code: 3001)"
        );
    }

    #[test]
    fn test_memory_access_error() {
        let error = create_memory_access_error(100, 32, 64, "load");
        assert_eq!(
            error.to_string(),
            "Memory access out of bounds during load: address=100, size=32, memory_size=64 (code: 4002)"
        );
    }

    #[test]
    fn test_type_mismatch_error() {
        let error = create_type_mismatch_error("i32", "f64", "function call");
        assert_eq!(
            error.to_string(),
            "Type mismatch in function call: expected i32, got f64 (code: 6001)"
        );
    }
}

// No-std versions
#[cfg(not(feature = "alloc"))]
pub fn create_simple_parse_error<T: core::fmt::Display>(message: T) -> Error {
    Error::new(ErrorCategory::Validation, codes::PARSE_ERROR, message)
}

#[cfg(not(feature = "alloc"))]
pub fn create_simple_validation_error<T: core::fmt::Display>(message: T) -> Error {
    Error::new(ErrorCategory::Validation, codes::VALIDATION_ERROR, message)
}

#[cfg(not(feature = "alloc"))]
pub fn create_simple_type_error<T: core::fmt::Display>(message: T) -> Error {
    Error::new(ErrorCategory::Type, codes::TYPE_MISMATCH_ERROR, message)
}

#[cfg(not(feature = "alloc"))]
pub fn create_simple_memory_error<T: core::fmt::Display>(message: T) -> Error {
    Error::new(ErrorCategory::Memory, codes::MEMORY_OUT_OF_BOUNDS, message)
}

#[cfg(not(feature = "alloc"))]
pub fn create_simple_resource_error<T: core::fmt::Display>(message: T) -> Error {
    Error::new(ErrorCategory::Resource, codes::RESOURCE_ERROR, message)
}

#[cfg(not(feature = "alloc"))]
pub fn create_simple_component_error<T: core::fmt::Display>(message: T) -> Error {
    Error::new(
        ErrorCategory::Component,
        codes::COMPONENT_LINKING_ERROR,
        message,
    )
}

#[cfg(not(feature = "alloc"))]
pub fn create_simple_runtime_error<T: core::fmt::Display>(message: T) -> Error {
    Error::new(ErrorCategory::Runtime, codes::RUNTIME_ERROR, message)
}

#[cfg(not(feature = "alloc"))]
pub fn create_simple_system_error<T: core::fmt::Display>(message: T) -> Error {
    Error::new(ErrorCategory::System, codes::SYSTEM_ERROR, message)
}
