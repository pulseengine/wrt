//! Common traits for type conversions
//!
//! This module provides common traits used for type conversions between format
//! and runtime representations.

/// Trait for types that can be converted from a format representation
pub trait FromFormat<T> {
    /// Convert from a format representation
    fn from_format(format: &T) -> Self;
}

/// Trait for types that can be converted to a format representation
pub trait ToFormat<T> {
    /// Convert to a format representation
    fn to_format(&self) -> T;
}
