// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! WebAssembly Component Model decoding.
//!
//! This module provides functions for decoding WebAssembly Component Model
//! components from binary format.

pub mod analysis;
pub mod binary_parser;
#[cfg(test)]
pub mod binary_parser_tests;
pub mod component_name_section;
pub mod decode;
// Binary std/no_std choice
pub mod decode_no_alloc;
mod encode;
pub mod name_section;
mod parse;
pub mod section;
pub mod types;
pub mod utils;
pub mod val_type;
pub mod validation;

#[cfg(feature = "std")]
pub use analysis::{
    analyze_component, analyze_component_extended, extract_embedded_modules, extract_inline_module,
    extract_module_info, is_valid_module, AliasInfo, ComponentSummary, CoreInstanceInfo,
    CoreModuleInfo, ExtendedExportInfo, ExtendedImportInfo, ModuleExportInfo, ModuleImportInfo,
};
pub use binary_parser::{
    parse_component_binary, parse_component_binary_with_validation, ComponentBinaryParser,
    ComponentHeader, ComponentSectionId, ValidationLevel,
};
#[cfg(feature = "std")]
pub use decode::decode_component as decode_component_internal;
#[cfg(feature = "std")]
pub use encode::encode_component;
pub use name_section::{
    generate_component_name_section, parse_component_name_section, ComponentNameSection, NameMap,
    NameMapEntry, SortIdentifier,
};
pub use types::{
    Component, ComponentAnalyzer, ComponentMetadata, ComponentType, CoreExternType, CoreInstance,
    CoreType, Export, ExportInfo, ExternType, Import, ImportInfo, Instance, ModuleInfo, Start,
    ValType,
};
pub use utils::*;
pub use val_type::encode_val_type;
pub use validation::{validate_component, validate_component_with_config, ValidationConfig};
use wrt_error::{codes, Error, ErrorCategory, Result};

use crate::{prelude::*, utils::BinaryType};

/// Decode a component from binary data
///
/// This is the public entry point for decoding a component from binary data.
///
/// # Arguments
///
/// * `binary` - The binary data containing the component
///
/// # Returns
///
/// * `Result<Component>` - The decoded component or an error
#[cfg(feature = "component-model-core")]
pub fn decode_component_binary(binary: &[u8]) -> Result<Component> {
    decode_component_internal(binary)
}

/// Decode a WebAssembly Component Model component
///
/// This function takes a WebAssembly Component Model binary and decodes it
/// into a structured Component representation.
///
/// # Arguments
///
/// * `binary` - The WebAssembly Component Model binary data
///
/// # Returns
///
/// * `Result<Component>` - The decoded component or an error
pub fn decode_component(binary: &[u8]) -> Result<Component> {
    // Detect binary type first
    match crate::utils::detect_binary_type(binary)? {
        BinaryType::CoreModule => {
            // Can't decode a core module as a component
            Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Cannot decode a WebAssembly core module as a Component",
            ))
        }
        BinaryType::Component => {
            // Verify component header
            if binary.len() < 8 {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Component binary too short",
                ));
            }

            if binary[0..4] != [0x00, 0x63, 0x6D, 0x70] {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Invalid Component Model magic number",
                ));
            }

            if binary[4..8] != [0x01, 0x00, 0x00, 0x00] {
                return Err(Error::new(
                    ErrorCategory::Parse,
                    codes::PARSE_ERROR,
                    "Unsupported Component version",
                ));
            }

            // Parse component (skip magic number and version)
            let mut component = Component::default();

            // Store the binary data
            component.binary = Some(binary.to_vec());

            // Parse component sections
            parse_component_sections(&binary[8..], &mut component)?;

            Ok(component)
        }
    }
}

/// Parse the sections of a Component Model component
fn parse_component_sections(data: &[u8], component: &mut Component) -> Result<()> {
    let mut offset = 0;

    // Parse each section
    while offset < data.len() {
        // Read section ID
        let section_id = data[offset];
        offset += 1;

        // Read section size
        let (section_size, size_len) = wrt_format::binary::read_leb128_u32(data, offset)?;
        offset += size_len;

        // Ensure the section size is valid
        if offset + section_size as usize > data.len() {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
"Section size exceeds remaining data size",
            ));
        }

        let section_data = &data[offset..offset + section_size as usize];

        // Parse the section based on its ID
        match section_id {
            0x00 => {
                // Custom section - delegate to common custom section parser
                let (name, bytes_read) = crate::utils::read_name_as_string(section_data, 0)?;
                let custom_data = &section_data[bytes_read..];

                // Store custom section as needed
                // component doesn't have a custom_sections field
                // just process it if it's a name section
                if name == "name" {
                    if let Ok(name_section) =
                        component_name_section::parse_component_name_section(custom_data)
                    {
                        if let Some(component_name) = name_section.component_name {
                            component.name = Some(component_name);
                        }
                    }
                }
            }
            0x01 => {
                // Type section
                let (types, _) = parse::parse_component_type_section(section_data)?;
                component.types = types;
            }
            0x02 => {
                // Import section
                let (imports, _) = parse::parse_import_section(section_data)?;
                component.imports = imports;
            }
            0x03 => {
                // Core module section
                let (modules, _) = parse::parse_core_module_section(section_data)?;
                component.modules = modules;
            }
            0x04 => {
                // Function section
                // Skip - currently not implemented for component model
                // Functions are handled differently in the component model
            }
            0x05 => {
                // Table section
                // Skip - currently not implemented for component model
                // Tables are handled differently in the component model
            }
            0x06 => {
                // Memory section
                // Skip - currently not implemented for component model
                // Memories are handled differently in the component model
            }
            0x07 => {
                // Global section
                // Skip - currently not implemented for component model
                // Globals are handled differently in the component model
            }
            0x08 => {
                // Export section
                let (exports, _) = parse::parse_export_section(section_data)?;
                component.exports = exports;
            }
            0x09 => {
                // Start section
                let (start, _) = parse::parse_start_section(section_data)?;
                component.start = Some(start);
            }
            0x0A => {
                // Element section
                // Skip - currently not implemented for component model
                // Elements are handled differently in the component model
            }
            0x0B => {
                // Data section
                // Skip - currently not implemented for component model
                // Data sections are handled differently in the component model
            }
            0x10 => {
                // Instance section
                let (instances, _) = parse::parse_instance_section(section_data)?;
                component.instances = instances;
            }
            0x11 => {
                // Component section
                let (components, _) = parse::parse_component_section(section_data)?;
                component.components = components;
            }
            0x12 => {
                // Alias section
                let (aliases, _) = parse::parse_alias_section(section_data)?;
                component.aliases = aliases;
            }
            0x13 => {
                // Core instance section
                let (core_instances, _) = parse::parse_core_instance_section(section_data)?;
                component.core_instances = core_instances;
            }
            0x14 => {
                // Core type section
                let (core_types, _) = parse::parse_core_type_section(section_data)?;
                component.core_types = core_types;
            }
            0x15 => {
                // Canon section
                let (canons, _) = parse::parse_canon_section(section_data)?;
                component.canonicals = canons;
            }
            _ => {
                // Unknown section - ignore for now
                // We could log a warning here
            }
        }

        // Move to the next section
        offset += section_size as usize;
    }

    Ok(())
}
