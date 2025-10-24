// WRT - wrt-component
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! No-std compatible instance implementation for the WebAssembly Component
//! Model.
//!
//! This module provides the instance types for component instances in a no_std
//! environment.

use wrt_format::component::ComponentTypeDefinition;
use wrt_foundation::{
    bounded::{
        BoundedString,
        BoundedVec,
        MAX_WASM_NAME_LENGTH,
    },
    budget_aware_provider::CrateId,
    safe_managed_alloc,
    safe_memory::NoStdProvider,
    traits::{Checksummable, FromBytes, ToBytes},
};

use crate::{
    export::Export,
    prelude::*,
};

/// Maximum number of exports per instance
pub const MAX_INSTANCE_EXPORTS: usize = 64;

/// No-std compatible representation of an instance value in a component
#[derive(Debug, Clone)]
pub struct InstanceValue {
    /// The name of the instance
    pub name:    BoundedString<MAX_WASM_NAME_LENGTH>,
    /// Instance type
    pub ty:      ComponentTypeDefinition,
    /// Instance exports
    pub exports: BoundedVec<Export, MAX_INSTANCE_EXPORTS, NoStdProvider<16384>>,
}

impl PartialEq for InstanceValue {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for InstanceValue {}

impl Default for InstanceValue {
    fn default() -> Self {
        use wrt_format::component::ComponentTypeDefinition;
        Self {
            name: BoundedString::from_str_truncate("")
                .unwrap_or_else(|_| panic!("Failed to create default InstanceValue name")),
            ty: ComponentTypeDefinition::Instance { exports: vec![] },
            exports: BoundedVec::new(NoStdProvider::default()).unwrap(),
        }
    }
}

impl Checksummable for InstanceValue {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        if let Ok(bytes) = self.name.as_bytes() {
            checksum.update_slice(bytes.as_ref());
        }
    }
}

impl ToBytes for InstanceValue {
    fn to_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut wrt_foundation::traits::WriteStream<'a>,
        _provider: &P,
    ) -> wrt_error::Result<()> {
        // Simple stub implementation - just write the name length
        writer.write_u32_le(self.name.len() as u32)?;
        Ok(())
    }
}

impl FromBytes for InstanceValue {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        _reader: &mut wrt_foundation::traits::ReadStream<'a>,
        _provider: &P,
    ) -> wrt_error::Result<Self> {
        Ok(Self::default())
    }
}

impl InstanceValue {
    /// Creates a new instance value
    pub fn new(name: &str, ty: ComponentTypeDefinition, exports: &[Export]) -> Result<Self> {
        let name_provider = safe_managed_alloc!(512, CrateId::Component)?;
        let bounded_name = BoundedString::try_from_str(name)
            .map_err(|_| Error::parameter_validation_error("Instance name too long"))?;

        let provider = safe_managed_alloc!(16384, CrateId::Component)?;
        let mut bounded_exports = BoundedVec::new(provider)?;
        for export in exports {
            bounded_exports
                .push(export.clone())
                .map_err(|_| Error::capacity_exceeded("Maximum number of exports exceeded"))?;
        }

        Ok(Self {
            name: bounded_name,
            ty,
            exports: bounded_exports,
        })
    }

    /// Gets an export by name
    pub fn get_export(&self, name: &str) -> Option<Export> {
        self.exports.iter().find(|export| export.name == name)
    }

    /// Gets a mutable export by name
    /// Note: Due to BoundedVec's serialization architecture, this returns an owned value
    /// that can be modified and set back using other methods
    pub fn get_export_mut(&mut self, name: &str) -> Option<Export> {
        // BoundedVecIteratorMut doesn't have find, so we need to iterate manually
        for i in 0..self.exports.len() {
            if let Ok(export) = self.exports.get(i) {
                if export.name == name {
                    return Some(export);
                }
            }
        }
        None
    }

    /// Creates a builder for this instance value
    pub fn builder() -> InstanceValueBuilder {
        InstanceValueBuilder::new()
    }
}

/// Builder for InstanceValue
pub struct InstanceValueBuilder {
    /// The name of the instance
    name:    Option<String>,
    /// Instance type
    ty:      Option<ComponentTypeDefinition>,
    /// Instance exports
    exports: Vec<Export>,
}

impl InstanceValueBuilder {
    /// Creates a new instance value builder
    pub fn new() -> Self {
        Self {
            name:    None,
            ty:      None,
            exports: Vec::new(),
        }
    }

    /// Sets the instance name
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Sets the instance type
    pub fn with_type(mut self, ty: ComponentTypeDefinition) -> Self {
        self.ty = Some(ty);
        self
    }

    /// Adds an export to the instance
    pub fn with_export(mut self, export: Export) -> Self {
        self.exports.push(export);
        self
    }

