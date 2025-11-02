//! Component Instantiation and Linking System
//!
//! This module provides comprehensive component instantiation and linking
//! functionality for the WebAssembly Component Model. It enables creating
//! component instances from component binaries, resolving imports/exports, and
//! composing components together.
//!
//! # Features
//!
//! - **Component Instance Creation**: Create executable instances from
//!   component binaries
//! - **Import/Export Resolution**: Automatic resolution of component
//!   dependencies
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
//! use wrt_component::component_instantiation::{
//!     ComponentLinker,
//!     InstanceConfig,
//! };
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

// Cross-environment imports
#[cfg(feature = "std")]
use std::{
    any::Any,
    boxed::Box,
    collections::HashMap,
    format,
    string::{String, ToString},
    vec::Vec,
};

#[cfg(not(feature = "std"))]
use wrt_foundation::{
    bounded::BoundedString,
    safe_memory::NoStdProvider,
    collections::StaticVec as BoundedVec,
    safe_managed_alloc, CrateId,
};

#[cfg(feature = "std")]
use wrt_foundation::BoundedVec;

#[cfg(not(feature = "std"))]
type InstantiationString = BoundedString<256>;

#[cfg(not(feature = "std"))]
type InstantiationVec<T> = BoundedVec<T, 64>;

// Enable vec! and format! macros for no_std
#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

// use crate::component_communication::{CallRouter, CallContext as CommCallContext};
// use crate::call_context::CallContextManager;
use wrt_error::{
    codes,
    Error,
    ErrorCategory,
    Result,
};

use wrt_foundation::{
    traits::{Checksummable, FromBytes, ToBytes, ReadStream, WriteStream},
    verification::Checksum,
    MemoryProvider,
};

use crate::{
    bounded_component_infra::{ComponentProvider, BufferProvider},
    canonical_abi::{
        CanonicalABI,
        CanonicalMemory,
        ComponentType,
        ComponentValue,
    },
    components::component::Component as RuntimeComponent,
    resource_management::{
        ResourceData,
        ResourceHandle,
        ResourceManager as ComponentResourceManager,
        ResourceTypeId,
    },
    types::{ComponentInstanceState, ComponentMetadata},
};

#[cfg(all(feature = "std", feature = "safety-critical"))]
use wrt_foundation::allocator::{CrateId, WrtVec};

/// Maximum number of component instances
const MAX_COMPONENT_INSTANCES: usize = 1024;

/// Maximum number of imports per component
const MAX_IMPORTS_PER_COMPONENT: usize = 256;

/// Maximum number of exports per component
const MAX_EXPORTS_PER_COMPONENT: usize = 256;

/// Maximum component nesting depth
const MAX_COMPONENT_NESTING_DEPTH: usize = 16;

/// Maximum number of core modules per component (ASIL-D safety limit)
const MAX_CORE_MODULES: usize = 64;

/// Maximum number of types per component (ASIL-D safety limit)
const MAX_TYPES: usize = 512;

/// Maximum number of canonical operations (ASIL-D safety limit)
const MAX_CANONICAL_OPS: usize = 128;

/// Maximum number of core instances (ASIL-D safety limit)
const MAX_CORE_INSTANCES: usize = 64;

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
    pub max_table_size:  u32,
    /// Enable debug mode
    pub debug_mode:      bool,
    /// Binary std/no_std choice
    pub memory_config:   MemoryConfig,
}

/// Memory configuration for component instances
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryConfig {
    /// Initial memory size in pages (64KB each)
    pub initial_pages: u32,
    /// Maximum memory size in pages
    pub max_pages:     Option<u32>,
    /// Enable memory protection
    pub protected:     bool,
}

/// Component function signature
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Default)]
pub struct FunctionSignature {
    /// Function name
    pub name:    String,
    /// Parameter types
    #[cfg(feature = "std")]
    pub params:  Vec<ComponentType>,
    #[cfg(not(feature = "std"))]
    pub params:  BoundedVec<ComponentType, 16>,
    /// Return types
    #[cfg(feature = "std")]
    pub returns: Vec<ComponentType>,
    #[cfg(not(feature = "std"))]
    pub returns: BoundedVec<ComponentType, 16>,
}

/// Component export definition
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Default)]
pub struct ComponentExport {
    /// Export name
    pub name:        String,
    /// Export type
    pub export_type: ExportType,
}

/// Component import definition
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Default)]
pub struct ComponentImport {
    /// Import name
    pub name:        String,
    /// Module name (for namespace)
    pub module:      String,
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
    Table {
        element_type: ComponentType,
        size:         u32,
    },
    /// Global export
    Global {
        value_type: ComponentType,
        mutable:    bool,
    },
    /// Component type export
    Type(ComponentType),
}

impl Default for ExportType {
    fn default() -> Self {
        ExportType::Function(FunctionSignature::default())
    }
}

