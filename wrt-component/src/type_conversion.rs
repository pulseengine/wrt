/// Type conversion utilities for Component Model types
use wrt_error::{kinds, Error, Result};
use wrt_format::component::ExternType as FormatExternType;
use wrt_format::component::ValType as FormatValType;
use wrt_format::component::{ComponentTypeDefinition, FuncType as FormatFuncType};
use wrt_types::component::{ComponentType, FuncType as TypesFuncType, InstanceType, ResourceType};
use wrt_types::component_value::ValType as TypesValType;
use wrt_types::types::ValueType;
use wrt_types::ExternType as TypesExternType;

/// Convert a ValueType to a FormatValType
///
/// This function converts from wrt_types::types::ValueType to
/// wrt_format::component::ValType directly.
pub fn value_type_to_format_val_type(value_type: &ValueType) -> Result<FormatValType> {
    match value_type {
        ValueType::I32 => Ok(FormatValType::S32),
        ValueType::I64 => Ok(FormatValType::S64),
        ValueType::F32 => Ok(FormatValType::F32),
        ValueType::F64 => Ok(FormatValType::F64),
        ValueType::V128 => Err(Error::new(kinds::NotImplementedError(
            "V128 type not supported in Component Model".to_string(),
        ))),
        ValueType::FuncRef => Err(Error::new(kinds::NotImplementedError(
            "FuncRef type not directly convertible to component format".to_string(),
        ))),
        ValueType::ExternRef => Err(Error::new(kinds::NotImplementedError(
            "ExternRef type not directly convertible to component format".to_string(),
        ))),
    }
}

/// Convert a FormatValType to a ValueType
///
/// This function converts from wrt_format::component::ValType to
/// wrt_types::types::ValueType, handling only the primitive types that
/// can be directly mapped.
pub fn format_val_type_to_value_type(format_val_type: &FormatValType) -> Result<ValueType> {
    match format_val_type {
        FormatValType::S32 => Ok(ValueType::I32),
        FormatValType::S64 => Ok(ValueType::I64),
        FormatValType::F32 => Ok(ValueType::F32),
        FormatValType::F64 => Ok(ValueType::F64),
        _ => Err(Error::new(kinds::NotImplementedError(format!(
            "Cannot convert {:?} to core ValueType",
            format_val_type
        )))),
    }
}

/// Convert ValueType to TypesValType
pub fn value_type_to_types_valtype(value_type: &ValueType) -> TypesValType {
    match value_type {
        ValueType::I32 => TypesValType::S32,
        ValueType::I64 => TypesValType::S64,
        ValueType::F32 => TypesValType::F32,
        ValueType::F64 => TypesValType::F64,
        ValueType::V128 => TypesValType::Tuple(vec![TypesValType::S64, TypesValType::S64]),
        ValueType::FuncRef => TypesValType::Own(0), // Default to resource type 0
        ValueType::ExternRef => TypesValType::Ref(0), // Default to type index 0
    }
}

