//! Streaming parser for WebAssembly modules
//!
//! This module provides a streaming parser interface for WebAssembly modules,
//! allowing for efficient incremental processing of module sections without
//! requiring the entire module to be parsed at once.

use crate::prelude::*;
use crate::section_error::{self};
use wrt_error::{Result};
use wrt_format::binary::read_leb128_u32;
use wrt_format::section::*;
use wrt_format::module::{Import, ImportDesc, Table, Global};
use wrt_format::types::{ValueType};
use wrt_types::types::GlobalType;
use wrt_format::types::parse_value_type;

/// Represents a payload produced by the WebAssembly parser
#[derive(Debug)]
pub enum Payload<'a> {
    /// WebAssembly version
    Version(u32),
    
    /// Type section
    TypeSection(&'a [u8], usize),
    
    /// Import section
    ImportSection(&'a [u8], usize),
    
    /// Function section
    FunctionSection(&'a [u8], usize),
    
    /// Table section
    TableSection(&'a [u8], usize),
    
    /// Memory section
    MemorySection(&'a [u8], usize),
    
    /// Global section
    GlobalSection(&'a [u8], usize),
    
    /// Export section
    ExportSection(&'a [u8], usize),
    
    /// Start section
    StartSection(u32),
    
    /// Element section
    ElementSection(&'a [u8], usize),
    
    /// Code section
    CodeSection(&'a [u8], usize),
    
    /// Data section
    DataSection(&'a [u8], usize),
    
    /// Custom section
    CustomSection {
        /// Name of the custom section
        name: String,
        /// Data of the custom section
        data: &'a [u8],
        /// Size of the data
        size: usize,
    },
    
    /// End of module
    End,
}

/// A streaming parser for WebAssembly modules
///
/// This parser iterates through sections of a WebAssembly module, yielding
/// each section as a `Payload` instance. This allows for efficient processing
/// of modules without requiring the entire module to be parsed at once.
pub struct Parser<'a> {
    /// The WebAssembly binary data
    binary: &'a [u8],
    
    /// Current offset in the binary
    current_offset: usize,
    
    /// Whether the parser has processed the header
    header_processed: bool,
}

impl<'a> Parser<'a> {
    /// Create a new parser for a WebAssembly binary
    pub fn new(binary: &'a [u8]) -> Self {
        Self {
            binary,
            current_offset: 0,
            header_processed: false,
        }
    }
    
    /// Get the current offset in the binary
    pub fn current_offset(&self) -> usize {
        self.current_offset
    }
    
    /// Create an import section reader from an import section payload
    pub fn create_import_section_reader(payload: &Payload<'a>) -> Result<ImportSectionReader<'a>> {
        match payload {
            Payload::ImportSection(data, _) => ImportSectionReader::new(data),
            _ => Err(section_error::invalid_section(
                IMPORT_ID,
                0,
                "Expected import section payload",
            )),
        }
    }
    
    /// Process the WebAssembly header
    fn process_header(&mut self) -> Result<Payload<'a>> {
        // Verify the binary has at least a header
        if self.binary.len() < 8 {
            return Err(section_error::unexpected_end(0, 8, self.binary.len()));
        }
        
        // Verify magic bytes
        let expected_magic = [0x00, 0x61, 0x73, 0x6D]; // \0asm
        let actual_magic = &self.binary[0..4];
        if actual_magic != expected_magic {
            let mut actual_magic_array = [0; 4];
            actual_magic_array.copy_from_slice(actual_magic);
            return Err(section_error::invalid_magic(0, expected_magic, actual_magic_array));
        }
        
        // Read version
        let version_bytes = &self.binary[4..8];
        let version = u32::from_le_bytes(version_bytes.try_into().unwrap());
        
        // Advance past the header
        self.current_offset = 8;
        self.header_processed = true;
        
        Ok(Payload::Version(version))
    }
    
    /// Process a custom section
    fn process_custom_section(&mut self, section_size: usize) -> Result<Payload<'a>> {
        let _section_start = self.current_offset;
        
        // Read the string's length
        let (name_len, name_len_size) = read_leb128_u32(self.binary, self.current_offset)?;
        self.current_offset += name_len_size;
        
        // Ensure we have enough bytes for the name
        if self.current_offset + name_len as usize > self.binary.len() {
            return Err(section_error::unexpected_end(
                self.current_offset,
                name_len as usize,
                self.binary.len() - self.current_offset,
            ));
        }
        
        // Read the name
        let name_bytes = &self.binary[self.current_offset..self.current_offset + name_len as usize];
        let name = match std::str::from_utf8(name_bytes) {
            Ok(s) => s.to_string(),
            Err(_) => return Err(section_error::invalid_section(
                CUSTOM_ID,
                self.current_offset,
                "Invalid UTF-8 in custom section name",
            )),
        };
        self.current_offset += name_len as usize;
        
        // The data follows the name
        let name_total_size = name_len_size + name_len as usize;
        let data_size = section_size - name_total_size;
        let data = &self.binary[self.current_offset..self.current_offset + data_size];
        
        // Advance past the data
        self.current_offset += data_size;
        
        Ok(Payload::CustomSection {
            name,
            data,
            size: data_size,
        })
    }
    
    /// Process a standard (non-custom) section
    fn process_standard_section(&mut self, id: u8, section_size: usize) -> Result<Payload<'a>> {
        let data = &self.binary[self.current_offset..self.current_offset + section_size];
        let result = match id {
            TYPE_ID => Payload::TypeSection(data, section_size),
            IMPORT_ID => Payload::ImportSection(data, section_size),
            FUNCTION_ID => Payload::FunctionSection(data, section_size),
            TABLE_ID => Payload::TableSection(data, section_size),
            MEMORY_ID => Payload::MemorySection(data, section_size),
            GLOBAL_ID => Payload::GlobalSection(data, section_size),
            EXPORT_ID => Payload::ExportSection(data, section_size),
            START_ID => {
                // Start section contains a single u32 index
                if section_size == 0 {
                    return Err(section_error::invalid_section(
                        id,
                        self.current_offset,
                        "Start section cannot be empty",
                    ));
                }
                
                let (start_index, _) = read_leb128_u32(self.binary, self.current_offset)?;
                Payload::StartSection(start_index)
            }
            ELEMENT_ID => Payload::ElementSection(data, section_size),
            CODE_ID => Payload::CodeSection(data, section_size),
            DATA_ID => Payload::DataSection(data, section_size),
            _ => {
                // Unknown section - treat it like a custom section but without a name
                Payload::CustomSection {
                    name: format!("unknown_{}", id),
                    data,
                    size: section_size,
                }
            }
        };
        
        // Advance past the section
        self.current_offset += section_size;
        
        Ok(result)
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Payload<'a>>;
    
    fn next(&mut self) -> Option<Self::Item> {
        // Process header if not done yet
        if !self.header_processed {
            return Some(self.process_header());
        }
        
        // Check if we've reached the end
        if self.current_offset >= self.binary.len() {
            return Some(Ok(Payload::End));
        }
        
        // Read section ID
        let id = match self.binary.get(self.current_offset) {
            Some(&id) => id,
            None => return Some(Ok(Payload::End)),
        };
        self.current_offset += 1;
        
        // Read section size
        let (section_size, size_bytes) = match read_leb128_u32(self.binary, self.current_offset) {
            Ok((size, bytes)) => (size as usize, bytes),
            Err(e) => return Some(Err(e)),
        };
        self.current_offset += size_bytes;
        
        // Check if section extends beyond the end of the binary
        if self.current_offset + section_size > self.binary.len() {
            return Some(Err(section_error::section_size_exceeds_module(
                id,
                section_size as u32,
                self.binary.len() - self.current_offset,
                self.current_offset - size_bytes - 1, // Position of section ID
            )));
        }
        
        // Process section based on ID
        let result = if id == CUSTOM_ID {
            self.process_custom_section(section_size)
        } else {
            self.process_standard_section(id, section_size)
        };
        
        Some(result)
    }
}

/// Specialized reader for import section entries
pub struct ImportSectionReader<'a> {
    /// The raw section data
    data: &'a [u8],
    
    /// Number of entries in the section
    count: u32,
    
    /// Current entry index
    current: u32,
    
    /// Current offset within the section data
    offset: usize,
}

impl<'a> ImportSectionReader<'a> {
    /// Create a new import section reader
    pub fn new(data: &'a [u8]) -> Result<Self> {
        // Read the count of entries
        let (count, count_size) = read_leb128_u32(data, 0)?;
        
        Ok(Self {
            data,
            count,
            current: 0,
            offset: count_size,
        })
    }
    
    /// Get the total number of entries
    pub fn count(&self) -> u32 {
        self.count
    }
}

/// Helper function to read a name string from binary data
fn read_name(data: &[u8], offset: usize) -> Result<(String, usize)> {
    // Read the string's length
    let (name_len, name_len_size) = read_leb128_u32(data, offset)?;
    let name_offset = offset + name_len_size;
    
    // Ensure we have enough bytes for the name
    if name_offset + name_len as usize > data.len() {
        return Err(section_error::unexpected_end(
            name_offset, 
            name_len as usize,
            data.len() - name_offset,
        ));
    }
    
    // Read the name
    let name_bytes = &data[name_offset..name_offset + name_len as usize];
    let name = match std::str::from_utf8(name_bytes) {
        Ok(s) => s.to_string(),
        Err(_) => return Err(section_error::invalid_utf8(name_offset)),
    };
    
    Ok((name, name_len_size + name_len as usize))
}

impl<'a> Iterator for ImportSectionReader<'a> {
    type Item = Result<Import>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.count {
            return None;
        }
        
        self.current += 1;
        
        // Read module name
        let (module_name, module_name_size) = match read_name(self.data, self.offset) {
            Ok((name, size)) => (name, size),
            Err(e) => return Some(Err(e)),
        };
        self.offset += module_name_size;
        
        // Read field name
        let (field_name, field_name_size) = match read_name(self.data, self.offset) {
            Ok((name, size)) => (name, size),
            Err(e) => return Some(Err(e)),
        };
        self.offset += field_name_size;
        
        // Read import kind byte
        if self.offset >= self.data.len() {
            return Some(Err(section_error::unexpected_end(
                self.offset,
                1,
                0,
            )));
        }
        
        let import_kind = self.data[self.offset];
        self.offset += 1;
        
        // Parse based on import kind
        let desc = match import_kind {
            0x00 => {
                // Function import
                let (type_idx, type_idx_size) = match read_leb128_u32(self.data, self.offset) {
                    Ok((idx, size)) => (idx, size),
                    Err(e) => return Some(Err(e)),
                };
                self.offset += type_idx_size;
                
                ImportDesc::Function(type_idx)
            },
            0x01 => {
                // Table import
                let element_type = match self.data.get(self.offset) {
                    Some(&0x70) => ValueType::FuncRef,
                    Some(&0x6F) => ValueType::ExternRef,
                    Some(&ty) => return Some(Err(section_error::invalid_value_type(ty, self.offset))),
                    None => return Some(Err(section_error::unexpected_end(self.offset, 1, 0))),
                };
                self.offset += 1;

                // Parse limits
                let (limits, limits_size) = match crate::sections::parsers::parse_limits(&self.data[self.offset..]) {
                    Ok((limits, size)) => (limits, size),
                    Err(e) => return Some(Err(e)),
                };
                self.offset += limits_size;
                
                ImportDesc::Table(Table {
                    element_type,
                    limits,
                })
            },
            0x02 => {
                // Memory import
                let (memory, memory_size) = match crate::sections::parsers::parse_memory_type(&self.data[self.offset..]) {
                    Ok((memory, size)) => (memory, size),
                    Err(e) => return Some(Err(e)),
                };
                self.offset += memory_size;
                
                ImportDesc::Memory(memory)
            },
            0x03 => {
                // Global import
                let value_type = match self.data.get(self.offset) {
                    Some(&byte) => match parse_value_type(byte) {
                        Ok(ty) => ty,
                        Err(e) => return Some(Err(e)),
                    },
                    None => return Some(Err(section_error::unexpected_end(self.offset, 1, 0))),
                };
                self.offset += 1;
                
                // Read mutable flag
                let mutable = match self.data.get(self.offset) {
                    Some(&0) => false,
                    Some(&1) => true,
                    Some(&other) => return Some(Err(section_error::invalid_mutability(other, self.offset))),
                    None => return Some(Err(section_error::unexpected_end(self.offset, 1, 0))),
                };
                self.offset += 1;
                
                let global_type = GlobalType { 
                    value_type, 
                    mutable 
                };
                
                // Globals in imports have empty init expressions (they're initialized by the host)
                ImportDesc::Global(Global {
                    global_type,
                    init: Vec::new(),
                })
            },
            _ => return Some(Err(section_error::invalid_import_kind(import_kind, self.offset - 1))),
        };
        
        Some(Ok(Import {
            module: module_name,
            name: field_name,
            desc,
        }))
    }
}

/// Helper function to calculate the size of a LEB128 encoded value
fn varuint_size(value: u32) -> usize {
    let mut size = 1;
    let mut val = value >> 7;
    while val != 0 {
        size += 1;
        val >>= 7;
    }
    size
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WASM_MAGIC;
    use wrt_format::binary::WASM_VERSION;
    
    // Helper to create a minimal valid WebAssembly module with specific sections
    fn create_test_module(sections: &[(u8, Vec<u8>)]) -> Vec<u8> {
        let mut module = Vec::new();
        
        // Add header
        module.extend_from_slice(&WASM_MAGIC);
        module.extend_from_slice(&WASM_VERSION);
        
        // Add sections
        for (id, data) in sections {
            module.push(*id);
            
            // Write the size as LEB128
            let mut size = data.len();
            loop {
                let mut byte = (size & 0x7F) as u8;
                size >>= 7;
                if size != 0 {
                    byte |= 0x80;
                }
                module.push(byte);
                if size == 0 {
                    break;
                }
            }
            
            // Add the section data
            module.extend_from_slice(data);
        }
        
        module
    }
    
    // Helper to create LEB128 encoded u32
    fn create_leb128_u32(value: u32) -> Vec<u8> {
        let mut result = Vec::new();
        let mut val = value;
        loop {
            let mut byte = (val & 0x7F) as u8;
            val >>= 7;
            if val != 0 {
                byte |= 0x80;
            }
            result.push(byte);
            if val == 0 {
                break;
            }
        }
        result
    }
    
    // Helper to encode a name string with LEB128 length prefix
    fn encode_name(name: &str) -> Vec<u8> {
        let mut result = create_leb128_u32(name.len() as u32);
        result.extend_from_slice(name.as_bytes());
        result
    }
    
    #[test]
    fn test_import_section_reader() {
        // Create an import section with:
        // 1. Function import from "env" / "func1" with type index 0
        // 2. Memory import from "env" / "memory" with min=1, max=2
        // 3. Table import from "env" / "table" with element_type=funcref, min=1, max=None
        // 4. Global import from "env" / "global" with type=i32, mutable=true
        
        let mut import_section = Vec::new();
        
        // Count of imports (4)
        import_section.extend_from_slice(&[0x04]);
        
        // Import 1: Function from "env"/"func1" with type index 0
        import_section.extend_from_slice(&encode_name("env"));
        import_section.extend_from_slice(&encode_name("func1"));
        import_section.extend_from_slice(&[0x00]); // Function import
        import_section.extend_from_slice(&create_leb128_u32(0)); // Type index 0
        
        // Import 2: Memory from "env"/"memory" with min=1, max=2
        import_section.extend_from_slice(&encode_name("env"));
        import_section.extend_from_slice(&encode_name("memory"));
        import_section.extend_from_slice(&[0x02]); // Memory import
        import_section.extend_from_slice(&[0x01]); // Has max
        import_section.extend_from_slice(&create_leb128_u32(1)); // Min pages = 1
        import_section.extend_from_slice(&create_leb128_u32(2)); // Max pages = 2
        
        // Import 3: Table from "env"/"table" with element_type=funcref, min=1, max=None
        import_section.extend_from_slice(&encode_name("env"));
        import_section.extend_from_slice(&encode_name("table"));
        import_section.extend_from_slice(&[0x01]); // Table import
        import_section.extend_from_slice(&[0x70]); // funcref
        import_section.extend_from_slice(&[0x00]); // No max
        import_section.extend_from_slice(&create_leb128_u32(1)); // Min = 1
        
        // Import 4: Global from "env"/"global" with type=i32, mutable=true
        import_section.extend_from_slice(&encode_name("env"));
        import_section.extend_from_slice(&encode_name("global"));
        import_section.extend_from_slice(&[0x03]); // Global import
        import_section.extend_from_slice(&[0x7F]); // i32
        import_section.extend_from_slice(&[0x01]); // mutable = true
        
        // Create a module with just the import section
        let module = create_test_module(&[(IMPORT_ID, import_section)]);
        
        // Create a parser and find the import section
        let mut parser = Parser::new(&module);
        
        // Skip version
        parser.next().unwrap().unwrap();
        
        // Get import section
        let payload = parser.next().unwrap().unwrap();
        
        match &payload {
            Payload::ImportSection(_, _) => {} // Expected
            other => panic!("Unexpected payload: {:?}", other),
        }
        
        // Create an import section reader
        let reader = Parser::create_import_section_reader(&payload).unwrap();
        assert_eq!(reader.count, 4);
        
        // Collect all imports
        let imports: Vec<_> = reader.collect::<Result<Vec<_>>>().unwrap();
        assert_eq!(imports.len(), 4);
        
        // Check the first import (function)
        assert_eq!(imports[0].module, "env");
        assert_eq!(imports[0].name, "func1");
        match &imports[0].desc {
            ImportDesc::Function(idx) => assert_eq!(*idx, 0),
            other => panic!("Unexpected import desc: {:?}", other),
        }
        
        // Check the second import (memory)
        assert_eq!(imports[1].module, "env");
        assert_eq!(imports[1].name, "memory");
        match &imports[1].desc {
            ImportDesc::Memory(memory) => {
                assert_eq!(memory.limits.min, 1);
                assert_eq!(memory.limits.max, Some(2));
                assert_eq!(memory.shared, false);
            }
            other => panic!("Unexpected import desc: {:?}", other),
        }
        
        // Check the third import (table)
        assert_eq!(imports[2].module, "env");
        assert_eq!(imports[2].name, "table");
        match &imports[2].desc {
            ImportDesc::Table(table) => {
                assert_eq!(table.element_type, ValueType::FuncRef);
                assert_eq!(table.limits.min, 1);
                assert_eq!(table.limits.max, None);
            }
            other => panic!("Unexpected import desc: {:?}", other),
        }
        
        // Check the fourth import (global)
        assert_eq!(imports[3].module, "env");
        assert_eq!(imports[3].name, "global");
        match &imports[3].desc {
            ImportDesc::Global(global) => {
                assert_eq!(global.global_type.value_type, ValueType::I32);
                assert_eq!(global.global_type.mutable, true);
            }
            other => panic!("Unexpected import desc: {:?}", other),
        }
    }
    
    #[test]
    fn test_parser_header() {
        // Create a minimal valid module
        let module = create_test_module(&[]);
        
        // Create a parser and get the first item
        let mut parser = Parser::new(&module);
        let result = parser.next().unwrap().unwrap();
        
        // Check that we got a version payload
        match result {
            Payload::Version(1) => {} // Expected result
            other => panic!("Unexpected payload: {:?}", other),
        }
        
        // Check that the next item is End
        let result = parser.next().unwrap().unwrap();
        match result {
            Payload::End => {} // Expected result
            other => panic!("Unexpected payload: {:?}", other),
        }
    }
    
    #[test]
    fn test_parser_sections() {
        // Create a module with multiple sections
        let type_section = vec![0x01, 0x60, 0x00, 0x00]; // 1 type, func() -> ()
        let function_section = vec![0x01, 0x00]; // 1 function, type index 0
        let code_section = vec![0x01, 0x04, 0x00, 0x0B]; // 1 function, empty body
        
        let module = create_test_module(&[
            (TYPE_ID, type_section),
            (FUNCTION_ID, function_section),
            (CODE_ID, code_section),
        ]);
        
        // Create a parser - limit the number of iterations to avoid infinite loops
        let mut parser = Parser::new(&module);
        let mut payloads = Vec::new();
        
        // Collect up to 10 payloads to prevent potential infinite loops
        for _ in 0..10 {
            match parser.next() {
                Some(Ok(payload)) => {
                    let is_end = matches!(payload, Payload::End);
                    payloads.push(payload);
                    if is_end {
                        break;
                    }
                },
                Some(Err(e)) => panic!("Parser error: {:?}", e),
                None => break,
            }
        }
        
        // Check we got the expected number of payloads (version + 3 sections + end)
        assert_eq!(payloads.len(), 5);
        
        // Check the types of the payloads
        match &payloads[0] {
            Payload::Version(1) => {} // Expected
            other => panic!("Unexpected first payload: {:?}", other),
        }
        
        match &payloads[1] {
            Payload::TypeSection(_, _) => {} // Expected
            other => panic!("Unexpected second payload: {:?}", other),
        }
        
        match &payloads[2] {
            Payload::FunctionSection(_, _) => {} // Expected
            other => panic!("Unexpected third payload: {:?}", other),
        }
        
        match &payloads[3] {
            Payload::CodeSection(_, _) => {} // Expected
            other => panic!("Unexpected fourth payload: {:?}", other),
        }
        
        match &payloads[4] {
            Payload::End => {} // Expected
            other => panic!("Unexpected fifth payload: {:?}", other),
        }
    }
    
    #[test]
    fn test_parser_custom_section() {
        // Create a module with a custom section
        let mut custom_data = Vec::new();
        
        // Name "test"
        custom_data.push(4); // name length
        custom_data.extend_from_slice(b"test");
        
        // Some data
        custom_data.extend_from_slice(b"custom data");
        
        let module = create_test_module(&[(CUSTOM_ID, custom_data)]);
        
        // Create a parser and check the payloads
        let mut parser = Parser::new(&module);
        
        // Skip version
        parser.next().unwrap().unwrap();
        
        // Check custom section
        let payload = parser.next().unwrap().unwrap();
        match payload {
            Payload::CustomSection { name, data, size } => {
                assert_eq!(name, "test");
                assert_eq!(data, b"custom data");
                assert_eq!(size, 11); // "custom data".len()
            }
            other => panic!("Unexpected payload: {:?}", other),
        }
        
        // Check end
        let payload = parser.next().unwrap().unwrap();
        match payload {
            Payload::End => {} // Expected
            other => panic!("Unexpected payload: {:?}", other),
        }
    }
    
    #[test]
    fn test_parser_invalid_header() {
        // Create an invalid module (wrong magic)
        let mut module = vec![0x01, 0x61, 0x73, 0x6D]; // wrong first byte
        module.extend_from_slice(&WASM_VERSION);
        
        // Create a parser and check it fails
        let mut parser = Parser::new(&module);
        let result = parser.next().unwrap();
        assert!(result.is_err());
    }
    
    #[test]
    fn test_parser_section_too_large() {
        // Create a module with a section that extends beyond the end
        let mut module = Vec::new();
        
        // Add header
        module.extend_from_slice(&WASM_MAGIC);
        module.extend_from_slice(&WASM_VERSION);
        
        // Add a section with a size that's too large
        module.push(TYPE_ID);
        module.push(0xFF); // Size of 127, but we'll only include 5 bytes
        module.extend_from_slice(&[0x01, 0x60, 0x00, 0x00, 0x00]); // Truncated data
        
        // Create a parser and check it fails when processing the section
        let mut parser = Parser::new(&module);
        
        // Skip version
        parser.next().unwrap().unwrap();
        
        // Check section fails
        let result = parser.next().unwrap();
        assert!(result.is_err());
    }
} 