    /// Adds multiple exports to the instance
    pub fn with_exports(mut self, exports: Vec<Export>) -> Self {
        self.exports.extend(exports);
        self
    }

    /// Builds the instance value
    pub fn build(self) -> Result<InstanceValue> {
        let name = self
            .name
            .ok_or_else(|| Error::parameter_validation_error("Instance name is required"))?;

        let ty = self
            .ty
            .ok_or_else(|| Error::parameter_validation_error("Instance type is required"))?;

        InstanceValue::new(&name, ty, &self.exports)
    }
}

impl Default for InstanceValueBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A collection of instances with bounded capacity
pub struct InstanceCollection {
    instances: BoundedVec<InstanceValue, MAX_INSTANCE_EXPORTS, NoStdProvider<65536>>,
}

impl InstanceCollection {
    /// Creates a new empty instance collection
    pub fn new() -> Result<Self> {
        let provider = safe_managed_alloc!(65536, CrateId::Component)?;
        Ok(Self {
            instances: BoundedVec::new(provider)?,
        })
    }

    /// Adds an instance to the collection
    pub fn add_instance(&mut self, instance: InstanceValue) -> Result<()> {
        self.instances
            .push(instance)
            .map_err(|_| Error::capacity_exceeded("Maximum number of instances exceeded"))
    }

    /// Gets an instance by name
    pub fn get_instance(&self, name: &str) -> Option<InstanceValue> {
        self.instances.iter().find(|instance| instance.name.as_str().ok() == Some(name))
    }

    /// Gets a mutable instance by name
    /// Note: Due to BoundedVec's serialization architecture, this returns an owned value
    /// that can be modified and set back using other methods
    pub fn get_instance_mut(&mut self, name: &str) -> Option<InstanceValue> {
        // BoundedVecIteratorMut doesn't have find, so we need to iterate manually
        for i in 0..self.instances.len() {
            if let Ok(instance) = self.instances.get(i) {
                if instance.name.as_str().ok() == Some(name) {
                    return Some(instance);
                }
            }
        }
        None
    }

    /// Returns the number of instances in the collection
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    /// Checks if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }

    /// Returns an iterator over the instances
    /// Note: Due to BoundedVec's serialization architecture, this returns owned values
    pub fn iter(&self) -> impl Iterator<Item = InstanceValue> + '_ {
        self.instances.iter()
    }
}

impl Default for InstanceCollection {
    fn default() -> Self {
        Self::new().expect("Failed to create default InstanceCollection")
    }
}

#[cfg(test)]
mod tests {
    use wrt::types::{
        FuncType,
        ValueType,
    };
    use wrt_format::component::{
        ExternType,
        ValType,
    };

    use super::*;
    use crate::export::Export;

    #[test]
    fn test_instance_value_builder() {
        let func_type = FuncType {
            params:  vec![ValueType::I32, ValueType::I32],
            results: vec![ValueType::I32],
        };

        // Since most of the types require ownership, we'll mock them for testing
        // In a real implementation, we would need to use the actual types
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
            // Mocking this as we can't create an actual ExternValue in tests
            value: None,
        };

        let instance_type = ComponentTypeDefinition::Instance {
            exports: vec![("add".to_owned(), component_func_type)],
        };

        // Build the instance
        let instance = InstanceValue::builder()
            .with_name("math")
            .with_type(instance_type)
            .with_export(export)
            .build()
            .unwrap();

        // Check the instance
        assert_eq!(instance.name.as_str().unwrap(), "math");
        assert_eq!(instance.exports.len(), 1);
        assert_eq!(instance.get_export("add").unwrap().name, "add");
        assert!(instance.get_export("non_existent").is_none());
    }

    #[test]
    fn test_instance_collection() {
        // Create a simple instance
        let instance_type = ComponentTypeDefinition::Instance { exports: vec![] };

        let instance1 = InstanceValue::builder()
            .with_name("instance1")
            .with_type(instance_type.clone())
            .build()
            .unwrap();

        let instance2 = InstanceValue::builder()
            .with_name("instance2")
            .with_type(instance_type)
            .build()
            .unwrap();

        // Create a collection and add the instances
        let mut collection = InstanceCollection::new().unwrap();
        assert!(collection.is_empty());

        collection.add_instance(instance1).unwrap();
        collection.add_instance(instance2).unwrap();

        // Check the collection
        assert_eq!(collection.len(), 2);
        assert!(!collection.is_empty());
        assert!(collection.get_instance("instance1").is_some());
        assert!(collection.get_instance("instance2").is_some());
        assert!(collection.get_instance("non_existent").is_none());

        // Test iteration
        let names: Vec<&str> = collection
            .iter()
            .filter_map(|i| i.name.as_str().ok())
            .collect();
        assert_eq!(names, vec!["instance1", "instance2"]);
    }
}
