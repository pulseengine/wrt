//! Defines the main `Error` struct.

use super::kinds;
use super::source::ErrorSource;
use core::fmt::{self, Debug, Display};

// Conditionally bring in alloc types when 'alloc' feature is enabled.
#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::string::String;

/// The main error type for WRT operations.
///
/// This type can only be used when the `alloc` feature is enabled.
#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct Error {
    kind: Box<dyn ErrorSource + Send + Sync>,
}

#[cfg(feature = "alloc")]
impl Error {
    /// Creates a new error from any type implementing `ErrorSource`.
    pub fn new<E>(error: E) -> Self
    where
        E: ErrorSource + Send + Sync + 'static,
    {
        Error {
            kind: Box::new(error),
        }
    }

    /// Returns a reference to the underlying error value if it is of type `T`.
    ///
    /// This function requires the `std` feature since it needs the `Any` trait.
    #[cfg(feature = "std")]
    pub fn downcast_ref<T: ErrorSource + 'static>(&self) -> Option<&T> {
        // For this to work properly, ErrorSource would need to extend Any,
        // but we can't modify that trait hierarchy without breaking changes.
        // For now, we'll return None, and we might revisit this in the future.
        None
    }

    // Factory methods for common error types

    /// Creates an ExecutionError with the given message
    pub fn execution_error(message: impl Into<String>) -> Self {
        Error::new(kinds::ExecutionError(message.into()))
    }

    /// Creates an InvalidTypeError with the given message
    pub fn invalid_type(message: impl Into<String>) -> Self {
        Error::new(kinds::InvalidTypeError(message.into()))
    }

    /// Creates a StackUnderflowError
    pub fn stack_underflow() -> Self {
        Error::new(kinds::StackUnderflowError)
    }

    /// Creates a DivisionByZeroError
    pub fn division_by_zero() -> Self {
        Error::new(kinds::DivisionByZeroError)
    }

    /// Creates an IntegerOverflowError
    pub fn integer_overflow() -> Self {
        Error::new(kinds::IntegerOverflowError)
    }

    /// Creates an InvalidMemoryAccessError
    pub fn invalid_memory_access() -> Self {
        Error::new(kinds::InvalidMemoryAccessError)
    }

    /// Creates a MemoryAccessOutOfBoundsError
    pub fn memory_access_out_of_bounds(address: u64, length: u64) -> Self {
        Error::new(kinds::MemoryAccessOutOfBoundsError { address, length })
    }

    /// Creates an InvalidFunctionIndexError
    pub fn invalid_function_index(index: usize) -> Self {
        Error::new(kinds::InvalidFunctionIndexError(index))
    }

    /// Creates a Trap with the given message
    pub fn trap(message: impl Into<String>) -> Self {
        Error::new(kinds::Trap(message.into()))
    }

    /// Creates a NotImplementedError with the given message
    pub fn not_implemented(message: impl Into<String>) -> Self {
        Error::new(kinds::NotImplementedError(message.into()))
    }

    /// Creates an InitializationError with the given message
    pub fn initialization_error(message: impl Into<String>) -> Self {
        Error::new(kinds::InitializationError(message.into()))
    }

    /// Creates a MemoryAccessError with the given message
    pub fn memory_access_error(message: impl Into<String>) -> Self {
        Error::new(kinds::MemoryAccessError(message.into()))
    }

    /// Creates a TypeMismatchError with the given message
    pub fn type_mismatch(message: impl Into<String>) -> Self {
        Error::new(kinds::TypeMismatchError(message.into()))
    }

    /// Creates a ValidationFailureError with the given message
    pub fn validation_failure(message: impl Into<String>) -> Self {
        Error::new(kinds::ValidationFailureError(message.into()))
    }

    /// Creates a ChecksumMismatchError with the expected and actual checksums
    pub fn checksum_mismatch(expected: u32, actual: u32, description: impl Into<String>) -> Self {
        Error::new(kinds::ChecksumMismatchError {
            expected,
            actual,
            description: description.into(),
        })
    }

    /// Creates a BoundedCapacityExceededError with collection info
    pub fn bounded_capacity_exceeded(
        collection_type: impl Into<String>,
        capacity: usize,
        attempted_size: usize,
    ) -> Self {
        Error::new(kinds::BoundedCapacityExceededError {
            collection_type: collection_type.into(),
            capacity,
            attempted_size,
        })
    }

    /// Creates a BoundedCollectionAccessError with access details
    pub fn bounded_collection_access(
        collection_type: impl Into<String>,
        index: usize,
        size: usize,
    ) -> Self {
        Error::new(kinds::BoundedCollectionAccessError {
            collection_type: collection_type.into(),
            index,
            size,
        })
    }

    /// Creates an IntegrityViolationError with the given message
    pub fn integrity_violation(message: impl Into<String>) -> Self {
        Error::new(kinds::IntegrityViolationError(message.into()))
    }

    /// Creates a VerificationLevelViolationError with operation and level details
    pub fn verification_level_violation(
        operation: impl Into<String>,
        required_level: impl Into<String>,
        current_level: impl Into<String>,
    ) -> Self {
        Error::new(kinds::VerificationLevelViolationError {
            operation: operation.into(),
            required_level: required_level.into(),
            current_level: current_level.into(),
        })
    }

    // Component Model error factory methods

    /// Creates an InvalidFunctionIndex error with the given index
    pub fn invalid_function_idx(index: usize) -> Self {
        Error::new(kinds::InvalidFunctionIndex(index))
    }

    /// Creates a TypeMismatch error with the given message
    pub fn type_mismatch_error(message: impl Into<String>) -> Self {
        Error::new(kinds::TypeMismatch(message.into()))
    }

    /// Creates an EncodingError with the given message
    pub fn encoding_error(message: impl Into<String>) -> Self {
        Error::new(kinds::EncodingError(message.into()))
    }

    /// Creates an ExecutionLimitExceeded error with the given message
    pub fn execution_limit_exceeded(message: impl Into<String>) -> Self {
        Error::new(kinds::ExecutionLimitExceeded(message.into()))
    }

    /// Creates a ResourceError with the given message
    pub fn resource_error(message: impl Into<String>) -> Self {
        Error::new(kinds::ResourceError(message.into()))
    }

    /// Creates a ComponentInstantiationError with the given message
    pub fn component_instantiation_error(message: impl Into<String>) -> Self {
        Error::new(kinds::ComponentInstantiationError(message.into()))
    }

    /// Creates a CanonicalABIError with the given message
    pub fn canonical_abi_error(message: impl Into<String>) -> Self {
        Error::new(kinds::CanonicalABIError(message.into()))
    }

    /// Creates a ComponentLinkingError with the given message
    pub fn component_linking_error(message: impl Into<String>) -> Self {
        Error::new(kinds::ComponentLinkingError(message.into()))
    }

    /// Creates a specific MemoryAccessError with the given message
    pub fn memory_access_err(message: impl Into<String>) -> Self {
        Error::new(kinds::MemoryAccessError(message.into()))
    }

    /// Creates a ConversionError with the given message
    pub fn conversion_error(message: impl Into<String>) -> Self {
        Error::new(kinds::ConversionError(message.into()))
    }

    /// Creates a ResourceLimitExceeded error with the given message
    pub fn resource_limit_exceeded(message: impl Into<String>) -> Self {
        Error::new(kinds::ResourceLimitExceeded(message.into()))
    }

    /// Creates a specific ResourceAccessError with the given message
    pub fn resource_access_error(message: impl Into<String>) -> Self {
        Error::new(kinds::ResourceAccessError(message.into()))
    }

    /// Creates an OutOfBoundsAccess error with the given message
    pub fn out_of_bounds_access(message: impl Into<String>) -> Self {
        Error::new(kinds::OutOfBoundsAccess(message.into()))
    }

    /// Creates an InvalidValueError with the given message
    pub fn invalid_value_error(message: impl Into<String>) -> Self {
        Error::new(kinds::InvalidValue(message.into()))
    }

    /// Creates an InvalidData error with the given message
    pub fn invalid_data(message: impl Into<String>) -> Self {
        Error::new(kinds::InvalidValue(message.into()))
    }

    /// Converts any error type implementing ErrorSource into an Error.
    ///
    /// Since we can't implement From<E> for Error due to conflicts with the
    /// standard library, we provide this as an alternative.
    pub fn from<E>(error: E) -> Self
    where
        E: ErrorSource + Send + Sync + 'static,
    {
        Error::new(error)
    }

    /// Checks if the error is a FuelExhaustedError
    pub fn is_fuel_exhausted(&self) -> bool {
        // Since we can't use downcast_ref reliably without std,
        // we'll check by comparing the error message
        self.kind.to_string().contains("Fuel exhausted")
    }
}

