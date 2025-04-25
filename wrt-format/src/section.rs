//! WebAssembly section handling.
//!
//! This module provides types and utilities for working with WebAssembly sections.

use crate::{String, Vec};
use wrt_error::{Error, Result};

/// WebAssembly section ID constants
pub const CUSTOM_ID: u8 = 0;
pub const TYPE_ID: u8 = 1;
pub const IMPORT_ID: u8 = 2;
pub const FUNCTION_ID: u8 = 3;
pub const TABLE_ID: u8 = 4;
pub const MEMORY_ID: u8 = 5;
pub const GLOBAL_ID: u8 = 6;
pub const EXPORT_ID: u8 = 7;
pub const START_ID: u8 = 8;
pub const ELEMENT_ID: u8 = 9;
pub const CODE_ID: u8 = 10;
pub const DATA_ID: u8 = 11;
pub const DATA_COUNT_ID: u8 = 12;

/// WebAssembly section identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionId {
    /// Custom section
    Custom = 0,
    /// Type section
    Type = 1,
    /// Import section
    Import = 2,
    /// Function section
    Function = 3,
    /// Table section
    Table = 4,
    /// Memory section
    Memory = 5,
    /// Global section
    Global = 6,
    /// Export section
    Export = 7,
    /// Start section
    Start = 8,
    /// Element section
    Element = 9,
    /// Code section
    Code = 10,
    /// Data section
    Data = 11,
    /// Data count section
    DataCount = 12,
}

impl SectionId {
    /// Convert a u8 to a SectionId
    pub fn from_u8(id: u8) -> Option<Self> {
        match id {
            0 => Some(SectionId::Custom),
            1 => Some(SectionId::Type),
            2 => Some(SectionId::Import),
            3 => Some(SectionId::Function),
            4 => Some(SectionId::Table),
            5 => Some(SectionId::Memory),
            6 => Some(SectionId::Global),
            7 => Some(SectionId::Export),
            8 => Some(SectionId::Start),
            9 => Some(SectionId::Element),
            10 => Some(SectionId::Code),
            11 => Some(SectionId::Data),
            12 => Some(SectionId::DataCount),
            _ => None,
        }
    }
}

/// WebAssembly section
#[derive(Debug, Clone)]
pub enum Section {
    /// Custom section
    Custom(CustomSection),
    /// Type section
    Type(Vec<u8>),
    /// Import section
    Import(Vec<u8>),
    /// Function section
    Function(Vec<u8>),
    /// Table section
    Table(Vec<u8>),
    /// Memory section
    Memory(Vec<u8>),
    /// Global section
    Global(Vec<u8>),
    /// Export section
    Export(Vec<u8>),
    /// Start section
    Start(Vec<u8>),
    /// Element section
    Element(Vec<u8>),
    /// Code section
    Code(Vec<u8>),
    /// Data section
    Data(Vec<u8>),
    /// Data count section
    DataCount(Vec<u8>),
}

/// WebAssembly custom section
#[derive(Debug, Clone)]
pub struct CustomSection {
    /// Section name
    pub name: String,
    /// Section data
    pub data: Vec<u8>,
}

impl CustomSection {
    /// Create a new custom section
    pub fn new(name: String, data: Vec<u8>) -> Self {
        Self { name, data }
    }

    /// Serialize the custom section to binary
    pub fn to_binary(&self) -> Vec<u8> {
        let mut section_data = Vec::new();

        // Add name as encoded string (name length + name bytes)
        let name_len = self.name.len() as u32;
        let name_len_bytes = crate::binary::write_leb128_u32(name_len);
        section_data.extend_from_slice(&name_len_bytes);
        section_data.extend_from_slice(self.name.as_bytes());

        // Add data
        section_data.extend_from_slice(&self.data);

        section_data
    }
}

/// Component section types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentSectionType {
    /// Custom section - for metadata, debug info, etc.
    Custom = 0,
    /// Core module section - contains WebAssembly modules
    CoreModule = 1,
    /// Core instance section - contains core WebAssembly instances
    CoreInstance = 2,
    /// Core type section - contains core WebAssembly types
    CoreType = 3,
    /// Component section - contains nested components
    Component = 4,
    /// Component instance section - contains component instances
    Instance = 5,
    /// Alias section - contains aliases
    Alias = 6,
    /// Type section - contains component types
    Type = 7,
    /// Canonical section - contains canonical function conversions
    Canon = 8,
    /// Start section - contains the component start function
    Start = 9,
    /// Import section - contains imports
    Import = 10,
    /// Export section - contains exports
    Export = 11,
    /// Value section - contains component values
    Value = 12,
}

impl ComponentSectionType {
    /// Get the section ID for this section type
    pub fn id(&self) -> u8 {
        *self as u8
    }

    /// Parse a section ID into a ComponentSectionType
    pub fn from_u8(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::Custom),
            1 => Some(Self::CoreModule),
            2 => Some(Self::CoreInstance),
            3 => Some(Self::CoreType),
            4 => Some(Self::Component),
            5 => Some(Self::Instance),
            6 => Some(Self::Alias),
            7 => Some(Self::Type),
            8 => Some(Self::Canon),
            9 => Some(Self::Start),
            10 => Some(Self::Import),
            11 => Some(Self::Export),
            12 => Some(Self::Value),
            _ => None,
        }
    }
}

