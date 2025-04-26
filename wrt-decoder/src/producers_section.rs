//! WebAssembly producers section handling
//!
//! This module provides utilities for parsing and generating the WebAssembly producers section.
//! The producers section is a custom section that provides information about the tools
//! that produced the WebAssembly module.

use crate::prelude::{String, Vec};
use wrt_error::Result;
use wrt_format::binary;

/// Field name for the language field in producers section
pub const FIELD_LANGUAGE: &str = "language";
/// Field name for the processed-by field in producers section
pub const FIELD_PROCESSED_BY: &str = "processed-by";
/// Field name for the SDK field in producers section
pub const FIELD_SDK: &str = "sdk";

/// Represents a tool with its name and version
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProducerInfo {
    /// Name of the tool/language
    pub name: String,
    /// Version of the tool/language
    pub version: String,
}

/// WebAssembly producers section
#[derive(Debug, Clone, Default)]
pub struct ProducersSection {
    /// The source languages used, if present
    pub languages: Vec<ProducerInfo>,
    /// The tools that processed the module, if present
    pub processed_by: Vec<ProducerInfo>,
    /// The SDKs used, if present
    pub sdks: Vec<ProducerInfo>,
}

impl ProducersSection {
    /// Creates a new empty producers section
    pub fn new() -> Self {
        Self {
            languages: Vec::new(),
            processed_by: Vec::new(),
            sdks: Vec::new(),
        }
    }

    /// Adds a language entry to the producers section
    pub fn add_language(&mut self, name: String, version: String) {
        self.languages.push(ProducerInfo { name, version });
    }

    /// Adds a processed-by entry to the producers section
    pub fn add_processed_by(&mut self, name: String, version: String) {
        self.processed_by.push(ProducerInfo { name, version });
    }

    /// Adds an SDK entry to the producers section
    pub fn add_sdk(&mut self, name: String, version: String) {
        self.sdks.push(ProducerInfo { name, version });
    }

    /// Encodes this producers section to binary format
    pub fn to_binary(&self) -> Vec<u8> {
        let mut data = Vec::new();

        // Calculate the number of fields we'll include
        let mut field_count = 0;
        if !self.languages.is_empty() {
            field_count += 1;
        }
        if !self.processed_by.is_empty() {
            field_count += 1;
        }
        if !self.sdks.is_empty() {
            field_count += 1;
        }

        // Write field count
        let field_count_bytes = binary::write_leb128_u32(field_count);
        data.extend_from_slice(&field_count_bytes);

        // Write language field if it has entries
        if !self.languages.is_empty() {
            self.write_field_to_data(FIELD_LANGUAGE, &self.languages, &mut data);
        }

        // Write processed-by field if it has entries
        if !self.processed_by.is_empty() {
            self.write_field_to_data(FIELD_PROCESSED_BY, &self.processed_by, &mut data);
        }

        // Write sdk field if it has entries
        if !self.sdks.is_empty() {
            self.write_field_to_data(FIELD_SDK, &self.sdks, &mut data);
        }

        data
    }

    // Helper function to write a field to the data buffer
    fn write_field_to_data(&self, field_name: &str, values: &[ProducerInfo], data: &mut Vec<u8>) {
        // Write field name
        let field_name_bytes = binary::write_string(field_name);
        data.extend_from_slice(&field_name_bytes);

        // Write number of producer values
        let value_count_bytes = binary::write_leb128_u32(values.len() as u32);
        data.extend_from_slice(&value_count_bytes);

        // Write each producer name and version
        for producer in values {
            let name_bytes = binary::write_string(&producer.name);
            data.extend_from_slice(&name_bytes);

            let version_bytes = binary::write_string(&producer.version);
            data.extend_from_slice(&version_bytes);
        }
    }
}

/// Parse a WebAssembly producers section
pub fn parse_producers_section(data: &[u8]) -> Result<ProducersSection> {
    let mut producers = ProducersSection::new();
    let mut offset = 0;

    // Read field count
    let (field_count, bytes_read) = binary::read_leb128_u32(data, offset)?;
    offset += bytes_read;

    // Read each field
    for _ in 0..field_count {
        // Read field name
        let (field_name, bytes_read) = binary::read_string(data, offset)?;
        offset += bytes_read;

        // Read value count
        let (value_count, bytes_read) = binary::read_leb128_u32(data, offset)?;
        offset += bytes_read;

        // Read each name-value pair
        for _ in 0..value_count {
            // Read name
            let (name, bytes_read) = binary::read_string(data, offset)?;
            offset += bytes_read;

            // Read version
            let (version, bytes_read) = binary::read_string(data, offset)?;
            offset += bytes_read;

            // Add to appropriate field
            match field_name.as_str() {
                FIELD_LANGUAGE => {
                    producers.add_language(name, version);
                }
                FIELD_PROCESSED_BY => {
                    producers.add_processed_by(name, version);
                }
                FIELD_SDK => {
                    producers.add_sdk(name, version);
                }
                _ => {
                    // Unknown field name, we could warn here but the spec says to ignore
                    // unknown field names, so we'll just add it as a processed-by entry
                    producers.add_processed_by(name, version);
                }
            }
        }
    }

    Ok(producers)
}

