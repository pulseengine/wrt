//! WebAssembly core module parsing and validation
//!
//! This module provides a high-level API for parsing and validating
//! WebAssembly core modules.

// No direct imports needed

// Re-export core module utilities
// Re-export module-related functions
// Re-export with more convenient names
// Additional alias for backwards compatibility
pub use crate::{
    decoder_core::validate::validate_module,
    module::{
        decode_module, decode_module_with_binary as decode, decode_module_with_binary,
        encode_module, encode_module as encode,
    },
    name_section::*,
};
