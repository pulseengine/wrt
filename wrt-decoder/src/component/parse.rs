// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use wrt_error::{kinds, Error, Result};
use wrt_format::{
    binary,
    component::{
        Alias, Canon, Component, ComponentType, CoreInstance, CoreType, Export, Import, Instance,
        Start, Value,
    },
    module::Module,
};
use wrt_types::resource;

use crate::prelude::*;

// Define a macro for conditionally selecting format based on environment
#[cfg(feature = "std")]
macro_rules! env_format {
    ($($arg:tt)*) => {
        format!($($arg)*)
    };
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
macro_rules! env_format {
    ($($arg:tt)*) => {
        alloc::format!($($arg)*)
    };
}

#[cfg(not(any(feature = "std", feature = "alloc")))]
macro_rules! env_format {
    ($($arg:tt)*) => {
        // For environments without formatting capabilities,
        // create a static string (this is not ideal but provides a fallback)
        "format error (no formatting available in this environment)"
    };
}

// Define a helper function for converting format strings to String
fn format_to_string(message: &str, value: impl core::fmt::Display) -> String {
    #[cfg(feature = "std")]
    {
        format!("{}: {}", message, value)
    }

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    {
        alloc::format!("{}: {}", message, value)
    }

    #[cfg(not(any(feature = "std", feature = "alloc")))]
    {
        let mut s = String::new();
        s.push_str(message);
        s.push_str(": ");
        // For no_std without alloc, we'll just append a placeholder
        // since we can't use the Display trait without allocation
        s.push_str("[value]");
        s
    }
}

/// Parse a core module section
pub fn parse_core_module_section(bytes: &[u8]) -> Result<(Vec<Module>, usize)> {
    // Read a vector of modules
    let (count, mut offset) = binary::read_leb128_u32(bytes, 0)?;
    let mut modules = Vec::with_capacity(count as usize);

    for _ in 0..count {
        // Read a module binary size
        let (module_size, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        if offset + module_size as usize > bytes.len() {
            return Err(Error::parse_error_from_kind(kinds::ParseError(
                "Module size exceeds section size".to_string(),
            )));
        }

        // Extract the module binary
        let module_end = offset + module_size as usize;
        let module_bytes = &bytes[offset..module_end];

        // Parse the module binary
        let module = binary::parse_binary(module_bytes)?;
        modules.push(module);

        offset = module_end;
    }

    Ok((modules, offset))
}

/// Parse a core instance section
pub fn parse_core_instance_section(bytes: &[u8]) -> Result<(Vec<CoreInstance>, usize)> {
    // Read a vector of core instances
    let (count, mut offset) = binary::read_leb128_u32(bytes, 0)?;
    let mut instances = Vec::with_capacity(count as usize);

    for _ in 0..count {
        // Parse the instance expression
        let (instance_expr, bytes_read) = parse_core_instance_expr(&bytes[offset..])?;
        offset += bytes_read;

        // Create the instance
        instances.push(CoreInstance { instance_expr });
    }

    Ok((instances, offset))
}

/// Parse a core instance expression
fn parse_core_instance_expr(
    bytes: &[u8],
) -> Result<(wrt_format::component::CoreInstanceExpr, usize)> {
    if bytes.is_empty() {
        return Err(Error::parse_error_from_kind(kinds::ParseError(
            "Unexpected end of input while parsing core instance expression".to_string(),
        )));
    }

    // Read the expression tag
    let tag = bytes[0];
    let mut offset = 1;

    match tag {
        0x00 => {
            // Instantiate a module
            let (module_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Read argument vector
            let (args_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut args = Vec::with_capacity(args_count as usize);
            for _ in 0..args_count {
                // Read name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read instance index
                let (instance_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                args.push(wrt_format::component::CoreInstantiateArg { name, instance_idx });
            }

            Ok((wrt_format::component::CoreInstanceExpr::Instantiate { module_idx, args }, offset))
        }
        0x01 => {
            // Inline exports
            let (exports_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut exports = Vec::with_capacity(exports_count as usize);
            for _ in 0..exports_count {
                // Read name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read kind byte
                if offset >= bytes.len() {
                    return Err(Error::parse_error_from_kind(kinds::ParseError(
                        "Unexpected end of input while parsing export kind".to_string(),
                    )));
                }
                let kind_byte = bytes[offset];
                offset += 1;

                // Convert to CoreSort
                let sort = match kind_byte {
                    binary::COMPONENT_CORE_SORT_FUNC => wrt_format::component::CoreSort::Function,
                    binary::COMPONENT_CORE_SORT_TABLE => wrt_format::component::CoreSort::Table,
                    binary::COMPONENT_CORE_SORT_MEMORY => wrt_format::component::CoreSort::Memory,
                    binary::COMPONENT_CORE_SORT_GLOBAL => wrt_format::component::CoreSort::Global,
                    binary::COMPONENT_CORE_SORT_TYPE => wrt_format::component::CoreSort::Type,
                    binary::COMPONENT_CORE_SORT_MODULE => wrt_format::component::CoreSort::Module,
                    binary::COMPONENT_CORE_SORT_INSTANCE => {
                        wrt_format::component::CoreSort::Instance
                    }
                    _ => {
                        return Err(Error::parse_error_from_kind(kinds::ParseError(
                            format_to_string("Invalid core sort kind", kind_byte),
                        )));
                    }
                };

                // Read index
                let (idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                exports.push(wrt_format::component::CoreInlineExport { name, sort, idx });
            }

            Ok((wrt_format::component::CoreInstanceExpr::InlineExports(exports), offset))
        }
        _ => Err(Error::parse_error_from_kind(kinds::ParseError(env_format!(
            "Invalid core instance expression tag: {:#x}",
            tag
        )))),
    }
}

/// Parse a core type section
pub fn parse_core_type_section(bytes: &[u8]) -> Result<(Vec<CoreType>, usize)> {
    // Read a vector of core types
    let (count, mut offset) = binary::read_leb128_u32(bytes, 0)?;
    let mut types = Vec::with_capacity(count as usize);

    for _ in 0..count {
        // Read type definition
        let (definition, bytes_read) = parse_core_type_definition(&bytes[offset..])?;
        offset += bytes_read;

        // Create the type
        types.push(CoreType { definition });
    }

    Ok((types, offset))
}

/// Parse a core type definition
fn parse_core_type_definition(
    bytes: &[u8],
) -> Result<(wrt_format::component::CoreTypeDefinition, usize)> {
    if bytes.is_empty() {
        return Err(Error::parse_error_from_kind(kinds::ParseError(
            "Unexpected end of input while parsing core type definition".to_string(),
        )));
    }

    // Read the form tag
    let form = bytes[0];
    let mut offset = 1;

    match form {
        0x60 => {
            // Function type

            // Read parameter types
            let (param_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut params = Vec::with_capacity(param_count as usize);
            for _ in 0..param_count {
                // Read value type
                if offset >= bytes.len() {
                    return Err(Error::parse_error_from_kind(kinds::ParseError(
                        "Unexpected end of input while parsing function parameter type".to_string(),
                    )));
                }

                let vtype = match bytes[offset] {
                    binary::I32_TYPE => wrt_format::types::ValueType::I32,
                    binary::I64_TYPE => wrt_format::types::ValueType::I64,
                    binary::F32_TYPE => wrt_format::types::ValueType::F32,
                    binary::F64_TYPE => wrt_format::types::ValueType::F64,
                    binary::V128_TYPE => wrt_format::types::ValueType::ExternRef,
                    binary::FUNCREF_TYPE => wrt_format::types::ValueType::FuncRef,
                    binary::EXTERNREF_TYPE => wrt_format::types::ValueType::ExternRef,
                    _ => {
                        return Err(Error::parse_error_from_kind(kinds::ParseError(
                            format_to_string("Invalid value type", bytes[offset]),
                        )));
                    }
                };

                params.push(vtype);
                offset += 1;
            }

            // Read result types
            let (result_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut results = Vec::with_capacity(result_count as usize);
            for _ in 0..result_count {
                // Read value type
                if offset >= bytes.len() {
                    return Err(Error::parse_error_from_kind(kinds::ParseError(
                        "Unexpected end of input while parsing function result type".to_string(),
                    )));
                }

                let vtype = match bytes[offset] {
                    binary::I32_TYPE => wrt_format::types::ValueType::I32,
                    binary::I64_TYPE => wrt_format::types::ValueType::I64,
                    binary::F32_TYPE => wrt_format::types::ValueType::F32,
                    binary::F64_TYPE => wrt_format::types::ValueType::F64,
                    binary::V128_TYPE => wrt_format::types::ValueType::ExternRef,
                    binary::FUNCREF_TYPE => wrt_format::types::ValueType::FuncRef,
                    binary::EXTERNREF_TYPE => wrt_format::types::ValueType::ExternRef,
                    _ => {
                        return Err(Error::parse_error_from_kind(kinds::ParseError(
                            format_to_string("Invalid value type", bytes[offset]),
                        )));
                    }
                };

                results.push(vtype);
                offset += 1;
            }

            Ok((wrt_format::component::CoreTypeDefinition::Function { params, results }, offset))
        }
        0x61 => {
            // Module type

            // Read import vector
            let (import_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut imports = Vec::with_capacity(import_count as usize);
            for _ in 0..import_count {
                // Read module name
                let (module_name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read field name
                let (field_name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read import type
                let (import_type, bytes_read) = parse_core_extern_type(&bytes[offset..])?;
                offset += bytes_read;

                imports.push((module_name, field_name, import_type));
            }

            // Read export vector
            let (export_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut exports = Vec::with_capacity(export_count as usize);
            for _ in 0..export_count {
                // Read export name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read export type
                let (export_type, bytes_read) = parse_core_extern_type(&bytes[offset..])?;
                offset += bytes_read;

                exports.push((name, export_type));
            }

            Ok((wrt_format::component::CoreTypeDefinition::Module { imports, exports }, offset))
        }
        _ => Err(Error::parse_error_from_kind(kinds::ParseError(format!(
            "Invalid core type form: {:#x}",
            form
        )))),
    }
}

/// Parse a core external type
fn parse_core_extern_type(bytes: &[u8]) -> Result<(wrt_format::component::CoreExternType, usize)> {
    if bytes.is_empty() {
        return Err(Error::parse_error_from_kind(kinds::ParseError(
            "Unexpected end of input while parsing core external type".to_string(),
        )));
    }

    // Read the type tag
    let tag = bytes[0];
    let mut offset = 1;

    match tag {
        0x00 => {
            // Function type

            // Read type index
            let (_type_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Function types are referenced by index, but we need to inline the
            // params/results In a real implementation, this would look up the
            // type in the type section For now, we'll just return an empty
            // function type
            Ok((
                wrt_format::component::CoreExternType::Function {
                    params: Vec::new(),
                    results: Vec::new(),
                },
                offset,
            ))
        }
        0x01 => {
            // Table type

            // Read element type
            if offset >= bytes.len() {
                return Err(Error::parse_error_from_kind(kinds::ParseError(
                    "Unexpected end of input while parsing table element type".to_string(),
                )));
            }

            let element_type = match bytes[offset] {
                binary::FUNCREF_TYPE => wrt_format::types::ValueType::FuncRef,
                binary::EXTERNREF_TYPE => wrt_format::types::ValueType::ExternRef,
                _ => {
                    return Err(Error::parse_error_from_kind(kinds::ParseError(format_to_string(
                        "Invalid table element type",
                        bytes[offset],
                    ))));
                }
            };
            offset += 1;

            // Read limits
            if offset >= bytes.len() {
                return Err(Error::parse_error_from_kind(kinds::ParseError(
                    "Unexpected end of input while parsing table limits".to_string(),
                )));
            }

            let limit_flag = bytes[offset];
            offset += 1;

            // Read minimum size
            let (min, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Read maximum size (if present)
            let max = if limit_flag & 0x01 != 0 {
                let (max, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;
                Some(max)
            } else {
                None
            };

            Ok((wrt_format::component::CoreExternType::Table { element_type, min, max }, offset))
        }
        0x02 => {
            // Memory type

            // Read limits
            if offset >= bytes.len() {
                return Err(Error::parse_error_from_kind(kinds::ParseError(
                    "Unexpected end of input while parsing memory limits".to_string(),
                )));
            }

            let limit_flag = bytes[offset];
            offset += 1;

            // Check if memory is shared
            let shared = (limit_flag & 0x02) != 0;

            // Read minimum size
            let (min, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Read maximum size (if present)
            let max = if limit_flag & 0x01 != 0 {
                let (max, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;
                Some(max)
            } else {
                None
            };

            Ok((wrt_format::component::CoreExternType::Memory { min, max, shared }, offset))
        }
        0x03 => {
            // Global type

            // Read value type
            if offset >= bytes.len() {
                return Err(Error::parse_error_from_kind(kinds::ParseError(
                    "Unexpected end of input while parsing global value type".to_string(),
                )));
            }

            let value_type = match bytes[offset] {
                binary::I32_TYPE => wrt_format::types::ValueType::I32,
                binary::I64_TYPE => wrt_format::types::ValueType::I64,
                binary::F32_TYPE => wrt_format::types::ValueType::F32,
                binary::F64_TYPE => wrt_format::types::ValueType::F64,
                binary::V128_TYPE => wrt_format::types::ValueType::ExternRef,
                binary::FUNCREF_TYPE => wrt_format::types::ValueType::FuncRef,
                binary::EXTERNREF_TYPE => wrt_format::types::ValueType::ExternRef,
                _ => {
                    return Err(Error::parse_error_from_kind(kinds::ParseError(format_to_string(
                        "Invalid global value type",
                        bytes[offset],
                    ))));
                }
            };
            offset += 1;

            // Read mutability flag
            if offset >= bytes.len() {
                return Err(Error::parse_error_from_kind(kinds::ParseError(
                    "Unexpected end of input while parsing global mutability".to_string(),
                )));
            }

            let mutable = bytes[offset] != 0;
            offset += 1;

            Ok((wrt_format::component::CoreExternType::Global { value_type, mutable }, offset))
        }
        _ => Err(Error::parse_error_from_kind(kinds::ParseError(format!(
            "Invalid core external type tag: {:#x}",
            tag
        )))),
    }
}

/// Parse a component section
pub fn parse_component_section(bytes: &[u8]) -> Result<(Vec<Component>, usize)> {
    // Read a vector of components
    let (count, mut offset) = binary::read_leb128_u32(bytes, 0)?;
    let mut components = Vec::with_capacity(count as usize);

    for _ in 0..count {
        // Read component size
        let (component_size, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        if offset + component_size as usize > bytes.len() {
            return Err(Error::parse_error_from_kind(kinds::ParseError(
                "Component size exceeds section size".to_string(),
            )));
        }

        // Extract the component binary
        let component_end = offset + component_size as usize;
        let component_bytes = &bytes[offset..component_end];

        // Parse the component binary using the decoder
        match crate::component::decode_component(component_bytes) {
            Ok(component) => components.push(component),
            Err(e) => {
                return Err(Error::parse_error_from_kind(kinds::ParseError(format_to_string(
                    "Failed to parse nested component",
                    e,
                ))));
            }
        }

        offset = component_end;
    }

    Ok((components, offset))
}

/// Parse an instance section
pub fn parse_instance_section(bytes: &[u8]) -> Result<(Vec<Instance>, usize)> {
    // Read a vector of instances
    let (count, mut offset) = binary::read_leb128_u32(bytes, 0)?;
    let mut instances = Vec::with_capacity(count as usize);

    for _ in 0..count {
        // Parse the instance expression
        let (instance_expr, bytes_read) = parse_instance_expr(&bytes[offset..])?;
        offset += bytes_read;

        // Create the instance
        instances.push(Instance { instance_expr });
    }

    Ok((instances, offset))
}

/// Parse an instance expression
fn parse_instance_expr(bytes: &[u8]) -> Result<(wrt_format::component::InstanceExpr, usize)> {
    if bytes.is_empty() {
        return Err(Error::parse_error_from_kind(kinds::ParseError(
            "Unexpected end of input while parsing instance expression".to_string(),
        )));
    }

    // Read the expression tag
    let tag = bytes[0];
    let mut offset = 1;

    match tag {
        0x00 => {
            // Instantiate a component
            let (component_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Read argument vector
            let (args_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut args = Vec::with_capacity(args_count as usize);
            for _ in 0..args_count {
                // Read name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read sort byte
                if offset >= bytes.len() {
                    return Err(Error::parse_error_from_kind(kinds::ParseError(
                        "Unexpected end of input while parsing instantiation argument sort"
                            .to_string(),
                    )));
                }
                let sort_byte = bytes[offset];
                offset += 1;

                // Parse sort
                let sort = parse_sort(sort_byte)?;

                // Read index
                let (idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                args.push(wrt_format::component::InstantiateArg { name, sort, idx });
            }

            Ok((wrt_format::component::InstanceExpr::Instantiate { component_idx, args }, offset))
        }
        0x01 => {
            // Inline exports
            let (exports_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut exports = Vec::with_capacity(exports_count as usize);
            for _ in 0..exports_count {
                // Read name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read sort byte
                if offset >= bytes.len() {
                    return Err(Error::parse_error_from_kind(kinds::ParseError(
                        "Unexpected end of input while parsing export sort".to_string(),
                    )));
                }
                let sort_byte = bytes[offset];
                offset += 1;

                // Parse sort
                let sort = parse_sort(sort_byte)?;

                // Read index
                let (idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                exports.push(wrt_format::component::InlineExport { name, sort, idx });
            }

            Ok((wrt_format::component::InstanceExpr::InlineExports(exports), offset))
        }
        _ => Err(Error::parse_error_from_kind(kinds::ParseError(format_to_string(
            "Invalid instance expression tag",
            tag,
        )))),
    }
}

/// Parse a sort byte
fn parse_sort(sort_byte: u8) -> Result<wrt_format::component::Sort> {
    match sort_byte {
        binary::COMPONENT_SORT_CORE => {
            Ok(wrt_format::component::Sort::Core(wrt_format::component::CoreSort::Function))
        }
        binary::COMPONENT_SORT_FUNC => Ok(wrt_format::component::Sort::Function),
        binary::COMPONENT_SORT_MODULE => Ok(wrt_format::component::Sort::Component),
        binary::COMPONENT_SORT_INSTANCE => Ok(wrt_format::component::Sort::Instance),
        binary::COMPONENT_SORT_COMPONENT => Ok(wrt_format::component::Sort::Component),
        binary::COMPONENT_SORT_VALUE => Ok(wrt_format::component::Sort::Value),
        binary::COMPONENT_SORT_TYPE => Ok(wrt_format::component::Sort::Type),
        _ => Err(Error::parse_error_from_kind(kinds::ParseError(format_to_string(
            "Invalid sort byte",
            sort_byte,
        )))),
    }
}

/// Parse a canon section
pub fn parse_canon_section(bytes: &[u8]) -> Result<(Vec<Canon>, usize)> {
    // Read a vector of canon operations
    let (count, mut offset) = binary::read_leb128_u32(bytes, 0)?;
    let mut canons = Vec::with_capacity(count as usize);

    for _ in 0..count {
        // Read canon operation
        let (operation, bytes_read) = parse_canon_operation(&bytes[offset..])?;
        offset += bytes_read;

        // Create the canon
        canons.push(Canon { operation });
    }

    Ok((canons, offset))
}

/// Parse a canon operation
fn parse_canon_operation(bytes: &[u8]) -> Result<(wrt_format::component::CanonOperation, usize)> {
    if bytes.is_empty() {
        return Err(Error::parse_error_from_kind(kinds::ParseError(
            "Unexpected end of input while parsing canon operation".to_string(),
        )));
    }

    // Read the operation tag
    let tag = bytes[0];
    let mut offset = 1;

    match tag {
        0x00 => {
            // Lift operation

            // Read core function index
            let (func_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Read type index
            let (type_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Read options
            let (options, bytes_read) = parse_lift_options(&bytes[offset..])?;
            offset += bytes_read;

            Ok((
                wrt_format::component::CanonOperation::Lift { func_idx, type_idx, options },
                offset,
            ))
        }
        0x01 => {
            // Lower operation

            // Read function index
            let (func_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Read options
            let (options, bytes_read) = parse_lower_options(&bytes[offset..])?;
            offset += bytes_read;

            Ok((wrt_format::component::CanonOperation::Lower { func_idx, options }, offset))
        }
        0x02 => {
            // Resource operations

            // Read resource operation
            let (resource_op, bytes_read) = parse_resource_operation(&bytes[offset..])?;
            offset += bytes_read;

            // Convert from ResourceCanonicalOperation to FormatResourceOperation
            let format_resource_op = match resource_op {
                resource::ResourceCanonicalOperation::New(new) => {
                    wrt_format::component::FormatResourceOperation::New(new)
                }
                resource::ResourceCanonicalOperation::Drop(drop) => {
                    wrt_format::component::FormatResourceOperation::Drop(drop)
                }
                resource::ResourceCanonicalOperation::Rep(rep) => {
                    wrt_format::component::FormatResourceOperation::Rep(rep)
                }
            };

            Ok((wrt_format::component::CanonOperation::Resource(format_resource_op), offset))
        }
        _ => Err(Error::parse_error_from_kind(kinds::ParseError(format_to_string(
            "Invalid canon operation tag",
            tag,
        )))),
    }
}

/// Parse lift options
fn parse_lift_options(bytes: &[u8]) -> Result<(wrt_format::component::LiftOptions, usize)> {
    let mut offset = 0;

    // Read memory index (optional)
    let (has_memory, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    let memory_idx = if has_memory != 0 {
        let (idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;
        Some(idx)
    } else {
        None
    };

    // Read string encoding (optional)
    let (has_encoding, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    let string_encoding = if has_encoding != 0 {
        if offset >= bytes.len() {
            return Err(Error::parse_error_from_kind(kinds::ParseError(
                "Unexpected end of input while parsing string encoding".to_string(),
            )));
        }

        let encoding_byte = bytes[offset];
        offset += 1;

        let encoding = match encoding_byte {
            0x00 => wrt_format::component::StringEncoding::UTF8,
            0x01 => wrt_format::component::StringEncoding::UTF16,
            0x02 => wrt_format::component::StringEncoding::Latin1,
            0x03 => wrt_format::component::StringEncoding::ASCII,
            _ => {
                return Err(Error::parse_error_from_kind(kinds::ParseError(format_to_string(
                    "Invalid string encoding",
                    encoding_byte,
                ))));
            }
        };

        Some(encoding)
    } else {
        None
    };

    let string_encoding_value = match string_encoding {
        Some(encoding) => encoding,
        None => wrt_format::component::StringEncoding::UTF8,
    };

    Ok((
        wrt_format::component::LiftOptions {
            memory_idx: Some(memory_idx.unwrap_or(0)),
            string_encoding: Some(string_encoding_value),
            realloc_func_idx: None,
            post_return_func_idx: None,
            is_async: false,
        },
        offset,
    ))
}

/// Parse lower options
fn parse_lower_options(bytes: &[u8]) -> Result<(wrt_format::component::LowerOptions, usize)> {
    let mut offset = 0;

    // Read memory index (optional)
    let (has_memory, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    let memory_idx = if has_memory != 0 {
        let (idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;
        Some(idx)
    } else {
        None
    };

    // Read string encoding (optional)
    let (has_encoding, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    let string_encoding = if has_encoding != 0 {
        if offset >= bytes.len() {
            return Err(Error::parse_error_from_kind(kinds::ParseError(
                "Unexpected end of input while parsing string encoding".to_string(),
            )));
        }

        let encoding_byte = bytes[offset];
        offset += 1;

        let encoding = match encoding_byte {
            0x00 => wrt_format::component::StringEncoding::UTF8,
            0x01 => wrt_format::component::StringEncoding::UTF16,
            0x02 => wrt_format::component::StringEncoding::Latin1,
            0x03 => wrt_format::component::StringEncoding::ASCII,
            _ => {
                return Err(Error::parse_error_from_kind(kinds::ParseError(format_to_string(
                    "Invalid string encoding",
                    encoding_byte,
                ))));
            }
        };

        Some(encoding)
    } else {
        None
    };

    let string_encoding_value = match string_encoding {
        Some(encoding) => encoding,
        None => wrt_format::component::StringEncoding::UTF8,
    };

    Ok((
        wrt_format::component::LowerOptions {
            memory_idx: Some(memory_idx.unwrap_or(0)),
            string_encoding: Some(string_encoding_value),
            realloc_func_idx: None,
            is_async: false,
            error_mode: None,
        },
        offset,
    ))
}

/// Parse resource operation
fn parse_resource_operation(bytes: &[u8]) -> Result<(resource::ResourceCanonicalOperation, usize)> {
    if bytes.is_empty() {
        return Err(Error::parse_error_from_kind(kinds::ParseError(
            "Unexpected end of input while parsing resource operation".to_string(),
        )));
    }

    // Read the tag
    let tag = bytes[0];
    let mut offset = 1;

    match tag {
        0x00 => {
            // New operation
            let (type_idx, type_idx_size) = binary::read_leb128_u32(bytes, offset)?;
            offset += type_idx_size;

            Ok((
                resource::ResourceCanonicalOperation::New(resource::ResourceNew { type_idx }),
                offset,
            ))
        }
        0x01 => {
            // Drop operation
            let (type_idx, type_idx_size) = binary::read_leb128_u32(bytes, offset)?;
            offset += type_idx_size;

            Ok((
                resource::ResourceCanonicalOperation::Drop(resource::ResourceDrop { type_idx }),
                offset,
            ))
        }
        0x02 => {
            // Rep operation
            let (type_idx, type_idx_size) = binary::read_leb128_u32(bytes, offset)?;
            offset += type_idx_size;

            Ok((
                resource::ResourceCanonicalOperation::Rep(resource::ResourceRep { type_idx }),
                offset,
            ))
        }
        _ => Err(Error::parse_error_from_kind(kinds::ParseError(format_to_string(
            "Invalid resource operation tag",
            tag,
        )))),
    }
}

/// Parse a component type section
pub fn parse_component_type_section(bytes: &[u8]) -> Result<(Vec<ComponentType>, usize)> {
    // Read a vector of component types
    let (count, mut offset) = binary::read_leb128_u32(bytes, 0)?;
    let mut types = Vec::with_capacity(count as usize);

    for _ in 0..count {
        // Read type definition
        let (definition, bytes_read) = parse_component_type_definition(&bytes[offset..])?;
        offset += bytes_read;

        // Create the type
        types.push(ComponentType { definition });
    }

    Ok((types, offset))
}

/// Parse a component type definition
fn parse_component_type_definition(
    bytes: &[u8],
) -> Result<(wrt_format::component::ComponentTypeDefinition, usize)> {
    if bytes.is_empty() {
        return Err(Error::parse_error_from_kind(kinds::ParseError(
            "Unexpected end of input while parsing component type definition".to_string(),
        )));
    }

    // Read the form tag
    let form = bytes[0];
    let mut offset = 1;

    match form {
        0x00 => {
            // Component type

            // Read import vector
            let (import_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut imports = Vec::with_capacity(import_count as usize);
            for _ in 0..import_count {
                // Read namespace
                let (namespace, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read type
                let (extern_type, bytes_read) = parse_extern_type(&bytes[offset..])?;
                offset += bytes_read;

                imports.push((namespace, name, extern_type));
            }

            // Read export vector
            let (export_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut exports = Vec::with_capacity(export_count as usize);
            for _ in 0..export_count {
                // Read name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read type
                let (extern_type, bytes_read) = parse_extern_type(&bytes[offset..])?;
                offset += bytes_read;

                exports.push((name, extern_type));
            }

            Ok((
                wrt_format::component::ComponentTypeDefinition::Component { imports, exports },
                offset,
            ))
        }
        0x01 => {
            // Instance type

            // Read export vector
            let (export_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut exports = Vec::with_capacity(export_count as usize);
            for _ in 0..export_count {
                // Read name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read type
                let (extern_type, bytes_read) = parse_extern_type(&bytes[offset..])?;
                offset += bytes_read;

                exports.push((name, extern_type));
            }

            Ok((wrt_format::component::ComponentTypeDefinition::Instance { exports }, offset))
        }
        0x02 => {
            // Function type

            // Read parameter vector
            let (param_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut params = Vec::with_capacity(param_count as usize);
            for _ in 0..param_count {
                // Read parameter name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read parameter type
                let (val_type, bytes_read) = parse_val_type(&bytes[offset..])?;
                offset += bytes_read;

                params.push((name, val_type));
            }

            // Read result vector
            let (result_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut results = Vec::with_capacity(result_count as usize);
            for _ in 0..result_count {
                // Read result type
                let (val_type, bytes_read) = parse_val_type(&bytes[offset..])?;
                offset += bytes_read;

                results.push(val_type);
            }

            Ok((
                wrt_format::component::ComponentTypeDefinition::Function {
                    params: params
                        .into_iter()
                        .map(|(name, ty)| (name, val_type_to_format_val_type(ty)))
                        .collect(),
                    results: results.into_iter().map(val_type_to_format_val_type).collect(),
                },
                offset,
            ))
        }
        0x03 => {
            // Value type

            // Read value type
            let (val_type, bytes_read) = parse_val_type(&bytes[offset..])?;
            offset += bytes_read;

            Ok((
                wrt_format::component::ComponentTypeDefinition::Value(val_type_to_format_val_type(
                    val_type,
                )),
                offset,
            ))
        }
        0x04 => {
            // Resource type

            // Read representation
            let (representation, bytes_read) = parse_resource_representation(&bytes[offset..])?;
            offset += bytes_read;

            // Read nullable flag
            if offset >= bytes.len() {
                return Err(Error::parse_error_from_kind(kinds::ParseError(
                    "Unexpected end of input while parsing resource nullable flag".to_string(),
                )));
            }
            let nullable = bytes[offset] != 0;
            offset += 1;

            Ok((
                wrt_format::component::ComponentTypeDefinition::Resource {
                    representation,
                    nullable,
                },
                offset,
            ))
        }
        _ => Err(Error::parse_error_from_kind(kinds::ParseError(format_to_string(
            "Invalid component type form",
            form,
        )))),
    }
}

/// Parse a resource representation
fn parse_resource_representation(
    bytes: &[u8],
) -> Result<(resource::ResourceRepresentation, usize)> {
    if bytes.is_empty() {
        return Err(Error::parse_error_from_kind(kinds::ParseError(
            "Unexpected end of input while parsing resource representation".to_string(),
        )));
    }

    // Read the tag
    let tag = bytes[0];
    let mut offset = 1;

    match tag {
        0x00 => {
            // Handle32
            Ok((resource::ResourceRepresentation::Handle32, offset))
        }
        0x01 => {
            // Handle64
            Ok((resource::ResourceRepresentation::Handle64, offset))
        }
        0x02 => {
            // Record representation

            // Read field vector
            let (field_count, bytes_read) = match binary::read_leb128_u32(bytes, offset) {
                Ok(result) => result,
                Err(e) => {
                    return Err(Error::parse_error_from_kind(kinds::ParseError(format!(
                        "Failed to read field count in resource record representation: {}",
                        e
                    ))))
                }
            };
            offset += bytes_read;

            let mut fields = Vec::with_capacity(field_count as usize);
            for i in 0..field_count {
                // Read field name
                let (name, bytes_read) = match binary::read_string(bytes, offset) {
                    Ok(result) => result,
                    Err(e) => {
                        return Err(Error::parse_error_from_kind(kinds::ParseError(format!(
                            "Failed to read field name {} in resource record representation: {}",
                            i, e
                        ))))
                    }
                };
                offset += bytes_read;

                fields.push(name);
            }

            Ok((
                #[cfg(feature = "alloc")]
                resource::ResourceRepresentation::Record(fields),
                #[cfg(not(feature = "alloc"))]
                resource::ResourceRepresentation::Record,
                offset,
            ))
        }
        0x03 => {
            // Aggregate representation

            // Read type indices
            let (index_count, bytes_read) = match binary::read_leb128_u32(bytes, offset) {
                Ok(result) => result,
                Err(e) => {
                    return Err(Error::parse_error_from_kind(kinds::ParseError(format!(
                        "Failed to read index count in resource aggregate representation: {}",
                        e
                    ))))
                }
            };
            offset += bytes_read;

            let mut indices = Vec::with_capacity(index_count as usize);
            for i in 0..index_count {
                // Read type index
                let (idx, bytes_read) = match binary::read_leb128_u32(bytes, offset) {
                    Ok(result) => result,
                    Err(e) => {
                        return Err(Error::parse_error(env_format!(
                            "Failed to read type index {} in resource aggregate representation: {}",
                            i,
                            e
                        )))
                    }
                };
                offset += bytes_read;

                indices.push(idx);
            }

            #[cfg(feature = "alloc")]
            let repr = resource::ResourceRepresentation::Aggregate(indices);
            #[cfg(not(feature = "alloc"))]
            let repr = resource::ResourceRepresentation::Record;

            Ok((repr, offset))
        }
        _ => {
            Err(Error::parse_error(env_format!("Invalid resource representation tag: {:#x}", tag)))
        }
    }
}

/// Parse an external type
fn parse_extern_type(bytes: &[u8]) -> Result<(wrt_format::component::ExternType, usize)> {
    if bytes.is_empty() {
        return Err(Error::parse_error(
            "Unexpected end of input while parsing external type".to_string(),
        ));
    }

    // Read the tag
    let tag = bytes[0];
    let mut offset = 1;

    match tag {
        0x00 => {
            // Function type

            // Read parameter vector
            let (param_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut params = Vec::with_capacity(param_count as usize);
            for _ in 0..param_count {
                // Read parameter name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read parameter type
                let (val_type, bytes_read) = parse_val_type(&bytes[offset..])?;
                offset += bytes_read;

                params.push((name, val_type));
            }

            // Read result vector
            let (result_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut results = Vec::with_capacity(result_count as usize);
            for _ in 0..result_count {
                // Read result type
                let (val_type, bytes_read) = parse_val_type(&bytes[offset..])?;
                offset += bytes_read;

                results.push(val_type);
            }

            Ok((
                wrt_format::component::ExternType::Function {
                    params: params
                        .into_iter()
                        .map(|(name, ty)| (name, val_type_to_format_val_type(ty)))
                        .collect(),
                    results: results.into_iter().map(val_type_to_format_val_type).collect(),
                },
                offset,
            ))
        }
        0x01 => {
            // Value type

            // Read value type
            let (val_type, bytes_read) = parse_val_type(&bytes[offset..])?;
            offset += bytes_read;

            Ok((
                wrt_format::component::ExternType::Value(val_type_to_format_val_type(val_type)),
                offset,
            ))
        }
        0x02 => {
            // Type reference

            // Read type index
            let (type_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            Ok((wrt_format::component::ExternType::Type(type_idx), offset))
        }
        0x03 => {
            // Instance type

            // Read export vector
            let (export_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut exports = Vec::with_capacity(export_count as usize);
            for _ in 0..export_count {
                // Read export name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read export type
                let (extern_type, bytes_read) = parse_extern_type(&bytes[offset..])?;
                offset += bytes_read;

                exports.push((name, extern_type));
            }

            Ok((wrt_format::component::ExternType::Instance { exports }, offset))
        }
        0x04 => {
            // Component type

            // Read import vector
            let (import_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut imports = Vec::with_capacity(import_count as usize);
            for _ in 0..import_count {
                // Read namespace
                let (namespace, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read type
                let (extern_type, bytes_read) = parse_extern_type(&bytes[offset..])?;
                offset += bytes_read;

                imports.push((namespace, name, extern_type));
            }

            // Read export vector
            let (export_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut exports = Vec::with_capacity(export_count as usize);
            for _ in 0..export_count {
                // Read name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read type
                let (extern_type, bytes_read) = parse_extern_type(&bytes[offset..])?;
                offset += bytes_read;

                exports.push((name, extern_type));
            }

            Ok((wrt_format::component::ExternType::Component { imports, exports }, offset))
        }
        _ => Err(Error::parse_error(env_format!("Invalid external type tag: {:#x}", tag))),
    }
}

/// Parse a value type
fn parse_val_type(bytes: &[u8]) -> Result<(wrt_format::component::ValType, usize)> {
    if bytes.is_empty() {
        return Err(Error::parse_error(
            "Unexpected end of input while parsing value type".to_string(),
        ));
    }

    // Read the type tag
    let tag = bytes[0];
    let mut offset = 1;

    match tag {
        0x7F => Ok((wrt_format::component::ValType::Bool, offset)),
        0x7E => Ok((wrt_format::component::ValType::S8, offset)),
        0x7D => Ok((wrt_format::component::ValType::U8, offset)),
        0x7C => Ok((wrt_format::component::ValType::S16, offset)),
        0x7B => Ok((wrt_format::component::ValType::U16, offset)),
        0x7A => Ok((wrt_format::component::ValType::S32, offset)),
        0x79 => Ok((wrt_format::component::ValType::U32, offset)),
        0x78 => Ok((wrt_format::component::ValType::S64, offset)),
        0x77 => Ok((wrt_format::component::ValType::U64, offset)),
        0x76 => Ok((wrt_format::component::ValType::F32, offset)),
        0x75 => Ok((wrt_format::component::ValType::F64, offset)),
        0x74 => Ok((wrt_format::component::ValType::Char, offset)),
        0x73 => Ok((wrt_format::component::ValType::String, offset)),
        0x72 => {
            // Reference type
            let (idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;
            Ok((wrt_format::component::ValType::Ref(idx), offset))
        }
        0x71 => {
            // Record type
            let (field_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut fields = Vec::with_capacity(field_count as usize);
            for _ in 0..field_count {
                // Read field name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read field type
                let (field_type, bytes_read) = parse_val_type(&bytes[offset..])?;
                offset += bytes_read;

                fields.push((name, field_type));
            }

            Ok((wrt_format::component::ValType::Record(fields), offset))
        }
        0x70 => {
            // Variant type
            let (case_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut cases = Vec::with_capacity(case_count as usize);
            for _ in 0..case_count {
                // Read case name
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;

                // Read case type flag
                let has_type = bytes[offset] != 0;
                offset += 1;

                let mut case_type = None;
                if has_type {
                    let (ty, bytes_read) = parse_val_type(&bytes[offset..])?;
                    offset += bytes_read;
                    case_type = Some(ty);
                }

                cases.push((name, case_type));
            }

            Ok((wrt_format::component::ValType::Variant(cases), offset))
        }
        0x6F => {
            // List type
            let (element_type, bytes_read) = parse_val_type(&bytes[offset..])?;
            offset += bytes_read;
            Ok((wrt_format::component::ValType::List(Box::new(element_type)), offset))
        }
        0x6E => {
            // Fixed-length list type (🔧)
            let (element_type, bytes_read) = parse_val_type(&bytes[offset..])?;
            offset += bytes_read;

            // Read the length
            let (length, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            Ok((wrt_format::component::ValType::FixedList(Box::new(element_type), length), offset))
        }
        0x6D => {
            // Tuple type
            let (field_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut fields = Vec::with_capacity(field_count as usize);
            for _ in 0..field_count {
                let (field_type, bytes_read) = parse_val_type(&bytes[offset..])?;
                offset += bytes_read;
                fields.push(field_type);
            }

            Ok((wrt_format::component::ValType::Tuple(fields), offset))
        }
        0x6C => {
            // Flags type
            let (flag_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut flags = Vec::with_capacity(flag_count as usize);
            for _ in 0..flag_count {
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;
                flags.push(name);
            }

            Ok((wrt_format::component::ValType::Flags(flags), offset))
        }
        0x6B => {
            // Enum type
            let (variant_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut variants = Vec::with_capacity(variant_count as usize);
            for _ in 0..variant_count {
                let (name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;
                variants.push(name);
            }

            Ok((wrt_format::component::ValType::Enum(variants), offset))
        }
        0x6A => {
            // Option type
            let (inner_type, bytes_read) = parse_val_type(&bytes[offset..])?;
            offset += bytes_read;
            Ok((wrt_format::component::ValType::Option(Box::new(inner_type)), offset))
        }
        0x69 => {
            // Result type (ok only)
            let (ok_type, bytes_read) = parse_val_type(&bytes[offset..])?;
            offset += bytes_read;
            Ok((wrt_format::component::ValType::Result(Box::new(ok_type)), offset))
        }
        0x68 => {
            // Result type (err only)
            let (err_type, bytes_read) = parse_val_type(&bytes[offset..])?;
            offset += bytes_read;
            Ok((wrt_format::component::ValType::ResultErr(Box::new(err_type)), offset))
        }
        0x67 => {
            // Result type (ok and err)
            let (ok_type, bytes_read) = parse_val_type(&bytes[offset..])?;
            offset += bytes_read;
            let (err_type, bytes_read) = parse_val_type(&bytes[offset..])?;
            offset += bytes_read;
            Ok((
                wrt_format::component::ValType::ResultBoth(Box::new(ok_type), Box::new(err_type)),
                offset,
            ))
        }
        0x66 => {
            // Own a resource
            let (idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;
            Ok((wrt_format::component::ValType::Own(idx), offset))
        }
        0x65 => {
            // Borrow a resource
            let (idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;
            Ok((wrt_format::component::ValType::Borrow(idx), offset))
        }
        0x64 => {
            // Error context type
            Ok((wrt_format::component::ValType::ErrorContext, offset))
        }
        _ => Err(Error::parse_error(env_format!("Invalid value type tag: {:#x}", tag))),
    }
}

/// Parse a start section
pub fn parse_start_section(bytes: &[u8]) -> Result<(Start, usize)> {
    let mut offset = 0;

    // Read function index
    let (func_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    // Read argument vector
    let (arg_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    let mut args = Vec::with_capacity(arg_count as usize);
    for _ in 0..arg_count {
        // Read argument index
        let (arg_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        args.push(arg_idx);
    }

    // Read results count
    let (results, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
    offset += bytes_read;

    Ok((Start { func_idx, args, results }, offset))
}

/// Parse an import section
pub fn parse_import_section(bytes: &[u8]) -> Result<(Vec<Import>, usize)> {
    // Read a vector of imports
    let (count, mut offset) = binary::read_leb128_u32(bytes, 0)?;
    let mut imports = Vec::with_capacity(count as usize);

    for _ in 0..count {
        // Read import name
        let (namespace, bytes_read) = binary::read_string(bytes, offset)?;
        offset += bytes_read;

        let (name, bytes_read) = binary::read_string(bytes, offset)?;
        offset += bytes_read;

        // Check if there are nested namespaces or package information
        let mut nested = Vec::new();
        let mut package = None;

        // Read nested namespace flag if present
        if offset < bytes.len() {
            let has_nested = bytes[offset] != 0;
            offset += 1;

            if has_nested {
                // Read count of nested namespaces
                let (nested_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                // Read each nested namespace
                for _ in 0..nested_count {
                    let (nested_name, bytes_read) = binary::read_string(bytes, offset)?;
                    offset += bytes_read;
                    nested.push(nested_name);
                }
            }

            // Read package flag if present
            if offset < bytes.len() {
                let has_package = bytes[offset] != 0;
                offset += 1;

                if has_package {
                    // Read package name
                    let (package_name, bytes_read) = binary::read_string(bytes, offset)?;
                    offset += bytes_read;

                    // Read version flag
                    let has_version = bytes[offset] != 0;
                    offset += 1;

                    let mut version = None;
                    if has_version {
                        let (ver, bytes_read) = binary::read_string(bytes, offset)?;
                        offset += bytes_read;
                        version = Some(ver);
                    }

                    // Read hash flag
                    let has_hash = bytes[offset] != 0;
                    offset += 1;

                    let mut hash = None;
                    if has_hash {
                        let (h, bytes_read) = binary::read_string(bytes, offset)?;
                        offset += bytes_read;
                        hash = Some(h);
                    }

                    package = Some(wrt_format::component::PackageReference {
                        name: package_name,
                        version,
                        hash,
                    });
                }
            }
        }

        // Read import type
        let (extern_type, bytes_read) = parse_extern_type(&bytes[offset..])?;
        offset += bytes_read;

        // Create the import
        imports.push(Import {
            name: wrt_format::component::ImportName { namespace, name, nested, package },
            ty: extern_type,
        });
    }

    Ok((imports, offset))
}

/// Parse an export section
pub fn parse_export_section(bytes: &[u8]) -> Result<(Vec<Export>, usize)> {
    // Read a vector of exports
    let (count, mut offset) = binary::read_leb128_u32(bytes, 0)?;
    let mut exports = Vec::with_capacity(count as usize);

    for _ in 0..count {
        // Read export name
        let (basic_name, bytes_read) = binary::read_string(bytes, offset)?;
        offset += bytes_read;

        // Read flags
        if offset >= bytes.len() {
            return Err(Error::parse_error_from_kind(kinds::ParseError(
                "Unexpected end of input while parsing export flags".to_string(),
            )));
        }
        let flags = bytes[offset];
        offset += 1;

        // Parse flags
        let is_resource = (flags & 0x01) != 0;
        let has_semver = (flags & 0x02) != 0;
        let has_integrity = (flags & 0x04) != 0;
        let has_nested = (flags & 0x08) != 0;

        // Read semver (if present)
        let semver = if has_semver {
            let (ver, bytes_read) = binary::read_string(bytes, offset)?;
            offset += bytes_read;
            Some(ver)
        } else {
            None
        };

        // Read integrity (if present)
        let integrity = if has_integrity {
            let (hash, bytes_read) = binary::read_string(bytes, offset)?;
            offset += bytes_read;
            Some(hash)
        } else {
            None
        };

        // Read nested namespaces (if present)
        let nested = if has_nested {
            // Read count of nested namespaces
            let (nested_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut nested_names = Vec::with_capacity(nested_count as usize);
            for _ in 0..nested_count {
                let (nested_name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;
                nested_names.push(nested_name);
            }
            nested_names
        } else {
            Vec::new()
        };

        // Create export name
        let export_name = wrt_format::component::ExportName {
            name: basic_name,
            is_resource,
            semver,
            integrity,
            nested,
        };

        // Read sort byte
        if offset >= bytes.len() {
            return Err(Error::parse_error_from_kind(kinds::ParseError(
                "Unexpected end of input while parsing export sort".to_string(),
            )));
        }
        let sort_byte = bytes[offset];
        offset += 1;

        // Parse sort
        let sort = parse_sort(sort_byte)?;

        // Read index
        let (idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        // Read type flag
        if offset >= bytes.len() {
            return Err(Error::parse_error_from_kind(kinds::ParseError(
                "Unexpected end of input while parsing export type flag".to_string(),
            )));
        }
        let has_type = bytes[offset] != 0;
        offset += 1;

        // Read type (if present)
        let ty = if has_type {
            let (extern_type, bytes_read) = parse_extern_type(&bytes[offset..])?;
            offset += bytes_read;
            Some(extern_type)
        } else {
            None
        };

        // Create the export
        exports.push(Export { name: export_name, sort, idx, ty });
    }

    Ok((exports, offset))
}

/// Parse a value section
pub fn parse_value_section(bytes: &[u8]) -> Result<(Vec<Value>, usize)> {
    // Read a vector of values
    let (count, mut offset) = binary::read_leb128_u32(bytes, 0)?;
    let mut values = Vec::with_capacity(count as usize);

    for _ in 0..count {
        // Read value type
        let (val_type, bytes_read) = parse_val_type(&bytes[offset..])?;
        offset += bytes_read;

        // Read value bytes size
        let (data_size, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
        offset += bytes_read;

        if offset + data_size as usize > bytes.len() {
            return Err(Error::parse_error_from_kind(kinds::ParseError(
                "Value data size exceeds section size".to_string(),
            )));
        }

        // Extract the value data
        let data_end = offset + data_size as usize;
        let data = bytes[offset..data_end].to_vec();
        offset = data_end;

        // Check for expression flag
        let mut expression = None;
        if offset < bytes.len() {
            let has_expr = bytes[offset] != 0;
            offset += 1;

            if has_expr {
                let (expr, bytes_read) = parse_value_expression(&bytes[offset..])?;
                offset += bytes_read;
                expression = Some(expr);
            }
        }

        // Check for name flag
        let mut name = None;
        if offset < bytes.len() {
            let has_name = bytes[offset] != 0;
            offset += 1;

            if has_name {
                let (value_name, bytes_read) = binary::read_string(bytes, offset)?;
                offset += bytes_read;
                name = Some(value_name);
            }
        }

        // Create the value
        values.push(Value { ty: val_type_to_format_val_type(val_type), data, expression, name });
    }

    Ok((values, offset))
}

/// Parse a value expression
fn parse_value_expression(bytes: &[u8]) -> Result<(wrt_format::component::ValueExpression, usize)> {
    if bytes.is_empty() {
        return Err(Error::parse_error_from_kind(kinds::ParseError(
            "Unexpected end of input while parsing value expression".to_string(),
        )));
    }

    // Read the expression tag
    let tag = bytes[0];
    let mut offset = 1;

    match tag {
        0x00 => {
            // Reference to an item
            let kind_byte = bytes[offset];
            offset += 1;

            // Convert to Sort
            let sort = parse_sort(kind_byte)?;

            // Read index
            let (idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            Ok((wrt_format::component::ValueExpression::ItemRef { sort, idx }, offset))
        }
        0x01 => {
            // Global initialization
            let (global_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            Ok((wrt_format::component::ValueExpression::GlobalInit { global_idx }, offset))
        }
        0x02 => {
            // Function call
            let (func_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Read args vector
            let (args_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let mut args = Vec::with_capacity(args_count as usize);
            for _ in 0..args_count {
                let (arg_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;
                args.push(arg_idx);
            }

            Ok((wrt_format::component::ValueExpression::FunctionCall { func_idx, args }, offset))
        }
        0x03 => {
            // Constant value
            let (const_value, bytes_read) = parse_const_value(&bytes[offset..])?;
            offset += bytes_read;

            Ok((wrt_format::component::ValueExpression::Const(const_value), offset))
        }
        _ => Err(Error::parse_error_from_kind(kinds::ParseError(format!(
            "Invalid value expression tag: {:#x}",
            tag
        )))),
    }
}

/// Parse a constant value
fn parse_const_value(bytes: &[u8]) -> Result<(wrt_format::component::ConstValue, usize)> {
    if bytes.is_empty() {
        return Err(Error::parse_error_from_kind(kinds::ParseError(
            "Unexpected end of input while parsing constant value".to_string(),
        )));
    }

    // Read the value tag
    let tag = bytes[0];
    let mut offset = 1;

    match tag {
        0x00 => {
            // Boolean value
            let value = bytes[offset] != 0;
            offset += 1;
            Ok((wrt_format::component::ConstValue::Bool(value), offset))
        }
        0x01 => {
            // S8 value
            let value = bytes[offset] as i8;
            offset += 1;
            Ok((wrt_format::component::ConstValue::S8(value), offset))
        }
        0x02 => {
            // U8 value
            let value = bytes[offset];
            offset += 1;
            Ok((wrt_format::component::ConstValue::U8(value), offset))
        }
        0x03 => {
            // S16 value
            let value = i16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
            offset += 2;
            Ok((wrt_format::component::ConstValue::S16(value), offset))
        }
        0x04 => {
            // U16 value
            let value = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
            offset += 2;
            Ok((wrt_format::component::ConstValue::U16(value), offset))
        }
        0x05 => {
            // S32 value
            let value = i32::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]);
            offset += 4;
            Ok((wrt_format::component::ConstValue::S32(value), offset))
        }
        0x06 => {
            // U32 value
            let value = u32::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]);
            offset += 4;
            Ok((wrt_format::component::ConstValue::U32(value), offset))
        }
        0x07 => {
            // S64 value
            let value = i64::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
                bytes[offset + 4],
                bytes[offset + 5],
                bytes[offset + 6],
                bytes[offset + 7],
            ]);
            offset += 8;
            Ok((wrt_format::component::ConstValue::S64(value), offset))
        }
        0x08 => {
            // U64 value
            let value = u64::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
                bytes[offset + 4],
                bytes[offset + 5],
                bytes[offset + 6],
                bytes[offset + 7],
            ]);
            offset += 8;
            Ok((wrt_format::component::ConstValue::U64(value), offset))
        }
        0x09 => {
            // F32 value
            let value_bits = u32::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]);
            let value = f32::from_bits(value_bits);
            offset += 4;
            Ok((wrt_format::component::ConstValue::F32(value), offset))
        }
        0x0A => {
            // F64 value
            let value_bits = u64::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
                bytes[offset + 4],
                bytes[offset + 5],
                bytes[offset + 6],
                bytes[offset + 7],
            ]);
            let value = f64::from_bits(value_bits);
            offset += 8;
            Ok((wrt_format::component::ConstValue::F64(value), offset))
        }
        0x0B => {
            // Char value
            let (value_str, bytes_read) = binary::read_string(bytes, offset)?;
            offset += bytes_read;

            // Validate that the string is a single Unicode scalar value
            let mut chars = value_str.chars();
            let first_char = chars.next().ok_or_else(|| {
                Error::parse_error_from_kind(kinds::ParseError(
                    "Empty string found when parsing char value".to_string(),
                ))
            })?;
            if chars.next().is_some() {
                return Err(Error::parse_error_from_kind(kinds::ParseError(
                    "Multiple characters found when parsing char value".to_string(),
                )));
            }

            Ok((wrt_format::component::ConstValue::Char(first_char), offset))
        }
        0x0C => {
            // String value
            let (value, bytes_read) = binary::read_string(bytes, offset)?;
            offset += bytes_read;
            Ok((wrt_format::component::ConstValue::String(value), offset))
        }
        0x0D => {
            // Null value
            Ok((wrt_format::component::ConstValue::Null, offset))
        }
        _ => Err(Error::parse_error_from_kind(kinds::ParseError(format!(
            "Invalid constant value tag: {:#x}",
            tag
        )))),
    }
}

/// Parse an alias section
pub fn parse_alias_section(bytes: &[u8]) -> Result<(Vec<Alias>, usize)> {
    // Read a vector of aliases
    let (count, mut offset) = binary::read_leb128_u32(bytes, 0)?;
    let mut aliases = Vec::with_capacity(count as usize);

    for _ in 0..count {
        // Read alias target
        let (target, bytes_read) = parse_alias_target(&bytes[offset..])?;
        offset += bytes_read;

        // Create the alias
        aliases.push(Alias { target });
    }

    Ok((aliases, offset))
}

/// Parse an alias target
fn parse_alias_target(bytes: &[u8]) -> Result<(wrt_format::component::AliasTarget, usize)> {
    if bytes.is_empty() {
        return Err(Error::parse_error_from_kind(kinds::ParseError(
            "Unexpected end of input while parsing alias target".to_string(),
        )));
    }

    // Read the target tag
    let tag = bytes[0];
    let mut offset = 1;

    match tag {
        0x00 => {
            // Core instance export

            // Read instance index
            let (instance_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Read export name
            let (name, bytes_read) = binary::read_string(bytes, offset)?;
            offset += bytes_read;

            // Read kind byte
            if offset >= bytes.len() {
                return Err(Error::parse_error_from_kind(kinds::ParseError(
                    "Unexpected end of input while parsing core export kind".to_string(),
                )));
            }
            let kind_byte = bytes[offset];
            offset += 1;

            // Convert to CoreSort
            let kind = match kind_byte {
                binary::COMPONENT_CORE_SORT_FUNC => wrt_format::component::CoreSort::Function,
                binary::COMPONENT_CORE_SORT_TABLE => wrt_format::component::CoreSort::Table,
                binary::COMPONENT_CORE_SORT_MEMORY => wrt_format::component::CoreSort::Memory,
                binary::COMPONENT_CORE_SORT_GLOBAL => wrt_format::component::CoreSort::Global,
                binary::COMPONENT_CORE_SORT_TYPE => wrt_format::component::CoreSort::Type,
                binary::COMPONENT_CORE_SORT_MODULE => wrt_format::component::CoreSort::Module,
                binary::COMPONENT_CORE_SORT_INSTANCE => wrt_format::component::CoreSort::Instance,
                _ => {
                    return Err(Error::parse_error_from_kind(kinds::ParseError(env_format!(
                        "Invalid core sort kind: {:#x}",
                        kind_byte
                    ))));
                }
            };

            Ok((
                wrt_format::component::AliasTarget::CoreInstanceExport { instance_idx, name, kind },
                offset,
            ))
        }
        0x01 => {
            // Instance export

            // Read instance index
            let (instance_idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Read export name
            let (name, bytes_read) = binary::read_string(bytes, offset)?;
            offset += bytes_read;

            // Read kind byte
            if offset >= bytes.len() {
                return Err(Error::parse_error_from_kind(kinds::ParseError(
                    "Unexpected end of input while parsing export kind".to_string(),
                )));
            }
            let kind_byte = bytes[offset];
            offset += 1;

            // Parse sort
            let kind = parse_sort(kind_byte)?;

            Ok((
                wrt_format::component::AliasTarget::InstanceExport { instance_idx, name, kind },
                offset,
            ))
        }
        0x02 => {
            // Outer definition

            // Read count
            let (count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Read kind byte
            if offset >= bytes.len() {
                return Err(Error::parse_error_from_kind(kinds::ParseError(
                    "Unexpected end of input while parsing outer kind".to_string(),
                )));
            }
            let kind_byte = bytes[offset];
            offset += 1;

            // Parse sort
            let kind = parse_sort(kind_byte)?;

            // Read index
            let (idx, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            Ok((wrt_format::component::AliasTarget::Outer { count, kind, idx }, offset))
        }
        _ => Err(Error::parse_error_from_kind(kinds::ParseError(format!(
            "Invalid alias target tag: {:#x}",
            tag
        )))),
    }
}

/// Parse a name from a byte array
///
/// This is a utility function to parse a name field, which is common in
/// various WebAssembly and Component Model sections.
#[allow(dead_code)]
pub fn parse_name(bytes: &[u8]) -> Result<(String, usize)> {
    binary::read_string(bytes, 0)
}

/// Convert ValType to FormatValType for type compatibility
fn val_type_to_format_val_type(
    val_type: wrt_format::component::ValType,
) -> wrt_format::component::FormatValType {
    match val_type {
        wrt_format::component::ValType::Bool => wrt_format::component::FormatValType::Bool,
        wrt_format::component::ValType::S8 => wrt_format::component::FormatValType::S8,
        wrt_format::component::ValType::U8 => wrt_format::component::FormatValType::U8,
        wrt_format::component::ValType::S16 => wrt_format::component::FormatValType::S16,
        wrt_format::component::ValType::U16 => wrt_format::component::FormatValType::U16,
        wrt_format::component::ValType::S32 => wrt_format::component::FormatValType::S32,
        wrt_format::component::ValType::U32 => wrt_format::component::FormatValType::U32,
        wrt_format::component::ValType::S64 => wrt_format::component::FormatValType::S64,
        wrt_format::component::ValType::U64 => wrt_format::component::FormatValType::U64,
        wrt_format::component::ValType::F32 => wrt_format::component::FormatValType::F32,
        wrt_format::component::ValType::F64 => wrt_format::component::FormatValType::F64,
        wrt_format::component::ValType::Char => wrt_format::component::FormatValType::Char,
        wrt_format::component::ValType::String => wrt_format::component::FormatValType::String,
        wrt_format::component::ValType::Ref(idx) => wrt_format::component::FormatValType::Ref(idx),
        wrt_format::component::ValType::List(inner) => wrt_format::component::FormatValType::List(
            Box::new(val_type_to_format_val_type(*inner)),
        ),
        wrt_format::component::ValType::FixedList(inner, len) => {
            wrt_format::component::FormatValType::FixedList(
                Box::new(val_type_to_format_val_type(*inner)),
                len,
            )
        }
        wrt_format::component::ValType::Tuple(items) => {
            wrt_format::component::FormatValType::Tuple(
                items.into_iter().map(val_type_to_format_val_type).collect(),
            )
        }
        wrt_format::component::ValType::Option(inner) => {
            wrt_format::component::FormatValType::Option(Box::new(val_type_to_format_val_type(
                *inner,
            )))
        }
        wrt_format::component::ValType::Result(ok) => {
            wrt_format::component::FormatValType::Result(Box::new(val_type_to_format_val_type(*ok)))
        }
        wrt_format::component::ValType::ResultErr(err) => {
            wrt_format::component::FormatValType::Result(Box::new(val_type_to_format_val_type(
                *err,
            )))
        }
        wrt_format::component::ValType::ResultBoth(ok, _err) => {
            wrt_format::component::FormatValType::Result(Box::new(val_type_to_format_val_type(*ok)))
        }
        wrt_format::component::ValType::Record(fields) => {
            wrt_format::component::FormatValType::Record(
                fields
                    .into_iter()
                    .map(|(name, ty)| (name, val_type_to_format_val_type(ty)))
                    .collect(),
            )
        }
        wrt_format::component::ValType::Variant(cases) => {
            wrt_format::component::FormatValType::Variant(
                cases
                    .into_iter()
                    .map(|(name, ty)| (name, ty.map(val_type_to_format_val_type)))
                    .collect(),
            )
        }
        wrt_format::component::ValType::Flags(names) => {
            wrt_format::component::FormatValType::Flags(names)
        }
        wrt_format::component::ValType::Enum(names) => {
            wrt_format::component::FormatValType::Enum(names)
        }
        wrt_format::component::ValType::Own(idx) => wrt_format::component::FormatValType::Own(idx),
        wrt_format::component::ValType::Borrow(idx) => {
            wrt_format::component::FormatValType::Borrow(idx)
        }
        wrt_format::component::ValType::Void => wrt_format::component::FormatValType::Void,
        wrt_format::component::ValType::ErrorContext => {
            wrt_format::component::FormatValType::ErrorContext
        }
    }
}
