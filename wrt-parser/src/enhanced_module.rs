//! Enhanced module structure with Component Model support
//!
//! This module provides the EnhancedModule structure that wraps SimpleModule
//! with optional Component Model support while maintaining backward compatibility.

use wrt_error::{Error, ErrorCategory, Result, codes};
use crate::simple_module::SimpleModule;
use crate::component_types::{ComponentType, ComponentValueType, TypeRef};
use crate::component_registry::{ComponentRegistry, ComponentParserState};
use crate::bounded_types::{SimpleBoundedVec, SimpleBoundedString};

/// Enhanced module with Component Model support
/// 
/// Wraps SimpleModule with optional Component Model support. When Component Model
/// features are not used, this adds zero overhead to existing parsing workflows.
#[derive(Debug, Clone)]
pub struct EnhancedModule {
    /// Core WebAssembly module (always present)
    pub core: SimpleModule,
    
    /// Component Model support (optional)
    pub component: Option<ComponentModel>,
    
    /// Parser mode used to create this module
    pub parser_mode: ParserMode,
}

/// Component Model data for enhanced modules
#[derive(Debug, Clone)]
pub struct ComponentModel {
    /// Component types registry
    pub types: SimpleBoundedVec<ComponentType, 512>,
    
    /// Component imports
    pub imports: SimpleBoundedVec<ComponentImport, 256>,
    
    /// Component exports  
    pub exports: SimpleBoundedVec<ComponentExport, 256>,
    
    /// Component instances
    pub instances: SimpleBoundedVec<ComponentInstance, 128>,
    
    /// Component functions (canonical functions)
    pub functions: SimpleBoundedVec<ComponentFunction, 256>,
    
    /// Component values
    pub values: SimpleBoundedVec<ComponentValue, 64>,
    
    /// Component start function (optional)
    pub start: Option<ComponentStart>,
    
    /// Embedded core modules
    pub core_modules: SimpleBoundedVec<crate::simple_module::SimpleModule, 32>,
    
    /// Core function types
    pub core_types: SimpleBoundedVec<crate::types::FuncType, 128>,
    
    /// Nested components (as raw bytes for now)
    pub nested_components: SimpleBoundedVec<Vec<u8>, 16>,
    
    /// Component aliases
    pub aliases: SimpleBoundedVec<ComponentAlias, 256>,
}

/// Component import definition
#[derive(Debug, Clone)]
pub struct ComponentImport {
    /// Import namespace and name
    pub name: ImportName,
    /// Type of the imported item
    pub ty: TypeRef,
}

/// Component export definition
#[derive(Debug, Clone)]
pub struct ComponentExport {
    /// Export name
    pub name: SimpleBoundedString<128>,
    /// Type of the exported item
    pub ty: TypeRef,
    /// Index of the exported item
    pub item_index: u32,
}

/// Component instance definition
#[derive(Debug, Clone)]
pub struct ComponentInstance {
    /// Module or component index being instantiated
    pub module_index: u32,
    /// Instance type reference
    pub ty: TypeRef,
    /// Instantiation arguments
    pub args: SimpleBoundedVec<InstantiationArg, 32>,
}

/// Component function (canonical function)
#[derive(Debug, Clone)]
pub struct ComponentFunction {
    /// Function type reference
    pub ty: TypeRef,
    /// Core function index (for lifted functions)
    pub core_func_index: Option<u32>,
    /// Options for canonical conversion
    pub options: CanonicalOptions,
}

/// Component value definition
#[derive(Debug, Clone)]
pub enum ComponentValue {
    /// Instance value reference
    Instance(u32),
    /// Function value reference
    Function(u32),
    /// Other value with type and data
    Other {
        /// Value type reference
        ty: TypeRef,
        /// Value data
        data: SimpleBoundedVec<u8, 1024>,
    }
}

/// Component start function
#[derive(Debug, Clone)]
pub struct ComponentStart {
    /// Function index to start
    pub func_index: u32,
    /// Arguments for start function
    pub args: SimpleBoundedVec<u32, 16>,
}

