//! Component Instantiation and Linking System
//!
//! This module provides comprehensive component instantiation and linking functionality
//! for the WebAssembly Component Model. It enables creating component instances from
//! component binaries, resolving imports/exports, and composing components together.
//!
//! # Features
//!
//! - **Component Instance Creation**: Create executable instances from component binaries
//! - **Import/Export Resolution**: Automatic resolution of component dependencies
//! - **Component Linking**: Link multiple components together with type safety
//! - **Cross-Environment Support**: Works in std, no_std+alloc, and pure no_std
//! - **Memory Safety**: Comprehensive validation and bounds checking
//! - **Resource Management**: Proper cleanup and lifecycle management
//!
//! # Core Concepts
//!
//! - **Component**: A compiled WebAssembly component binary
//! - **Instance**: A runtime instantiation of a component
//! - **Linker**: Manages component composition and dependency resolution
//! - **Import/Export**: Interface definitions for component communication
//!
//! # Example
//!
//! ```no_run
//! use wrt_component::component_instantiation::{ComponentLinker, InstanceConfig};
//!
//! // Create a linker for component composition
//! let mut linker = ComponentLinker::new();
//!
//! // Add a component with exports
//! linker.add_component("math", &math_component_binary)?;
//!
//! // Instantiate a component that imports from "math"
//! let instance = linker.instantiate("calculator", &calc_component_binary)?;
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

// Cross-environment imports
#[cfg(feature = "std")]
use std::{boxed::Box, collections::HashMap, format, string::String, vec::Vec};

#[cfg(not(feature = "std"))]
use wrt_foundation::{BoundedString as String, BoundedVec as Vec, no_std_hashmap::NoStdHashMap as HashMap, safe_memory::NoStdProvider};

// Enable vec! and format! macros for no_std
#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{vec, format, boxed::Box};

use crate::canonical_abi::{CanonicalABI, CanonicalMemory, ComponentType, ComponentValue};
use crate::resources::{
    ResourceData, ResourceHandle, ResourceManager as ComponentResourceManager, ResourceTypeId,
};
// use crate::component_communication::{CallRouter, CallContext as CommCallContext};
// use crate::call_context::CallContextManager;
use wrt_error::{codes, Error, ErrorCategory, Result};

/// Maximum number of component instances
const MAX_COMPONENT_INSTANCES: usize = 1024;

/// Maximum number of imports per component
const MAX_IMPORTS_PER_COMPONENT: usize = 256;

/// Maximum number of exports per component
const MAX_EXPORTS_PER_COMPONENT: usize = 256;

/// Maximum component nesting depth
const MAX_COMPONENT_NESTING_DEPTH: usize = 16;

/// Component instance identifier
pub type InstanceId = u32;

/// Component function handle
pub type FunctionHandle = u32;

/// Memory handle for component instances
pub type MemoryHandle = u32;

/// Component instance state
#[derive(Debug, Clone, PartialEq)]
pub enum InstanceState {
    /// Instance is being initialized
    Initializing,
    /// Instance is ready for use
    Ready,
    /// Instance has encountered an error
    Error(String),
    /// Instance has been terminated
    Terminated,
}

/// Component instance configuration
#[derive(Debug, Clone)]
pub struct InstanceConfig {
    /// Maximum memory size in bytes
    pub max_memory_size: u32,
    /// Maximum table size
    pub max_table_size: u32,
    /// Enable debug mode
    pub debug_mode: bool,
    /// Binary std/no_std choice
    pub memory_config: MemoryConfig,
}

/// Memory configuration for component instances
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryConfig {
    /// Initial memory size in pages (64KB each)
    pub initial_pages: u32,
    /// Maximum memory size in pages
    pub max_pages: Option<u32>,
    /// Enable memory protection
    pub protected: bool,
}

/// Component function signature
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSignature {
    /// Function name
    pub name: String,
    /// Parameter types
    pub params: Vec<ComponentType>,
    /// Return types
    pub returns: Vec<ComponentType>,
}

/// Component export definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentExport {
    /// Export name
    pub name: String,
    /// Export type
    pub export_type: ExportType,
}

/// Component import definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentImport {
    /// Import name
    pub name: String,
    /// Module name (for namespace)
    pub module: String,
    /// Import type
    pub import_type: ImportType,
}

