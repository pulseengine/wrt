//! Logging infrastructure for the WebAssembly Runtime (WRT).
//!
//! This crate provides logging functionality for WebAssembly components,
//! allowing components to log messages to the host environment.
//! It extends the wrt-host crate with logging-specific capabilities.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "kani", feature(kani))]

// When no_std but alloc is available
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{boxed::Box, string::String, vec::Vec};

#[cfg(feature = "std")]
use std::{boxed::Box, string::String, vec::Vec};

// Reexports for convenience
pub use wrt_error::{Error, Result};
pub use wrt_host::CallbackRegistry;

// Export modules
pub mod handler;
pub mod level;
pub mod operation;

// Reexport types
pub use handler::{LogHandler, LoggingExt};
pub use level::LogLevel;
pub use operation::LogOperation;

// Include verification module when the kani feature is enabled
#[cfg(feature = "kani")]
pub mod verify;
