// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! WebAssembly Component Model name section handling
//!
//! This module provides utilities for parsing and generating the WebAssembly
//! Component Model name section. The name section is a custom section that
//! provides debug information for components.

use wrt_format::binary;
use wrt_types::ToString;

use crate::{prelude::*, Error, Result};

/// WebAssembly Component Model name section subsection types
pub const COMPONENT_NAME_COMPONENT: u8 = 0;
pub const COMPONENT_NAME_SORT: u8 = 1;
pub const COMPONENT_NAME_IMPORT: u8 = 2;
pub const COMPONENT_NAME_EXPORT: u8 = 3;
pub const COMPONENT_NAME_CANONICAL: u8 = 4;
pub const COMPONENT_NAME_TYPE: u8 = 5;

/// Component name section subsection identifiers
pub enum ComponentNameSubsectionId {
    Module = 0,
    Function = 1,
    CoreFunction = 2,
    CoreTable = 3,
    CoreMemory = 4,
    CoreGlobal = 5,
    CoreType = 6,
    Type = 7,
    Component = 8,
    Instance = 9,
    CoreInstance = 10,
}

/// Core sort identifier for subsections
pub enum CoreSortIdentifier {
    Function = 0,
    Table = 1,
    Memory = 2,
    Global = 3,
    Type = 4,
}

/// Sort identifier for subsections
#[derive(Debug, Clone, Copy)]
pub enum SortIdentifier {
    Module = 0,
    Function = 1,
    CoreFunction = 2,
    CoreTable = 3,
    CoreMemory = 4,
    CoreGlobal = 5,
    CoreType = 6,
    Type = 7,
    Component = 8,
    Instance = 9,
    CoreInstance = 10,
    Value = 11,
}

/// Entry in a name map
#[derive(Debug, Clone)]
pub struct NameMapEntry {
    pub index: u32,
    pub name: String,
}

/// Name map - maps indices to names
#[derive(Debug, Clone, Default)]
pub struct NameMap {
    pub entries: Vec<NameMapEntry>,
}

impl NameMap {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn parse(data: &[u8], offset: usize) -> Result<(Self, usize)> {
        // Read count of entries using wrt-format's read_leb128_u32
        let (count, count_len) = binary::read_leb128_u32(data, offset)?;

        let mut current_offset = offset + count_len;
        let mut entries = Vec::with_capacity(count as usize);

        for _ in 0..count {
            if current_offset >= data.len() {
                break;
            }

            // Parse index using wrt-format's read_leb128_u32
            let (index, index_len) = binary::read_leb128_u32(data, current_offset)?;
            current_offset += index_len;

            // Parse name
            if current_offset >= data.len() {
                return Err(Error::parse_error("Truncated name in name map".to_string()));
            }

            // Use wrt-format's read_string to parse the name
            let (name, name_len) = binary::read_string(data, current_offset)?;
            current_offset += name_len;

            entries.push(NameMapEntry { index, name });
        }

        Ok((Self { entries }, current_offset - offset))
    }
}

/// A component name section structure
///
/// This struct represents the name section of a component,
/// which can be used to store names for various component entities.
#[derive(Debug, Clone, Default)]
pub struct ComponentNameSection {
    /// Name of the component itself
    pub component_name: Option<String>,
    /// Map of names for various sorted items (functions, instances, etc.)
    pub sort_names: Vec<(SortIdentifier, NameMap)>,
    /// Map of import names
    pub import_names: NameMap,
    /// Map of export names
    pub export_names: NameMap,
    /// Map of canonical names
    pub canonical_names: NameMap,
    /// Map of type names
    pub type_names: NameMap,
}