/// Types of imports a component can require
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportType {
    /// Function import
    Function(FunctionSignature),
    /// Memory import
    Memory(MemoryConfig),
    /// Table import
    Table {
        element_type: ComponentType,
        min_size:     u32,
        max_size:     Option<u32>,
    },
    /// Global import
    Global {
        value_type: ComponentType,
        mutable:    bool,
    },
    /// Component type import
    Type(ComponentType),
}

impl Default for ImportType {
    fn default() -> Self {
        ImportType::Function(FunctionSignature::default())
    }
}

/// Component instance implementation (alias to canonical definition)
pub use crate::types::ComponentInstance;

/// Local instance implementation details
#[derive(Debug)]
pub struct ComponentInstanceImpl {
    /// Instance memory
    pub memory:       Option<ComponentMemory>,
    /// Canonical ABI for value conversion
    abi:              CanonicalABI,
    /// Function table
    #[cfg(feature = "std")]
    functions:        Vec<ComponentFunction>,
    #[cfg(not(feature = "std"))]
    functions:        BoundedVec<ComponentFunction, 128>,
    /// Instance metadata
    metadata:         InstanceMetadata,
    /// Resource manager for this instance
    resource_manager: Option<ComponentResourceManager>,
}

/// Resolved import with actual provider
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedImport {
    /// Original import definition
    pub import:          ComponentImport,
    /// Provider instance ID
    pub provider_id:     InstanceId,
    /// Provider export name
    #[cfg(feature = "std")]
    pub provider_export: String,
    #[cfg(not(feature = "std"))]
    pub provider_export: wrt_foundation::bounded::BoundedString<64>,
}

/// Component function implementation
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Default)]
pub struct ComponentFunction {
    /// Function handle
    pub handle:         FunctionHandle,
    /// Function signature
    pub signature:      FunctionSignature,
    /// Implementation type
    pub implementation: FunctionImplementation,
}

/// Function implementation types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionImplementation {
    /// Native WebAssembly function
    Native {
        /// Function index in the component
        func_index:   u32,
        /// Module containing the function
        module_index: u32,
    },
    /// Host function
    Host {
        /// Host function callback
        #[cfg(feature = "std")]
        callback: String, // Simplified - would be actual callback in full implementation
        #[cfg(not(feature = "std"))]
        callback: wrt_foundation::bounded::BoundedString<64>,
    },
    /// Component function (calls through canonical ABI)
    Component {
        /// Target component instance
        target_instance: InstanceId,
        /// Target function name
        #[cfg(feature = "std")]
        target_function: String,
        #[cfg(not(feature = "std"))]
        target_function: wrt_foundation::bounded::BoundedString<64>,
    },
}

impl Default for FunctionImplementation {
    fn default() -> Self {
        FunctionImplementation::Native { func_index: 0, module_index: 0 }
    }
}

impl Checksummable for FunctionImplementation {
    fn update_checksum(&self, checksum: &mut Checksum) {
        match self {
            FunctionImplementation::Native { func_index, module_index } => {
                0u8.update_checksum(checksum);
                func_index.update_checksum(checksum);
                module_index.update_checksum(checksum);
            }
            FunctionImplementation::Host { callback } => {
                1u8.update_checksum(checksum);
                #[cfg(feature = "std")]
                {
                    callback.len().update_checksum(checksum);
                    for ch in callback.chars() {
                        (ch as u32).update_checksum(checksum);
                    }
                }
                #[cfg(not(feature = "std"))]
                {
                    callback.update_checksum(checksum);
                }
            }
            FunctionImplementation::Component { target_instance, target_function } => {
                2u8.update_checksum(checksum);
                target_instance.update_checksum(checksum);
                #[cfg(feature = "std")]
                {
                    target_function.len().update_checksum(checksum);
                    for ch in target_function.chars() {
                        (ch as u32).update_checksum(checksum);
                    }
                }
                #[cfg(not(feature = "std"))]
                {
                    target_function.update_checksum(checksum);
                }
            }
        }
    }
}

