//! WebAssembly state serialization.
//!
//! This module provides utilities for serializing and deserializing WebAssembly
//! runtime state using custom sections.

use crate::compression::{rle_decode, rle_encode, CompressionType};
use crate::section::CustomSection;
use crate::version::{STATE_MAGIC, STATE_VERSION};
use wrt_error::kinds;
use wrt_error::{Error, Result};

/// Constants for state section names
pub const STATE_SECTION_PREFIX: &str = "wrt-state";

/// Types of state sections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateSection {
    /// Metadata section
    Meta = 0,
    /// Stack state section
    Stack = 1,
    /// Frame state section
    Frames = 2,
    /// Global variables section
    Globals = 3,
    /// Memory section
    Memory = 4,
}

impl StateSection {
    /// Get the section name for this state section type
    pub fn name(&self) -> String {
        match self {
            Self::Meta => format!("{}-meta", STATE_SECTION_PREFIX),
            Self::Stack => format!("{}-stack", STATE_SECTION_PREFIX),
            Self::Frames => format!("{}-frames", STATE_SECTION_PREFIX),
            Self::Globals => format!("{}-globals", STATE_SECTION_PREFIX),
            Self::Memory => format!("{}-memory", STATE_SECTION_PREFIX),
        }
    }

    /// Convert a section name to a StateSection
    pub fn from_name(name: &str) -> Option<Self> {
        if name == format!("{}-meta", STATE_SECTION_PREFIX) {
            Some(Self::Meta)
        } else if name == format!("{}-stack", STATE_SECTION_PREFIX) {
            Some(Self::Stack)
        } else if name == format!("{}-frames", STATE_SECTION_PREFIX) {
            Some(Self::Frames)
        } else if name == format!("{}-globals", STATE_SECTION_PREFIX) {
            Some(Self::Globals)
        } else if name == format!("{}-memory", STATE_SECTION_PREFIX) {
            Some(Self::Memory)
        } else {
            None
        }
    }

    /// Convert a u32 to a StateSection
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Meta),
            1 => Some(Self::Stack),
            2 => Some(Self::Frames),
            3 => Some(Self::Globals),
            4 => Some(Self::Memory),
            _ => None,
        }
    }
}

/// Header information for a state section
#[derive(Debug)]
pub struct StateHeader {
    /// State format version
    pub version: u32,
    /// Section type
    pub section_type: StateSection,
    /// Compression type used
    pub compression: CompressionType,
    /// Size of data after decompression
    pub decompressed_size: u32,
}

/// Create a state section with proper header
///
/// # Arguments
///
/// * `section_type` - Type of state section
/// * `data` - Raw data to include in the section
/// * `compression` - Compression type to use
///
/// # Returns
///
/// A CustomSection containing the state data with proper header
pub fn create_state_section(
    section_type: StateSection,
    data: &[u8],
    compression: CompressionType,
) -> Result<CustomSection> {
    // Create header structure
    let mut header = Vec::with_capacity(17);

    // Magic bytes
    header.extend_from_slice(STATE_MAGIC);

    // Version (little-endian)
    header.extend_from_slice(&STATE_VERSION.to_le_bytes());

    // Section type (u32, little-endian)
    header.extend_from_slice(&(section_type as u32).to_le_bytes());

    // Compression type
    header.push(compression as u8);

    // Apply compression if needed
    let compressed_data = match compression {
        CompressionType::None => data.to_vec(),
        CompressionType::RLE => rle_encode(data),
    };

    // Data length (decompressed size, little-endian)
    header.extend_from_slice(&(data.len() as u32).to_le_bytes());

    // Combine header and data
    let mut section_data = header;
    section_data.extend_from_slice(&compressed_data);

    Ok(CustomSection {
        name: section_type.name(),
        data: section_data,
    })
}

