//! WebAssembly Core Module Decoder
//!
//! Functions for decoding WebAssembly core modules from binary format.

use crate::Result;
use wrt_format::Module;

/// Decode a WebAssembly binary module
///
/// This function takes a WebAssembly binary and decodes it into a structured
/// Module representation. It performs validation by default.
///
/// # Arguments
///
/// * `data` - The WebAssembly binary data
///
/// # Returns
///
/// * `Result<Module>` - The decoded module or an error
///
/// # Errors
///
/// Returns an error if the binary is invalid or cannot be decoded.
pub fn decode_module(data: &[u8]) -> Result<Module> {
    // Parse the module first
    let module = crate::core::parse::parse_module(data)?;

    // Validate the module
    crate::core::validation::validate_module(&module)?;

    // Return the validated module
    Ok(module)
}

/// Decode a WebAssembly binary module without validation
///
/// This function takes a WebAssembly binary and decodes it into a structured
/// Module representation without performing validation.
///
/// # Arguments
///
/// * `data` - The WebAssembly binary data
///
/// # Returns
///
/// * `Result<Module>` - The decoded module or an error
///
/// # Errors
///
/// Returns an error if the binary cannot be parsed.
pub fn decode_module_unchecked(data: &[u8]) -> Result<Module> {
    crate::core::parse::parse_module(data)
}