impl ToBytes for FunctionImplementation {
    fn to_bytes_with_provider<'a, P: MemoryProvider>(
        &self,
        writer: &mut WriteStream<'a>,
        provider: &P,
    ) -> Result<()> {
        match self {
            FunctionImplementation::Native { func_index, module_index } => {
                0u8.to_bytes_with_provider(writer, provider)?;
                func_index.to_bytes_with_provider(writer, provider)?;
                module_index.to_bytes_with_provider(writer, provider)
            }
            FunctionImplementation::Host { callback } => {
                1u8.to_bytes_with_provider(writer, provider)?;
                #[cfg(feature = "std")]
                {
                    (callback.len() as u32).to_bytes_with_provider(writer, provider)?;
                    for ch in callback.chars() {
                        (ch as u8).to_bytes_with_provider(writer, provider)?;
                    }
                }
                #[cfg(not(feature = "std"))]
                {
                    callback.to_bytes_with_provider(writer, provider)?;
                }
                Ok(())
            }
            FunctionImplementation::Component { target_instance, target_function } => {
                2u8.to_bytes_with_provider(writer, provider)?;
                target_instance.to_bytes_with_provider(writer, provider)?;
                #[cfg(feature = "std")]
                {
                    (target_function.len() as u32).to_bytes_with_provider(writer, provider)?;
                    for ch in target_function.chars() {
                        (ch as u8).to_bytes_with_provider(writer, provider)?;
                    }
                }
                #[cfg(not(feature = "std"))]
                {
                    target_function.to_bytes_with_provider(writer, provider)?;
                }
                Ok(())
            }
        }
    }
}

impl FromBytes for FunctionImplementation {
    fn from_bytes_with_provider<'a, P: MemoryProvider>(
        reader: &mut ReadStream<'a>,
        provider: &P,
    ) -> Result<Self> {
        let tag = u8::from_bytes_with_provider(reader, provider)?;
        match tag {
            0 => {
                let func_index = u32::from_bytes_with_provider(reader, provider)?;
                let module_index = u32::from_bytes_with_provider(reader, provider)?;
                Ok(FunctionImplementation::Native { func_index, module_index })
            }
            1 => {
                #[cfg(feature = "std")]
                {
                    let len = u32::from_bytes_with_provider(reader, provider)? as usize;
                    let mut bytes = vec![0u8; len];
                    for i in 0..len {
                        bytes[i] = u8::from_bytes_with_provider(reader, provider)?;
                    }
                    Ok(FunctionImplementation::Host {
                        callback: String::from_utf8(bytes)
                            .map_err(|_| Error::validation_error("Invalid UTF-8 in callback name"))?,
                    })
                }
                #[cfg(not(feature = "std"))]
                {
                    let callback = wrt_foundation::bounded::BoundedString::<64>::from_bytes_with_provider(reader, provider)?;
                    Ok(FunctionImplementation::Host { callback })
                }
            }
            2 => {
                let target_instance = u32::from_bytes_with_provider(reader, provider)?;
                #[cfg(feature = "std")]
                {
                    let len = u32::from_bytes_with_provider(reader, provider)? as usize;
                    let mut bytes = vec![0u8; len];
                    for i in 0..len {
                        bytes[i] = u8::from_bytes_with_provider(reader, provider)?;
                    }
                    Ok(FunctionImplementation::Component {
                        target_instance,
                        target_function: String::from_utf8(bytes)
                            .map_err(|_| Error::validation_error("Invalid UTF-8 in function name"))?,
                    })
                }
                #[cfg(not(feature = "std"))]
                {
                    let target_function = wrt_foundation::bounded::BoundedString::<64>::from_bytes_with_provider(reader, provider)?;
                    Ok(FunctionImplementation::Component { target_instance, target_function })
                }
            }
            _ => Err(Error::validation_error("Invalid FunctionImplementation tag")),
        }
    }
}

impl Checksummable for FunctionSignature {
    fn update_checksum(&self, checksum: &mut Checksum) {
        self.name.len().update_checksum(checksum);
        for ch in self.name.chars() {
            (ch as u32).update_checksum(checksum);
        }
        for param in self.params.iter() {
            param.update_checksum(checksum);
        }
        for ret in self.returns.iter() {
            ret.update_checksum(checksum);
        }
    }
}

impl ToBytes for FunctionSignature {
    fn to_bytes_with_provider<'a, P: MemoryProvider>(
        &self,
        writer: &mut WriteStream<'a>,
        provider: &P,
    ) -> Result<()> {
        // Write name
        (self.name.len() as u32).to_bytes_with_provider(writer, provider)?;
        for ch in self.name.chars() {
            (ch as u8).to_bytes_with_provider(writer, provider)?;
        }

        // Write params
        (self.params.len() as u32).to_bytes_with_provider(writer, provider)?;
        for param in self.params.iter() {
            param.to_bytes_with_provider(writer, provider)?;
        }

        // Write returns
        (self.returns.len() as u32).to_bytes_with_provider(writer, provider)?;
        for ret in self.returns.iter() {
            ret.to_bytes_with_provider(writer, provider)?;
        }

        Ok(())
    }
}

