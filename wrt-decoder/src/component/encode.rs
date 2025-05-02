use crate::prelude::*;
use wrt_error::Result;
use wrt_format::binary;
use wrt_format::component::Component;

/// Encode a WebAssembly Component Model component into binary format
pub fn encode_component(component: &Component) -> Result<Vec<u8>> {
    let mut binary = Vec::new();

    // Write magic and version
    binary.extend_from_slice(&binary::COMPONENT_MAGIC); // \0asm
    binary.extend_from_slice(&binary::COMPONENT_VERSION); // Version and layer

    // Encode and write sections
    encode_sections(component, &mut binary)?;

    Ok(binary)
}

fn encode_sections(component: &Component, binary: &mut Vec<u8>) -> Result<()> {
    // If the component has a name, add a name section
    if let Some(name) = &component.name {
        // Create a name section with the component name
        let name_section = crate::component::name_section::ComponentNameSection {
            component_name: component.name.clone(),
            sort_names: Vec::new(),
            import_names: Default::default(),
            export_names: Default::default(),
            canonical_names: Default::default(),
            type_names: Default::default(),
        };

        let name_section_bytes =
            crate::component::name_section::generate_component_name_section(&name_section)?;

        // Create the custom section content
        let mut custom_section_content = Vec::new();
        custom_section_content.extend_from_slice(&binary::write_string("name"));
        custom_section_content.extend_from_slice(&name_section_bytes);

        // Add the custom section
        crate::component::utils::add_section(
            binary,
            binary::COMPONENT_CUSTOM_SECTION_ID,
            &custom_section_content,
        );
    }

    // Encode core module section
    if !component.modules.is_empty() {
        let module_data = encode_core_module_section(&component.modules)?;
        crate::component::utils::add_section(
            binary,
            binary::COMPONENT_CORE_MODULE_SECTION_ID,
            &module_data,
        );
    }

    // Encode core instance section
    if !component.core_instances.is_empty() {
        let instance_data = encode_core_instance_section(&component.core_instances)?;
        crate::component::utils::add_section(
            binary,
            binary::COMPONENT_CORE_INSTANCE_SECTION_ID,
            &instance_data,
        );
    }

    // Encode import section
    if !component.imports.is_empty() {
        let import_data = encode_import_section(&component.imports)?;
        crate::component::utils::add_section(
            binary,
            binary::COMPONENT_IMPORT_SECTION_ID,
            &import_data,
        );
    }

    // Encode export section
    if !component.exports.is_empty() {
        let export_data = encode_export_section(&component.exports)?;
        crate::component::utils::add_section(
            binary,
            binary::COMPONENT_EXPORT_SECTION_ID,
            &export_data,
        );
    }

    Ok(())
}

fn encode_core_module_section(modules: &[wrt_format::module::Module]) -> Result<Vec<u8>> {
    let mut data = Vec::new();

    // Write count of modules
    data.extend_from_slice(&binary::write_leb128_u32(modules.len() as u32));

    // Encode each module
    for module in modules {
        // Encode module binary
        let module_binary = module.binary.as_ref().ok_or_else(|| {
            wrt_error::Error::new(
                wrt_error::ErrorCategory::Parse,
                wrt_error::codes::PARSE_ERROR,
                "Module binary not available for encoding",
            )
        })?;

        // Write module size
        data.extend_from_slice(&binary::write_leb128_u32(module_binary.len() as u32));
        // Write module binary
        data.extend_from_slice(module_binary);
    }

    Ok(data)
}

fn encode_core_instance_section(
    instances: &[wrt_format::component::CoreInstance],
) -> Result<Vec<u8>> {
    let mut data = Vec::new();

    // Write count of instances
    data.extend_from_slice(&binary::write_leb128_u32(instances.len() as u32));

    // Encode each instance
    for instance in instances {
        match &instance.instance_expr {
            wrt_format::component::CoreInstanceExpr::Instantiate { module_idx, args } => {
                // Write instantiate tag
                data.push(binary::CORE_INSTANCE_INSTANTIATE_TAG);

                // Write module index
                data.extend_from_slice(&binary::write_leb128_u32(*module_idx));

                // Write count of arguments
                data.extend_from_slice(&binary::write_leb128_u32(args.len() as u32));

                // Write each argument
                for arg in args {
                    // Write argument name
                    data.extend_from_slice(&binary::write_string(&arg.name));
                    // Write instance index
                    data.extend_from_slice(&binary::write_leb128_u32(arg.instance_idx));
                }
            }
            wrt_format::component::CoreInstanceExpr::InlineExports(exports) => {
                // Write inline exports tag
                data.push(binary::CORE_INSTANCE_INLINE_EXPORTS_TAG);

                // Write count of exports
                data.extend_from_slice(&binary::write_leb128_u32(exports.len() as u32));

                // Write each export
                for export in exports {
                    // Write export name
                    data.extend_from_slice(&binary::write_string(&export.name));
                    // Write sort
                    data.push(export.sort as u8);
                    // Write index
                    data.extend_from_slice(&binary::write_leb128_u32(export.idx));
                }
            }
        }
    }

    Ok(data)
}

