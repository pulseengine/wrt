//! Decoder integration for wrt
//!
//! This module provides the integration layer between wrt-decoder and wrt,
//! handling the safe and efficient transfer of decoded WebAssembly modules
//! with functional safety guarantees.

use wrt_decoder;
use wrt_decoder::from_binary as decode_binary;
use wrt_error::{kinds, Error, Result};

// Use decoder types from prelude
// Import directly from wrt-format and wrt-decoder
use wrt_decoder::module::CodeSection;
use wrt_format::module::{Data, Element, Export, Global, Import, ImportDesc};
use wrt_format::section::{CustomSection, Section};
use wrt_format::types::Limits;

// Import from wrt-types
use wrt_types::{
    safe_memory::{MemoryProvider, SafeSlice},
    sections::SectionId,
};

// Import from prelude for consistent type definitions
use crate::prelude::{
    BlockType, ExternType, FuncType, GlobalType, MemoryType, RefType, TableType, ValueType,
};

// Import from our crate modules
use crate::module::{
    CustomSection as WrtCustomSection, Function, Import as WrtImport, OtherExport,
};

#[cfg(feature = "std")]
use std::sync::Arc;

#[cfg(not(feature = "std"))]
use alloc::sync::Arc;

#[cfg(feature = "std")]
use wrt_types::safe_memory::StdMemoryProvider;

#[cfg(not(feature = "std"))]
use wrt_types::safe_memory::NoStdMemoryProvider;

// Imports for the wrt module types
use crate::instructions::Instruction;
use crate::module::{Data as WrtData, Element as WrtElement, Module};

/// A safe WebAssembly module representation for runtime
///
/// This struct provides a bridge between the decoder and runtime,
/// with integrated safety features for ASIL-B compliance.
pub struct SafeModule {
    /// Module data provider
    memory_provider: Arc<dyn MemoryProvider>,
    /// Custom sections
    pub custom_sections: Vec<CustomSection>,
    /// Module name from custom section (if any)
    pub name: Option<String>,
    /// Start function index (if any)
    pub start: Option<u32>,
    /// Function types
    pub types: Vec<FuncType>,
    /// Imports
    pub imports: Vec<Import>,
    /// Functions (type indices)
    pub functions: Vec<u32>,
    /// Tables
    pub tables: Vec<TableType>,
    /// Memories
    pub memories: Vec<MemoryType>,
    /// Globals
    pub globals: Vec<Global>,
    /// Exports
    pub exports: Vec<Export>,
    /// Elements
    pub elements: Vec<Element>,
    /// Function bodies
    pub bodies: Vec<CodeSection>,
    /// Data segments
    pub data: Vec<Data>,
    /// Data count (if present)
    pub data_count: Option<u32>,
}

impl SafeModule {
    /// Create a new empty SafeModule
    #[cfg(feature = "std")]
    pub fn new() -> Self {
        Self {
            memory_provider: Arc::new(StdMemoryProvider::new(Vec::new())),
            custom_sections: Vec::new(),
            name: None,
            start: None,
            types: Vec::new(),
            imports: Vec::new(),
            functions: Vec::new(),
            tables: Vec::new(),
            memories: Vec::new(),
            globals: Vec::new(),
            exports: Vec::new(),
            elements: Vec::new(),
            bodies: Vec::new(),
            data: Vec::new(),
            data_count: None,
        }
    }

    /// Create a new SafeModule from binary data
    #[cfg(feature = "std")]
    pub fn from_binary(binary: &[u8]) -> Result<Self> {
        // Create memory provider with the binary data
        let memory_provider = Arc::new(StdMemoryProvider::new(binary.to_vec()));

        // Decode the module using wrt-decoder
        let decoder_module = decode_binary(binary)?;

        // Create the safe module
        let mut module = Self {
            memory_provider,
            custom_sections: decoder_module.custom_sections,
            name: decoder_module.name,
            start: decoder_module.start,
            types: Vec::new(),
            imports: Vec::new(),
            functions: Vec::new(),
            tables: Vec::new(),
            memories: Vec::new(),
            globals: Vec::new(),
            exports: Vec::new(),
            elements: Vec::new(),
            bodies: Vec::new(),
            data: Vec::new(),
            data_count: decoder_module.data_count,
        };

        // Convert types - these have built-in safety validation
        module.types = decoder_module.types;

        // Convert imports
        module.imports = decoder_module.imports;

        // Convert functions (type indices)
        module.functions = decoder_module.functions;

        // Convert tables
        module.tables = decoder_module.tables;

        // Convert memories
        module.memories = decoder_module.memories;

        // Convert globals
        module.globals = decoder_module.globals;

        // Convert exports
        module.exports = decoder_module.exports;

        // Convert elements
        module.elements = decoder_module.elements;

        // Convert function bodies
        module.bodies = decoder_module.code;

        // Convert data segments
        module.data = decoder_module.data;

        Ok(module)
    }

