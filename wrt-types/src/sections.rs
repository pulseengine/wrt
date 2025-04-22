//! WebAssembly section definitions
//!
//! This module provides types and utilities for working with
//! WebAssembly sections, with built-in safety checks.

use crate::safe_memory::SafeSlice;
use crate::types::{GlobalType, MemoryType, TableType, ValueType};
use crate::verification::Checksum;

use wrt_error::{kinds, Error, Result};

#[cfg(feature = "std")]
use std::fmt;

#[cfg(not(feature = "std"))]
use core::fmt;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

/// WebAssembly section ID values
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
    /// Data count section (12)
    DataCount = 12,
}

impl SectionId {
    /// Convert from the WebAssembly binary format value
    pub fn from_binary(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Custom),
            1 => Ok(Self::Type),
            2 => Ok(Self::Import),
            3 => Ok(Self::Function),
            4 => Ok(Self::Table),
            5 => Ok(Self::Memory),
            6 => Ok(Self::Global),
            7 => Ok(Self::Export),
            8 => Ok(Self::Start),
            9 => Ok(Self::Element),
            10 => Ok(Self::Code),
            11 => Ok(Self::Data),
            12 => Ok(Self::DataCount),
            _ => Err(Error::new(kinds::ParseError(format!(
                "Invalid section id: {}",
                value
            )))),
        }
    }
}

/// WebAssembly section with safety verification
#[derive(Clone)]
pub struct Section<'a> {
    /// Section id
    pub id: SectionId,
    /// Section data
    pub data: SafeSlice<'a>,
    /// Size of the section in bytes
    pub size: u32,
    /// Offset of the section in the module
    pub offset: u32,
}

impl<'a> Section<'a> {
    /// Create a new section
    pub fn new(id: SectionId, data: SafeSlice<'a>, size: u32, offset: u32) -> Self {
        Self {
            id,
            data,
            size,
            offset,
        }
    }

    /// Verify the section's integrity
    pub fn verify(&self) -> Result<()> {
        // Verify data integrity
        self.data.verify_integrity()?;

        // Verify size matches data length
        if self.size as usize != self.data.len() {
            return Err(Error::new(kinds::ValidationError(format!(
                "Section size mismatch: expected {}, got {}",
                self.size,
                self.data.len()
            ))));
        }

        Ok(())
    }
}

impl fmt::Debug for Section<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Section")
            .field("id", &self.id)
            .field("size", &self.size)
            .field("offset", &self.offset)
            .finish()
    }
}

/// WebAssembly custom section
#[derive(Clone)]
pub struct CustomSection {
    /// Name of the custom section
    pub name: String,
    /// Custom section data
    pub data: Vec<u8>,
    /// Checksum for safety validation
    checksum: Checksum,
}

impl CustomSection {
    /// Create a new custom section
    pub fn new(name: String, data: Vec<u8>) -> Self {
        Self {
            name,
            data: data.clone(),
            checksum: Checksum::compute(&data),
        }
    }

    /// Verify the section's integrity
    pub fn verify(&self) -> Result<()> {
        let current = Checksum::compute(&self.data);
        if current == self.checksum {
            Ok(())
        } else {
            Err(Error::new(kinds::ValidationError(
                "Custom section integrity check failed: checksum mismatch".into(),
            )))
        }
    }
}

impl fmt::Debug for CustomSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CustomSection")
            .field("name", &self.name)
            .field("size", &self.data.len())
            .finish()
    }
}

/// WebAssembly import description
#[derive(Debug, Clone)]
pub enum ImportDesc {
    /// Function import
    Func(u32), // type index
    /// Table import
    Table(TableType),
    /// Memory import
    Memory(MemoryType),
    /// Global import
    Global(GlobalType),
}

/// WebAssembly import entry
#[derive(Debug, Clone)]
pub struct Import {
    /// Module name
    pub module: String,
    /// Name of the import
    pub name: String,
    /// Import description
    pub desc: ImportDesc,
}

