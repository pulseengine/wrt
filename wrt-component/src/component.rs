//! Component Model implementation for the WebAssembly Runtime.
//!
//! This module provides types and implementations for the WebAssembly Component Model.

use crate::{
    canonical::CanonicalABI,
    export::{Export, ExportList},
    import::{Import, ImportList},
    namespace::Namespace,
    resources::{BufferPool, MemoryStrategy, Resource, ResourceTable},
    values::{deserialize_component_values, serialize_component_values},
};

#[cfg(feature = "std")]
use log::{debug, error, info, trace, warn};

#[cfg(feature = "std")]
use std::collections::{HashMap, HashSet};
#[cfg(feature = "std")]
use std::sync::{Arc, Mutex};

#[cfg(not(feature = "std"))]
use alloc::collections::{BTreeMap as HashMap, BTreeSet as HashSet};
#[cfg(not(feature = "std"))]
use alloc::sync::{Arc, Mutex};

use wrt_error::{kinds, kinds::*, Error, Result};
use wrt_types::ComponentValue;

// Use direct imports from wrt-types
use wrt_types::{
    builtin::BuiltinType, ComponentType, ExportType, ExternType as TypesExternType, FuncType,
    GlobalType, InstanceType, Limits, MemoryType as TypesMemoryType, Namespace, ResourceType,
    TableType as TypesTableType, TypeIndex, ValueType,
};

// Use format types with explicit namespacing
use wrt_format::{
    binary,
    component::{
        ComponentSection, ComponentTypeDefinition as FormatComponentTypeDefinition,
        ComponentTypeSection, ExportSection, ExternType as FormatExternType, ImportSection,
        InstanceSection, ResourceOperation as FormatResourceOperation, ValType as FormatValType,
    },
};

// Import conversion functions
use wrt_format::component_conversion::{
    component_type_to_format_type_def, format_type_def_to_component_type,
};

use wrt_host::function::HostFunctionHandler;
use wrt_host::CallbackRegistry;
use wrt_intercept::{InterceptionResult, LinkInterceptor, LinkInterceptorStrategy};

// Runtime types with explicit namespacing
use wrt_runtime::types::{MemoryType, TableType};
use wrt_runtime::{
    func::FuncType as RuntimeFuncType,
    global::{Global, GlobalType},
    memory::Memory,
    table::Table,
};

// Standard imports
#[cfg(feature = "std")]
use std::{
    collections::HashMap,
    string::String,
    sync::{Arc, RwLock},
    vec,
    vec::Vec,
};

#[cfg(not(feature = "std"))]
use alloc::{
    collections::BTreeMap as HashMap,
    string::{String, ToString},
    sync::Arc,
    vec,
    vec::Vec,
};

#[cfg(not(feature = "std"))]
use core::cell::RefCell as RwLock;

use crate::execution::{run_with_time_bounds, TimeBoundedConfig, TimeBoundedOutcome};

#[cfg(not(feature = "std"))]
use core::any::Any;
#[cfg(feature = "std")]
use std::any::Any;

#[cfg(not(feature = "std"))]
use alloc::borrow::Cow;
#[cfg(feature = "std")]
use std::borrow::Cow;

#[cfg(not(feature = "std"))]
use alloc::collections::VecDeque;
#[cfg(feature = "std")]
use std::collections::VecDeque;

#[cfg(feature = "std")]
use core::str;
#[cfg(not(feature = "std"))]
use core::str;

// Import type conversion utilities for clear transformations
use crate::type_conversion::{
    common_to_format_val_type, extern_type_to_func_type, format_to_common_val_type,
    format_to_types_extern_type, format_val_type_to_types_valtype, format_val_type_to_value_type,
    types_to_format_extern_type, types_valtype_to_format_valtype, value_type_to_format_val_type,
};

use crate::debug_println;

// Import wrt_decoder types
use wrt_decoder::{
    Component as DecodedComponent, ComponentDecoder, ExportType, ImportType, ProducersSection,
    TypeDef,
};

