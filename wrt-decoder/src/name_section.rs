//! WebAssembly name section handling
//!
//! This module provides utilities for working with WebAssembly name sections,
//! which contain debug information about functions, locals, etc.

use wrt_error::{codes, Error, ErrorCategory, Result};
use wrt_format::binary;

use crate::{prelude::*, types::*};

/// Subsection types in the name section
pub const FUNCTION_SUBSECTION: u8 = 1;
pub const LOCAL_SUBSECTION: u8 = 2;
pub const MODULE_SUBSECTION: u8 = 0;

/// WebAssembly name section types
pub const NAME_MODULE: u8 = 0;
pub const NAME_FUNCTION: u8 = 1;
pub const NAME_LOCAL: u8 = 2;

/// WebAssembly name section
#[derive(Debug, Clone)]
pub struct NameSection {
    /// The module name, if present
    #[cfg(feature = "alloc")]
    pub module_name: Option<String>,
    #[cfg(not(feature = "alloc"))]
    pub module_name: Option<
        wrt_foundation::BoundedString<
            MAX_NAME_LENGTH,
            wrt_foundation::NoStdProvider<MAX_NAME_LENGTH>,
        >,
    >,
    /// Function names, indexed by function index
    #[cfg(feature = "alloc")]
    pub function_names: Vec<(u32, String)>,
    #[cfg(not(feature = "alloc"))]
    pub function_names: NameMapVec,
    /// Local names, indexed by function index and local index
    #[cfg(feature = "alloc")]
    pub local_names: Vec<(u32, Vec<(u32, String)>)>,
    #[cfg(not(feature = "alloc"))]
    pub local_names: LocalNamesVec,
}

#[cfg(feature = "alloc")]
impl Default for NameSection {
    fn default() -> Self {
        Self { module_name: None, function_names: Vec::new(), local_names: Vec::new() }
    }
}