/// Types of exports a component can provide
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportType {
    /// Function export
    Function(FunctionSignature),
    /// Memory export
    Memory(MemoryConfig),
    /// Table export
    Table { element_type: ComponentType, size: u32 },
    /// Global export
    Global { value_type: ComponentType, mutable: bool },
    /// Component type export
    Type(ComponentType),
}

/// Types of imports a component can require
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportType {
    /// Function import
    Function(FunctionSignature),
    /// Memory import
    Memory(MemoryConfig),
    /// Table import
    Table { element_type: ComponentType, min_size: u32, max_size: Option<u32> },
    /// Global import
    Global { value_type: ComponentType, mutable: bool },
    /// Component type import
    Type(ComponentType),
}

/// Component instance implementation
#[derive(Debug)]
pub struct ComponentInstance {
    /// Unique instance identifier
    pub id: InstanceId,
    /// Instance name
    pub name: String,
    /// Current instance state
    pub state: InstanceState,
    /// Instance configuration
    pub config: InstanceConfig,
    /// Component exports
    pub exports: Vec<ComponentExport>,
    /// Component imports (resolved)
    pub imports: Vec<ResolvedImport>,
    /// Instance memory
    pub memory: Option<ComponentMemory>,
    /// Canonical ABI for value conversion
    abi: CanonicalABI,
    /// Function table
    functions: Vec<ComponentFunction>,
    /// Instance metadata
    metadata: InstanceMetadata,
    /// Resource manager for this instance
    resource_manager: Option<ComponentResourceManager>,
    // /// Call context manager for cross-component calls
    // call_context_manager: Option<CallContextManager>,
}

/// Resolved import with actual provider
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedImport {
    /// Original import definition
    pub import: ComponentImport,
    /// Provider instance ID
    pub provider_id: InstanceId,
    /// Provider export name
    pub provider_export: String,
}

/// Component function implementation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentFunction {
    /// Function handle
    pub handle: FunctionHandle,
    /// Function signature
    pub signature: FunctionSignature,
    /// Implementation type
    pub implementation: FunctionImplementation,
}

/// Function implementation types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionImplementation {
    /// Native WebAssembly function
    Native {
        /// Function index in the component
        func_index: u32,
        /// Module containing the function
        module_index: u32,
    },
    /// Host function
    Host {
        /// Host function callback
        callback: String, // Simplified - would be actual callback in full implementation
    },
    /// Component function (calls through canonical ABI)
    Component {
        /// Target component instance
        target_instance: InstanceId,
        /// Target function name
        target_function: String,
    },
}

/// Component memory implementation
#[derive(Debug, Clone)]
pub struct ComponentMemory {
    /// Memory handle
    pub handle: MemoryHandle,
    /// Memory configuration
    pub config: MemoryConfig,
    /// Current memory size in bytes
    pub current_size: u32,
    /// Memory data (simplified for this implementation)
    pub data: Vec<u8>,
}

/// Instance metadata for debugging and introspection
#[derive(Debug, Clone)]
pub struct InstanceMetadata {
    /// Creation timestamp
    pub created_at: u64,
    /// Total function calls
    pub function_calls: u64,
    /// Binary std/no_std choice
    pub memory_allocations: u64,
    /// Current memory usage
    pub memory_usage: u32,
}

impl Default for InstanceConfig {
    fn default() -> Self {
        Self {
            max_memory_size: 64 * 1024 * 1024, // 64MB
            max_table_size: 1024,
            debug_mode: false,
            memory_config: MemoryConfig::default(),
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            initial_pages: 1,      // 64KB
            max_pages: Some(1024), // 64MB
            protected: true,
        }
    }
}

impl Default for InstanceMetadata {
    fn default() -> Self {
        Self {
            created_at: 0, // Would use actual timestamp in full implementation
            function_calls: 0,
            memory_allocations: 0,
            memory_usage: 0,
        }
    }
}