/// Represents a component type
#[derive(Debug, Clone)]
pub struct WrtComponentType {
    /// Component imports
    pub imports: Vec<(String, String, ExternType)>,
    /// Component exports
    pub exports: Vec<(String, ExternType)>,
    /// Component instances
    pub instances: Vec<wrt_format::component::ComponentTypeDefinition>,
}

impl WrtComponentType {
    /// Creates a new empty component type
    pub fn new() -> Self {
        Self {
            imports: Vec::new(),
            exports: Vec::new(),
            instances: Vec::new(),
        }
    }

    /// Create a new empty component type
    pub fn empty() -> Self {
        Self {
            imports: Vec::new(),
            exports: Vec::new(),
            instances: Vec::new(),
        }
    }
}

impl Default for WrtComponentType {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a component instance
#[derive(Debug)]
pub struct Component {
    /// Component type
    pub(crate) component_type: WrtComponentType,
    /// Component exports
    pub(crate) exports: Vec<Export>,
    /// Component imports
    pub(crate) imports: Vec<Import>,
    /// Component instances
    pub(crate) instances: Vec<InstanceValue>,
    /// Linked components with their namespaces
    pub(crate) linked_components: HashMap<String, Arc<Component>>,
    /// Host callback registry
    pub(crate) callback_registry: Option<Arc<CallbackRegistry>>,
    /// Runtime instance
    pub(crate) runtime: Option<RuntimeInstance>,
    /// Interceptor for function calls
    pub(crate) interceptor: Option<Arc<LinkInterceptor>>,
    /// Resource table for managing component resources
    pub resource_table: ResourceTable,
    /// Built-in requirements
    pub(crate) built_in_requirements: Option<BuiltinRequirements>,
    /// Original binary
    pub(crate) original_binary: Option<Vec<u8>>,
}

/// Represents an instance value
#[derive(Debug, Clone)]
pub struct InstanceValue {
    /// Instance type
    pub ty: wrt_format::component::ComponentTypeDefinition,
    /// Instance exports
    pub exports: Vec<Export>,
}

/// Represents a runtime instance for executing WebAssembly code
#[derive(Debug)]
pub struct RuntimeInstance {
    // Runtime implementation details would go here in a concrete implementation
    // Fields might include:
    // - Reference to WebAssembly module instance
    // - Memory and table instances
    // - Execution context
}

impl Default for RuntimeInstance {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeInstance {
    /// Creates a new runtime instance
    pub fn new() -> Self {
        Self {}
    }

    /// Executes a WebAssembly function by name with the given arguments
    pub fn execute_function(&self, _name: &str, _args: Vec<Value>) -> Result<Vec<Value>> {
        // This would be implemented by the concrete runtime implementation
        // It would:
        // 1. Look up the function by name
        // 2. Convert arguments to the appropriate format
        // 3. Execute the function
        // 4. Convert results back to Values

        Err(Error::new(kinds::NotImplementedError(format!(
            "Function execution for '{}' requires implementation by a concrete runtime",
            _name
        ))))
    }

    /// Reads from WebAssembly memory
    pub fn read_memory(&self, _offset: u32, _size: u32, _buffer: &mut [u8]) -> Result<()> {
        // This would be implemented by the concrete runtime implementation
        Err(Error::new(kinds::NotImplementedError(
            "Memory reading requires implementation by a concrete runtime".to_string(),
        )))
    }

