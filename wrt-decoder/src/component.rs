//! WebAssembly Component Model decoding.
//!
//! This module provides functions for decoding WebAssembly Component Model
//! components from binary format.

use crate::component_name_section::{
    generate_component_name_section, parse_component_name_section, ComponentNameSection,
};
use crate::prelude::*;
use wrt_error::{kinds, Error, Result};
use wrt_format::binary;
use wrt_format::{
    component::{
        Alias, AliasTarget, Component, CoreInlineExport, CoreInstance, CoreInstanceExpr,
        CoreInstantiateArg, CoreSort, CoreType, CoreTypeDefinition, Export, ExportName, Import,
        ImportName, Instance, Sort, Start, ValType, Value,
    },
    Module,
};

const COMPONENT_VERSION: [u8; 4] = [0x01, 0x00, 0x00, 0x00];

/// Component Model binary format magic bytes (same as core: \0asm)
pub const COMPONENT_MAGIC: [u8; 4] = binary::COMPONENT_MAGIC;
/// Component Model layer identifier
pub const COMPONENT_LAYER: [u8; 2] = binary::COMPONENT_LAYER;

/// Decode a WebAssembly Component Model binary into a structured component representation
pub fn decode_component(bytes: &[u8]) -> Result<Component> {
    // Verify magic bytes, version, and layer
    if bytes.len() < 8 {
        return Err(Error::new(kinds::ParseError(
            "Component binary too short".to_string(),
        )));
    }

    // Check magic bytes (\0asm)
    if bytes[0..4] != COMPONENT_MAGIC {
        return Err(Error::new(kinds::ParseError(
            "Invalid Component magic bytes".to_string(),
        )));
    }

    // Check version (0xD.0)
    if bytes[4..8] != COMPONENT_VERSION {
        return Err(Error::new(kinds::ParseError(
            "Unsupported Component version".to_string(),
        )));
    }

    // Check layer (1)
    if bytes[6..8] != COMPONENT_LAYER {
        return Err(Error::new(kinds::ParseError(
            "Invalid Component layer identifier (not a component)".to_string(),
        )));
    }

    // Create an empty component with the binary stored
    let mut component = Component::new();
    component.binary = Some(bytes.to_vec());

    // Parse sections
    let mut offset = 8;
    let mut name_section: Option<ComponentNameSection> = None;

    while offset < bytes.len() {
        let section_id = bytes[offset];
        offset += 1;

        let (size, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        let section_start = offset;
        let section_end = section_start + size as usize;

        if section_end > bytes.len() {
            return Err(Error::new(kinds::ParseError(format!(
                "Component section size {} for section ID {} exceeds binary size",
                size, section_id
            ))));
        }

        let section_bytes = &bytes[section_start..section_end];

        // Parse each section type
        match section_id {
            binary::COMPONENT_CUSTOM_SECTION_ID => {
                // Handle custom section
                if !section_bytes.is_empty() {
                    // Try to read name
                    let (name, name_size) = binary::read_string(section_bytes, 0)?;

                    // Check if this is a "name" section
                    if name == "name" && section_bytes.len() > name_size {
                        // Parse component name section
                        let name_data = &section_bytes[name_size..];
                        name_section = Some(parse_component_name_section(name_data)?);
                    }
                }
            }
            binary::COMPONENT_CORE_MODULE_SECTION_ID => {
                // Core module section
                let (modules, _) = parse_core_module_section(section_bytes)?;
                component.modules.extend(modules);
            }
            binary::COMPONENT_CORE_INSTANCE_SECTION_ID => {
                // Core instance section
                let (instances, _) = parse_core_instance_section(section_bytes)?;
                component.core_instances.extend(instances);
            }
            binary::COMPONENT_CORE_TYPE_SECTION_ID => {
                // Core type section
                let (types, _) = parse_core_type_section(section_bytes)?;
                component.core_types.extend(types);
            }
            binary::COMPONENT_COMPONENT_SECTION_ID => {
                // Nested component section
                let (components, _) = parse_component_section(section_bytes)?;
                component.components.extend(components);
            }
            binary::COMPONENT_INSTANCE_SECTION_ID => {
                // Instance section
                let (instances, _) = parse_instance_section(section_bytes)?;
                component.instances.extend(instances);
            }
            binary::COMPONENT_ALIAS_SECTION_ID => {
                // Alias section
                let (aliases, _) = parse_alias_section(section_bytes)?;
                component.aliases.extend(aliases);
            }
            binary::COMPONENT_TYPE_SECTION_ID => {
                // Type section
                let (types, _) = parse_type_section(section_bytes)?;
                component.types.extend(types);
            }
            binary::COMPONENT_CANON_SECTION_ID => {
                // Canon section
                let (canons, _) = parse_canon_section(section_bytes)?;
                component.canonicals.extend(canons);
            }
            binary::COMPONENT_START_SECTION_ID => {
                // Start section
                let (start, _) = parse_start_section(section_bytes)?;
                component.start = Some(start);
            }
            binary::COMPONENT_IMPORT_SECTION_ID => {
                // Import section
                let (imports, _) = parse_import_section(section_bytes)?;
                component.imports.extend(imports);
            }
            binary::COMPONENT_EXPORT_SECTION_ID => {
                // Export section
                let (exports, _) = parse_export_section(section_bytes)?;
                component.exports.extend(exports);
            }
            binary::COMPONENT_VALUE_SECTION_ID => {
                // Value section
                let (values, _) = parse_value_section(section_bytes)?;
                component.values.extend(values);
            }
            _ => {
                return Err(Error::new(kinds::ParseError(format!(
                    "Unknown Component section ID: {}",
                    section_id
                ))));
            }
        }

        offset = section_end;
    }

    // Store name section information if present
    if let Some(name_section) = name_section {
        // Store component name
        if let Some(name) = name_section.component_name {
            component.name = Some(name);
        }

        // We could also process and store other name information here
        // For example, function names, instance names, etc.
        // This would be useful for debugging and error reporting
    }

    Ok(component)
}

/// Encode a component to binary format
pub fn encode_component(component: &Component) -> Result<Vec<u8>> {
    // If the component has the original binary and hasn't been modified,
    // we can just return that
    if let Some(binary) = &component.binary {
        return Ok(binary.clone());
    }

    // Otherwise, we need to generate the binary
    let mut result = Vec::new();

    // Write preamble
    result.extend_from_slice(&COMPONENT_MAGIC);
    result.extend_from_slice(&COMPONENT_VERSION);
    result.extend_from_slice(&COMPONENT_LAYER);

    // Generate and add sections in the correct order

    // 1. Core module section
    if !component.modules.is_empty() {
        let section_contents = encode_core_module_section(&component.modules)?;
        add_section(
            &mut result,
            binary::COMPONENT_CORE_MODULE_SECTION_ID,
            &section_contents,
        );
    }

    // 2. Core instance section
    if !component.core_instances.is_empty() {
        let section_contents = encode_core_instance_section(&component.core_instances)?;
        add_section(
            &mut result,
            binary::COMPONENT_CORE_INSTANCE_SECTION_ID,
            &section_contents,
        );
    }

    // 3. Core type section
    if !component.core_types.is_empty() {
        let section_contents = encode_core_type_section(&component.core_types)?;
        add_section(
            &mut result,
            binary::COMPONENT_CORE_TYPE_SECTION_ID,
            &section_contents,
        );
    }

    // 4. Component section
    if !component.components.is_empty() {
        let section_contents = encode_component_section(&component.components)?;
        add_section(
            &mut result,
            binary::COMPONENT_COMPONENT_SECTION_ID,
            &section_contents,
        );
    }

    // 5. Instance section
    if !component.instances.is_empty() {
        let section_contents = encode_instance_section(&component.instances)?;
        add_section(
            &mut result,
            binary::COMPONENT_INSTANCE_SECTION_ID,
            &section_contents,
        );
    }

    // 6. Alias section
    if !component.aliases.is_empty() {
        let section_contents = encode_alias_section(&component.aliases)?;
        add_section(
            &mut result,
            binary::COMPONENT_ALIAS_SECTION_ID,
            &section_contents,
        );
    }

    // 7. Type section
    if !component.types.is_empty() {
        let section_contents = encode_type_section(&component.types)?;
        add_section(
            &mut result,
            binary::COMPONENT_TYPE_SECTION_ID,
            &section_contents,
        );
    }

    // 8. Canon section
    if !component.canonicals.is_empty() {
        let section_contents = encode_canon_section(&component.canonicals)?;
        add_section(
            &mut result,
            binary::COMPONENT_CANON_SECTION_ID,
            &section_contents,
        );
    }

    // 9. Start section
    if let Some(start) = &component.start {
        let section_contents = encode_start_section(start)?;
        add_section(
            &mut result,
            binary::COMPONENT_START_SECTION_ID,
            &section_contents,
        );
    }

    // 10. Import section
    if !component.imports.is_empty() {
        let section_contents = encode_import_section(&component.imports)?;
        add_section(
            &mut result,
            binary::COMPONENT_IMPORT_SECTION_ID,
            &section_contents,
        );
    }

    // 11. Export section
    if !component.exports.is_empty() {
        let section_contents = encode_export_section(&component.exports)?;
        add_section(
            &mut result,
            binary::COMPONENT_EXPORT_SECTION_ID,
            &section_contents,
        );
    }

    // 12. Value section
    if !component.values.is_empty() {
        let section_contents = encode_value_section(&component.values)?;
        add_section(
            &mut result,
            binary::COMPONENT_VALUE_SECTION_ID,
            &section_contents,
        );
    }

    // Name section (custom section)
    if let Some(name) = &component.name {
        // Create a name section with the component name
        let name_section = ComponentNameSection {
            component_name: Some(name.clone()),
            sort_names: Vec::new(),
            import_names: Vec::new(),
            export_names: Vec::new(),
            canonical_names: Vec::new(),
            type_names: Vec::new(),
        };

        // Generate name section binary
        let name_section_data = generate_component_name_section(&name_section)?;

        // Create custom section content with "name" as the identifier
        let mut custom_section_content = binary::write_string("name");
        custom_section_content.extend_from_slice(&name_section_data);

        // Add as custom section
        add_section(
            &mut result,
            binary::COMPONENT_CUSTOM_SECTION_ID,
            &custom_section_content,
        );
    }

    Ok(result)
}

/// Add a section to the binary
fn add_section(binary: &mut Vec<u8>, section_id: u8, content: &[u8]) {
    // Section ID
    binary.push(section_id);

    // Section size
    binary.extend_from_slice(&binary::write_leb128_u32(content.len() as u32));

    // Section content
    binary.extend_from_slice(content);
}

/// Parse the core module section
///
/// The core module section contains a vector of WebAssembly core modules that
/// can be instantiated by the component.
fn parse_core_module_section(bytes: &[u8]) -> Result<(Vec<Module>, usize)> {
    let mut offset = 0;
    let mut modules = Vec::new();

    // Read the number of modules in this section
    let (count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    // Parse each module
    for _ in 0..count {
        // Read module size
        let (module_size, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        if offset + module_size as usize > bytes.len() {
            return Err(Error::new(kinds::ParseError(
                "Module size exceeds section size".to_string(),
            )));
        }

        // Extract the module bytes
        let module_bytes = &bytes[offset..offset + module_size as usize];

        // Parse the module using the standard WebAssembly module decoder
        let mut module = Module::new();

        // Check that the module starts with valid magic and version
        if module_bytes.len() < 8
            || module_bytes[0..4] != binary::WASM_MAGIC
            || module_bytes[4..8] != binary::WASM_VERSION
        {
            return Err(Error::new(kinds::ParseError(
                "Invalid WebAssembly module header in component".to_string(),
            )));
        }

        // Store the binary for later parsing
        module.binary = Some(module_bytes.to_vec());

        // In a full implementation, we would also parse the module structure here
        // We could use wrt_decoder::module::decode_module for this, but that would
        // create a circular dependency. For now, we just store the binary.

        modules.push(module);
        offset += module_size as usize;
    }

    Ok((modules, offset))
}

/// Parse the core instance section
///
/// The core instance section contains definitions of core WebAssembly instances,
/// which can be either instantiations of core modules or collections of exports.
fn parse_core_instance_section(bytes: &[u8]) -> Result<(Vec<CoreInstance>, usize)> {
    let mut offset = 0;
    let mut instances = Vec::new();

    // Read the number of instances in this section
    let (count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    // Parse each instance
    for _ in 0..count {
        // Read the instance expression tag
        if offset >= bytes.len() {
            return Err(Error::new(kinds::ParseError(
                "Unexpected end of core instance section".to_string(),
            )));
        }

        let tag = bytes[offset];
        offset += 1;

        let instance_expr = match tag {
            // Instantiate a module
            0x00 => {
                // Read module index
                let (module_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Read argument count
                let (arg_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Parse arguments
                let mut args = Vec::with_capacity(arg_count as usize);
                for _ in 0..arg_count {
                    // Read argument name
                    let (name, bytes_read) = binary::read_string(bytes, offset)?;
                    offset += bytes_read;

                    // Read instance index for the argument
                    let (instance_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                    offset += bytes_read;

                    args.push(CoreInstantiateArg { name, instance_idx });
                }

                CoreInstanceExpr::Instantiate { module_idx, args }
            }
            // Inline exports
            0x01 => {
                // Read export count
                let (export_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Parse exports
                let mut exports = Vec::with_capacity(export_count as usize);
                for _ in 0..export_count {
                    // Read export name
                    let (name, bytes_read) = binary::read_string(bytes, offset)?;
                    offset += bytes_read;

                    // Read sort kind
                    if offset >= bytes.len() {
                        return Err(Error::new(kinds::ParseError(
                            "Unexpected end of core instance exports".to_string(),
                        )));
                    }

                    let sort_byte = bytes[offset];
                    offset += 1;

                    let sort = match sort_byte {
                        binary::COMPONENT_CORE_SORT_FUNC => CoreSort::Function,
                        binary::COMPONENT_CORE_SORT_TABLE => CoreSort::Table,
                        binary::COMPONENT_CORE_SORT_MEMORY => CoreSort::Memory,
                        binary::COMPONENT_CORE_SORT_GLOBAL => CoreSort::Global,
                        binary::COMPONENT_CORE_SORT_TYPE => CoreSort::Type,
                        binary::COMPONENT_CORE_SORT_MODULE => CoreSort::Module,
                        binary::COMPONENT_CORE_SORT_INSTANCE => CoreSort::Instance,
                        _ => {
                            return Err(Error::new(kinds::ParseError(format!(
                                "Invalid core sort kind: {}",
                                sort_byte
                            ))))
                        }
                    };

                    // Read sort index
                    let (idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                    offset += bytes_read;

                    exports.push(CoreInlineExport { name, sort, idx });
                }

                CoreInstanceExpr::InlineExports(exports)
            }
            _ => {
                return Err(Error::new(kinds::ParseError(format!(
                    "Invalid core instance expression tag: {}",
                    tag
                ))))
            }
        };

        instances.push(CoreInstance { instance_expr });
    }

    Ok((instances, offset))
}

/// Parse the core type section
///
/// The core type section contains type definitions used by core WebAssembly modules
/// in the component, including function types and module types.
fn parse_core_type_section(bytes: &[u8]) -> Result<(Vec<CoreType>, usize)> {
    let mut offset = 0;
    let mut types = Vec::new();

    // Read the number of types in this section
    let (count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    // Parse each type
    for _ in 0..count {
        // Read the type tag
        if offset >= bytes.len() {
            return Err(Error::new(kinds::ParseError(
                "Unexpected end of core type section".to_string(),
            )));
        }

        let tag = bytes[offset];
        offset += 1;

        let definition = match tag {
            // Function type
            0x00 => {
                // Read parameter count
                let (param_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Parse parameter types
                let mut params = Vec::with_capacity(param_count as usize);
                for _ in 0..param_count {
                    if offset >= bytes.len() {
                        return Err(Error::new(kinds::ParseError(
                            "Unexpected end of function type params".to_string(),
                        )));
                    }

                    // Parse value type
                    let val_type_byte = bytes[offset];
                    offset += 1;

                    let val_type =
                        wrt_format::types::parse_value_type(val_type_byte).map_err(|_| {
                            Error::new(kinds::ParseError(format!(
                                "Invalid value type byte: 0x{:02x}",
                                val_type_byte
                            )))
                        })?;

                    params.push(val_type);
                }

                // Read result count
                let (result_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Parse result types
                let mut results = Vec::with_capacity(result_count as usize);
                for _ in 0..result_count {
                    if offset >= bytes.len() {
                        return Err(Error::new(kinds::ParseError(
                            "Unexpected end of function type results".to_string(),
                        )));
                    }

                    // Parse value type
                    let val_type_byte = bytes[offset];
                    offset += 1;

                    let val_type =
                        wrt_format::types::parse_value_type(val_type_byte).map_err(|_| {
                            Error::new(kinds::ParseError(format!(
                                "Invalid value type byte: 0x{:02x}",
                                val_type_byte
                            )))
                        })?;

                    results.push(val_type);
                }

                CoreTypeDefinition::Function { params, results }
            }
            // Module type
            0x01 => {
                // Read import count
                let (import_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Parse imports
                let mut imports = Vec::with_capacity(import_count as usize);
                for _ in 0..import_count {
                    // Read module name
                    let (module_name, bytes_read) = binary::read_string(bytes, offset)?;
                    offset += bytes_read;

                    // Read import name
                    let (import_name, bytes_read) = binary::read_string(bytes, offset)?;
                    offset += bytes_read;

                    // Read external type
                    let (extern_type, bytes_read) = parse_core_extern_type(bytes, offset)?;
                    offset += bytes_read;

                    imports.push((module_name, import_name, extern_type));
                }

                // Read export count
                let (export_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Parse exports
                let mut exports = Vec::with_capacity(export_count as usize);
                for _ in 0..export_count {
                    // Read export name
                    let (export_name, bytes_read) = binary::read_string(bytes, offset)?;
                    offset += bytes_read;

                    // Read external type
                    let (extern_type, bytes_read) = parse_core_extern_type(bytes, offset)?;
                    offset += bytes_read;

                    exports.push((export_name, extern_type));
                }

                CoreTypeDefinition::Module { imports, exports }
            }
            _ => {
                return Err(Error::new(kinds::ParseError(format!(
                    "Invalid core type tag: {}",
                    tag
                ))))
            }
        };

        types.push(CoreType { definition });
    }

    Ok((types, offset))
}

/// Parse a core external type from the binary format
fn parse_core_extern_type(
    bytes: &[u8],
    pos: usize,
) -> Result<(wrt_format::component::CoreExternType, usize)> {
    if pos >= bytes.len() {
        return Err(Error::new(kinds::ParseError(
            "Unexpected end of core extern type".to_string(),
        )));
    }

    let tag = bytes[pos];
    let mut offset = pos + 1;

    match tag {
        // Function type
        0x00 => {
            // Read parameter count
            let (param_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Parse parameter types
            let mut params = Vec::with_capacity(param_count as usize);
            for _ in 0..param_count {
                if offset >= bytes.len() {
                    return Err(Error::new(kinds::ParseError(
                        "Unexpected end of function type params".to_string(),
                    )));
                }

                // Parse value type
                let val_type_byte = bytes[offset];
                offset += 1;

                let val_type =
                    wrt_format::types::parse_value_type(val_type_byte).map_err(|_| {
                        Error::new(kinds::ParseError(format!(
                            "Invalid value type byte: 0x{:02x}",
                            val_type_byte
                        )))
                    })?;

                params.push(val_type);
            }

            // Read result count
            let (result_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Parse result types
            let mut results = Vec::with_capacity(result_count as usize);
            for _ in 0..result_count {
                if offset >= bytes.len() {
                    return Err(Error::new(kinds::ParseError(
                        "Unexpected end of function type results".to_string(),
                    )));
                }

                // Parse value type
                let val_type_byte = bytes[offset];
                offset += 1;

                let val_type =
                    wrt_format::types::parse_value_type(val_type_byte).map_err(|_| {
                        Error::new(kinds::ParseError(format!(
                            "Invalid value type byte: 0x{:02x}",
                            val_type_byte
                        )))
                    })?;

                results.push(val_type);
            }

            Ok((
                wrt_format::component::CoreExternType::Function { params, results },
                offset - pos,
            ))
        }
        // Table type
        0x01 => {
            // Read element type
            if offset >= bytes.len() {
                return Err(Error::new(kinds::ParseError(
                    "Unexpected end of table type".to_string(),
                )));
            }

            let elem_type_byte = bytes[offset];
            offset += 1;

            let element_type =
                wrt_format::types::parse_value_type(elem_type_byte).map_err(|_| {
                    Error::new(kinds::ParseError(format!(
                        "Invalid element type byte: 0x{:02x}",
                        elem_type_byte
                    )))
                })?;

            // Read limits
            let (min, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Read max (if present)
            let has_max = bytes[offset] != 0;
            offset += 1;

            let max = if has_max {
                let (max_val, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;
                Some(max_val)
            } else {
                None
            };

            Ok((
                wrt_format::component::CoreExternType::Table {
                    element_type,
                    min,
                    max,
                },
                offset - pos,
            ))
        }
        // Memory type
        0x02 => {
            // Read limits
            let (min, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Read max (if present) and flags
            let flags = bytes[offset];
            offset += 1;

            let has_max = (flags & 0x01) != 0;
            let shared = (flags & 0x02) != 0;

            let max = if has_max {
                let (max_val, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;
                Some(max_val)
            } else {
                None
            };

            Ok((
                wrt_format::component::CoreExternType::Memory { min, max, shared },
                offset - pos,
            ))
        }
        // Global type
        0x03 => {
            // Read value type
            if offset >= bytes.len() {
                return Err(Error::new(kinds::ParseError(
                    "Unexpected end of global type".to_string(),
                )));
            }

            let val_type_byte = bytes[offset];
            offset += 1;

            let value_type = wrt_format::types::parse_value_type(val_type_byte).map_err(|_| {
                Error::new(kinds::ParseError(format!(
                    "Invalid value type byte: 0x{:02x}",
                    val_type_byte
                )))
            })?;

            // Read mutability flag
            if offset >= bytes.len() {
                return Err(Error::new(kinds::ParseError(
                    "Unexpected end of global type".to_string(),
                )));
            }

            let mutable = bytes[offset] != 0;
            offset += 1;

            Ok((
                wrt_format::component::CoreExternType::Global {
                    value_type,
                    mutable,
                },
                offset - pos,
            ))
        }
        _ => Err(Error::new(kinds::ParseError(format!(
            "Invalid core extern type tag: {}",
            tag
        )))),
    }
}

/// Parse the component section
///
/// The component section contains nested WebAssembly components that
/// can be instantiated by the parent component.
fn parse_component_section(bytes: &[u8]) -> Result<(Vec<Component>, usize)> {
    let mut offset = 0;
    let mut components = Vec::new();

    // Read the number of components in this section
    let (count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    // Parse each component
    for _ in 0..count {
        // Read component size
        let (component_size, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        if offset + component_size as usize > bytes.len() {
            return Err(Error::new(kinds::ParseError(
                "Component size exceeds section size".to_string(),
            )));
        }

        // Extract the component bytes
        let component_bytes = &bytes[offset..offset + component_size as usize];

        // Parse the component using the component decoder
        // We need to validate that the component has the correct header
        if component_bytes.len() < 8
            || component_bytes[0..4] != COMPONENT_MAGIC
            || component_bytes[4..6] != COMPONENT_VERSION
            || component_bytes[6..8] != COMPONENT_LAYER
        {
            return Err(Error::new(kinds::ParseError(
                "Invalid WebAssembly component header in nested component".to_string(),
            )));
        }

        // Use the component decoder to parse the nested component
        let component = decode_component(component_bytes)?;

        components.push(component);
        offset += component_size as usize;
    }

    Ok((components, offset))
}

/// Parse the instance section
fn parse_instance_section(bytes: &[u8]) -> Result<(Vec<Instance>, usize)> {
    // This is a placeholder implementation
    // In a real implementation, we would parse each instance
    Ok((Vec::new(), bytes.len()))
}

/// Parse the alias section
///
/// The alias section contains definitions of aliases to imported items or items from
/// instantiated modules or components.
fn parse_alias_section(bytes: &[u8]) -> Result<(Vec<Alias>, usize)> {
    let mut offset = 0;
    let mut aliases = Vec::new();

    // Read the number of aliases in this section
    let (count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    // Parse each alias
    for _ in 0..count {
        if offset >= bytes.len() {
            return Err(Error::new(kinds::ParseError(
                "Unexpected end of data while parsing alias".to_string(),
            )));
        }

        // Read the alias kind
        let alias_kind = bytes[offset];
        offset += 1;

        let target = match alias_kind {
            // Core instance export
            0x00 => {
                // Read instance index
                let (instance_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Read kind
                if offset >= bytes.len() {
                    return Err(Error::new(kinds::ParseError(
                        "Unexpected end of data while parsing alias kind".to_string(),
                    )));
                }
                let kind_byte = bytes[offset];
                offset += 1;

                let kind = match kind_byte {
                    binary::COMPONENT_CORE_SORT_FUNC => CoreSort::Function,
                    binary::COMPONENT_CORE_SORT_TABLE => CoreSort::Table,
                    binary::COMPONENT_CORE_SORT_MEMORY => CoreSort::Memory,
                    binary::COMPONENT_CORE_SORT_GLOBAL => CoreSort::Global,
                    binary::COMPONENT_CORE_SORT_TYPE => CoreSort::Type,
                    binary::COMPONENT_CORE_SORT_MODULE => CoreSort::Module,
                    binary::COMPONENT_CORE_SORT_INSTANCE => CoreSort::Instance,
                    _ => {
                        return Err(Error::new(kinds::ParseError(format!(
                            "Invalid core sort kind: {}",
                            kind_byte
                        ))))
                    }
                };

                // Read name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                AliasTarget::CoreInstanceExport {
                    instance_idx,
                    name,
                    kind,
                }
            }
            // Core module export - Not fully implemented in the specification yet
            0x01 => {
                // For now, we'll use CoreInstanceExport as a placeholder
                // Read module index
                let (module_index, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Read name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // This is a placeholder until the specification is finalized
                AliasTarget::CoreInstanceExport {
                    instance_idx: module_index,
                    name,
                    kind: CoreSort::Module,
                }
            }
            // Component export - Not fully implemented in the specification yet
            0x02 => {
                // For now, we'll use InstanceExport as a placeholder
                // Read component index
                let (component_index, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Read kind
                if offset >= bytes.len() {
                    return Err(Error::new(kinds::ParseError(
                        "Unexpected end of data while parsing alias kind".to_string(),
                    )));
                }
                let kind_byte = bytes[offset];
                offset += 1;

                let kind = match kind_byte {
                    binary::COMPONENT_SORT_CORE => {
                        // For core sorts, we need to read the core sort kind
                        if offset >= bytes.len() {
                            return Err(Error::new(kinds::ParseError(
                                "Unexpected end of component sort core".to_string(),
                            )));
                        }

                        let core_sort_byte = bytes[offset];
                        offset += 1;

                        let core_sort = match core_sort_byte {
                            binary::COMPONENT_CORE_SORT_FUNC => CoreSort::Function,
                            binary::COMPONENT_CORE_SORT_TABLE => CoreSort::Table,
                            binary::COMPONENT_CORE_SORT_MEMORY => CoreSort::Memory,
                            binary::COMPONENT_CORE_SORT_GLOBAL => CoreSort::Global,
                            binary::COMPONENT_CORE_SORT_TYPE => CoreSort::Type,
                            binary::COMPONENT_CORE_SORT_MODULE => CoreSort::Module,
                            binary::COMPONENT_CORE_SORT_INSTANCE => CoreSort::Instance,
                            _ => {
                                return Err(Error::new(kinds::ParseError(format!(
                                    "Invalid core sort kind: {}",
                                    core_sort_byte
                                ))))
                            }
                        };

                        wrt_format::component::Sort::Core(core_sort)
                    }
                    binary::COMPONENT_SORT_FUNC => wrt_format::component::Sort::Function,
                    binary::COMPONENT_SORT_VALUE => wrt_format::component::Sort::Value,
                    binary::COMPONENT_SORT_TYPE => wrt_format::component::Sort::Type,
                    binary::COMPONENT_SORT_COMPONENT => wrt_format::component::Sort::Component,
                    binary::COMPONENT_SORT_INSTANCE => wrt_format::component::Sort::Instance,
                    _ => {
                        return Err(Error::new(kinds::ParseError(format!(
                            "Invalid component sort kind: {}",
                            kind_byte
                        ))))
                    }
                };

                // Read name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                AliasTarget::InstanceExport {
                    instance_idx: component_index,
                    name,
                    kind,
                }
            }
            // Instance export
            0x03 => {
                // Read instance index
                let (instance_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Read kind
                if offset >= bytes.len() {
                    return Err(Error::new(kinds::ParseError(
                        "Unexpected end of data while parsing alias kind".to_string(),
                    )));
                }
                let kind_byte = bytes[offset];
                offset += 1;

                let kind = match kind_byte {
                    binary::COMPONENT_SORT_CORE => {
                        // For core sorts, we need to read the core sort kind
                        if offset >= bytes.len() {
                            return Err(Error::new(kinds::ParseError(
                                "Unexpected end of component sort core".to_string(),
                            )));
                        }

                        let core_sort_byte = bytes[offset];
                        offset += 1;

                        let core_sort = match core_sort_byte {
                            binary::COMPONENT_CORE_SORT_FUNC => CoreSort::Function,
                            binary::COMPONENT_CORE_SORT_TABLE => CoreSort::Table,
                            binary::COMPONENT_CORE_SORT_MEMORY => CoreSort::Memory,
                            binary::COMPONENT_CORE_SORT_GLOBAL => CoreSort::Global,
                            binary::COMPONENT_CORE_SORT_TYPE => CoreSort::Type,
                            binary::COMPONENT_CORE_SORT_MODULE => CoreSort::Module,
                            binary::COMPONENT_CORE_SORT_INSTANCE => CoreSort::Instance,
                            _ => {
                                return Err(Error::new(kinds::ParseError(format!(
                                    "Invalid core sort kind: {}",
                                    core_sort_byte
                                ))))
                            }
                        };

                        wrt_format::component::Sort::Core(core_sort)
                    }
                    binary::COMPONENT_SORT_FUNC => wrt_format::component::Sort::Function,
                    binary::COMPONENT_SORT_VALUE => wrt_format::component::Sort::Value,
                    binary::COMPONENT_SORT_TYPE => wrt_format::component::Sort::Type,
                    binary::COMPONENT_SORT_COMPONENT => wrt_format::component::Sort::Component,
                    binary::COMPONENT_SORT_INSTANCE => wrt_format::component::Sort::Instance,
                    _ => {
                        return Err(Error::new(kinds::ParseError(format!(
                            "Invalid component sort kind: {}",
                            kind_byte
                        ))))
                    }
                };

                // Read name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                AliasTarget::InstanceExport {
                    instance_idx,
                    name,
                    kind,
                }
            }
            // Outer alias
            0x04 => {
                // Read count of outer levels
                let (count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Read kind
                if offset >= bytes.len() {
                    return Err(Error::new(kinds::ParseError(
                        "Unexpected end of data while parsing alias kind".to_string(),
                    )));
                }
                let kind_byte = bytes[offset];
                offset += 1;

                let kind = match kind_byte {
                    binary::COMPONENT_SORT_CORE => {
                        // For core sorts, we need to read the core sort kind
                        if offset >= bytes.len() {
                            return Err(Error::new(kinds::ParseError(
                                "Unexpected end of component sort core".to_string(),
                            )));
                        }

                        let core_sort_byte = bytes[offset];
                        offset += 1;

                        let core_sort = match core_sort_byte {
                            binary::COMPONENT_CORE_SORT_FUNC => CoreSort::Function,
                            binary::COMPONENT_CORE_SORT_TABLE => CoreSort::Table,
                            binary::COMPONENT_CORE_SORT_MEMORY => CoreSort::Memory,
                            binary::COMPONENT_CORE_SORT_GLOBAL => CoreSort::Global,
                            binary::COMPONENT_CORE_SORT_TYPE => CoreSort::Type,
                            binary::COMPONENT_CORE_SORT_MODULE => CoreSort::Module,
                            binary::COMPONENT_CORE_SORT_INSTANCE => CoreSort::Instance,
                            _ => {
                                return Err(Error::new(kinds::ParseError(format!(
                                    "Invalid core sort kind: {}",
                                    core_sort_byte
                                ))))
                            }
                        };

                        wrt_format::component::Sort::Core(core_sort)
                    }
                    binary::COMPONENT_SORT_FUNC => wrt_format::component::Sort::Function,
                    binary::COMPONENT_SORT_VALUE => wrt_format::component::Sort::Value,
                    binary::COMPONENT_SORT_TYPE => wrt_format::component::Sort::Type,
                    binary::COMPONENT_SORT_COMPONENT => wrt_format::component::Sort::Component,
                    binary::COMPONENT_SORT_INSTANCE => wrt_format::component::Sort::Instance,
                    _ => {
                        return Err(Error::new(kinds::ParseError(format!(
                            "Invalid component sort kind: {}",
                            kind_byte
                        ))))
                    }
                };

                // Read index
                let (idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                AliasTarget::Outer { count, kind, idx }
            }
            _ => {
                return Err(Error::new(kinds::ParseError(format!(
                    "Unknown alias kind: {}",
                    alias_kind
                ))));
            }
        };

        aliases.push(Alias { target });
    }

    Ok((aliases, offset))
}

/// Parse the type section
///
/// The type section contains type definitions for component-model types
/// such as function types, value types, and instance types.
fn parse_type_section(bytes: &[u8]) -> Result<(Vec<wrt_format::component::ComponentType>, usize)> {
    let mut offset = 0;
    let mut types = Vec::new();

    // Read the number of types in this section
    let (count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    // Parse each type definition
    for _ in 0..count {
        // Read the type tag
        if offset >= bytes.len() {
            return Err(Error::new(kinds::ParseError(
                "Unexpected end of component type section".to_string(),
            )));
        }

        let tag = bytes[offset];
        offset += 1;

        let definition = match tag {
            // Component type
            0x00 => {
                // Read number of import types
                let (import_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Parse imports
                let mut imports = Vec::with_capacity(import_count as usize);
                for _ in 0..import_count {
                    // Read import namespace
                    let (namespace, bytes_read) = binary::read_string(bytes, offset)?;
                    offset += bytes_read;

                    // Read import name
                    let (name, bytes_read) = binary::read_string(bytes, offset)?;
                    offset += bytes_read;

                    // Read import type
                    let (import_type, bytes_read) = parse_extern_type(bytes, offset)?;
                    offset += bytes_read;

                    imports.push((namespace, name, import_type));
                }

                // Read number of export types
                let (export_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Parse exports
                let mut exports = Vec::with_capacity(export_count as usize);
                for _ in 0..export_count {
                    // Read export name
                    let (name, bytes_read) = binary::read_string(bytes, offset)?;
                    offset += bytes_read;

                    // Read export type
                    let (export_type, bytes_read) = parse_extern_type(bytes, offset)?;
                    offset += bytes_read;

                    exports.push((name, export_type));
                }

                wrt_format::component::ComponentTypeDefinition::Component { imports, exports }
            }
            // Instance type
            0x01 => {
                // Read number of export types
                let (export_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Parse exports
                let mut exports = Vec::with_capacity(export_count as usize);
                for _ in 0..export_count {
                    // Read export name
                    let (name, bytes_read) = binary::read_string(bytes, offset)?;
                    offset += bytes_read;

                    // Read export type
                    let (export_type, bytes_read) = parse_extern_type(bytes, offset)?;
                    offset += bytes_read;

                    exports.push((name, export_type));
                }

                wrt_format::component::ComponentTypeDefinition::Instance { exports }
            }
            // Function type
            0x02 => {
                // Read parameter count
                let (param_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Parse parameter types
                let mut params = Vec::with_capacity(param_count as usize);
                for _ in 0..param_count {
                    // Read parameter name
                    let (name, bytes_read) = binary::read_string(bytes, offset)?;
                    offset += bytes_read;

                    // Read parameter type
                    let (val_type, bytes_read) = parse_val_type(bytes, offset)?;
                    offset += bytes_read;

                    params.push((name, val_type));
                }

                // Read result count
                let (result_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Parse result types
                let mut results = Vec::with_capacity(result_count as usize);
                for _ in 0..result_count {
                    // Read result type
                    let (val_type, bytes_read) = parse_val_type(bytes, offset)?;
                    offset += bytes_read;

                    results.push(val_type);
                }

                wrt_format::component::ComponentTypeDefinition::Function { params, results }
            }
            // Value type
            0x03 => {
                // Read value type
                let (val_type, bytes_read) = parse_val_type(bytes, offset)?;
                offset += bytes_read;

                wrt_format::component::ComponentTypeDefinition::Value(val_type)
            }
            // Resource type
            0x04 => {
                // Read representation type
                if offset >= bytes.len() {
                    return Err(Error::new(kinds::ParseError(
                        "Unexpected end of resource type data".to_string(),
                    )));
                }

                let rep_byte = bytes[offset];
                offset += 1;

                let representation = match rep_byte {
                    // Handle 32
                    0x00 => wrt_format::component::ResourceRepresentation::Handle32,
                    // Handle 64
                    0x01 => wrt_format::component::ResourceRepresentation::Handle64,
                    // Record representation
                    0x02 => {
                        // Read field count
                        let (field_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                        offset += bytes_read;

                        // Parse field names
                        let mut field_names = Vec::with_capacity(field_count as usize);
                        for _ in 0..field_count {
                            // Read field name
                            let (name, bytes_read) = binary::read_string(bytes, offset)?;
                            offset += bytes_read;

                            field_names.push(name);
                        }

                        wrt_format::component::ResourceRepresentation::Record(field_names)
                    }
                    // Aggregate representation
                    0x03 => {
                        // Read index count
                        let (index_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                        offset += bytes_read;

                        // Parse indices
                        let mut indices = Vec::with_capacity(index_count as usize);
                        for _ in 0..index_count {
                            // Read index
                            let (idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                            offset += bytes_read;

                            indices.push(idx);
                        }

                        wrt_format::component::ResourceRepresentation::Aggregate(indices)
                    }
                    _ => {
                        return Err(Error::new(kinds::ParseError(format!(
                            "Invalid resource representation type: {}",
                            rep_byte
                        ))))
                    }
                };

                // Read nullable flag
                if offset >= bytes.len() {
                    return Err(Error::new(kinds::ParseError(
                        "Unexpected end of resource type data".to_string(),
                    )));
                }

                let nullable = bytes[offset] != 0;
                offset += 1;

                wrt_format::component::ComponentTypeDefinition::Resource {
                    representation,
                    nullable,
                }
            }
            _ => {
                return Err(Error::new(kinds::ParseError(format!(
                    "Invalid component type tag: {}",
                    tag
                ))))
            }
        };

        types.push(wrt_format::component::ComponentType { definition });
    }

    Ok((types, offset))
}

/// Parse a component value type
fn parse_val_type(bytes: &[u8], pos: usize) -> Result<(wrt_format::component::ValType, usize)> {
    let mut offset = pos;

    // Make sure we have at least one byte
    if offset >= bytes.len() {
        return Err(Error::new(kinds::ParseError(
            "Unexpected end of value type data".to_string(),
        )));
    }

    let type_byte = bytes[offset];
    offset += 1;

    let val_type = match type_byte {
        // Basic types
        0x7F => wrt_format::component::ValType::Bool,
        0x7E => wrt_format::component::ValType::S8,
        0x7D => wrt_format::component::ValType::U8,
        0x7C => wrt_format::component::ValType::S16,
        0x7B => wrt_format::component::ValType::U16,
        0x7A => wrt_format::component::ValType::S32,
        0x79 => wrt_format::component::ValType::U32,
        0x78 => wrt_format::component::ValType::S64,
        0x77 => wrt_format::component::ValType::U64,
        0x76 => wrt_format::component::ValType::F32,
        0x75 => wrt_format::component::ValType::F64,
        0x74 => wrt_format::component::ValType::Char,
        0x73 => wrt_format::component::ValType::String,

        // Reference type
        0x70 => {
            // Read type index
            let (idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            wrt_format::component::ValType::Ref(idx)
        }

        // Record type
        0x6F => {
            // Read field count
            let (field_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Parse fields
            let mut fields = Vec::with_capacity(field_count as usize);
            for _ in 0..field_count {
                // Read field name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read field type
                let (field_type, bytes_read) = parse_val_type(bytes, offset)?;
                offset += bytes_read;

                fields.push((name, field_type));
            }

            wrt_format::component::ValType::Record(fields)
        }

        // Variant type
        0x6E => {
            // Read case count
            let (case_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Parse cases
            let mut cases = Vec::with_capacity(case_count as usize);
            for _ in 0..case_count {
                // Read case name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read has_type flag
                if offset >= bytes.len() {
                    return Err(Error::new(kinds::ParseError(
                        "Unexpected end of variant case data".to_string(),
                    )));
                }

                let has_type = bytes[offset] != 0;
                offset += 1;

                let case_type = if has_type {
                    // Read case type
                    let (val_type, bytes_read) = parse_val_type(bytes, offset)?;
                    offset += bytes_read;

                    Some(val_type)
                } else {
                    None
                };

                cases.push((name, case_type));
            }

            wrt_format::component::ValType::Variant(cases)
        }

        // List type
        0x6D => {
            // Read element type
            let (element_type, bytes_read) = parse_val_type(bytes, offset)?;
            offset += bytes_read;

            wrt_format::component::ValType::List(Box::new(element_type))
        }

        // Tuple type
        0x6C => {
            // Read element count
            let (element_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Parse elements
            let mut elements = Vec::with_capacity(element_count as usize);
            for _ in 0..element_count {
                // Read element type
                let (element_type, bytes_read) = parse_val_type(bytes, offset)?;
                offset += bytes_read;

                elements.push(element_type);
            }

            wrt_format::component::ValType::Tuple(elements)
        }

        // Flags type
        0x6B => {
            // Read flag count
            let (flag_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Parse flags
            let mut flags = Vec::with_capacity(flag_count as usize);
            for _ in 0..flag_count {
                // Read flag name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                flags.push(name);
            }

            wrt_format::component::ValType::Flags(flags)
        }

        // Enum type
        0x6A => {
            // Read case count
            let (case_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Parse cases
            let mut cases = Vec::with_capacity(case_count as usize);
            for _ in 0..case_count {
                // Read case name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                cases.push(name);
            }

            wrt_format::component::ValType::Enum(cases)
        }

        // Option type
        0x69 => {
            // Read inner type
            let (inner_type, bytes_read) = parse_val_type(bytes, offset)?;
            offset += bytes_read;

            wrt_format::component::ValType::Option(Box::new(inner_type))
        }

        // Result type (ok only)
        0x68 => {
            // Read ok type
            let (ok_type, bytes_read) = parse_val_type(bytes, offset)?;
            offset += bytes_read;

            wrt_format::component::ValType::Result(Box::new(ok_type))
        }

        // Result type (err only)
        0x67 => {
            // Read err type
            let (err_type, bytes_read) = parse_val_type(bytes, offset)?;
            offset += bytes_read;

            wrt_format::component::ValType::ResultErr(Box::new(err_type))
        }

        // Result type (ok and err)
        0x66 => {
            // Read ok type
            let (ok_type, bytes_read) = parse_val_type(bytes, offset)?;
            offset += bytes_read;

            // Read err type
            let (err_type, bytes_read) = parse_val_type(bytes, offset)?;
            offset += bytes_read;

            wrt_format::component::ValType::ResultBoth(Box::new(ok_type), Box::new(err_type))
        }

        // Own type
        0x65 => {
            // Read resource index
            let (idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            wrt_format::component::ValType::Own(idx)
        }

        // Borrow type
        0x64 => {
            // Read resource index
            let (idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            wrt_format::component::ValType::Borrow(idx)
        }

        _ => {
            return Err(Error::new(kinds::ParseError(format!(
                "Invalid value type byte: {}",
                type_byte
            ))))
        }
    };

    Ok((val_type, offset - pos))
}

/// Parse a component external type
fn parse_extern_type(
    bytes: &[u8],
    pos: usize,
) -> Result<(wrt_format::component::ExternType, usize)> {
    let mut offset = pos;

    // Make sure we have at least one byte
    if offset >= bytes.len() {
        return Err(Error::new(kinds::ParseError(
            "Unexpected end of extern type data".to_string(),
        )));
    }

    let type_byte = bytes[offset];
    offset += 1;

    let extern_type = match type_byte {
        // Function extern type
        0x00 => {
            // Read parameter count
            let (param_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Parse parameter types
            let mut params = Vec::with_capacity(param_count as usize);
            for _ in 0..param_count {
                // Read parameter name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read parameter type
                let (val_type, bytes_read) = parse_val_type(bytes, offset)?;
                offset += bytes_read;

                params.push((name, val_type));
            }

            // Read result count
            let (result_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Parse result types
            let mut results = Vec::with_capacity(result_count as usize);
            for _ in 0..result_count {
                // Read result type
                let (val_type, bytes_read) = parse_val_type(bytes, offset)?;
                offset += bytes_read;

                results.push(val_type);
            }

            wrt_format::component::ExternType::Function { params, results }
        }

        // Value extern type
        0x01 => {
            // Read value type
            let (val_type, bytes_read) = parse_val_type(bytes, offset)?;
            offset += bytes_read;

            wrt_format::component::ExternType::Value(val_type)
        }

        // Type reference
        0x02 => {
            // Read type index
            let (idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            wrt_format::component::ExternType::Type(idx)
        }

        // Instance extern type
        0x03 => {
            // Read export count
            let (export_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Parse exports
            let mut exports = Vec::with_capacity(export_count as usize);
            for _ in 0..export_count {
                // Read export name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read export type
                let (export_type, bytes_read) = parse_extern_type(bytes, offset)?;
                offset += bytes_read;

                exports.push((name, export_type));
            }

            wrt_format::component::ExternType::Instance { exports }
        }

        // Component extern type
        0x04 => {
            // Read import count
            let (import_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Parse imports
            let mut imports = Vec::with_capacity(import_count as usize);
            for _ in 0..import_count {
                // Read import namespace
                let (namespace, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read import name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read import type
                let (import_type, bytes_read) = parse_extern_type(bytes, offset)?;
                offset += bytes_read;

                imports.push((namespace, name, import_type));
            }

            // Read export count
            let (export_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Parse exports
            let mut exports = Vec::with_capacity(export_count as usize);
            for _ in 0..export_count {
                // Read export name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read export type
                let (export_type, bytes_read) = parse_extern_type(bytes, offset)?;
                offset += bytes_read;

                exports.push((name, export_type));
            }

            wrt_format::component::ExternType::Component { imports, exports }
        }

        _ => {
            return Err(Error::new(kinds::ParseError(format!(
                "Invalid extern type byte: {}",
                type_byte
            ))))
        }
    };

    Ok((extern_type, offset - pos))
}

/// Parse the canon section
///
/// The canon section contains declarations of canonical function conversions
/// between the host and component ABI.
fn parse_canon_section(bytes: &[u8]) -> Result<(Vec<wrt_format::component::Canon>, usize)> {
    let mut offset = 0;
    let mut canons = Vec::new();

    // Read count of canon operations
    let (count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    for _ in 0..count {
        // Read the operation tag
        if offset >= bytes.len() {
            return Err(Error::new(kinds::ParseError(
                "Unexpected end of canon section".to_string(),
            )));
        }

        let tag = bytes[offset];
        offset += 1;

        let operation = match tag {
            // Lift operation
            0x00 => {
                // Read function index
                let (func_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Read type index
                let (type_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Read options
                // In a full implementation, we would parse memory_idx and string_encoding
                let options = wrt_format::component::LiftOptions {
                    memory_idx: None,
                    string_encoding: None,
                };

                wrt_format::component::CanonOperation::Lift {
                    func_idx,
                    type_idx,
                    options,
                }
            }
            // Lower operation
            0x01 => {
                // Read function index
                let (func_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Read options
                // In a full implementation, we would parse memory_idx and string_encoding
                let options = wrt_format::component::LowerOptions {
                    memory_idx: None,
                    string_encoding: None,
                };

                wrt_format::component::CanonOperation::Lower { func_idx, options }
            }
            // Resource operations (abbreviated implementation)
            0x02 => {
                // In a full implementation, we would parse resource operations
                // For now, use a placeholder
                let resource_op = wrt_format::component::ResourceOperation::New(
                    wrt_format::component::ResourceNew { type_idx: 0 },
                );

                wrt_format::component::CanonOperation::Resource(resource_op)
            }
            _ => {
                return Err(Error::new(kinds::ParseError(format!(
                    "Unknown canon operation tag: {}",
                    tag
                ))));
            }
        };

        canons.push(wrt_format::component::Canon { operation });
    }

    Ok((canons, offset))
}

/// Parse the start section
///
/// The start section contains information about a function that should be
/// automatically executed when the component is instantiated.
fn parse_start_section(bytes: &[u8]) -> Result<(Start, usize)> {
    let mut offset = 0;

    // Read function index
    let (func_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    // Read arguments count
    let (arg_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    // Parse arguments
    let mut args = Vec::with_capacity(arg_count as usize);
    for _ in 0..arg_count {
        // Read argument index
        let (arg_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        args.push(arg_idx);
    }

    // Read result count
    let (results, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    let start = Start {
        func_idx,
        args,
        results,
    };

    Ok((start, offset))
}

/// Parse the import section
///
/// The import section contains a list of imports that the component requires from
/// its host environment. Each import has a name in "namespace:name" format and a type.
fn parse_import_section(bytes: &[u8]) -> Result<(Vec<Import>, usize)> {
    let mut offset = 0;
    let mut imports = Vec::new();

    // Read count of imports
    let (count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    for _ in 0..count {
        // Parse import name - it comes in "namespace:name" format with a 0x00 prefix
        if offset >= bytes.len() || bytes[offset] != 0x00 {
            return Err(Error::new(kinds::ParseError(
                "Invalid import name prefix".to_string(),
            )));
        }

        offset += 1; // Skip the 0x00 prefix

        // Read the length of the name string (namespace:name)
        let (full_name_len, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        if offset + full_name_len as usize > bytes.len() {
            return Err(Error::new(kinds::ParseError(
                "Unexpected end of import name".to_string(),
            )));
        }

        // Extract the full name string
        let full_name_bytes = &bytes[offset..offset + full_name_len as usize];
        offset += full_name_len as usize;

        // Convert to UTF-8 string
        let full_name = match std::str::from_utf8(full_name_bytes) {
            Ok(s) => s.to_string(),
            Err(e) => {
                return Err(Error::new(kinds::ParseError(format!(
                    "Invalid UTF-8 in import name: {}",
                    e
                ))))
            }
        };

        // Split into namespace and name
        let parts: Vec<&str> = full_name.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(Error::new(kinds::ParseError(
                "Import name must be in 'namespace:name' format".to_string(),
            )));
        }

        let import_name = ImportName {
            namespace: parts[0].to_string(),
            name: parts[1].to_string(),
        };

        // For now, we'll just create a placeholder type
        // A full implementation would parse the actual type
        let extern_type = wrt_format::component::ExternType::Value(ValType::Bool);

        let import = Import {
            name: import_name,
            ty: extern_type,
        };

        imports.push(import);
    }

    Ok((imports, offset))
}

/// Parse the export section
///
/// The export section contains a list of exports that the component provides to
/// its host environment. Each export has a name, sort (kind), index, and optional type.
fn parse_export_section(bytes: &[u8]) -> Result<(Vec<Export>, usize)> {
    let mut offset = 0;
    let mut exports = Vec::new();

    // Read count of exports
    let (count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    for _ in 0..count {
        // Read export name
        let (name_data, name_bytes_read) = binary::read_string(bytes, offset)?;
        offset += name_bytes_read;

        // Parse export name details (basic name, resource flag, semver, integrity)
        let mut export_name = ExportName {
            name: name_data.clone(),
            is_resource: false, // Default to not a resource
            semver: None,
            integrity: None,
        };

        // Decode the name to check for annotations
        if name_data.starts_with("resource ") {
            // Resource export
            export_name.is_resource = true;
            export_name.name = name_data[9..].to_string(); // Remove "resource " prefix
        }

        // Check for semver annotation: "$name@$semver"
        if let Some(at_pos) = export_name.name.find('@') {
            // Format appears to have semver, extract it
            let raw_version = &export_name.name[at_pos + 1..];

            // Validate semver format (major.minor.patch)
            if is_valid_semver(raw_version) {
                export_name.semver = Some(raw_version.to_string());
                export_name.name = export_name.name[..at_pos].to_string();
            }
        }

        // Check for integrity hash annotation: "$name?$integrity"
        if let Some(q_pos) = export_name.name.find('?') {
            // Format appears to have integrity hash, extract it
            let raw_integrity = &export_name.name[q_pos + 1..];

            // Basic validation of the integrity hash format
            if is_valid_integrity(raw_integrity) {
                export_name.integrity = Some(raw_integrity.to_string());
                export_name.name = export_name.name[..q_pos].to_string();
            }
        }

        // Read sort kind
        if offset >= bytes.len() {
            return Err(Error::new(kinds::ParseError(
                "Unexpected end of export data".to_string(),
            )));
        }

        let sort_byte = bytes[offset];
        offset += 1;

        let sort = match sort_byte {
            binary::COMPONENT_SORT_CORE => {
                // Core sort - read the core sort kind
                if offset >= bytes.len() {
                    return Err(Error::new(kinds::ParseError(
                        "Unexpected end of export data".to_string(),
                    )));
                }

                let core_sort_byte = bytes[offset];
                offset += 1;

                match core_sort_byte {
                    binary::COMPONENT_CORE_SORT_FUNC => Sort::Core(CoreSort::Function),
                    binary::COMPONENT_CORE_SORT_TABLE => Sort::Core(CoreSort::Table),
                    binary::COMPONENT_CORE_SORT_MEMORY => Sort::Core(CoreSort::Memory),
                    binary::COMPONENT_CORE_SORT_GLOBAL => Sort::Core(CoreSort::Global),
                    binary::COMPONENT_CORE_SORT_TYPE => Sort::Core(CoreSort::Type),
                    binary::COMPONENT_CORE_SORT_MODULE => Sort::Core(CoreSort::Module),
                    binary::COMPONENT_CORE_SORT_INSTANCE => Sort::Core(CoreSort::Instance),
                    _ => {
                        return Err(Error::new(kinds::ParseError(format!(
                            "Invalid core sort kind: {}",
                            core_sort_byte
                        ))))
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
                    "Invalid export sort kind: {}",
                    sort_byte
                ))))
            }
        };

        // Read index within the sort
        let (idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        // Check for optional type declaration
        let ty = if offset < bytes.len() && bytes[offset] != 0 {
            // There is an explicit type declaration
            offset += 1; // Skip the "has type" flag

            // Parse the extern type
            let (extern_type, bytes_read) = parse_extern_type(bytes, offset)?;
            offset += bytes_read;

            Some(extern_type)
        } else if offset < bytes.len() {
            // There's a byte, but it's 0, indicating no type
            offset += 1;
            None
        } else {
            // No more bytes, so there can't be a type
            None
        };

        let export = Export {
            name: export_name,
            sort,
            idx,
            ty,
        };

        exports.push(export);
    }

    Ok((exports, offset))
}

/// Check if a string represents a valid SemVer version
fn is_valid_semver(version: &str) -> bool {
    // Simple SemVer validation (major.minor.patch format)
    let parts: Vec<&str> = version.split('.').collect();

    // Must have 3 parts: major.minor.patch
    if parts.len() != 3 {
        return false;
    }

    // Each part must be a valid number
    for part in parts {
        if part.is_empty() || !part.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
    }

    true
}

/// Check if a string represents a valid integrity hash
fn is_valid_integrity(integrity: &str) -> bool {
    // Simple integrity hash validation
    // Format should be "algo-base64hash" - like "sha256-a1b2c3d4..."

    let parts: Vec<&str> = integrity.split('-').collect();

    // Must have 2 parts: algorithm-hash
    if parts.len() != 2 {
        return false;
    }

    // First part should be a valid hash algorithm
    let algo = parts[0];
    let valid_algos = ["sha256", "sha384", "sha512"];
    if !valid_algos.contains(&algo) {
        return false;
    }

    // Second part should be a base64-looking string
    let hash = parts[1];
    if hash.is_empty() {
        return false;
    }

    // Valid base64 characters are A-Z, a-z, 0-9, +, /, and = (padding)
    for c in hash.chars() {
        if !c.is_ascii_alphanumeric() && c != '+' && c != '/' && c != '=' {
            return false;
        }
    }

    true
}

/// Parse the value section
fn parse_value_section(bytes: &[u8]) -> Result<(Vec<Value>, usize)> {
    let mut offset = 0;
    let mut values = Vec::new();

    // Read count of values
    let (count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    for _ in 0..count {
        // Parse value type
        let (val_type, bytes_read) = parse_val_type(bytes, offset)?;
        offset += bytes_read;

        // Parse value length
        let (value_len, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        // Parse value data
        if offset + value_len as usize > bytes.len() {
            return Err(Error::new(kinds::ParseError(
                "Unexpected end of value data".to_string(),
            )));
        }

        // Extract the raw bytes
        let value_bytes = &bytes[offset..(offset + value_len as usize)];
        offset += value_len as usize;

        // Parse the value based on its type
        let (decoded_data, _) = decode_value(&val_type, value_bytes, 0)?;

        let value = Value {
            ty: val_type,
            data: value_bytes.to_vec(), // Store the raw bytes
        };

        values.push(value);
    }

    Ok((values, offset))
}

/// Decode a value from bytes according to its type
fn decode_value(val_type: &ValType, bytes: &[u8], pos: usize) -> Result<(Vec<u8>, usize)> {
    let mut offset = pos;

    match val_type {
        // Boolean
        ValType::Bool => {
            if offset >= bytes.len() {
                return Err(Error::new(kinds::ParseError(
                    "Unexpected end of boolean value data".to_string(),
                )));
            }

            let value = bytes[offset];
            if value != 0 && value != 1 {
                return Err(Error::new(kinds::ParseError(format!(
                    "Invalid boolean value: {}",
                    value
                ))));
            }

            offset += 1;
            Ok((vec![value], offset - pos))
        }

        // Integer types
        ValType::S8 | ValType::U8 => {
            if offset >= bytes.len() {
                return Err(Error::new(kinds::ParseError(
                    "Unexpected end of 8-bit value data".to_string(),
                )));
            }

            let value = bytes[offset];
            offset += 1;
            Ok((vec![value], offset - pos))
        }

        ValType::S16 | ValType::U16 => {
            if offset + 2 > bytes.len() {
                return Err(Error::new(kinds::ParseError(
                    "Unexpected end of 16-bit value data".to_string(),
                )));
            }

            let value = &bytes[offset..offset + 2];
            offset += 2;
            Ok((value.to_vec(), offset - pos))
        }

        ValType::S32 | ValType::U32 => {
            if offset + 4 > bytes.len() {
                return Err(Error::new(kinds::ParseError(
                    "Unexpected end of 32-bit value data".to_string(),
                )));
            }

            let value = &bytes[offset..offset + 4];
            offset += 4;
            Ok((value.to_vec(), offset - pos))
        }

        ValType::S64 | ValType::U64 => {
            if offset + 8 > bytes.len() {
                return Err(Error::new(kinds::ParseError(
                    "Unexpected end of 64-bit value data".to_string(),
                )));
            }

            let value = &bytes[offset..offset + 8];
            offset += 8;
            Ok((value.to_vec(), offset - pos))
        }

        // Floating point types
        ValType::F32 => {
            if offset + 4 > bytes.len() {
                return Err(Error::new(kinds::ParseError(
                    "Unexpected end of f32 value data".to_string(),
                )));
            }

            let value = &bytes[offset..offset + 4];
            offset += 4;
            Ok((value.to_vec(), offset - pos))
        }

        ValType::F64 => {
            if offset + 8 > bytes.len() {
                return Err(Error::new(kinds::ParseError(
                    "Unexpected end of f64 value data".to_string(),
                )));
            }

            let value = &bytes[offset..offset + 8];
            offset += 8;
            Ok((value.to_vec(), offset - pos))
        }

        // Character
        ValType::Char => {
            // Char is encoded as a UTF-8 sequence
            if offset >= bytes.len() {
                return Err(Error::new(kinds::ParseError(
                    "Unexpected end of char value data".to_string(),
                )));
            }

            // Determine the length of the UTF-8 sequence
            let first_byte = bytes[offset];
            let char_len = if first_byte & 0x80 == 0 {
                1 // ASCII
            } else if first_byte & 0xE0 == 0xC0 {
                2 // 2-byte UTF-8
            } else if first_byte & 0xF0 == 0xE0 {
                3 // 3-byte UTF-8
            } else if first_byte & 0xF8 == 0xF0 {
                4 // 4-byte UTF-8
            } else {
                return Err(Error::new(kinds::ParseError(format!(
                    "Invalid UTF-8 sequence start byte: {}",
                    first_byte
                ))));
            };

            if offset + char_len > bytes.len() {
                return Err(Error::new(kinds::ParseError(
                    "Unexpected end of char value data".to_string(),
                )));
            }

            // Validate the UTF-8 sequence
            let char_bytes = &bytes[offset..offset + char_len];
            let _ = std::str::from_utf8(char_bytes).map_err(|e| {
                Error::new(kinds::ParseError(format!("Invalid UTF-8 sequence: {}", e)))
            })?;

            offset += char_len;
            Ok((char_bytes.to_vec(), offset - pos))
        }

        // String
        ValType::String => {
            // Validate the string is proper UTF-8
            let str_bytes = &bytes[offset..];
            let _ = std::str::from_utf8(str_bytes).map_err(|e| {
                Error::new(kinds::ParseError(format!("Invalid UTF-8 string: {}", e)))
            })?;

            offset += str_bytes.len();
            Ok((str_bytes.to_vec(), offset - pos))
        }

        // Reference
        ValType::Ref(_) => {
            // Type index reference - encoded as a u32
            let (idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;
            Ok((binary::write_leb128_u32(idx), offset - pos))
        }

        // Record
        ValType::Record(fields) => {
            let mut result = Vec::new();

            for (_, field_type) in fields {
                let (field_value, bytes_read) = decode_value(field_type, bytes, offset)?;
                offset += bytes_read;
                result.extend_from_slice(&field_value);
            }

            Ok((result, offset - pos))
        }

        // Variant
        ValType::Variant(cases) => {
            // Read the case index
            let (case_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            if (case_idx as usize) >= cases.len() {
                return Err(Error::new(kinds::ParseError(format!(
                    "Invalid variant case index: {}",
                    case_idx
                ))));
            }

            let mut result = binary::write_leb128_u32(case_idx);

            // Read the case payload if it has one
            if let Some(case_type) = &cases[case_idx as usize].1 {
                let (payload, bytes_read) = decode_value(case_type, bytes, offset)?;
                offset += bytes_read;
                result.extend_from_slice(&payload);
            }

            Ok((result, offset - pos))
        }

        // List
        ValType::List(element_type) => {
            // Read the list length
            let (length, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut result = binary::write_leb128_u32(length);

            // Read each element
            for _ in 0..length {
                let (element, bytes_read) = decode_value(element_type, bytes, offset)?;
                offset += bytes_read;
                result.extend_from_slice(&element);
            }

            Ok((result, offset - pos))
        }

        // Tuple
        ValType::Tuple(elements) => {
            let mut result = Vec::new();

            for element_type in elements {
                let (element, bytes_read) = decode_value(element_type, bytes, offset)?;
                offset += bytes_read;
                result.extend_from_slice(&element);
            }

            Ok((result, offset - pos))
        }

        // Flags
        ValType::Flags(labels) => {
            // Flags are encoded as a sequence of bytes, with each bit representing a flag
            let num_bytes = labels.len().div_ceil(8);

            if offset + num_bytes > bytes.len() {
                return Err(Error::new(kinds::ParseError(
                    "Unexpected end of flags value data".to_string(),
                )));
            }

            let flag_bytes = &bytes[offset..offset + num_bytes];
            offset += num_bytes;

            Ok((flag_bytes.to_vec(), offset - pos))
        }

        // Enum
        ValType::Enum(cases) => {
            // Read the case index
            let (case_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            if (case_idx as usize) >= cases.len() {
                return Err(Error::new(kinds::ParseError(format!(
                    "Invalid enum case index: {}",
                    case_idx
                ))));
            }

            Ok((binary::write_leb128_u32(case_idx), offset - pos))
        }

        // Option
        ValType::Option(inner_type) => {
            // Read the option tag
            if offset >= bytes.len() {
                return Err(Error::new(kinds::ParseError(
                    "Unexpected end of option value data".to_string(),
                )));
            }

            let tag = bytes[offset];
            offset += 1;
            let mut result = vec![tag];

            if tag == 0 {
                // None case
            } else if tag == 1 {
                // Some case - read the inner value
                let (inner_value, bytes_read) = decode_value(inner_type, bytes, offset)?;
                offset += bytes_read;
                result.extend_from_slice(&inner_value);
            } else {
                return Err(Error::new(kinds::ParseError(format!(
                    "Invalid option tag: {}",
                    tag
                ))));
            }

            Ok((result, offset - pos))
        }

        // Result types
        ValType::Result(ok_type) => {
            // Read the result tag
            if offset >= bytes.len() {
                return Err(Error::new(kinds::ParseError(
                    "Unexpected end of result value data".to_string(),
                )));
            }

            let tag = bytes[offset];
            offset += 1;
            let mut result = vec![tag];

            if tag == 0 {
                // Ok case - read the ok value
                let (ok_value, bytes_read) = decode_value(ok_type, bytes, offset)?;
                offset += bytes_read;
                result.extend_from_slice(&ok_value);
            } else if tag == 1 {
                // Error case with no payload
            } else {
                return Err(Error::new(kinds::ParseError(format!(
                    "Invalid result tag: {}",
                    tag
                ))));
            }

            Ok((result, offset - pos))
        }

        ValType::ResultErr(err_type) => {
            // Read the result tag
            if offset >= bytes.len() {
                return Err(Error::new(kinds::ParseError(
                    "Unexpected end of result value data".to_string(),
                )));
            }

            let tag = bytes[offset];
            offset += 1;
            let mut result = vec![tag];

            if tag == 0 {
                // Ok case with no payload
            } else if tag == 1 {
                // Error case - read the error value
                let (err_value, bytes_read) = decode_value(err_type, bytes, offset)?;
                offset += bytes_read;
                result.extend_from_slice(&err_value);
            } else {
                return Err(Error::new(kinds::ParseError(format!(
                    "Invalid result tag: {}",
                    tag
                ))));
            }

            Ok((result, offset - pos))
        }

        ValType::ResultBoth(ok_type, err_type) => {
            // Read the result tag
            if offset >= bytes.len() {
                return Err(Error::new(kinds::ParseError(
                    "Unexpected end of result value data".to_string(),
                )));
            }

            let tag = bytes[offset];
            offset += 1;
            let mut result = vec![tag];

            if tag == 0 {
                // Ok case - read the ok value
                let (ok_value, bytes_read) = decode_value(ok_type, bytes, offset)?;
                offset += bytes_read;
                result.extend_from_slice(&ok_value);
            } else if tag == 1 {
                // Error case - read the error value
                let (err_value, bytes_read) = decode_value(err_type, bytes, offset)?;
                offset += bytes_read;
                result.extend_from_slice(&err_value);
            } else {
                return Err(Error::new(kinds::ParseError(format!(
                    "Invalid result tag: {}",
                    tag
                ))));
            }

            Ok((result, offset - pos))
        }

        // Resource types
        ValType::Own(idx) => {
            // Resource handle - encoded as a u32
            let (handle, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;
            Ok((binary::write_leb128_u32(handle), offset - pos))
        }

        ValType::Borrow(idx) => {
            // Resource handle - encoded as a u32
            let (handle, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;
            Ok((binary::write_leb128_u32(handle), offset - pos))
        }
    }
}

// Placeholder encoding functions
fn encode_core_module_section(modules: &[Module]) -> Result<Vec<u8>> {
    // Placeholder - would need actual implementation
    let mut result = Vec::new();

    // Write the number of modules
    result.extend_from_slice(&binary::write_leb128_u32(modules.len() as u32));

    // For each module, we would encode it
    // This is just a placeholder
    Ok(result)
}

fn encode_core_instance_section(instances: &[CoreInstance]) -> Result<Vec<u8>> {
    // Placeholder - would need actual implementation
    let mut result = Vec::new();

    // Write the number of instances
    result.extend_from_slice(&binary::write_leb128_u32(instances.len() as u32));

    // For each instance, we would encode it
    // This is just a placeholder
    Ok(result)
}

fn encode_core_type_section(types: &[CoreType]) -> Result<Vec<u8>> {
    // Placeholder - would need actual implementation
    let mut result = Vec::new();

    // Write the number of types
    result.extend_from_slice(&binary::write_leb128_u32(types.len() as u32));

    // For each type, we would encode it
    // This is just a placeholder
    Ok(result)
}

// Other placeholder functions would be similarly implemented
fn encode_component_section(components: &[Component]) -> Result<Vec<u8>> {
    unimplemented!("encode_component_section not implemented")
}

fn encode_instance_section(instances: &[Instance]) -> Result<Vec<u8>> {
    unimplemented!("encode_instance_section not implemented")
}

fn encode_alias_section(aliases: &[Alias]) -> Result<Vec<u8>> {
    unimplemented!("encode_alias_section not implemented")
}

fn encode_type_section(types: &[wrt_format::component::ComponentType]) -> Result<Vec<u8>> {
    unimplemented!("encode_type_section not implemented")
}

fn encode_canon_section(canons: &[wrt_format::component::Canon]) -> Result<Vec<u8>> {
    unimplemented!("encode_canon_section not implemented")
}

fn encode_start_section(start: &Start) -> Result<Vec<u8>> {
    unimplemented!("encode_start_section not implemented")
}

fn encode_import_section(imports: &[Import]) -> Result<Vec<u8>> {
    unimplemented!("encode_import_section not implemented")
}

fn encode_export_section(exports: &[Export]) -> Result<Vec<u8>> {
    unimplemented!("encode_export_section not implemented")
}

fn encode_value_section(values: &[Value]) -> Result<Vec<u8>> {
    let mut result = Vec::new();

    // Write the number of values
    result.extend_from_slice(&binary::write_leb128_u32(values.len() as u32));

    // For each value, encode its type and data
    for value in values {
        // Encode the value type
        let type_encoding = encode_val_type(&value.ty)?;
        result.extend_from_slice(&type_encoding);

        // Encode the value length and data
        result.extend_from_slice(&binary::write_leb128_u32(value.data.len() as u32));
        result.extend_from_slice(&value.data);
    }

    Ok(result)
}

/// Encode a value type to binary format
fn encode_val_type(val_type: &wrt_format::component::ValType) -> Result<Vec<u8>> {
    let mut result = Vec::new();

    match val_type {
        // Basic types
        wrt_format::component::ValType::Bool => result.push(0x7F),
        wrt_format::component::ValType::S8 => result.push(0x7E),
        wrt_format::component::ValType::U8 => result.push(0x7D),
        wrt_format::component::ValType::S16 => result.push(0x7C),
        wrt_format::component::ValType::U16 => result.push(0x7B),
        wrt_format::component::ValType::S32 => result.push(0x7A),
        wrt_format::component::ValType::U32 => result.push(0x79),
        wrt_format::component::ValType::S64 => result.push(0x78),
        wrt_format::component::ValType::U64 => result.push(0x77),
        wrt_format::component::ValType::F32 => result.push(0x76),
        wrt_format::component::ValType::F64 => result.push(0x75),
        wrt_format::component::ValType::Char => result.push(0x74),
        wrt_format::component::ValType::String => result.push(0x73),

        // Reference type
        wrt_format::component::ValType::Ref(idx) => {
            result.push(0x70);
            result.extend_from_slice(&binary::write_leb128_u32(*idx));
        }

        // Record type
        wrt_format::component::ValType::Record(fields) => {
            result.push(0x6F);
            result.extend_from_slice(&binary::write_leb128_u32(fields.len() as u32));

            for (name, field_type) in fields {
                // Write field name
                let name_bytes = binary::write_string(name);
                result.extend_from_slice(&name_bytes);

                // Write field type
                let type_bytes = encode_val_type(field_type)?;
                result.extend_from_slice(&type_bytes);
            }
        }

        // Variant type
        wrt_format::component::ValType::Variant(cases) => {
            result.push(0x6E);
            result.extend_from_slice(&binary::write_leb128_u32(cases.len() as u32));

            for (name, case_type) in cases {
                // Write case name
                let name_bytes = binary::write_string(name);
                result.extend_from_slice(&name_bytes);

                // Write has_type flag and case type if present
                if let Some(case_type) = case_type {
                    result.push(0x01); // has_type = true
                    let type_bytes = encode_val_type(case_type)?;
                    result.extend_from_slice(&type_bytes);
                } else {
                    result.push(0x00); // has_type = false
                }
            }
        }

        // List type
        wrt_format::component::ValType::List(element_type) => {
            result.push(0x6D);
            let type_bytes = encode_val_type(element_type)?;
            result.extend_from_slice(&type_bytes);
        }

        // Tuple type
        wrt_format::component::ValType::Tuple(elements) => {
            result.push(0x6C);
            result.extend_from_slice(&binary::write_leb128_u32(elements.len() as u32));

            for element_type in elements {
                let type_bytes = encode_val_type(element_type)?;
                result.extend_from_slice(&type_bytes);
            }
        }

        // Flags type
        wrt_format::component::ValType::Flags(flags) => {
            result.push(0x6B);
            result.extend_from_slice(&binary::write_leb128_u32(flags.len() as u32));

            for flag in flags {
                let flag_bytes = binary::write_string(flag);
                result.extend_from_slice(&flag_bytes);
            }
        }

        // Enum type
        wrt_format::component::ValType::Enum(cases) => {
            result.push(0x6A);
            result.extend_from_slice(&binary::write_leb128_u32(cases.len() as u32));

            for case in cases {
                let case_bytes = binary::write_string(case);
                result.extend_from_slice(&case_bytes);
            }
        }

        // Option type
        wrt_format::component::ValType::Option(inner_type) => {
            result.push(0x69);
            let type_bytes = encode_val_type(inner_type)?;
            result.extend_from_slice(&type_bytes);
        }

        // Result type (ok only)
        wrt_format::component::ValType::Result(ok_type) => {
            result.push(0x68);
            let type_bytes = encode_val_type(ok_type)?;
            result.extend_from_slice(&type_bytes);
        }

        // Result type (err only)
        wrt_format::component::ValType::ResultErr(err_type) => {
            result.push(0x67);
            let type_bytes = encode_val_type(err_type)?;
            result.extend_from_slice(&type_bytes);
        }

        // Result type (ok and err)
        wrt_format::component::ValType::ResultBoth(ok_type, err_type) => {
            result.push(0x66);
            let ok_bytes = encode_val_type(ok_type)?;
            result.extend_from_slice(&ok_bytes);
            let err_bytes = encode_val_type(err_type)?;
            result.extend_from_slice(&err_bytes);
        }

        // Own type
        wrt_format::component::ValType::Own(idx) => {
            result.push(0x65);
            result.extend_from_slice(&binary::write_leb128_u32(*idx));
        }

        // Borrow type
        wrt_format::component::ValType::Borrow(idx) => {
            result.push(0x64);
            result.extend_from_slice(&binary::write_leb128_u32(*idx));
        }
    }

    Ok(result)
}
