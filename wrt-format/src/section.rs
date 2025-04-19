//! WebAssembly section handling.
//!
//! This module provides types and utilities for working with WebAssembly binary sections.

use wrt_error::Result;

/// Standard WebAssembly section IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SectionId {
    /// Custom section (0)
    Custom = 0,
    /// Type section (1)
    Type = 1,
    /// Import section (2)
    Import = 2,
    /// Function section (3)
    Function = 3,
    /// Table section (4)
    Table = 4,
    /// Memory section (5)
    Memory = 5,
    /// Global section (6)
    Global = 6,
    /// Export section (7)
    Export = 7,
    /// Start section (8)
    Start = 8,
    /// Element section (9)
    Element = 9,
    /// Code section (10)
    Code = 10,
    /// Data section (11)
    Data = 11,
}

impl SectionId {
    /// Convert a u8 to a SectionId
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Custom),
            1 => Some(Self::Type),
            2 => Some(Self::Import),
            3 => Some(Self::Function),
            4 => Some(Self::Table),
            5 => Some(Self::Memory),
            6 => Some(Self::Global),
            7 => Some(Self::Export),
            8 => Some(Self::Start),
            9 => Some(Self::Element),
            10 => Some(Self::Code),
            11 => Some(Self::Data),
            _ => None,
        }
    }
}

/// Represents a WebAssembly section
#[derive(Debug, Clone)]
pub struct Section {
    /// Section ID
    pub id: SectionId,
    /// Section data
    pub data: Vec<u8>,
}

/// Represents a WebAssembly custom section
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

    /// Convert a custom section to a standard section
    pub fn to_section(&self) -> Result<Section> {
        let mut section_data = Vec::new();

        // Add name length as LEB128
        let name_len = self.name.len();
        // Simple LEB128 encoding for now (only works for small names)
        section_data.push(name_len as u8);

        // Add name
        section_data.extend_from_slice(self.name.as_bytes());

        // Add data
        section_data.extend_from_slice(&self.data);

        Ok(Section {
            id: SectionId::Custom,
            data: section_data,
        })
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
