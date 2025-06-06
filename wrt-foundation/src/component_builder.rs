// WRT - wrt-foundation
// Module: Component Builders
//
// Copyright (c) 2025 Ralf Anton Beier
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! Builder pattern implementation for WebAssembly Component Model types.
//!
//! This module provides builders for complex types in the WebAssembly Component
//! Model, ensuring proper initialization, validation, and resource management.

#[cfg(all(not(feature = "std")))]
extern crate alloc;

#[cfg(all(not(feature = "std")))]
use std::vec::Vec;
#[cfg(feature = "std")]
use core::fmt::Debug;
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(feature = "std")]
use crate::{
    bounded::{BoundedString, BoundedVec, WasmName, MAX_WASM_NAME_LENGTH},
    component::{
        ComponentAlias, ComponentInstance, ComponentType, CoreInstance, CoreType, Export,
        ExternType, Import, ImportKey, Namespace, MAX_COMPONENT_ALIASES, MAX_COMPONENT_EXPORTS,
        MAX_COMPONENT_IMPORTS, MAX_COMPONENT_INSTANCES, MAX_COMPONENT_TYPES, MAX_CORE_INSTANCES,
        MAX_CORE_TYPES, MAX_NAMESPACE_ELEMENTS, MAX_NAME_LEN,
    },
    component_type_store::TypeRef,
    resource::{Resource, ResourceRepr, ResourceType},
    verification::VerificationLevel,
    Error, MemoryProvider, WrtResult,
};

#[cfg(feature = "std")]
/// Builder for `ComponentType` instances.
///
/// Provides a fluent API for constructing Component Model types with
/// proper initialization of all collections.
#[derive(Debug)]
pub struct ComponentTypeBuilder<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    provider: P,
    imports: Vec<Import<P>>,
    exports: Vec<Export<P>>,
    aliases: Vec<ComponentAlias<P>>,
    instances: Vec<ComponentInstance<P>>,
    core_instances: Vec<CoreInstance<P>>,
    component_types: Vec<TypeRef>,
    core_types: Vec<CoreType<P>>,
}

#[cfg(feature = "std")]
impl<P: MemoryProvider + Default + Clone + PartialEq + Eq> Default for ComponentTypeBuilder<P> {
    fn default() -> Self {
        Self {
            provider: P::default(),
            imports: Vec::new(),
            exports: Vec::new(),
            aliases: Vec::new(),
            instances: Vec::new(),
            core_instances: Vec::new(),
            component_types: Vec::new(),
            core_types: Vec::new(),
        }
    }
}

#[cfg(feature = "std")]
impl<P: MemoryProvider + Default + Clone + PartialEq + Eq> ComponentTypeBuilder<P> {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the memory provider to use.
    pub fn with_provider(mut self, provider: P) -> Self {
        self.provider = provider;
        self
    }

    /// Adds an import to the component type.
    pub fn with_import(mut self, import: Import<P>) -> Self {
        self.imports.push(import);
        self
    }

    /// Adds multiple imports to the component type.
    pub fn with_imports(mut self, imports: impl IntoIterator<Item = Import<P>>) -> Self {
        self.imports.extend(imports);
        self
    }

    /// Adds an export to the component type.
    pub fn with_export(mut self, export: Export<P>) -> Self {
        self.exports.push(export);
        self
    }

    /// Adds multiple exports to the component type.
    pub fn with_exports(mut self, exports: impl IntoIterator<Item = Export<P>>) -> Self {
        self.exports.extend(exports);
        self
    }

    /// Adds an alias to the component type.
    pub fn with_alias(mut self, alias: ComponentAlias<P>) -> Self {
        self.aliases.push(alias);
        self
    }

    /// Adds multiple aliases to the component type.
    pub fn with_aliases(mut self, aliases: impl IntoIterator<Item = ComponentAlias<P>>) -> Self {
        self.aliases.extend(aliases);
        self
    }

    /// Adds a component instance to the component type.
    pub fn with_instance(mut self, instance: ComponentInstance<P>) -> Self {
        self.instances.push(instance);
        self
    }

    /// Adds multiple component instances to the component type.
    pub fn with_instances(
        mut self,
        instances: impl IntoIterator<Item = ComponentInstance<P>>,
    ) -> Self {
        self.instances.extend(instances);
        self
    }

    /// Adds a core instance to the component type.
    pub fn with_core_instance(mut self, core_instance: CoreInstance<P>) -> Self {
        self.core_instances.push(core_instance);
        self
    }

    /// Adds multiple core instances to the component type.
    pub fn with_core_instances(
        mut self,
        core_instances: impl IntoIterator<Item = CoreInstance<P>>,
    ) -> Self {
        self.core_instances.extend(core_instances);
        self
    }

