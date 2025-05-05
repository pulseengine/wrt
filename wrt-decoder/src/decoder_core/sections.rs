//! WebAssembly Core Module Section Handling
//!
//! This module provides functions for parsing and generating WebAssembly module sections.

use crate::Result;
use wrt_error::{codes, Error, ErrorCategory};
use wrt_format::Section;
use crate::prelude::*;

// All collection types are now imported from the prelude

// Section IDs from the WebAssembly spec
const CUSTOM_SECTION_ID: u8 = 0;
const TYPE_SECTION_ID: u8 = 1;
const IMPORT_SECTION_ID: u8 = 2;
const FUNCTION_SECTION_ID: u8 = 3;
const TABLE_SECTION_ID: u8 = 4;
const MEMORY_SECTION_ID: u8 = 5;
const GLOBAL_SECTION_ID: u8 = 6;
const EXPORT_SECTION_ID: u8 = 7;
const START_SECTION_ID: u8 = 8;
const ELEMENT_SECTION_ID: u8 = 9;
const CODE_SECTION_ID: u8 = 10;
const DATA_SECTION_ID: u8 = 11;
const DATA_COUNT_SECTION_ID: u8 = 12;

// Error codes
const ERROR_INVALID_OFFSET: u16 = codes::PARSE_ERROR;
const ERROR_INVALID_SECTION: u16 = codes::PARSE_ERROR;

/// Parse a WebAssembly section from a byte array
///
/// # Arguments
///
/// * `data` - The byte array containing the section
/// * `offset` - The offset in the byte array at which the section starts
///
/// # Returns
///
/// * `Result<(Section, usize)>` - The parsed section and the number of bytes read
///
/// # Errors
///
/// Returns an error if the section cannot be parsed.
pub fn parse_section(data: &[u8], offset: usize) -> Result<(Section, usize)> {
    if offset >= data.len() {
        return Err(Error::new(
            ErrorCategory::Parse,
            ERROR_INVALID_OFFSET,
            "Section offset exceeds data length",
        ));
    }

    // Read section ID
    let section_id = data[offset];
    let mut bytes_read = 1;

    // Read section size
    let (section_size, size_bytes) = read_leb128_u32(&data[offset + bytes_read..], 0)?;
    bytes_read += size_bytes;

    // Read section content
    let section_data = &data[offset + bytes_read..offset + bytes_read + section_size as usize];
    bytes_read += section_size as usize;

    // Parse section based on ID
    let section = match section_id {
        CUSTOM_SECTION_ID => {
            // Custom section
            let (name, name_bytes) = read_string(section_data, 0)?;
            let payload = &section_data[name_bytes..];
            let custom_section = wrt_format::section::CustomSection {
                name,
                data: payload.to_vec(),
            };
            Section::Custom(custom_section)
        }
        TYPE_SECTION_ID => {
            // Type section
            Section::Type(section_data.to_vec())
        }
        IMPORT_SECTION_ID => {
            // Import section
            Section::Import(section_data.to_vec())
        }
        FUNCTION_SECTION_ID => {
            // Function section
            Section::Function(section_data.to_vec())
        }
        TABLE_SECTION_ID => {
            // Table section
            Section::Table(section_data.to_vec())
        }
        MEMORY_SECTION_ID => {
            // Memory section
            Section::Memory(section_data.to_vec())
        }
        GLOBAL_SECTION_ID => {
            // Global section
            Section::Global(section_data.to_vec())
        }
        EXPORT_SECTION_ID => {
            // Export section
            Section::Export(section_data.to_vec())
        }
        START_SECTION_ID => {
            // Start section
            Section::Start(section_data.to_vec())
        }
        ELEMENT_SECTION_ID => {
            // Element section
            Section::Element(section_data.to_vec())
        }
        CODE_SECTION_ID => {
            // Code section
            Section::Code(section_data.to_vec())
        }
        DATA_SECTION_ID => {
            // Data section
            Section::Data(section_data.to_vec())
        }
        DATA_COUNT_SECTION_ID => {
            // Data count section
            Section::DataCount(section_data.to_vec())
        }
        _ => {
            // Unknown section
            return Err(Error::new(
                ErrorCategory::Parse,
                ERROR_INVALID_SECTION,
                format!("Unknown section ID: {}", section_id),
            ));
        }
    };

    Ok((section, bytes_read))
}

/// Generate a WebAssembly section
///
/// # Arguments
///
/// * `section` - The section to encode
///
/// # Returns
///
/// * `Result<Vec<u8>>` - The encoded section
///
/// # Errors
///
/// Returns an error if the section cannot be encoded.
pub fn generate_section(section: &Section) -> Result<Vec<u8>> {
    let mut result = Vec::new();

    // Add section ID
    match section {
        Section::Custom(_) => result.push(CUSTOM_SECTION_ID),
        Section::Type(_) => result.push(TYPE_SECTION_ID),
        Section::Import(_) => result.push(IMPORT_SECTION_ID),
        Section::Function(_) => result.push(FUNCTION_SECTION_ID),
        Section::Table(_) => result.push(TABLE_SECTION_ID),
        Section::Memory(_) => result.push(MEMORY_SECTION_ID),
        Section::Global(_) => result.push(GLOBAL_SECTION_ID),
        Section::Export(_) => result.push(EXPORT_SECTION_ID),
        Section::Start(_) => result.push(START_SECTION_ID),
        Section::Element(_) => result.push(ELEMENT_SECTION_ID),
        Section::Code(_) => result.push(CODE_SECTION_ID),
        Section::Data(_) => result.push(DATA_SECTION_ID),
        Section::DataCount(_) => result.push(DATA_COUNT_SECTION_ID),
    }

    // Get section data
    let section_data = match section {
        Section::Custom(custom_section) => {
            // Custom section
            let mut custom_data = Vec::new();
            custom_data.extend_from_slice(&write_string(&custom_section.name));
            custom_data.extend_from_slice(&custom_section.data);
            custom_data
        }
        Section::Type(data) => data.clone(),
        Section::Import(data) => data.clone(),
        Section::Function(data) => data.clone(),
        Section::Table(data) => data.clone(),
        Section::Memory(data) => data.clone(),
        Section::Global(data) => data.clone(),
        Section::Export(data) => data.clone(),
        Section::Start(data) => data.clone(),
        Section::Element(data) => data.clone(),
        Section::Code(data) => data.clone(),
        Section::Data(data) => data.clone(),
        Section::DataCount(data) => data.clone(),
    };

    // Add section size
    result.extend_from_slice(&write_leb128_u32(section_data.len() as u32));

    // Add section data
    result.extend_from_slice(&section_data);

    Ok(result)
}