    /// Writes to WebAssembly memory
    pub fn write_memory(&self, _offset: u32, _data: &[u8]) -> Result<()> {
        // This would be implemented by the concrete runtime implementation
        Err(Error::new(kinds::NotImplementedError(
            "Memory writing requires implementation by a concrete runtime".to_string(),
        )))
    }
}

/// Helper function to convert FormatValType to ValueType
fn convert_to_valuetype(val_type_pair: &(String, FormatValType)) -> ValueType {
    format_val_type_to_value_type(&val_type_pair.1).expect("Failed to convert format value type")
}

/// Represents an external value
#[derive(Debug, Clone)]
pub enum ExternValue {
    /// Function value
    Function(FunctionValue),
    /// Table value
    Table(TableValue),
    /// Memory value
    Memory(MemoryValue),
    /// Global value
    Global(GlobalValue),
    /// Trap value
    Trap(String),
}

/// Represents a function value
#[derive(Debug, Clone)]
pub struct FunctionValue {
    /// Function type
    pub ty: wrt_runtime::func::FuncType,
    /// Export name that this function refers to
    pub export_name: String,
}

/// Represents a table value
#[derive(Debug, Clone)]
pub struct TableValue {
    /// Table type
    pub ty: TableType,
    /// Table instance
    pub table: Table,
}

/// Represents a memory value
#[derive(Debug, Clone)]
pub struct MemoryValue {
    /// Memory type
    pub ty: MemoryType,
    /// Memory instance
    pub memory: Arc<RwLock<Memory>>,
}

impl MemoryValue {
    /// Creates a new memory value
    ///
    /// # Arguments
    ///
    /// * `ty` - The memory type
    ///
    /// # Returns
    ///
    /// A new memory value
    ///
    /// # Errors
    ///
    /// Returns an error if the memory cannot be created
    pub fn new(ty: MemoryType) -> Result<Self> {
        let memory = Memory::new(ty.clone())?;
        Ok(Self {
            ty,
            memory: Arc::new(RwLock::new(memory)),
        })
    }

    /// Creates a new memory value with a debug name
    ///
    /// # Arguments
    ///
    /// * `ty` - The memory type
    /// * `name` - The debug name for the memory
    ///
    /// # Returns
    ///
    /// A new memory value
    ///
    /// # Errors
    ///
    /// Returns an error if the memory cannot be created
    pub fn new_with_name(ty: MemoryType, name: &str) -> Result<Self> {
        let memory = Memory::new_with_name(ty.clone(), name)?;
        Ok(Self {
            ty,
            memory: Arc::new(RwLock::new(memory)),
        })
    }

    /// Reads from memory
    ///
    /// # Arguments
    ///
    /// * `offset` - The offset to read from
    /// * `size` - The number of bytes to read
    ///
    /// # Returns
    ///
    /// The bytes read from memory
    ///
    /// # Errors
    ///
    /// Returns an error if the read fails
    pub fn read(&self, offset: u32, size: u32) -> Result<Vec<u8>> {
        let memory = self.memory.read().map_err(|e| {
            Error::new(kinds::MemoryAccessError(format!(
                "Failed to acquire memory read lock: {}",
                e
            )))
        })?;

        let mut buffer = vec![0; size as usize];
        memory.read(offset, &mut buffer).map_err(|e| {
            Error::new(kinds::MemoryAccessError(format!(
                "Memory read error: {}",
                e
            )))
        })?;

        Ok(buffer)
    }

    /// Writes to memory
    ///
    /// # Arguments
    ///
    /// * `offset` - The offset to write to
    /// * `bytes` - The bytes to write
    ///
    /// # Returns
    ///
    /// Ok(()) if the write succeeds
    ///
    /// # Errors
    ///
    /// Returns an error if the write fails
    pub fn write(&self, offset: u32, bytes: &[u8]) -> Result<()> {
        let mut memory = self.memory.write().map_err(|e| {
            Error::new(kinds::MemoryAccessError(format!(
                "Failed to acquire memory write lock: {}",
                e
            )))
        })?;

        memory.write(offset, bytes).map_err(|e| {
            Error::new(kinds::MemoryAccessError(format!(
                "Memory write error: {}",
                e
            )))
        })
    }

    /// Grows the memory by the given number of pages
    ///
    /// # Arguments
    ///
    /// * `pages` - The number of pages to grow by
    ///
    /// # Returns
    ///
    /// The previous size in pages
    ///
    /// # Errors
    ///
    /// Returns an error if the memory cannot be grown
    pub fn grow(&self, pages: u32) -> Result<u32> {
        let mut memory = self.memory.write().map_err(|e| {
            Error::new(kinds::MemoryAccessError(format!(
                "Failed to acquire memory write lock: {}",
                e
            )))
        })?;

        memory.grow(pages).map_err(|e| {
            Error::new(kinds::MemoryAccessError(format!(
                "Memory grow error: {}",
                e
            )))
        })
    }

