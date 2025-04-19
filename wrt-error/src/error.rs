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
        // In std context, we should ideally return the underlying error source
        // but we can't safely convert between the trait objects
        None
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