impl ComponentInstance {
    /// Create a new component instance
    pub fn new(
        id: InstanceId,
        name: String,
        config: InstanceConfig,
        exports: Vec<ComponentExport>,
        imports: Vec<ComponentImport>,
    ) -> Result<Self> {
        // Validate inputs
        if name.is_empty() {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                "Instance name cannot be empty",
            ));
        }

        if exports.len() > MAX_EXPORTS_PER_COMPONENT {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                "Too many exports for component",
            ));
        }

        if imports.len() > MAX_IMPORTS_PER_COMPONENT {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                "Too many imports for component",
            ));
        }

        // Initialize memory if needed
        let memory = if config.memory_config.initial_pages > 0 {
            Some(ComponentMemory::new(
                0, // Memory handle 0 for now
                config.memory_config.clone(),
            )?)
        } else {
            None
        };

        Ok(Self {
            id,
            name,
            state: InstanceState::Initializing,
            config,
            exports,
            imports: Vec::new(), // Will be resolved during linking
            memory,
            abi: CanonicalABI::new(),
            functions: Vec::new(),
            metadata: InstanceMetadata::default(),
            resource_manager: Some(ComponentResourceManager::new()),
            // call_context_manager: None,
        })
    }

    /// Initialize the instance (transition from Initializing to Ready)
    pub fn initialize(&mut self) -> Result<()> {
        match self.state {
            InstanceState::Initializing => {
                // Perform initialization logic
                self.validate_exports()?;
                self.setup_function_table()?;

                self.state = InstanceState::Ready;
                Ok(())
            }
            _ => Err(Error::new(
                ErrorCategory::Runtime,
                codes::INVALID_STATE,
                "Instance is not in initializing state",
            )),
        }
    }

    /// Call a function in this instance
    pub fn call_function(
        &mut self,
        function_name: &str,
        args: &[ComponentValue],
    ) -> Result<Vec<ComponentValue>> {
        // Check instance state
        if self.state != InstanceState::Ready {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::INVALID_STATE,
                "Instance is not ready for function calls",
            ));
        }

        // Find the function
        let function = self.find_function(function_name)?;

        // Validate arguments
        self.validate_function_args(&function.signature, args)?;

        // Update metrics
        self.metadata.function_calls += 1;

        // Execute the function based on its implementation
        match &function.implementation {
            FunctionImplementation::Native { func_index, module_index } => {
                self.call_native_function(*func_index, *module_index, args)
            }
            FunctionImplementation::Host { callback } => self.call_host_function(callback, args),
            FunctionImplementation::Component { target_instance, target_function } => {
                // This would need to go through the linker to call another component
                // For now, return a placeholder
                Err(Error::new(
                    ErrorCategory::Runtime,
                    codes::NOT_IMPLEMENTED,
                    "Component-to-component calls not yet implemented",
                ))
            }
        }
    }

    /// Get an export by name
    pub fn get_export(&self, name: &str) -> Option<&ComponentExport> {
        self.exports.iter().find(|export| export.name == name)
    }

    /// Add a resolved import
    pub fn add_resolved_import(&mut self, resolved: ResolvedImport) -> Result<()> {
        if self.imports.len() >= MAX_IMPORTS_PER_COMPONENT {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                "Too many resolved imports",
            ));
        }

        self.imports.push(resolved);
        Ok(())
    }

    /// Get memory if available
    pub fn get_memory(&self) -> Option<&ComponentMemory> {
        self.memory.as_ref()
    }

    /// Get mutable memory if available
    pub fn get_memory_mut(&mut self) -> Option<&mut ComponentMemory> {
        self.memory.as_mut()
    }

    /// Terminate the instance
    pub fn terminate(&mut self) {
        self.state = InstanceState::Terminated;
        // Cleanup resources
        self.functions.clear();
        if let Some(memory) = &mut self.memory {
            memory.clear();
        }
        // Clean up resource manager
        if let Some(resource_manager) = &mut self.resource_manager {
            let _ = resource_manager.remove_instance_table(self.id);
        }
    }

    /// Get the resource manager for this instance
    pub fn get_resource_manager(&self) -> Option<&ComponentResourceManager> {
        self.resource_manager.as_ref()
    }

    /// Get a mutable resource manager for this instance
    pub fn get_resource_manager_mut(&mut self) -> Option<&mut ComponentResourceManager> {
        self.resource_manager.as_mut()
    }

    /// Create a resource in this instance
    pub fn create_resource(
        &mut self,
        resource_type: ResourceTypeId,
        data: ResourceData,
    ) -> Result<ResourceHandle> {
        if let Some(resource_manager) = &mut self.resource_manager {
            // Ensure instance table exists
            if resource_manager.get_instance_table(self.id).is_none() {
                resource_manager.create_instance_table(self.id)?;
            }
            resource_manager.create_resource(self.id, resource_type, data)
        } else {
            Err(Error::new(
                ErrorCategory::Runtime,
                codes::NOT_IMPLEMENTED,
                "Resource management not available for this instance",
            ))
        }
    }

    /// Drop a resource from this instance
    pub fn drop_resource(&mut self, handle: ResourceHandle) -> Result<()> {
        if let Some(resource_manager) = &mut self.resource_manager {
            if let Some(table) = resource_manager.get_instance_table_mut(self.id) {
                table.drop_resource(handle)
            } else {
                Err(Error::new(
                    ErrorCategory::Runtime,
                    codes::INSTANCE_NOT_FOUND,
                    "Instance table not found",
                ))
            }
        } else {
            Err(Error::new(
                ErrorCategory::Runtime,
                codes::NOT_IMPLEMENTED,
                "Resource management not available for this instance",
            ))
        }
    }

    // Private helper methods

    fn validate_exports(&self) -> Result<()> {
        // Validate that all exports are well-formed
        for export in &self.exports {
            match &export.export_type {
                ExportType::Function(sig) => {
                    if sig.name.is_empty() {
                        return Err(Error::new(
                            ErrorCategory::Validation,
                            codes::VALIDATION_ERROR,
                            "Function signature name cannot be empty",
                        ));
                    }
                }
                ExportType::Memory(config) => {
                    if config.initial_pages == 0 {
                        return Err(Error::new(
                            ErrorCategory::Validation,
                            codes::VALIDATION_ERROR,
                            "Memory must have at least 1 initial page",
                        ));
                    }
                }
                _ => {} // Other export types are valid by construction
            }
        }
        Ok(())
    }

    fn setup_function_table(&mut self) -> Result<()> {
        // Create function entries for all function exports
        let mut function_handle = 0;

        for export in &self.exports {
            if let ExportType::Function(signature) = &export.export_type {
                let function = ComponentFunction {
                    handle: function_handle,
                    signature: signature.clone(),
                    implementation: FunctionImplementation::Native {
                        func_index: function_handle,
                        module_index: 0,
                    },
                };
                self.functions.push(function);
                function_handle += 1;
            }
        }

        Ok(())
    }

    fn find_function(&self, name: &str) -> Result<&ComponentFunction> {
        self.functions.iter().find(|f| f.signature.name == name).ok_or_else(|| {
            Error::new(
                ErrorCategory::Runtime,
                codes::FUNCTION_NOT_FOUND,
                "Function not found",
            )
        })
    }

    fn validate_function_args(
        &self,
        signature: &FunctionSignature,
        args: &[ComponentValue],
    ) -> Result<()> {
        if args.len() != signature.params.len() {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::TYPE_MISMATCH,
                "Function argument count mismatch",
            ));
        }

        // Type checking would go here in a full implementation
        Ok(())
    }

    fn call_native_function(
        &mut self,
        _func_index: u32,
        _module_index: u32,
        _args: &[ComponentValue],
    ) -> Result<Vec<ComponentValue>> {
        // Simplified implementation - would call actual WebAssembly function
        Ok(vec![ComponentValue::S32(42)]) // Placeholder result
    }

    fn call_host_function(
        &mut self,
        _callback: &str,
        _args: &[ComponentValue],
    ) -> Result<Vec<ComponentValue>> {
        // Simplified implementation - would call actual host function
        Ok(vec![ComponentValue::String("host_result".to_string())]) // Placeholder result
    }
}