/// Extract producers information from a module
pub fn extract_producers_section(
    module: &crate::module::Module,
) -> Result<Option<ProducersSection>> {
    // Find the producers custom section
    let producers_section = module
        .custom_sections
        .iter()
        .find(|section| section.name == "producers");

    if let Some(section) = producers_section {
        // Parse the producers section
        let producers = parse_producers_section(&section.data)?;
        Ok(Some(producers))
    } else {
        // No producers section found
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::{ToString, Vec};

    #[cfg(not(feature = "std"))]
    use alloc::vec;

    #[test]
    fn test_parse_producers_section() {
        // Create a simple producers section in binary format
        let mut section_data = Vec::new();

        // Field count: 2
        section_data.extend_from_slice(&binary::write_leb128_u32(2));

        // Field 1: "language"
        section_data.extend_from_slice(&binary::write_string(FIELD_LANGUAGE));
        // Value count: 1
        section_data.extend_from_slice(&binary::write_leb128_u32(1));
        // Name-value pair: "Rust" "1.50.0"
        section_data.extend_from_slice(&binary::write_string("Rust"));
        section_data.extend_from_slice(&binary::write_string("1.50.0"));

        // Field 2: "processed-by"
        section_data.extend_from_slice(&binary::write_string(FIELD_PROCESSED_BY));
        // Value count: 2
        section_data.extend_from_slice(&binary::write_leb128_u32(2));
        // Name-value pair 1: "rustc" "1.50.0"
        section_data.extend_from_slice(&binary::write_string("rustc"));
        section_data.extend_from_slice(&binary::write_string("1.50.0"));
        // Name-value pair 2: "wasm-bindgen" "0.2.70"
        section_data.extend_from_slice(&binary::write_string("wasm-bindgen"));
        section_data.extend_from_slice(&binary::write_string("0.2.70"));

        // Parse the producers section
        let producers = parse_producers_section(&section_data).unwrap();

        // Check results
        assert_eq!(producers.languages.len(), 1);
        assert_eq!(producers.languages[0].name, "Rust");
        assert_eq!(producers.languages[0].version, "1.50.0");

        assert_eq!(producers.processed_by.len(), 2);
        assert_eq!(producers.processed_by[0].name, "rustc");
        assert_eq!(producers.processed_by[0].version, "1.50.0");
        assert_eq!(producers.processed_by[1].name, "wasm-bindgen");
        assert_eq!(producers.processed_by[1].version, "0.2.70");

        assert_eq!(producers.sdks.len(), 0);
    }

    #[test]
    fn test_round_trip() {
        let mut producers = ProducersSection::new();
        producers.add_language("Rust".to_string(), "1.50.0".to_string());
        producers.add_processed_by("rustc".to_string(), "1.50.0".to_string());
        producers.add_processed_by("wasm-bindgen".to_string(), "0.2.70".to_string());
        producers.add_sdk("Emscripten".to_string(), "2.0.0".to_string());

        // Encode to binary
        let binary_data = producers.to_binary();

        // Parse back from binary
        let parsed = parse_producers_section(&binary_data).unwrap();

        // Check that we get the same data back
        assert_eq!(parsed.languages.len(), 1);
        assert_eq!(parsed.languages[0].name, "Rust");
        assert_eq!(parsed.languages[0].version, "1.50.0");

        assert_eq!(parsed.processed_by.len(), 2);
        assert_eq!(parsed.processed_by[0].name, "rustc");
        assert_eq!(parsed.processed_by[0].version, "1.50.0");
        assert_eq!(parsed.processed_by[1].name, "wasm-bindgen");
        assert_eq!(parsed.processed_by[1].version, "0.2.70");

        assert_eq!(parsed.sdks.len(), 1);
        assert_eq!(parsed.sdks[0].name, "Emscripten");
        assert_eq!(parsed.sdks[0].version, "2.0.0");
    }
}
