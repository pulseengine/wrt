//! Convenient re-exports for common parsing operations
//!
//! This module provides a convenient prelude that imports the most commonly
//! used types and functions for WebAssembly parsing.

// Core parsing functionality
pub use crate::streaming_parser::{StreamingParser, ParseResult, ComponentStreamingParser};
pub use crate::module_builder::{Module, Function, Global, Export, Import, Element, Data};
pub use crate::component_parser::{Component, ComponentType, ComponentImport, ComponentExport};

// Type system
pub use crate::types::{ValueType, FuncType, GlobalType, MemoryType, TableType, BlockType, Limits};

// Validation
pub use crate::validation::{ValidationConfig, ValidationError, ModuleValidator, ComponentValidator};

// Binary format utilities
pub use crate::binary_constants::{
    WASM_MAGIC, WASM_VERSION, COMPONENT_MAGIC, COMPONENT_VERSION,
    TYPE_SECTION_ID, IMPORT_SECTION_ID, FUNCTION_SECTION_ID, TABLE_SECTION_ID,
    MEMORY_SECTION_ID, GLOBAL_SECTION_ID, EXPORT_SECTION_ID, START_SECTION_ID,
    ELEMENT_SECTION_ID, CODE_SECTION_ID, DATA_SECTION_ID, DATA_COUNT_SECTION_ID,
    CUSTOM_SECTION_ID,
};

// LEB128 utilities
pub use crate::leb128::{
    read_leb128_u32, read_leb128_i32, read_leb128_u64, read_leb128_i64,
    write_leb128_u32_to_slice,
};

// Error handling
pub use wrt_error::{Error, ErrorCategory, Result, codes};

// Memory provider
pub use crate::ParserProvider;

// Main parsing functions
pub use crate::{parse_wasm, parse_component, validate_header};