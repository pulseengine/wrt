//! Helper crate for WebAssembly Runtime
//!
//! This crate provides common utilities and helpers for the WebAssembly Runtime.

#![no_std]

/// Version of the helper crate
pub const VERSION: &str = env!(\CARGO_PKG_VERSION\);

/// Helper function to check if a feature is enabled
#[macro_export]
macro_rules! has_feature {
    (xpr) => {
        cfg!(feature = )
    };
}