/// WebAssembly export description
#[derive(Debug, Clone, Copy)]
pub enum ExportDesc {
    /// Function export
    Func(u32),
    /// Table export
    Table(u32),
    /// Memory export
    Memory(u32),
    /// Global export
    Global(u32),
}

/// WebAssembly export entry
#[derive(Debug, Clone)]
pub struct Export {
    /// Name of the export
    pub name: String,
    /// Export description
    pub desc: ExportDesc,
}

/// WebAssembly function body
#[derive(Clone)]
pub struct FunctionBody {
    /// Local variables
    pub locals: Vec<LocalEntry>,
    /// Function code
    pub code: Vec<u8>,
    /// Size of the function body
    pub size: u32,
    /// Checksum for safety validation
    checksum: Checksum,
}

impl FunctionBody {
    /// Create a new FunctionBody with safety validation
    pub fn new(locals: Vec<LocalEntry>, code: Vec<u8>, size: u32) -> Self {
        let mut hasher = Checksum::new();

        // Update with locals
        for local in &locals {
            hasher.update_slice(&local.count.to_le_bytes());
            hasher.update_slice(&[local.type_value as u8]);
        }

        // Update with code
        hasher.update_slice(&code);

        Self {
            locals,
            code,
            size,
            checksum: hasher,
        }
    }

    /// Verify the function body's integrity
    pub fn verify(&self) -> Result<()> {
        let mut hasher = Checksum::new();

        // Update with locals
        for local in &self.locals {
            hasher.update_slice(&local.count.to_le_bytes());
            hasher.update_slice(&[local.type_value as u8]);
        }

        // Update with code
        hasher.update_slice(&self.code);

        if hasher == self.checksum {
            Ok(())
        } else {
            Err(Error::new(kinds::ValidationError(
                "Function body integrity check failed: checksum mismatch".into(),
            )))
        }
    }
}

impl fmt::Debug for FunctionBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FunctionBody")
            .field("locals_count", &self.locals.len())
            .field("code_size", &self.code.len())
            .field("size", &self.size)
            .finish()
    }
}

/// WebAssembly local entry
#[derive(Debug, Clone, Copy)]
pub struct LocalEntry {
    /// Number of locals with this type
    pub count: u32,
    /// Type of the locals
    pub type_value: ValueType,
}

/// WebAssembly global variable
#[derive(Debug, Clone)]
pub struct Global {
    /// Type of the global
    pub ty: GlobalType,
    /// Initialization expression
    pub init: Vec<u8>,
}

/// WebAssembly element segment
#[derive(Debug, Clone)]
pub struct Element {
    /// Table index
    pub table: u32,
    /// Offset expression
    pub offset: Vec<u8>,
    /// Function indices
    pub functions: Vec<u32>,
}

/// WebAssembly data segment
#[derive(Debug, Clone)]
pub struct Data {
    /// Memory index
    pub memory: u32,
    /// Offset expression
    pub offset: Vec<u8>,
    /// Initial data
    pub init: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_id_conversion() {
        assert_eq!(SectionId::from_binary(0).unwrap(), SectionId::Custom);
        assert_eq!(SectionId::from_binary(1).unwrap(), SectionId::Type);
        assert_eq!(SectionId::from_binary(10).unwrap(), SectionId::Code);

        // Invalid section id should result in error
        assert!(SectionId::from_binary(100).is_err());
    }

    #[test]
    fn test_custom_section_integrity() {
        let data = vec![1, 2, 3, 4, 5];
        let section = CustomSection::new("test".to_string(), data);

        // Verification should pass
        assert!(section.verify().is_ok());

        // Modify section data to simulate corruption
        let mut corrupt = section.clone();
        corrupt.data[0] = 99;

        // Verification should fail
        assert!(corrupt.verify().is_err());
    }
}