/// Convert FormatValType to TypesValType
pub fn format_valtype_to_types_valtype(format_val_type: &FormatValType) -> TypesValType {
    match format_val_type {
        FormatValType::Bool => TypesValType::Bool,
        FormatValType::S8 => TypesValType::S8,
        FormatValType::U8 => TypesValType::U8,
        FormatValType::S16 => TypesValType::S16,
        FormatValType::U16 => TypesValType::U16,
        FormatValType::S32 => TypesValType::S32,
        FormatValType::U32 => TypesValType::U32,
        FormatValType::S64 => TypesValType::S64,
        FormatValType::U64 => TypesValType::U64,
        FormatValType::F32 => TypesValType::F32,
        FormatValType::F64 => TypesValType::F64,
        FormatValType::Char => TypesValType::Char,
        FormatValType::String => TypesValType::String,
        FormatValType::Ref(idx) => TypesValType::Ref(*idx),
        FormatValType::Record(fields) => {
            let converted_fields = fields
                .iter()
                .map(|(name, val_type)| (name.clone(), format_valtype_to_types_valtype(val_type)))
                .collect();
            TypesValType::Record(converted_fields)
        }
        FormatValType::Variant(cases) => {
            let converted_cases = cases
                .iter()
                .map(|(name, opt_type)| {
                    (
                        name.clone(),
                        opt_type
                            .as_ref()
                            .map(|val_type| format_valtype_to_types_valtype(val_type)),
                    )
                })
                .collect();
            TypesValType::Variant(converted_cases)
        }
        FormatValType::List(elem_type) => {
            TypesValType::List(Box::new(format_valtype_to_types_valtype(elem_type)))
        }
        FormatValType::Tuple(types) => {
            let converted_types = types
                .iter()
                .map(|val_type| format_valtype_to_types_valtype(val_type))
                .collect();
            TypesValType::Tuple(converted_types)
        }
        FormatValType::Flags(names) => TypesValType::Flags(names.clone()),
        FormatValType::Enum(variants) => TypesValType::Enum(variants.clone()),
        FormatValType::Option(inner_type) => {
            TypesValType::Option(Box::new(format_valtype_to_types_valtype(inner_type)))
        }
        FormatValType::Result(ok_type) => {
            TypesValType::Result(Box::new(format_valtype_to_types_valtype(ok_type)))
        }
        FormatValType::ResultErr(err_type) => {
            TypesValType::Result(Box::new(format_valtype_to_types_valtype(err_type)))
        }
        FormatValType::ResultBoth(ok_type, err_type) => {
            // Create a tuple type containing both ok and err types
            TypesValType::Result(Box::new(TypesValType::Tuple(vec![
                format_valtype_to_types_valtype(ok_type),
                format_valtype_to_types_valtype(err_type),
            ])))
        }
        FormatValType::Own(idx) => TypesValType::Own(*idx),
        FormatValType::Borrow(idx) => TypesValType::Borrow(*idx),
    }
}

/// Convert TypesValType to FormatValType
pub fn types_valtype_to_format_valtype(types_val_type: &TypesValType) -> FormatValType {
    match types_val_type {
        TypesValType::Bool => FormatValType::Bool,
        TypesValType::S8 => FormatValType::S8,
        TypesValType::U8 => FormatValType::U8,
        TypesValType::S16 => FormatValType::S16,
        TypesValType::U16 => FormatValType::U16,
        TypesValType::S32 => FormatValType::S32,
        TypesValType::U32 => FormatValType::U32,
        TypesValType::S64 => FormatValType::S64,
        TypesValType::U64 => FormatValType::U64,
        TypesValType::F32 => FormatValType::F32,
        TypesValType::F64 => FormatValType::F64,
        TypesValType::Char => FormatValType::Char,
        TypesValType::String => FormatValType::String,
        TypesValType::List(elem_type) => {
            FormatValType::List(Box::new(types_valtype_to_format_valtype(elem_type)))
        }
        TypesValType::Record(fields) => {
            let format_fields = fields
                .iter()
                .map(|(name, val_type)| (name.clone(), types_valtype_to_format_valtype(val_type)))
                .collect();
            FormatValType::Record(format_fields)
        }
        TypesValType::Variant(cases) => {
            let format_cases = cases
                .iter()
                .map(|(name, val_type)| {
                    (
                        name.clone(),
                        val_type
                            .as_ref()
                            .map(|vt| types_valtype_to_format_valtype(vt)),
                    )
                })
                .collect();
            FormatValType::Variant(format_cases)
        }
        TypesValType::Enum(cases) => FormatValType::Enum(cases.clone()),
        TypesValType::Option(inner_type) => {
            FormatValType::Option(Box::new(types_valtype_to_format_valtype(inner_type)))
        }
        TypesValType::Result(inner_type) => {
            // Check if the inner type is a tuple with 2 elements (ok and err)
            if let TypesValType::Tuple(tuple_types) = &**inner_type {
                if tuple_types.len() == 2 {
                    FormatValType::ResultBoth(
                        Box::new(types_valtype_to_format_valtype(&tuple_types[0])),
                        Box::new(types_valtype_to_format_valtype(&tuple_types[1])),
                    )
                } else {
                    FormatValType::Result(Box::new(types_valtype_to_format_valtype(inner_type)))
                }
            } else {
                FormatValType::Result(Box::new(types_valtype_to_format_valtype(inner_type)))
            }
        }
        TypesValType::Tuple(types) => {
            let format_types = types
                .iter()
                .map(|val_type| types_valtype_to_format_valtype(val_type))
                .collect();
            FormatValType::Tuple(format_types)
        }
        TypesValType::Flags(names) => FormatValType::Flags(names.clone()),
        TypesValType::Own(type_idx) => FormatValType::Own(*type_idx),
        TypesValType::Borrow(type_idx) => FormatValType::Borrow(*type_idx),
    }
}