/// Import name with namespace
#[derive(Debug, Clone)]
pub struct ImportName {
    /// Namespace (e.g., "wasi:filesystem")
    pub namespace: SimpleBoundedString<128>,
    /// Item name (e.g., "read-file")
    pub name: SimpleBoundedString<128>,
}

/// Instantiation argument
#[derive(Debug, Clone)]
pub struct InstantiationArg {
    /// Argument name
    pub name: SimpleBoundedString<64>,
    /// Item index
    pub index: u32,
    /// Item kind
    pub kind: ItemKind,
}

/// Component alias definition
#[derive(Debug, Clone)]
pub struct ComponentAlias {
    /// Alias kind
    pub kind: u8,
    /// Target instance index
    pub instance_index: u32,
    /// Target item index
    pub item_index: u32,
}

/// Canonical function options
#[derive(Debug, Clone)]
pub struct CanonicalOptions {
    /// Memory index for string/list conversions
    pub memory: Option<u32>,
    /// Realloc function index
    pub realloc: Option<u32>,
    /// String encoding
    pub string_encoding: StringEncoding,
}

/// String encoding for canonical functions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringEncoding {
    Utf8,
    Utf16,
    Latin1,
}

/// Item kind for imports/exports
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Function,
    Table,
    Memory,
    Global,
    Type,
    Instance,
    Component,
    Value,
}

/// Parser mode for enhanced modules
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParserMode {
    /// Core WebAssembly only (zero overhead)
    CoreOnly,
    /// Component Model aware with memory budget
    ComponentAware {
        /// Memory budget for Component Model types
        type_budget: usize,
        /// Maximum number of types
        max_types: usize,
    },
}

impl EnhancedModule {
    /// Create a new enhanced module with core only
    pub fn new_core_only(core: SimpleModule) -> Self {
        Self {
            core,
            component: None,
            parser_mode: ParserMode::CoreOnly,
        }
    }
    
    /// Create a new enhanced module with Component Model support
    pub fn new_with_component(
        core: SimpleModule,
        mode: ParserMode,
    ) -> Result<Self> {
        let component = match mode {
            ParserMode::CoreOnly => None,
            ParserMode::ComponentAware { .. } => Some(ComponentModel::new()?),
        };
        
        Ok(Self {
            core,
            component,
            parser_mode: mode,
        })
    }
    
    /// Check if Component Model is enabled
    pub fn has_component_model(&self) -> bool {
        self.component.is_some()
    }
    
    /// Get component model (if enabled)
    pub fn component(&self) -> Option<&ComponentModel> {
        self.component.as_ref()
    }
    
    /// Get mutable component model (if enabled)
    pub fn component_mut(&mut self) -> Option<&mut ComponentModel> {
        self.component.as_mut()
    }
    
    /// Get parser mode
    pub fn parser_mode(&self) -> ParserMode {
        self.parser_mode
    }
    
    /// Convert to core module (backward compatibility)
    pub fn into_core(self) -> SimpleModule {
        self.core
    }
    
    /// Get core module reference
    pub fn core(&self) -> &SimpleModule {
        &self.core
    }
    
    /// Get mutable core module reference
    pub fn core_mut(&mut self) -> &mut SimpleModule {
        &mut self.core
    }
}

impl ComponentModel {
    /// Create a new empty component model
    pub fn new() -> Result<Self> {
        Ok(Self {
            types: SimpleBoundedVec::new(),
            imports: SimpleBoundedVec::new(),
            exports: SimpleBoundedVec::new(),
            instances: SimpleBoundedVec::new(),
            functions: SimpleBoundedVec::new(),
            values: SimpleBoundedVec::new(),
            start: None,
            core_modules: SimpleBoundedVec::new(),
            core_types: SimpleBoundedVec::new(),
            nested_components: SimpleBoundedVec::new(),
            aliases: SimpleBoundedVec::new(),
        })
    }
    