fn encode_import_section(imports: &[wrt_format::component::Import]) -> Result<Vec<u8>> {
    let mut data = Vec::new();

    // Write count of imports
    data.extend_from_slice(&binary::write_leb128_u32(imports.len() as u32));

    // Encode each import
    for import in imports {
        // Write import name
        data.extend_from_slice(&binary::write_string(&import.name.full_path()));

        // Write import type
        encode_extern_type(&import.ty, &mut data)?;
    }

    Ok(data)
}

fn encode_extern_type(ty: &wrt_format::component::ExternType, data: &mut Vec<u8>) -> Result<()> {
    match ty {
        wrt_format::component::ExternType::Function { params, results } => {
            // Write function tag
            data.push(binary::EXTERN_TYPE_FUNCTION_TAG);

            // Write parameter count
            data.extend_from_slice(&binary::write_leb128_u32(params.len() as u32));

            // Write each parameter
            for (name, param_ty) in params {
                // Write parameter name
                data.extend_from_slice(&binary::write_string(name));
                // Write parameter type
                encode_val_type(&format_val_type_to_val_type(param_ty), data)?;
            }

            // Write result count
            data.extend_from_slice(&binary::write_leb128_u32(results.len() as u32));

            // Write each result type
            for result_ty in results {
                encode_val_type(&format_val_type_to_val_type(result_ty), data)?;
            }
        }
        wrt_format::component::ExternType::Value(val_ty) => {
            // Write value tag
            data.push(binary::EXTERN_TYPE_VALUE_TAG);
            // Write value type
            encode_val_type(&format_val_type_to_val_type(val_ty), data)?;
        }
        wrt_format::component::ExternType::Type(type_idx) => {
            // Write type tag
            data.push(binary::EXTERN_TYPE_TYPE_TAG);
            // Write type index
            data.extend_from_slice(&binary::write_leb128_u32(*type_idx));
        }
        wrt_format::component::ExternType::Instance { exports } => {
            // Write instance tag
            data.push(binary::EXTERN_TYPE_INSTANCE_TAG);

            // Write export count
            data.extend_from_slice(&binary::write_leb128_u32(exports.len() as u32));

            // Write each export
            for (name, export_ty) in exports {
                // Write export name
                data.extend_from_slice(&binary::write_string(name));
                // Write export type
                encode_extern_type(export_ty, data)?;
            }
        }
        wrt_format::component::ExternType::Component { imports, exports } => {
            // Write component tag
            data.push(binary::EXTERN_TYPE_COMPONENT_TAG);

            // Write import count
            data.extend_from_slice(&binary::write_leb128_u32(imports.len() as u32));

            // Write each import
            for (namespace, name, import_ty) in imports {
                // Write namespace
                data.extend_from_slice(&binary::write_string(namespace));
                // Write name
                data.extend_from_slice(&binary::write_string(name));
                // Write import type
                encode_extern_type(import_ty, data)?;
            }

            // Write export count
            data.extend_from_slice(&binary::write_leb128_u32(exports.len() as u32));

            // Write each export
            for (name, export_ty) in exports {
                // Write export name
                data.extend_from_slice(&binary::write_string(name));
                // Write export type
                encode_extern_type(export_ty, data)?;
            }
        }
    }

    Ok(())
}

