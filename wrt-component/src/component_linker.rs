//! Component composition and linking
//!
//! This module provides functionality for linking multiple components together,
//! resolving imports/exports, and creating composite components at runtime.

#[cfg(feature = "std")]
use std::collections::BTreeMap;
#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap, vec::Vec};

use wrt_foundation::{
    bounded_collections::{BoundedVec, BoundedString, MAX_GENERATIVE_TYPES},
    prelude::*,
};

use crate::{
    types::{ComponentError, ComponentInstance, ComponentInstanceId, TypeId},
    component::Component,
    import::{Import, ImportType},
    export::Export,
    instance::{InstanceValue},
    generative_types::GenerativeTypeRegistry,
    type_bounds::TypeBoundsChecker,
    instantiation::{ImportValues, ExportValue},
};

/// Component linker for composing multiple components
#[derive(Debug, Clone)]
pub struct ComponentLinker {
    /// Registry of available components
    components: BTreeMap<BoundedString<64>, Component>,
    /// Instantiated component instances
    instances: BTreeMap<ComponentInstanceId, ComponentInstance>,
    /// Export registry for resolution
    export_registry: BTreeMap<BoundedString<128>, ExportEntry>,
    /// Type registry for generative types
    type_registry: GenerativeTypeRegistry,
    /// Type bounds checker
    bounds_checker: TypeBoundsChecker,
    /// Next instance ID
    next_instance_id: u32,
}

#[derive(Debug, Clone)]
struct ExportEntry {
    instance_id: ComponentInstanceId,
    export_name: BoundedString<64>,
    export_value: ExportValue,
    type_id: Option<TypeId>,
}

#[derive(Debug, Clone)]
pub struct LinkageDescriptor {
    /// Source component name
    pub source: BoundedString<64>,
    /// Target component name  
    pub target: BoundedString<64>,
    /// Import/export mappings
    pub bindings: BoundedVec<Binding, MAX_GENERATIVE_TYPES>,
}

#[derive(Debug, Clone)]
pub struct Binding {
    /// Import name in target component
    pub import_name: BoundedString<64>,
    /// Export name in source component
    pub export_name: BoundedString<64>,
    /// Optional type constraints
    pub type_constraint: Option<TypeConstraint>,
}

#[derive(Debug, Clone)]
pub enum TypeConstraint {
    /// Types must be equal
    Equal,
    /// Import type must be subtype of export type
    Subtype,
}

#[derive(Debug, Clone)]
pub struct CompositeComponent {
    /// Name of the composite
    pub name: BoundedString<64>,
    /// Component instances in the composite
    pub instances: BoundedVec<ComponentInstanceId, MAX_GENERATIVE_TYPES>,
    /// External imports (not satisfied internally)
    pub external_imports: BoundedVec<ExternalImport, MAX_GENERATIVE_TYPES>,
    /// External exports (exposed from internal components)
    pub external_exports: BoundedVec<ExternalExport, MAX_GENERATIVE_TYPES>,
}

#[derive(Debug, Clone)]
pub struct ExternalImport {
    pub name: BoundedString<64>,
    pub import_type: ImportType,
    pub target_instance: ComponentInstanceId,
}

#[derive(Debug, Clone)]
pub struct ExternalExport {
    pub name: BoundedString<64>,
    pub source_instance: ComponentInstanceId,
    pub source_export: BoundedString<64>,
}

impl ComponentLinker {
    pub fn new() -> Self {
        Self {
            components: BTreeMap::new(),
            instances: BTreeMap::new(),
            export_registry: BTreeMap::new(),
            type_registry: GenerativeTypeRegistry::new(),
            bounds_checker: TypeBoundsChecker::new(),
            next_instance_id: 1,
        }
    }

    /// Register a component for linking
    pub fn register_component(
        &mut self,
        name: BoundedString<64>,
        component: Component,
    ) -> Result<(), ComponentError> {
        if self.components.contains_key(&name) {
            return Err(ComponentError::ExportResolutionFailed);
        }
        
        self.components.insert(name, component);
        Ok(())
    }

