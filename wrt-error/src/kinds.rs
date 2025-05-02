// Define a custom string-like type that works in all environments
#[cfg(feature = "std")]
extern crate std;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Use alloc String if alloc is enabled but not std
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use alloc::format;
#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use alloc::string::String;

// Use std String if std is enabled
#[cfg(feature = "std")]
pub use std::format;
#[cfg(feature = "std")]
pub use std::string::String;

// For no_std without alloc, we use a placeholder String that can be constructed but doesn't store content
#[cfg(not(any(feature = "std", feature = "alloc")))]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct String {
    _private: (),
}

#[cfg(not(any(feature = "std", feature = "alloc")))]
impl String {
    pub fn new() -> Self {
        Self { _private: () }
    }

    // Utility method to create a String from a static str
    pub fn from_static(_msg: &'static str) -> Self {
        Self::new()
    }
}

// Implement From for &'static str in no_std mode - just creates an empty placeholder
#[cfg(not(any(feature = "std", feature = "alloc")))]
impl From<&'static str> for String {
    fn from(_value: &'static str) -> Self {
        Self::new()
    }
}

/// Validation error for integrity or consistency checks
#[derive(Debug, Clone)]
pub struct ValidationError(pub String);

/// Out of bounds error for memory access or index violations
#[derive(Debug, Clone)]
pub struct OutOfBoundsError(pub String);

/// Parse error for decoding binary formats
#[derive(Debug, Clone)]
pub struct ParseError(pub String);

/// Type error for type mismatches or invalid types
#[derive(Debug, Clone)]
pub struct InvalidType(pub String);

/// Type error for conversions that failed
#[derive(Debug, Clone)]
pub struct ConversionError(pub String);

/// Division by zero error
#[derive(Debug, Clone)]
pub struct DivisionByZeroError;

/// Integer overflow error
#[derive(Debug, Clone)]
pub struct IntegerOverflowError;

/// Stack underflow error
#[derive(Debug, Clone)]
pub struct StackUnderflow;

/// Type mismatch error
#[derive(Debug, Clone)]
pub struct TypeMismatch(pub String);

/// Invalid table index error
#[derive(Debug, Clone)]
pub struct InvalidTableIndexError(pub u32);

/// Resource error for resource access or creation issues
#[derive(Debug, Clone)]
pub struct ResourceError(pub String);

/// Component error for component instantiation or linking issues
#[derive(Debug, Clone)]
pub struct ComponentError(pub String);

/// Runtime error for generic execution issues
#[derive(Debug, Clone)]
pub struct RuntimeError(pub String);

/// Poisoned lock error for mutex failures
#[derive(Debug, Clone)]
pub struct PoisonedLockError(pub String);

/// Memory access out of bounds error
#[derive(Debug, Clone)]
pub struct MemoryAccessOutOfBoundsError {
    pub address: u64,
    pub length: u64,
}

/// Type mismatch error
#[derive(Debug, Clone)]
pub struct TypeMismatchError(pub String);

/// Table access out of bounds error
#[derive(Debug, Clone)]
pub struct TableAccessOutOfBounds;

/// Helper function for creating ValidationError
pub fn validation_error(message: impl Into<String>) -> ValidationError {
    ValidationError(message.into())
}

/// Helper function for creating OutOfBoundsError
pub fn out_of_bounds_error(message: impl Into<String>) -> OutOfBoundsError {
    OutOfBoundsError(message.into())
}

/// Helper function for creating ParseError
pub fn parse_error(message: impl Into<String>) -> ParseError {
    ParseError(message.into())
}

/// Helper function for creating InvalidType
pub fn invalid_type(message: impl Into<String>) -> InvalidType {
    InvalidType(message.into())
}

/// Helper function for creating ConversionError
pub fn conversion_error(message: impl Into<String>) -> ConversionError {
    ConversionError(message.into())
}

/// Helper function for creating DivisionByZeroError
pub fn division_by_zero_error() -> DivisionByZeroError {
    DivisionByZeroError
}

/// Helper function for creating IntegerOverflowError
pub fn integer_overflow_error() -> IntegerOverflowError {
    IntegerOverflowError
}

/// Helper function for creating StackUnderflow
pub fn stack_underflow() -> StackUnderflow {
    StackUnderflow
}

/// Helper function for creating TypeMismatch
pub fn type_mismatch(message: impl Into<String>) -> TypeMismatch {
    TypeMismatch(message.into())
}

/// Helper function for creating InvalidTableIndexError
pub fn invalid_table_index_error(index: u32) -> InvalidTableIndexError {
    InvalidTableIndexError(index)
}

/// Helper function for creating ResourceError
pub fn resource_error(message: impl Into<String>) -> ResourceError {
    ResourceError(message.into())
}

/// Helper function for creating ComponentError
pub fn component_error(message: impl Into<String>) -> ComponentError {
    ComponentError(message.into())
}

/// Helper function for creating RuntimeError
pub fn runtime_error(message: impl Into<String>) -> RuntimeError {
    RuntimeError(message.into())
}

/// Helper function for creating PoisonedLockError
pub fn poisoned_lock_error(message: impl Into<String>) -> PoisonedLockError {
    PoisonedLockError(message.into())
}

