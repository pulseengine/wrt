//! Conversion utilities for WASM types
//!
//! This module contains functions to convert between format types and runtime types.

use wrt_error::errors::codes;
use wrt_error::{Error, ErrorCategory, Result};
use wrt_format::{section::CustomSection, Error as WrtFormatError, ValueType as FormatValueType};

// Import RefType directly from wrt-format
use wrt_format::RefType as FormatRefType;

// Import common types from prelude
use crate::prelude::*;

// Import types from wrt-types
use wrt_types::{
    types::{FuncType, GlobalType, Limits, MemoryType, RefType, TableType},
    ValueType,
};

/// Convert a format binary value type to runtime value type
///
/// This function maps the binary format value types (from wrt-format)
/// to the runtime value types (from wrt-types).
pub fn byte_to_value_type(byte: u8) -> Result<ValueType> {
    match byte {
        0x7F => Ok(ValueType::I32),
        0x7E => Ok(ValueType::I64),
        0x7D => Ok(ValueType::F32),
        0x7C => Ok(ValueType::F64),
        0x70 => Ok(ValueType::FuncRef),
        0x6F => Ok(ValueType::ExternRef),
        _ => Err(Error::new(
            ErrorCategory::Type,
            codes::INVALID_TYPE,
            "Invalid WebAssembly value type.",
        )),
    }
}

/// Convert a runtime value type to format binary value type
///
/// This function maps the runtime value types (from wrt-types)
/// to the binary format value types (from wrt-format).
pub fn value_type_to_byte(val_type: &ValueType) -> u8 {
    match val_type {
        ValueType::I32 => 0x7F,
        ValueType::I64 => 0x7E,
        ValueType::F32 => 0x7D,
        ValueType::F64 => 0x7C,
        ValueType::FuncRef => 0x70,
        ValueType::ExternRef => 0x6F,
    }
}

/// Convert a format error to a wrt error
pub fn format_error_to_wrt_error<E: Debug>(error: E) -> Error {
    let code = codes::PARSE_ERROR; // Default to generic parse error

    Error::new(
        ErrorCategory::Parse,
        code,
        format!("Format error: {error:?}"),
    )
}

/// Convert a format error into a wrt error
pub fn convert_to_wrt_error(error: WrtFormatError) -> Error {
    format_error_to_wrt_error(error)
}

/// Convert a section code into a section type
pub fn section_code_to_section_type(section_code: u8) -> wrt_format::section::Section {
    // Simple conversion to section enum
    match section_code {
        0 => wrt_format::section::Section::Custom(CustomSection {
            name: String::new(),
            data: Vec::new(),
        }),
        1 => wrt_format::section::Section::Type(Vec::new()),
        2 => wrt_format::section::Section::Import(Vec::new()),
        3 => wrt_format::section::Section::Function(Vec::new()),
        4 => wrt_format::section::Section::Table(Vec::new()),
        5 => wrt_format::section::Section::Memory(Vec::new()),
        6 => wrt_format::section::Section::Global(Vec::new()),
        7 => wrt_format::section::Section::Export(Vec::new()),
        8 => wrt_format::section::Section::Start(Vec::new()),
        9 => wrt_format::section::Section::Element(Vec::new()),
        10 => wrt_format::section::Section::Code(Vec::new()),
        11 => wrt_format::section::Section::Data(Vec::new()),
        12 => wrt_format::section::Section::DataCount(Vec::new()),
        _ => wrt_format::section::Section::Custom(CustomSection {
            name: format!("Unknown_{}", section_code),
            data: Vec::new(),
        }),
    }
}

/// Convert a section type into a section code
pub fn section_type_to_section_code(section_type: wrt_format::section::Section) -> u8 {
    // Simple conversion from section enum
    match section_type {
        wrt_format::section::Section::Custom(_) => 0,
        wrt_format::section::Section::Type(_) => 1,
        wrt_format::section::Section::Import(_) => 2,
        wrt_format::section::Section::Function(_) => 3,
        wrt_format::section::Section::Table(_) => 4,
        wrt_format::section::Section::Memory(_) => 5,
        wrt_format::section::Section::Global(_) => 6,
        wrt_format::section::Section::Export(_) => 7,
        wrt_format::section::Section::Start(_) => 8,
        wrt_format::section::Section::Element(_) => 9,
        wrt_format::section::Section::Code(_) => 10,
        wrt_format::section::Section::Data(_) => 11,
        wrt_format::section::Section::DataCount(_) => 12,
    }
}

