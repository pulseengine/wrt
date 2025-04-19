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
pub mod component;
pub mod component_validation;
pub mod instructions;
pub mod module;
pub mod name_section;
pub mod sections;
pub mod types;
pub mod validation;

// Re-export main components for ease of use
pub use component::decode_component;
pub use component_validation::validate_component;
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

/// Check if the binary is a WebAssembly component or core module
///
/// Returns true if the binary appears to be a component, false if it's a core module,
/// or an error if the format is invalid.
pub fn is_component(bytes: &[u8]) -> wrt_error::Result<bool> {
    use wrt_error::{kinds, Error};

    // Check if we have enough bytes for a header
    if bytes.len() < 8 {
        return Err(Error::new(kinds::ParseError(
            "Binary too short for WebAssembly header".to_string(),
        )));
    }

    // Check magic bytes (\0asm) - both component and core module have the same magic
    if bytes[0..4] != WASM_MAGIC {
        return Err(Error::new(kinds::ParseError(
            "Invalid WebAssembly magic bytes".to_string(),
        )));
    }

    // Check the layer identifier in bytes 6-7
    // For component it's [0x01, 0x00], for core module it's part of the version [0x00, 0x00]
    Ok(bytes[6..8] == [0x01, 0x00])
}

/// Encode a WebAssembly module into a binary format
pub fn encode(module: &Module) -> wrt_error::Result<Vec<u8>> {
    // In a real implementation, this would encode the module to binary
    // For now, if the module has the original binary, return it
    if let Some(binary) = &module.binary {
        return Ok(binary.clone());
    }

    // Otherwise, return an error
    Err(wrt_error::Error::new(wrt_error::kinds::RuntimeError(
        "Module encoding not yet implemented".to_string(),
    )))
}

/// Validate a WebAssembly binary module
///
/// This checks that the module follows the WebAssembly specification.
pub fn validate(bytes: &[u8]) -> wrt_error::Result<()> {
    let module = decode(bytes)?;
    validation::validate_module(&module)
}

/// Validate a WebAssembly binary component or module
///
/// This checks that the binary follows either the WebAssembly Core Specification
/// or the WebAssembly Component Model Specification, depending on the binary type.
pub fn validate_binary(bytes: &[u8]) -> wrt_error::Result<()> {
    // First determine if this is a component or a core module
    let is_comp = is_component(bytes)?;

    if is_comp {
        // Decode and validate as a component
        let component = decode_component(bytes)?;
        component_validation::validate_component(&component)
    } else {
        // Decode and validate as a core module
        validate(bytes)
    }
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
