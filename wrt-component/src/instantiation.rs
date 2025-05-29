//! Component instantiation and linking
//!
//! This module implements the instantiation process for WebAssembly components,
//! including import resolution, export extraction, and instance isolation.

#[cfg(not(feature = "std"))]
use core::{fmt, mem};
#[cfg(feature = "std")]
use std::{fmt, mem};

#[cfg(any(feature = "std", feature = "alloc"))]
use alloc::{boxed::Box, collections::BTreeMap, string::String, vec::Vec};

use wrt_foundation::{
    bounded::BoundedVec,
    component::ComponentType,
    component_value::ComponentValue,
    prelude::*,
};

use crate::{
    canonical::CanonicalAbi,
    component::{Component, ComponentInstance},
    execution_engine::ComponentExecutionEngine,
    export::Export,
    import::Import,
    resource_lifecycle::ResourceLifecycleManager,
    types::{Value, ValType},
    WrtResult,
};

/// Maximum number of imports in no_std environments
const MAX_IMPORTS: usize = 256;
/// Maximum number of exports in no_std environments  
const MAX_EXPORTS: usize = 256;
/// Maximum number of instances in no_std environments
const MAX_INSTANCES: usize = 64;

/// Import value provided during instantiation
#[derive(Debug, Clone)]
pub enum ImportValue {
    /// A function import
    Function(FunctionImport),
    /// A value import (global, memory, table)
    Value(ComponentValue),
    /// An instance import
    Instance(InstanceImport),
    /// A type import
    Type(ComponentType),
}

/// Function import descriptor
#[derive(Debug, Clone)]
pub struct FunctionImport {
    /// Function signature
    pub signature: ComponentType,
    /// Function implementation
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub implementation: Box<dyn Fn(&[Value]) -> WrtResult<Value>>,
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub implementation: fn(&[Value]) -> WrtResult<Value>,
}

/// Instance import descriptor
#[derive(Debug, Clone)]
pub struct InstanceImport {
    /// Instance exports
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub exports: BTreeMap<String, ExportValue>,
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub exports: BoundedVec<(BoundedString<64>, ExportValue), MAX_EXPORTS>,
}

/// Export value from an instance
#[derive(Debug, Clone)]
pub enum ExportValue {
    /// A function export
    Function(FunctionExport),
    /// A value export
    Value(ComponentValue),
    /// An instance export
    Instance(Box<ComponentInstance>),
    /// A type export
    Type(ComponentType),
}

/// Function export descriptor
#[derive(Debug, Clone)]
pub struct FunctionExport {
    /// Function signature
    pub signature: ComponentType,
    /// Function index in the instance
    pub index: u32,
}

/// Import values provided during instantiation
pub struct ImportValues {
    /// Map of import names to values
    #[cfg(any(feature = "std", feature = "alloc"))]
    imports: BTreeMap<String, ImportValue>,
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    imports: BoundedVec<(BoundedString<64>, ImportValue), MAX_IMPORTS>,
}

impl ImportValues {
    /// Create new import values
    pub fn new() -> Self {
        Self {
            #[cfg(any(feature = "std", feature = "alloc"))]
            imports: BTreeMap::new(),
            #[cfg(not(any(feature = "std", feature = "alloc")))]
            imports: BoundedVec::new(),
        }
    }

    /// Add an import value
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn add(&mut self, name: String, value: ImportValue) -> WrtResult<()> {
        self.imports.insert(name, value);
        Ok(())
    }

    /// Add an import value (no_std version)
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub fn add(&mut self, name: BoundedString<64>, value: ImportValue) -> WrtResult<()> {
        self.imports.push((name, value)).map_err(|_| {
            wrt_foundation::WrtError::ResourceExhausted("Too many imports".into())
        })
    }

    /// Get an import value by name
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub fn get(&self, name: &str) -> Option<&ImportValue> {
        self.imports.get(name)
    }