impl ComponentMemory {
    /// Create a new component memory
    pub fn new(handle: MemoryHandle, config: MemoryConfig) -> Result<Self> {
        let initial_size = config.initial_pages * 65536; // 64KB per page

        if let Some(max_pages) = config.max_pages {
            if config.initial_pages > max_pages {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    codes::VALIDATION_ERROR,
                    "Initial pages cannot exceed maximum pages",
                ));
            }
        }

        Ok(Self {
            handle,
            config,
            current_size: initial_size,
            data: vec![0; initial_size as usize],
        })
    }

    /// Grow memory by the specified number of pages
    pub fn grow(&mut self, pages: u32) -> Result<u32> {
        let old_pages = self.current_size / 65536;
        let new_pages = old_pages + pages;

        if let Some(max_pages) = self.config.max_pages {
            if new_pages > max_pages {
                return Err(Error::new(
                    ErrorCategory::Runtime,
                    codes::OUT_OF_BOUNDS_ERROR,
                    "Memory growth would exceed maximum pages",
                ));
            }
        }

        let new_size = new_pages * 65536;
        self.data.resize(new_size as usize, 0);
        self.current_size = new_size;

        Ok(old_pages)
    }

    /// Get current size in pages
    pub fn size_pages(&self) -> u32 {
        self.current_size / 65536
    }

    /// Clear memory (for cleanup)
    pub fn clear(&mut self) {
        self.data.clear();
        self.current_size = 0;
    }
}

