#[cfg(feature = "std")]
use std::{string::String, vec, vec::Vec};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec, vec::Vec};

/// Registry-based type conversions
///
/// This module implements conversions between format and runtime types using
/// the TypeConversionRegistry, providing a consistent and extensible approach
/// to type conversion.

#[cfg(feature = "std")]
use std::{string::String, vec, vec::Vec};

use wrt_format::component::{
    ComponentTypeDefinition, ExternType as FormatExternType, ValType<NoStdProvider<65536>> as FormatValType<NoStdProvider<65536>>,
};
use wrt_foundation::{
    component::{ComponentType, FuncType, InstanceType},
    component_value::ValType<NoStdProvider<65536>>,
    types::ValueType,
    ExternType as TypesExternType,
};

use super::{
    registry::{ConversionError, ConversionErrorKind, TypeConversionRegistry},
    wrappers::{
        FormatComponentType, FormatInstanceType, RuntimeComponentType, RuntimeInstanceType,
    },
};

/// Register ValType<NoStdProvider<65536>> conversions in the TypeConversionRegistry
pub fn register_valtype_conversions(registry: &mut TypeConversionRegistry) {
    // Format ValType<NoStdProvider<65536>> to Types ValType<NoStdProvider<65536>> - primitive types
    registry.register(|format_val_type: &FormatValType<NoStdProvider<65536>>| -> core::result::Result<ValType<NoStdProvider<65536>>, ConversionError> {
        match format_val_type {
            FormatValType<NoStdProvider<65536>>::Bool => Ok(ValType<NoStdProvider<65536>>::Bool),
            FormatValType<NoStdProvider<65536>>::S8 => Ok(ValType<NoStdProvider<65536>>::S8),
            FormatValType<NoStdProvider<65536>>::U8 => Ok(ValType<NoStdProvider<65536>>::U8),
            FormatValType<NoStdProvider<65536>>::S16 => Ok(ValType<NoStdProvider<65536>>::S16),
            FormatValType<NoStdProvider<65536>>::U16 => Ok(ValType<NoStdProvider<65536>>::U16),
            FormatValType<NoStdProvider<65536>>::S32 => Ok(ValType<NoStdProvider<65536>>::S32),
            FormatValType<NoStdProvider<65536>>::U32 => Ok(ValType<NoStdProvider<65536>>::U32),
            FormatValType<NoStdProvider<65536>>::S64 => Ok(ValType<NoStdProvider<65536>>::S64),
            FormatValType<NoStdProvider<65536>>::U64 => Ok(ValType<NoStdProvider<65536>>::U64),
            FormatValType<NoStdProvider<65536>>::F32 => Ok(ValType<NoStdProvider<65536>>::F32),
            FormatValType<NoStdProvider<65536>>::F64 => Ok(ValType<NoStdProvider<65536>>::F64),
            FormatValType<NoStdProvider<65536>>::Char => Ok(ValType<NoStdProvider<65536>>::Char),
            FormatValType<NoStdProvider<65536>>::String => Ok(ValType<NoStdProvider<65536>>::String),
            FormatValType<NoStdProvider<65536>>::Ref(idx) => Ok(ValType<NoStdProvider<65536>>::Ref(*idx)),
            FormatValType<NoStdProvider<65536>>::Flags(names) => Ok(ValType<NoStdProvider<65536>>::Flags(names.clone())),
            FormatValType<NoStdProvider<65536>>::Enum(cases) => Ok(ValType<NoStdProvider<65536>>::Enum(cases.clone())),
            FormatValType<NoStdProvider<65536>>::Own(idx) => Ok(ValType<NoStdProvider<65536>>::Own(*idx)),
            FormatValType<NoStdProvider<65536>>::Borrow(idx) => Ok(ValType<NoStdProvider<65536>>::Borrow(*idx)),
            // Complex types handled elsewhere or not supported
            _ => Err(ConversionError {
                kind: ConversionErrorKind::NotImplemented,
                source_type: "FormatValType<NoStdProvider<65536>>",
                target_type: "ValType<NoStdProvider<65536>>",
                context: Some("Complex type conversion requires registry capabilities".to_string()),
                source: None,
            }),
        }
    });

    // Types ValType<NoStdProvider<65536>> to Format ValType<NoStdProvider<65536>> - primitive types
    registry.register(|types_val_type: &ValType<NoStdProvider<65536>>| -> core::result::Result<FormatValType<NoStdProvider<65536>>, ConversionError> {
        match types_val_type {
            ValType<NoStdProvider<65536>>::Bool => Ok(FormatValType<NoStdProvider<65536>>::Bool),
            ValType<NoStdProvider<65536>>::S8 => Ok(FormatValType<NoStdProvider<65536>>::S8),
            ValType<NoStdProvider<65536>>::U8 => Ok(FormatValType<NoStdProvider<65536>>::U8),
            ValType<NoStdProvider<65536>>::S16 => Ok(FormatValType<NoStdProvider<65536>>::S16),
            ValType<NoStdProvider<65536>>::U16 => Ok(FormatValType<NoStdProvider<65536>>::U16),
            ValType<NoStdProvider<65536>>::S32 => Ok(FormatValType<NoStdProvider<65536>>::S32),
            ValType<NoStdProvider<65536>>::U32 => Ok(FormatValType<NoStdProvider<65536>>::U32),
            ValType<NoStdProvider<65536>>::S64 => Ok(FormatValType<NoStdProvider<65536>>::S64),
            ValType<NoStdProvider<65536>>::U64 => Ok(FormatValType<NoStdProvider<65536>>::U64),
            ValType<NoStdProvider<65536>>::F32 => Ok(FormatValType<NoStdProvider<65536>>::F32),
            ValType<NoStdProvider<65536>>::F64 => Ok(FormatValType<NoStdProvider<65536>>::F64),
            ValType<NoStdProvider<65536>>::Char => Ok(FormatValType<NoStdProvider<65536>>::Char),
            ValType<NoStdProvider<65536>>::String => Ok(FormatValType<NoStdProvider<65536>>::String),
            ValType<NoStdProvider<65536>>::Ref(idx) => Ok(FormatValType<NoStdProvider<65536>>::Ref(*idx)),
            ValType<NoStdProvider<65536>>::Flags(names) => Ok(FormatValType<NoStdProvider<65536>>::Flags(names.clone())),
            ValType<NoStdProvider<65536>>::Enum(cases) => Ok(FormatValType<NoStdProvider<65536>>::Enum(cases.clone())),
            ValType<NoStdProvider<65536>>::Own(idx) => Ok(FormatValType<NoStdProvider<65536>>::Own(*idx)),
            ValType<NoStdProvider<65536>>::Borrow(idx) => Ok(FormatValType<NoStdProvider<65536>>::Borrow(*idx)),
            // Complex types handled elsewhere or not supported
            _ => Err(ConversionError {
                kind: ConversionErrorKind::NotImplemented,
                source_type: "ValType<NoStdProvider<65536>>",
                target_type: "FormatValType<NoStdProvider<65536>>",
                context: Some("Complex type conversion requires registry capabilities".to_string()),
                source: None,
            }),
        }
    });

    // ValueType to FormatValType<NoStdProvider<65536>> conversion
    registry.register(|value_type: &ValueType| -> core::result::Result<FormatValType<NoStdProvider<65536>>, ConversionError> {
        match value_type {
            ValueType::I32 => Ok(FormatValType<NoStdProvider<65536>>::S32),
            ValueType::I64 => Ok(FormatValType<NoStdProvider<65536>>::S64),
            ValueType::F32 => Ok(FormatValType<NoStdProvider<65536>>::F32),
            ValueType::F64 => Ok(FormatValType<NoStdProvider<65536>>::F64),
            ValueType::FuncRef | ValueType::ExternRef => Err(ConversionError {
                kind: ConversionErrorKind::InvalidVariant,
                source_type: "ValueType::FuncRef/ExternRef",
                target_type: "FormatValType<NoStdProvider<65536>>",
                context: Some(
                    "Reference types cannot be directly converted to component format types"
                        .to_string(),
                ),
                source: None,
            }),
        }
    });

    // FormatValType<NoStdProvider<65536>> to ValueType conversion
    registry.register(|format_val_type: &FormatValType<NoStdProvider<65536>>| -> core::result::Result<ValueType, ConversionError> {
        match format_val_type {
            FormatValType<NoStdProvider<65536>>::S32 => Ok(ValueType::I32),
            FormatValType<NoStdProvider<65536>>::S64 => Ok(ValueType::I64),
            FormatValType<NoStdProvider<65536>>::F32 => Ok(ValueType::F32),
            FormatValType<NoStdProvider<65536>>::F64 => Ok(ValueType::F64),
            _ => Err(ConversionError {
                kind: ConversionErrorKind::InvalidVariant,
                source_type: "FormatValType<NoStdProvider<65536>>",
                target_type: "ValueType",
                context: Some("Component not found"),
                source: None,
            }),
        }
    });

    // ValueType to ValType<NoStdProvider<65536>> conversion
    registry.register(|value_type: &ValueType| -> core::result::Result<ValType<NoStdProvider<65536>>, ConversionError> {
        match value_type {
            ValueType::I32 => Ok(ValType<NoStdProvider<65536>>::S32),
            ValueType::I64 => Ok(ValType<NoStdProvider<65536>>::S64),
            ValueType::F32 => Ok(ValType<NoStdProvider<65536>>::F32),
            ValueType::F64 => Ok(ValType<NoStdProvider<65536>>::F64),
            ValueType::FuncRef => Ok(ValType<NoStdProvider<65536>>::Own(0)), // Default to resource type 0
            ValueType::ExternRef => Ok(ValType<NoStdProvider<65536>>::Ref(0)), // Default to type index 0
        }
    });
}

