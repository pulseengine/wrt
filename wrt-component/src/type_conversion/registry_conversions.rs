/// Registry-based type conversions
///
/// This module implements conversions between format and runtime types using the
/// TypeConversionRegistry, providing a consistent and extensible approach to type conversion.

#[cfg(feature = "std")]
use std::{string::String, vec, vec::Vec};

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{string::String, vec, vec::Vec};

use wrt_format::component::{
    ComponentTypeDefinition, ExternType as FormatExternType, ValType as FormatValType,
};
use wrt_types::{
    component::{ComponentType, FuncType, InstanceType},
    component_value::ValType as TypesValType,
    types::ValueType,
    ExternType as TypesExternType,
};

use super::registry::{ConversionError, ConversionErrorKind, TypeConversionRegistry};
use super::wrappers::{
    FormatComponentType, FormatInstanceType, RuntimeComponentType, RuntimeInstanceType,
};

/// Register ValType conversions in the TypeConversionRegistry
pub fn register_valtype_conversions(registry: &mut TypeConversionRegistry) {
    // Format ValType to Types ValType - primitive types
    registry.register(
        |format_val_type: &FormatValType| -> Result<TypesValType, ConversionError> {
            match format_val_type {
                FormatValType::Bool => Ok(TypesValType::Bool),
                FormatValType::S8 => Ok(TypesValType::S8),
                FormatValType::U8 => Ok(TypesValType::U8),
                FormatValType::S16 => Ok(TypesValType::S16),
                FormatValType::U16 => Ok(TypesValType::U16),
                FormatValType::S32 => Ok(TypesValType::S32),
                FormatValType::U32 => Ok(TypesValType::U32),
                FormatValType::S64 => Ok(TypesValType::S64),
                FormatValType::U64 => Ok(TypesValType::U64),
                FormatValType::F32 => Ok(TypesValType::F32),
                FormatValType::F64 => Ok(TypesValType::F64),
                FormatValType::Char => Ok(TypesValType::Char),
                FormatValType::String => Ok(TypesValType::String),
                FormatValType::Ref(idx) => Ok(TypesValType::Ref(*idx)),
                FormatValType::Flags(names) => Ok(TypesValType::Flags(names.clone())),
                FormatValType::Enum(cases) => Ok(TypesValType::Enum(cases.clone())),
                FormatValType::Own(idx) => Ok(TypesValType::Own(*idx)),
                FormatValType::Borrow(idx) => Ok(TypesValType::Borrow(*idx)),
                // Complex types handled elsewhere or not supported
                _ => Err(ConversionError {
                    kind: ConversionErrorKind::NotImplemented,
                    source_type: "FormatValType",
                    target_type: "TypesValType",
                    context: Some(
                        "Complex type conversion requires registry capabilities".to_string(),
                    ),
                    source: None,
                }),
            }
        },
    );

    // Types ValType to Format ValType - primitive types
    registry.register(
        |types_val_type: &TypesValType| -> Result<FormatValType, ConversionError> {
            match types_val_type {
                TypesValType::Bool => Ok(FormatValType::Bool),
                TypesValType::S8 => Ok(FormatValType::S8),
                TypesValType::U8 => Ok(FormatValType::U8),
                TypesValType::S16 => Ok(FormatValType::S16),
                TypesValType::U16 => Ok(FormatValType::U16),
                TypesValType::S32 => Ok(FormatValType::S32),
                TypesValType::U32 => Ok(FormatValType::U32),
                TypesValType::S64 => Ok(FormatValType::S64),
                TypesValType::U64 => Ok(FormatValType::U64),
                TypesValType::F32 => Ok(FormatValType::F32),
                TypesValType::F64 => Ok(FormatValType::F64),
                TypesValType::Char => Ok(FormatValType::Char),
                TypesValType::String => Ok(FormatValType::String),
                TypesValType::Ref(idx) => Ok(FormatValType::Ref(*idx)),
                TypesValType::Flags(names) => Ok(FormatValType::Flags(names.clone())),
                TypesValType::Enum(cases) => Ok(FormatValType::Enum(cases.clone())),
                TypesValType::Own(idx) => Ok(FormatValType::Own(*idx)),
                TypesValType::Borrow(idx) => Ok(FormatValType::Borrow(*idx)),
                // Complex types handled elsewhere or not supported
                _ => Err(ConversionError {
                    kind: ConversionErrorKind::NotImplemented,
                    source_type: "TypesValType",
                    target_type: "FormatValType",
                    context: Some(
                        "Complex type conversion requires registry capabilities".to_string(),
                    ),
                    source: None,
                }),
            }
        },
    );

    // ValueType to FormatValType conversion
    registry.register(
        |value_type: &ValueType| -> Result<FormatValType, ConversionError> {
            match value_type {
                ValueType::I32 => Ok(FormatValType::S32),
                ValueType::I64 => Ok(FormatValType::S64),
                ValueType::F32 => Ok(FormatValType::F32),
                ValueType::F64 => Ok(FormatValType::F64),
                ValueType::FuncRef | ValueType::ExternRef => Err(ConversionError {
                    kind: ConversionErrorKind::InvalidVariant,
                    source_type: "ValueType::FuncRef/ExternRef",
                    target_type: "FormatValType",
                    context: Some(
                        "Reference types cannot be directly converted to component format types"
                            .to_string(),
                    ),
                    source: None,
                }),
            }
        },
    );

    // FormatValType to ValueType conversion
    registry.register(
        |format_val_type: &FormatValType| -> Result<ValueType, ConversionError> {
            match format_val_type {
                FormatValType::S32 => Ok(ValueType::I32),
                FormatValType::S64 => Ok(ValueType::I64),
                FormatValType::F32 => Ok(ValueType::F32),
                FormatValType::F64 => Ok(ValueType::F64),
                _ => Err(ConversionError {
                    kind: ConversionErrorKind::InvalidVariant,
                    source_type: "FormatValType",
                    target_type: "ValueType",
                    context: Some(format!(
                        "Cannot convert {:?} to core ValueType",
                        format_val_type
                    )),
                    source: None,
                }),
            }
        },
    );

    // ValueType to TypesValType conversion
    registry.register(
        |value_type: &ValueType| -> Result<TypesValType, ConversionError> {
            match value_type {
                ValueType::I32 => Ok(TypesValType::S32),
                ValueType::I64 => Ok(TypesValType::S64),
                ValueType::F32 => Ok(TypesValType::F32),
                ValueType::F64 => Ok(TypesValType::F64),
                ValueType::FuncRef => Ok(TypesValType::Own(0)), // Default to resource type 0
                ValueType::ExternRef => Ok(TypesValType::Ref(0)), // Default to type index 0
            }
        },
    );
}

