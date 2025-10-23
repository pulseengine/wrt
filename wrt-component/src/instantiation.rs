//! Component instantiation and linking
//!
//! This module implements the instantiation process for WebAssembly components,
//! including import resolution, export extraction, and instance isolation.

#[cfg(not(feature = "std"))]
use core::{
    fmt,
    mem,
};
#[cfg(all(feature = "std", not(feature = "safety-critical")))]
use std::{
    boxed::Box,
    collections::BTreeMap,
    string::String,
    vec::Vec,
};
#[cfg(feature = "std")]
use std::{
    fmt,
    mem,
};

#[cfg(all(feature = "std", feature = "safety-critical"))]
use wrt_foundation::allocator::{
    CrateId,
    WrtHashMap,
    WrtVec,
};
use wrt_foundation::{
    bounded::{
        BoundedString,
    },
    budget_aware_provider::CrateId,
    collections::StaticVec,
    collections::StaticVec as BoundedVec,
    prelude::*,
    safe_managed_alloc,
    traits::{Checksummable, ToBytes, FromBytes},
};

use crate::{
    bounded_component_infra::{ComponentProvider, InstantiationProvider},
    canonical_abi::canonical::CanonicalABI,
    components::component::{
        Component,
    },
    execution_engine::ComponentExecutionEngine,
    export::Export,
    import::Import,
    prelude::{
        ExportKind,
        WrtComponentType,
        WrtComponentValue,
    },
    resources::resource_lifecycle::ResourceLifecycleManager,
    resources::ResourceTable as ComponentResourceTable,
    types::{
        ComponentInstance,
        ValType,
        Value,
    },
    WrtResult,
};

/// Maximum number of imports in no_std environments
const MAX_IMPORTS: usize = 256;
/// Maximum number of exports in no_std environments  
const MAX_EXPORTS: usize = 256;
/// Maximum number of instances in no_std environments
const MAX_INSTANCES: usize = 64;

// Type alias for instantiation provider - REMOVED
// Provider instances are now created using safe_managed_alloc! macro instead of type aliases

/// Import value provided during instantiation
#[derive(Debug, Clone)]
pub enum ImportValue {
    /// A function import
    Function(FunctionImport),
    /// A value import (global, memory, table)
    Value(WrtComponentValue<ComponentProvider>),
    /// An instance import
    Instance(InstanceImport),
    /// A type import
    Type(WrtComponentType<ComponentProvider>),
}

impl Default for ImportValue {
    fn default() -> Self {
        Self::Value(WrtComponentValue::Unit)
    }
}

impl PartialEq for ImportValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Function(_), Self::Function(_)) => true,
            (Self::Value(a), Self::Value(b)) => a == b,
            (Self::Instance(_), Self::Instance(_)) => true,
            (Self::Type(a), Self::Type(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for ImportValue {}

impl Checksummable for ImportValue {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        match self {
            Self::Function(_) => 0u8.update_checksum(checksum),
            Self::Value(v) => { 1u8.update_checksum(checksum); v.update_checksum(checksum); },
            Self::Instance(_) => 2u8.update_checksum(checksum),
            Self::Type(t) => { 3u8.update_checksum(checksum); t.update_checksum(checksum); },
        }
    }
}

impl ToBytes for ImportValue {
    fn to_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut wrt_foundation::traits::WriteStream<'a>,
        provider: &P,
    ) -> wrt_error::Result<()> {
        match self {
            Self::Function(_) => 0u8.to_bytes_with_provider(writer, provider),
            Self::Value(v) => { 1u8.to_bytes_with_provider(writer, provider)?; v.to_bytes_with_provider(writer, provider) },
            Self::Instance(_) => 2u8.to_bytes_with_provider(writer, provider),
            Self::Type(t) => { 3u8.to_bytes_with_provider(writer, provider)?; t.to_bytes_with_provider(writer, provider) },
        }
    }
}

impl FromBytes for ImportValue {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        provider: &P,
    ) -> wrt_error::Result<Self> {
        Ok(Self::default())
    }
}

/// Function import descriptor
#[derive(Debug, Clone)]
pub struct FunctionImport {
    /// Function signature
    pub signature:      WrtComponentType<ComponentProvider>,
    /// Function implementation
    #[cfg(feature = "std")]
    pub implementation: Box<dyn Fn(&[Value]) -> WrtResult<Value>>,
    #[cfg(not(any(feature = "std",)))]
    pub implementation: fn(&[Value]) -> WrtResult<Value>,
}

/// Instance import descriptor
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InstanceImport {
    /// Instance exports
    #[cfg(all(feature = "std", feature = "safety-critical"))]
    pub exports: WrtHashMap<String, Box<ExportValue>, { CrateId::Component as u8 }, 256>,
    #[cfg(all(feature = "std", not(feature = "safety-critical")))]
    pub exports: BTreeMap<String, Box<ExportValue>>,
    #[cfg(not(feature = "std"))]
    pub exports: BoundedVec<
        (BoundedString<64>, Box<ExportValue>),
        MAX_EXPORTS
    >,
}