/// Extract data from a state section
///
/// # Arguments
///
/// * `section` - The CustomSection to extract data from
///
/// # Returns
///
/// Header information and decompressed data
pub fn extract_state_section(section: &CustomSection) -> Result<(StateHeader, Vec<u8>)> {
    // Ensure section has a valid name
    let section_type = StateSection::from_name(&section.name)
        .ok_or_else(|| Error::new(kinds::ParseError("Invalid state section name".to_string())))?;

    // Check header size
    if section.data.len() < 17 {
        return Err(Error::new(kinds::ParseError(
            "State section header too small".to_string(),
        )));
    }

    // Extract magic bytes
    let magic = &section.data[0..4];
    if magic != STATE_MAGIC {
        return Err(Error::new(kinds::ParseError(
            "Invalid state section magic bytes".to_string(),
        )));
    }

    // Extract version
    let version = u32::from_le_bytes([
        section.data[4],
        section.data[5],
        section.data[6],
        section.data[7],
    ]);

    // Extract section type
    let section_type_id = u32::from_le_bytes([
        section.data[8],
        section.data[9],
        section.data[10],
        section.data[11],
    ]);

    // Validate section type matches name
    let parsed_section_type = StateSection::from_u32(section_type_id)
        .ok_or_else(|| Error::new(kinds::ParseError("Invalid section type ID".to_string())))?;

    if parsed_section_type != section_type {
        return Err(Error::new(kinds::ParseError(
            "Section type mismatch".to_string(),
        )));
    }

    // Extract compression type
    let compression = match section.data[12] {
        0 => CompressionType::None,
        1 => CompressionType::RLE,
        _ => {
            return Err(Error::new(kinds::ParseError(
                "Unknown compression type".to_string(),
            )))
        }
    };

    // Extract decompressed size
    let decompressed_size = u32::from_le_bytes([
        section.data[13],
        section.data[14],
        section.data[15],
        section.data[16],
    ]);

    // Extract and decompress data
    let compressed_data = &section.data[17..];
    let data = match compression {
        CompressionType::None => compressed_data.to_vec(),
        CompressionType::RLE => rle_decode(compressed_data)?,
    };

    // Verify decompressed size
    if data.len() != decompressed_size as usize {
        return Err(Error::new(kinds::ParseError(
            "Decompressed size mismatch".to_string(),
        )));
    }

    Ok((
        StateHeader {
            version,
            section_type,
            compression,
            decompressed_size,
        },
        data,
    ))
}

/// Check if a module contains state sections
///
/// # Arguments
///
/// * `custom_sections` - Vector of custom sections to check
///
/// # Returns
///
/// `true` if the module contains at least one state section
pub fn has_state_sections(custom_sections: &[CustomSection]) -> bool {
    custom_sections
        .iter()
        .any(|section| section.name.starts_with(STATE_SECTION_PREFIX))
}

#[cfg(feature = "kani")]
mod verification {
    use super::*;
    use kani::*;

    #[kani::proof]
    fn verify_state_section_roundtrip() {
        // Create small test data - Kani works best with small bounds
        let mut test_data = Vec::with_capacity(5);
        for _ in 0..5 {
            test_data.push(any::<u8>());
        }

        // Create state section
        let section =
            create_state_section(StateSection::Stack, &test_data, CompressionType::None).unwrap();

        // Extract section data
        let (header, data) = extract_state_section(&section).unwrap();

        // Verify properties
        assert_eq!(header.section_type, StateSection::Stack);
        assert_eq!(header.compression, CompressionType::None);
        assert_eq!(header.decompressed_size as usize, test_data.len());
        assert_eq!(data, test_data);
    }

    #[kani::proof]
    fn verify_section_type_roundtrip() {
        let section_type: u32 = any();

        if let Some(state_section) = StateSection::from_u32(section_type) {
            // Valid section type
            assert!(section_type <= 4);

            // Verify roundtrip
            assert_eq!(section_type, state_section as u32);

            // Verify name conversions
            let name = state_section.name();
            let from_name = StateSection::from_name(&name);
            assert!(from_name.is_some());
            assert_eq!(from_name.unwrap(), state_section);
        } else {
            // Invalid section type
            assert!(section_type > 4);
        }
    }
}