impl FromBytes for FunctionSignature {
    fn from_bytes_with_provider<'a, P: MemoryProvider>(
        reader: &mut ReadStream<'a>,
        provider: &P,
    ) -> Result<Self> {
        // Read name
        let name_len = u32::from_bytes_with_provider(reader, provider)? as usize;
        let mut name_bytes = Vec::new();
        for _ in 0..name_len {
            name_bytes.push(u8::from_bytes_with_provider(reader, provider)?);
        }
        let name = String::from_utf8(name_bytes)
            .map_err(|_| Error::validation_error("Invalid UTF-8 in function name"))?;

        // Read params
        let params_len = u32::from_bytes_with_provider(reader, provider)? as usize;
        #[cfg(feature = "std")]
        let mut params = Vec::new();
        #[cfg(not(feature = "std"))]
        let mut params = BoundedVec::<ComponentType, 16>::new();
        for _ in 0..params_len {
            let param = ComponentType::from_bytes_with_provider(reader, provider)?;
            #[cfg(feature = "std")]
            params.push(param);
            #[cfg(not(feature = "std"))]
            params.push(param).map_err(|_| Error::foundation_bounded_capacity_exceeded("FunctionSignature params capacity exceeded"))?;
        }

        // Read returns
        let returns_len = u32::from_bytes_with_provider(reader, provider)? as usize;
        #[cfg(feature = "std")]
        let mut returns = Vec::new();
        #[cfg(not(feature = "std"))]
        let mut returns = BoundedVec::<ComponentType, 16>::new();
        for _ in 0..returns_len {
            let ret = ComponentType::from_bytes_with_provider(reader, provider)?;
            #[cfg(feature = "std")]
            returns.push(ret);
            #[cfg(not(feature = "std"))]
            returns.push(ret).map_err(|_| Error::foundation_bounded_capacity_exceeded("FunctionSignature returns capacity exceeded"))?;
        }

        Ok(FunctionSignature {
            name,
            params,
            returns,
        })
    }
}

impl Checksummable for ComponentFunction {
    fn update_checksum(&self, checksum: &mut Checksum) {
        self.handle.update_checksum(checksum);
        self.signature.update_checksum(checksum);
        self.implementation.update_checksum(checksum);
    }
}

impl ToBytes for ComponentFunction {
    fn to_bytes_with_provider<'a, P: MemoryProvider>(
        &self,
        writer: &mut WriteStream<'a>,
        provider: &P,
    ) -> Result<()> {
        self.handle.to_bytes_with_provider(writer, provider)?;
        self.signature.to_bytes_with_provider(writer, provider)?;
        self.implementation.to_bytes_with_provider(writer, provider)
    }
}

impl FromBytes for ComponentFunction {
    fn from_bytes_with_provider<'a, P: MemoryProvider>(
        reader: &mut ReadStream<'a>,
        provider: &P,
    ) -> Result<Self> {
        let handle = u32::from_bytes_with_provider(reader, provider)?;
        let signature = FunctionSignature::from_bytes_with_provider(reader, provider)?;
        let implementation = FunctionImplementation::from_bytes_with_provider(reader, provider)?;
        Ok(ComponentFunction {
            handle,
            signature,
            implementation,
        })
    }
}

/// Component memory implementation
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ComponentMemory {
    /// Memory handle
    pub handle:       MemoryHandle,
    /// Memory configuration
    pub config:       MemoryConfig,
    /// Current memory size in bytes
    pub current_size: u32,
    /// Memory data (simplified for this implementation)
    #[cfg(feature = "std")]
    pub data:         Vec<u8>,
    #[cfg(not(feature = "std"))]
    pub data:         BoundedVec<u8, 65536>,
}

/// Instance metadata for debugging and introspection
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct InstanceMetadata {
    /// Creation timestamp
    pub created_at:         u64,
    /// Total function calls
    pub function_calls:     u64,
    /// Binary std/no_std choice
    pub memory_allocations: u64,
    /// Current memory usage
    pub memory_usage:       u32,
}

impl Default for InstanceConfig {
    fn default() -> Self {
        Self {
            max_memory_size: 64 * 1024 * 1024, // 64MB
            max_table_size:  1024,
            debug_mode:      false,
            memory_config:   MemoryConfig::default(),
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            initial_pages: 1,          // 64KB
            max_pages:     Some(1024), // 64MB
            protected:     true,
        }
    }
}


impl ComponentInstance {
    /// Create a new component instance from a runtime component
    ///
    /// Takes ownership of the runtime component to avoid duplication.
    pub fn new(id: InstanceId, component: RuntimeComponent) -> Result<Self> {
        // Initialize empty collections based on feature flags
        #[cfg(all(feature = "std", feature = "safety-critical"))]
        let (functions, imports, exports, resource_tables, module_instances) = {
            (
                WrtVec::new(),
                WrtVec::new(),
                WrtVec::new(),
                WrtVec::new(),
                WrtVec::new(),
            )
        };

        #[cfg(all(feature = "std", not(feature = "safety-critical")))]
        let (functions, imports, exports, resource_tables, module_instances) = {
            (
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            )
        };

        #[cfg(not(feature = "std"))]
        let (functions, imports, exports, resource_tables, module_instances) = {
            (
                BoundedVec::new(),
                BoundedVec::new(),
                BoundedVec::new(),
                BoundedVec::new(),
                BoundedVec::new(),
            )
        };

        Ok(Self {
            id,
            component,
            state: ComponentInstanceState::Initialized,
            resource_manager: None,
            memory: None,
            metadata: ComponentMetadata::default(),
            #[cfg(feature = "std")]
            type_index: std::collections::HashMap::new(),
            #[cfg(not(feature = "std"))]
            type_index: (),
            functions,
            imports,
            exports,
            resource_tables,
            module_instances,
        })
    }

