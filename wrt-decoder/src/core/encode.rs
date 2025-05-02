//! WebAssembly Core Module Encoder
//!
//! Functions for encoding WebAssembly core modules to binary format.

use crate::Result;
use wrt_format::Module;

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::vec::Vec;

/// Encode a WebAssembly module to binary format
///
/// This function takes a structured Module representation and encodes it
/// to WebAssembly binary format.
///
/// # Arguments
///
/// * `module` - The structured Module to encode
///
/// # Returns
///
/// * `Result<Vec<u8>>` - The encoded binary or an error
///
/// # Errors
///
/// Returns an error if the module cannot be encoded.
pub fn encode_module(module: &Module) -> Result<Vec<u8>> {
    // Create a buffer to hold the encoded module
    let mut buffer = Vec::new();

    // Add the WebAssembly magic number and version
    buffer.extend_from_slice(&[0x00, 0x61, 0x73, 0x6D]); // \0asm
    buffer.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // version 1

    // Encode each section in the module
    // This is a simplified implementation - a real implementation would handle
    // all section types and properly encode each section's contents

    // For name sections, use the name section generator
    if let Some(custom_section) = module.find_custom_section("name") {
        buffer.extend_from_slice(&custom_section.data);
    }

    // Return the encoded binary
    Ok(buffer)
}