/// Helper function for creating TypeMismatchError
pub fn type_mismatch_error(message: impl Into<String>) -> TypeMismatchError {
    TypeMismatchError(message.into())
}

/// Implementation of the Display trait for ValidationError
impl core::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(any(feature = "std", feature = "alloc"))]
        return write!(f, "Validation error: {}", self.0);

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        return write!(f, "Validation error");
    }
}

/// Implementation of the Display trait for OutOfBoundsError
impl core::fmt::Display for OutOfBoundsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(any(feature = "std", feature = "alloc"))]
        return write!(f, "Out of bounds error: {}", self.0);

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        return write!(f, "Out of bounds error");
    }
}

/// Implementation of the Display trait for ParseError
impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(any(feature = "std", feature = "alloc"))]
        return write!(f, "Parse error: {}", self.0);

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        return write!(f, "Parse error");
    }
}

/// Implementation of the Display trait for InvalidType
impl core::fmt::Display for InvalidType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(any(feature = "std", feature = "alloc"))]
        return write!(f, "Invalid type: {}", self.0);

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        return write!(f, "Invalid type");
    }
}

/// Implementation of the Display trait for ConversionError
impl core::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(any(feature = "std", feature = "alloc"))]
        return write!(f, "Conversion error: {}", self.0);

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        return write!(f, "Conversion error");
    }
}

/// Implementation of the Display trait for DivisionByZeroError
impl core::fmt::Display for DivisionByZeroError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Division by zero error")
    }
}

/// Implementation of the Display trait for IntegerOverflowError
impl core::fmt::Display for IntegerOverflowError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Integer overflow error")
    }
}

/// Implementation of the Display trait for StackUnderflow
impl core::fmt::Display for StackUnderflow {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Stack underflow")
    }
}

/// Implementation of the Display trait for TypeMismatch
impl core::fmt::Display for TypeMismatch {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(any(feature = "std", feature = "alloc"))]
        return write!(f, "Type mismatch: {}", self.0);

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        return write!(f, "Type mismatch");
    }
}

/// Implementation of the Display trait for InvalidTableIndexError
impl core::fmt::Display for InvalidTableIndexError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Invalid table index: {}", self.0)
    }
}

/// Implementation of the Display trait for ResourceError
impl core::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(any(feature = "std", feature = "alloc"))]
        return write!(f, "Resource error: {}", self.0);

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        return write!(f, "Resource error");
    }
}

/// Implementation of the Display trait for ComponentError
impl core::fmt::Display for ComponentError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(any(feature = "std", feature = "alloc"))]
        return write!(f, "Component error: {}", self.0);

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        return write!(f, "Component error");
    }
}

/// Implementation of the Display trait for RuntimeError
impl core::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(any(feature = "std", feature = "alloc"))]
        return write!(f, "Runtime error: {}", self.0);

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        return write!(f, "Runtime error");
    }
}

/// Implementation of the Display trait for PoisonedLockError
impl core::fmt::Display for PoisonedLockError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(any(feature = "std", feature = "alloc"))]
        return write!(f, "Poisoned lock error: {}", self.0);

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        return write!(f, "Poisoned lock error");
    }
}

/// Implementation of the Display trait for MemoryAccessOutOfBoundsError
impl core::fmt::Display for MemoryAccessOutOfBoundsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(any(feature = "std", feature = "alloc"))]
        return write!(
            f,
            "Memory access out of bounds: address 0x{:x}, length {}",
            self.address, self.length
        );

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        return write!(f, "Memory access out of bounds");
    }
}

/// Implementation of the Display trait for TypeMismatchError
impl core::fmt::Display for TypeMismatchError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(any(feature = "std", feature = "alloc"))]
        return write!(f, "Type mismatch: {}", self.0);

        #[cfg(not(any(feature = "std", feature = "alloc")))]
        return write!(f, "Type mismatch");
    }
}

/// Implementation of the Display trait for TableAccessOutOfBounds
impl core::fmt::Display for TableAccessOutOfBounds {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Table access out of bounds")
    }
}

// Add From implementations for the new error types

#[cfg(feature = "alloc")]
impl From<ConversionError> for String {
    fn from(e: ConversionError) -> String {
        e.to_string()
    }
}

#[cfg(feature = "alloc")]
impl From<DivisionByZeroError> for String {
    fn from(e: DivisionByZeroError) -> String {
        e.to_string()
    }
}

#[cfg(feature = "alloc")]
impl From<IntegerOverflowError> for String {
    fn from(e: IntegerOverflowError) -> String {
        e.to_string()
    }
}

#[cfg(feature = "alloc")]
impl From<StackUnderflow> for String {
    fn from(e: StackUnderflow) -> String {
        e.to_string()
    }
}

#[cfg(feature = "alloc")]
impl From<TypeMismatch> for String {
    fn from(e: TypeMismatch) -> String {
        e.to_string()
    }
}

#[cfg(feature = "alloc")]
impl From<InvalidTableIndexError> for String {
    fn from(e: InvalidTableIndexError) -> String {
        e.to_string()
    }
}

#[cfg(feature = "alloc")]
impl From<ValidationError> for String {
    fn from(e: ValidationError) -> String {
        e.to_string()
    }
}
