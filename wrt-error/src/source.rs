//! Defines the `ErrorSource` trait and implements it for common error types.

use core::fmt::{Debug, Display};

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::string::String;

/// The trait all errors must implement to be used with `wrt_error::Error`.
///
/// This is similar to `std::error::Error` but usable in `no_std` environments
/// (provided the `alloc` feature is enabled).
pub trait ErrorSource: Debug + Display {
    /// Returns the lower-level source of this error, if any.
    ///
    /// This is only implemented when the `std` feature is enabled.
    #[cfg(feature = "std")]
    fn source(&self) -> Option<&(dyn ErrorSource + 'static)> {
        None
    }

    /// Returns the error code for this error type
    ///
    /// By default, returns 0 (unspecified error)
    fn code(&self) -> u16 {
        0 // Default error code (unspecified)
    }

    /// Returns a string message for this error
    ///
    /// By default, uses Display implementation
    #[cfg(feature = "alloc")]
    fn message(&self) -> String {
        format!("{}", self)
    }
}

// Implement for basic String errors (requires alloc)
#[cfg(feature = "alloc")]
impl ErrorSource for String {}

#[cfg(feature = "alloc")]
impl ErrorSource for &str {}

// Implement for std::io::Error when std is enabled
#[cfg(feature = "std")]
impl ErrorSource for std::io::Error {
    fn source(&self) -> Option<&(dyn ErrorSource + 'static)> {
        // We can't directly cast std::error::Error to ErrorSource
        // so we return None for now
        None
    }
}

// Implement for other crates when feature flags are enabled

// Implement for wasmparser::BinaryReaderError
#[cfg(feature = "wasmparser")]
impl ErrorSource for wasmparser::BinaryReaderError {
    #[cfg(feature = "std")]
    fn source(&self) -> Option<&(dyn ErrorSource + 'static)> {
        None
    }
}

// Implement for serde_json::Error
#[cfg(feature = "serde_json")]
impl ErrorSource for serde_json::Error {
    #[cfg(feature = "std")]
    fn source(&self) -> Option<&(dyn ErrorSource + 'static)> {
        None
    }
}

// Implement for bincode::error::EncodeError and bincode::error::DecodeError
// Note: bincode 2.0 changed the error types from a single Error to specific EncodeError and DecodeError
#[cfg(feature = "bincode")]
impl ErrorSource for bincode::error::EncodeError {
    #[cfg(feature = "std")]
    fn source(&self) -> Option<&(dyn ErrorSource + 'static)> {
        None
    }
}

#[cfg(feature = "bincode")]
impl ErrorSource for bincode::error::DecodeError {
    #[cfg(feature = "std")]
    fn source(&self) -> Option<&(dyn ErrorSource + 'static)> {
        None
    }
}

// Implement for wat::Error
#[cfg(feature = "wat")]
impl ErrorSource for wat::Error {
    #[cfg(feature = "std")]
    fn source(&self) -> Option<&(dyn ErrorSource + 'static)> {
        None
    }
}

// Implement for wasi_common::Error
#[cfg(feature = "wasi")]
impl ErrorSource for wasi_common::Error {
    #[cfg(feature = "std")]
    fn source(&self) -> Option<&(dyn ErrorSource + 'static)> {
        None
    }
}

// cap-std uses std::io::Error for most of its error handling,
// so we don't need to implement a specific handler for cap-std errors

// Implement for Box<dyn ErrorSource + ...>
#[cfg(feature = "alloc")]
impl ErrorSource for Box<dyn ErrorSource + Send + Sync + 'static> {
    #[cfg(feature = "std")]
    fn source(&self) -> Option<&(dyn ErrorSource + 'static)> {
        (**self).source()
    }
}