    /// Get an import value by name (no_std version)
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub fn get(&self, name: &str) -> Option<&ImportValue> {
        self.imports
            .iter()
            .find(|(n, _)| n.as_str() == name)
            .map(|(_, v)| v)
    }
}

impl Default for ImportValues {
    fn default() -> Self {
        Self::new()
    }
}

/// Component instantiation context
pub struct InstantiationContext {
    /// Canonical ABI processor
    pub canonical_abi: CanonicalAbi,
    /// Resource lifecycle manager
    pub resource_manager: ResourceLifecycleManager,
    /// Execution engine
    pub execution_engine: ComponentExecutionEngine,
    /// Instance counter for unique IDs
    next_instance_id: u32,
}

impl InstantiationContext {
    /// Create a new instantiation context
    pub fn new() -> Self {
        Self {
            canonical_abi: CanonicalAbi::new(),
            resource_manager: ResourceLifecycleManager::new(),
            execution_engine: ComponentExecutionEngine::new(),
            next_instance_id: 0,
        }
    }

    /// Get the next instance ID
    fn next_instance_id(&mut self) -> u32 {
        let id = self.next_instance_id;
        self.next_instance_id += 1;
        id
    }
}

impl Default for InstantiationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Component instantiation implementation
impl Component {
    /// Instantiate a component with the provided imports
    pub fn instantiate(
        &self,
        imports: &ImportValues,
        context: &mut InstantiationContext,
    ) -> WrtResult<ComponentInstance> {
        // Generate unique instance ID
        let instance_id = context.next_instance_id();

        // Step 1: Validate imports match component requirements
        self.validate_imports(imports)?;

        // Step 2: Create resource tables for this instance
        let resource_tables = self.create_resource_tables()?;

        // Step 3: Resolve imports into concrete values
        let resolved_imports = self.resolve_imports(imports, context)?;

        // Step 4: Initialize embedded modules (if any)
        let module_instances = self.initialize_modules(&resolved_imports, context)?;

        // Step 5: Extract and validate exports
        let exports = self.extract_exports(&module_instances, context)?;

        // Step 6: Create the component instance
        let instance = ComponentInstance {
            id: instance_id,
            component: self.clone(),
            #[cfg(any(feature = "std", feature = "alloc"))]
            imports: resolved_imports,
            #[cfg(any(feature = "std", feature = "alloc"))]
            exports,
            #[cfg(any(feature = "std", feature = "alloc"))]
            resource_tables,
            #[cfg(any(feature = "std", feature = "alloc"))]
            module_instances,
            #[cfg(not(any(feature = "std", feature = "alloc")))]
            imports: BoundedVec::new(),
            #[cfg(not(any(feature = "std", feature = "alloc")))]
            exports: BoundedVec::new(),
            #[cfg(not(any(feature = "std", feature = "alloc")))]
            resource_tables: BoundedVec::new(),
            #[cfg(not(any(feature = "std", feature = "alloc")))]
            module_instances: BoundedVec::new(),
        };

        Ok(instance)
    }

