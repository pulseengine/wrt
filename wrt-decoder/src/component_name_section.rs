//! WebAssembly Component Model name section handling
//!
//! This module provides utilities for parsing and generating the WebAssembly Component Model name section.
//! The name section is a custom section that provides debug information for components.

use crate::{String, Vec};
use wrt_error::{kinds, Error, Result};
use wrt_format::{binary, component::Sort};

/// WebAssembly Component Model name section subsection types
pub const COMPONENT_NAME_COMPONENT: u8 = 0;
pub const COMPONENT_NAME_SORT: u8 = 1;
pub const COMPONENT_NAME_IMPORT: u8 = 2;
pub const COMPONENT_NAME_EXPORT: u8 = 3;
pub const COMPONENT_NAME_CANONICAL: u8 = 4;
pub const COMPONENT_NAME_TYPE: u8 = 5;

/// WebAssembly Component Model name section
#[derive(Debug, Clone, Default)]
pub struct ComponentNameSection {
    /// The component name, if present
    pub component_name: Option<String>,
    /// Sort-specific name maps
    pub sort_names: Vec<(Sort, Vec<(u32, String)>)>,
    /// Import names
    pub import_names: Vec<(u32, String)>,
    /// Export names
    pub export_names: Vec<(u32, String)>,
    /// Canonical function names
    pub canonical_names: Vec<(u32, String)>,
    /// Type names
    pub type_names: Vec<(u32, String)>,
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
            return Err(Error::new(kinds::ParseError(format!(
                "Component name subsection size {} exceeds data size",
                subsection_size
            ))));
        }

        let subsection_data = &data[subsection_start..subsection_end];

        match name_type {
            COMPONENT_NAME_COMPONENT => {
                // Component name
                let (name, _) = binary::read_string(subsection_data, 0)?;
                name_section.component_name = Some(name);
            }
            COMPONENT_NAME_SORT => {
                // Sort-specific names
                let (sort, bytes_read) = parse_sort(subsection_data, 0)?;
                let (names, _) = parse_name_map(&subsection_data[bytes_read..])?;
                name_section.sort_names.push((sort, names));
            }
            COMPONENT_NAME_IMPORT => {
                // Import names
                let (names, _) = parse_name_map(subsection_data)?;
                name_section.import_names = names;
            }
            COMPONENT_NAME_EXPORT => {
                // Export names
                let (names, _) = parse_name_map(subsection_data)?;
                name_section.export_names = names;
            }
            COMPONENT_NAME_CANONICAL => {
                // Canonical function names
                let (names, _) = parse_name_map(subsection_data)?;
                name_section.canonical_names = names;
            }
            COMPONENT_NAME_TYPE => {
                // Type names
                let (names, _) = parse_name_map(subsection_data)?;
                name_section.type_names = names;
            }
            _ => {
                // Unknown name subsection, ignore
            }
        }

        offset = subsection_end;
    }

    Ok(name_section)
}