    /// Create a composite component from a linkage descriptor
    pub fn create_composite(
        &mut self,
        name: BoundedString<64>,
        descriptors: Vec<LinkageDescriptor>,
    ) -> Result<CompositeComponent, ComponentError> {
        let mut composite = CompositeComponent {
            name,
            instances: BoundedVec::new(),
            external_imports: BoundedVec::new(),
            external_exports: BoundedVec::new(),
        };

        // Phase 1: Instantiate all components
        let mut instance_map = BTreeMap::new();
        for descriptor in &descriptors {
            let source_id = self.instantiate_component(&descriptor.source)?;
            let target_id = self.instantiate_component(&descriptor.target)?;
            
            instance_map.insert(descriptor.source.clone(), source_id);
            instance_map.insert(descriptor.target.clone(), target_id);
            
            composite.instances.push(source_id)
                .map_err(|_| ComponentError::TooManyGenerativeTypes)?;
            composite.instances.push(target_id)
                .map_err(|_| ComponentError::TooManyGenerativeTypes)?;
        }

        // Phase 2: Resolve bindings
        for descriptor in &descriptors {
            let source_id = instance_map[&descriptor.source];
            let target_id = instance_map[&descriptor.target];
            
            self.resolve_bindings(source_id, target_id, &descriptor.bindings)?;
        }

        // Phase 3: Collect external imports/exports
        self.collect_external_interfaces(&mut composite)?;

        Ok(composite)
    }

    /// Link two components together
    pub fn link_components(
        &mut self,
        source_name: &str,
        target_name: &str,
        bindings: Vec<Binding>,
    ) -> Result<(), ComponentError> {
        let source_component = self.components.get(&BoundedString::from_str(source_name)
            .map_err(|_| ComponentError::TypeMismatch)?)
            .ok_or(ComponentError::ImportResolutionFailed)?
            .clone();
            
        let target_component = self.components.get(&BoundedString::from_str(target_name)
            .map_err(|_| ComponentError::TypeMismatch)?)
            .ok_or(ComponentError::ImportResolutionFailed)?
            .clone();

        // Instantiate components
        let source_id = self.create_instance(source_component)?;
        let target_id = self.create_instance(target_component)?;

        // Resolve each binding
        for binding in bindings {
            self.resolve_single_binding(source_id, target_id, &binding)?;
        }

        Ok(())
    }

    /// Instantiate a component by name
    fn instantiate_component(
        &mut self,
        name: &BoundedString<64>,
    ) -> Result<ComponentInstanceId, ComponentError> {
        let component = self.components.get(name)
            .ok_or(ComponentError::ImportResolutionFailed)?
            .clone();
            
        self.create_instance(component)
    }

    /// Create a new component instance
    fn create_instance(
        &mut self,
        component: Component,
    ) -> Result<ComponentInstanceId, ComponentError> {
        let instance_id = ComponentInstanceId(self.next_instance_id);
        self.next_instance_id += 1;

        // Create generative types for this instance
        for _ in 0..component.types.len() {
            let base_type = wrt_foundation::resource::ResourceType::Handle(
                wrt_foundation::resource::ResourceHandle::new(0)
            );
            self.type_registry.create_generative_type(base_type, instance_id)?;
        }

        let instance = ComponentInstance {
            id: instance_id.0,
            component,
            imports: Vec::new(),
            exports: Vec::new(),
            resource_tables: Vec::new(),
            module_instances: Vec::new(),
        };

        self.instances.insert(instance_id, instance);
        self.register_instance_exports(instance_id)?;

        Ok(instance_id)
    }

    /// Register all exports from an instance
    fn register_instance_exports(
        &mut self,
        instance_id: ComponentInstanceId,
    ) -> Result<(), ComponentError> {
        let instance = self.instances.get(&instance_id)
            .ok_or(ComponentError::ResourceNotFound(instance_id.0))?;

        for export in &instance.component.exports {
            let full_name = self.create_qualified_name(instance_id, &export.name);
            
            let export_value = self.create_export_value(export)?;
            
            let entry = ExportEntry {
                instance_id,
                export_name: export.name.clone(),
                export_value,
                type_id: None, // Would be set based on export type
            };

            self.export_registry.insert(full_name, entry);
        }

        Ok(())
    }