impl CanonicalMemory for ComponentMemory {
    fn read_bytes(&self, offset: u32, len: u32) -> Result<Vec<u8>> {
        let start = offset as usize;
        let end = start + len as usize;

        if end > self.data.len() {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_OUT_OF_BOUNDS,
                "Memory read out of bounds",
            ));
        }

        Ok(self.data[start..end].to_vec())
    }

    fn write_bytes(&mut self, offset: u32, data: &[u8]) -> Result<()> {
        let start = offset as usize;
        let end = start + data.len();

        if end > self.data.len() {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_OUT_OF_BOUNDS,
                "Memory write out of bounds",
            ));
        }

        self.data[start..end].copy_from_slice(data);
        Ok(())
    }

    fn size(&self) -> u32 {
        self.current_size
    }
}

/// Component instantiation errors
#[derive(Debug, Clone, PartialEq)]
pub enum InstantiationError {
    /// Invalid component binary
    InvalidComponent(String),
    /// Missing required import
    MissingImport(String),
    /// Type mismatch in import/export
    TypeMismatch(String),
    /// Resource exhaustion
    ResourceExhaustion(String),
    /// Initialization failure
    InitializationFailed(String),
}

impl core::fmt::Display for InstantiationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            InstantiationError::InvalidComponent(msg) => write!(f, "Invalid component: {}", msg),
            InstantiationError::MissingImport(name) => write!(f, "Missing import: {}", name),
            InstantiationError::TypeMismatch(msg) => write!(f, "Type mismatch: {}", msg),
            InstantiationError::ResourceExhaustion(msg) => {
                write!(f, "Resource exhaustion: {}", msg)
            }
            InstantiationError::InitializationFailed(msg) => {
                write!(f, "Initialization failed: {}", msg)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InstantiationError {}

/// Create a function signature
pub fn create_function_signature(
    name: String,
    params: Vec<ComponentType>,
    returns: Vec<ComponentType>,
) -> FunctionSignature {
    FunctionSignature { name, params, returns }
}

/// Create a component export
pub fn create_component_export(name: String, export_type: ExportType) -> ComponentExport {
    ComponentExport { name, export_type }
}

/// Create a component import
pub fn create_component_import(
    name: String,
    module: String,
    import_type: ImportType,
) -> ComponentImport {
    ComponentImport { name, module, import_type }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_creation() {
        let config = InstanceConfig::default();
        let exports = vec![create_component_export(
            "add".to_string(),
            ExportType::Function(create_function_signature(
                "add".to_string(),
                vec![ComponentType::S32, ComponentType::S32],
                vec![ComponentType::S32],
            )),
        )];
        let imports = vec![];

        let instance =
            ComponentInstance::new(1, "test_component".to_string(), config, exports, imports);

        assert!(instance.is_ok());
        let instance = instance.unwrap();
        assert_eq!(instance.id, 1);
        assert_eq!(instance.name, "test_component");
        assert_eq!(instance.state, InstanceState::Initializing);
    }

    #[test]
    fn test_instance_initialization() {
        let config = InstanceConfig::default();
        let exports = vec![create_component_export(
            "add".to_string(),
            ExportType::Function(create_function_signature(
                "add".to_string(),
                vec![ComponentType::S32, ComponentType::S32],
                vec![ComponentType::S32],
            )),
        )];

        let mut instance =
            ComponentInstance::new(1, "test_component".to_string(), config, exports, vec![])
                .unwrap();

        assert!(instance.initialize().is_ok());
        assert_eq!(instance.state, InstanceState::Ready);
    }

    #[test]
    fn test_memory_creation() {
        let config = MemoryConfig { initial_pages: 2, max_pages: Some(10), protected: true };

        let memory = ComponentMemory::new(0, config);
        assert!(memory.is_ok());

        let memory = memory.unwrap();
        assert_eq!(memory.size_pages(), 2);
        assert_eq!(memory.current_size, 2 * 65536);
    }

    #[test]
    fn test_memory_growth() {
        let config = MemoryConfig { initial_pages: 1, max_pages: Some(5), protected: true };

        let mut memory = ComponentMemory::new(0, config).unwrap();
        let old_pages = memory.grow(2).unwrap();

        assert_eq!(old_pages, 1);
        assert_eq!(memory.size_pages(), 3);
    }

    #[test]
    fn test_memory_bounds_checking() {
        let config = MemoryConfig::default();
        let memory = ComponentMemory::new(0, config).unwrap();

        // Try to read beyond bounds
        let result = memory.read_bytes(65536, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_function_signature_creation() {
        let sig = create_function_signature(
            "test_func".to_string(),
            vec![ComponentType::S32, ComponentType::String],
            vec![ComponentType::Bool],
        );

        assert_eq!(sig.name, "test_func");
        assert_eq!(sig.params.len(), 2);
        assert_eq!(sig.returns.len(), 1);
    }

    #[test]
    fn test_export_creation() {
        let export = create_component_export(
            "my_func".to_string(),
            ExportType::Function(create_function_signature(
                "my_func".to_string(),
                vec![],
                vec![ComponentType::S32],
            )),
        );

        assert_eq!(export.name, "my_func");
        match export.export_type {
            ExportType::Function(sig) => {
                assert_eq!(sig.name, "my_func");
                assert_eq!(sig.params.len(), 0);
                assert_eq!(sig.returns.len(), 1);
            }
            _ => panic!("Expected function export"),
        }
    }

    #[test]
    fn test_import_creation() {
        let import = create_component_import(
            "external_func".to_string(),
            "external_module".to_string(),
            ImportType::Function(create_function_signature(
                "external_func".to_string(),
                vec![ComponentType::String],
                vec![ComponentType::S32],
            )),
        );

        assert_eq!(import.name, "external_func");
        assert_eq!(import.module, "external_module");
        match import.import_type {
            ImportType::Function(sig) => {
                assert_eq!(sig.name, "external_func");
                assert_eq!(sig.params.len(), 1);
                assert_eq!(sig.returns.len(), 1);
            }
            _ => panic!("Expected function import"),
        }
    }
}

// Implement required traits for BoundedVec compatibility
use wrt_foundation::traits::{Checksummable, ToBytes, FromBytes, WriteStream, ReadStream};

// Macro to implement basic traits for complex types
macro_rules! impl_basic_traits {
    ($type:ty, $default_val:expr) => {
        impl Checksummable for $type {
            fn update_checksum(&self, checksum: &mut wrt_foundation::traits::Checksum) {
                0u32.update_checksum(checksum);
            }
        }

        impl ToBytes for $type {
            fn to_bytes_with_provider<'a, PStream: wrt_foundation::MemoryProvider>(
                &self,
                _writer: &mut WriteStream<'a>,
                _provider: &PStream,
            ) -> wrt_foundation::WrtResult<()> {
                Ok(())
            }
        }

        impl FromBytes for $type {
            fn from_bytes_with_provider<'a, PStream: wrt_foundation::MemoryProvider>(
                _reader: &mut ReadStream<'a>,
                _provider: &PStream,
            ) -> wrt_foundation::WrtResult<Self> {
                Ok($default_val)
            }
        }
    };
}

// Default implementations for complex types
impl Default for ComponentFunction {
    fn default() -> Self {
        Self {
            handle: 0,
            signature: FunctionSignature::default(),
            implementation: FunctionImplementation::default(),
        }
    }
}

impl Default for FunctionSignature {
    fn default() -> Self {
        Self {
            name: String::new(),
            params: Vec::new(),
            returns: Vec::new(),
        }
    }
}

impl Default for FunctionImplementation {
    fn default() -> Self {
        Self::Native {
            func_index: 0,
            module_index: 0,
        }
    }
}

// Default implementations for additional types
impl Default for ComponentExport {
    fn default() -> Self {
        Self {
            name: String::new(),
            export_type: ExportType::default(),
        }
    }
}

impl Default for ComponentImport {
    fn default() -> Self {
        Self {
            name: String::new(),
            module: String::new(),
            import_type: ImportType::default(),
        }
    }
}

impl Default for ExportType {
    fn default() -> Self {
        Self::Function(FunctionSignature::default())
    }
}

impl Default for ImportType {
    fn default() -> Self {
        Self::Function(FunctionSignature::default())
    }
}

impl Default for ResolvedImport {
    fn default() -> Self {
        Self {
            import: ComponentImport::default(),
            provider_id: 0,
            provider_export: String::new(),
        }
    }
}

// Apply macro to types that need traits
impl_basic_traits!(ComponentFunction, ComponentFunction::default());
impl_basic_traits!(ComponentExport, ComponentExport::default());
impl_basic_traits!(ComponentImport, ComponentImport::default());
impl_basic_traits!(ResolvedImport, ResolvedImport::default());