/// Convert FormatExternType to TypesExternType
pub fn format_to_types_extern_type(
    format_extern_type: &FormatExternType,
) -> Result<TypesExternType> {
    match format_extern_type {
        FormatExternType::Function { params, results } => {
            // Convert parameter and result types from FormatValType to ValueType
            let params_value_types: Result<Vec<ValueType>> = params
                .iter()
                .map(|(_, val_type)| format_val_type_to_value_type(val_type))
                .collect();

            let results_value_types: Result<Vec<ValueType>> =
                results.iter().map(format_val_type_to_value_type).collect();

            Ok(TypesExternType::Function(wrt_types::component::FuncType {
                params: params_value_types?,
                results: results_value_types?,
            }))
        }
        FormatExternType::Instance { exports } => {
            // Convert exports to TypesExternType
            let converted_exports: Result<Vec<(String, TypesExternType)>> = exports
                .iter()
                .map(|(name, ext_type)| Ok((name.clone(), format_to_types_extern_type(ext_type)?)))
                .collect();

            Ok(TypesExternType::Instance(InstanceType {
                exports: converted_exports?,
            }))
        }
        FormatExternType::Component { imports, exports } => {
            // Convert imports to TypesExternType
            let converted_imports: Result<Vec<(String, String, TypesExternType)>> = imports
                .iter()
                .map(|(ns, name, ext_type)| {
                    Ok((
                        ns.clone(),
                        name.clone(),
                        format_to_types_extern_type(ext_type)?,
                    ))
                })
                .collect();

            // Convert exports to TypesExternType
            let converted_exports: Result<Vec<(String, TypesExternType)>> = exports
                .iter()
                .map(|(name, ext_type)| Ok((name.clone(), format_to_types_extern_type(ext_type)?)))
                .collect();

            Ok(TypesExternType::Component(ComponentType {
                imports: converted_imports?,
                exports: converted_exports?,
                instances: vec![],
            }))
        }
        FormatExternType::Value(val_type) => {
            // Handle value type - convert to appropriate TypesExternType
            Err(Error::new(kinds::NotImplementedError(
                "Value ExternType not supported in wrt_types".to_string(),
            )))
        }
        FormatExternType::Type(type_idx) => {
            // Handle type reference - convert to appropriate TypesExternType
            Err(Error::new(kinds::NotImplementedError(
                "Type reference ExternType not supported in wrt_types".to_string(),
            )))
        }
    }
}