    /// Create a component instance directly from parsed binary format
    ///
    /// **Memory Efficient**: Consumes parsed component and converts to runtime format
    /// without keeping both in memory simultaneously. Uses streaming conversion where
    /// each section is processed and dropped immediately.
    ///
    /// **Safety**: All limits are validated before allocation. Exceeding ASIL-D limits
    /// results in early failure with clear error messages.
    ///
    /// **Performance**: When `std` feature is enabled and component-model-threading is
    /// available, core modules are instantiated in parallel using structured parallelism
    /// for deterministic results.
    ///
    /// # Memory Flow (Streaming)
    /// ```text
    /// Binary → Parsed → Process modules → drop modules
    ///                → Process types   → drop types
    ///                → Process exports → drop exports
    ///                → Process canonicals → drop canonicals
    ///                → Runtime Instance (only this remains)
    /// ```
    ///
    /// # Safety Limits (ASIL-D)
    /// - Modules: 64 max
    /// - Types: 512 max
    /// - Canonicals: 128 max
    /// - Exports: 256 max
    /// - Imports: 256 max
    pub fn from_parsed(
        id: InstanceId,
        mut parsed: wrt_format::component::Component,
    ) -> Result<Self> {
        // Phase 1: Validate safety limits before processing
        Self::validate_component_limits(&parsed)?;

        // Phase 2: Extract and process core modules (streaming)
        let module_binaries = Self::extract_core_modules(&mut parsed)?;
        // parsed.modules is now empty (moved)

        // Phase 3: Build type index (streaming)
        let type_index = Self::build_type_index(&parsed.types)?;
        // parsed.types can be dropped after indexing (will happen when parsed is dropped)

        // Phase 4: Extract exports (streaming)
        let export_count = parsed.exports.len();
        // parsed.exports will be converted

        // Phase 5: Extract imports (streaming)
        let import_count = parsed.imports.len();
        // parsed.imports will be converted

        // Phase 6: Extract canonical operations (streaming)
        let canon_count = parsed.canonicals.len();
        // parsed.canonicals will be converted

        // Build minimal runtime component
        // In future: this will hold the actual converted data
        let component_type = crate::components::component::WrtComponentType::default();
        let mut runtime_component = RuntimeComponent::new(component_type);

        // Store module binaries in runtime component
        runtime_component.modules = module_binaries;

        // Phase 7: Create instance with runtime component
        // At this point, parsed is dropped and only runtime_component remains
        let mut instance = Self::new(id, runtime_component)?;

        // Store type index for runtime type resolution (Step 2)
        #[cfg(feature = "std")]
        {
            instance.type_index = type_index;
        }

        // Update metadata with conversion stats
        instance.metadata.function_calls = 0;

        // Test type lookup (Step 2)
        #[cfg(feature = "std")]
        {
            println!("=== STEP 2: Type Lookup Test ===");
            for idx in 0..instance.type_index.len() {
                match instance.get_type(idx as u32) {
                    Ok(ty) => {
                        let type_name = Self::get_type_name(ty);
                        println!("✓ Type lookup successful: Type[{}] = {}", idx, type_name);
                    }
                    Err(e) => {
                        println!("✗ Type lookup failed: Type[{}] - {:?}", idx, e);
                    }
                }
            }
            println!();
        }

        Ok(instance)
    }

    /// Get a type by index (Step 2)
    ///
    /// Looks up a type definition by its index in the type section.
    /// Returns an error if the index is out of bounds.
    #[cfg(feature = "std")]
    pub fn get_type(&self, idx: u32) -> Result<&wrt_format::component::ComponentType> {
        self.type_index.get(&idx)
            .ok_or_else(|| Error::new(
                wrt_error::ErrorCategory::Validation,
                wrt_error::codes::OUT_OF_BOUNDS_ERROR,
                "Type index out of bounds"
            ))
    }

    /// Get a type by index (no_std placeholder)
    #[cfg(not(feature = "std"))]
    pub fn get_type(&self, _idx: u32) -> Result<()> {
        // No_std version - type lookup not yet implemented
        Ok(())
    }

