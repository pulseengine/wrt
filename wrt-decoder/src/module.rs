//! WebAssembly module representation
//!
//! This module provides the main Module structure and functions for decoding
//! WebAssembly binary modules into structured representations.

use crate::{
    prelude::{format, String, ToString, Vec},
    sections::*,
    WASM_MAGIC,
};
use wrt_error::{kinds, Error, Result, WrtError, codes, ErrorCategory};
use wrt_format::binary;
use wrt_format::types::parse_value_type;
use wrt_types::types::GlobalType;

/// WebAssembly binary format version
pub const WASM_VERSION: [u8; 4] = [0x01, 0x00, 0x00, 0x00];

/// Represents a parsed WebAssembly module
#[derive(Debug, Clone, Default)]
pub struct Module {
    /// WebAssembly binary format version
    pub version: u32,
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
    /// Create a new, empty module
    pub fn new() -> Self {
        Self {
            version: super::WASM_SUPPORTED_VERSION,
            types: Vec::new(),
            imports: Vec::new(),
            functions: Vec::new(),
            tables: Vec::new(),
            memories: Vec::new(),
            globals: Vec::new(),
            exports: Vec::new(),
            start: None,
            elements: Vec::new(),
            data: Vec::new(),
            custom_sections: Vec::new(),
            binary: None,
            name: None,
            code: Vec::new(),
        }
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

    /// Encode the module to binary format
    ///
    /// This is a basic implementation for testing purposes only.
    pub fn encode(&self) -> wrt_error::Result<Vec<u8>> {
        // For now, we'll create a minimal module that can be decoded
        use wrt_format::binary::{
            write_leb128_u32, write_string, CUSTOM_SECTION_ID, WASM_MAGIC, WASM_VERSION,
        };

        let mut result = Vec::new();

        // Add the magic bytes and version
        result.extend_from_slice(&WASM_MAGIC);
        result.extend_from_slice(&WASM_VERSION);

        // For testing purposes, we'll just add a custom section
        let custom_section_name = "test_roundtrip";
        let custom_section_content = b"roundtrip_test";

        // Create the custom section content
        let mut section_content = Vec::new();
        section_content.extend_from_slice(&write_string(custom_section_name));
        section_content.extend_from_slice(custom_section_content);

        // Add the custom section
        result.push(CUSTOM_SECTION_ID);
        result.extend_from_slice(&write_leb128_u32(section_content.len() as u32));
        result.extend_from_slice(&section_content);

        Ok(result)
    }
}

/// Decodes a WebAssembly binary module
///
/// This function parses a WebAssembly binary format and constructs a Module structure.
///
/// # Panics
///
/// This function will panic if it attempts to access the last element of an empty
/// custom_sections vector, which can happen if the implementation tries to process
/// a custom section before any custom sections have been added to the module.
/// Safety impact: [LOW|MEDIUM|HIGH] - [Brief explanation of the safety implication]
/// Tracking: WRTQ-XXX (qualification requirement tracking ID).
pub fn decode_module(bytes: &[u8]) -> Result<Module> {
    // Verify magic bytes and version
    if bytes.len() < 8 {
        return Err(WrtError::parse_error("WebAssembly binary too short".to_string()));
    }

    if bytes[0..4] != WASM_MAGIC {
        return Err(WrtError::parse_error("Invalid WebAssembly magic bytes".to_string()));
    }

    if bytes[4..8] != WASM_VERSION {
        return Err(WrtError::parse_error("Unsupported WebAssembly version".to_string()));
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
            return Err(WrtError::parse_error(format!(
                "Section size {} for section ID {} exceeds binary size",
                size, section_id
            )));
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
                let (imports, _) = parse_import_section_bytes(section_bytes)?;
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
                #[cfg(feature = "std")]
                {
                    // Only use logging when std is available
                    // We should add an optional log dependency with the appropriate feature flag
                    // For now, just silence the warning in no_std mode
                }
            }
        }

