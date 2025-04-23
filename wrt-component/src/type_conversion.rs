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
            let converted_params: Vec<(String, TypesValType)> = params
                .iter()
                .map(|(name, val_type)| (name.clone(), format_valtype_to_types_valtype(val_type)))
                .collect();

            let converted_results: Vec<(String, TypesValType)> = results
                .iter()
                .map(|(name, val_type)| (name.clone(), format_valtype_to_types_valtype(val_type)))
                .collect();

            Ok(TypesExternType::Function(TypesFuncType {
                params: converted_params,
                results: converted_results,
            }))
        }
        FormatExternType::Instance { exports } => {
            // Convert instance type - this is simplified
            // Create a default InstanceType since we don't have proper way to convert yet
            Ok(TypesExternType::Instance(InstanceType { exports: vec![] }))
        }
        FormatExternType::Component { imports, exports } => {
            // Convert component type - this is simplified
            // Create a default ComponentType since we don't have proper way to convert yet
            Ok(TypesExternType::Component(ComponentType::default()))
        }
    }
}

/// Convert TypesExternType to FormatExternType
pub fn types_to_format_extern_type(
    types_extern_type: &TypesExternType,
) -> Result<FormatExternType> {
    match types_extern_type {
        TypesExternType::Function(func_type) => {
            let converted_params: Vec<(String, FormatValType)> = func_type
                .params
                .iter()
                .map(|(name, val_type)| (name.clone(), types_valtype_to_format_valtype(val_type)))
                .collect();

            let converted_results: Vec<(String, FormatValType)> = func_type
                .results
                .iter()
                .map(|(name, val_type)| (name.clone(), types_valtype_to_format_valtype(val_type)))
                .collect();

            Ok(FormatExternType::Function {
                params: converted_params,
                results: converted_results,
            })
        }
        TypesExternType::Instance(instance_type) => {
            // Convert instance type - simplified for now
            // Create an empty exports map for now
            Ok(FormatExternType::Instance { exports: vec![] })
        }
        TypesExternType::Component(component_type) => {
            // Convert component type - simplified for now
            Ok(FormatExternType::Component {
                imports: vec![],
                exports: vec![],
            })
        }
        TypesExternType::Resource(_) => {
            // wrt_format doesn't have a resource external type right now
            Err(Error::new(kinds::NotImplementedError(
                "Resource ExternType not supported in wrt_format".to_string(),
            )))
        }
    }
}