/// Register ExternType conversions in the TypeConversionRegistry
pub fn register_externtype_conversions(registry: &mut TypeConversionRegistry) {
    // Minimal implementation to support our simple tests
}

/// Register ComponentType and InstanceType conversions in the
/// TypeConversionRegistry
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
        let format_val_type = FormatValType<NoStdProvider<65536>>::S32;
        let types_val_type = registry.convert::<FormatValType<NoStdProvider<65536>>, ValType<NoStdProvider<65536>>>(&format_val_type).unwrap();
        assert!(matches!(types_val_type, ValType<NoStdProvider<65536>>::S32));

        // Test Types to Format ValType
        let types_val_type = ValType<NoStdProvider<65536>>::Bool;
        let format_val_type = registry.convert::<ValType<NoStdProvider<65536>>, FormatValType<NoStdProvider<65536>>>(&types_val_type).unwrap();
        assert!(matches!(format_val_type, FormatValType<NoStdProvider<65536>>::Bool));

        // Test ValueType to FormatValType
        let value_type = ValueType::I32;
        let format_val_type = registry.convert::<ValueType, FormatValType<NoStdProvider<65536>>>(&value_type).unwrap();
        assert!(matches!(format_val_type, FormatValType<NoStdProvider<65536>>::S32));
    }
}