// Display shows only the top-level error message.
#[cfg(feature = "alloc")]
impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.kind, f)
    }
}

// Implement the standard Error trait only when `std` feature is available.
#[cfg(all(feature = "std", feature = "alloc"))]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // In std context, we implement source properly
        // Replace ambiguous call with explicit trait method
        ErrorSource::source(&self.kind).and({
            // Safely handle the conversion - we can't directly cast between trait objects
            // Instead, we'll return None for now until we implement proper conversion
            None
        })
    }
}

// Implement ErrorSource for Error to allow error chaining
#[cfg(feature = "alloc")]
impl ErrorSource for Error {
    #[cfg(feature = "std")]
    fn source(&self) -> Option<&(dyn ErrorSource + 'static)> {
        self.kind.source()
    }
}

// Implement Clone for Error to allow duplication
#[cfg(feature = "alloc")]
impl Clone for Error {
    fn clone(&self) -> Self {
        // Create a new error with the same display representation
        Error::new(kinds::ExecutionError(format!("{}", self)))
    }
}

#[cfg(feature = "std")]
impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::new(crate::kinds::DecodingError(format!(
            "UTF-8 conversion error: {}",
            e
        )))
    }
}

#[cfg(feature = "std")]
impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        // Compare error messages for equality
        // This is a simplified implementation, could be improved
        self.to_string() == other.to_string()
    }
}

#[cfg(feature = "std")]
impl Eq for Error {}