    /// Validate that provided imports match component requirements
    fn validate_imports(&self, imports: &ImportValues) -> WrtResult<()> {
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            for import in &self.imports {
                match imports.get(&import.name) {
                    Some(value) => {
                        // Validate import type matches
                        self.validate_import_type(import, value)?;
                    }
                    None => {
                        return Err(wrt_foundation::WrtError::InvalidInput(
                            format!("Missing required import: {}", import.name).into()
                        ));
                    }
                }
            }
        }
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            // In no_std, we have limited validation
            // Just check that we have some imports if required
            if self.imports.len() > 0 && imports.imports.len() == 0 {
                return Err(wrt_foundation::WrtError::InvalidInput(
                    "Missing required imports".into()
                ));
            }
        }

        Ok(())
    }

    /// Validate that an import value matches the expected type
    fn validate_import_type(&self, import: &Import, value: &ImportValue) -> WrtResult<()> {
        match (&import.import_type, value) {
            (crate::import::ImportType::Function(expected), ImportValue::Function(actual)) => {
                // Check function signature compatibility
                if !self.is_function_compatible(expected, &actual.signature) {
                    return Err(wrt_foundation::WrtError::TypeError(
                        "Function import type mismatch".into()
                    ));
                }
            }
            (crate::import::ImportType::Value(expected), ImportValue::Value(actual)) => {
                // Check value type compatibility
                if !self.is_value_compatible(expected, actual) {
                    return Err(wrt_foundation::WrtError::TypeError(
                        "Value import type mismatch".into()
                    ));
                }
            }
            (crate::import::ImportType::Instance(_), ImportValue::Instance(_)) => {
                // Instance validation is more complex, simplified for now
                // TODO: Validate instance exports match expected interface
            }
            (crate::import::ImportType::Type(_), ImportValue::Type(_)) => {
                // Type validation
                // TODO: Implement type equality checking
            }
            _ => {
                return Err(wrt_foundation::WrtError::TypeError(
                    "Import kind mismatch".into()
                ));
            }
        }
        Ok(())
    }

    /// Check if function types are compatible
    fn is_function_compatible(&self, expected: &ComponentType, actual: &ComponentType) -> bool {
        // Check basic type equality for now
        // In a full implementation, this would check subtyping rules
        match (expected, actual) {
            (ComponentType::Unit, ComponentType::Unit) => true,
            // For other types, check structural equality
            _ => expected == actual,
        }
    }

    /// Check if value types are compatible
    fn is_value_compatible(&self, expected: &ComponentType, actual: &ComponentValue) -> bool {
        // Basic type compatibility check
        match (expected, actual) {
            (ComponentType::Unit, ComponentValue::Unit) => true,
            // For other types, this would need more complex checking
            _ => true, // Allow for now
        }
    }

    /// Create resource tables for the instance
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn create_resource_tables(&self) -> WrtResult<Vec<ResourceTable>> {
        let mut tables = Vec::new();
        
        // Create resource tables based on component types
        // For each resource type in the component, create a table
        for (type_id, _) in self.types.iter().enumerate() {
            // Create a table for this resource type
            let table = ResourceTable {
                type_id: type_id as u32,
            };
            tables.push(table);
        }
        
        Ok(tables)
    }

    #[cfg(not(any(feature = "std", feature = "alloc")))]
    fn create_resource_tables(&self) -> WrtResult<BoundedVec<ResourceTable, 16>> {
        let mut tables = BoundedVec::new();
        
        // Create resource tables based on component types
        for (type_id, _) in self.types.iter().enumerate() {
            let table = ResourceTable {
                type_id: type_id as u32,
            };
            tables.push(table).map_err(|_| {
                wrt_foundation::WrtError::ResourceExhausted("Too many resource tables".into())
            })?;
        }
        
        Ok(tables)
    }

    /// Resolve imports into concrete values
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn resolve_imports(
        &self,
        imports: &ImportValues,
        context: &mut InstantiationContext,
    ) -> WrtResult<Vec<ResolvedImport>> {
        let mut resolved = Vec::new();

        for import in &self.imports {
            if let Some(value) = imports.get(&import.name) {
                let resolved_import = self.resolve_import(import, value, context)?;
                resolved.push(resolved_import);
            }
        }

        Ok(resolved)
    }

    #[cfg(not(any(feature = "std", feature = "alloc")))]
    fn resolve_imports(
        &self,
        imports: &ImportValues,
        context: &mut InstantiationContext,
    ) -> WrtResult<BoundedVec<ResolvedImport, MAX_IMPORTS>> {
        let mut resolved = BoundedVec::new();

        for import in &self.imports {
            // Find matching import by name
            for (name, value) in imports.imports.iter() {
                if name.as_str() == import.name.as_str() {
                    let resolved_import = self.resolve_import(import, value, context)?;
                    resolved.push(resolved_import).map_err(|_| {
                        wrt_foundation::WrtError::ResourceExhausted("Too many resolved imports".into())
                    })?;
                    break;
                }
            }
        }

        Ok(resolved)
    }

    /// Resolve a single import
    fn resolve_import(
        &self,
        import: &Import,
        value: &ImportValue,
        context: &mut InstantiationContext,
    ) -> WrtResult<ResolvedImport> {
        match value {
            ImportValue::Function(func) => {
                // Register the function with the execution engine
                #[cfg(any(feature = "std", feature = "alloc"))]
                let func_index = {
                    // Create a host function wrapper
                    let implementation = func.implementation.clone();
                    context.execution_engine.register_host_function(
                        Box::new(HostFunctionWrapper {
                            signature: func.signature.clone(),
                            implementation,
                        })
                    )?
                };
                #[cfg(not(any(feature = "std", feature = "alloc")))]
                let func_index = context.execution_engine.register_host_function(func.implementation)?;

                Ok(ResolvedImport::Function(func_index))
            }
            ImportValue::Value(val) => {
                Ok(ResolvedImport::Value(val.clone()))
            }
            ImportValue::Instance(inst) => {
                Ok(ResolvedImport::Instance(inst.clone()))
            }
            ImportValue::Type(ty) => {
                Ok(ResolvedImport::Type(ty.clone()))
            }
        }
    }

    /// Initialize embedded modules
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn initialize_modules(
        &self,
        resolved_imports: &[ResolvedImport],
        context: &mut InstantiationContext,
    ) -> WrtResult<Vec<ModuleInstance>> {
        let mut instances = Vec::new();
        
        // Initialize each embedded module
        for (module_index, _module) in self.modules.iter().enumerate() {
            // Create module instance
            let instance = ModuleInstance {
                module_index: module_index as u32,
            };
            instances.push(instance);
        }
        
        Ok(instances)
    }

    #[cfg(not(any(feature = "std", feature = "alloc")))]
    fn initialize_modules(
        &self,
        resolved_imports: &BoundedVec<ResolvedImport, MAX_IMPORTS>,
        context: &mut InstantiationContext,
    ) -> WrtResult<BoundedVec<ModuleInstance, MAX_INSTANCES>> {
        let mut instances = BoundedVec::new();
        
        // Initialize each embedded module
        for (module_index, _module) in self.modules.iter().enumerate() {
            let instance = ModuleInstance {
                module_index: module_index as u32,
            };
            instances.push(instance).map_err(|_| {
                wrt_foundation::WrtError::ResourceExhausted("Too many module instances".into())
            })?;
        }
        
        Ok(instances)
    }

    /// Extract exports from the instance
    #[cfg(any(feature = "std", feature = "alloc"))]
    fn extract_exports(
        &self,
        module_instances: &[ModuleInstance],
        context: &mut InstantiationContext,
    ) -> WrtResult<Vec<ResolvedExport>> {
        let mut exports = Vec::new();

        for export in &self.exports {
            // Resolve export to actual value based on export kind
            let resolved = match &export.kind {
                crate::export::ExportKind::Func(func_idx) => {
                    // Create function export
                    let func_export = FunctionExport {
                        signature: ComponentType::Unit, // TODO: Get actual signature
                        index: *func_idx,
                    };
                    ResolvedExport {
                        name: export.name.clone(),
                        value: ExportValue::Function(func_export),
                    }
                }
                crate::export::ExportKind::Value(val_idx) => {
                    // Create value export
                    ResolvedExport {
                        name: export.name.clone(),
                        value: ExportValue::Value(ComponentValue::Unit),
                    }
                }
                crate::export::ExportKind::Type(type_idx) => {
                    // Create type export
                    ResolvedExport {
                        name: export.name.clone(),
                        value: ExportValue::Type(ComponentType::Unit),
                    }
                }
                crate::export::ExportKind::Instance(inst_idx) => {
                    // Create instance export - simplified
                    ResolvedExport {
                        name: export.name.clone(),
                        value: ExportValue::Value(ComponentValue::Unit),
                    }
                }
            };
            exports.push(resolved);
        }

        Ok(exports)
    }

    #[cfg(not(any(feature = "std", feature = "alloc")))]
    fn extract_exports(
        &self,
        module_instances: &BoundedVec<ModuleInstance, MAX_INSTANCES>,
        context: &mut InstantiationContext,
    ) -> WrtResult<BoundedVec<ResolvedExport, MAX_EXPORTS>> {
        let mut exports = BoundedVec::new();

        for export in &self.exports {
            let resolved = match &export.kind {
                crate::export::ExportKind::Func(func_idx) => {
                    let func_export = FunctionExport {
                        signature: ComponentType::Unit,
                        index: *func_idx,
                    };
                    ResolvedExport {
                        name: export.name.clone(),
                        value: ExportValue::Function(func_export),
                    }
                }
                crate::export::ExportKind::Value(_) => {
                    ResolvedExport {
                        name: export.name.clone(),
                        value: ExportValue::Value(ComponentValue::Unit),
                    }
                }
                crate::export::ExportKind::Type(_) => {
                    ResolvedExport {
                        name: export.name.clone(),
                        value: ExportValue::Type(ComponentType::Unit),
                    }
                }
                crate::export::ExportKind::Instance(_) => {
                    ResolvedExport {
                        name: export.name.clone(),
                        value: ExportValue::Value(ComponentValue::Unit),
                    }
                }
            };
            exports.push(resolved).map_err(|_| {
                wrt_foundation::WrtError::ResourceExhausted("Too many exports".into())
            })?;
        }

        Ok(exports)
    }
}

