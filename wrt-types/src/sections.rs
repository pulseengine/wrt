//! WebAssembly section definitions
//!
//! This module defines the WebAssembly module sections with
//! safety verification built in.

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use crate::safe_memory::SafeSlice;
use wrt_error::{kinds, Error, Result};

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
            #[cfg(feature = "std")]
            _ => Err(Error::new(kinds::ParseError(format!(
                "Invalid section id: {}",
                value
            )))),
            #[cfg(all(not(feature = "std"), feature = "alloc"))]
            _ => Err(Error::new(kinds::ParseError(format!(
                "Invalid section id: {}",
                value
            )))),
            #[cfg(not(any(feature = "std", feature = "alloc")))]
            _ => Err(Error::new(kinds::ParseError("Invalid section id".into()))),
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
            #[cfg(feature = "std")]
            return Err(Error::new(kinds::ValidationError(format!(
                "Section size mismatch: expected {}, got {}",
                self.size,
                self.data.len()
            ))));

            #[cfg(all(not(feature = "std"), feature = "alloc"))]
            return Err(Error::new(kinds::ValidationError(format!(
                "Section size mismatch: expected {}, got {}",
                self.size,
                self.data.len()
            ))));
        }
        Ok(())
    }
}