impl Checksummable for InstanceImport {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        #[cfg(feature = "std")]
        {
            self.exports.len().update_checksum(checksum);
            for (k, v) in &self.exports {
                k.as_bytes().update_checksum(checksum);
                v.update_checksum(checksum);
            }
        }
        #[cfg(not(feature = "std"))]
        {
            self.exports.len().update_checksum(checksum);
            for (k, v) in &self.exports {
                if let Ok(bytes) = k.as_bytes() {
                    bytes.as_ref().update_checksum(checksum);
                }
                v.update_checksum(checksum);
            }
        }
    }
}

impl ToBytes for InstanceImport {
    fn to_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut wrt_foundation::traits::WriteStream<'a>,
        provider: &P,
    ) -> wrt_error::Result<()> {
        #[cfg(feature = "std")]
        {
            (self.exports.len() as u32).to_bytes_with_provider(writer, provider)?;
            for (k, v) in &self.exports {
                (k.len() as u32).to_bytes_with_provider(writer, provider)?;
                k.as_bytes().to_bytes_with_provider(writer, provider)?;
                v.to_bytes_with_provider(writer, provider)?;
            }
        }
        #[cfg(not(feature = "std"))]
        {
            (self.exports.len() as u32).to_bytes_with_provider(writer, provider)?;
            for (k, v) in &self.exports {
                (k.len() as u32).to_bytes_with_provider(writer, provider)?;
                k.as_bytes()?.as_ref().to_bytes_with_provider(writer, provider)?;
                v.to_bytes_with_provider(writer, provider)?;
            }
        }
        Ok(())
    }
}

impl FromBytes for InstanceImport {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        provider: &P,
    ) -> wrt_error::Result<Self> {
        Ok(Self::default())
    }
}

/// Export value from an instance
#[derive(Debug, Clone)]
pub enum ExportValue {
    /// A function export
    Function(FunctionExport),
    /// A value export
    Value(WrtComponentValue<ComponentProvider>),
    /// An instance export
    Instance(InstanceImport),
    /// A type export
    Type(WrtComponentType<ComponentProvider>),
}

impl Default for ExportValue {
    fn default() -> Self {
        Self::Value(WrtComponentValue::Unit)
    }
}

impl PartialEq for ExportValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Function(a), Self::Function(b)) => a.index == b.index,
            (Self::Value(a), Self::Value(b)) => a == b,
            (Self::Instance(_), Self::Instance(_)) => true,
            (Self::Type(a), Self::Type(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for ExportValue {}

impl Checksummable for ExportValue {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        match self {
            Self::Function(f) => { 0u8.update_checksum(checksum); f.index.update_checksum(checksum); },
            Self::Value(v) => { 1u8.update_checksum(checksum); v.update_checksum(checksum); },
            Self::Instance(_) => 2u8.update_checksum(checksum),
            Self::Type(t) => { 3u8.update_checksum(checksum); t.update_checksum(checksum); },
        }
    }
}

impl ToBytes for ExportValue {
    fn to_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut wrt_foundation::traits::WriteStream<'a>,
        provider: &P,
    ) -> wrt_error::Result<()> {
        match self {
            Self::Function(f) => { 0u8.to_bytes_with_provider(writer, provider)?; f.index.to_bytes_with_provider(writer, provider) },
            Self::Value(v) => { 1u8.to_bytes_with_provider(writer, provider)?; v.to_bytes_with_provider(writer, provider) },
            Self::Instance(_) => 2u8.to_bytes_with_provider(writer, provider),
            Self::Type(t) => { 3u8.to_bytes_with_provider(writer, provider)?; t.to_bytes_with_provider(writer, provider) },
        }
    }
}

impl FromBytes for ExportValue {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        provider: &P,
    ) -> wrt_error::Result<Self> {
        Ok(Self::default())
    }
}

/// Function export descriptor
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct FunctionExport {
    /// Function signature
    pub signature: WrtComponentType<ComponentProvider>,
    /// Function index in the instance
    pub index:     u32,
}

/// Import values provided during instantiation
pub struct ImportValues {
    /// Map of import names to values
    #[cfg(all(feature = "std", feature = "safety-critical"))]
    imports: WrtHashMap<String, ImportValue, { CrateId::Component as u8 }, 256>,
    #[cfg(all(feature = "std", not(feature = "safety-critical")))]
    imports: BTreeMap<String, ImportValue>,
    #[cfg(not(feature = "std"))]
    imports: BoundedVec<
        (BoundedString<64>, ImportValue),
        MAX_IMPORTS
    >,
}

impl ImportValues {
    /// Create new import values
    pub fn new() -> WrtResult<Self> {
        Ok(Self {
            #[cfg(all(feature = "std", feature = "safety-critical"))]
            imports: WrtHashMap::new(),
            #[cfg(all(feature = "std", not(feature = "safety-critical")))]
            imports: BTreeMap::new(),
            #[cfg(not(feature = "std"))]
            imports: {
                let provider = safe_managed_alloc!(65536, CrateId::Component)?;
                BoundedVec::new().unwrap()
            },
        })
    }

