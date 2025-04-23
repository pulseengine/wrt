/// Type conversion utilities for Component Model types
use wrt_error::{kinds, Error, Result};
use wrt_format::component::ValType as FormatValType;
use wrt_types::types::ValueType;
use wrt_types::ExternType;

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

/// Convert a FormatExternType to a TypesExternType
///
/// This is a placeholder for the actual implementation, which
/// should be implemented based on the requirements.
pub fn format_to_types_extern_type(
    format_extern_type: &wrt_format::component::ExternType,
) -> Result<ExternType> {
    Err(Error::new(kinds::NotImplementedError(
        "format_to_types_extern_type not implemented".to_string(),
    )))
}
