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
//! - Component model support (planned)

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// Import std when available
#[cfg(feature = "std")]
extern crate std;

// Import alloc for no_std
#[cfg(not(feature = "std"))]
extern crate alloc;

// Import std/alloc modules based on feature flag
#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

// Export module components
pub mod instructions;
pub mod module;
pub mod name_section;
pub mod sections;
pub mod types;
pub mod validation;

// Re-export main components for ease of use
pub use instructions::Instruction;
pub use module::Module;
pub use name_section::NameSection;
pub use sections::*;

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

/// Encode a structured module representation into a WebAssembly binary
///
/// This is used to serialize a module back to its binary representation.
pub fn encode(module: &Module) -> wrt_error::Result<Vec<u8>> {
    module::encode_module(module)
}

/// Validate a WebAssembly binary module
///
/// This checks that the module follows the WebAssembly specification.
pub fn validate(bytes: &[u8]) -> wrt_error::Result<()> {
    let module = decode(bytes)?;
    validation::validate_module(&module)
}

/// Extract custom sections from a WebAssembly module
///
/// This is useful for retrieving name sections or other metadata without
/// fully parsing the module.
pub fn extract_custom_sections(bytes: &[u8], name: &str) -> wrt_error::Result<Vec<Vec<u8>>> {
    let module = decode(bytes)?;
    let sections = module
        .custom_sections
        .iter()
        .filter(|s| s.name == name)
        .map(|s| s.data.clone())
        .collect();
    Ok(sections)
}

/// Convert WebAssembly text format (WAT) to binary format
///
/// This is provided as a convenience function for development and testing.
#[cfg(all(feature = "std", feature = "wat"))]
pub fn wat_to_wasm(wat: &str) -> wrt_error::Result<Vec<u8>> {
    // This requires the 'wat' crate which is optional
    match ::wat::parse_str(wat) {
        Ok(bytes) => Ok(bytes),
        Err(e) => Err(wrt_error::Error::new(wrt_error::kinds::ParseError(
            format!("Failed to parse WAT: {}", e),
        ))),
    }
}

/// Create a placeholder module for testing or other purposes
pub fn create_empty_module() -> Module {
    Module::new()
}