fn encode_val_type(ty: &wrt_format::component::ValType, data: &mut Vec<u8>) -> Result<()> {
    match ty {
        wrt_format::component::ValType::Bool => {
            data.push(binary::VAL_TYPE_BOOL_TAG);
        }
        wrt_format::component::ValType::S8 => {
            data.push(binary::VAL_TYPE_S8_TAG);
        }
        wrt_format::component::ValType::U8 => {
            data.push(binary::VAL_TYPE_U8_TAG);
        }
        wrt_format::component::ValType::S16 => {
            data.push(binary::VAL_TYPE_S16_TAG);
        }
        wrt_format::component::ValType::U16 => {
            data.push(binary::VAL_TYPE_U16_TAG);
        }
        wrt_format::component::ValType::S32 => {
            data.push(binary::VAL_TYPE_S32_TAG);
        }
        wrt_format::component::ValType::U32 => {
            data.push(binary::VAL_TYPE_U32_TAG);
        }
        wrt_format::component::ValType::S64 => {
            data.push(binary::VAL_TYPE_S64_TAG);
        }
        wrt_format::component::ValType::U64 => {
            data.push(binary::VAL_TYPE_U64_TAG);
        }
        wrt_format::component::ValType::F32 => {
            data.push(binary::VAL_TYPE_F32_TAG);
        }
        wrt_format::component::ValType::F64 => {
            data.push(binary::VAL_TYPE_F64_TAG);
        }
        wrt_format::component::ValType::Char => {
            data.push(binary::VAL_TYPE_CHAR_TAG);
        }
        wrt_format::component::ValType::String => {
            data.push(binary::VAL_TYPE_STRING_TAG);
        }
        wrt_format::component::ValType::Ref(type_idx) => {
            data.push(binary::VAL_TYPE_REF_TAG);
            data.extend_from_slice(&binary::write_leb128_u32(*type_idx));
        }
        wrt_format::component::ValType::Record(fields) => {
            data.push(binary::VAL_TYPE_RECORD_TAG);
            data.extend_from_slice(&binary::write_leb128_u32(fields.len() as u32));
            for (name, field_ty) in fields {
                data.extend_from_slice(&binary::write_string(name));
                encode_val_type(field_ty, data)?;
            }
        }
        wrt_format::component::ValType::Variant(cases) => {
            data.push(binary::VAL_TYPE_VARIANT_TAG);
            data.extend_from_slice(&binary::write_leb128_u32(cases.len() as u32));
            for (name, case_ty) in cases {
                data.extend_from_slice(&binary::write_string(name));
                match case_ty {
                    Some(ty) => {
                        data.push(1);
                        encode_val_type(ty, data)?;
                    }
                    None => {
                        data.push(0);
                    }
                }
            }
        }
        wrt_format::component::ValType::List(element_ty) => {
            data.push(binary::VAL_TYPE_LIST_TAG);
            encode_val_type(element_ty, data)?;
        }
        wrt_format::component::ValType::FixedList(element_ty, length) => {
            data.push(binary::VAL_TYPE_FIXED_LIST_TAG);
            encode_val_type(element_ty, data)?;
            data.extend_from_slice(&binary::write_leb128_u32(*length));
        }
        wrt_format::component::ValType::Tuple(types) => {
            data.push(binary::VAL_TYPE_TUPLE_TAG);
            data.extend_from_slice(&binary::write_leb128_u32(types.len() as u32));
            for ty in types {
                encode_val_type(ty, data)?;
            }
        }
        wrt_format::component::ValType::Flags(names) => {
            data.push(binary::VAL_TYPE_FLAGS_TAG);
            data.extend_from_slice(&binary::write_leb128_u32(names.len() as u32));
            for name in names {
                data.extend_from_slice(&binary::write_string(name));
            }
        }
        wrt_format::component::ValType::Enum(names) => {
            data.push(binary::VAL_TYPE_ENUM_TAG);
            data.extend_from_slice(&binary::write_leb128_u32(names.len() as u32));
            for name in names {
                data.extend_from_slice(&binary::write_string(name));
            }
        }
        wrt_format::component::ValType::Option(element_ty) => {
            data.push(binary::VAL_TYPE_OPTION_TAG);
            encode_val_type(element_ty, data)?;
        }
        wrt_format::component::ValType::Result(ok_ty) => {
            data.push(binary::VAL_TYPE_RESULT_TAG);
            encode_val_type(ok_ty, data)?;
        }
        wrt_format::component::ValType::ResultErr(err_ty) => {
            data.push(binary::VAL_TYPE_RESULT_ERR_TAG);
            encode_val_type(err_ty, data)?;
        }
        wrt_format::component::ValType::ResultBoth(ok_ty, err_ty) => {
            data.push(binary::VAL_TYPE_RESULT_BOTH_TAG);
            encode_val_type(ok_ty, data)?;
            encode_val_type(err_ty, data)?;
        }
        wrt_format::component::ValType::Own(type_idx) => {
            data.push(binary::VAL_TYPE_OWN_TAG);
            data.extend_from_slice(&binary::write_leb128_u32(*type_idx));
        }
        wrt_format::component::ValType::Borrow(type_idx) => {
            data.push(binary::VAL_TYPE_BORROW_TAG);
            data.extend_from_slice(&binary::write_leb128_u32(*type_idx));
        }
        wrt_format::component::ValType::Void => {
            // There doesn't seem to be a Void tag in the binary constants
            // We'll need to add this or map it to the appropriate value
            return Err(Error::validation_error("Void type encoding not yet implemented").into());
        }
        wrt_format::component::ValType::ErrorContext => {
            data.push(binary::VAL_TYPE_ERROR_CONTEXT_TAG);
        }
    }

    Ok(())
}

