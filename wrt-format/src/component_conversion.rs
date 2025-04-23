// Component format conversion module
//
// This module provides direct conversion between binary format types and runtime types
// without requiring intermediate conversions.

use wrt_error::{kinds, Error, Result};
use wrt_types::{FromFormat, ToFormat, ValueType};

// Temporary imported type aliases to make the transition easier
use crate::component::ValType as FormatValType;

// Implement FromFormat for ValueType to convert from FormatValType
impl FromFormat<FormatValType> for ValueType {
    fn from_format(format_type: FormatValType) -> Result<Self> {
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
            | FormatValType::Enum(_) => Ok(ValueType::I32),

            FormatValType::S64 | FormatValType::U64 => Ok(ValueType::I64),

            FormatValType::F32 => Ok(ValueType::F32),
            FormatValType::F64 => Ok(ValueType::F64),

            // References and handles
            FormatValType::String
            | FormatValType::Record(_)
            | FormatValType::Variant(_)
            | FormatValType::List(_)
            | FormatValType::Tuple(_)
            | FormatValType::Option(_)
            | FormatValType::Result(_)
            | FormatValType::ResultErr(_)
            | FormatValType::ResultBoth(_, _)
            | FormatValType::Own(_)
            | FormatValType::Borrow(_)
            | FormatValType::Ref(_) => Ok(ValueType::ExternRef),
        }
    }
}

// Implement ToFormat for ValueType to convert to FormatValType
impl ToFormat<FormatValType> for ValueType {
    fn to_format(&self) -> Result<FormatValType> {
        match self {
            ValueType::I32 => Ok(FormatValType::S32),
            ValueType::I64 => Ok(FormatValType::S64),
            ValueType::F32 => Ok(FormatValType::F32),
            ValueType::F64 => Ok(FormatValType::F64),
            ValueType::V128 => Err(Error::new(kinds::ConversionError(
                "V128 not supported in Component Model".to_string(),
            ))),
            ValueType::FuncRef => Ok(FormatValType::Ref(0)), // Use a default reference index
            ValueType::ExternRef => Ok(FormatValType::String), // Default to string format
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type_conversion() {
        // Test basic primitive types
        let s32_val = FormatValType::S32;
        let i32_val = ValueType::from_format(s32_val).unwrap();
        assert_eq!(i32_val, ValueType::I32);

        let f64_val = FormatValType::F64;
        let f64_runtime = ValueType::from_format(f64_val).unwrap();
        assert_eq!(f64_runtime, ValueType::F64);

        // Test complex types (all map to ExternRef)
        let string_val = FormatValType::String;
        let string_runtime = ValueType::from_format(string_val).unwrap();
        assert_eq!(string_runtime, ValueType::ExternRef);

        // Test roundtrip conversion for basic types
        let i32_val = ValueType::I32;
        let format_val = i32_val.to_format().unwrap();
        let roundtrip = ValueType::from_format(format_val).unwrap();
        assert_eq!(i32_val, roundtrip);
    }
}
