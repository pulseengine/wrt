//! WRT Error handling library
//!
//! This library provides a comprehensive error handling system for the WRT runtime.
//! It includes error types, helper functions, and utilities for creating and managing errors.
//!
//! # Error Categories
//!
//! Errors are organized into several categories, each with its own range of error codes:
//!
//! ## Core Errors (1000-1008)
//! - Stack underflow
//! - Memory access violations
//! - Instance index errors
//! - Execution errors
//! - Type mismatches
//!
//! ## Runtime Errors (2000-2021)
//! - Invalid indices (local, global, function)
//! - Memory and table access errors
//! - Resource exhaustion
//! - Validation failures
//! - Parse errors
//!
//! ## Component Errors (3000-3013)
//! - Function index errors
//! - Type mismatches
//! - Resource limits
//! - Component lifecycle errors
//! - ABI errors
//!
//! ## System Errors (0x1000-0x1001)
//! - Async operation errors
//! - Threading errors
//!
//! # Usage
//!
//! The library provides both low-level error types and high-level helper functions:
//!
//! ```rust
//! use wrt_error::{Error, helpers};
//!
//! // Using helper functions for common errors
//! let error = helpers::create_index_error("function", 42);
//! let error = helpers::create_memory_access_error(100, 32, 64, "load");
//!
//! // Direct error creation
//! let error = Error::execution_error("Failed to execute instruction");
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::missing_panics_doc)]

// Import std when available
#[cfg(feature = "std")]
extern crate std;

// Import alloc when needed for no_std
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Modules
pub mod context;
pub mod errors;
pub mod helpers;
pub mod kinds;

// Include verification module conditionally, but exclude during coverage builds
#[cfg(all(not(coverage), doc))]
pub mod verify;

// Re-export key types
pub use errors::Error;
pub use errors::{codes, ErrorCategory, ErrorSource};

#[cfg(feature = "alloc")]
pub use context::ResultExt;

/// A specialized `Result` type for WRT operations.
/// When the `alloc` feature is enabled, this defaults to using `wrt_error::Error` as the error type.
/// When `alloc` is not available, the specific error type must be provided.
pub type Result<T, E = Error> = core::result::Result<T, E>;

// Re-export error kinds for convenience
pub use kinds::{
    component_error, invalid_type, out_of_bounds_error, parse_error, poisoned_lock_error,
    resource_error, runtime_error, validation_error, ComponentError, InvalidType, OutOfBoundsError,
    ParseError, PoisonedLockError, ResourceError, RuntimeError, ValidationError,
};

#[cfg(all(test, feature = "alloc"))]
mod tests {
    use super::*;

    #[test]
    fn test_error_categories() {
        let resource_err = Error::resource_error("Resource not found");
        assert!(resource_err.is_resource_error());
        assert!(!resource_err.is_memory_error());

        let memory_err = Error::memory_error("Memory access out of bounds");
        assert!(memory_err.is_memory_error());
        assert!(!resource_err.is_resource_error());
    }

    #[test]
    fn test_error_codes() {
        let err = Error::resource_error("Test error");
        assert_eq!(err.code, codes::RESOURCE_ERROR);
    }
}

// Re-export additional helpers
pub use helpers::*;