    /// Gets the current size of the memory in pages
    ///
    /// # Returns
    ///
    /// The current size in pages
    pub fn size(&self) -> Result<u32> {
        let memory = self.memory.read().map_err(|e| {
            Error::new(kinds::MemoryAccessError(format!(
                "Failed to acquire memory read lock: {}",
                e
            )))
        })?;

        Ok(memory.size())
    }

    /// Gets the current size of the memory in bytes
    ///
    /// # Returns
    ///
    /// The current size in bytes
    pub fn size_in_bytes(&self) -> Result<usize> {
        let memory = self.memory.read().map_err(|e| {
            Error::new(kinds::MemoryAccessError(format!(
                "Failed to acquire memory read lock: {}",
                e
            )))
        })?;

        Ok(memory.size_in_bytes())
    }

    /// Gets the peak memory usage in bytes
    ///
    /// # Returns
    ///
    /// The peak memory usage in bytes
    pub fn peak_usage(&self) -> Result<usize> {
        let memory = self.memory.read().map_err(|e| {
            Error::new(kinds::MemoryAccessError(format!(
                "Failed to acquire memory read lock: {}",
                e
            )))
        })?;

        Ok(memory.peak_usage())
    }

    /// Gets the number of memory accesses performed
    ///
    /// # Returns
    ///
    /// The number of memory accesses
    pub fn access_count(&self) -> Result<u64> {
        let memory = self.memory.read().map_err(|e| {
            Error::new(kinds::MemoryAccessError(format!(
                "Failed to acquire memory read lock: {}",
                e
            )))
        })?;

        Ok(memory.access_count())
    }

    /// Gets a debug name for this memory, if any
    ///
    /// # Returns
    ///
    /// The debug name, if any
    pub fn debug_name(&self) -> Result<Option<String>> {
        let memory = self.memory.read().map_err(|e| {
            Error::new(kinds::MemoryAccessError(format!(
                "Failed to acquire memory read lock: {}",
                e
            )))
        })?;

        Ok(memory.debug_name().map(String::from))
    }

    /// Sets a debug name for this memory
    ///
    /// # Arguments
    ///
    /// * `name` - The debug name to set
    pub fn set_debug_name(&self, name: &str) -> Result<()> {
        let mut memory = self.memory.write().map_err(|e| {
            Error::new(kinds::MemoryAccessError(format!(
                "Failed to acquire memory write lock: {}",
                e
            )))
        })?;

        memory.set_debug_name(name);
        Ok(())
    }

    /// Verifies the integrity of the memory
    ///
    /// # Returns
    ///
    /// Ok(()) if the memory is valid
    ///
    /// # Errors
    ///
    /// Returns an error if the memory is invalid
    pub fn verify_integrity(&self) -> Result<()> {
        let memory = self.memory.read().map_err(|e| {
            Error::new(kinds::MemoryAccessError(format!(
                "Failed to acquire memory read lock: {}",
                e
            )))
        })?;

        memory.verify_integrity()
    }
}

/// Represents a global value
#[derive(Debug, Clone)]
pub struct GlobalValue {
    /// Global type
    pub ty: GlobalType,
    /// Global instance
    pub global: Global,
}

/// Represents a host with callable functions
pub struct Host {
    /// Host functions indexed by name
    functions: HashMap<String, FunctionValue>,
}

impl Default for Host {
    fn default() -> Self {
        Self::new()
    }
}

impl Host {
    /// Creates a new empty host
    #[must_use]
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    /// Adds a host function
    pub fn add_function(&mut self, name: String, func: FunctionValue) {
        self.functions.insert(name, func);
    }

    /// Gets a host function by name
    #[must_use]
    pub fn get_function(&self, name: &str) -> Option<&FunctionValue> {
        self.functions.get(name)
    }