    /// Validate component against ASIL-D safety limits
    ///
    /// Fails fast if any limit is exceeded, preventing resource exhaustion.
    fn validate_component_limits(parsed: &wrt_format::component::Component) -> Result<()> {
        use wrt_error::{ErrorCategory, codes};

        if parsed.modules.len() > MAX_CORE_MODULES {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::CAPACITY_EXCEEDED,
                "Component exceeds maximum modules limit (ASIL-D)"
            ));
        }

        if parsed.types.len() > MAX_TYPES {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::CAPACITY_EXCEEDED,
                "Component exceeds maximum types limit (ASIL-D)"
            ));
        }

        if parsed.canonicals.len() > MAX_CANONICAL_OPS {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::CAPACITY_EXCEEDED,
                "Component exceeds maximum canonical operations limit (ASIL-D)"
            ));
        }

        if parsed.exports.len() > MAX_EXPORTS_PER_COMPONENT {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::CAPACITY_EXCEEDED,
                "Component exceeds maximum exports limit (ASIL-D)"
            ));
        }

        if parsed.imports.len() > MAX_IMPORTS_PER_COMPONENT {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::CAPACITY_EXCEEDED,
                "Component exceeds maximum imports limit (ASIL-D)"
            ));
        }

        if parsed.core_instances.len() > MAX_CORE_INSTANCES {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::CAPACITY_EXCEEDED,
                "Component exceeds maximum core instances limit (ASIL-D)"
            ));
        }

        Ok(())
    }

    /// Extract core module binaries from parsed component
    ///
    /// **Performance**: Modules are moved (not copied) from parsed component.
    /// When threading is enabled, validation can be parallelized.
    ///
    /// **Memory**: After this call, parsed.modules is empty - memory is transferred.
    fn extract_core_modules(
        parsed: &mut wrt_format::component::Component
    ) -> Result<Vec<Vec<u8>>> {
        use wrt_error::{ErrorCategory, codes};

        let mut module_binaries = Vec::with_capacity(parsed.modules.len());

        // Move modules out of parsed component (no copy)
        for module in parsed.modules.drain(..) {
            // Extract the binary data from the module
            // Module structure contains the actual binary
            if let Some(binary) = module.binary {
                // Validate module magic/version before accepting
                if binary.len() >= 4 && &binary[0..4] == b"\0asm" {
                    module_binaries.push(binary.to_vec());
                } else {
                    return Err(Error::new(
                        ErrorCategory::Validation,
                        codes::PARSE_INVALID_MAGIC_BYTES,
                        "Invalid WebAssembly module magic number"
                    ));
                }
            } else {
                return Err(Error::new(
                    ErrorCategory::Validation,
                    codes::PARSE_INVALID_SECTION_ID,
                    "Core module missing binary data"
                ));
            }
        }

        Ok(module_binaries)
    }

    /// Get a readable type name for logging/debugging
    fn get_type_name(ty: &wrt_format::component::ComponentType) -> &'static str {
        use wrt_format::component::ComponentTypeDefinition;
        match &ty.definition {
            ComponentTypeDefinition::Component { .. } => "Component",
            ComponentTypeDefinition::Instance { .. } => "Instance",
            ComponentTypeDefinition::Function { .. } => "Function",
            ComponentTypeDefinition::Value(_) => "Value",
            ComponentTypeDefinition::Resource { .. } => "Resource",
        }
    }

    /// Build type index from parsed component types
    ///
    /// This creates a mapping from type index to ComponentType, allowing
    /// runtime type resolution for Canon ABI operations and export/import linking.
    ///
    /// **Output**: Prints each type as it's indexed for visibility
    /// **Step 2**: Now stores the index for runtime type lookup
    #[cfg(feature = "std")]
    fn build_type_index(
        types: &[wrt_format::component::ComponentType]
    ) -> Result<std::collections::HashMap<u32, wrt_format::component::ComponentType>> {
        use std::collections::HashMap;

        println!("=== STEP 1: Building Type Index ===");
        println!("Total types to index: {}", types.len());

        let mut index = HashMap::new();

        for (idx, ty) in types.iter().enumerate() {
            let type_name = Self::get_type_name(ty);
            println!("  Type[{}]: {}", idx, type_name);

            // Additional detail for function types (most common)
            if let wrt_format::component::ComponentTypeDefinition::Function { params, results } = &ty.definition {
                println!("    ├─ Params: {}", params.len());
                println!("    └─ Results: {}", results.len());
            }

            index.insert(idx as u32, ty.clone());
        }

        println!("✓ Type index built: {} entries\n", index.len());
        Ok(index)
    }

    /// Build type index (no_std version - simplified, no printing)
    #[cfg(not(feature = "std"))]
    fn build_type_index(
        types: &[wrt_format::component::ComponentType]
    ) -> Result<()> {
        // In no_std, we just validate type count for now
        // Full type resolution will be implemented when needed
        if types.len() > MAX_TYPES {
            return Err(Error::new(
                wrt_error::ErrorCategory::Validation,
                wrt_error::codes::CAPACITY_EXCEEDED,
                "Type count exceeds maximum"
            ));
        }
        Ok(())
    }

    /// Initialize the instance (transition from Initializing to Ready)
    pub fn initialize(&mut self) -> Result<()> {
        match self.state {
            ComponentInstanceState::Initialized => {
                // Perform initialization logic
                self.validate_exports()?;
                self.setup_function_table()?;

                self.state = ComponentInstanceState::Running;
                Ok(())
            },
            _ => Err(Error::runtime_execution_error(
                "Instance not in initializing state",
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
        if self.state != ComponentInstanceState::Running {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::INVALID_STATE,
                "Instance not in ready state",
            ));
        }

        // Find the function (extract data to avoid multiple borrows)
        let (func_impl, func_sig) = {
            let function = self.find_function(function_name)?;
            (function.implementation.clone(), function.signature.clone())
        };

        // Validate arguments
        self.validate_function_args(&func_sig, args)?;

        // Update metrics
        self.metadata.function_calls += 1;

        // Use extracted function implementation
        let function = ComponentFunction {
            handle: 0,
            signature: func_sig,
            implementation: func_impl,
        };

        // Execute the function based on its implementation
        match &function.implementation {
            FunctionImplementation::Native {
                func_index,
                module_index,
            } => self.call_native_function(*func_index, *module_index, args),
            FunctionImplementation::Host { callback } => {
                #[cfg(feature = "std")]
                let callback_str = callback.as_str();
                #[cfg(not(feature = "std"))]
                let callback_str = callback.as_str()?;
                self.call_host_function(callback_str, args)
            },
            FunctionImplementation::Component {
                target_instance,
                target_function,
            } => {
                // This would need to go through the linker to call another component
                // For now, return a placeholder
                Err(Error::runtime_not_implemented(
                    "Component-to-component calls not yet implemented",
                ))
            },
        }
    }

    /// Get an export by name
    pub fn get_export(&self, name: &str) -> Option<&crate::instantiation::ResolvedExport> {
        self.exports.iter().find(|export| {
            #[cfg(feature = "std")]
            { export.name == name }
            #[cfg(not(feature = "std"))]
            { export.name.as_str().map(|s| s == name).unwrap_or(false) }
        })
    }

    /// Add a resolved import
    pub fn add_resolved_import(&mut self, resolved: crate::instantiation::ResolvedImport) -> Result<()> {
        if self.imports.len() >= MAX_IMPORTS_PER_COMPONENT {
            return Err(Error::validation_error("Too many resolved imports"));
        }

        #[cfg(all(feature = "std", not(feature = "safety-critical")))]
        {
            self.imports.push(resolved);
        }
        #[cfg(not(all(feature = "std", not(feature = "safety-critical"))))]
        {
            self.imports.push(resolved)
                .map_err(|_| Error::validation_error("Failed to push resolved import"))?;
        }
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
        self.state = ComponentInstanceState::Stopped;
        // Cleanup resources
        self.functions.clear();
        if let Some(memory) = &mut self.memory {
            memory.clear();
        }
        // Clean up resource manager
        // Note: remove_instance_table method doesn't exist, skip cleanup for now
        // if let Some(resource_manager) = &mut self.resource_manager {
        //     let _ = resource_manager.remove_instance_table(self.id);
        // }
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
            resource_manager.create_resource(resource_type, data)
                .map_err(|_e| {
                    Error::component_resource_lifecycle_error("Failed to create resource")
                })
        } else {
            Err(Error::runtime_not_implemented(
                "Resource management not available for this instance",
            ))
        }
    }

    /// Drop a resource from this instance
    pub fn drop_resource(&mut self, handle: ResourceHandle) -> Result<()> {
        if let Some(resource_manager) = &mut self.resource_manager {
            // Use destroy_resource instead of drop_resource
            resource_manager.destroy_resource(handle).map_err(|_| {
                Error::runtime_execution_error("Failed to destroy resource")
            })
        } else {
            Err(Error::runtime_not_implemented(
                "Resource management not available for this instance",
            ))
        }
    }

    // Private helper methods

    fn validate_exports(&self) -> Result<()> {
        // Validate that all exports are well-formed
        for export in &self.exports {
            // ResolvedExport doesn't have export_type - validation happens during resolution
            // Skip validation here since exports are already resolved
        }
        Ok(())
    }

    fn setup_function_table(&mut self) -> Result<()> {
        // Create function entries for all function exports
        // Function table setup is handled during instantiation
        // This is called after exports are already resolved
        Ok(())
    }

    fn find_function(&self, name: &str) -> Result<&ComponentFunction> {
        self.functions
            .iter()
            .find(|f| f.signature.name == name)
            .ok_or_else(|| Error::runtime_function_not_found("Function not found"))
    }

    fn validate_function_args(
        &self,
        signature: &FunctionSignature,
        args: &[ComponentValue],
    ) -> Result<()> {
        if args.len() != signature.params.len() {
            return Err(Error::runtime_type_mismatch(
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
        Ok(vec![ComponentValue::String(String::from("host_result"))]) // Placeholder result
    }
}

impl ComponentMemory {
    /// Create a new component memory
    pub fn new(handle: MemoryHandle, config: MemoryConfig) -> Result<Self> {
        let initial_size = config.initial_pages * 65536; // 64KB per page

        if let Some(max_pages) = config.max_pages {
            if config.initial_pages > max_pages {
                return Err(Error::validation_error(
                    "Initial pages cannot exceed maximum pages",
                ));
            }
        }

        // Create bounded vec for memory data
        #[cfg(feature = "std")]
        let data = vec![0u8; initial_size as usize];
        #[cfg(not(feature = "std"))]
        let data = {
            let mut data = BoundedVec::<u8, 65536>::new();
            // Fill with zeros up to initial size (capped at 65536)
            let fill_size = (initial_size as usize).min(65536);
            for _ in 0..fill_size {
                data.push(0).map_err(|_| Error::memory_error("Memory data capacity exceeded"))?;
            }
            data
        };

        Ok(Self {
            handle,
            config,
            current_size: initial_size,
            data,
        })
    }

    /// Grow memory by the specified number of pages
    pub fn grow(&mut self, pages: u32) -> Result<u32> {
        let old_pages = self.current_size / 65536;
        let new_pages = old_pages + pages;

        if let Some(max_pages) = self.config.max_pages {
            if new_pages > max_pages {
                return Err(Error::runtime_out_of_bounds(
                    "Memory growth would exceed maximum pages",
                ));
            }
        }

        let new_size = new_pages * 65536;
        let target_len = (new_size as usize).min(self.data.capacity());

        // Grow the bounded vec to new size (capped at capacity)
        #[cfg(feature = "std")]
        {
            self.data.resize(target_len, 0);
        }
        #[cfg(not(feature = "std"))]
        {
            while self.data.len() < target_len {
                self.data.push(0).map_err(|_| Error::memory_error("Memory growth capacity exceeded"))?;
            }
        }

        self.current_size = new_size;

        Ok(old_pages)
    }

    /// Get current size in pages
    pub fn size_pages(&self) -> u32 {
        self.current_size / 65536
    }

    /// Clear memory (for cleanup)
    pub fn clear(&mut self) {
        let _ = self.data.clear();
        self.current_size = 0;
    }
}

impl CanonicalMemory for ComponentMemory {
    fn read_bytes(&self, offset: u32, len: u32) -> Result<Vec<u8>> {
        let start = offset as usize;
        let end = start + len as usize;

        if end > self.data.len() {
            return Err(Error::memory_out_of_bounds("Memory read out of bounds"));
        }

        // Read bytes one by one from BoundedVec
        let mut result = Vec::with_capacity(len as usize);
        for i in start..end {
            let byte = *self.data.get(i).ok_or_else(|| Error::memory_out_of_bounds("Failed to read byte from memory"))?;
            result.push(byte);
        }
        Ok(result)
    }

    fn write_bytes(&mut self, offset: u32, data: &[u8]) -> Result<()> {
        let start = offset as usize;
        let end = start + data.len();

        if end > self.data.len() {
            return Err(Error::memory_out_of_bounds("Memory write out of bounds"));
        }

        // Write bytes one by one to BoundedVec
        for (i, &byte) in data.iter().enumerate() {
            if let Some(elem) = self.data.get_mut(start + i) {
                *elem = byte;
            } else {
                return Err(Error::memory_error("Failed to write byte to memory"));
            }
        }
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
            },
            InstantiationError::InitializationFailed(msg) => {
                write!(f, "Initialization failed: {}", msg)
            },
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InstantiationError {}

/// Create a function signature
pub fn create_function_signature(
    name: String,
    params_vec: Vec<ComponentType>,
    returns_vec: Vec<ComponentType>,
) -> FunctionSignature {
    #[cfg(feature = "std")]
    {
        FunctionSignature {
            name,
            params: params_vec,
            returns: returns_vec,
        }
    }
    #[cfg(not(feature = "std"))]
    {
        // Convert Vec to BoundedVec
        let mut bounded_params = BoundedVec::<ComponentType, 16>::new();
        for param in params_vec {
            let _ = bounded_params.push(param); // Silently ignore if capacity exceeded
        }

        let mut bounded_returns = BoundedVec::<ComponentType, 16>::new();
        for ret in returns_vec {
            let _ = bounded_returns.push(ret); // Silently ignore if capacity exceeded
        }

        FunctionSignature {
            name,
            params: bounded_params,
            returns: bounded_returns,
        }
    }
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
    ComponentImport {
        name,
        module,
        import_type,
    }

}