/// Parse a WebAssembly Component Model name section
pub fn parse_component_name_section(data: &[u8]) -> Result<ComponentNameSection> {
    let mut name_section = ComponentNameSection::default();
    let mut offset = 0;

    while offset < data.len() {
        if offset + 1 > data.len() {
            break; // End of data
        }

        // Read name type
        let name_type = data[offset];
        offset += 1;

        // Read subsection size
        let (subsection_size, bytes_read) = binary::read_leb128_u32(data, offset)?;
        offset += bytes_read;

        let subsection_start = offset;
        let subsection_end = subsection_start + subsection_size as usize;

        if subsection_end > data.len() {
            return Err(Error::parse_error(format!(
                "Component name subsection size {} exceeds data size",
                subsection_size
            )));
        }

        let subsection_data = &data[subsection_start..subsection_end];

        match name_type {
            COMPONENT_NAME_COMPONENT => {
                // Component name
                if !subsection_data.is_empty() {
                    let (name, _) = binary::read_string(subsection_data, 0)?;
                    name_section.component_name = Some(name);
                }
            }
            COMPONENT_NAME_SORT => {
                // Sort names
                if !subsection_data.is_empty() {
                    let mut pos = 0;
                    while pos < subsection_data.len() {
                        let (sort, sort_size) = parse_sort(subsection_data, pos)?;
                        pos += sort_size;

                        let (name_map, name_map_size) = parse_name_map(subsection_data, pos)?;
                        pos += name_map_size;

                        name_section.sort_names.push((sort, name_map));
                    }
                }
            }
            COMPONENT_NAME_IMPORT => {
                // Import names
                if !subsection_data.is_empty() {
                    let (name_map, _) = parse_name_map(subsection_data, 0)?;
                    name_section.import_names = name_map;
                }
            }
            COMPONENT_NAME_EXPORT => {
                // Export names
                if !subsection_data.is_empty() {
                    let (name_map, _) = parse_name_map(subsection_data, 0)?;
                    name_section.export_names = name_map;
                }
            }
            COMPONENT_NAME_CANONICAL => {
                // Canonical names
                if !subsection_data.is_empty() {
                    let (name_map, _) = parse_name_map(subsection_data, 0)?;
                    name_section.canonical_names = name_map;
                }
            }
            COMPONENT_NAME_TYPE => {
                // Type names
                if !subsection_data.is_empty() {
                    let (name_map, _) = parse_name_map(subsection_data, 0)?;
                    name_section.type_names = name_map;
                }
            }
            _ => {
                // Skip unknown subsection
                offset = subsection_end;
                continue;
            }
        }

        offset = subsection_end;
    }

    Ok(name_section)
}

fn parse_sort(bytes: &[u8], pos: usize) -> Result<(SortIdentifier, usize)> {
    if pos >= bytes.len() {
        return Err(Error::parse_error("Unexpected end of input".to_string()));
    }

    let sort_byte = bytes[pos];
    let sort = match sort_byte {
        0 => SortIdentifier::Module,
        1 => SortIdentifier::Function,
        2 => SortIdentifier::CoreFunction,
        3 => SortIdentifier::CoreTable,
        4 => SortIdentifier::CoreMemory,
        5 => SortIdentifier::CoreGlobal,
        6 => SortIdentifier::CoreType,
        7 => SortIdentifier::Type,
        8 => SortIdentifier::Component,
        9 => SortIdentifier::Instance,
        10 => SortIdentifier::CoreInstance,
        11 => SortIdentifier::Value,
        _ => {
            return Err(Error::parse_error(format!("Invalid sort identifier: {}", sort_byte)));
        }
    };

    Ok((sort, 1))
}

fn parse_name_map(data: &[u8], pos: usize) -> Result<(NameMap, usize)> {
    NameMap::parse(data, pos)
}

pub fn generate_component_name_section(name_section: &ComponentNameSection) -> Result<Vec<u8>> {
    let mut data = Vec::new();

    // Component name
    if let Some(name) = &name_section.component_name {
        // Name type
        data.push(COMPONENT_NAME_COMPONENT);

        // Generate data for name
        let mut subsection_data = Vec::new();
        let name_bytes = binary::write_string(name);
        subsection_data.extend_from_slice(&name_bytes);

        // Add subsection size and data
        data.extend_from_slice(&binary::write_leb128_u32(subsection_data.len() as u32));
        data.extend_from_slice(&subsection_data);
    }

    // Sort names
    if !name_section.sort_names.is_empty() {
        // Name type
        data.push(COMPONENT_NAME_SORT);

        // Generate data for sorts
        let mut subsection_data = Vec::new();
        for (sort, name_map) in &name_section.sort_names {
            let sort_bytes = generate_sort(sort)?;
            subsection_data.extend_from_slice(&sort_bytes);

            let name_map_bytes = generate_name_map(name_map)?;
            subsection_data.extend_from_slice(&name_map_bytes);
        }

        // Add subsection size and data
        data.extend_from_slice(&binary::write_leb128_u32(subsection_data.len() as u32));
        data.extend_from_slice(&subsection_data);
    }

    // Import names
    if !name_section.import_names.entries.is_empty() {
        // Name type
        data.push(COMPONENT_NAME_IMPORT);

        // Generate data for import names
        let subsection_data = generate_name_map(&name_section.import_names)?;

        // Add subsection size and data
        data.extend_from_slice(&binary::write_leb128_u32(subsection_data.len() as u32));
        data.extend_from_slice(&subsection_data);
    }

    // Export names
    if !name_section.export_names.entries.is_empty() {
        // Name type
        data.push(COMPONENT_NAME_EXPORT);

        // Generate data for export names
        let subsection_data = generate_name_map(&name_section.export_names)?;

        // Add subsection size and data
        data.extend_from_slice(&binary::write_leb128_u32(subsection_data.len() as u32));
        data.extend_from_slice(&subsection_data);
    }

    // Canonical names
    if !name_section.canonical_names.entries.is_empty() {
        // Name type
        data.push(COMPONENT_NAME_CANONICAL);

        // Generate data for canonical names
        let subsection_data = generate_name_map(&name_section.canonical_names)?;

        // Add subsection size and data
        data.extend_from_slice(&binary::write_leb128_u32(subsection_data.len() as u32));
        data.extend_from_slice(&subsection_data);
    }

    // Type names
    if !name_section.type_names.entries.is_empty() {
        // Name type
        data.push(COMPONENT_NAME_TYPE);

        // Generate data for type names
        let subsection_data = generate_name_map(&name_section.type_names)?;

        // Add subsection size and data
        data.extend_from_slice(&binary::write_leb128_u32(subsection_data.len() as u32));
        data.extend_from_slice(&subsection_data);
    }

    Ok(data)
}

