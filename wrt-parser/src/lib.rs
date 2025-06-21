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
pub mod bounded_types;
pub mod simple_module;
pub mod simple_parser;
pub mod types;
pub mod expression_parser;
pub mod validation;
pub mod instruction_parser;
pub mod advanced_validation;
pub mod memory_optimization;

// Component Model modules
pub mod component_types;
pub mod component_registry;
pub mod enhanced_module;
pub mod component_section_parser;

// Unified parsing modules
pub mod format_detection;
pub mod unified_parser;

// WIT parsing modules
pub mod wit_parser;
pub mod wit_lexer;
pub mod wit_parser_impl;

// Re-export core functionality
pub use simple_module::{SimpleModule, Export, Import, FunctionBody};
pub use simple_parser::SimpleParser;
pub use types::{ValueType, FuncType, GlobalType, MemoryType, TableType};
pub use validation::{ModuleValidator, ValidationConfig, ValidationError, validate_name};
pub use expression_parser::{ExpressionParser, ConstExpr, ConstValue};
pub use instruction_parser::{InstructionParser, Instruction, Opcode, MemArg, BrTable, BlockType, ControlFrame};
pub use advanced_validation::{AdvancedValidator, PlatformLimits, ValidationIssue, ValidationSeverity, ValidationLocation};
pub use memory_optimization::{MemoryPool, MemoryLayoutCalculator, MemoryLayout, MemorySegment, MemoryOptimizedParser, MemoryStats};
pub use wrt_error::{Error, Result};

// Re-export Component Model functionality
pub use component_types::{
    ComponentType, ComponentValueType, ComponentTypeDefinition, TypeRef, 
    StreamingTypeIntern, ComponentMemoryBudget
};
pub use component_registry::{ComponentRegistry, ComponentParserState};
pub use enhanced_module::{
    EnhancedModule, ComponentModel, ParserMode, 
    ComponentImport, ComponentExport, ComponentFunction, ComponentValue,
    ComponentAlias, InstantiationArg, ItemKind, StringEncoding, CanonicalOptions
};
pub use component_section_parser::ComponentSectionParser;

// Re-export unified parsing functionality
pub use format_detection::{FormatDetector, BinaryFormat, BinaryInfo, detect_format, quick_detect_format};
pub use unified_parser::{
    UnifiedParser, UnifiedParserConfig, UnifiedParseResult,
    parse_wasm_binary, parse_wasm_with_config, parse_core_module, parse_component
};

// Re-export WIT parsing functionality
pub use wit_parser::{
    WitDocument, WitPackage, WitInterface, WitWorld, WitFunction, WitType, WitTypeDef,
    WitTypeDefKind, WitRecord, WitVariant, WitEnum, WitFlags, WitResource,
    WitImport, WitExport, WitItem, WitParam, WitResult, WitUse, WitUseItem,
    WitRecordField, WitVariantCase,
    MAX_WIT_TYPES, MAX_WIT_FUNCTIONS, MAX_WIT_PARAMS, MAX_WIT_RESULTS,
    MAX_WIT_IMPORTS, MAX_WIT_EXPORTS, MAX_WIT_IDENTIFIER_LEN, MAX_WIT_STRING_LEN
};
pub use wit_lexer::{WitLexer, Token, Position, Span};
pub use wit_parser_impl::WitParser;

// Convenience functions are defined below

// Memory provider type alias for consistent usage
use wrt_foundation::safe_memory::NoStdProvider;
pub type ParserProvider = NoStdProvider<8192>;

/// Parse a WebAssembly binary into a runtime module
/// 
/// This function provides backward compatibility by using the unified parser
/// and extracting the core module from the result.
pub fn parse_wasm(binary: &[u8]) -> Result<SimpleModule> {
    let result = parse_wasm_binary(binary)?;
    Ok(result.into_core_module())
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

/// Parse a WIT (WebAssembly Interface Types) source file into a structured document
///
/// This function provides high-level parsing of WIT source code with memory-bounded
/// operation suitable for resource-constrained environments.
///
/// # Example
/// ```rust,no_run
/// use wrt_parser::parse_wit;
/// 
/// let wit_source = r#"
///     interface hello {
///         func greet(name: string) -> string
///     }
/// "#;
/// 
/// let document = parse_wit(wit_source)?;
/// assert_eq!(document.interfaces.len(), 1);
/// # Ok::<(), wrt_error::Error>(())
/// ```
pub fn parse_wit(source: &str) -> Result<WitDocument> {
    let mut parser = wit_parser_impl::WitParser::new(source)?;
    parser.parse_document()
}