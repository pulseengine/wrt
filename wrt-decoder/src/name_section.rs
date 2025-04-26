//! WebAssembly name section handling
//!
//! This module provides utilities for parsing and generating the WebAssembly name section.
//! The name section is a custom section that provides debug information.

use crate::prelude::{format, String, Vec};
use wrt_error::{kinds, Error, Result};
use wrt_format::binary;

#[cfg(not(feature = "std"))]
use alloc::vec;

/// WebAssembly name section types
pub const NAME_MODULE: u8 = 0;
pub const NAME_FUNCTION: u8 = 1;
pub const NAME_LOCAL: u8 = 2;

/// WebAssembly name section
#[derive(Debug, Clone, Default)]
pub struct NameSection {
    /// The module name, if present
    pub module_name: Option<String>,
    /// Function names, indexed by function index
    pub function_names: Vec<(u32, String)>,
    /// Local names, indexed by function index and local index
    pub local_names: Vec<(u32, Vec<(u32, String)>)>,
}

/// Parse a WebAssembly name section
pub fn parse_name_section(data: &[u8]) -> Result<NameSection> {
    let mut name_section = NameSection::default();
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
                "Name subsection size {} exceeds data size",
                subsection_size
            ))));
        }

        let subsection_data = &data[subsection_start..subsection_end];

        match name_type {
            NAME_MODULE => {
                // Module name
                let (name, _) = binary::read_string(subsection_data, 0)?;
                name_section.module_name = Some(name);
            }
            NAME_FUNCTION => {
                // Function names
                let (function_names, _) = parse_name_map(subsection_data)?;
                name_section.function_names = function_names;
            }
            NAME_LOCAL => {
                // Local names
                let (local_names, _) = parse_indirect_name_map(subsection_data)?;
                name_section.local_names = local_names;
            }
            _ => {
                // Unknown name subsection, ignore
            }
        }

        offset = subsection_end;
    }

    Ok(name_section)
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

/// Parse an indirect name map from a byte array
///
/// An indirect name map is a vector of (index, name_map) pairs.
fn parse_indirect_name_map(bytes: &[u8]) -> Result<(Vec<(u32, Vec<(u32, String)>)>, usize)> {
    let mut offset = 0;

    // Read count
    let (count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    let mut result = Vec::with_capacity(count as usize);

    for _ in 0..count {
        // Read function index
        let (func_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        // Read local name map
        let (local_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        let mut locals = Vec::with_capacity(local_count as usize);

        for _ in 0..local_count {
            // Read local index
            let (local_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Read local name
            let (name, bytes_read) = binary::read_string(bytes, offset)?;
            offset += bytes_read;

            locals.push((local_idx, name));
        }

        result.push((func_idx, locals));
    }

    Ok((result, offset))
}

/// Generate a WebAssembly name section
pub fn generate_name_section(name_section: &NameSection) -> Result<Vec<u8>> {
    let mut result = Vec::new();

    // Add module name if present
    if let Some(ref module_name) = name_section.module_name {
        // Subsection type
        result.push(NAME_MODULE);

        // Generate name data
        let name_data = binary::write_string(module_name);

        // Subsection size
        result.extend_from_slice(&binary::write_leb128_u32(name_data.len() as u32));

        // Name data
        result.extend_from_slice(&name_data);
    }

    // Add function names if present
    if !name_section.function_names.is_empty() {
        // Subsection type
        result.push(NAME_FUNCTION);

        // Generate name map data
        let mut func_name_data = Vec::new();

        // Count
        func_name_data.extend_from_slice(&binary::write_leb128_u32(
            name_section.function_names.len() as u32,
        ));

        // Function names
        for &(index, ref name) in &name_section.function_names {
            func_name_data.extend_from_slice(&binary::write_leb128_u32(index));
            func_name_data.extend_from_slice(&binary::write_string(name));
        }

        // Subsection size
        result.extend_from_slice(&binary::write_leb128_u32(func_name_data.len() as u32));

        // Name map data
        result.extend_from_slice(&func_name_data);
    }

    // Add local names if present
    if !name_section.local_names.is_empty() {
        // Subsection type
        result.push(NAME_LOCAL);

        // Generate indirect name map data
        let mut local_name_data = Vec::new();

        // Count
        local_name_data.extend_from_slice(&binary::write_leb128_u32(
            name_section.local_names.len() as u32,
        ));

        // Function local names
        for &(func_idx, ref locals) in &name_section.local_names {
            local_name_data.extend_from_slice(&binary::write_leb128_u32(func_idx));
            local_name_data.extend_from_slice(&binary::write_leb128_u32(locals.len() as u32));

            for &(local_idx, ref name) in locals {
                local_name_data.extend_from_slice(&binary::write_leb128_u32(local_idx));
                local_name_data.extend_from_slice(&binary::write_string(name));
            }
        }

        // Subsection size
        result.extend_from_slice(&binary::write_leb128_u32(local_name_data.len() as u32));

        // Indirect name map data
        result.extend_from_slice(&local_name_data);
    }

    Ok(result)
}

/// Extract function names from a module's name section
pub fn extract_function_names(data: &[u8]) -> Result<Vec<(u32, String)>> {
    let name_section = parse_name_section(data)?;
    Ok(name_section.function_names)
}

/// Set function names in a module's name section
pub fn create_function_names_section(names: &[(u32, String)]) -> Result<Vec<u8>> {
    let name_section = NameSection {
        module_name: None,
        function_names: names.to_vec(),
        local_names: Vec::new(),
    };

    generate_name_section(&name_section)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_module_name() {
        let name = "test_module";

        let name_section = NameSection {
            module_name: Some(name.to_string()),
            function_names: Vec::new(),
            local_names: Vec::new(),
        };

        let encoded = generate_name_section(&name_section).unwrap();
        let decoded = parse_name_section(&encoded).unwrap();

        assert_eq!(decoded.module_name, Some(name.to_string()));
    }

    #[test]
    fn test_roundtrip_function_names() {
        let function_names = vec![
            (0, "main".to_string()),
            (1, "factorial".to_string()),
            (2, "fibonacci".to_string()),
        ];

        let name_section = NameSection {
            module_name: None,
            function_names: function_names.clone(),
            local_names: Vec::new(),
        };

        let encoded = generate_name_section(&name_section).unwrap();
        let decoded = parse_name_section(&encoded).unwrap();

        assert_eq!(decoded.function_names, function_names);
    }

    #[test]
    fn test_roundtrip_local_names() {
        let local_names = vec![
            (0, vec![(0, "arg1".to_string()), (1, "result".to_string())]),
            (1, vec![(0, "n".to_string()), (1, "temp".to_string())]),
        ];

        let name_section = NameSection {
            module_name: None,
            function_names: Vec::new(),
            local_names: local_names.clone(),
        };

        let encoded = generate_name_section(&name_section).unwrap();
        let decoded = parse_name_section(&encoded).unwrap();

        assert_eq!(decoded.local_names, local_names);
    }
}