/// Component section header containing section ID and size
#[derive(Debug, Clone)]
pub struct ComponentSectionHeader {
    /// Section type
    pub section_type: ComponentSectionType,
    /// Section size in bytes (excluding the header)
    pub size: u32,
}

/// Parse a component section header from a byte array
pub fn parse_component_section_header(
    bytes: &[u8],
    pos: usize,
) -> Result<(ComponentSectionHeader, usize)> {
    let (id, size, new_pos) = crate::binary::read_section_header(bytes, pos)?;

    let section_type = ComponentSectionType::from_u8(id).ok_or_else(|| {
        Error::new(wrt_error::kinds::ParseError(format!(
            "Invalid component section ID: {}",
            id
        )))
    })?;

    Ok((ComponentSectionHeader { section_type, size }, new_pos))
}

/// Write a component section header to a byte array
pub fn write_component_section_header(
    section_type: ComponentSectionType,
    content_size: u32,
) -> Vec<u8> {
    crate::binary::write_section_header(section_type.id(), content_size)
}

/// Format a component section into a byte array
pub fn format_component_section<F>(section_type: ComponentSectionType, content_fn: F) -> Vec<u8>
where
    F: FnOnce() -> Vec<u8>,
{
    let content = content_fn();
    let header = write_component_section_header(section_type, content.len() as u32);

    let mut result = Vec::with_capacity(header.len() + content.len());
    result.extend_from_slice(&header);
    result.extend_from_slice(&content);

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary;

    #[test]
    fn test_section_id_conversion() {
        // Test conversion from u8 to SectionId
        assert_eq!(SectionId::from_u8(0), Some(SectionId::Custom));
        assert_eq!(SectionId::from_u8(1), Some(SectionId::Type));
        assert_eq!(SectionId::from_u8(12), Some(SectionId::DataCount));
        assert_eq!(SectionId::from_u8(13), None);
        assert_eq!(SectionId::from_u8(255), None);

        // Test conversion from SectionId to u8
        assert_eq!(SectionId::Custom as u8, CUSTOM_ID);
        assert_eq!(SectionId::Type as u8, TYPE_ID);
        assert_eq!(SectionId::DataCount as u8, DATA_COUNT_ID);
    }

    #[test]
    fn test_component_section_type_conversion() {
        // Test conversion from u8 to ComponentSectionType
        assert_eq!(
            ComponentSectionType::from_u8(0),
            Some(ComponentSectionType::Custom)
        );
        assert_eq!(
            ComponentSectionType::from_u8(1),
            Some(ComponentSectionType::CoreModule)
        );
        assert_eq!(
            ComponentSectionType::from_u8(12),
            Some(ComponentSectionType::Value)
        );
        assert_eq!(ComponentSectionType::from_u8(13), None);
        assert_eq!(ComponentSectionType::from_u8(255), None);

        // Test conversion from ComponentSectionType to u8
        assert_eq!(ComponentSectionType::Custom.id(), 0);
        assert_eq!(ComponentSectionType::CoreModule.id(), 1);
        assert_eq!(ComponentSectionType::Value.id(), 12);
    }

    #[test]
    fn test_custom_section_serialization() {
        // Create a custom section
        let section = CustomSection::new("test-section".to_string(), vec![1, 2, 3, 4]);

        // Serialize to binary
        let binary_data = section.to_binary();

        // Verify structure:
        // First bytes should be the name length as leb128 encoded u32 (name.len() = 12)
        let (decoded_len, name_pos) = binary::read_leb128_u32(&binary_data, 0).unwrap();
        assert_eq!(decoded_len, 12);

        // Next comes the name itself
        let name_slice = &binary_data[name_pos..name_pos + 12];
        assert_eq!(name_slice, "test-section".as_bytes());

        // Finally, the data
        let data_slice = &binary_data[name_pos + 12..];
        assert_eq!(data_slice, &[1, 2, 3, 4]);
    }

    #[test]
    fn test_component_section_header() {
        // Create binary data for a component section header
        let section_type = ComponentSectionType::CoreModule;
        let content_size = 42;

        let header_bytes = write_component_section_header(section_type, content_size);

        // Parse the header
        let (header, pos) = parse_component_section_header(&header_bytes, 0).unwrap();

        // Verify the parsed header
        assert_eq!(header.section_type, section_type);
        assert_eq!(header.size, content_size);
        assert_eq!(pos, header_bytes.len());
    }

    #[test]
    fn test_format_component_section() {
        // Create a section
        let section_type = ComponentSectionType::CoreInstance;
        let section_content = vec![1, 2, 3, 4, 5];

        // Format the section
        let formatted = format_component_section(section_type, || section_content.clone());

        // Parse the formatted section
        let (header, content_pos) = parse_component_section_header(&formatted, 0).unwrap();

        // Verify the header
        assert_eq!(header.section_type, section_type);
        assert_eq!(header.size, section_content.len() as u32);

        // Verify the content
        let content = &formatted[content_pos..];
        assert_eq!(content, &section_content);
    }

    #[test]
    fn test_invalid_component_section_id() {
        // Create binary data with an invalid section ID
        let invalid_id = 255;
        let size = 10;
        let binary_data = binary::write_section_header(invalid_id, size);

        // Try to parse the header
        let result = parse_component_section_header(&binary_data, 0);

        // Should return an error
        assert!(result.is_err());
    }
}
