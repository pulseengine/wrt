//! Format adapter for wrt-format integration.
//!
//! This module provides adapters between wrt and wrt-format types,
//! serving as a bridge between the two module representations.

use wrt_format::{
    create_state_section, extract_state_section, CompressionType, CustomSection,
    Module as FormatModule, StateSection,
};

use crate::error::Result;

/// Convert a wrt::module::Module to a wrt_format::Module
pub fn convert_to_format_module(module: &crate::module::Module) -> Result<FormatModule> {
    // Create a new format module
    let mut format_module = FormatModule::new();

    // Copy the binary if available
    if let Some(binary) = &module.binary {
        format_module.binary = Some(binary.clone());
    }

    // Copy custom sections
    for section in &module.custom_sections {
        let custom_section = CustomSection {
            name: section.name.clone(),
            data: section.data.clone(),
        };
        format_module.add_custom_section(custom_section);
    }

    // In future implementations, we'll convert all module components
    // (types, functions, tables, etc.)

    Ok(format_module)
}

/// Convert a wrt_format::Module to a wrt::module::Module
pub fn convert_from_format_module(format_module: FormatModule) -> Result<crate::module::Module> {
    // For now, just use the binary if available
    if let Some(binary) = &format_module.binary {
        return crate::module::Module::from_bytes(binary);
    }

    // Otherwise, create an empty module
    let mut module = crate::module::Module::new()?;

    // Copy custom sections
    for section in format_module.custom_sections {
        let custom_section = crate::module::CustomSection {
            name: section.name,
            data: section.data,
        };
        module.custom_sections.push(custom_section);
    }

    Ok(module)
}

/// Determine if a module contains serialized state sections
pub fn has_state_sections(module: &crate::module::Module) -> Result<bool> {
    // Convert to format module
    let format_module = convert_to_format_module(module)?;

    // Check for state sections
    Ok(format_module.has_state_sections())
}

/// Create a state section for serializing engine state
pub fn create_engine_state_section(
    section_type: StateSection,
    data: &[u8],
    use_compression: bool,
) -> Result<CustomSection> {
    // Use compression if requested
    let compression = if use_compression {
        CompressionType::RLE
    } else {
        CompressionType::None
    };

    // Create the section
    create_state_section(section_type, data, compression)
}

/// Extract state from a module
pub fn extract_engine_state(
    module: &crate::module::Module,
    section_type: StateSection,
) -> Result<Vec<u8>> {
    // Convert to format module
    let format_module = convert_to_format_module(module)?;

    // Find the section
    let section = format_module
        .find_custom_section(&section_type.name())
        .ok_or_else(|| {
            crate::error::Error::execution_error(format!(
                "State section {} not found",
                section_type.name()
            ))
        })?;

    // Extract the data
    let (_, data) = extract_state_section(section)?;

    Ok(data)
}