    /// Calls a host function
    pub fn call_function(&self, name: &str, args: &[Value]) -> Result<Vec<Value>> {
        // Find the function
        let function = self.get_function(name).ok_or_else(|| {
            Error::new(kinds::FunctionNotFoundError(format!(
                "Host function {name} not found"
            )))
        })?;

        // Validate arguments
        if args.len() != function.ty.params.len() {
            return Err(Error::new(kinds::ValidationError(format!(
                "Expected {} arguments, got {}",
                function.ty.params.len(),
                args.len()
            ))));
        }

        // Actual function calling would happen here
        // This requires integration with the actual host function mechanism
        Err(Error::new(kinds::NotImplementedError(
            "Host function calling requires implementation by a concrete host".to_string(),
        )))
    }
}

impl Component {
    /// Creates a new component
    ///
    /// # Arguments
    ///
    /// * `component_type` - The component type
    ///
    /// # Returns
    ///
    /// A new component
    pub fn new(component_type: WrtComponentType) -> Self {
        Self {
            component_type,
            exports: Vec::new(),
            imports: Vec::new(),
        }
    }
}

/// Container for built-in type requirements
#[derive(Debug, Clone, Default)]
pub struct BuiltinRequirements {
    /// The set of required built-in types
    requirements: HashSet<BuiltinType>,
}

impl BuiltinRequirements {
    /// Create a new, empty set of built-in requirements
    pub fn new() -> Self {
        Self {
            requirements: HashSet::new(),
        }
    }

    /// Add a requirement for a specific built-in type
    pub fn add_requirement(&mut self, builtin_type: BuiltinType) {
        self.requirements.insert(builtin_type);
    }

    /// Get the set of required built-ins
    pub fn get_requirements(&self) -> &HashSet<BuiltinType> {
        &self.requirements
    }

    /// Check if all the requirements can be satisfied with the current runtime
    pub fn is_supported(&self) -> bool {
        // For now, all built-ins are supported
        // In the future, this could check if experimental features are enabled
        true
    }

    /// Get a list of unsupported built-ins
    pub fn get_unsupported(&self) -> Vec<BuiltinType> {
        // For now, all built-ins are supported
        vec![]
    }