    /// Add an import value
    #[cfg(all(feature = "std", feature = "safety-critical"))]
    pub fn add(&mut self, name: String, value: ImportValue) -> WrtResult<()> {
        self.imports
            .insert(name, value)
            .map_err(|_| wrt_error::Error::resource_exhausted("Too many imports (limit: 256)"))?;
        Ok(())
    }

    /// Add an import value
    #[cfg(all(feature = "std", not(feature = "safety-critical")))]
    pub fn add(&mut self, name: String, value: ImportValue) -> WrtResult<()> {
        self.imports.insert(name, value);
        Ok(())
    }

    /// Add an import value (no_std version)
    #[cfg(not(any(feature = "std",)))]
    pub fn add(
        &mut self,
        name: BoundedString<64>,
        value: ImportValue,
    ) -> WrtResult<()> {
        self.imports
            .push((name, value))
            .map_err(|_| wrt_error::Error::resource_exhausted("Too many imports"))
    }

    /// Get an import value by name
    #[cfg(all(feature = "std", feature = "safety-critical"))]
    pub fn get(&self, name: &str) -> Option<&ImportValue> {
        self.imports.get(name)
    }

    /// Get an import value by name
    #[cfg(all(feature = "std", not(feature = "safety-critical")))]
    pub fn get(&self, name: &str) -> Option<&ImportValue> {
        self.imports.get(name)
    }

    /// Get an import value by name (no_std version)
    #[cfg(not(any(feature = "std",)))]
    pub fn get(&self, name: &str) -> Option<&ImportValue> {
        self.imports.iter().find(|(n, _)| n.as_str().ok() == Some(name)).map(|(_, v)| v)
    }
}

impl Default for ImportValues {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback for default construction in case allocation fails
            Self {
                #[cfg(all(feature = "std", feature = "safety-critical"))]
                imports: WrtHashMap::new(),
                #[cfg(all(feature = "std", not(feature = "safety-critical")))]
                imports: BTreeMap::new(),
                #[cfg(not(feature = "std"))]
                imports: {
                    // Use fallback provider only in default construction
                    let provider = InstantiationProvider::default();
                    BoundedVec::new().unwrap()
                },
            }
        })
    }
}

/// Component instantiation context
pub struct InstantiationContext {
    /// Canonical ABI processor
    pub canonical_abi:    CanonicalABI,
    /// Resource lifecycle manager
    pub resource_manager: ResourceLifecycleManager,
    /// Execution engine
    pub execution_engine: ComponentExecutionEngine,
    /// Instance counter for unique IDs
    next_instance_id:     u32,
}

impl InstantiationContext {
    /// Create a new instantiation context
    pub fn new() -> WrtResult<Self> {
        Ok(Self {
            canonical_abi:    CanonicalABI::new(64),
            resource_manager: ResourceLifecycleManager::new(),
            execution_engine: ComponentExecutionEngine::new()?,
            next_instance_id: 0,
        })
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
        Self::new().unwrap_or_else(|_| {
            Self {
                canonical_abi:    CanonicalABI::new(64),
                resource_manager: ResourceLifecycleManager::new(),
                execution_engine: ComponentExecutionEngine::default(),
                next_instance_id: 0,
            }
        })
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

        // Step 6: Create the component instance with all required fields
        // Note: Component field should ideally be Arc<Component> to avoid cloning,
        // but for now we create a minimal component reference
        let component_ref = Component {
            id: self.id.clone(),
            component_type: self.component_type.clone(),
            types: self.types.clone(),
            modules: self.modules.clone(),
            exports: self.exports.clone(),
            imports: self.imports.clone(),
            instances: self.instances.clone(),
            linked_components: Default::default(),
            callback_registry: None,
            runtime: None,
            interceptor: None,
            resource_table: ComponentResourceTable::new().unwrap_or_else(|_| ComponentResourceTable::default()),
            built_in_requirements: None,
            original_binary: None,
            verification_level: wrt_foundation::verification::VerificationLevel::Standard,
        };

        let instance = ComponentInstance {
            id: instance_id,
            component: component_ref,
            state: crate::types::ComponentInstanceState::Initialized,
            resource_manager: None,
            memory: None,
            metadata: crate::types::ComponentMetadata::default(),
            functions: {
                #[cfg(all(feature = "std", feature = "safety-critical"))]
                {
                    use wrt_foundation::allocator::WrtVec;
                    WrtVec::new()
                }
                #[cfg(all(feature = "std", not(feature = "safety-critical")))]
                {
                    Vec::new()
                }
                #[cfg(not(feature = "std"))]
                {
                    BoundedVec::new()
                }
            },
            imports: resolved_imports,
            exports: exports,
            resource_tables: resource_tables,
            module_instances: module_instances,
        };

        Ok(instance)
    }

