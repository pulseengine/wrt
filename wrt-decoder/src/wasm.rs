//! WebAssembly Core Module Functionality
//!
//! This module provides high-level WebAssembly-specific decoder functionality
//! for working with core WebAssembly modules.
//!
//! It serves as the main entry point for operations related to WebAssembly modules,
//! including decoding, encoding, and validation.

// Re-export core module functionality
pub use crate::core::*;

// Re-export the name section handling for backward compatibility
pub use crate::core::name_section::*;

// Re-export specific types from the decoder's core module
pub use crate::core::decode::decode_module;
pub use crate::core::encode::encode_module;
pub use crate::core::validation::validate_module;
