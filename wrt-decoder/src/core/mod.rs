//! WebAssembly Core Module Functionality
//!
//! This module provides functions for decoding, encoding, parsing, and validating
//! WebAssembly core modules according to the WebAssembly specification.

pub mod decode;
pub mod encode;
pub mod name_section;
pub mod parse;
pub mod sections;
pub mod validation;

// Re-exports
pub use decode::decode_module;
pub use encode::encode_module;
pub use name_section::{
    generate_name_section, parse_name_section, FunctionNameMap, LocalNameMap, NameMap, NameSection,
};
pub use parse::{parse_binary, parse_module};
pub use validation::{validate_module, validate_module_with_config, ValidationConfig};