    /// Create a new SafeModule from binary data in no_std environment
    #[cfg(not(feature = "std"))]
    pub fn from_binary<const N: usize>(binary: &[u8]) -> Result<Self> {
        // Check if binary is too large for our fixed buffer
        if binary.len() > N {
            return Err(Error::new(kinds::OutOfBoundsError(format!(
                "Binary too large: {} bytes (max: {} bytes)",
                binary.len(),
                N
            ))));
        }

        // Create memory provider with fixed buffer
        let mut provider = NoStdMemoryProvider::<N>::new();
        provider.set_data(binary)?;
        let memory_provider = Arc::new(provider);

        // Decode the module using wrt-decoder
        let decoder_module = decode_binary(binary)?;

        // Create the safe module (similar to std version)
        let mut module = Self {
            memory_provider,
            custom_sections: decoder_module.custom_sections,
            name: decoder_module.name,
            start: decoder_module.start,
            types: Vec::new(),
            imports: Vec::new(),
            functions: Vec::new(),
            tables: Vec::new(),
            memories: Vec::new(),
            globals: Vec::new(),
            exports: Vec::new(),
            elements: Vec::new(),
            bodies: Vec::new(),
            data: Vec::new(),
            data_count: decoder_module.data_count,
        };

        // Convert the same way as in std version
        module.types = decoder_module.types;
        module.imports = decoder_module.imports;
        module.functions = decoder_module.functions;
        module.tables = decoder_module.tables;
        module.memories = decoder_module.memories;
        module.globals = decoder_module.globals;
        module.exports = decoder_module.exports;
        module.elements = decoder_module.elements;
        module.bodies = decoder_module.code;
        module.data = decoder_module.data;

        Ok(module)
    }

    /// Verify the module's integrity
    pub fn verify(&self) -> Result<()> {
        // Verify all types
        for ty in &self.types {
            ty.verify()?;
        }

        // Verify custom sections
        for section in &self.custom_sections {
            section.verify()?;
        }

        // Verify function bodies
        for body in &self.bodies {
            body.verify()?;
        }

        // Verify function indices
        let func_count = self.functions.len();
        if self.bodies.len() != func_count {
            return Err(Error::new(kinds::ValidationError(format!(
                "Function count mismatch: {} types, {} bodies",
                func_count,
                self.bodies.len()
            ))));
        }

        // More validation as needed for ASIL-B requirements

        Ok(())
    }

    /// Get the total number of functions (imported + defined)
    pub fn num_functions(&self) -> usize {
        let imported = self
            .imports
            .iter()
            .filter(|import| matches!(import.desc, ImportDesc::Func(_)))
            .count();

        imported + self.functions.len()
    }

    /// Get the number of imported functions
    pub fn num_imported_functions(&self) -> usize {
        self.imports
            .iter()
            .filter(|import| matches!(import.desc, ImportDesc::Func(_)))
            .count()
    }

    /// Get the total number of tables (imported + defined)
    pub fn num_tables(&self) -> usize {
        let imported = self
            .imports
            .iter()
            .filter(|import| matches!(import.desc, ImportDesc::Table(_)))
            .count();

        imported + self.tables.len()
    }

    /// Get the total number of memories (imported + defined)
    pub fn num_memories(&self) -> usize {
        let imported = self
            .imports
            .iter()
            .filter(|import| matches!(import.desc, ImportDesc::Memory(_)))
            .count();

        imported + self.memories.len()
    }

    /// Get the total number of globals (imported + defined)
    pub fn num_globals(&self) -> usize {
        let imported = self
            .imports
            .iter()
            .filter(|import| matches!(import.desc, ImportDesc::Global(_)))
            .count();

        imported + self.globals.len()
    }
}

/// Convert from wrt-types CustomSection to wrt CustomSection
fn convert_custom_section(section: &CustomSection) -> crate::module::CustomSection {
    crate::module::CustomSection {
        name: section.name.clone(),
        data: section.data.clone(),
    }
}

/// Convert from wrt-format ImportDesc to wrt ExternType
fn convert_import_desc(desc: &ImportDesc) -> ExternType {
    match desc {
        ImportDesc::Function(type_idx) => ExternType::Func(*type_idx),
        ImportDesc::Table(table_type) => ExternType::Table(convert_table_type(table_type)),
        ImportDesc::Memory(memory_type) => ExternType::Memory(convert_memory_type(memory_type)),
        ImportDesc::Global(global_type) => ExternType::Global(convert_global_type(global_type)),
    }
}

/// Convert from wrt-format Table to wrt TableType
fn convert_table_type(table: &wrt_format::module::Table) -> TableType {
    TableType::new(
        // Use FuncRef for now; ideally would convert more precisely
        RefType::FuncRef,
        table.limits.min as u32,
        table.limits.max.map(|m| m as u32),
    )
}

/// Convert from wrt-format Memory to wrt MemoryType
fn convert_memory_type(memory: &wrt_format::module::Memory) -> MemoryType {
    MemoryType::new(
        memory.limits.min as u32,
        memory.limits.max.map(|m| m as u32),
    )
}

/// Convert from wrt-format Global to wrt GlobalType
fn convert_global_type(global: &wrt_format::module::Global) -> GlobalType {
    GlobalType::new(
        // Simplify for now, assuming i32
        ValueType::I32,
        global.global_type.mutable,
    )
}