/// Parse a sort from a byte array
fn parse_sort(bytes: &[u8], pos: usize) -> Result<(Sort, usize)> {
    if pos >= bytes.len() {
        return Err(Error::new(kinds::ParseError(
            "Truncated sort identifier".to_string(),
        )));
    }

    let sort_byte = bytes[pos];
    let mut offset = pos + 1;

    let sort = match sort_byte {
        binary::COMPONENT_SORT_CORE => {
            if offset >= bytes.len() {
                return Err(Error::new(kinds::ParseError(
                    "Truncated core sort identifier".to_string(),
                )));
            }

            let core_sort_byte = bytes[offset];
            offset += 1;

            match core_sort_byte {
                binary::COMPONENT_CORE_SORT_FUNC => {
                    Sort::Core(wrt_format::component::CoreSort::Function)
                }
                binary::COMPONENT_CORE_SORT_TABLE => {
                    Sort::Core(wrt_format::component::CoreSort::Table)
                }
                binary::COMPONENT_CORE_SORT_MEMORY => {
                    Sort::Core(wrt_format::component::CoreSort::Memory)
                }
                binary::COMPONENT_CORE_SORT_GLOBAL => {
                    Sort::Core(wrt_format::component::CoreSort::Global)
                }
                binary::COMPONENT_CORE_SORT_TYPE => {
                    Sort::Core(wrt_format::component::CoreSort::Type)
                }
                binary::COMPONENT_CORE_SORT_MODULE => {
                    Sort::Core(wrt_format::component::CoreSort::Module)
                }
                binary::COMPONENT_CORE_SORT_INSTANCE => {
                    Sort::Core(wrt_format::component::CoreSort::Instance)
                }
                _ => {
                    return Err(Error::new(kinds::ParseError(format!(
                        "Unknown core sort: {}",
                        core_sort_byte
                    ))));
                }
            }
        }
        binary::COMPONENT_SORT_FUNC => Sort::Function,
        binary::COMPONENT_SORT_VALUE => Sort::Value,
        binary::COMPONENT_SORT_TYPE => Sort::Type,
        binary::COMPONENT_SORT_COMPONENT => Sort::Component,
        binary::COMPONENT_SORT_INSTANCE => Sort::Instance,
        _ => {
            return Err(Error::new(kinds::ParseError(format!(
                "Unknown sort: {}",
                sort_byte
            ))));
        }
    };

    Ok((sort, offset - pos))
}