    /// Check if the requirements can be satisfied with the given available built-ins
    pub fn can_be_satisfied(&self, available: &HashSet<BuiltinType>) -> bool {
        self.requirements.iter().all(|req| available.contains(req))
    }
}

/// Scan a component binary for built-in usage
///
/// # Arguments
///
/// * `bytes` - The component binary
///
/// # Returns
///
/// A result containing the set of built-in requirements or an error
pub fn scan_builtins(bytes: &[u8]) -> Result<BuiltinRequirements> {
    let mut requirements = BuiltinRequirements::new();

    // Extract the component
    let component = match ComponentDecoder::new().parse(bytes) {
        Ok(component) => component,
        Err(err) => {
            return Err(Error::new(kinds::DecodingError(format!(
                "Failed to decode component during built-in scan: {}",
                err
            ))));
        }
    };

    // Scan the component functions for built-in usage
    scan_functions_for_builtins(&component, &mut requirements)?;

    // Extract and scan any embedded modules
    let modules = extract_embedded_modules(bytes)?;
    for module in modules {
        scan_module_for_builtins(&module, &mut requirements)?;
    }

    Ok(requirements)
}

/// Scan a WebAssembly module for built-in usage
///
/// # Arguments
///
/// * `module` - The WebAssembly module
/// * `requirements` - The requirements to be updated
///
/// # Returns
///
/// A result indicating success or an error
fn scan_module_for_builtins(module: &[u8], requirements: &mut BuiltinRequirements) -> Result<()> {
    // Use our parser module to get the required builtins
    match crate::parser::get_required_builtins(module) {
        Ok(builtins) => {
            // Add all found builtins to the requirements
            for builtin_type in builtins {
                requirements.add_requirement(builtin_type);
            }
            Ok(())
        }
        Err(err) => Err(Error::new(kinds::DecodingError(format!(
            "Failed to scan module for builtins: {}",
            err
        )))),
    }
}

/// Map an import name to a built-in type
fn map_import_to_builtin(import_name: &str) -> Option<BuiltinType> {
    // This function is now defined in the parser module, but we keep it here
    // for backward compatibility
    crate::parser::map_import_to_builtin(import_name)
}

/// Scan component functions for built-in usage
///
/// # Arguments
///
/// * `component` - The component to scan
/// * `requirements` - The requirements to be updated
///
/// # Returns
///
/// A result indicating success or an error
fn scan_functions_for_builtins(
    component: &DecodedComponent,
    requirements: &mut BuiltinRequirements,
) -> Result<()> {
    // Check for resource types which indicate built-in usage
    for type_def in component.types() {
        if let TypeDef::Resource(_) = type_def {
            // Resources typically require the ResourceCreate built-in
            requirements.add_requirement(BuiltinType::ResourceCreate);
            requirements.add_requirement(BuiltinType::ResourceDrop);
            requirements.add_requirement(BuiltinType::ResourceRep);
            requirements.add_requirement(BuiltinType::ResourceGet);
        }
    }

    #[cfg(feature = "component-model-async")]
    // Check for async functions which require the AsyncWait built-in
    for func in component.functions() {
        if func.is_async() {
            requirements.add_requirement(BuiltinType::AsyncWait);
            requirements.add_requirement(BuiltinType::AsyncNew);
            requirements.add_requirement(BuiltinType::AsyncGet);
            requirements.add_requirement(BuiltinType::AsyncPoll);
            break; // Only need to identify one async function
        }
    }

    Ok(())
}

/// Extract embedded modules from a component binary
///
/// # Arguments
///
/// * `bytes` - The component binary
///
/// # Returns
///
/// A result containing a vector of embedded modules or an error
fn extract_embedded_modules(bytes: &[u8]) -> Result<Vec<Vec<u8>>> {
    let mut modules = Vec::new();

    // This is a simplified implementation - in practice, we would need to fully parse
    // the component and extract all embedded modules
    // For now, we'll just look for the first module

    // Parse the component
    let component = match ComponentDecoder::new().parse(bytes) {
        Ok(component) => component,
        Err(err) => {
            return Err(Error::new(kinds::DecodingError(format!(
                "Failed to decode component while extracting modules: {}",
                err
            ))));
        }
    };

    // Check if there's an embedded module and extract it
    // This is a placeholder - the actual implementation would need to extract all modules
    // from the component binary
    if let Some(first_module) = component.module_section().first() {
        modules.push(first_module.content().to_vec());
    }

    Ok(modules)
}

/// Convert a component value to a runtime value
pub fn component_value_to_value(
    component_value: &wrt_types::ComponentValue,
) -> wrt_intercept::Value {
    use crate::type_conversion::types_componentvalue_to_core_value;
    use wrt_intercept::Value;

    // Use the new conversion function
    types_componentvalue_to_core_value(component_value).unwrap_or(Value::I32(0))
    // Provide a sensible default on error
}

/// Convert a runtime value to a component value
pub fn value_to_component_value(value: &wrt_intercept::Value) -> wrt_types::ComponentValue {
    use crate::type_conversion::core_value_to_types_componentvalue;
    use wrt_types::ComponentValue;

    // Use the new conversion function
    core_value_to_types_componentvalue(value).unwrap_or(ComponentValue::Void) // Provide a sensible default on error
}

/// Convert parameter to value type
pub fn convert_param_to_value_type(
    param: &wrt_format::component::ValType,
) -> wrt_types::types::ValueType {
    crate::type_conversion::format_val_type_to_value_type(param)
        .unwrap_or(wrt_types::types::ValueType::I32)
}

/// Convert verification level
pub fn convert_verification_level(
    level: wrt_types::VerificationLevel,
) -> crate::resources::VerificationLevel {
    match level {
        wrt_types::VerificationLevel::None => crate::resources::VerificationLevel::None,
        wrt_types::VerificationLevel::Sampling => crate::resources::VerificationLevel::Critical,
        wrt_types::VerificationLevel::Standard => crate::resources::VerificationLevel::Critical,
        wrt_types::VerificationLevel::Full => crate::resources::VerificationLevel::Full,
    }
}