/// Convert from wrt-types Import to wrt Import
fn convert_import(import: &Import) -> WrtImport {
    WrtImport {
        module: import.module.clone(),
        name: import.name.clone(),
        desc: convert_import_desc(&import.desc),
    }
}

/// Map from wrt-format ExportKind to wrt ExportKind
fn map_export_kind(kind: &wrt_format::module::ExportKind) -> crate::module::ExportKind {
    match kind {
        wrt_format::module::ExportKind::Function => crate::module::ExportKind::Function,
        wrt_format::module::ExportKind::Table => crate::module::ExportKind::Table,
        wrt_format::module::ExportKind::Memory => crate::module::ExportKind::Memory,
        wrt_format::module::ExportKind::Global => crate::module::ExportKind::Global,
    }
}

/// Convert from wrt-format Export to wrt OtherExport
fn convert_export(export: &Export) -> crate::module::OtherExport {
    crate::module::OtherExport {
        name: export.name.clone(),
        kind: map_export_kind(&export.kind),
        index: export.index,
    }
}

/// Convert from wrt-decoder CodeSection to wrt Function
fn convert_function_body(body: &CodeSection, type_idx: u32) -> Result<Function> {
    // Parse the code section
    let code = parse_function_body(&body.body)?;

    // Since the CodeSection doesn't have locals field, we'll create an empty Vec for now
    // In a real implementation, locals would be parsed from the function body
    Ok(Function {
        type_idx,
        locals: Vec::new(),
        code,
    })
}

/// Parse function body to extract instructions
fn parse_function_body(_code: &[u8]) -> Result<Vec<Instruction>> {
    unimplemented!()
}

/// Convert from wrt-types Element to wrt Element
fn convert_element(element: &Element) -> crate::module::Element {
    crate::module::Element {
        table_idx: element.table,
        offset: parse_expr(&element.offset).unwrap_or_default(),
        items: element.functions.clone(),
    }
}

/// Convert from wrt-types Data to wrt Data
fn convert_data(data: &Data) -> crate::module::Data {
    crate::module::Data {
        memory_idx: data.memory,
        offset: parse_expr(&data.offset).unwrap_or_default(),
        init: data.init.clone(),
    }
}

/// Parse expression from binary code
fn parse_expr(_code: &[u8]) -> Result<Vec<Instruction>> {
    unimplemented!()
}

/// Convert a SafeModule to a runtime Module
pub fn convert_to_runtime_module(safe_module: &SafeModule) -> Result<Module> {
    let mut module = Module::new()?;

    // Set basic properties
    module.name = safe_module.name.clone();
    module.start = safe_module.start;

    // Convert custom sections
    module.custom_sections = safe_module
        .custom_sections
        .iter()
        .map(convert_custom_section)
        .collect();

    // Since FuncType is now consolidated, we can directly use it without conversion
    module.types = safe_module.types.clone();

    // Convert imports
    let imports: Vec<WrtImport> = safe_module.imports.iter().map(convert_import).collect();
    for import in imports {
        module.imports.push(import);
    }

    // Convert functions
    let functions: Result<Vec<Function>> = safe_module
        .functions
        .iter()
        .enumerate()
        .map(|(idx, &type_idx)| {
            let body = &safe_module.bodies[idx];
            convert_function_body(body, type_idx)
        })
        .collect();
    module.functions = functions?;

    // Convert exports
    module.exports = safe_module.exports.iter().map(convert_export).collect();

    // Convert elements
    module.elements = safe_module.elements.iter().map(convert_element).collect();

    // Convert data segments
    module.data = safe_module.data.iter().map(convert_data).collect();

    Ok(module)
}

/// Load a WebAssembly module using the safe decoding path
pub fn load_module_from_binary(binary: &[u8]) -> Result<Module> {
    // Create a SafeModule first
    let safe_module = SafeModule::from_binary(binary)?;

    // Verify the module's integrity
    safe_module.verify()?;

    // Convert to runtime module
    convert_to_runtime_module(&safe_module)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "std")]
    fn test_empty_module() {
        let module = SafeModule::new();
        assert_eq!(module.types.len(), 0);
        assert_eq!(module.functions.len(), 0);
        assert_eq!(module.num_functions(), 0);
    }

    #[test]
    #[cfg(all(feature = "std", feature = "wat-parsing"))]
    fn test_simple_module() {
        use wrt_decoder::wat_to_wasm;

        // Simple module with a function
        let wat = r#"(module
            (func (export "answer") (result i32)
                i32.const 42
            )
        )"#;

        let wasm = wat_to_wasm(wat).unwrap();
        let module = SafeModule::from_binary(&wasm).unwrap();

        // Verify should pass
        assert!(module.verify().is_ok());

        // Check exports
        assert_eq!(module.exports.len(), 1);
        assert_eq!(module.exports[0].name, "answer");

        // Convert to runtime module
        let runtime = convert_to_runtime_module(&module).unwrap();

        // Check runtime module
        assert_eq!(runtime.exports.len(), 1);
        assert_eq!(runtime.exports[0].name, "answer");
    }
}