    /// Adds a component type reference to the component type.
    pub fn with_component_type(mut self, component_type: TypeRef) -> Self {
        self.component_types.push(component_type);
        self
    }

    /// Adds multiple component type references to the component type.
    pub fn with_component_types(
        mut self,
        component_types: impl IntoIterator<Item = TypeRef>,
    ) -> Self {
        self.component_types.extend(component_types);
        self
    }

    /// Adds a core type to the component type.
    pub fn with_core_type(mut self, core_type: CoreType<P>) -> Self {
        self.core_types.push(core_type);
        self
    }

    /// Adds multiple core types to the component type.
    pub fn with_core_types(mut self, core_types: impl IntoIterator<Item = CoreType<P>>) -> Self {
        self.core_types.extend(core_types);
        self
    }

    /// Builds and returns a configured `ComponentType`.
    pub fn build(self) -> WrtResult<ComponentType<P>> {
        // Create bounded vectors for each collection
        let mut imports = BoundedVec::new(self.provider.clone())?;
        let mut exports = BoundedVec::new(self.provider.clone())?;
        let mut aliases = BoundedVec::new(self.provider.clone())?;
        let mut instances = BoundedVec::new(self.provider.clone())?;
        let mut core_instances = BoundedVec::new(self.provider.clone())?;
        let mut component_types = BoundedVec::new(self.provider.clone())?;
        let mut core_types = BoundedVec::new(self.provider.clone())?;

        // Populate bounded vectors
        for import in self.imports {
            imports.push(import)?;
        }

        for export in self.exports {
            exports.push(export)?;
        }

        for alias in self.aliases {
            aliases.push(alias)?;
        }

        for instance in self.instances {
            instances.push(instance)?;
        }

        for core_instance in self.core_instances {
            core_instances.push(core_instance)?;
        }

        for component_type in self.component_types {
            component_types.push(component_type)?;
        }

        for core_type in self.core_types {
            core_types.push(core_type)?;
        }

        Ok(ComponentType {
            imports,
            exports,
            aliases,
            instances,
            core_instances,
            component_types,
            core_types,
        })
    }
}

#[cfg(feature = "std")]
/// Builder for `Import` instances.
#[derive(Debug)]
pub struct ImportBuilder<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    provider: P,
    namespace: Option<Namespace<P>>,
    name: Option<WasmName<MAX_NAME_LEN, P>>,
    ty: Option<ExternType<P>>,
}

#[cfg(feature = "std")]
impl<P: MemoryProvider + Default + Clone + PartialEq + Eq> Default for ImportBuilder<P> {
    fn default() -> Self {
        Self { provider: P::default(), namespace: None, name: None, ty: None }
    }
}

#[cfg(feature = "std")]
impl<P: MemoryProvider + Default + Clone + PartialEq + Eq> ImportBuilder<P> {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the memory provider to use.
    pub fn with_provider(mut self, provider: P) -> Self {
        self.provider = provider;
        self
    }

    /// Sets the namespace for the import.
    pub fn with_namespace(mut self, namespace: Namespace<P>) -> Self {
        self.namespace = Some(namespace);
        self
    }

    /// Sets the namespace for the import from a string.
    pub fn with_namespace_str(mut self, namespace_str: &str) -> WrtResult<Self> {
        let namespace = Namespace::from_str(namespace_str, self.provider.clone())?;
        self.namespace = Some(namespace);
        Ok(self)
    }

    /// Sets the name for the import.
    pub fn with_name(mut self, name: WasmName<MAX_NAME_LEN, P>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets the name for the import from a string.
    pub fn with_name_str(mut self, name_str: &str) -> WrtResult<Self> {
        let name = WasmName::from_str(name_str, self.provider.clone())?;
        self.name = Some(name);
        Ok(self)
    }

    /// Sets the type for the import.
    pub fn with_type(mut self, ty: ExternType<P>) -> Self {
        self.ty = Some(ty);
        self
    }

    /// Builds and returns a configured `Import`.
    pub fn build(self) -> WrtResult<Import<P>> {
        let namespace = self
            .namespace
            .ok_or_else(|| Error::validation_error("Import namespace is required"))?;

        let name = self.name.ok_or_else(|| Error::validation_error("Import name is required"))?;

        let ty = self.ty.ok_or_else(|| Error::validation_error("Import type is required"))?;

        Ok(Import { key: ImportKey { namespace, name }, ty })
    }
}

#[cfg(feature = "std")]
/// Builder for `Export` instances.
#[derive(Debug)]
pub struct ExportBuilder<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    provider: P,
    name: Option<WasmName<MAX_NAME_LEN, P>>,
    ty: Option<ExternType<P>>,
    desc: Option<WasmName<MAX_NAME_LEN, P>>,
}