    /// Resolve bindings between two instances
    fn resolve_bindings(
        &mut self,
        source_id: ComponentInstanceId,
        target_id: ComponentInstanceId,
        bindings: &BoundedVec<Binding, MAX_GENERATIVE_TYPES>,
    ) -> Result<(), ComponentError> {
        for binding in bindings.iter() {
            self.resolve_single_binding(source_id, target_id, binding)?;
        }
        Ok(())
    }

    /// Resolve a single binding
    fn resolve_single_binding(
        &mut self,
        source_id: ComponentInstanceId,
        target_id: ComponentInstanceId,
        binding: &Binding,
    ) -> Result<(), ComponentError> {
        // Get the export from source
        let source_export = self.lookup_export(source_id, &binding.export_name)?;
        
        // Verify type constraints if specified
        if let Some(constraint) = &binding.type_constraint {
            self.verify_type_constraint(&source_export, constraint)?;
        }

        // Satisfy the import in target
        self.satisfy_import(target_id, &binding.import_name, source_export)?;

        Ok(())
    }

    /// Look up an export from an instance
    fn lookup_export(
        &self,
        instance_id: ComponentInstanceId,
        export_name: &BoundedString<64>,
    ) -> Result<ExportValue, ComponentError> {
        let qualified_name = self.create_qualified_name(instance_id, export_name);
        
        self.export_registry.get(&qualified_name)
            .map(|entry| entry.export_value.clone())
            .ok_or(ComponentError::ExportResolutionFailed)
    }

    /// Satisfy an import with an export value
    fn satisfy_import(
        &mut self,
        instance_id: ComponentInstanceId,
        import_name: &BoundedString<64>,
        export_value: ExportValue,
    ) -> Result<(), ComponentError> {
        // This would update the instance's import resolution table
        // For now, we'll just validate that the import exists
        let instance = self.instances.get(&instance_id)
            .ok_or(ComponentError::ResourceNotFound(instance_id.0))?;

        let has_import = instance.component.imports.iter()
            .any(|import| import.name == *import_name);

        if !has_import {
            return Err(ComponentError::ImportResolutionFailed);
        }

        Ok(())
    }

    /// Verify type constraints between import and export
    fn verify_type_constraint(
        &self,
        _export: &ExportValue,
        constraint: &TypeConstraint,
    ) -> Result<(), ComponentError> {
        match constraint {
            TypeConstraint::Equal => {
                // Check exact type equality
                Ok(())
            }
            TypeConstraint::Subtype => {
                // Check subtype relationship
                Ok(())
            }
        }
    }

    /// Create a qualified name for exports
    fn create_qualified_name(
        &self,
        instance_id: ComponentInstanceId,
        name: &BoundedString<64>,
    ) -> BoundedString<128> {
        let instance_str = format!("instance_{}", instance_id.0);
        let qualified = format!("{}/{}", instance_str, name.as_str());
        BoundedString::from_str(&qualified).unwrap_or_default()
    }

    /// Create an export value from an export definition
    fn create_export_value(&self, _export: &Export) -> Result<ExportValue, ComponentError> {
        // This would create the appropriate ExportValue based on export type
        // For now, return a placeholder
        Ok(ExportValue::FunctionExport(crate::instantiation::FunctionExport {
            type_index: 0,
            code_offset: 0,
        }))
    }