    /// Add a component type
    pub fn add_type(&mut self, component_type: ComponentType) -> Result<TypeRef> {
        let type_ref = self.types.len() as TypeRef;
        self.types.push(component_type).map_err(|_| Error::new(
            ErrorCategory::Memory,
            codes::CAPACITY_EXCEEDED,
            "Component types capacity exceeded"
        ))?;
        Ok(type_ref)
    }
    
    /// Get a component type by reference
    pub fn get_type(&self, type_ref: TypeRef) -> Option<&ComponentType> {
        self.types.get(type_ref as usize)
    }
    
    /// Add a component import
    pub fn add_import(&mut self, import: ComponentImport) -> Result<()> {
        self.imports.push(import).map_err(|_| Error::new(
            ErrorCategory::Memory,
            codes::CAPACITY_EXCEEDED,
            "Component imports capacity exceeded"
        ))?;
        Ok(())
    }
    
    /// Add a component export
    pub fn add_export(&mut self, export: ComponentExport) -> Result<()> {
        self.exports.push(export).map_err(|_| Error::new(
            ErrorCategory::Memory,
            codes::CAPACITY_EXCEEDED,
            "Component exports capacity exceeded"
        ))?;
        Ok(())
    }
    
    /// Add a component instance
    pub fn add_instance(&mut self, instance: ComponentInstance) -> Result<u32> {
        let index = self.instances.len() as u32;
        self.instances.push(instance).map_err(|_| Error::new(
            ErrorCategory::Memory,
            codes::CAPACITY_EXCEEDED,
            "Component instances capacity exceeded"
        ))?;
        Ok(index)
    }
    
    /// Add a component function
    pub fn add_function(&mut self, function: ComponentFunction) -> Result<u32> {
        let index = self.functions.len() as u32;
        self.functions.push(function).map_err(|_| Error::new(
            ErrorCategory::Memory,
            codes::CAPACITY_EXCEEDED,
            "Component functions capacity exceeded"
        ))?;
        Ok(index)
    }
    
    /// Add a component value
    pub fn add_value(&mut self, value: ComponentValue) -> Result<u32> {
        let index = self.values.len() as u32;
        self.values.push(value).map_err(|_| Error::new(
            ErrorCategory::Memory,
            codes::CAPACITY_EXCEEDED,
            "Component values capacity exceeded"
        ))?;
        Ok(index)
    }
    
    /// Set component start function
    pub fn set_start(&mut self, start: ComponentStart) {
        self.start = Some(start);
    }
    
    /// Get component start function
    pub fn start(&self) -> Option<&ComponentStart> {
        self.start.as_ref()
    }
    
    /// Add a core module
    pub fn add_core_module(&mut self, module: crate::simple_module::SimpleModule) -> Result<u32> {
        let index = self.core_modules.len() as u32;
        self.core_modules.push(module).map_err(|_| Error::new(
            ErrorCategory::Memory,
            codes::CAPACITY_EXCEEDED,
            "Core modules capacity exceeded"
        ))?;
        Ok(index)
    }
    
    /// Add a core type
    pub fn add_core_type(&mut self, func_type: crate::types::FuncType) -> Result<u32> {
        let index = self.core_types.len() as u32;
        self.core_types.push(func_type).map_err(|_| Error::new(
            ErrorCategory::Memory,
            codes::CAPACITY_EXCEEDED,
            "Core types capacity exceeded"
        ))?;
        Ok(index)
    }
    
    /// Add a nested component
    pub fn add_nested_component(&mut self, component_data: Vec<u8>) -> Result<u32> {
        let index = self.nested_components.len() as u32;
        self.nested_components.push(component_data).map_err(|_| Error::new(
            ErrorCategory::Memory,
            codes::CAPACITY_EXCEEDED,
            "Nested components capacity exceeded"
        ))?;
        Ok(index)
    }
    