/// Convert a format value type to a runtime value type
pub fn format_value_type_to_value_type(format_type: &FormatValueType) -> ValueType {
    match format_type {
        FormatValueType::I32 => ValueType::I32,
        FormatValueType::I64 => ValueType::I64,
        FormatValueType::F32 => ValueType::F32,
        FormatValueType::F64 => ValueType::F64,
        FormatValueType::FuncRef => ValueType::FuncRef,
        FormatValueType::ExternRef => ValueType::ExternRef,
    }
}

/// Convert a runtime value type to a format value type
pub fn value_type_to_format_value_type(value_type: &ValueType) -> FormatValueType {
    match value_type {
        ValueType::I32 => FormatValueType::I32,
        ValueType::I64 => FormatValueType::I64,
        ValueType::F32 => FormatValueType::F32,
        ValueType::F64 => FormatValueType::F64,
        ValueType::FuncRef => FormatValueType::FuncRef,
        ValueType::ExternRef => FormatValueType::ExternRef,
    }
}

/// Convert a sequence of format value types to runtime value types
pub fn format_value_types_to_value_types(format_types: &[FormatValueType]) -> Vec<ValueType> {
    format_types
        .iter()
        .map(format_value_type_to_value_type)
        .collect()
}

/// Convert format limits to runtime limits
pub fn format_limits_to_types_limits(format_limits: &wrt_format::types::Limits) -> Limits {
    Limits {
        min: format_limits.min as u32,
        max: format_limits.max.map(|m| m as u32),
    }
}

/// Convert runtime limits to format limits
pub fn types_limits_to_format_limits(types_limits: &Limits) -> wrt_format::types::Limits {
    wrt_format::types::Limits {
        min: types_limits.min as u64,
        max: types_limits.max.map(|m| m as u64),
        memory64: false,
        shared: false,
    }
}

/// Convert format limits to component limits
pub fn format_limits_to_component_limits(
    format_limits: &wrt_format::types::Limits,
) -> wrt_types::component::Limits {
    wrt_types::component::Limits {
        min: format_limits.min as u32,
        max: format_limits.max.map(|m| m as u32),
    }
}

/// Convert component limits to format limits
pub fn component_limits_to_format_limits(
    comp_limits: &wrt_types::component::Limits,
) -> wrt_format::types::Limits {
    wrt_format::types::Limits {
        min: comp_limits.min as u64,
        max: comp_limits.max.map(|m| m as u64),
        memory64: false,
        shared: false,
    }
}

/// Convert format ref type to runtime ref type
pub fn format_ref_type_to_types_ref_type(format_type: &FormatRefType) -> RefType {
    match format_type {
        FormatRefType::Funcref => RefType::Funcref,
        FormatRefType::Externref => RefType::Externref,
    }
}

/// Convert runtime ref type to format ref type
pub fn types_ref_type_to_format_ref_type(types_type: &RefType) -> FormatRefType {
    match types_type {
        RefType::Funcref => FormatRefType::Funcref,
        RefType::Externref => FormatRefType::Externref,
    }
}

/// Convert a format function type to a runtime function type
pub fn format_func_type_to_types_func_type(format_type: &wrt_format::FuncType) -> FuncType {
    FuncType::new(
        format_value_types_to_value_types(&format_type.params),
        format_value_types_to_value_types(&format_type.results),
    )
}

/// Convert a format global type to a runtime global type
pub fn format_global_type_to_types_global_type(
    format_type: &wrt_types::types::GlobalType,
) -> GlobalType {
    GlobalType {
        value_type: format_value_type_to_value_type(&value_type_to_format_value_type(
            &format_type.value_type,
        )),
        mutable: format_type.mutable,
    }
}

/// Convert a format memory type to a runtime memory type
pub fn format_memory_type_to_types_memory_type(
    format_type: &wrt_format::module::Memory,
) -> MemoryType {
    MemoryType {
        limits: format_limits_to_types_limits(&format_type.limits),
        shared: format_type.shared,
    }
}

/// Convert a format table type to a runtime table type
pub fn format_table_type_to_types_table_type(format_type: &wrt_format::module::Table) -> TableType {
    TableType {
        element_type: format_value_type_to_value_type(&format_type.element_type),
        limits: format_limits_to_types_limits(&format_type.limits),
    }
}
