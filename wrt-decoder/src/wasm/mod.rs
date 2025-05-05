//! WebAssembly core module parsing and validation
//!
//! This module provides a high-level API for parsing and validating
//! WebAssembly core modules.

// No direct imports needed

// Re-export core module utilities
pub use crate::name_section::*;

// Re-export module-related functions
pub use crate::decoder_core::validate::validate_module;
pub use crate::module::decode_module;
pub use crate::module::encode_module;

// Re-export with more convenient names
pub use crate::module::decode_module_with_binary;
pub use crate::module::encode_module as encode;

// Additional alias for backwards compatibility
pub use crate::module::decode_module_with_binary as decode;