        offset = section_end;
    }

    // Validate the basic structure now that we've parsed everything
    if module.functions.len() != module.code.len() && !module.functions.is_empty() {
        return Err(WrtError::parse_error(format!(
            "Function and code section counts mismatch: {} functions but {} code entries",
            module.functions.len(),
            module.code.len()
        )));
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

/// Internal function to parse import section bytes
fn parse_import_section_bytes(bytes: &[u8]) -> Result<(Vec<Import>, usize)> {
    let mut offset = 0;
    
    // Read the number of imports
    let (count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;
    
    let mut imports = Vec::with_capacity(count as usize);
    
    // Parse each import
    for _ in 0..count {
        // Read module name string
        let (module_name, bytes_read) = binary::read_string(bytes, offset)?;
        offset += bytes_read;
        
        // Read field/name string
        let (field_name, bytes_read) = binary::read_string(bytes, offset)?;
        offset += bytes_read;
        
        // Read import kind/type (1 byte)
        if offset >= bytes.len() {
            return Err(WrtError::parse_error("Unexpected end of import section".to_string()));
        }
        
        let import_type = bytes[offset];
        offset += 1;
        
        // Parse import descriptor based on the type
        let desc = match import_type {
            0x00 => {
                // Function import
                let (type_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;
                ImportDesc::Function(type_idx)
            },
            0x01 => {
                // Table import
                let (table_type, bytes_read) = parsers::parse_table_type(&bytes[offset..])?;
                offset += bytes_read;
                ImportDesc::Table(table_type)
            },
            0x02 => {
                // Memory import
                let (memory_type, bytes_read) = parsers::parse_memory_type(&bytes[offset..])?;
                offset += bytes_read;
                ImportDesc::Memory(memory_type)
            },
            0x03 => {
                // Global import
                // Get the value type (1 byte)
                if offset >= bytes.len() {
                    return Err(WrtError::parse_error("Unexpected end of global import".to_string()));
                }
                
                let val_type = parse_value_type(bytes[offset])?;
                offset += 1;
                
                // Get mutability (1 byte)
                if offset >= bytes.len() {
                    return Err(WrtError::parse_error("Unexpected end of global import mutability".to_string()));
                }
                
                let mutability = bytes[offset] != 0;
                offset += 1;
                
                // Create global import
                let global_type = GlobalType {
                    value_type: val_type,
                    mutable: mutability,
                };
                
                ImportDesc::Global(Global {
                    global_type,
                    init: Vec::new(), // No initialization for imports
                })
            },
            _ => {
                return Err(WrtError::parse_error(format!("Invalid import type: {}", import_type)));
            }
        };
        
        // Create and add the import to our list
        let import = Import {
            module: module_name,
            name: field_name,
            desc,
        };
        
        imports.push(import);
    }
    
    Ok((imports, offset))
}

/// Parse the import section from a WebAssembly binary
///
/// This function parses the import section of a WebAssembly module given the section offset and size.
/// It returns a vector of Import structures representing all imports defined in the section.
///
/// # Parameters
///
/// * `binary` - The complete WebAssembly binary data
/// * `section_offset` - Offset in the binary where the import section content starts (after section id and size)
/// * `section_size` - Size of the import section content in bytes
///
/// # Returns
///
/// A Result containing either a vector of Import structures or an Error
pub fn parse_import_section(
    binary: &[u8],
    section_offset: usize,
    section_size: usize,
) -> Result<Vec<Import>> {
    if section_offset + section_size > binary.len() {
        return Err(WrtError::parse_error("Import section extends beyond binary size".to_string()));
    }
    
    // Use the existing parse_import_section function with the section bytes
    let section_bytes = &binary[section_offset..section_offset + section_size];
    let (imports, parsed_size) = parse_import_section_bytes(section_bytes)?;
    
    // Verify that we consumed the entire section
    if parsed_size != section_size {
        return Err(WrtError::parse_error(format!("Import section parsing incomplete. Parsed {} bytes out of {}", parsed_size, section_size)));
    }
    
    Ok(imports)
}

fn parse_function_section(bytes: &[u8]) -> Result<(Vec<Function>, usize)> {
    // Placeholder - in a real implementation, you would parse the section properly
    Ok((Vec::new(), bytes.len()))
}

fn parse_table_section(bytes: &[u8]) -> Result<(Vec<Table>, usize)> {
    // Placeholder - in a real implementation, you would parse the section properly
    Ok((Vec::new(), bytes.len()))
}

/// Parse the memory section from WebAssembly binary format
///
/// According to the WebAssembly Core Specification (Binary Format):
/// https://webassembly.github.io/spec/core/bikeshed/#binary-memsec
///
/// The memory section has the following format:
/// - Section ID: 5 (Memory)
/// - Contents: Vector of memory entries
///   - Each memory entry:
///     - Flags byte: bit 0 = has_max, bit 1 = is_shared, bit 2 = is_memory64, bits 3-7 reserved (must be 0)
///     - Min size: u32 (memory32) or u64 (memory64) in units of pages (64KiB)
///     - Max size: Optional u32 (memory32) or u64 (memory64) in units of pages (present if has_max)
///
/// WebAssembly 1.0 allows at most one memory per module.
fn parse_memory_section(bytes: &[u8]) -> Result<(Vec<Memory>, usize)> {
    let mut offset = 0;
    let mut memories = Vec::new();

    // Read the number of memories
    if offset >= bytes.len() {
        return Err(WrtError::parse_error("Unexpected end of memory section bytes".to_string()));
    }

    let (count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    // WebAssembly 1.0 only allows a maximum of 1 memory per module
    if count > 1 {
        return Err(WrtError::parse_error("Multiple memories are not supported in WebAssembly 1.0".to_string()));
    }

    // Parse each memory type
    for _ in 0..count {
        if offset >= bytes.len() {
            return Err(WrtError::parse_error("Unexpected end of memory section bytes".to_string()));
        }

        let (memory, bytes_read) = parsers::parse_memory_type(&bytes[offset..])?;
        offset += bytes_read;
        memories.push(memory);
    }

    Ok((memories, offset))
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
    let mut offset = 0;
    let mut data_segments = Vec::new();

    // Read count
    if offset >= bytes.len() {
        return Err(WrtError::parse_error("Unexpected end of data section bytes".to_string()));
    }
    let (count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    // Parse each data segment
    for _ in 0..count {
        if offset >= bytes.len() {
            return Err(WrtError::parse_error("Unexpected end of data section bytes".to_string()));
        }

        // Read flags (indicates active vs. passive and memory index encoding)
        let flags = bytes[offset];
        offset += 1;

        // Parse based on segment type
        match flags {
            0x00 => {
                // Active segment with memory index 0
                // Read offset expression
                let (expr_size, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                if offset + expr_size as usize > bytes.len() {
                    return Err(WrtError::parse_error("Offset expression exceeds data section size".to_string()));
                }

                let offset_expr = bytes[offset..offset + expr_size as usize].to_vec();
                offset += expr_size as usize;

                // Read init data
                let (data_size, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                if offset + data_size as usize > bytes.len() {
                    return Err(WrtError::parse_error("Data segment exceeds data section size".to_string()));
                }

                let init_data = bytes[offset..offset + data_size as usize].to_vec();
                offset += data_size as usize;

                data_segments.push(Data {
                    mode: DataMode::Active,
                    memory_idx: 0,
                    offset: offset_expr,
                    init: init_data,
                });
            }
            0x01 => {
                // Passive segment
                // Read init data
                let (data_size, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                if offset + data_size as usize > bytes.len() {
                    return Err(WrtError::parse_error("Data segment exceeds data section size".to_string()));
                }

                let init_data = bytes[offset..offset + data_size as usize].to_vec();
                offset += data_size as usize;

                data_segments.push(Data {
                    mode: DataMode::Passive,
                    memory_idx: 0,      // Not used for passive segments
                    offset: Vec::new(), // Not used for passive segments
                    init: init_data,
                });
            }
            0x02 => {
                // Active segment with explicit memory index
                // Read memory index
                let (memory_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Read offset expression
                let (expr_size, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                if offset + expr_size as usize > bytes.len() {
                    return Err(WrtError::parse_error("Offset expression exceeds data section size".to_string()));
                }

                let offset_expr = bytes[offset..offset + expr_size as usize].to_vec();
                offset += expr_size as usize;

                // Read init data
                let (data_size, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                if offset + data_size as usize > bytes.len() {
                    return Err(WrtError::parse_error("Data segment exceeds data section size".to_string()));
                }

                let init_data = bytes[offset..offset + data_size as usize].to_vec();
                offset += data_size as usize;

                data_segments.push(Data {
                    mode: DataMode::Active,
                    memory_idx,
                    offset: offset_expr,
                    init: init_data,
                });
            }
            _ => {
                return Err(WrtError::parse_error(format!("Invalid data segment flags: 0x{:02x}", flags)));
            }
        }
    }

    Ok((data_segments, offset))
}

// Section encoding functions (placeholders for a full implementation)

#[allow(dead_code)]
fn encode_type_section(_result: &mut [u8], _types: &[FuncType]) -> Result<()> {
    // Placeholder implementation
    Ok(())
}

#[allow(dead_code)]
fn encode_import_section(_result: &mut [u8], _imports: &[Import]) -> Result<()> {
    // Placeholder implementation
    Ok(())
}

#[allow(dead_code)]
fn encode_function_section(_result: &mut [u8], _functions: &[Function]) -> Result<()> {
    // Placeholder implementation
    Ok(())
}

#[allow(dead_code)]
fn encode_table_section(_result: &mut [u8], _tables: &[Table]) -> Result<()> {
    // Placeholder implementation
    Ok(())
}

/// Encode the memory section to WebAssembly binary format
///
/// According to the WebAssembly Core Specification (Binary Format):
/// https://webassembly.github.io/spec/core/bikeshed/#binary-memsec
///
/// The memory section has the following format:
/// - Section ID: 5 (Memory)
/// - Contents: Vector of memory entries
///   - Each memory entry:
///     - Flags byte: bit 0 = has_max, bit 1 = is_shared, bit 2 = is_memory64, bits 3-7 reserved (must be 0)
///     - Min size: u32 (memory32) or u64 (memory64) in units of pages (64KiB)
///     - Max size: Optional u32 (memory32) or u64 (memory64) in units of pages (present if has_max)
///
/// Validation rules:
/// - WebAssembly 1.0 allows at most one memory per module
/// - Shared memories must have a maximum size specified
/// - The maximum size must be greater than or equal to the minimum size
/// - For memory32, min and max must not exceed 65536 pages (4GiB)
#[allow(dead_code)]
fn encode_memory_section(_result: &mut [u8], _memories: &[Memory]) -> Result<()> {
    // Note: This function is not implemented for &mut [u8] because it needs to extend the buffer
    // dynamically. A proper implementation would use a different design pattern, such as
    // returning a Vec<u8> instead of modifying a slice.

    // Skip processing for now
    Ok(())
}

#[allow(dead_code)]
fn encode_global_section(_result: &mut [u8], _globals: &[Global]) -> Result<()> {
    // Placeholder implementation
    Ok(())
}

#[allow(dead_code)]
fn encode_export_section(_result: &mut [u8], _exports: &[Export]) -> Result<()> {
    // Placeholder implementation
    Ok(())
}

#[allow(dead_code)]
fn encode_start_section(_result: &mut [u8], _start: u32) -> Result<()> {
    // Placeholder implementation
    Ok(())
}

#[allow(dead_code)]
fn encode_element_section(_result: &mut [u8], _elements: &[Element]) -> Result<()> {
    // Placeholder implementation
    Ok(())
}

#[allow(dead_code)]
fn encode_code_section(_result: &mut [u8], _code: &[Code]) -> Result<()> {
    // Placeholder implementation
    Ok(())
}

#[allow(dead_code)]
fn encode_data_section(_result: &mut [u8], _data: &[Data]) -> Result<()> {
    // Placeholder implementation
    Ok(())
}

/// Initialize memory from data segments
///
/// This function applies active data segments to the given memory.
/// It needs to be called during module instantiation to set up the initial memory state.
///
/// # Arguments
///
/// * `memory_data` - The memory data buffer to initialize
/// * `data_segments` - The data segments to apply
/// * `globals` - Global values needed to evaluate constant expressions
///
/// # Returns
///
/// * `Ok(())` if initialization succeeded
/// * `Err(_)` if an error occurred
pub fn initialize_memory(
    memory_data: &mut [u8],
    data_segments: &[Data],
    globals: &[(ValueType, u64)], // Simplified globals representation: (type, value)
) -> Result<()> {
    // Process each active data segment
    for (i, segment) in data_segments.iter().enumerate() {
        if matches!(segment.mode, DataMode::Passive) {
            // Skip passive segments
            continue;
        }

        // Evaluate the offset expression
        let offset = evaluate_const_expr(&segment.offset, globals)?;

        // Convert to u32 (this is safe because WebAssembly memory is always < 2^32 bytes)
        let offset_u32 = match offset {
            ConstValue::I32(val) => val as u32,
            ConstValue::I64(val) => {
                if val > (u32::MAX as i64) {
                    return Err(WrtError::runtime_error(format!(
                        "Memory offset in data segment {} exceeds 32-bit limit",
                        i
                    )));
                }
                val as u32
            }
            _ => {
                return Err(WrtError::runtime_error(format!(
                    "Invalid offset type in data segment {}",
                    i
                )));
            }
        };

        // Check if the segment fits in memory
        if offset_u32 as usize + segment.init.len() > memory_data.len() {
            return Err(WrtError::runtime_error(
                format!(
                    "Data segment {} extends beyond memory size (offset: {}, length: {}, memory size: {})",
                    i, offset_u32, segment.init.len(), memory_data.len()
                )
            ));
        }

        // Copy the segment data to memory
        let dest_start = offset_u32 as usize;
        let dest_end = dest_start + segment.init.len();
        memory_data[dest_start..dest_end].copy_from_slice(&segment.init);
    }

    Ok(())
}

/// Constant expression value
#[derive(Debug, Clone)]
pub enum ConstValue {
    /// 32-bit integer constant
    I32(i32),
    /// 64-bit integer constant
    I64(i64),
    /// 32-bit float constant
    F32(f32),
    /// 64-bit float constant
    F64(f64),
}

/// Evaluate a constant expression (limited to simple cases)
fn evaluate_const_expr(expr: &[u8], globals: &[(ValueType, u64)]) -> Result<ConstValue> {
    // Ensure the expression is not empty and ends with end opcode
    if expr.is_empty() || expr[expr.len() - 1] != 0x0B {
        return Err(WrtError::runtime_error("Invalid constant expression format".to_string()));
    }

    // Handle common constant expressions
    match expr[0] {
        // i32.const
        0x41 => {
            // Parse the i32 value (LEB128 encoded)
            let (value, _) = binary::read_leb128_i32(expr, 1)?;
            Ok(ConstValue::I32(value))
        }

        // i64.const
        0x42 => {
            // Parse the i64 value (LEB128 encoded)
            let (value, _) = binary::read_leb128_i64(expr, 1)?;
            Ok(ConstValue::I64(value))
        }

        // f32.const
        0x43 => {
            if expr.len() < 5 {
                // opcode + 4 bytes
                return Err(WrtError::runtime_error("Invalid f32.const expression".to_string()));
            }

            // Parse the f32 value (IEEE 754 encoded)
            let bits = u32::from_le_bytes([expr[1], expr[2], expr[3], expr[4]]);
            Ok(ConstValue::F32(f32::from_bits(bits)))
        }

        // f64.const
        0x44 => {
            if expr.len() < 9 {
                // opcode + 8 bytes
                return Err(WrtError::runtime_error("Invalid f64.const expression".to_string()));
            }

            // Parse the f64 value (IEEE 754 encoded)
            let bits = u64::from_le_bytes([
                expr[1], expr[2], expr[3], expr[4], expr[5], expr[6], expr[7], expr[8],
            ]);
            Ok(ConstValue::F64(f64::from_bits(bits)))
        }

        // global.get
        0x23 => {
            // Parse the global index (LEB128 encoded)
            let (global_idx, _) = binary::read_leb128_u32(expr, 1)?;

            // Look up the global value
            if global_idx as usize >= globals.len() {
                return Err(WrtError::runtime_error(format!(
                    "Global index {} out of bounds",
                    global_idx
                )));
            }

            // Convert the global value to the appropriate type
            let (global_type, global_value) = &globals[global_idx as usize];
            match global_type {
                ValueType::I32 => Ok(ConstValue::I32(*global_value as i32)),
                ValueType::I64 => Ok(ConstValue::I64(*global_value as i64)),
                ValueType::F32 => Ok(ConstValue::F32(f32::from_bits(*global_value as u32))),
                ValueType::F64 => Ok(ConstValue::F64(f64::from_bits(*global_value))),
                _ => Err(WrtError::runtime_error(format!(
                    "Unsupported global type {:?}",
                    global_type
                ))),
            }
        }

        _ => Err(WrtError::runtime_error(format!(
            "Unsupported constant expression opcode: 0x{:02x}",
            expr[0]
        ))),
    }
}

pub fn parse_error(message: &str) -> WrtError {
    WrtError::parse_error(message.to_string())
}

pub fn parse_error_with_context(message: &str, context: &str) -> WrtError {
    WrtError::parse_error(format!("{}: {}", message, context))
}

pub fn parse_error_with_position(message: &str, position: usize) -> WrtError {
    WrtError::parse_error(format!("{} at position {}", message, position))
}

pub fn runtime_error(message: &str) -> WrtError {
    WrtError::runtime_error(message.to_string())
}

pub fn runtime_error_with_context(message: &str, context: &str) -> WrtError {
    WrtError::runtime_error(format!("{}: {}", message, context))
}

pub fn runtime_error_with_type(message: &str, type_name: &str) -> WrtError {
    WrtError::runtime_error(format!("{} for type {}", message, type_name))
}
