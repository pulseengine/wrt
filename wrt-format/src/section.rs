//! WebAssembly section handling.
//!
//! This module provides types and utilities for working with WebAssembly sections.

use crate::{String, Vec};

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

#[cfg(feature = "kani")]
mod verification {
    use super::*;
    use kani::*;

    #[kani::proof]
    fn verify_section_id_from_u8() {
        let id: u8 = any();

        if let Some(section_id) = SectionId::from_u8(id) {
            // Verify section_id is valid
            assert!(id <= 11);

            // Verify roundtrip
            assert_eq!(id, section_id as u8);
        } else {
            // Verify section_id is invalid
            assert!(id > 11);
        }
    }
}
