// Component format conversion module
//
// This module provides direct conversion between binary format types and runtime types
// without requiring intermediate conversions.

use wrt_error::Result;
use wrt_types::{FromFormat, ToFormat, ValueType};

// Temporary imported type aliases to make the transition easier
use crate::component::ValType as FormatValType;

// Implement FromFormat for ValueType to convert from FormatValType
impl FromFormat<FormatValType> for ValueType {
    fn from_format(format_type: &FormatValType) -> Self {
        match format_type {
            FormatValType::S8
            | FormatValType::U8
            | FormatValType::S16
            | FormatValType::U16
            | FormatValType::S32
            | FormatValType::U32
            | FormatValType::Bool
            | FormatValType::Char
            | FormatValType::Flags(_)
            | FormatValType::Enum(_)
            | FormatValType::ErrorContext => ValueType::I32,

            FormatValType::S64 | FormatValType::U64 => ValueType::I64,

            FormatValType::F32 => ValueType::F32,
            FormatValType::F64 => ValueType::F64,

            // References and handles
            FormatValType::String
            | FormatValType::Record(_)
            | FormatValType::Variant(_)
            | FormatValType::List(_)
            | FormatValType::FixedList(_, _)
            | FormatValType::Tuple(_)
            | FormatValType::Option(_)
            | FormatValType::Result(_)
            | FormatValType::ResultErr(_)
            | FormatValType::ResultBoth(_, _)
            | FormatValType::Own(_)
            | FormatValType::Borrow(_)
            | FormatValType::Ref(_) => ValueType::ExternRef,
        }
    }
}

// Implement ToFormat for ValueType to convert to FormatValType
impl ToFormat<FormatValType> for ValueType {
    fn to_format(&self) -> FormatValType {
        match self {
            ValueType::I32 => FormatValType::S32,
            ValueType::I64 => FormatValType::S64,
            ValueType::F32 => FormatValType::F32,
            ValueType::F64 => FormatValType::F64,
            ValueType::FuncRef => FormatValType::Own(0), // Map to handle
            ValueType::ExternRef => FormatValType::Own(0), // Map to handle
        }
    }
}

// Helper function for error handling when converting from format type
pub fn format_val_type_to_value_type(format_type: &FormatValType) -> Result<ValueType> {
    Ok(ValueType::from_format(format_type))
}

// Helper function for error handling when converting to format type
pub fn value_type_to_format_val_type(value_type: &ValueType) -> Result<FormatValType> {
    Ok(value_type.to_format())
}

// Map a core WebAssembly ValueType to a Component Model ValType
pub fn map_wasm_type_to_component(ty: ValueType) -> FormatValType {
    match ty {
        ValueType::I32 => FormatValType::S32,
        ValueType::I64 => FormatValType::S64,
        ValueType::F32 => FormatValType::F32,
        ValueType::F64 => FormatValType::F64,
        ValueType::FuncRef => FormatValType::Own(0), // Map to handle
        ValueType::ExternRef => FormatValType::Own(0), // Map to handle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type_conversion() {
        // Test basic primitive types
        let s32_val = FormatValType::S32;
        let i32_val = ValueType::from_format(&s32_val);
        assert_eq!(i32_val, ValueType::I32);

        let f64_val = FormatValType::F64;
        let f64_runtime = ValueType::from_format(&f64_val);
        assert_eq!(f64_runtime, ValueType::F64);

        // Test complex types (all map to ExternRef)
        let string_val = FormatValType::String;
        let string_runtime = ValueType::from_format(&string_val);
        assert_eq!(string_runtime, ValueType::ExternRef);

        // Test roundtrip conversion for basic types
        let i32_val = ValueType::I32;
        let format_val = i32_val.to_format();
        let roundtrip = ValueType::from_format(&format_val);
        assert_eq!(i32_val, roundtrip);
    }
}