fn sort_to_u8(sort: &wrt_format::component::Sort) -> u8 {
    match sort {
        wrt_format::component::Sort::Core(core_sort) => match core_sort {
            wrt_format::component::CoreSort::Function => binary::COMPONENT_CORE_SORT_FUNC,
            wrt_format::component::CoreSort::Table => binary::COMPONENT_CORE_SORT_TABLE,
            wrt_format::component::CoreSort::Memory => binary::COMPONENT_CORE_SORT_MEMORY,
            wrt_format::component::CoreSort::Global => binary::COMPONENT_CORE_SORT_GLOBAL,
            wrt_format::component::CoreSort::Type => binary::COMPONENT_CORE_SORT_TYPE,
            wrt_format::component::CoreSort::Module => binary::COMPONENT_CORE_SORT_MODULE,
            wrt_format::component::CoreSort::Instance => binary::COMPONENT_CORE_SORT_INSTANCE,
        },
        wrt_format::component::Sort::Function => binary::COMPONENT_SORT_FUNC,
        wrt_format::component::Sort::Value => binary::COMPONENT_SORT_VALUE,
        wrt_format::component::Sort::Type => binary::COMPONENT_SORT_TYPE,
        wrt_format::component::Sort::Component => binary::COMPONENT_SORT_COMPONENT,
        wrt_format::component::Sort::Instance => binary::COMPONENT_SORT_INSTANCE,
    }
}

fn encode_export_section(exports: &[wrt_format::component::Export]) -> Result<Vec<u8>> {
    let mut data = Vec::new();

    // Write count of exports
    data.extend_from_slice(&binary::write_leb128_u32(exports.len() as u32));

    // Encode each export
    for export in exports {
        // Write export name
        data.extend_from_slice(&binary::write_string(&export.name.full_path()));

        // Write sort
        data.push(sort_to_u8(&export.sort));

        // Write index
        data.extend_from_slice(&binary::write_leb128_u32(export.idx));

        // Write declared type if present
        if let Some(ty) = &export.ty {
            data.push(1); // Type is present
            encode_extern_type(ty, &mut data)?;
        } else {
            data.push(0); // No type
        }
    }

    Ok(data)
}