    /// Validate that provided imports match component requirements
    fn validate_imports(&self, imports: &ImportValues) -> WrtResult<()> {
        #[cfg(feature = "std")]
        {
            for import in &self.imports {
                match imports.get(&import.name) {
                    Some(value) => {
                        // Validate import type matches
                        self.validate_import_type(import, value)?;
                    },
                    None => {
                        return Err(wrt_error::Error::validation_invalid_input(
                            "Missing required import",
                        ));
                    },
                }
            }
        }
        #[cfg(not(any(feature = "std",)))]
        {
            // In no_std, we have limited validation
            // Just check that we have some imports if required
            if self.imports.len() > 0 && imports.imports.len() == 0 {
                return Err(wrt_error::Error::validation_invalid_input(
                    "Missing required imports",
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
                    return Err(wrt_error::Error::type_mismatch_error(
                        "Function import type mismatch",
                    ));
                }
            },
            (crate::import::ImportType::Value(expected), ImportValue::Value(actual)) => {
                // Check value type compatibility
                if !self.is_value_compatible(expected, actual) {
                    return Err(wrt_error::Error::type_mismatch_error(
                        "Value import type mismatch",
                    ));
                }
            },
            (crate::import::ImportType::Instance(_), ImportValue::Instance(_)) => {
                // Instance validation is more complex, simplified for now
                // TODO: Validate instance exports match expected interface
            },
            (crate::import::ImportType::Type(_), ImportValue::Type(_)) => {
                // Type validation
                // TODO: Implement type equality checking
            },
            _ => {
                return Err(wrt_error::Error::type_mismatch_error(
                    "Import kind mismatch",
                ));
            },
        }
        Ok(())
    }

    /// Check if function types are compatible
    fn is_function_compatible(
        &self,
        expected: &WrtComponentType<ComponentProvider>,
        actual: &WrtComponentType<ComponentProvider>,
    ) -> bool {
        // Check basic type equality for now
        // In a full implementation, this would check subtyping rules
        // Check if both are unit types (empty imports/exports/etc)
        if expected.imports.len() == 0 && expected.exports.len() == 0 &&
           actual.imports.len() == 0 && actual.exports.len() == 0 {
            return true;
        }
        // For other types, check structural equality
        expected == actual
    }

    /// Check if value types are compatible
    fn is_value_compatible(&self, expected: &WrtComponentType<ComponentProvider>, actual: &WrtComponentValue<ComponentProvider>) -> bool {
        // Basic type compatibility check
        // Check if expected is unit type and actual is Unit value
        if expected.imports.len() == 0 && expected.exports.len() == 0 &&
           matches!(actual, WrtComponentValue::Unit) {
            return true;
        }
        // For other types, this would need more complex checking
        true // Allow for now
    }

    /// Create resource tables for the instance with budget enforcement
    #[cfg(all(feature = "std", feature = "safety-critical"))]
    fn create_resource_tables(
        &self,
    ) -> WrtResult<WrtVec<ResourceTable, { CrateId::Component as u8 }, 16>> {
        let mut tables = WrtVec::new();

        // Create resource tables based on component types
        // For each resource type in the component, create a table
        for _type_id in 0..self.types.len().min(16) {
            // Create a budget-aware table for this resource type
            let table = ResourceTable::new()?;
            tables.push(table).map_err(|_| {
                wrt_error::Error::resource_exhausted("Too many resource tables (limit: 16)")
            })?;
        }

        Ok(tables)
    }

    /// Create resource tables for the instance
    #[cfg(all(feature = "std", not(feature = "safety-critical")))]
    fn create_resource_tables(&self) -> WrtResult<Vec<ResourceTable>> {
        let mut tables = Vec::new();

        // Create resource tables based on component types
        // For each resource type in the component, create a table
        for _type_id in 0..self.types.len() {
            // Create a budget-aware table for this resource type
            let table = ResourceTable::new()?;
            tables.push(table);
        }

        Ok(tables)
    }

    #[cfg(not(any(feature = "std",)))]
    fn create_resource_tables(
        &self,
    ) -> WrtResult<StaticVec<ResourceTable, 16>>
    {
        use wrt_foundation::collections::StaticVec;

        let mut tables = StaticVec::new();

        // Create resource tables based on component types
        for _type_id in 0..self.types.len().min(16) {
            let table = ResourceTable { type_id: _type_id as u32 };
            tables
                .push(table)
                .map_err(|_| wrt_error::Error::resource_exhausted("Too many resource tables"))?;
        }

        Ok(tables)
    }

    /// Resolve imports into concrete values
    #[cfg(all(feature = "std", feature = "safety-critical"))]
    fn resolve_imports(
        &self,
        imports: &ImportValues,
        context: &mut InstantiationContext,
    ) -> WrtResult<WrtVec<ResolvedImport, { CrateId::Component as u8 }, 256>> {
        let mut resolved = WrtVec::new();

        for import in &self.imports {
            if let Some(value) = imports.get(&import.name) {
                let resolved_import = self.resolve_import(import, value, context)?;
                resolved.push(resolved_import).map_err(|_| {
                    wrt_error::Error::resource_exhausted("Too many resolved imports (limit: 256)")
                })?;
            }
        }

        Ok(resolved)
    }

    /// Resolve imports into concrete values
    #[cfg(all(feature = "std", not(feature = "safety-critical")))]
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

    #[cfg(not(any(feature = "std",)))]
    fn resolve_imports(
        &self,
        imports: &ImportValues,
        context: &mut InstantiationContext,
    ) -> WrtResult<BoundedVec<ResolvedImport, 256>> {
        let mut resolved = BoundedVec::new();

        for import in &self.imports {
            // Find matching import by name
            for (name, value) in imports.imports.iter() {
                if let Ok(name_str) = name.as_str() {
                    // import.name is already a String, no need for as_str()
                    if name_str == import.name.as_str() {
                        let resolved_import = self.resolve_import(import, value, context)?;
                        resolved.push(resolved_import).map_err(|_| {
                            wrt_error::Error::resource_exhausted("Too many resolved imports")
                        })?;
                        break;
                    }
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
                #[cfg(feature = "std")]
                let func_index = {
                    // Create a host function wrapper
                    let implementation = func.implementation.clone();
                    context.execution_engine.register_host_function(Box::new(
                        HostFunctionWrapper {
                            signature: func.signature.clone(),
                            implementation,
                        },
                    ))?
                };
                #[cfg(not(any(feature = "std",)))]
                let func_index =
                    context.execution_engine.register_host_function(func.implementation)?;

                Ok(ResolvedImport::Function(func_index))
            },
            ImportValue::Value(val) => Ok(ResolvedImport::Value(val.clone())),
            ImportValue::Instance(inst) => Ok(ResolvedImport::Instance(inst.clone())),
            ImportValue::Type(ty) => Ok(ResolvedImport::Type(ty.clone())),
        }
    }

    /// Initialize embedded modules
    #[cfg(all(feature = "std", feature = "safety-critical"))]
    fn initialize_modules(
        &self,
        resolved_imports: &[ResolvedImport],
        context: &mut InstantiationContext,
    ) -> WrtResult<WrtVec<ModuleInstance, { CrateId::Component as u8 }, 64>> {
        let mut instances = WrtVec::new();

        // Initialize each embedded module
        for (module_index, _module) in self.modules.iter().enumerate() {
            // Create module instance
            let instance = ModuleInstance {
                module_index: module_index as u32,
            };
            instances.push(instance).map_err(|_| {
                wrt_error::Error::resource_exhausted("Too many module instances (limit: 64)")
            })?;
        }

        Ok(instances)
    }

    /// Initialize embedded modules
    #[cfg(all(feature = "std", not(feature = "safety-critical")))]
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

    #[cfg(not(any(feature = "std",)))]
    fn initialize_modules(
        &self,
        resolved_imports: &BoundedVec<ResolvedImport, 256>,
        context: &mut InstantiationContext,
    ) -> WrtResult<BoundedVec<ModuleInstance, 64>> {
        let mut instances = BoundedVec::new();

        // Initialize each embedded module
        for (module_index, _module) in self.modules.iter().enumerate() {
            let instance = ModuleInstance {
                module_index: module_index as u32,
            };
            instances
                .push(instance)
                .map_err(|_| wrt_error::Error::resource_exhausted("Too many module instances"))?;
        }

        Ok(instances)
    }

    /// Extract exports from the instance
    #[cfg(all(feature = "std", feature = "safety-critical"))]
    fn extract_exports(
        &self,
        module_instances: &[ModuleInstance],
        context: &mut InstantiationContext,
    ) -> WrtResult<WrtVec<ResolvedExport, { CrateId::Component as u8 }, 256>> {
        let mut exports = WrtVec::new();

        for export in &self.exports {
            // Resolve export to actual value based on export kind
            let resolved = match &export.kind {
                ExportKind::Function { function_index } => {
                    // Create function export
                    let func_export = FunctionExport {
                        signature: WrtComponentType::Unit, // TODO: Get actual signature
                        index:     *function_index,
                    };
                    let export_val = ExportValue::Function(func_export);
                    ResolvedExport {
                        name:        export.name.clone(),
                        value:       export_val.clone(),
                        export_type: export_val,
                    }
                },
                ExportKind::Value { value_index } => {
                    // Create value export
                    let export_val = ExportValue::Value(WrtComponentValue::Unit);
                    ResolvedExport {
                        name:        export.name.clone(),
                        value:       export_val.clone(),
                        export_type: export_val,
                    }
                },
                ExportKind::Type { type_index } => {
                    // Create type export
                    let export_val = ExportValue::Type(WrtComponentType::Unit);
                    ResolvedExport {
                        name:        export.name.clone(),
                        value:       export_val.clone(),
                        export_type: export_val,
                    }
                },
                ExportKind::Instance { instance_index } => {
                    // Create instance export - simplified
                    let export_val = ExportValue::Value(WrtComponentValue::Unit);
                    ResolvedExport {
                        name:        export.name.clone(),
                        value:       export_val.clone(),
                        export_type: export_val,
                    }
                },
            };
            exports.push(resolved).map_err(|_| {
                wrt_error::Error::resource_exhausted("Too many exports (limit: 256)")
            })?;
        }

        Ok(exports)
    }

    /// Extract exports from the instance
    #[cfg(all(feature = "std", not(feature = "safety-critical")))]
    fn extract_exports(
        &self,
        module_instances: &[ModuleInstance],
        context: &mut InstantiationContext,
    ) -> WrtResult<Vec<ResolvedExport>> {
        let mut exports = Vec::new();

        for export in &self.exports {
            // Resolve export to actual value based on export kind
            let resolved = match &export.kind {
                ExportKind::Function { function_index } => {
                    // Create function export
                    let func_export = FunctionExport {
                        signature: WrtComponentType::Unit, // TODO: Get actual signature
                        index:     *function_index,
                    };
                    let export_val = ExportValue::Function(func_export);
                    ResolvedExport {
                        name:        export.name.clone(),
                        value:       export_val.clone(),
                        export_type: export_val,
                    }
                },
                ExportKind::Value { value_index } => {
                    // Create value export
                    let export_val = ExportValue::Value(WrtComponentValue::Unit);
                    ResolvedExport {
                        name:        export.name.clone(),
                        value:       export_val.clone(),
                        export_type: export_val,
                    }
                },
                ExportKind::Type { type_index } => {
                    // Create type export
                    let export_val = ExportValue::Type(WrtComponentType::Unit);
                    ResolvedExport {
                        name:        export.name.clone(),
                        value:       export_val.clone(),
                        export_type: export_val,
                    }
                },
                ExportKind::Instance { instance_index } => {
                    // Create instance export - simplified
                    let export_val = ExportValue::Value(WrtComponentValue::Unit);
                    ResolvedExport {
                        name:        export.name.clone(),
                        value:       export_val.clone(),
                        export_type: export_val,
                    }
                },
            };
            exports.push(resolved);
        }

        Ok(exports)
    }

    #[cfg(not(any(feature = "std",)))]
    fn extract_exports(
        &self,
        module_instances: &BoundedVec<ModuleInstance, 64>,
        context: &mut InstantiationContext,
    ) -> WrtResult<BoundedVec<ResolvedExport, 256>> {
        let mut exports = BoundedVec::new();

        for export in &self.exports {
            // Create provider for ComponentType if needed
            let provider = safe_managed_alloc!(4096, CrateId::Component)?;
            let unit_type = WrtComponentType::unit(provider)?;

            let resolved = match &export.kind {
                ExportKind::Function { function_index } => {
                    let func_export = FunctionExport {
                        signature: unit_type.clone(),
                        index:     *function_index,
                    };
                    let export_val = ExportValue::Function(func_export);
                    ResolvedExport {
                        #[cfg(feature = "std")]
                        name:        export.name.clone(),
                        #[cfg(not(feature = "std"))]
                        name:        {
                            let name_provider = safe_managed_alloc!(65536, CrateId::Component)?;
                            BoundedString::from_str(&export.name)?
                        },
                        value:       export_val.clone(),
                        export_type: export_val,
                    }
                },
                ExportKind::Value { value_index } => {
                    let export_val = ExportValue::Value(WrtComponentValue::Unit);
                    ResolvedExport {
                        #[cfg(feature = "std")]
                        name:        export.name.clone(),
                        #[cfg(not(feature = "std"))]
                        name:        {
                            let name_provider = safe_managed_alloc!(65536, CrateId::Component)?;
                            BoundedString::from_str(&export.name)?
                        },
                        value:       export_val.clone(),
                        export_type: export_val,
                    }
                },
                ExportKind::Type { type_index } => {
                    let export_val = ExportValue::Type(unit_type.clone());
                    ResolvedExport {
                        #[cfg(feature = "std")]
                        name:        export.name.clone(),
                        #[cfg(not(feature = "std"))]
                        name:        {
                            let name_provider = safe_managed_alloc!(65536, CrateId::Component)?;
                            BoundedString::from_str(&export.name)?
                        },
                        value:       export_val.clone(),
                        export_type: export_val,
                    }
                },
                ExportKind::Instance { instance_index } => {
                    let export_val = ExportValue::Value(WrtComponentValue::Unit);
                    ResolvedExport {
                        #[cfg(feature = "std")]
                        name:        export.name.clone(),
                        #[cfg(not(feature = "std"))]
                        name:        {
                            let name_provider = safe_managed_alloc!(65536, CrateId::Component)?;
                            BoundedString::from_str(&export.name)?
                        },
                        value:       export_val.clone(),
                        export_type: export_val,
                    }
                },
            };
            exports
                .push(resolved)
                .map_err(|_| wrt_error::Error::resource_exhausted("Too many exports"))?;
        }

        Ok(exports)
    }
}

/// Resolved import after validation and registration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedImport {
    Function(u32), // Function index in execution engine
    Value(WrtComponentValue<ComponentProvider>),
    Instance(InstanceImport),
    Type(WrtComponentType<ComponentProvider>),
}

impl Default for ResolvedImport {
    fn default() -> Self {
        Self::Value(WrtComponentValue::Unit)
    }
}

impl Checksummable for ResolvedImport {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        match self {
            Self::Function(idx) => { 0u8.update_checksum(checksum); idx.update_checksum(checksum); },
            Self::Value(v) => { 1u8.update_checksum(checksum); v.update_checksum(checksum); },
            Self::Instance(i) => { 2u8.update_checksum(checksum); i.update_checksum(checksum); },
            Self::Type(t) => { 3u8.update_checksum(checksum); t.update_checksum(checksum); },
        }
    }
}