#[cfg(not(feature = "alloc"))]
impl Default for NameSection {
    fn default() -> Self {
        Self {
            module_name: None,
            function_names: NameMapVec::new(wrt_foundation::NoStdProvider::default())
                .unwrap_or_default(),
            local_names: LocalNamesVec::new(wrt_foundation::NoStdProvider::default())
                .unwrap_or_default(),
        }
    }
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
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                format!("Name subsection size {} exceeds data size", subsection_size),
            ));
        }

        let subsection_data = &data[subsection_start..subsection_end];

        match name_type {
            NAME_MODULE => {
                // Module name
                let (name_str, _) = binary::read_string(subsection_data, 0)?;
                #[cfg(feature = "alloc")]
                {
                    name_section.module_name = Some(name_str);
                }
                #[cfg(not(feature = "alloc"))]
                {
                    let name = wrt_foundation::BoundedString::from_str(
                        &name_str,
                        wrt_foundation::NoStdProvider::default(),
                    )
                    .map_err(|_| Error::memory_error("Module name too long"))?;
                    name_section.module_name = Some(name);
                }
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
#[cfg(feature = "alloc")]
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

#[cfg(not(feature = "alloc"))]
fn parse_name_map(bytes: &[u8]) -> Result<(NameMapVec, usize)> {
    let mut offset = 0;

    // Read count
    let (count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    let mut result = NameMapVec::new(wrt_foundation::NoStdProvider::default())
        .map_err(|_| Error::memory_error("Failed to allocate name map"))?;

    for _ in 0..count {
        // Read index
        let (index, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        // Read name
        let (name_str, bytes_read) = binary::read_string(bytes, offset)?;
        offset += bytes_read;

        // Convert String to BoundedString
        let name = wrt_foundation::BoundedString::from_str(
            &name_str,
            wrt_foundation::NoStdProvider::default(),
        )
        .map_err(|_| Error::memory_error("Name too long for bounded string"))?;

        result
            .push((index, name))
            .map_err(|_| Error::memory_error("Name map capacity exceeded"))?;
    }

    Ok((result, offset))
}

/// Parse an indirect name map from a byte array
///
/// An indirect name map is a vector of (index, name_map) pairs.
#[cfg(feature = "alloc")]
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

#[cfg(not(feature = "alloc"))]
fn parse_indirect_name_map(bytes: &[u8]) -> Result<(LocalNamesVec, usize)> {
    let mut offset = 0;

    // Read count
    let (count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    let mut result = LocalNamesVec::new(wrt_foundation::NoStdProvider::default())
        .map_err(|_| Error::memory_error("Failed to allocate indirect name map"))?;

    for _ in 0..count {
        // Read function index
        let (func_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        // Read local name map
        let (local_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        let mut locals = BoundedVec::<
            (
                u32,
                wrt_foundation::BoundedString<
                    MAX_NAME_LENGTH,
                    wrt_foundation::NoStdProvider<MAX_NAME_LENGTH>,
                >,
            ),
            MAX_LOCAL_NAMES,
            wrt_foundation::NoStdProvider<{ MAX_LOCAL_NAMES * (4 + MAX_NAME_LENGTH) }>,
        >::new(wrt_foundation::NoStdProvider::default())
        .map_err(|_| Error::memory_error("Failed to allocate local names"))?;

        for _ in 0..local_count {
            // Read local index
            let (local_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Read local name
            let (name_str, bytes_read) = binary::read_string(bytes, offset)?;
            offset += bytes_read;

            let name = wrt_foundation::BoundedString::from_str(
                &name_str,
                wrt_foundation::NoStdProvider::default(),
            )
            .map_err(|_| Error::memory_error("Local name too long"))?;

            locals
                .push((local_idx, name))
                .map_err(|_| Error::memory_error("Too many local names"))?;
        }

        result
            .push((func_idx, locals))
            .map_err(|_| Error::memory_error("Too many function entries"))?;
    }

    Ok((result, offset))
}

/// Generate a WebAssembly name section
#[cfg(feature = "alloc")]
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
        func_name_data
            .extend_from_slice(&binary::write_leb128_u32(name_section.function_names.len() as u32));

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
        local_name_data
            .extend_from_slice(&binary::write_leb128_u32(name_section.local_names.len() as u32));

        // Function local names
        for &(func_idx, ref locals) in &name_section.local_names {
            local_name_data.extend_from_slice(&binary::write_leb128_u32(func_idx));
            let locals_len: u32 = locals.len() as u32;
            local_name_data.extend_from_slice(&binary::write_leb128_u32(locals_len));

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
#[cfg(feature = "alloc")]
pub fn extract_function_names(data: &[u8]) -> Result<Vec<(u32, String)>> {
    let name_section = parse_name_section(data)?;
    Ok(name_section.function_names)
}

#[cfg(not(feature = "alloc"))]
pub fn extract_function_names(data: &[u8]) -> Result<NameMapVec> {
    let name_section = parse_name_section(data)?;
    Ok(name_section.function_names)
}

/// Set function names in a module's name section
#[cfg(feature = "alloc")]
pub fn create_function_names_section(names: &[(u32, String)]) -> Result<Vec<u8>> {
    let name_section =
        NameSection { module_name: None, function_names: names.to_vec(), local_names: Vec::new() };

    generate_name_section(&name_section)
}

/// Create a parse error
pub fn parse_error(message: &str) -> Error {
    Error::new(ErrorCategory::Parse, codes::PARSE_ERROR, message.to_string())
}

/// Create a parse error with context
pub fn parse_error_with_context(message: &str, context: &str) -> Error {
    Error::new(ErrorCategory::Parse, codes::PARSE_ERROR, format!("{}: {}", message, context))
}

/// Create a parse error with position
pub fn parse_error_with_position(message: &str, position: usize) -> Error {
    Error::new(
        ErrorCategory::Parse,
        codes::PARSE_ERROR,
        format!("{} at position {}", message, position),
    )
}

/// Extract the module name from a name section payload
#[cfg(feature = "alloc")]
pub fn extract_module_name(data: &[u8]) -> Result<String> {
    // Parse the name section
    let name_section = parse_name_section(data)?;

    // Return the module name or error
    if let Some(name) = name_section.module_name {
        Ok(name)
    } else {
        Err(Error::new(
            ErrorCategory::Parse,
            codes::PARSE_ERROR,
            "No module name found in name section",
        ))
    }
}

#[cfg(not(feature = "alloc"))]
pub fn extract_module_name(
    data: &[u8],
) -> Result<
    wrt_foundation::BoundedString<MAX_NAME_LENGTH, wrt_foundation::NoStdProvider<MAX_NAME_LENGTH>>,
> {
    // Parse the name section
    let name_section = parse_name_section(data)?;

    // Return the module name or error
    if let Some(name) = name_section.module_name {
        Ok(name)
    } else {
        Err(Error::new(
            ErrorCategory::Parse,
            codes::PARSE_ERROR,
            "No module name found in name section",
        ))
    }
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
