//! Instance implementation for the WebAssembly Component Model.
//!
//! This module provides the instance types for component instances.

#[cfg(not(feature = "std"))]
use alloc::{
    string::String,
    vec::Vec,
};
#[cfg(feature = "std")]
use std::{
    string::String,
    vec::Vec,
};

use wrt_format::component::ComponentTypeDefinition;

use crate::export::Export;

// Type alias for compatibility
pub type Instance = InstanceValue;

/// Represents an instance value in a component
#[derive(Debug)]
pub struct InstanceValue {
    /// The name of the instance
    pub name:    String,
    /// Instance type
    pub ty:      ComponentTypeDefinition,
    /// Instance exports
    pub exports: Vec<Export>,
}

impl InstanceValue {
    /// Creates a new instance value
    pub fn new(name: String, ty: ComponentTypeDefinition, exports: Vec<Export>) -> Self {
        Self { name, ty, exports }
    }

    /// Gets an export by name
    pub fn get_export(&self, name: &str) -> Option<&Export> {
        self.exports.iter().find(|export| export.name == name)
    }

    /// Gets a mutable export by name
    pub fn get_export_mut(&mut self, name: &str) -> Option<&mut Export> {
        self.exports.iter_mut().find(|export| export.name == name)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "std"))]
    use alloc::vec;
    #[cfg(feature = "std")]
    use std::vec;

    use wrt_foundation::types::{
        FuncType,
        ValueType,
    };
    use wrt_format::component::{
        ExternType,
        ValType,
    };

    use super::*;
    use crate::components::component::{
        ExternValue,
        FunctionValue,
    };

    #[test]
    fn test_instance_value() {
        let func_type = FuncType {
            params:  vec![ValueType::I32, ValueType::I32],
            results: vec![ValueType::I32],
        };

        let func_value = FunctionValue {
            ty:          func_type.clone(),
            export_name: "add".to_owned(),
        };

        let component_func_type = ExternType::Func {
            params:  vec![
                ("a".to_owned(), ValType::S32),
                ("b".to_owned(), ValType::S32),
            ],
            results: vec![ValType::S32],
        };

        let export = Export {
            name:  "add".to_owned(),
            ty:    component_func_type.clone(),
            value: ExternValue::Function(func_value),
        };

        let instance_type = ComponentTypeDefinition::Instance {
            exports: vec![("add".to_owned(), component_func_type)],
        };

        let instance = InstanceValue::new("math".to_owned(), instance_type, vec![export]);

        assert_eq!(instance.name, "math");
        assert_eq!(instance.exports.len(), 1);

        let export = instance.get_export("add");
        assert!(export.is_some());
        assert_eq!(export.unwrap().name, "add");

        let not_found = instance.get_export("non_existent");
        assert!(not_found.is_none());
    }
}