impl ToBytes for ResolvedImport {
    fn to_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut wrt_foundation::traits::WriteStream<'a>,
        provider: &P,
    ) -> wrt_error::Result<()> {
        match self {
            Self::Function(idx) => { 0u8.to_bytes_with_provider(writer, provider)?; idx.to_bytes_with_provider(writer, provider) },
            Self::Value(v) => { 1u8.to_bytes_with_provider(writer, provider)?; v.to_bytes_with_provider(writer, provider) },
            Self::Instance(i) => { 2u8.to_bytes_with_provider(writer, provider)?; i.to_bytes_with_provider(writer, provider) },
            Self::Type(t) => { 3u8.to_bytes_with_provider(writer, provider)?; t.to_bytes_with_provider(writer, provider) },
        }
    }
}

impl FromBytes for ResolvedImport {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        provider: &P,
    ) -> wrt_error::Result<Self> {
        Ok(Self::default())
    }
}

/// Resolved export with actual values
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolvedExport {
    #[cfg(feature = "std")]
    pub name:        String,
    #[cfg(not(any(feature = "std",)))]
    pub name:        BoundedString<64>,
    pub value:       ExportValue,
    pub export_type: ExportValue,
}

impl Checksummable for ResolvedExport {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        #[cfg(feature = "std")]
        {
            self.name.as_bytes().update_checksum(checksum);
        }
        #[cfg(not(feature = "std"))]
        {
            if let Ok(bytes) = self.name.as_bytes() {
                bytes.as_ref().update_checksum(checksum);
            }
        }
        self.value.update_checksum(checksum);
        self.export_type.update_checksum(checksum);
    }
}