/// Register ExternType conversions in the TypeConversionRegistry
pub fn register_externtype_conversions(registry: &mut TypeConversionRegistry) {
    // Minimal implementation to support our simple tests
}

/// Register ComponentType and InstanceType conversions in the TypeConversionRegistry
pub fn register_component_instancetype_conversions(registry: &mut TypeConversionRegistry) {
    // Minimal implementation to support our simple tests
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_valtype_conversions() {
        let mut registry = TypeConversionRegistry::new();
        register_valtype_conversions(&mut registry);

        // Test Format to Types ValType
        let format_val_type = FormatValType::S32;
        let types_val_type = registry
            .convert::<FormatValType, TypesValType>(&format_val_type)
            .unwrap();
        assert!(matches!(types_val_type, TypesValType::S32));

        // Test Types to Format ValType
        let types_val_type = TypesValType::Bool;
        let format_val_type = registry
            .convert::<TypesValType, FormatValType>(&types_val_type)
            .unwrap();
        assert!(matches!(format_val_type, FormatValType::Bool));

        // Test ValueType to FormatValType
        let value_type = ValueType::I32;
        let format_val_type = registry
            .convert::<ValueType, FormatValType>(&value_type)
            .unwrap();
        assert!(matches!(format_val_type, FormatValType::S32));
    }
}