    /// Add a component alias
    pub fn add_alias(&mut self, alias: ComponentAlias) -> Result<u32> {
        let index = self.aliases.len() as u32;
        self.aliases.push(alias).map_err(|_| Error::new(
            ErrorCategory::Memory,
            codes::CAPACITY_EXCEEDED,
            "Component aliases capacity exceeded"
        ))?;
        Ok(index)
    }
    
    /// Get number of types
    pub fn type_count(&self) -> usize {
        self.types.len()
    }
    
    /// Get number of imports
    pub fn import_count(&self) -> usize {
        self.imports.len()
    }
    
    /// Get number of exports
    pub fn export_count(&self) -> usize {
        self.exports.len()
    }
}

impl ImportName {
    /// Create a new import name
    pub fn new(namespace: &str, name: &str) -> Self {
        Self {
            namespace: SimpleBoundedString::from_str(namespace),
            name: SimpleBoundedString::from_str(name),
        }
    }
    
    /// Get full import path
    pub fn full_path(&self) -> String {
        format!("{}:{}", self.namespace.as_str(), self.name.as_str())
    }
}

impl CanonicalOptions {
    /// Create default canonical options
    pub fn default() -> Self {
        Self {
            memory: None,
            realloc: None,
            string_encoding: StringEncoding::Utf8,
        }
    }
    
    /// Create with memory index
    pub fn with_memory(memory: u32) -> Self {
        Self {
            memory: Some(memory),
            realloc: None,
            string_encoding: StringEncoding::Utf8,
        }
    }
}

impl Default for ParserMode {
    fn default() -> Self {
        Self::CoreOnly
    }
}

impl Default for StringEncoding {
    fn default() -> Self {
        Self::Utf8
    }
}

impl Default for ComponentModel {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl Default for CanonicalOptions {
    fn default() -> Self {
        Self::default()
    }
}

impl From<SimpleModule> for EnhancedModule {
    fn from(core: SimpleModule) -> Self {
        Self::new_core_only(core)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component_types::{ComponentTypeDefinition, ComponentValueType};
    
    #[test]
    fn test_enhanced_module_core_only() {
        let core = SimpleModule::new();
        let enhanced = EnhancedModule::new_core_only(core);
        
        assert!(!enhanced.has_component_model());
        assert_eq!(enhanced.parser_mode(), ParserMode::CoreOnly);
        assert!(enhanced.component().is_none());
    }
    
    #[test]
    fn test_enhanced_module_with_component() {
        let core = SimpleModule::new();
        let mode = ParserMode::ComponentAware {
            type_budget: 64 * 1024,
            max_types: 512,
        };
        
        let enhanced = EnhancedModule::new_with_component(core, mode).unwrap();
        
        assert!(enhanced.has_component_model());
        assert_eq!(enhanced.parser_mode(), mode);
        assert!(enhanced.component().is_some());
    }
    
    #[test]
    fn test_component_model_type_management() {
        let mut component = ComponentModel::new().unwrap();
        
        let func_type = ComponentType {
            definition: ComponentTypeDefinition::Function {
                params: SimpleBoundedVec::new(),
                results: SimpleBoundedVec::new(),
            }
        };
        
        let type_ref = component.add_type(func_type.clone()).unwrap();
        assert_eq!(type_ref, 0);
        assert_eq!(component.type_count(), 1);
        
        let retrieved = component.get_type(type_ref).unwrap();
        assert_eq!(retrieved.definition, func_type.definition);
    }
    
    #[test]
    fn test_import_name_formatting() {
        let import_name = ImportName::new("wasi:filesystem", "read-file");
        assert_eq!(import_name.full_path(), "wasi:filesystem:read-file");
    }
    
    #[test]
    fn test_backward_compatibility() {
        let core = SimpleModule::new();
        let enhanced: EnhancedModule = core.into();
        
        assert!(!enhanced.has_component_model());
        assert_eq!(enhanced.parser_mode(), ParserMode::CoreOnly);
        
        // Can convert back to core module
        let _core_again = enhanced.into_core();
    }
}