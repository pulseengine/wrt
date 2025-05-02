//! WebAssembly Core Module Parser
//!
//! Functions for parsing WebAssembly core modules from binary format.

use crate::Result;
use wrt_error::{codes, Error, ErrorCategory};
use wrt_format::{binary::BinaryFormat, Module};


#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{string::String, vec::Vec};

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

// Custom error codes
const ERROR_INVALID_LENGTH: u16 = codes::PARSE_ERROR;
const ERROR_INVALID_MAGIC: u16 = codes::PARSE_ERROR;
const ERROR_INVALID_VERSION: u16 = codes::PARSE_ERROR;
const ERROR_INVALID_SECTION: u16 = codes::PARSE_ERROR;
const ERROR_INVALID_UTF8: u16 = codes::PARSE_ERROR;

/// Parse a WebAssembly binary module
///
/// This function takes a WebAssembly binary and parses it into a structured
/// Module representation.
///
/// # Arguments
///
/// * `data` - The WebAssembly binary data
///
/// # Returns
///
/// * `Result<Module>` - The parsed module or an error
///
/// # Errors
///
/// Returns an error if the binary cannot be parsed.
pub fn parse_module(data: &[u8]) -> Result<Module> {
    // First check for the WebAssembly binary magic number and version
    if data.len() < 8 {
        return Err(Error::new(
            ErrorCategory::Parse,
            ERROR_INVALID_LENGTH,
            "WebAssembly binary too short",
        ));
    }

    if data[0..4] != [0x00, 0x61, 0x73, 0x6D] {
        return Err(Error::new(
            ErrorCategory::Parse,
            ERROR_INVALID_MAGIC,
            "Invalid WebAssembly magic number",
        ));
    }

    if data[4..8] != [0x01, 0x00, 0x00, 0x00] {
        return Err(Error::new(
            ErrorCategory::Parse,
            codes::INVALID_VERSION,
            "Unsupported WebAssembly version",
        ));
    }

    // Parse the binary contents
    parse_binary(&data[8..])
}

/// Parse the WebAssembly binary content after the magic number and version
///
/// # Arguments
///
/// * `data` - The WebAssembly binary data after the magic number and version
///
/// # Returns
///
/// * `Result<Module>` - The parsed module or an error
///
/// # Errors
///
/// Returns an error if the binary cannot be parsed.
pub fn parse_binary(data: &[u8]) -> Result<Module> {
    let mut module = Module::new();
    module.binary = Some(data.to_vec());

    let mut offset = 0;

    // Parse each section
    while offset < data.len() {
        let section_id = data[offset];
        offset += 1;

        // Parse the section size
        let (section_size, bytes_read) = BinaryFormat::decode_leb_u32(&data[offset..])?;
        offset += bytes_read;

        // Parse the section content based on its ID
        match section_id {
            CUSTOM_SECTION_ID => {
                // Custom section handling
                let name_len = data[offset] as usize;
                offset += 1;

                if offset + name_len > data.len() {
                    return Err(Error::new(
                        ErrorCategory::Parse,
                        ERROR_INVALID_SECTION,
                        "Custom section name truncated",
                    ));
                }

                let name =
                    core::str::from_utf8(&data[offset..offset + name_len]).map_err(|_| {
                        Error::new(
                            ErrorCategory::Parse,
                            ERROR_INVALID_UTF8,
                            "Custom section name is not valid UTF-8",
                        )
                    })?;

                offset += name_len;

                // Extract the section data
                let section_data = &data[offset..offset + section_size as usize - (name_len + 1)];

                // Create and add the custom section
                let custom_section = wrt_format::section::CustomSection {
                    name: name.to_string(),
                    data: section_data.to_vec(),
                };

                module.add_custom_section(custom_section);

                offset += section_size as usize - (name_len + 1);
            }
            // Other section types would be handled here
            _ => {
                // Skip unhandled sections
                offset += section_size as usize;
            }
        }
    }

    Ok(module)
}
