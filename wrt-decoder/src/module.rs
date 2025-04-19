//! WebAssembly module representation
//!
//! This module provides the main Module structure and functions for decoding
//! WebAssembly binary modules into structured representations.

use crate::{sections::*, String, Vec, WASM_MAGIC};
use wrt_error::{kinds, Error, Result};
use wrt_format::binary;

/// WebAssembly binary format version
pub const WASM_VERSION: [u8; 4] = [0x01, 0x00, 0x00, 0x00];

/// Represents a parsed WebAssembly module
#[derive(Debug, Clone, Default)]
pub struct Module {
    /// Module types (function signatures)
    pub types: Vec<FuncType>,
    /// Imported functions, tables, memories, and globals
    pub imports: Vec<Import>,
    /// Function definitions (type indices)
    pub functions: Vec<Function>,
    /// Table definitions
    pub tables: Vec<Table>,
    /// Memory definitions
    pub memories: Vec<Memory>,
    /// Global variable definitions
    pub globals: Vec<Global>,
    /// Element segments for tables
    pub elements: Vec<Element>,
    /// Data segments for memories
    pub data: Vec<Data>,
    /// Start function index
    pub start: Option<u32>,
    /// Custom sections
    pub custom_sections: Vec<CustomSection>,
    /// Exports (functions, tables, memories, and globals)
    pub exports: Vec<Export>,
    /// Optional name for the module, often derived from the custom "name" section.
    pub name: Option<String>,
    /// Original binary (if available)
    pub binary: Option<Vec<u8>>,
    /// Code section (function bodies)
    pub code: Vec<Code>,
}

impl Module {
    /// Creates a new empty module
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a module from binary data
    pub fn from_binary(bytes: &[u8]) -> Result<Self> {
        decode_module(bytes)
    }

    /// Converts the module to binary format
    pub fn to_binary(&self) -> Result<Vec<u8>> {
        encode_module(self)
    }

    /// Validates the module structure
    pub fn validate(&self) -> Result<()> {
        crate::validation::validate_module(self)
    }

    /// Returns true if the module has a custom section with the given name
    pub fn has_custom_section(&self, name: &str) -> bool {
        self.custom_sections
            .iter()
            .any(|section| section.name == name)
    }

    /// Returns a reference to the custom section with the given name, if any
    pub fn get_custom_section(&self, name: &str) -> Option<&CustomSection> {
        self.custom_sections
            .iter()
            .find(|section| section.name == name)
    }

    /// Adds a custom section to the module
    pub fn add_custom_section(&mut self, name: String, data: Vec<u8>) {
        self.custom_sections.push(CustomSection { name, data });
    }

    /// Sets the module name
    pub fn set_name(&mut self, name: String) {
        self.name = Some(name.clone());

        // Also update the name section if it exists
        // Note: In a complete implementation, you would encode the name properly here
        self.add_custom_section("name".to_string(), name.into_bytes());
    }

    /// Finds the index of an export by name
    pub fn find_export(&self, name: &str) -> Option<usize> {
        self.exports.iter().position(|export| export.name == name)
    }

    /// Extracts function names from the name section if available
    pub fn extract_function_names(&self) -> Result<Vec<(u32, String)>> {
        let name_section = self.get_custom_section("name");

        // If no name section, return empty vector
        if name_section.is_none() {
            return Ok(Vec::new());
        }

        // In a complete implementation, you would parse the name section here
        // For now, return an empty vector
        Ok(Vec::new())
    }

    /// Whether this module uses memory
    pub fn uses_memory(&self) -> bool {
        !self.memories.is_empty()
            || self
                .imports
                .iter()
                .any(|i| matches!(i.desc, ImportDesc::Memory(_)))
    }

    /// Whether this module uses tables
    pub fn uses_tables(&self) -> bool {
        !self.tables.is_empty()
            || self
                .imports
                .iter()
                .any(|i| matches!(i.desc, ImportDesc::Table(_)))
    }

    /// Counts the total number of functions (imported + defined)
    pub fn count_functions(&self) -> usize {
        let imported_funcs = self
            .imports
            .iter()
            .filter(|i| matches!(i.desc, ImportDesc::Function(_)))
            .count();

        imported_funcs + self.functions.len()
    }

    /// Creates a zero-copy view of memory for a data section
    /// This allows the runtime to directly use the binary data without copying
    pub fn get_data_view(&self, data_idx: usize) -> Option<&[u8]> {
        if data_idx >= self.data.len() {
            return None;
        }

        Some(&self.data[data_idx].init)
    }

    /// Creates a zero-copy view of the module binary
    /// This allows the runtime to directly use the binary without copying
    pub fn get_binary_view(&self) -> Option<&[u8]> {
        self.binary.as_deref()
    }
}

