//! Unified WebAssembly binary parser for wrt runtime
//!
//! This crate provides a unified streaming API for parsing WebAssembly binaries
//! into structured representations that can be used by the wrt runtime.
//!
//! The parser handles both Core WebAssembly and Component Model formats,
//! processing sections one at a time without loading the entire binary
//! into memory for optimal performance in memory-constrained environments.
//!
//! # Features
//!
//! - Streaming binary parsing with minimal memory usage
//! - Built-in Component Model support
//! - ASIL-D compliant memory management
//! - Both std and no_std environment support
//! - Type-safe conversion from binary format to runtime types

#![forbid(unsafe_code)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(clippy::missing_panics_doc)]
#![allow(missing_docs)] // Temporarily disabled for build

// Import core
extern crate core;

// Import std when available
#[cfg(feature = "std")]
extern crate std;

// Binary std/no_std choice
#[cfg(any(feature = "std", feature = "alloc"))]
extern crate alloc;

// Core modules
pub mod binary_constants;
pub mod leb128;
pub mod streaming_parser;
pub mod module_builder;
pub mod section_parser;
pub mod component_parser;
pub mod validation;
pub mod types;
pub mod prelude;

// Re-export core functionality
pub use streaming_parser::{StreamingParser, ParseResult};
pub use module_builder::{ModuleBuilder, Module};
pub use types::{ValueType, BlockType, FuncType, GlobalType, MemoryType, TableType};
pub use validation::{ValidationConfig, ValidationError};
pub use wrt_error::{Error, Result};

// Memory provider type alias for consistent usage
use wrt_foundation::safe_memory::NoStdProvider;
pub type ParserProvider = NoStdProvider<8192>;

/// Parse a WebAssembly binary into a runtime module
pub fn parse_wasm(binary: &[u8]) -> Result<Module<ParserProvider>> {
    let mut parser = StreamingParser::new()?;
    parser.parse(binary)
}

/// Parse a WebAssembly Component binary into a runtime component
pub fn parse_component(binary: &[u8]) -> Result<component_parser::Component<ParserProvider>> {
    let mut parser = streaming_parser::ComponentStreamingParser::new()?;
    parser.parse(binary)
}

/// Validate WebAssembly binary header
pub fn validate_header(bytes: &[u8]) -> Result<()> {
    if bytes.len() < 8 {
        return Err(Error::new(
            wrt_error::ErrorCategory::Parse,
            wrt_error::codes::PARSE_ERROR,
            "Binary too small for WebAssembly header"
        ));
    }
    
    // Check magic number
    if &bytes[0..4] != &binary_constants::WASM_MAGIC {
        return Err(Error::new(
            wrt_error::ErrorCategory::Parse,
            wrt_error::codes::PARSE_ERROR,
            "Invalid WebAssembly magic number"
        ));
    }
    
    // Check version
    if &bytes[4..8] != &binary_constants::WASM_VERSION {
        return Err(Error::new(
            wrt_error::ErrorCategory::Parse,
            wrt_error::codes::PARSE_ERROR,
            "Unsupported WebAssembly version"
        ));
    }
    
    Ok(())
}