#[cfg(feature = "std")]
impl<P: MemoryProvider + Default + Clone + PartialEq + Eq> Default for ExportBuilder<P> {
    fn default() -> Self {
        Self { provider: P::default(), name: None, ty: None, desc: None }
    }
}

#[cfg(feature = "std")]
impl<P: MemoryProvider + Default + Clone + PartialEq + Eq> ExportBuilder<P> {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the memory provider to use.
    pub fn with_provider(mut self, provider: P) -> Self {
        self.provider = provider;
        self
    }

    /// Sets the name for the export.
    pub fn with_name(mut self, name: WasmName<MAX_NAME_LEN, P>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets the name for the export from a string.
    pub fn with_name_str(mut self, name_str: &str) -> WrtResult<Self> {
        let name = WasmName::from_str(name_str, self.provider.clone())?;
        self.name = Some(name);
        Ok(self)
    }

    /// Sets the type for the export.
    pub fn with_type(mut self, ty: ExternType<P>) -> Self {
        self.ty = Some(ty);
        self
    }

    /// Sets the description for the export.
    pub fn with_description(mut self, desc: WasmName<MAX_NAME_LEN, P>) -> Self {
        self.desc = Some(desc);
        self
    }

    /// Sets the description for the export from a string.
    pub fn with_description_str(mut self, desc_str: &str) -> WrtResult<Self> {
        let desc = WasmName::from_str(desc_str, self.provider.clone())?;
        self.desc = Some(desc);
        Ok(self)
    }

    /// Builds and returns a configured `Export`.
    pub fn build(self) -> WrtResult<Export<P>> {
        let name = self.name.ok_or_else(|| Error::validation_error("Export name is required"))?;

        let ty = self.ty.ok_or_else(|| Error::validation_error("Export type is required"))?;

        Ok(Export { name, ty, desc: self.desc })
    }
}

#[cfg(feature = "std")]
/// Builder for `Namespace` instances.
#[derive(Debug)]
pub struct NamespaceBuilder<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    provider: P,
    elements: Vec<WasmName<MAX_NAME_LEN, P>>,
}

#[cfg(feature = "std")]
impl<P: MemoryProvider + Default + Clone + PartialEq + Eq> Default for NamespaceBuilder<P> {
    fn default() -> Self {
        Self { provider: P::default(), elements: Vec::new() }
    }
}

#[cfg(feature = "std")]
impl<P: MemoryProvider + Default + Clone + PartialEq + Eq> NamespaceBuilder<P> {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the memory provider to use.
    pub fn with_provider(mut self, provider: P) -> Self {
        self.provider = provider;
        self
    }

    /// Adds an element to the namespace.
    pub fn with_element(mut self, element: WasmName<MAX_NAME_LEN, P>) -> Self {
        self.elements.push(element);
        self
    }

    /// Adds an element to the namespace from a string.
    pub fn with_element_str(mut self, element_str: &str) -> WrtResult<Self> {
        let element = WasmName::from_str(element_str, self.provider.clone())?;
        self.elements.push(element);
        Ok(self)
    }

    /// Adds multiple elements to the namespace.
    pub fn with_elements(
        mut self,
        elements: impl IntoIterator<Item = WasmName<MAX_NAME_LEN, P>>,
    ) -> Self {
        self.elements.extend(elements);
        self
    }

    /// Creates a namespace from a colon-separated string.
    pub fn from_str(namespace_str: &str, provider: P) -> WrtResult<Self> {
        let mut builder = Self::new().with_provider(provider.clone());

        for part in namespace_str.split(':') {
            if !part.is_empty() {
                builder = builder.with_element_str(part)?;
            }
        }

        Ok(builder)
    }

    /// Builds and returns a configured `Namespace`.
    pub fn build(self) -> WrtResult<Namespace<P>> {
        let mut elements = BoundedVec::new(self.provider)?;

        for element in self.elements {
            elements.push(element)?;
        }

        Ok(Namespace { elements })
    }
}

#[cfg(feature = "std")]
/// Builder for `ResourceType` instances.
#[derive(Debug)]
pub struct ResourceTypeBuilder<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    provider: P,
    variant: Option<ResourceTypeVariant<P>>,
}

#[cfg(feature = "std")]
/// Enum to represent the variants of `ResourceType`.
#[derive(Debug)]
enum ResourceTypeVariant<P: MemoryProvider + Default + Clone + PartialEq + Eq> {
    Record(Vec<BoundedString<MAX_WASM_NAME_LENGTH, P>>),
    Aggregate(Vec<u32>),
}

