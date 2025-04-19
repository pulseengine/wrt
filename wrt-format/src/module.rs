//! WebAssembly module structure.
//!
//! This module provides types and utilities for working with WebAssembly modules.

use crate::section::CustomSection;
use crate::types::{FuncType, Limits, ValueType};
use crate::{String, Vec};
use wrt_error::kinds;
use wrt_error::{Error, Result};

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

/// WebAssembly global type definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalType {
    /// The value type of the global
    pub value_type: ValueType,
    /// Whether the global is mutable
    pub mutable: bool,
}

/// WebAssembly function definition
#[derive(Debug, Clone)]
pub struct Function {
    /// Type index
    pub type_idx: u32,
    /// Local variables
    pub locals: Vec<ValueType>,
    /// Function body (instructions)
    pub code: Vec<u8>,
}

/// WebAssembly memory definition
///
/// A memory instance as defined in the WebAssembly Core Specification.
/// The memory section consists of a vector of memory definitions, each
/// defining a memory with limits, and optional shared flag for threading.
///
/// WebAssembly 1.0 allows at most one memory per module.
/// Memory64 extension allows memories with 64-bit addressing.
#[derive(Debug, Clone)]
pub struct Memory {
    /// Memory limits (minimum and optional maximum size in pages)
    /// Each page is 64KiB (65536 bytes)
    pub limits: Limits,
    /// Whether this memory is shared between threads
    /// Shared memory must have a maximum size specified
    pub shared: bool,
}

/// WebAssembly table definition
#[derive(Debug, Clone)]
pub struct Table {
    /// Element type
    pub element_type: ValueType,
    /// Table limits
    pub limits: Limits,
}

/// WebAssembly global definition
#[derive(Debug, Clone)]
pub struct Global {
    /// Global type
    pub global_type: GlobalType,
    /// Initialization expression
    pub init: Vec<u8>,
}

/// WebAssembly data segment types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataMode {
    /// Active data segment (explicitly placed into a memory)
    Active,
    /// Passive data segment (used with memory.init)
    Passive,
}

/// WebAssembly data segment
#[derive(Debug, Clone)]
pub struct Data {
    /// Data mode (active or passive)
    pub mode: DataMode,
    /// Memory index (for active data segments)
    pub memory_idx: u32,
    /// Offset expression (for active data segments)
    pub offset: Vec<u8>,
    /// Initial data
    pub init: Vec<u8>,
}

/// WebAssembly element segment
#[derive(Debug, Clone)]
pub struct Element {
    /// Table index
    pub table_idx: u32,
    /// Offset expression
    pub offset: Vec<u8>,
    /// Function indices
    pub init: Vec<u32>,
}

/// WebAssembly export
#[derive(Debug, Clone)]
pub struct Export {
    /// Export name
    pub name: String,
    /// Export kind
    pub kind: ExportKind,
    /// Export index
    pub index: u32,
}

/// WebAssembly export kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    /// Function export
    Function,
    /// Table export
    Table,
    /// Memory export
    Memory,
    /// Global export
    Global,
}

/// WebAssembly import
#[derive(Debug, Clone)]
pub struct Import {
    /// Module name
    pub module: String,
    /// Import name
    pub name: String,
    /// Import description
    pub desc: ImportDesc,
}

/// WebAssembly import description
#[derive(Debug, Clone)]
pub enum ImportDesc {
    /// Function import
    Function(u32),
    /// Table import
    Table(Table),
    /// Memory import
    Memory(Memory),
    /// Global import
    Global(Global),
}

/// WebAssembly module
#[derive(Debug, Clone)]
pub struct Module {
    /// Function types
    pub types: Vec<FuncType>,
    /// Function definitions
    pub functions: Vec<Function>,
    /// Table definitions
    pub tables: Vec<Table>,
    /// Memory definitions
    pub memories: Vec<Memory>,
    /// Global definitions
    pub globals: Vec<Global>,
    /// Element segments
    pub elements: Vec<Element>,
    /// Data segments
    pub data: Vec<Data>,
    /// Exports
    pub exports: Vec<Export>,
    /// Imports
    pub imports: Vec<Import>,
    /// Start function
    pub start: Option<u32>,
    /// Custom sections
    pub custom_sections: Vec<CustomSection>,
    /// Original binary (if available)
    pub binary: Option<Vec<u8>>,
}

impl Default for Module {
    fn default() -> Self {
        Self::new()
    }
}

impl Module {
    /// Create a new empty module
    pub fn new() -> Self {
        Self {
            types: Vec::new(),
            functions: Vec::new(),
            tables: Vec::new(),
            memories: Vec::new(),
            globals: Vec::new(),
            elements: Vec::new(),
            data: Vec::new(),
            exports: Vec::new(),
            imports: Vec::new(),
            start: None,
            custom_sections: Vec::new(),
            binary: None,
        }
    }

    /// Convert a WebAssembly binary to a Module.
    ///
    /// This is a convenience method that wraps Binary::from_bytes + Module::from_binary
    pub fn from_bytes(_wasm_bytes: &[u8]) -> Result<Self> {
        // This is a minimal implementation - will be expanded later
        Err(Error::new(kinds::ParseError(
            "Module::from_bytes not yet implemented".to_string(),
        )))
    }

    /// Convert a Module to a WebAssembly binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        // This is a minimal implementation - will be expanded later
        Err(Error::new(kinds::ParseError(
            "Module::to_bytes not yet implemented".to_string(),
        )))
    }

    /// Find a custom section by name
    pub fn find_custom_section(&self, name: &str) -> Option<&CustomSection> {
        self.custom_sections
            .iter()
            .find(|section| section.name == name)
    }

    /// Add a custom section
    pub fn add_custom_section(&mut self, section: CustomSection) {
        self.custom_sections.push(section);
    }

    /// Check if this module contains state sections
    pub fn has_state_sections(&self) -> bool {
        crate::state::has_state_sections(&self.custom_sections)
    }
}

#[cfg(feature = "kani")]
mod verification {
    use super::*;
    use kani::*;

    #[kani::proof]
    fn verify_module_new_is_empty() {
        let module = Module::new();
        assert!(module.types.is_empty());
        assert!(module.functions.is_empty());
        assert!(module.tables.is_empty());
        assert!(module.memories.is_empty());
        assert!(module.globals.is_empty());
        assert!(module.elements.is_empty());
        assert!(module.data.is_empty());
        assert!(module.exports.is_empty());
        assert!(module.imports.is_empty());
        assert!(module.start.is_none());
        assert!(module.custom_sections.is_empty());
        assert!(module.binary.is_none());
    }

    #[kani::proof]
    fn verify_find_custom_section() {
        // Create a module with a custom section
        let mut module = Module::new();

        // Add a custom section
        let section_name = "test-section";
        let section_data = vec![1, 2, 3, 4];
        module.add_custom_section(CustomSection {
            name: section_name.to_string(),
            data: section_data.clone(),
        });

        // Find the section
        let found = module.find_custom_section(section_name);
        assert!(found.is_some());

        // Verify the section data
        let section = found.unwrap();
        assert_eq!(section.name, section_name);
        assert_eq!(section.data, section_data);

        // Try to find a non-existent section
        let not_found = module.find_custom_section("non-existent");
        assert!(not_found.is_none());
    }
}