/// Convert FormatValType to ValType
fn format_val_type_to_val_type(
    val_type: &wrt_format::component::FormatValType,
) -> wrt_format::component::ValType {
    match val_type {
        wrt_format::component::FormatValType::Bool => wrt_format::component::ValType::Bool,
        wrt_format::component::FormatValType::S8 => wrt_format::component::ValType::S8,
        wrt_format::component::FormatValType::U8 => wrt_format::component::ValType::U8,
        wrt_format::component::FormatValType::S16 => wrt_format::component::ValType::S16,
        wrt_format::component::FormatValType::U16 => wrt_format::component::ValType::U16,
        wrt_format::component::FormatValType::S32 => wrt_format::component::ValType::S32,
        wrt_format::component::FormatValType::U32 => wrt_format::component::ValType::U32,
        wrt_format::component::FormatValType::S64 => wrt_format::component::ValType::S64,
        wrt_format::component::FormatValType::U64 => wrt_format::component::ValType::U64,
        wrt_format::component::FormatValType::F32 => wrt_format::component::ValType::F32,
        wrt_format::component::FormatValType::F64 => wrt_format::component::ValType::F64,
        wrt_format::component::FormatValType::Char => wrt_format::component::ValType::Char,
        wrt_format::component::FormatValType::String => wrt_format::component::ValType::String,
        wrt_format::component::FormatValType::Ref(idx) => wrt_format::component::ValType::Ref(*idx),
        wrt_format::component::FormatValType::List(inner) => {
            wrt_format::component::ValType::List(Box::new(format_val_type_to_val_type(inner)))
        }
        wrt_format::component::FormatValType::FixedList(inner, len) => {
            wrt_format::component::ValType::FixedList(
                Box::new(format_val_type_to_val_type(inner)),
                *len,
            )
        }
        wrt_format::component::FormatValType::Tuple(items) => {
            wrt_format::component::ValType::Tuple(
                items.iter().map(format_val_type_to_val_type).collect(),
            )
        }
        wrt_format::component::FormatValType::Option(inner) => {
            wrt_format::component::ValType::Option(Box::new(format_val_type_to_val_type(inner)))
        }
        wrt_format::component::FormatValType::Result(ok) => {
            wrt_format::component::ValType::Result(Box::new(format_val_type_to_val_type(ok)))
        }
        wrt_format::component::FormatValType::Record(fields) => {
            wrt_format::component::ValType::Record(
                fields
                    .iter()
                    .map(|(name, ty)| (name.clone(), format_val_type_to_val_type(ty)))
                    .collect(),
            )
        }
        wrt_format::component::FormatValType::Variant(cases) => {
            wrt_format::component::ValType::Variant(
                cases
                    .iter()
                    .map(|(name, ty)| (name.clone(), ty.as_ref().map(format_val_type_to_val_type)))
                    .collect(),
            )
        }
        wrt_format::component::FormatValType::Flags(names) => {
            wrt_format::component::ValType::Flags(names.clone())
        }
        wrt_format::component::FormatValType::Enum(names) => {
            wrt_format::component::ValType::Enum(names.clone())
        }
        wrt_format::component::FormatValType::Own(idx) => wrt_format::component::ValType::Own(*idx),
        wrt_format::component::FormatValType::Borrow(idx) => {
            wrt_format::component::ValType::Borrow(*idx)
        }
        wrt_format::component::FormatValType::Void => wrt_format::component::ValType::Void,
        wrt_format::component::FormatValType::ErrorContext => {
            wrt_format::component::ValType::ErrorContext
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wrt_format::component::{
        Component, CoreInlineExport, CoreInstance, CoreInstanceExpr, CoreSort, Export, ExportName,
        Import, ImportName, Sort, ValType,
    };

    #[test]
    fn test_encode_empty_component() {
        let component = Component::new();
        let binary = encode_component(&component).unwrap();

        // Check magic and version
        assert_eq!(&binary[0..4], binary::COMPONENT_MAGIC);
        assert_eq!(&binary[4..8], binary::COMPONENT_VERSION);

        // Should only have magic and version
        assert_eq!(binary.len(), 8);
    }

    #[test]
    fn test_encode_component_with_name() {
        let mut component = Component::new();
        component.name = Some("test_component".to_string());

        let binary = encode_component(&component).unwrap();

        // Check magic and version
        assert_eq!(&binary[0..4], binary::COMPONENT_MAGIC);
        assert_eq!(&binary[4..8], binary::COMPONENT_VERSION);

        // Should have name section
        assert!(binary.len() > 8);
    }

    #[test]
    fn test_encode_component_with_core_instance() {
        let mut component = Component::new();

        // Add a core instance with inline exports
        let instance = CoreInstance {
            instance_expr: CoreInstanceExpr::InlineExports(vec![CoreInlineExport {
                name: "test_export".to_string(),
                sort: CoreSort::Function,
                idx: 0,
            }]),
        };
        component.core_instances.push(instance);

        let binary = encode_component(&component).unwrap();

        // Check magic and version
        assert_eq!(&binary[0..4], binary::COMPONENT_MAGIC);
        assert_eq!(&binary[4..8], binary::COMPONENT_VERSION);

        // Should have core instance section
        assert!(binary.len() > 8);
    }

    #[test]
    fn test_encode_component_with_imports() {
        let mut component = Component::new();

        // Add an import
        let import = Import {
            name: ImportName::new("test_namespace".to_string(), "test_import".to_string()),
            ty: wrt_format::component::ExternType::Value(ValType::String),
        };
        component.imports.push(import);

        let binary = encode_component(&component).unwrap();

        // Check magic and version
        assert_eq!(&binary[0..4], binary::COMPONENT_MAGIC);
        assert_eq!(&binary[4..8], binary::COMPONENT_VERSION);

        // Should have import section
        assert!(binary.len() > 8);
    }

    #[test]
    fn test_encode_component_with_exports() {
        let mut component = Component::new();

        // Add an export
        let export = Export {
            name: ExportName::new("test_export".to_string()),
            sort: Sort::Function,
            idx: 0,
            ty: None,
        };
        component.exports.push(export);

        let binary = encode_component(&component).unwrap();

        // Check magic and version
        assert_eq!(&binary[0..4], binary::COMPONENT_MAGIC);
        assert_eq!(&binary[4..8], binary::COMPONENT_VERSION);

        // Should have export section
        assert!(binary.len() > 8);
    }
}