fn generate_sort(sort: &SortIdentifier) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    match sort {
        SortIdentifier::Module => data.push(0),
        SortIdentifier::Function => data.push(1),
        SortIdentifier::CoreFunction => data.push(2),
        SortIdentifier::CoreTable => data.push(3),
        SortIdentifier::CoreMemory => data.push(4),
        SortIdentifier::CoreGlobal => data.push(5),
        SortIdentifier::CoreType => data.push(6),
        SortIdentifier::Type => data.push(7),
        SortIdentifier::Component => data.push(8),
        SortIdentifier::Instance => data.push(9),
        SortIdentifier::CoreInstance => data.push(10),
        SortIdentifier::Value => data.push(11),
    }
    Ok(data)
}

fn generate_name_map(names: &NameMap) -> Result<Vec<u8>> {
    let mut data = Vec::new();

    // Number of entries
    data.extend_from_slice(&binary::write_leb128_u32(names.entries.len() as u32));

    // Each entry
    for entry in &names.entries {
        // Index
        data.extend_from_slice(&binary::write_leb128_u32(entry.index));

        // Name
        data.extend_from_slice(&binary::write_string(&entry.name));
    }

    Ok(data)
}

pub fn parse_error(message: &str) -> Error {
    Error::parse_error(message.to_string())
}

pub fn parse_error_with_context(message: &str, context: &str) -> Error {
    Error::parse_error(format!("{}: {}", message, context))
}

pub fn parse_error_with_position(message: &str, position: usize) -> Error {
    Error::parse_error(format!("{} at position {}", message, position))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_component_name() {
        let mut name_section = ComponentNameSection::default();
        name_section.component_name = Some("test_component".to_string());

        let bytes = generate_component_name_section(&name_section).unwrap();
        let parsed = parse_component_name_section(&bytes).unwrap();

        assert_eq!(parsed.component_name, Some("test_component".to_string()));
    }

    #[test]
    fn test_roundtrip_sort_names() {
        let mut name_section = ComponentNameSection::default();

        let mut name_map = NameMap::new();
        name_map.entries.push(NameMapEntry { index: 0, name: "func0".to_string() });
        name_map.entries.push(NameMapEntry { index: 1, name: "func1".to_string() });

        name_section.sort_names.push((SortIdentifier::Function, name_map));

        let bytes = generate_component_name_section(&name_section).unwrap();
        let parsed = parse_component_name_section(&bytes).unwrap();

        assert_eq!(parsed.sort_names.len(), 1);
        assert!(matches!(parsed.sort_names[0].0, SortIdentifier::Function));
        assert_eq!(parsed.sort_names[0].1.entries.len(), 2);
        assert_eq!(parsed.sort_names[0].1.entries[0].index, 0);
        assert_eq!(parsed.sort_names[0].1.entries[0].name, "func0");
        assert_eq!(parsed.sort_names[0].1.entries[1].index, 1);
        assert_eq!(parsed.sort_names[0].1.entries[1].name, "func1");
    }
}