#[cfg(feature = "std")]
impl<P: MemoryProvider + Default + Clone + PartialEq + Eq> Default for ResourceTypeBuilder<P> {
    fn default() -> Self {
        Self { provider: P::default(), variant: None }
    }
}

#[cfg(feature = "std")]
impl<P: MemoryProvider + Default + Clone + PartialEq + Eq> ResourceTypeBuilder<P> {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the memory provider to use.
    pub fn with_provider(mut self, provider: P) -> Self {
        self.provider = provider;
        self
    }

    /// Configures this as a record resource type with the given field names.
    pub fn as_record(mut self, field_names: Vec<BoundedString<MAX_WASM_NAME_LENGTH, P>>) -> Self {
        self.variant = Some(ResourceTypeVariant::Record(field_names));
        self
    }

    /// Configures this as an aggregate resource type with the given resource
    /// IDs.
    pub fn as_aggregate(mut self, resource_ids: Vec<u32>) -> Self {
        self.variant = Some(ResourceTypeVariant::Aggregate(resource_ids));
        self
    }

    /// Builds and returns a configured `ResourceType`.
    pub fn build(self) -> WrtResult<ResourceType<P>> {
        let variant = self
            .variant
            .ok_or_else(|| Error::validation_error("Resource type variant must be specified"))?;

        match variant {
            ResourceTypeVariant::Record(field_names) => {
                let mut fields = BoundedVec::new(self.provider.clone())?;
                for field in field_names {
                    // Convert BoundedString<MAX_WASM_NAME_LENGTH, P> to
                    // BoundedString<MAX_RESOURCE_FIELD_NAME_LEN, P>
                    let field_str = field.as_str().map_err(Error::from)?;
                    let provider_clone = self.provider.clone();
                    let converted_field = BoundedString::<
                        { crate::resource::MAX_RESOURCE_FIELD_NAME_LEN },
                        P,
                    >::from_str(field_str, provider_clone)
                    .map_err(Error::from)?;
                    fields.push(converted_field)?;
                }
                Ok(ResourceType::Record(fields))
            }
            ResourceTypeVariant::Aggregate(resource_ids) => {
                let mut ids = BoundedVec::new(self.provider)?;
                for id in resource_ids {
                    ids.push(id)?;
                }
                Ok(ResourceType::Aggregate(ids))
            }
        }
    }
}

#[cfg(all(test, ))]
mod tests {
    use super::*;
    use crate::{safe_memory::NoStdProvider, traits::BoundedCapacity};

    #[test]
    fn test_component_type_builder() {
        let provider = NoStdProvider::<1024>::default();

        let component_type =
            ComponentTypeBuilder::new().with_provider(provider.clone()).build().unwrap();

        // Verify empty collections were created
        assert_eq!(component_type.imports.len(), 0);
        assert_eq!(component_type.exports.len(), 0);
        assert_eq!(component_type.aliases.len(), 0);
        assert_eq!(component_type.instances.len(), 0);
        assert_eq!(component_type.core_instances.len(), 0);
        assert_eq!(component_type.component_types.len(), 0);
        assert_eq!(component_type.core_types.len(), 0);
    }

    #[test]
    fn test_namespace_builder() {
        let provider = NoStdProvider::<1024>::default();

        // Test building from parts
        let namespace_builder = NamespaceBuilder::new()
            .with_provider(provider.clone())
            .with_element_str("wasm")
            .unwrap()
            .with_element_str("component")
            .unwrap();

        let namespace = namespace_builder.build().unwrap();
        assert_eq!(namespace.elements.len(), 2);

        // Test building from string
        let namespace = NamespaceBuilder::from_str("wasm:component:model", provider.clone())
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(namespace.elements.len(), 3);
    }

    #[test]
    fn test_resource_type_builder() {
        let provider = NoStdProvider::<1024>::default();

        // Test record resource type
        let field_name = BoundedString::from_str("field", provider.clone()).unwrap();
        let resource_type = ResourceTypeBuilder::new()
            .with_provider(provider.clone())
            .as_record(vec![field_name])
            .build()
            .unwrap();

        match resource_type {
            ResourceType::Record(fields) => assert_eq!(fields.len(), 1),
            _ => panic!("Expected ResourceType::Record"),
        }

        // Test aggregate resource type
        let resource_type = ResourceTypeBuilder::new()
            .with_provider(provider.clone())
            .as_aggregate(vec![1, 2, 3])
            .build()
            .unwrap();

        match resource_type {
            ResourceType::Aggregate(ids) => assert_eq!(ids.len(), 3),
            _ => panic!("Expected ResourceType::Aggregate"),
        }
    }
}
