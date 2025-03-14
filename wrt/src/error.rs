use crate::String;
use std::fmt;

/// Represents errors that can occur in the WebAssembly runtime.
///
/// This enum covers the various categories of errors that can occur during WebAssembly
/// module validation, execution, parsing, I/O operations, and component model operations.
///
/// # Examples
///
/// ```
/// use wrt::Error;
///
/// let err = Error::Validation("Invalid type".to_string());
/// assert!(err.to_string().contains("Validation error"));
/// ```
#[derive(Debug)]
pub enum Error {
    /// Represents validation errors that occur when a WebAssembly module
    /// fails to meet the WebAssembly specification requirements.
    Validation(String),

    /// Represents errors that occur during the execution of a WebAssembly module,
    /// such as out-of-bounds memory access, stack overflow, or type mismatches.
    Execution(String),

    /// Represents input/output errors, typically when reading from or writing to
    /// a file system or other I/O operations.
    IO(String),

    /// Represents parsing errors that occur when decoding a WebAssembly binary
    /// or parsing a WebAssembly text format.
    Parse(String),

    /// Represents errors related to the WebAssembly Component Model, such as
    /// component instantiation failures or interface type mismatches.
    Component(String),

    /// Represents custom errors that don't fit into the other categories.
    /// This is useful for extension points or for wrapping errors from
    /// other libraries.
    Custom(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Validation(msg) => write!(f, "Validation error: {}", msg),
            Error::Execution(msg) => write!(f, "Execution error: {}", msg),
            Error::IO(msg) => write!(f, "IO error: {}", msg),
            Error::Parse(msg) => write!(f, "Parse error: {}", msg),
            Error::Component(msg) => write!(f, "Component error: {}", msg),
            Error::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// A type alias for Result with the error type fixed to [`Error`].
///
/// This allows for more concise return types in functions that can fail with
/// a WebAssembly runtime error.
///
/// # Examples
///
/// ```
/// use wrt::{Result, Error};
///
/// fn some_operation() -> Result<i32> {
///     // Implementation that might fail
///     Ok(42)
/// }
/// ```
pub type Result<T> = std::result::Result<T, Error>;
