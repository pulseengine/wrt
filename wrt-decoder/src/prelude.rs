//! Common imports for the decoder crate
//!
//! This module provides a consistent way to import common traits and types
//! that are used throughout the crate, especially for no_std compatibility.

// Re-export from either std or core
#[cfg(feature = "std")]
pub use std::fmt;
#[cfg(feature = "std")]
pub use std::string::ToString;

#[cfg(not(feature = "std"))]
pub use alloc::string::ToString;
#[cfg(not(feature = "std"))]
pub use core::fmt;

// Re-export common collections
#[cfg(feature = "std")]
pub use std::string::String;
#[cfg(feature = "std")]
pub use std::vec::Vec;

#[cfg(not(feature = "std"))]
pub use alloc::string::String;
#[cfg(not(feature = "std"))]
pub use alloc::vec::Vec;

// Re-export error types
pub use wrt_types::{Error, ErrorCategory};