/// Parse a name map from a byte array
///
/// A name map is a vector of (index, name) pairs.
fn parse_name_map(bytes: &[u8]) -> Result<(Vec<(u32, String)>, usize)> {
    let mut offset = 0;

    // Read count
    let (count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    let mut result = Vec::with_capacity(count as usize);

    for _ in 0..count {
        // Read index
        let (index, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        // Read name
        let (name, bytes_read) = binary::read_string(bytes, offset)?;
        offset += bytes_read;

        result.push((index, name));
    }

    Ok((result, offset))
}

/// Generate a WebAssembly Component Model name section
pub fn generate_component_name_section(name_section: &ComponentNameSection) -> Result<Vec<u8>> {
    let mut result = Vec::new();

    // Component name
    if let Some(name) = &name_section.component_name {
        let mut subsection = Vec::new();
        subsection.extend_from_slice(&binary::write_string(name));

        result.push(COMPONENT_NAME_COMPONENT);
        result.extend_from_slice(&binary::write_leb128_u32(subsection.len() as u32));
        result.extend_from_slice(&subsection);
    }

    // Sort-specific names
    for (sort, names) in &name_section.sort_names {
        let mut subsection = Vec::new();

        // Write the sort
        subsection.extend_from_slice(&generate_sort(sort)?);

        // Write the name map
        subsection.extend_from_slice(&generate_name_map(names)?);

        result.push(COMPONENT_NAME_SORT);
        result.extend_from_slice(&binary::write_leb128_u32(subsection.len() as u32));
        result.extend_from_slice(&subsection);
    }

    // Import names
    if !name_section.import_names.is_empty() {
        let subsection = generate_name_map(&name_section.import_names)?;

        result.push(COMPONENT_NAME_IMPORT);
        result.extend_from_slice(&binary::write_leb128_u32(subsection.len() as u32));
        result.extend_from_slice(&subsection);
    }

    // Export names
    if !name_section.export_names.is_empty() {
        let subsection = generate_name_map(&name_section.export_names)?;

        result.push(COMPONENT_NAME_EXPORT);
        result.extend_from_slice(&binary::write_leb128_u32(subsection.len() as u32));
        result.extend_from_slice(&subsection);
    }

    // Canonical function names
    if !name_section.canonical_names.is_empty() {
        let subsection = generate_name_map(&name_section.canonical_names)?;

        result.push(COMPONENT_NAME_CANONICAL);
        result.extend_from_slice(&binary::write_leb128_u32(subsection.len() as u32));
        result.extend_from_slice(&subsection);
    }

    // Type names
    if !name_section.type_names.is_empty() {
        let subsection = generate_name_map(&name_section.type_names)?;

        result.push(COMPONENT_NAME_TYPE);
        result.extend_from_slice(&binary::write_leb128_u32(subsection.len() as u32));
        result.extend_from_slice(&subsection);
    }

    Ok(result)
}

/// Generate binary representation of a sort
fn generate_sort(sort: &Sort) -> Result<Vec<u8>> {
    let mut result = Vec::new();

    match sort {
        Sort::Core(core_sort) => {
            result.push(binary::COMPONENT_SORT_CORE);

            match core_sort {
                wrt_format::component::CoreSort::Function => {
                    result.push(binary::COMPONENT_CORE_SORT_FUNC);
                }
                wrt_format::component::CoreSort::Table => {
                    result.push(binary::COMPONENT_CORE_SORT_TABLE);
                }
                wrt_format::component::CoreSort::Memory => {
                    result.push(binary::COMPONENT_CORE_SORT_MEMORY);
                }
                wrt_format::component::CoreSort::Global => {
                    result.push(binary::COMPONENT_CORE_SORT_GLOBAL);
                }
                wrt_format::component::CoreSort::Type => {
                    result.push(binary::COMPONENT_CORE_SORT_TYPE);
                }
                wrt_format::component::CoreSort::Module => {
                    result.push(binary::COMPONENT_CORE_SORT_MODULE);
                }
                wrt_format::component::CoreSort::Instance => {
                    result.push(binary::COMPONENT_CORE_SORT_INSTANCE);
                }
            }
        }
        Sort::Function => {
            result.push(binary::COMPONENT_SORT_FUNC);
        }
        Sort::Value => {
            result.push(binary::COMPONENT_SORT_VALUE);
        }
        Sort::Type => {
            result.push(binary::COMPONENT_SORT_TYPE);
        }
        Sort::Component => {
            result.push(binary::COMPONENT_SORT_COMPONENT);
        }
        Sort::Instance => {
            result.push(binary::COMPONENT_SORT_INSTANCE);
        }
    }

    Ok(result)
}

/// Generate binary representation of a name map
fn generate_name_map(names: &[(u32, String)]) -> Result<Vec<u8>> {
    let mut result = Vec::new();

    // Write the count of names
    result.extend_from_slice(&binary::write_leb128_u32(names.len() as u32));

    // Write each name
    for (idx, name) in names {
        // Write the index
        result.extend_from_slice(&binary::write_leb128_u32(*idx));

        // Write the name
        result.extend_from_slice(&binary::write_string(name));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_component_name() {
        let original = ComponentNameSection {
            component_name: Some("test_component".to_string()),
            sort_names: Vec::new(),
            import_names: Vec::new(),
            export_names: Vec::new(),
            canonical_names: Vec::new(),
            type_names: Vec::new(),
        };

        let encoded = generate_component_name_section(&original).unwrap();
        let decoded = parse_component_name_section(&encoded).unwrap();

        assert_eq!(decoded.component_name, original.component_name);
    }

    #[test]
    fn test_roundtrip_sort_names() {
        let original = ComponentNameSection {
            component_name: None,
            sort_names: vec![
                (
                    Sort::Function,
                    vec![(0, "func1".to_string()), (1, "func2".to_string())],
                ),
                (
                    Sort::Instance,
                    vec![(0, "instance1".to_string()), (1, "instance2".to_string())],
                ),
            ],
            import_names: Vec::new(),
            export_names: Vec::new(),
            canonical_names: Vec::new(),
            type_names: Vec::new(),
        };

        let encoded = generate_component_name_section(&original).unwrap();
        let decoded = parse_component_name_section(&encoded).unwrap();

        assert_eq!(decoded.sort_names.len(), original.sort_names.len());

        for i in 0..original.sort_names.len() {
            let (sort1, names1) = &original.sort_names[i];
            let (sort2, names2) = &decoded.sort_names[i];

            // Compare sorts
            assert!(matches!(sort1, sort2));

            // Compare name maps
            assert_eq!(names1.len(), names2.len());
            for j in 0..names1.len() {
                assert_eq!(names1[j].0, names2[j].0);
                assert_eq!(names1[j].1, names2[j].1);
            }
        }
    }
}