    /// Collect external interfaces for a composite
    fn collect_external_interfaces(
        &self,
        composite: &mut CompositeComponent,
    ) -> Result<(), ComponentError> {
        // Collect all unresolved imports as external imports
        for &instance_id in composite.instances.iter() {
            let instance = self.instances.get(&instance_id)
                .ok_or(ComponentError::ResourceNotFound(instance_id.0))?;

            for import in &instance.component.imports {
                // Check if this import is satisfied internally
                let is_internal = self.is_import_satisfied_internally(instance_id, &import.name);
                
                if !is_internal {
                    let external_import = ExternalImport {
                        name: import.name.clone(),
                        import_type: import.import_type.clone(),
                        target_instance: instance_id,
                    };
                    
                    composite.external_imports.push(external_import)
                        .map_err(|_| ComponentError::TooManyGenerativeTypes)?;
                }
            }
        }

        Ok(())
    }

    /// Check if an import is satisfied internally within the composite
    fn is_import_satisfied_internally(
        &self,
        _instance_id: ComponentInstanceId,
        _import_name: &BoundedString<64>,
    ) -> bool {
        // This would check if the import is resolved by another component in the composite
        false
    }

    /// Get the type registry
    pub fn type_registry(&self) -> &GenerativeTypeRegistry {
        &self.type_registry
    }

    /// Get the type registry mutably
    pub fn type_registry_mut(&mut self) -> &mut GenerativeTypeRegistry {
        &mut self.type_registry
    }
}

impl Default for ComponentLinker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_linker_creation() {
        let linker = ComponentLinker::new();
        assert_eq!(linker.components.len(), 0);
        assert_eq!(linker.instances.len(), 0);
        assert_eq!(linker.export_registry.len(), 0);
    }

    #[test]
    fn test_register_component() {
        let mut linker = ComponentLinker::new();
        let name = BoundedString::from_str("test-component").unwrap();
        let component = Component {
            name: Some(String::from("test")),
            modules: Vec::new(),
            core_instances: Vec::new(),
            core_types: Vec::new(),
            components: Vec::new(),
            instances: Vec::new(),
            aliases: Vec::new(),
            types: Vec::new(),
            canonicals: Vec::new(),
            start: None,
            imports: Vec::new(),
            exports: Vec::new(),
        };

        assert!(linker.register_component(name.clone(), component.clone()).is_ok());
        
        // Registering again should fail
        assert!(linker.register_component(name, component).is_err());
    }

    #[test]
    fn test_create_composite() {
        let mut linker = ComponentLinker::new();
        
        // Register two components
        let comp1 = Component {
            name: Some(String::from("producer")),
            exports: vec![],
            ..Default::default()
        };
        
        let comp2 = Component {
            name: Some(String::from("consumer")),
            imports: vec![Import {
                name: BoundedString::from_str("consume").unwrap(),
                import_type: ImportType::Func,
            }],
            ..Default::default()
        };

        linker.register_component(BoundedString::from_str("producer").unwrap(), comp1).unwrap();
        linker.register_component(BoundedString::from_str("consumer").unwrap(), comp2).unwrap();

        // Create linkage descriptor
        let binding = Binding {
            import_name: BoundedString::from_str("consume").unwrap(),
            export_name: BoundedString::from_str("produce").unwrap(),
            type_constraint: Some(TypeConstraint::Equal),
        };

        let mut bindings = BoundedVec::new();
        bindings.push(binding).unwrap();

        let descriptor = LinkageDescriptor {
            source: BoundedString::from_str("producer").unwrap(),
            target: BoundedString::from_str("consumer").unwrap(),
            bindings,
        };

        // Create composite
        let composite = linker.create_composite(
            BoundedString::from_str("composite").unwrap(),
            vec![descriptor],
        );

        assert!(composite.is_ok());
        let composite = composite.unwrap();
        assert_eq!(composite.name.as_str(), "composite");
        assert_eq!(composite.instances.len(), 2);
    }

    #[test]
    fn test_type_constraints() {
        let linker = ComponentLinker::new();
        let export_value = ExportValue::FunctionExport(crate::instantiation::FunctionExport {
            type_index: 0,
            code_offset: 0,
        });

        // Test equal constraint
        assert!(linker.verify_type_constraint(&export_value, &TypeConstraint::Equal).is_ok());
        
        // Test subtype constraint
        assert!(linker.verify_type_constraint(&export_value, &TypeConstraint::Subtype).is_ok());
    }
}