/// Decodes a WebAssembly binary module
pub fn decode_module(bytes: &[u8]) -> Result<Module> {
    // Verify magic bytes and version
    if bytes.len() < 8 {
        return Err(Error::new(kinds::ParseError(
            "WebAssembly binary too short".to_string(),
        )));
    }

    if bytes[0..4] != WASM_MAGIC {
        return Err(Error::new(kinds::ParseError(
            "Invalid WebAssembly magic bytes".to_string(),
        )));
    }

    if bytes[4..8] != WASM_VERSION {
        return Err(Error::new(kinds::ParseError(
            "Unsupported WebAssembly version".to_string(),
        )));
    }

    let mut module = Module::new();
    module.binary = Some(bytes.to_vec());

    // Parse sections
    let mut offset = 8;
    while offset < bytes.len() {
        let section_id = bytes[offset];
        offset += 1;

        let (size, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        let section_start = offset;
        let section_end = section_start + size as usize;

        if section_end > bytes.len() {
            return Err(Error::new(kinds::ParseError(format!(
                "Section size {} for section ID {} exceeds binary size",
                size, section_id
            ))));
        }

        let section_bytes = &bytes[section_start..section_end];

        match section_id {
            0x00 => {
                // Custom section
                let (name, bytes_read) = binary::read_string(section_bytes, 0)?;
                let data = section_bytes[bytes_read..].to_vec();
                module.custom_sections.push(CustomSection { name, data });

                // Extract module name if present
                if module.custom_sections.last().unwrap().name == "name" {
                    // In a complete implementation, you would parse the name section here
                }
            }
            0x01 => {
                // Type section
                let (types, _) = parse_type_section(section_bytes)?;
                module.types = types;
            }
            0x02 => {
                // Import section
                let (imports, _) = parse_import_section(section_bytes)?;
                module.imports = imports;
            }
            0x03 => {
                // Function section
                let (functions, _) = parse_function_section(section_bytes)?;
                module.functions = functions;
            }
            0x04 => {
                // Table section
                let (tables, _) = parse_table_section(section_bytes)?;
                module.tables = tables;
            }
            0x05 => {
                // Memory section
                let (memories, _) = parse_memory_section(section_bytes)?;
                module.memories = memories;
            }
            0x06 => {
                // Global section
                let (globals, _) = parse_global_section(section_bytes)?;
                module.globals = globals;
            }
            0x07 => {
                // Export section
                let (exports, _) = parse_export_section(section_bytes)?;
                module.exports = exports;
            }
            0x08 => {
                // Start section
                let (start, _) = binary::read_leb128_u32(section_bytes, 0)?;
                module.start = Some(start);
            }
            0x09 => {
                // Element section
                let (elements, _) = parse_element_section(section_bytes)?;
                module.elements = elements;
            }
            0x0A => {
                // Code section
                let (code, _) = parse_code_section(section_bytes)?;
                module.code = code;
            }
            0x0B => {
                // Data section
                let (data, _) = parse_data_section(section_bytes)?;
                module.data = data;
            }
            _ => {
                // Unknown section, ignore but warn
                log::warn!("Unknown section ID: {}", section_id);
            }
        }

        offset = section_end;
    }

    // Validate the basic structure now that we've parsed everything
    if module.functions.len() != module.code.len() && !module.functions.is_empty() {
        return Err(Error::new(kinds::ParseError(format!(
            "Function and code section counts mismatch: {} functions but {} code entries",
            module.functions.len(),
            module.code.len()
        ))));
    }

    Ok(module)
}

/// Encodes a WebAssembly binary module
pub fn encode_module(module: &Module) -> Result<Vec<u8>> {
    let mut result = Vec::new();

    // Write magic number and version
    result.extend_from_slice(&WASM_MAGIC);
    result.extend_from_slice(&WASM_VERSION);

    // Write custom sections that appear before any known section
    for section in module.custom_sections.iter() {
        // Skip name section and other sections that should appear at the end
        if section.name == "name" {
            continue;
        }
        encode_custom_section(&mut result, section)?;
    }

    // Add type section if not empty
    if !module.types.is_empty() {
        // placeholder - will be implemented in the future
    }

    // The rest of the sections would be encoded here
    // ...

    // Write name section at the end if it exists
    for section in module.custom_sections.iter() {
        if section.name == "name" {
            encode_custom_section(&mut result, section)?;
        }
    }

    Ok(result)
}

/// Encodes a custom section
fn encode_custom_section(result: &mut Vec<u8>, section: &CustomSection) -> Result<()> {
    // Write section ID (0)
    result.push(0x00);

    // Create a temporary buffer for the section content
    let mut content = Vec::new();

    // Write section name
    let name_bytes = binary::write_string(&section.name);
    content.extend_from_slice(&name_bytes);

    // Write section data
    content.extend_from_slice(&section.data);

    // Write section size
    let size_bytes = binary::write_leb128_u32(content.len() as u32);
    result.extend_from_slice(&size_bytes);

    // Write section content
    result.extend_from_slice(&content);

    Ok(())
}

// The following functions are placeholders and will be implemented in the future

fn parse_type_section(bytes: &[u8]) -> Result<(Vec<FuncType>, usize)> {
    // Placeholder - in a real implementation, you would parse the section properly
    Ok((Vec::new(), bytes.len()))
}

fn parse_import_section(bytes: &[u8]) -> Result<(Vec<Import>, usize)> {
    // Placeholder - in a real implementation, you would parse the section properly
    Ok((Vec::new(), bytes.len()))
}

fn parse_function_section(bytes: &[u8]) -> Result<(Vec<Function>, usize)> {
    // Placeholder - in a real implementation, you would parse the section properly
    Ok((Vec::new(), bytes.len()))
}

fn parse_table_section(bytes: &[u8]) -> Result<(Vec<Table>, usize)> {
    // Placeholder - in a real implementation, you would parse the section properly
    Ok((Vec::new(), bytes.len()))
}

fn parse_memory_section(bytes: &[u8]) -> Result<(Vec<Memory>, usize)> {
    // Placeholder - in a real implementation, you would parse the section properly
    Ok((Vec::new(), bytes.len()))
}

fn parse_global_section(bytes: &[u8]) -> Result<(Vec<Global>, usize)> {
    // Placeholder - in a real implementation, you would parse the section properly
    Ok((Vec::new(), bytes.len()))
}

fn parse_export_section(bytes: &[u8]) -> Result<(Vec<Export>, usize)> {
    // Placeholder - in a real implementation, you would parse the section properly
    Ok((Vec::new(), bytes.len()))
}

fn parse_element_section(bytes: &[u8]) -> Result<(Vec<Element>, usize)> {
    // Placeholder - in a real implementation, you would parse the section properly
    Ok((Vec::new(), bytes.len()))
}

fn parse_code_section(bytes: &[u8]) -> Result<(Vec<Code>, usize)> {
    // Placeholder - in a real implementation, you would parse the section properly
    Ok((Vec::new(), bytes.len()))
}

fn parse_data_section(bytes: &[u8]) -> Result<(Vec<Data>, usize)> {
    // Placeholder - in a real implementation, you would parse the section properly
    Ok((Vec::new(), bytes.len()))
}

// Section encoding functions (placeholders for a full implementation)

fn encode_type_section(_result: &mut Vec<u8>, _types: &[FuncType]) -> Result<()> {
    Ok(())
}

fn encode_import_section(result: &mut Vec<u8>, imports: &[Import]) -> Result<()> {
    // Placeholder - in a real implementation, you would encode the section properly
    Ok(())
}

fn encode_function_section(result: &mut Vec<u8>, functions: &[Function]) -> Result<()> {
    // Placeholder - in a real implementation, you would encode the section properly
    Ok(())
}

fn encode_table_section(result: &mut Vec<u8>, tables: &[Table]) -> Result<()> {
    // Placeholder - in a real implementation, you would encode the section properly
    Ok(())
}

fn encode_memory_section(result: &mut Vec<u8>, memories: &[Memory]) -> Result<()> {
    // Placeholder - in a real implementation, you would encode the section properly
    Ok(())
}

fn encode_global_section(result: &mut Vec<u8>, globals: &[Global]) -> Result<()> {
    // Placeholder - in a real implementation, you would encode the section properly
    Ok(())
}

fn encode_export_section(result: &mut Vec<u8>, exports: &[Export]) -> Result<()> {
    // Placeholder - in a real implementation, you would encode the section properly
    Ok(())
}

fn encode_start_section(result: &mut Vec<u8>, start: u32) -> Result<()> {
    // Placeholder - in a real implementation, you would encode the section properly
    Ok(())
}

fn encode_element_section(result: &mut Vec<u8>, elements: &[Element]) -> Result<()> {
    // Placeholder - in a real implementation, you would encode the section properly
    Ok(())
}

fn encode_code_section(result: &mut Vec<u8>, code: &[Code]) -> Result<()> {
    // Placeholder - in a real implementation, you would encode the section properly
    Ok(())
}

fn encode_data_section(result: &mut Vec<u8>, data: &[Data]) -> Result<()> {
    // Placeholder - in a real implementation, you would encode the section properly
    Ok(())
}