/// Convert TypesExternType to FormatExternType
pub fn types_to_format_extern_type(
    types_extern_type: &TypesExternType,
) -> Result<FormatExternType> {
    match types_extern_type {
        TypesExternType::Function(func_type) => {
            // Convert parameter and result types to named parameters for FormatExternType
            let converted_params: Result<Vec<(String, FormatValType)>> = func_type
                .params
                .iter()
                .enumerate()
                .map(|(i, val_type)| {
                    let param_name = format!("p{}", i);
                    let format_val_type = value_type_to_format_val_type(val_type)?;
                    Ok((param_name, format_val_type))
                })
                .collect();

            // Convert result types to unnamed results for FormatExternType
            let converted_results: Result<Vec<(String, FormatValType)>> = func_type
                .results
                .iter()
                .enumerate()
                .map(|(i, val_type)| {
                    let result_name = format!("r{}", i);
                    let format_val_type = value_type_to_format_val_type(val_type)?;
                    Ok((result_name, format_val_type))
                })
                .collect();

            Ok(FormatExternType::Function {
                params: converted_params?,
                results: converted_results?,
            })
        }
        TypesExternType::Instance(instance_type) => {
            // Convert exports to FormatExternType
            let converted_exports: Result<Vec<(String, FormatExternType)>> = instance_type
                .exports
                .iter()
                .map(|(name, ext_type)| Ok((name.clone(), types_to_format_extern_type(ext_type)?)))
                .collect();

            Ok(FormatExternType::Instance {
                exports: converted_exports?,
            })
        }
        TypesExternType::Component(component_type) => {
            // Convert imports to FormatExternType
            let converted_imports: Result<Vec<(String, String, FormatExternType)>> = component_type
                .imports
                .iter()
                .map(|(ns, name, ext_type)| {
                    Ok((
                        ns.clone(),
                        name.clone(),
                        types_to_format_extern_type(ext_type)?,
                    ))
                })
                .collect();

            // Convert exports to FormatExternType
            let converted_exports: Result<Vec<(String, FormatExternType)>> = component_type
                .exports
                .iter()
                .map(|(name, ext_type)| Ok((name.clone(), types_to_format_extern_type(ext_type)?)))
                .collect();

            Ok(FormatExternType::Component {
                imports: converted_imports?,
                exports: converted_exports?,
            })
        }
        TypesExternType::Table(table_type) => {
            // wrt_format doesn't have a table external type at the component level
            Err(Error::new(kinds::NotImplementedError(
                "Table ExternType not supported in wrt_format::component".to_string(),
            )))
        }
        TypesExternType::Memory(memory_type) => {
            // wrt_format doesn't have a memory external type at the component level
            Err(Error::new(kinds::NotImplementedError(
                "Memory ExternType not supported in wrt_format::component".to_string(),
            )))
        }
        TypesExternType::Global(global_type) => {
            // wrt_format doesn't have a global external type at the component level
            Err(Error::new(kinds::NotImplementedError(
                "Global ExternType not supported in wrt_format::component".to_string(),
            )))
        }
        TypesExternType::Resource(resource_type) => {
            // wrt_format doesn't have a direct resource external type
            Err(Error::new(kinds::NotImplementedError(
                "Resource ExternType not directly supported in wrt_format".to_string(),
            )))
        }
    }
}

/// Convert the format ValType to the common ValueType used in the runtime
pub fn format_to_common_val_type(val_type: &FormatValType) -> Result<ValueType> {
    match val_type {
        FormatValType::S32 => Ok(ValueType::I32),
        FormatValType::S64 => Ok(ValueType::I64),
        FormatValType::F32 => Ok(ValueType::F32),
        FormatValType::F64 => Ok(ValueType::F64),
        _ => Err(Error::new(kinds::NotImplementedError(format!(
            "Cannot convert {:?} to core ValueType",
            val_type
        )))),
    }
}

/// Convert the common ValueType to a format ValType
pub fn common_to_format_val_type(value_type: &ValueType) -> Result<FormatValType> {
    match value_type {
        ValueType::I32 => Ok(FormatValType::S32),
        ValueType::I64 => Ok(FormatValType::S64),
        ValueType::F32 => Ok(FormatValType::F32),
        ValueType::F64 => Ok(FormatValType::F64),
        _ => Err(Error::new(kinds::NotImplementedError(format!(
            "Value type {:?} cannot be directly mapped to component format",
            value_type
        )))),
    }
}

/// Convert an ExternType to a FuncType if it represents a function
///
/// # Arguments
///
/// * `extern_type` - The external type to convert
///
/// # Returns
///
/// The function type if the extern type is a function, or an error otherwise
pub fn extern_type_to_func_type(
    extern_type: &wrt_types::ExternType,
) -> Result<wrt_types::component::FuncType> {
    match extern_type {
        wrt_types::ExternType::Function(func_type) => Ok(func_type.clone()),
        _ => Err(Error::new(kinds::TypeMismatchError(format!(
            "Expected Function ExternType, got {:?}",
            extern_type
        )))),
    }
}
