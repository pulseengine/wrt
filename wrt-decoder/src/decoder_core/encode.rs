//! WebAssembly module encoding
//!
//! This module provides functionality for encoding WebAssembly modules.

/// Re-export the encode_module function from the module module
#[cfg(feature = "alloc")]
pub use crate::module::encode_module;

/// Encode a WebAssembly module to binary format
///
/// This is a wrapper around the `module::encode_module` function
/// that provides a more convenient API for encoding modules.
///
/// # Arguments
///
/// * `module` - The module to encode
///
/// # Returns
///
/// * `Result<Vec<u8>>` - The encoded module or an error
#[cfg(feature = "alloc")]
pub fn encode(module: &crate::module::Module) -> crate::prelude::Result<crate::prelude::Vec<u8>> {
    crate::module::encode_module(module)
}