/// Resolved import after validation and registration
#[derive(Debug, Clone)]
pub enum ResolvedImport {
    Function(u32), // Function index in execution engine
    Value(ComponentValue),
    Instance(InstanceImport),
    Type(ComponentType),
}

/// Resolved export with actual values
#[derive(Debug, Clone)]
pub struct ResolvedExport {
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub name: String,
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    pub name: BoundedString<64>,
    pub value: ExportValue,
}

/// Resource table for a specific resource type
#[derive(Debug, Clone)]
pub struct ResourceTable {
    pub type_id: u32,
    // Simplified for now, would contain actual resource entries
}

/// Module instance within a component
#[derive(Debug, Clone)]
pub struct ModuleInstance {
    pub module_index: u32,
    // Simplified for now, would contain actual runtime instance
}

/// Host function wrapper for the execution engine
#[cfg(any(feature = "std", feature = "alloc"))]
struct HostFunctionWrapper {
    signature: ComponentType,
    implementation: Box<dyn Fn(&[Value]) -> WrtResult<Value>>,
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl crate::execution_engine::HostFunction for HostFunctionWrapper {
    fn call(&mut self, args: &[Value]) -> WrtResult<Value> {
        (self.implementation)(args)
    }

    fn signature(&self) -> &ComponentType {
        &self.signature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_values() {
        let mut imports = ImportValues::new();
        
        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            let func = FunctionImport {
                signature: ComponentType::Unit,
                implementation: Box::new(|_args| Ok(Value::U32(42))),
            };
            imports.add("test_func".to_string(), ImportValue::Function(func)).unwrap();
            assert!(imports.get("test_func").is_some());
            assert!(imports.get("unknown").is_none());
        }
        
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            let func = FunctionImport {
                signature: ComponentType::Unit,
                implementation: |_args| Ok(Value::U32(42)),
            };
            let name = BoundedString::from_str("test_func").unwrap();
            imports.add(name, ImportValue::Function(func)).unwrap();
            assert!(imports.get("test_func").is_some());
            assert!(imports.get("unknown").is_none());
        }
    }

    #[test]
    fn test_instantiation_context() {
        let mut context = InstantiationContext::new();
        assert_eq!(context.next_instance_id(), 0);
        assert_eq!(context.next_instance_id(), 1);
        assert_eq!(context.next_instance_id(), 2);
    }
}