impl ToBytes for ResolvedExport {
    fn to_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut wrt_foundation::traits::WriteStream<'a>,
        provider: &P,
    ) -> wrt_error::Result<()> {
        #[cfg(feature = "std")]
        {
            (self.name.len() as u32).to_bytes_with_provider(writer, provider)?;
            self.name.as_bytes().to_bytes_with_provider(writer, provider)?;
        }
        #[cfg(not(feature = "std"))]
        {
            (self.name.len() as u32).to_bytes_with_provider(writer, provider)?;
            self.name.as_bytes()?.as_ref().to_bytes_with_provider(writer, provider)?;
        }
        self.value.to_bytes_with_provider(writer, provider)?;
        self.export_type.to_bytes_with_provider(writer, provider)
    }
}

impl FromBytes for ResolvedExport {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        provider: &P,
    ) -> wrt_error::Result<Self> {
        Ok(Self::default())
    }
}

/// Resource table for a specific resource type
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct ResourceTable {
    pub type_id: u32,
    // Simplified for now, would contain actual resource entries
}

/// Module instance within a component
#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct ModuleInstance {
    pub module_index: u32,
    // Simplified for now, would contain actual runtime instance
}

impl Checksummable for ModuleInstance {
    fn update_checksum(&self, checksum: &mut wrt_foundation::verification::Checksum) {
        self.module_index.update_checksum(checksum);
    }
}

