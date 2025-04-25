//! WebAssembly module decoder for wrt runtime
//!
//! This crate provides a high-level API for decoding WebAssembly binary modules
//! into structured representations that can be used by the wrt runtime.
//!
//! The decoder sits between the low-level binary format handling in `wrt-format`
//! and the runtime execution in `wrt`.
//!
//! # Features
//!
//! - Decoding WebAssembly modules from binary format
//! - Encoding modules back to binary format
//! - Validating module structure
//! - Memory-efficient handling of WASM modules
//! - Component model support

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(clippy::missing_panics_doc)]

// Import std when available
#[cfg(feature = "std")]
extern crate std;

// Import alloc for no_std
#[cfg(not(feature = "std"))]
extern crate alloc;

// Re-export core types from wrt-types
pub use wrt_types::{safe_memory::SafeSlice, FuncType, ValueType};

// Create a prelude module with common imports
#[cfg(not(feature = "std"))]
pub(crate) mod prelude {
    pub use alloc::collections::{BTreeMap as HashMap, BTreeSet as HashSet};
    pub use alloc::format;
    pub use alloc::string::{String, ToString};
    pub use alloc::vec::Vec;
}

#[cfg(feature = "std")]
pub(crate) mod prelude {
    pub use std::collections::HashSet;
    pub use std::format;
    pub use std::string::ToString;
    pub use std::vec::Vec;
}

// Export module components
pub mod component;
pub mod component_name_section;
pub mod component_val_type;
pub mod component_validation;
pub mod instructions;
pub mod module;
pub mod name_section;
pub mod producers_section;
pub mod sections;
pub mod types;
pub mod validation;

// Re-export main components for ease of use
pub use component::decode_component;
pub use component_validation::validate_component;
pub use instructions::Instruction;
pub use module::Module;
pub use name_section::NameSection;
pub use producers_section::{extract_producers_section, ProducersSection};
pub use sections::*;
// Re-export types module
pub use types::{parse_value_type, BlockType, Limits};

/// Version of the WebAssembly binary format supported by this decoder
pub const WASM_SUPPORTED_VERSION: u32 = 1;

// Magic bytes for WebAssembly modules: \0asm
pub const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];

/// Decode a WebAssembly binary module into a structured module representation
///
/// This is the main entry point for clients using this crate.
pub fn decode(bytes: &[u8]) -> wrt_error::Result<Module> {
    module::decode_module(bytes)
}

/// Check if the binary is a WebAssembly component or core module
///
/// Returns true if the bytes represent a WebAssembly component.
pub fn is_component(bytes: &[u8]) -> wrt_error::Result<bool> {
    component::utils::is_component(bytes)
}

/// Encode a WebAssembly module into binary format
///
/// Currently, this function is a placeholder and returns an empty byte vector.
pub fn encode(_module: &Module) -> wrt_error::Result<Vec<u8>> {
    // In a real implementation, this would encode the module to binary
    // For now, return an empty vector as a placeholder
    Ok(Vec::new())
}

/// Validate a WebAssembly module
///
/// This is the main entry point for validation.
pub fn validate(module: &Module) -> wrt_error::Result<()> {
    validation::validate_module(module)
}

/// Extract custom sections with the given name from a module.
///
/// This function returns a vector of references to the raw data of custom sections
/// with the specified name. This is useful for extracting specific metadata like
/// name sections.
pub fn extract_custom_sections<'a>(module: &'a Module, name: &str) -> Vec<&'a [u8]> {
    module
        .custom_sections
        .iter()
        .filter(|section| section.name == name)
        .map(|section| section.data.as_slice())
        .collect()
}

#[cfg(feature = "no_std")]
pub use alloc::{
    borrow::ToOwned,
    collections::BTreeMap as HashMap,
    string::{String, ToString},
    vec::Vec,
};

#[cfg(not(feature = "no_std"))]
pub use std::{
    borrow::ToOwned,
    collections::HashMap,
    string::{String, ToString},
    vec::Vec,
};