impl ToBytes for ModuleInstance {
    fn to_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        &self,
        writer: &mut wrt_foundation::traits::WriteStream<'a>,
        provider: &P,
    ) -> wrt_error::Result<()> {
        self.module_index.to_bytes_with_provider(writer, provider)
    }
}

impl FromBytes for ModuleInstance {
    fn from_bytes_with_provider<'a, P: wrt_foundation::MemoryProvider>(
        reader: &mut wrt_foundation::traits::ReadStream<'a>,
        provider: &P,
    ) -> wrt_error::Result<Self> {
        Ok(Self {
            module_index: u32::from_bytes_with_provider(reader, provider)?,
        })
    }
}

/// Host function wrapper for the execution engine
#[cfg(feature = "std")]
struct HostFunctionWrapper {
    signature:      WrtComponentType,
    implementation: Box<dyn Fn(&[Value]) -> WrtResult<Value>>,
}

#[cfg(feature = "std")]
impl crate::execution_engine::HostFunction for HostFunctionWrapper {
    fn call(&mut self, args: &[Value]) -> WrtResult<Value> {
        (self.implementation)(args)
    }

    fn signature(&self) -> &WrtComponentType {
        &self.signature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_values() {
        let mut imports = ImportValues::new();

        #[cfg(feature = "std")]
        {
            let func = FunctionImport {
                signature:      WrtComponentType::Unit,
                implementation: Box::new(|_args| Ok(Value::U32(42))),
            };
            imports.add("test_func".to_owned(), ImportValue::Function(func)).unwrap();
            assert!(imports.get("test_func").is_some());
            assert!(imports.get("unknown").is_none());
        }

        #[cfg(not(any(feature = "std",)))]
        {
            let func = FunctionImport {
                signature:      WrtComponentType::Unit,
                implementation: |_args| Ok(Value::U32(42)),
            };
            let provider = safe_managed_alloc!(512, CrateId::Component).unwrap();
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

    // Tests migrated from simple_instantiation_test.rs

    #[test]
    fn test_value_imports() {
        let mut imports = ImportValues::new();

        let value_import = ImportValue::Value(WrtComponentValue::U32(100));

        #[cfg(feature = "std")]
        {
            let result = imports.add("test_value".to_owned(), value_import);
            assert!(result.is_ok());

            let retrieved = imports.get("test_value");
            assert!(retrieved.is_some());

            match retrieved.unwrap() {
                ImportValue::Value(WrtComponentValue::U32(val)) => {
                    assert_eq!(*val, 100);
                },
                _ => panic!("Expected value import"),
            }
        }

        #[cfg(not(feature = "std"))]
        {
            let provider = safe_managed_alloc!(512, CrateId::Component).unwrap();
            let name = BoundedString::from_str("test_value").unwrap();
            let result = imports.add(name, value_import);
            assert!(result.is_ok());

            let retrieved = imports.get("test_value");
            assert!(retrieved.is_some());

            match retrieved.unwrap() {
                ImportValue::Value(WrtComponentValue::U32(val)) => {
                    assert_eq!(*val, 100);
                },
                _ => panic!("Expected value import"),
            }
        }
    }

    // Note: The following tests are commented out as they depend on
    // components that may not be fully implemented yet.
    // They should be uncommented and adjusted as the implementation progresses.

    // #[test]
    // fn test_full_instantiation_context() {
    // let mut context = InstantiationContext::new();
    //
    // Verify all subsystems are initialized
    // assert_eq!(context.canonical_abi.string_encoding(),
    // crate::string_encoding::StringEncoding::Utf8; assert_eq!(context.
    // execution_engine.state(),
    // &crate::execution_engine::ExecutionState::Ready;
    //
    // Test registering a host function
    // #[cfg(not(feature = "std"))]
    // {
    // fn test_host_func(_args: &[Value]) -> WrtResult<Value> {
    // Ok(Value::Bool(true)
    // }
    //
    // let func_index =
    // context.execution_engine.register_host_function(test_host_func;
    // assert!(func_index.is_ok());
    // assert_eq!(func_index.unwrap(), 0);
    // }
    //
    // #[cfg(feature = "std")]
    // {
    // use crate::execution_engine::HostFunction;
    //
    // struct TestHostFunc;
    // impl HostFunction for TestHostFunc {
    // fn call(&mut self, _args: &[Value]) -> WrtResult<Value> {
    // Ok(Value::Bool(true)
    // }
    //
    // fn signature(&self) -> &WrtComponentType {
    // &WrtComponentType::Unit
    // }
    // }
    //
    // let func_index =
    // context.execution_engine.register_host_function(Box::new(TestHostFunc;
    // assert!(func_index.is_ok());
    // assert_eq!(func_index.unwrap(), 0);
    // }
    // }
    //
    // #[test]
    // fn test_resource_management() {
    // let mut context = InstantiationContext::new();
    //
    // Create a resource
    // let resource_data = WrtComponentValue::String("test_resource".into();
    // let handle = context.resource_manager.create_resource(1, resource_data;
    // assert!(handle.is_ok());
    //
    // let handle = handle.unwrap();
    //
    // Borrow the resource
    // let borrowed = context.resource_manager.borrow_resource(handle;
    // assert!(borrowed.is_ok());
    //
    // match borrowed.unwrap() {
    // WrtComponentValue::String(s) => {
    // assert_eq!(s.as_str(), "test_resource";
    // }
    // _ => panic!("Expected string resource"),
    // }
    //
    // Drop the resource
    // let drop_result = context.resource_manager.drop_resource(handle;
    // assert!(drop_result.is_ok());
    // }
    //
    // #[test]
    // fn test_memory_layout_integration() {
    // let _context = InstantiationContext::new();
    //
    // Test basic type layouts
    // use crate::types::ValType;
    // use crate::memory_layout;
    //
    // let bool_layout = memory_layout::calculate_layout(&ValType::Bool;
    // assert_eq!(bool_layout.size, 1);
    // assert_eq!(bool_layout.align, 1);
    //
    // let u32_layout = memory_layout::calculate_layout(&ValType::U32;
    // assert_eq!(u32_layout.size, 4;
    // assert_eq!(u32_layout.align, 4;
    //
    // let u64_layout = memory_layout::calculate_layout(&ValType::U64;
    // assert_eq!(u64_layout.size, 8;
    // assert_eq!(u64_layout.align, 8;
    // }
    //
    // #[test]
    // fn test_string_encoding_integration() {
    // let _context = InstantiationContext::new();
    //
    // use crate::string_encoding::{StringEncoding, encode_string};
    //
    // Test UTF-8 encoding (default)
    // let test_string = "Hello, 世界!";
    // let encoded = encode_string(test_string, StringEncoding::Utf8;
    // assert!(encoded.is_ok());
    //
    // let encoded_bytes = encoded.unwrap();
    // assert_eq!(encoded_bytes, test_string.as_bytes);
    //
    // Test UTF-16LE encoding
    // let encoded_utf16 = encode_string("Hello", StringEncoding::Utf16Le;
    // assert!(encoded_utf16.is_ok());
    //
    // UTF-16LE encoding of "Hello" should be: H(0x48,0x00) e(0x65,0x00)
    // l(0x6C,0x00) l(0x6C,0x00) o(0x6F,0x00) let expected_utf16 =
    // vec![0x48, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F, 0x00];
    // assert_eq!(encoded_utf16.unwrap(), expected_utf16;
    // }
}
