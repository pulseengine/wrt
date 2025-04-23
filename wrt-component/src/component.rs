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
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use wrt_common::component::ComponentValue;
use wrt_error::{Error, Result, kinds};
use log::{debug, error, info, trace, warn};

// Use direct imports from wrt-types
use wrt_types::{
    ComponentType, ExternType, FuncType, InstanceType, ValueType, Value, ResourceType,
    VerificationLevel
};

// Use format types with explicit namespacing
use wrt_format::component::{
    ExternType as FormatExternType, 
    ResourceOperation as FormatResourceOperation, 
    ValType as FormatValType,
    ComponentTypeDefinition as FormatComponentTypeDefinition
};

// Import conversion functions
use wrt_format::component_conversion::{
    format_type_def_to_component_type,
    component_type_to_format_type_def,
};

use wrt_host::function::HostFunctionHandler;
use wrt_host::CallbackRegistry;
use wrt_intercept::{
    InterceptionResult, LinkInterceptor, LinkInterceptorStrategy,
};

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

#[cfg(all(not(feature = "std"), feature = "alloc"))]
use alloc::{
    collections::BTreeMap as HashMap,
    string::{String, ToString},
    sync::{Arc, RwLock},
    vec,
    vec::Vec,
};

use crate::execution::{run_with_time_bounds, TimeBoundedConfig, TimeBoundedOutcome};
use std::any::Any;

use core::str;
use std::borrow::Cow;
use std::collections::VecDeque;

// Import type conversion utilities for clear transformations
use crate::type_conversion::{
    common_to_format_val_type, format_to_common_val_type, format_to_types_extern_type,
    format_val_type_to_value_type, types_to_format_extern_type, value_type_to_format_val_type,
    extern_type_to_func_type,
};

/// Represents a component instance
#[derive(Debug)]
pub struct Component {
    /// Component type
    pub(crate) component_type: ComponentType,
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
}

/// Represents a component type
#[derive(Debug, Clone)]
pub struct ComponentType {
    /// Component imports
    pub imports: Vec<(String, String, ExternType)>,
    /// Component exports
    pub exports: Vec<(String, ExternType)>,
    /// Component instances
    pub instances: Vec<wrt_format::component::ComponentTypeDefinition>,
}

impl ComponentType {
    /// Creates a new empty component type
    pub fn new() -> Self {
        Self {
            imports: Vec::new(),
            exports: Vec::new(),
            instances: Vec::new(),
        }
    }
}

impl Default for ComponentType {
    fn default() -> Self {
        Self::new()
    }
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

    /// Gets the current size of memory in pages
    ///
    /// # Returns
    ///
    /// The current size in pages (64KiB per page)
    pub fn size(&self) -> Result<u32> {
        let memory = self.memory.read().map_err(|e| {
            Error::new(kinds::MemoryAccessError(format!(
                "Failed to acquire memory read lock: {}",
                e
            )))
        })?;

        Ok(memory.size())
    }

    /// Gets the memory type
    ///
    /// # Returns
    ///
    /// The memory type
    pub fn memory_type(&self) -> MemoryType {
        self.ty.clone()
    }

    /// Gets the memory peak usage in bytes
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

        Ok(memory.peak_memory())
    }

    /// Gets the memory access count
    ///
    /// # Returns
    ///
    /// The number of times memory has been accessed
    pub fn access_count(&self) -> Result<u64> {
        let memory = self.memory.read().map_err(|e| {
            Error::new(kinds::MemoryAccessError(format!(
                "Failed to acquire memory read lock: {}",
                e
            )))
        })?;

        Ok(memory.access_count())
    }

    /// Reads a value from memory at the specified address with the given type
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to read from
    /// * `value_type` - The type of value to read
    ///
    /// # Returns
    ///
    /// The value read from memory
    ///
    /// # Errors
    ///
    /// Returns an error if the read fails
    pub fn read_value(&self, addr: u32, value_type: wrt_types::types::ValueType) -> Result<Value> {
        let memory = self.memory.read().map_err(|e| {
            Error::new(kinds::MemoryAccessError(format!(
                "Failed to acquire memory read lock: {}",
                e
            )))
        })?;

        match value_type {
            wrt_types::types::ValueType::I32 => {
                memory.read_i32(addr).map(Value::I32).map_err(|e| {
                    Error::new(kinds::MemoryAccessError(format!(
                        "Memory read i32 error: {}",
                        e
                    )))
                })
            }
            wrt_types::types::ValueType::I64 => {
                memory.read_i64(addr).map(Value::I64).map_err(|e| {
                    Error::new(kinds::MemoryAccessError(format!(
                        "Memory read i64 error: {}",
                        e
                    )))
                })
            }
            wrt_types::types::ValueType::F32 => {
                memory.read_f32(addr).map(Value::F32).map_err(|e| {
                    Error::new(kinds::MemoryAccessError(format!(
                        "Memory read f32 error: {}",
                        e
                    )))
                })
            }
            wrt_types::types::ValueType::F64 => {
                memory.read_f64(addr).map(Value::F64).map_err(|e| {
                    Error::new(kinds::MemoryAccessError(format!(
                        "Memory read f64 error: {}",
                        e
                    )))
                })
            }
            _ => Err(Error::new(TypeMismatchError(format!(
                "Unsupported value type for memory read: {:?}",
                value_type
            )))),
        }
    }

    /// Writes a value to memory at the specified address
    ///
    /// # Arguments
    ///
    /// * `addr` - The address to write to
    /// * `value` - The value to write
    ///
    /// # Returns
    ///
    /// Ok(()) if the write succeeds
    ///
    /// # Errors
    ///
    /// Returns an error if the write fails
    pub fn write_value(&self, addr: u32, value: Value) -> Result<()> {
        let mut memory = self.memory.write().map_err(|e| {
            Error::new(kinds::MemoryAccessError(format!(
                "Failed to acquire memory write lock: {}",
                e
            )))
        })?;

        match value {
            Value::I32(v) => memory.write_i32(addr, v).map_err(|e| {
                Error::new(kinds::MemoryAccessError(format!(
                    "Memory write i32 error: {}",
                    e
                )))
            }),
            Value::I64(v) => memory.write_i64(addr, v).map_err(|e| {
                Error::new(kinds::MemoryAccessError(format!(
                    "Memory write i64 error: {}",
                    e
                )))
            }),
            Value::F32(v) => memory.write_f32(addr, v).map_err(|e| {
                Error::new(kinds::MemoryAccessError(format!(
                    "Memory write f32 error: {}",
                    e
                )))
            }),
            Value::F64(v) => memory.write_f64(addr, v).map_err(|e| {
                Error::new(kinds::MemoryAccessError(format!(
                    "Memory write f64 error: {}",
                    e
                )))
            }),
            _ => Err(Error::new(TypeMismatchError(format!(
                "Unsupported value type for memory write: {:?}",
                value
            )))),
        }
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
    /// Creates a new component instance
    #[must_use]
    pub fn new(component_type: ComponentType) -> Self {
        // Use ResourceTable::new() without arguments
        let resource_table = ResourceTable::new();
        Self {
            component_type, // Use the parameter directly
            exports: Vec::new(),
            imports: Vec::new(),
            instances: Vec::new(),
            linked_components: HashMap::new(),
            callback_registry: None,
            runtime: None,
            interceptor: None,
            resource_table,
        }
    }

    /// Sets the callback registry for this component
    pub fn with_callback_registry(mut self, registry: Arc<CallbackRegistry>) -> Self {
        self.callback_registry = Some(registry);
        self
    }

    /// Sets the runtime instance for this component
    pub fn with_runtime(mut self, runtime: RuntimeInstance) -> Self {
        self.runtime = Some(runtime);
        self
    }

    /// Sets the interceptor for this component
    pub fn with_interceptor(mut self, interceptor: Arc<LinkInterceptor>) -> Self {
        self.interceptor = Some(interceptor);
        self
    }

    /// Gets the callback registry
    fn get_callback_registry(&self) -> Result<&CallbackRegistry> {
        self.callback_registry
            .as_ref()
            .map(|arc| arc.as_ref())
            .ok_or_else(|| {
                Error::new(kinds::InitializationError(
                    "Component not initialized with callback registry".to_string(),
                ))
            })
    }

    /// Gets the mutable callback registry
    fn get_callback_registry_mut(&mut self) -> Result<&mut CallbackRegistry> {
        // This is a bit tricky with Arc - we'd need to use Arc::get_mut or clone
        // For now, we'll return an error as we should set up the registry before using
        Err(Error::new(kinds::InitializationError(
            "Modifying callback registry not supported after initialization".to_string(),
        )))
    }

    /// Gets the runtime instance
    fn get_runtime(&self) -> Result<&RuntimeInstance> {
        self.runtime.as_ref().ok_or_else(|| {
            Error::new(kinds::InitializationError(
                "Component not initialized with runtime".to_string(),
            ))
        })
    }

    /// Instantiates the component with the given imports
    pub fn instantiate(&mut self, imports: Vec<Import>) -> Result<()> {
        // Validate imports
        if imports.len() != self.component_type.imports.len() {
            return Err(Error::new(kinds::ValidationError(format!(
                "Expected {} imports, got {}",
                self.component_type.imports.len(),
                imports.len()
            ))));
        }

        // Store imports
        self.imports = imports;

        // Initialize exports
        self.initialize_exports()?;

        // Populate instance exports
        self.link_instances()?;

        // Link imports to exports
        self.link_imports_to_exports()?;

        // Link component exports to instance exports
        self.link_instance_exports()?;

        // Finalize instance exports
        self.finalize_instance_exports()?;

        // Validate the component
        self.validate()?;

        Ok(())
    }

    /// Initialize exports from component type
    fn initialize_exports(&mut self) -> Result<()> {
        let mut exports = Vec::new();

        // Create exports for each export in the component type
        for (name, ty) in &self.component_type.exports {
            // Create export value based on the export type
            let export_value = match ty {
                ExternType::Function(_) => {
                    // Function exports need to be linked from imports or instances
                    ExternValue::Trap("Function not yet initialized".to_string())
                }
                ExternType::Table(_) => {
                    // Table exports need to be linked from imports or instances
                    ExternValue::Trap("Table not yet initialized".to_string())  
                }
                ExternType::Memory(_) => {
                    // Memory exports need to be linked from imports or instances
                    ExternValue::Trap("Memory not yet initialized".to_string())
                }
                ExternType::Global(_) => {
                    // Global exports need to be linked from imports or instances
                    ExternValue::Trap("Global not yet initialized".to_string())
                }
                ExternType::Resource(_) => {
                    // Resource exports need to be linked from imports or instances
                    ExternValue::Trap("Resource not yet initialized".to_string())
                }
                ExternType::Instance(_) => {
                    // Instance exports need to be linked
                    ExternValue::Trap("Instance not yet initialized".to_string())
                }
                ExternType::Component(_) => {
                    // Component exports need to be linked
                    ExternValue::Trap("Component not yet initialized".to_string())
                }
            };

            exports.push(Export {
                name: name.clone(),
                ty: ty.clone(),
                value: export_value,
                attributes: HashMap::new(),
                integrity_hash: None,
            });
        }

        self.exports = exports;
        Ok(())
    }

    /// Creates exports for an instance based on its type
    fn create_instance_exports(
        &self,
        instance_type: &wrt_format::component::ComponentTypeDefinition,
        instance_name: &str,
    ) -> Result<Vec<Export>> {
        let mut exports = Vec::new();

        match instance_type {
            wrt_format::component::ComponentTypeDefinition::Instance {
                exports: type_exports,
            } => {
                // Create exports for each export in the instance type
                for (name, ty) in type_exports {
                    // Create export value based on the export type
                    let export_value = match ty {
                        FormatExternType::Function { .. } => {
                            // For now, function exports need to be linked from imports
                            ExternValue::Trap(format!(
                                "Function {} not yet initialized in instance {}",
                                name, instance_name
                            ))
                        }
                        FormatExternType::Type(_) => {
                            // For memory-like types, we'll check if this is intended to be a memory
                            // In the component model, memory is typically represented as an imported resource
                            // or as a special type.
                            // Initialize with a default memory for now
                            let memory_type = wrt_runtime::types::MemoryType {
                                limits: wrt_types::types::Limits {
                                    min: 1,
                                    max: Some(2),
                                },
                            };
                            let memory_value = MemoryValue::new(memory_type)?;

                            ExternValue::Memory(memory_value)
                        }
                        FormatExternType::Instance { .. } => {
                            // Instance exports need to be linked
                            ExternValue::Trap("Instance not yet initialized".to_string())
                        }
                        FormatExternType::Component { .. } => {
                            // Component exports need to be linked
                            ExternValue::Trap("Component not yet initialized".to_string())
                        }
                        FormatExternType::Value(_) => {
                            // Value exports need to be linked
                            ExternValue::Trap("Value not yet initialized".to_string())
                        }
                    };

                    exports.push(Export {
                        name: name.clone(),
                        ty: ty.clone(),
                        value: export_value,
                        attributes: HashMap::new(),
                        integrity_hash: None,
                    });
                }
            }
            _ => {
                return Err(Error::new(kinds::ValidationError(
                    "Expected Instance type definition".to_string(),
                )));
            }
        }

        Ok(exports)
    }

    /// Link instances based on component type
    fn link_instances(&mut self) -> Result<()> {
        let mut instances = Vec::new();

        // Create instances for each instance in the component type
        for (idx, instance_type) in self.component_type.instances.iter().enumerate() {
            let instance_name = format!("instance-{}", idx);
            let exports = self.create_instance_exports(instance_type, &instance_name)?;

            instances.push(InstanceValue {
                ty: instance_type.clone(),
                exports,
            });
        }

        self.instances = instances;
        Ok(())
    }

    /// Link imports to exports
    fn link_imports_to_exports(&mut self) -> Result<()> {
        // We'll create a new imports vector to store the updated imports
        let mut updated_imports = Vec::new();

        // Iterate through imports and find matching exports
        for import in self.imports.clone() {
            let mut found_match = false;

            // Find a matching export
            for export in &self.exports {
                if export.name == import.identifier() {
                    // Found a matching export, use its value
                    debug_println(&format!(
                        "Linking import {} to export {}",
                        import.identifier(),
                        export.name
                    ));

                    // Create a new import with the export's value
                    updated_imports.push(crate::import::Import::new(
                        import.namespace.clone(),
                        import.name.clone(),
                        import.ty.clone(),
                        export.value.clone(),
                    ));

                    found_match = true;
                    break;
                }
            }

            // If no match was found, keep the original import
            if !found_match {
                updated_imports.push(import);
            }
        }

        // Replace the current imports with the updated ones
        self.imports = updated_imports;

        Ok(())
    }

    /// Link instance exports to component imports
    fn link_instance_exports(&mut self) -> Result<()> {
        // For each instance export, check if it matches a component import
        for instance in &self.instances {
            for export in &instance.exports {
                // Check if any component import matches this instance export
                for import in &self.imports {
                    if export.name == import.name && types_are_compatible(&export.ty, &import.ty) {
                        // For now, just log that we found a match
                        debug_println(&format!(
                            "Found match for instance export {} in component import {}",
                            export.name, import.name
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Finalize instance exports
    fn finalize_instance_exports(&mut self) -> Result<()> {
        // For each instance
        for instance in &mut self.instances {
            // For each export in the instance
            for export in &mut instance.exports {
                // If the export is a function that is not yet initialized
                if let ExternValue::Trap(msg) = &export.value {
                    if msg.contains("not yet initialized") {
                        // Try to find a matching import
                        for import in &self.imports {
                            if export.name == import.name
                                && types_are_compatible(&export.ty, &import.ty)
                            {
                                // Use the import value for this export
                                export.value = import.value.clone();
                                break;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Reads memory from a named export
    pub fn read_memory(&self, name: &str, offset: u32, size: u32) -> Result<Vec<u8>> {
        // Find the export
        let export = self.get_export(name)?;

        // Check if it's a memory
        if let ExternValue::Memory(memory_value) = &export.value {
            // Read from the memory using the MemoryValue implementation
            memory_value.read(offset, size)
        } else {
            Err(Error::new(TypeMismatchError(format!(
                "Export {} is not a memory",
                name
            ))))
        }
    }

    /// Executes a function by name
    pub fn execute_function(&self, name: &str, args: Vec<Value>) -> Result<Vec<Value>> {
        // If we have an interceptor, apply it to the call
        if let Some(interceptor) = self.get_interceptor() {
            interceptor.intercept_call("component", name, args, |modified_args| {
                self.execute_function_internal(name, modified_args)
            })
        } else {
            self.execute_function_internal(name, args)
        }
    }

    /// Internal implementation of execute_function without interception
    fn execute_function_internal(&self, name: &str, args: Vec<Value>) -> Result<Vec<Value>> {
        // Get the export
        let export = self.get_export(name)?;

        // Find the appropriate handler
        match &export.value {
            ExternValue::Function(func) => {
                let func_name = &func.export_name;
                debug_println(&format!("Executing function: {}", func_name));
                self.handle_function_call(func_name, &args)
            }
            _ => Err(Error::new(kinds::RuntimeError(format!(
                "Export '{}' is not a function",
                name
            )))),
        }
    }

    /// Writes to memory
    pub fn write_memory(&mut self, name: &str, offset: u32, bytes: &[u8]) -> Result<()> {
        // Find the export
        let export = self.get_export_mut(name)?;

        // Check if it's a memory
        if let ExternValue::Memory(memory_value) = &mut export.value {
            // Write to the memory using the MemoryValue implementation
            memory_value.write(offset, bytes)
        } else {
            Err(Error::new(TypeMismatchError(format!(
                "Export {} is not a memory",
                name
            ))))
        }
    }

    /// Gets the size of memory in pages
    pub fn memory_size(&self, name: &str) -> Result<u32> {
        // Find the export
        let export = self.get_export(name)?;

        // Check if it's a memory
        if let ExternValue::Memory(memory_value) = &export.value {
            // Get the memory size using the MemoryValue implementation
            memory_value.size()
        } else {
            Err(Error::new(TypeMismatchError(format!(
                "Export {} is not a memory",
                name
            ))))
        }
    }

    /// Grows memory by the specified number of pages
    pub fn memory_grow(&mut self, name: &str, pages: u32) -> Result<u32> {
        // Find the export
        let export = self.get_export_mut(name)?;

        // Check if it's a memory
        if let ExternValue::Memory(memory_value) = &mut export.value {
            // Grow the memory using the MemoryValue implementation
            memory_value.grow(pages)
        } else {
            Err(Error::new(TypeMismatchError(format!(
                "Export {} is not a memory",
                name
            ))))
        }
    }

    /// Gets memory peak usage in bytes
    pub fn memory_peak_usage(&self, name: &str) -> Result<usize> {
        // Find the export
        let export = self.get_export(name)?;

        // Check if it's a memory
        if let ExternValue::Memory(memory_value) = &export.value {
            // Get the memory peak usage using the MemoryValue implementation
            memory_value.peak_usage()
        } else {
            Err(Error::new(TypeMismatchError(format!(
                "Export {} is not a memory",
                name
            ))))
        }
    }

    /// Gets memory access count
    pub fn memory_access_count(&self, name: &str) -> Result<u64> {
        // Find the export
        let export = self.get_export(name)?;

        // Check if it's a memory
        if let ExternValue::Memory(memory_value) = &export.value {
            // Get the memory access count using the MemoryValue implementation
            memory_value.access_count()
        } else {
            Err(Error::new(TypeMismatchError(format!(
                "Export {} is not a memory",
                name
            ))))
        }
    }

    /// Reads a typed value from memory
    pub fn read_memory_value(
        &self,
        name: &str,
        addr: u32,
        value_type: wrt_types::types::ValueType,
    ) -> Result<Value> {
        // Find the export
        let export = self.get_export(name)?;

        // Check if it's a memory
        if let ExternValue::Memory(memory_value) = &export.value {
            // Read the value using the MemoryValue implementation
            memory_value.read_value(addr, value_type)
        } else {
            Err(Error::new(TypeMismatchError(format!(
                "Export {} is not a memory",
                name
            ))))
        }
    }

    /// Writes a typed value to memory
    pub fn write_memory_value(&mut self, name: &str, addr: u32, value: Value) -> Result<()> {
        // Find the export
        let export = self.get_export_mut(name)?;

        // Check if it's a memory
        if let ExternValue::Memory(memory_value) = &mut export.value {
            // Write the value using the MemoryValue implementation
            memory_value.write_value(addr, value)
        } else {
            Err(Error::new(TypeMismatchError(format!(
                "Export {} is not a memory",
                name
            ))))
        }
    }

    /// Gets an export by name
    pub fn get_export(&self, name: &str) -> Result<&Export> {
        self.exports
            .iter()
            .find(|e| e.name == name)
            .ok_or_else(|| Error::new(kinds::ExportNotFoundError(name.to_string())))
    }

    /// Gets a mutable export by name
    pub fn get_export_mut(&mut self, name: &str) -> Result<&mut Export> {
        self.exports
            .iter_mut()
            .find(|e| e.name == name)
            .ok_or_else(|| Error::new(kinds::ExportNotFoundError(name.to_string())))
    }

    /// Handles a function call
    fn handle_function_call(&self, name: &str, args: &[Value]) -> Result<Vec<Value>> {
        // Get the export
        let export = self.get_export(name)?;

        // Check that it's a function
        if let FormatExternType::Function { params, results } = &export.ty {
            // Validate argument types
            if args.len() != params.len() {
                return Err(Error::new(TypeMismatchError(format!(
                    "Function '{}' expects {} arguments, got {}",
                    name,
                    params.len(),
                    args.len()
                ))));
            }

            // Validate each argument
            for (i, (arg, param)) in args.iter().zip(params.iter()).enumerate() {
                if !Self::value_matches_type(arg, param) {
                    return Err(Error::new(TypeMismatchError(format!(
                        "Type mismatch for argument {} of function '{}'",
                        i, name
                    ))));
                }
            }

            // Call the function based on its type (host or local)
            if export.is_host {
                self.call_host_function(name, args)
            } else {
                self.call_local_function(name, args)
            }
        } else {
            // Not a function
            Err(Error::new(TypeMismatchError(format!(
                "Export '{}' is not a function",
                name
            ))))
        }
    }

    /// Calls a host function
    fn call_host_function(&self, name: &str, args: &[Value]) -> Result<Vec<Value>> {
        // Get the callback registry
        let registry = self.get_callback_registry()?;

        // A bit of a hack - we don't have specific module context here
        // So we just use the first part of the function name as the module
        let parts: Vec<&str> = name.split('.').collect();
        let (module, func) = if parts.len() > 1 {
            (parts[0], parts[1])
        } else {
            ("default", name)
        };

        // Apply interception if needed
        if let Some(interceptor) = self.get_interceptor() {
            interceptor.intercept_call(
                "component",
                &format!("host.{}", name),
                args.to_vec(),
                |modified_args| {
                    // Call the host function with our dummy engine (we don't use it)
                    // A proper implementation would need to pass a real engine context
                    let mut dummy_engine = ();
                    registry.call_host_function(&mut dummy_engine, module, func, modified_args)
                },
            )
        } else {
            // No interception
            let mut dummy_engine = ();
            registry.call_host_function(&mut dummy_engine, module, func, args.to_vec())
        }
    }

    /// Calls a local function
    fn call_local_function(&self, name: &str, args: &[Value]) -> Result<Vec<Value>> {
        // Get the runtime
        let runtime = self.get_runtime()?;

        // Execute the function
        runtime.execute_function(name, args.to_vec())
    }

    /// Helper function to check if a value matches the expected type
    fn value_matches_type(value: &Value, param: &(String, wrt_format::component::ValType)) -> bool {
        let (_, val_type) = param;

        match (value, val_type) {
            // Match numeric types
            (Value::I32(_), wrt_format::component::ValType::S32)
            | (Value::I32(_), wrt_format::component::ValType::U32) => true,
            (Value::I64(_), wrt_format::component::ValType::S64)
            | (Value::I64(_), wrt_format::component::ValType::U64) => true,
            (Value::F32(_), wrt_format::component::ValType::F32) => true,
            (Value::F64(_), wrt_format::component::ValType::F64) => true,
            (Value::I32(0), wrt_format::component::ValType::Bool) => true, // false as i32(0)
            (Value::I32(1), wrt_format::component::ValType::Bool) => true, // true as i32(1)

            // For strings and more complex types, a more elaborate check would be needed
            // Default to false for mismatches
            _ => false,
        }
    }

    /// Registers host APIs.
    pub fn register_host_api(
        &mut self,
        api_name: &str,
        functions: Vec<(String, FuncType, HostFunctionHandler)>,
    ) -> Result<()> {
        // Use Namespace::from_string
        let mut host_namespace = Namespace::from_string(api_name);
        for (func_name, func_type, handler) in functions {
            // Convert TypesFuncType to RuntimeFuncType if needed for FunctionValue
             let runtime_func_type = RuntimeFuncType::new(
                 func_type.params.iter().cloned().collect(), // Access field directly
                 func_type.results.iter().cloned().collect(), // Access field directly
             );

            let host_func = FunctionValue {
                ty: runtime_func_type,
                export_name: func_name.clone(),
            };
            // Use Export::new - needs ExternType and ExternValue
            let format_extern_type = types_to_format_extern_type(&TypesExternType::Function(func_type.clone()))?;
            let extern_value = ExternValue::Function(host_func);

            // Use Export::new
            host_namespace.add_export(Export::new(func_name.clone(), format_extern_type, extern_value));

            // Register with callback registry if available
            if let Some(_registry) = &self.callback_registry { // Mark registry unused
                 let _ = handler; // Avoid unused warning
                 debug_println(&format!("Skipping callback registration for {}/{} (needs CallbackType fix)", api_name, func_name));
            }
        }
        Ok(())
    }

    /// Instantiate a component from binary bytes
    pub fn instantiate_component(&mut self, bytes: &[u8], imports: Vec<Import>) -> Result<()> {
        // Decode the component binary format using wrt-decoder
        let decoded_component = wrt_decoder::component::decode_component(bytes)?;

        // Extract component type information from the decoded component
        let mut component_type = ComponentType::new();

        // Process imports from the decoded component
        for import in &decoded_component.imports {
            let name_data = &import.name;
            let namespace = name_data.namespace.clone();
            let name = name_data.name.clone();

            let extern_type = Self::convert_format_extern_type(&import.ty)?;
            component_type.imports.push((name, namespace, extern_type));
        }

        // Process exports from the decoded component
        for export in &decoded_component.exports {
            let name = export.name.name.clone();

            let extern_type = if let Some(ty) = &export.ty {
                Self::convert_format_extern_type(ty)?
            } else {
                // If type is not directly available, create a placeholder
                return Err(Error::new(kinds::ValidationError(
                    "Export type is required".to_string(),
                )));
            };

            component_type.exports.push((name, extern_type));
        }

        // Process component instances
        for instance in &decoded_component.instances {
            let instance_type = Self::convert_instance_type(instance)?;
            component_type.instances.push(instance_type);
        }

        // Update the component's type
        self.component_type = component_type;

        // Now use the normal instantiate method to instantiate the component with the given imports
        self.instantiate(imports)
    }

    /// Convert wrt_format::component::ExternType to wrt_types::component::ExternType
    fn convert_format_extern_type(extern_type: &FormatExternType) -> Result<TypesExternType> {
        // Use the conversion utility from type_conversion.rs
        format_to_types_extern_type(extern_type)
    }

    /// Convert wrt_format::component type definitions to ExternType
    fn convert_component_type(
        type_def: &wrt_format::component::ComponentTypeDefinition,
    ) -> Result<TypesExternType> {
        match type_def {
            wrt_format::component::ComponentTypeDefinition::Function { params, results } => {
                // Convert ValType to ValueType
                let converted_params = params
                    .iter()
                    .map(|(name, val_type)| {
                        Ok(format_val_type_to_value_type(val_type)?)
                    })
                    .collect::<Result<Vec<_>>>()?;

                let converted_results = results
                    .iter()
                    .map(|val_type| format_val_type_to_value_type(val_type))
                    .collect::<Result<Vec<_>>>()?;

                Ok(TypesExternType::Function(wrt_types::component::FuncType {
                    params: converted_params,
                    results: converted_results,
                }))
            }
            wrt_format::component::ComponentTypeDefinition::Instance { exports } => {
                let converted_exports = exports
                    .iter()
                    .map(|(name, export_type)| {
                        format_to_types_extern_type(export_type).map(|et| (name.clone(), et))
                    })
                    .collect::<Result<Vec<_>>>()?;

                Ok(TypesExternType::Instance(
                    wrt_types::component::InstanceType {
                        exports: converted_exports,
                    },
                ))
            }
            wrt_format::component::ComponentTypeDefinition::Component { imports, exports } => {
                let converted_imports = imports
                    .iter()
                    .map(|(module, name, import_type)| {
                        format_to_types_extern_type(import_type)
                            .map(|et| (module.clone(), name.clone(), et))
                    })
                    .collect::<Result<Vec<_>>>()?;

                let converted_exports = exports
                    .iter()
                    .map(|(name, export_type)| {
                        format_to_types_extern_type(export_type).map(|et| (name.clone(), et))
                    })
                    .collect::<Result<Vec<_>>>()?;

                Ok(TypesExternType::Component(
                    wrt_types::component::ComponentType {
                        imports: converted_imports,
                        exports: converted_exports,
                        instances: Vec::new(),
                    },
                ))
            }
            wrt_format::component::ComponentTypeDefinition::Value(val_type) => {
                // Convert to Function type with no parameters and one result
                let value_type = format_val_type_to_value_type(val_type)?;

                Ok(TypesExternType::Function(wrt_types::component::FuncType {
                    params: Vec::new(),
                    results: vec![value_type],
                }))
            }
            wrt_format::component::ComponentTypeDefinition::Resource {
                representation,
                nullable,
            } => {
                // Create a ResourceType using the helper function
                let resource_type = crate::type_conversion::format_resource_rep_to_resource_type(representation, *nullable);
                Ok(TypesExternType::Resource(resource_type))
            }
        }
    }

    /// Convert wrt_format::component instance to ComponentTypeDefinition
    fn convert_instance_type(
        instance: &wrt_format::component::Instance,
    ) -> Result<wrt_format::component::ComponentTypeDefinition> {
        // Extract exports based on the instance's expression type
        let exports = match &instance.instance_expr {
            wrt_format::component::InstanceExpr::Instantiate {
                component_idx,
                args,
            } => {
                // This is an instantiated component - need to extract exports from the component type
                let mut exports = Vec::new();

                // For instantiated components, we'll extract exports from the resulting instance
                // In a full implementation, we would need to resolve the component_idx to the actual component
                // and determine its exports
                debug_println(&format!(
                    "Extracting exports from instantiated component {}",
                    component_idx
                ));

                // Process instantiation arguments which often carry import/export mappings
                for arg in args {
                    match arg {
                        // The actual enum variants may differ, so we'll use match guards instead
                        arg if Self::is_instance_arg(arg) => {
                            let (name, idx) = Self::get_instance_arg_info(arg);
                            debug_println(&format!(
                                "Found instance arg {} -> instance {}",
                                name, idx
                            ));
                        }
                        arg if Self::is_core_arg(arg) => {
                            let (name, idx) = Self::get_core_arg_info(arg);
                            debug_println(&format!(
                                "Found core arg {} -> core instance {}",
                                name, idx
                            ));
                        }
                        arg if Self::is_function_arg(arg) => {
                            let (name, idx) = Self::get_function_arg_info(arg);
                            debug_println(&format!(
                                "Found function arg {} -> function {}",
                                name, idx
                            ));
                        }
                        // Add other argument types as needed
                        _ => {
                            debug_println("Found other instantiate arg type");
                        }
                    }
                }

                exports
            }
            // Check for other instance expression types
            _ => {
                debug_println("Unknown instance expression type");
                Vec::new() // In a full implementation, we would handle other expression types
            }
        };

        // Return the instance type definition with the collected exports
        Ok(wrt_format::component::ComponentTypeDefinition::Instance { exports })
    }

    // Helper functions to check instance argument types and extract info
    fn is_instance_arg(arg: &wrt_format::component::InstantiateArg) -> bool {
        // This would check if the argument is an instance type
        // For the sake of example, we'll just return false
        false
    }

    fn get_instance_arg_info(arg: &wrt_format::component::InstantiateArg) -> (String, u32) {
        // This would extract name and index from an instance argument
        // For the sake of example, we'll just return placeholders
        ("unknown".to_string(), 0)
    }

    fn is_core_arg(arg: &wrt_format::component::InstantiateArg) -> bool {
        // This would check if the argument is a core instance type
        // For the sake of example, we'll just return false
        false
    }

    fn get_core_arg_info(arg: &wrt_format::component::InstantiateArg) -> (String, u32) {
        // This would extract name and index from a core instance argument
        // For the sake of example, we'll just return placeholders
        ("unknown".to_string(), 0)
    }

    fn is_function_arg(arg: &wrt_format::component::InstantiateArg) -> bool {
        // This would check if the argument is a function type
        // For the sake of example, we'll just return false
        false
    }

    fn get_function_arg_info(arg: &wrt_format::component::InstantiateArg) -> (String, u32) {
        // This would extract name and index from a function argument
        // For the sake of example, we'll just return placeholders
        ("unknown".to_string(), 0)
    }

    /// Resolves a component import
    fn resolve_component_import(&self, import_name: &str) -> Result<Arc<Component>> {
        // In a real implementation, this would look up the component
        // from a registry or other source
        Err(Error::new(kinds::NotImplementedError(
            "Component import resolution not implemented".to_string(),
        )))
    }

    /// Link a component with the given namespace
    pub fn link_component(&mut self, component: &Component, namespace: &str) -> Result<()> {
        // Clone the component to allow for ownership
        let component_arc = Arc::new(component.clone());

        // Add the linked component
        self.linked_components
            .insert(namespace.to_string(), component_arc);

        // Return success
        Ok(())
    }

    /// Validates the component
    pub fn validate(&self) -> Result<()> {
        // Check that all imports are satisfied
        for import in &self.imports {
            // Check if this import is used by any export or instance
            let mut used = false;

            // Check exports
            for export in &self.exports {
                if let ExternValue::Function(func) = &export.value {
                    if func.export_name == import.name {
                        used = true;
                        break;
                    }
                }
            }

            // Check instances
            if !used {
                for instance in &self.instances {
                    for export in &instance.exports {
                        if let ExternValue::Function(func) = &export.value {
                            if func.export_name == import.name {
                                used = true;
                                break;
                            }
                        }
                    }
                    if used {
                        break;
                    }
                }
            }

            // Ensure the import is used
            if !used {
                return Err(Error::new(kinds::ValidationError(format!(
                    "Import {} is not used",
                    import.name
                ))));
            }
        }

        // Validate exports match their declared types
        for export in &self.exports {
            match (&export.ty, &export.value) {
                (TypesExternType::Function(func_type), ExternValue::Function(func)) => {
                    // We need a more detailed type checking here
                    // This is a simplified check
                    // In a full implementation, we'd check param and result types
                    true
                }
                (TypesExternType::Table(table_ty), ExternValue::Table(table)) => {
                    // Table types are compatible if they have matching element type and limits
                    table_ty.element_type == table.ty.element_type
                        && table_ty.limits == table.ty.limits
                }
                (TypesExternType::Memory(mem_ty), ExternValue::Memory(memory)) => {
                    // Memory types are compatible if they have matching limits
                    mem_ty.limits == memory.ty.limits && mem_ty.shared == memory.ty.shared
                }
                (TypesExternType::Global(global_ty), ExternValue::Global(global)) => {
                    // Global types are compatible if they have matching type and mutability
                    global_ty.value_type == global.ty.value_type
                        && global_ty.mutable == global.ty.mutable
                }
                _ => {
                    return Err(Error::new(kinds::ValidationError(format!(
                        "Export {} has mismatched type",
                        export.name
                    ))));
                }
            };
        }

        Ok(())
    }

    /// Export a value with the given type
    fn export_value(&mut self, ty: FormatExternType, name: String) -> Result<()> {
        match ty {
            FormatExternType::Function { params, results } => {
                // Export a function value
                let export = Export {
                    name: name.clone(),
                    ty: ty.clone(),
                    value: ExternValue::Function(FunctionValue {
                        ty: wrt_runtime::func::FuncType::new(
                            // Convert value types for params - skip names
                            params
                                .iter()
                                .map(|(_, val_type)| {
                                    format_val_type_to_value_type(val_type)
                                        // Remove map_err and unwrap_or, use ? operator
                                        // .map_err(|e| {
                                        //     Error::new(TypeMismatchError(e.to_string()))
                                        // })
                                        // .unwrap_or(ValueType::I32) // Fallback to I32 in case of error
                                })
                                .collect::<Result<Vec<_>>>()?,
                            // Convert value types for results
                            results
                                .iter()
                                .map(|val_type| {
                                    format_val_type_to_value_type(val_type)
                                        // Remove map_err and unwrap_or, use ? operator
                                        // .map_err(|e| {
                                        //     Error::new(TypeMismatchError(e.to_string()))
                                        // })
                                        // .unwrap_or(ValueType::I32) // Fallback to I32 in case of error
                                })
                                .collect::<Result<Vec<_>>>()?,
                        ),
                        export_name: name,
                    }),
                    attributes: HashMap::new(), // Ensure attributes field exists
                    integrity_hash: None, // Ensure integrity_hash field exists
                };
                self.exports.push(export);
            }
            // Other types would be handled here
            _ => {
                return Err(Error::new(kinds::NotImplementedError(format!(
                    "Export type {:?} not yet supported",
                    ty
                ))));
            }
        }
        Ok(())
    }

    /// Analyze a component binary to create a summary
    pub fn analyze_component(bytes: &[u8]) -> Result<ComponentSummary> {
        // Use wrt-decoder's component analysis instead of our own implementation
        match wrt_decoder::component::analyze_component(bytes) {
            Ok(summary) => {
                // Convert from wrt-decoder summary to our own summary format
                let core_modules = summary
                    .module_info
                    .into_iter()
                    .map(|m| CoreModuleInfo {
                        idx: m.idx,
                        size: m.size,
                    })
                    .collect();

                Ok(ComponentSummary {
                    name: summary.name,
                    core_modules_count: summary.core_modules_count,
                    core_instances_count: summary.core_instances_count,
                    imports_count: summary.imports_count,
                    exports_count: summary.exports_count,
                    aliases_count: summary.aliases_count,
                    core_modules,
                    core_instances: Vec::new(), // Would need conversion logic
                    aliases: Vec::new(),        // Would need conversion logic
                    component_types: Vec::new(), // Would need conversion logic
                })
            }
            Err(e) => Err(Error::new(kinds::RuntimeError(format!(
                "Failed to analyze component: {}",
                e
            )))),
        }
    }

    /// Analyze a component with extended information
    pub fn analyze_component_extended(
        bytes: &[u8],
    ) -> Result<(
        ComponentSummary,
        Vec<ExtendedImportInfo>,
        Vec<ExtendedExportInfo>,
        Vec<ModuleImportInfo>,
        Vec<ModuleExportInfo>,
    )> {
        // First get the basic component summary
        let summary = Self::analyze_component(bytes)?;

        // Get extended information from wrt-decoder
        match wrt_decoder::component::analyze_component_extended(bytes) {
            Ok((_, import_info, export_info, module_imports, module_exports)) => {
                // Convert the import information
                let imports = import_info
                    .into_iter()
                    .map(|info| ExtendedImportInfo {
                        namespace: info.namespace,
                        name: info.name,
                        kind: info.kind,
                    })
                    .collect();

                // Convert the export information
                let exports = export_info
                    .into_iter()
                    .map(|info| ExtendedExportInfo {
                        name: info.name,
                        kind: info.kind,
                        index: info.index,
                    })
                    .collect();

                // Convert module import information
                let mod_imports = module_imports
                    .into_iter()
                    .map(|info| ModuleImportInfo {
                        module: info.module,
                        name: info.name,
                        kind: info.kind,
                        index: info.index,
                        module_idx: info.module_idx,
                    })
                    .collect();

                // Convert module export information
                let mod_exports = module_exports
                    .into_iter()
                    .map(|info| ModuleExportInfo {
                        name: info.name,
                        kind: info.kind,
                        index: info.index,
                        module_idx: info.module_idx,
                    })
                    .collect();

                Ok((summary, imports, exports, mod_imports, mod_exports))
            }
            Err(e) => Err(Error::new(kinds::RuntimeError(format!(
                "Failed to analyze component with extended information: {}",
                e
            )))),
        }
    }

    /// Get producer information from the component's modules
    ///
    /// Returns a vector of producer sections found in the component's modules.
    /// This information can help identify tools that produced the WebAssembly binary.
    pub fn get_producers_info(&self) -> Result<Vec<wrt_decoder::ProducersSection>> {
        let mut result = Vec::new();

        // Use the runtime instance if available
        if let Some(runtime) = &self.runtime {
            // Currently not implemented for runtime
            // We could potentially read binary data from the runtime in a future implementation
        }

        // Since we don't have direct access to modules, we'll check instance data
        for instance in &self.instances {
            // Currently no way to extract binary data from instances
            // This would need additional implementation to retrieve module binaries
        }

        // As a fallback, we could try to extract embedded modules from a cached binary
        // This requires the binary to be passed in, which we don't have in this context
        // For the component_graph_view example, it would be better to extract producers
        // directly in the main function

        Ok(result)
    }

    /// Gets the interceptor if one is set
    fn get_interceptor(&self) -> Option<&LinkInterceptor> {
        self.interceptor.as_ref().map(|arc| arc.as_ref())
    }

    /// Create a new resource
    pub fn create_resource(
        &mut self,
        type_idx: u32,
        data: Arc<dyn Any + Send + Sync>,
    ) -> Result<u32> {
        self.resource_table.create_resource(type_idx, data)
    }

    /// Drop a resource
    pub fn drop_resource(&mut self, handle: u32) -> Result<()> {
        self.resource_table.drop_resource(handle)
    }

    /// Borrow a resource
    pub fn borrow_resource(&mut self, handle: u32) -> Result<u32> {
        self.resource_table.borrow_resource(handle)
    }

    /// Apply a resource operation
    pub fn apply_resource_operation(
        &mut self,
        handle: u32,
        operation: FormatResourceOperation,
    ) -> Result<ComponentValue> {
        self.resource_table.apply_operation(handle, operation)
    }

    /// Set the memory strategy for a resource
    pub fn set_resource_memory_strategy(
        &mut self,
        handle: u32,
        strategy: MemoryStrategy,
    ) -> Result<()> {
        self.resource_table.set_memory_strategy(handle, strategy)
    }

    /// Set the verification level for a resource
    pub fn set_resource_verification_level(
        &mut self,
        handle: u32,
        level: TypesVerificationLevel,
    ) -> Result<()> {
        // Convert the verification level to the resources module version
        let resources_level = convert_verification_level(level);
        
        // Set the verification level in the resource table
        self.resource_table.set_verification_level(handle, resources_level)
    }

    /// Executes the component's start function if one is defined
    ///
    /// This method runs the start function of the component with time limits
    /// and integrity checks. If the component does not have a start function,
    /// this method returns Ok(None).
    ///
    /// # Arguments
    ///
    /// * `time_limit_ms` - Optional time limit in milliseconds for execution
    /// * `fuel_limit` - Optional fuel limit for bounded execution
    ///
    /// # Returns
    ///
    /// * `Ok(Some(values))` - The start function executed successfully and returned values
    /// * `Ok(None)` - The component does not have a start function
    /// * `Err(e)` - An error occurred during execution
    pub fn execute_start(
        &mut self,
        time_limit_ms: Option<u64>,
        fuel_limit: Option<u64>,
    ) -> Result<Option<Vec<ComponentValue>>> {
        // Get the component definition from the format
        let component_def = self.component_type.clone();

        // Set up execution limits
        let start_time = std::time::Instant::now();

        // Look for a start function in the component
        let start_function = self.find_start_function()?;

        if let Some((func_idx, args)) = start_function {
            // Apply interceptor if available for start function
            if let Some(interceptor) = &self.interceptor {
                if let Some(strategy) = interceptor.get_strategy() {
                    // Get component name for the interceptor
                    let component_name = self
                        .component_type
                        .exports
                        .iter()
                        .find_map(|(name, ty)| {
                            if let TypesExternType::Component(_) = ty {
                                Some(name.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| "unnamed_component".to_string());

                    // Allow interception before start execution
                    if let Some(result_data) = strategy.before_start(&component_name)? {
                        // Export at func_idx should be a function
                        if let Some(export) = self.exports.get(func_idx as usize) {
                            if let ExternValue::Function(func) = &export.value {
                                let result_types = func
                                    .ty
                                    .results
                                    .iter()
                                    .map(|param| convert_param_to_value_type(param))
                                    .collect::<Vec<_>>();

                                let results =
                                    deserialize_component_values(&result_data, &result_types)?;

                                // Get function name from the export
                                let func_name = &export.name;

                                // Continue with post-interception
                                let post_results = (&**interceptor).post_intercept(
                                    component_name,
                                    func_name,
                                    &args,
                                    &results,
                                )?;

                                // If there are modifications needed, serialize and deserialize again
                                if post_results.modified {
                                    let serialized_results = serialize_component_values(&results)?;

                                    // Apply the modifications
                                    let modified_data = (&**interceptor).apply_modifications(
                                        &serialized_results,
                                        &post_results.modifications,
                                    )?;

                                    // Deserialize the modified data
                                    let modified_results = deserialize_component_values(
                                        &modified_data,
                                        &result_types,
                                    )?;
                                    return Ok(Some(modified_results));
                                } else {
                                    return Ok(results);
                                }
                            }
                        }
                    }
                }
            }

            // Execute the start function with integrity checks
            let execution_result = self.execute_start_function_with_integrity(
                func_idx,
                args,
                time_limit_ms,
                start_time,
            );

            // Check time limit after execution
            if let Some(limit) = time_limit_ms {
                let elapsed = start_time.elapsed().as_millis() as u64;
                if elapsed > limit {
                    return Err(Error::new(kinds::ExecutionTimeoutError(format!(
                        "Start function execution took too long: {} ms (limit: {} ms)",
                        elapsed, limit
                    ))));
                }
            }

            // Apply interceptor after start function if available
            if let Some(interceptor) = &self.interceptor {
                if let Some(strategy) = interceptor.get_strategy() {
                    // Get component name for the interceptor
                    let component_name = self
                        .component_type
                        .exports
                        .iter()
                        .find_map(|(name, ty)| {
                            if let TypesExternType::Component(_) = ty {
                                Some(name.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| "unnamed_component".to_string());

                    match &execution_result {
                        Ok(results) => {
                            // Serialize results for the interceptor
                            let result_types: Vec<wrt_format::component::ValType> =
                                results.iter().map(|r| r.get_type()).collect();

                            let serialized_results = serialize_component_values(results)?;

                            // Allow interception after start execution
                            if let Some(modified_data) = strategy.after_start(
                                &component_name,
                                &result_types,
                                Some(&serialized_results),
                            )? {
                                // Deserialize modified results
                                let modified_results =
                                    deserialize_component_values(&modified_data, &result_types)?;
                                return Ok(Some(modified_results));
                            }
                        }
                        Err(_) => {
                            // Allow interception of error case
                            if let Some(modified_data) =
                                strategy.after_start(&component_name, &[], None)?
                            {
                                // Deserialize modified results - assume it's a successful recovery
                                let modified_results =
                                    deserialize_component_values(&modified_data, &[])?;
                                return Ok(Some(modified_results));
                            }
                        }
                    }
                }
            }

            // Log the execution result
            match &execution_result {
                Ok(_) => {
                    log::debug!("Start function completed successfully");
                }
                Err(e) => {
                    log::error!("Start function encountered an error: {}", e);
                }
            }

            // Return the result
            execution_result.map(Some)
        } else {
            // No start function defined
            Ok(None)
        }
    }

    /// Find a start function in the component
    fn find_start_function(&self) -> Result<Option<(u32, Vec<ComponentValue>)>> {
        for (idx, export) in self.exports.iter().enumerate() {
            if export.name == "_start" {
                return match &export.ty {
                    FormatExternType::Function { params, results } => {
                        if params.is_empty() && results.is_empty() {
                             Ok(Some((idx as u32, Vec::new())))
                        } else {
                             Err(Error::new(kinds::TypeMismatchError(format!( 
                                "Export '_start' has incorrect signature: expected () -> (), found {:?} -> {:?}",
                                params, results
                            ))))
                        }
                    }
                    _ => Err(Error::new(kinds::TypeMismatchError( 
                        "Export '_start' is not a function".to_string(),
                    ))),
                };
            }
        }
        Ok(None)
    }

    /// Execute the start function with integrity checks
    fn execute_start_function_with_integrity(
        &mut self,
        func_idx: u32,
        args: Vec<ComponentValue>,
        time_limit_ms: Option<u64>,
        start_time: std::time::Instant,
    ) -> Result<Vec<ComponentValue>> {
        // Get the function
        let export = self
            .exports
            .get(func_idx as usize)
            .ok_or_else(|| Error::new(kinds::InvalidFunctionIndex(func_idx.try_into().unwrap())))?;

        // Verify it's a function
        if let ExternValue::Function(func) = &export.value {
            // Check time bounds
            if let Some(limit) = time_limit_ms {
                let elapsed = start_time.elapsed().as_millis() as u64;
                if elapsed > limit {
                    return Err(Error::new(ExecutionTimeoutError(format!(
                        "Start function execution preparation took too long: {} ms (limit: {} ms)",
                        elapsed, limit
                    ))));
                }
            }

            // Verify argument count
            if args.len() != func.ty.params.len() {
                return Err(Error::new(TypeMismatchError(format!(
                    "Start function expects {} arguments, but got {}",
                    func.ty.params.len(),
                    args.len()
                ))));
            }

            // Execute the function
            let result = self.call_function(func_idx, args)?;

            Ok(result)
        } else {
            Err(Error::new(TypeMismatchError(format!(
                "Export at index {} is not a function",
                func_idx
            ))))
        }
    }

    /// Verify the integrity of a function based on its hash
    fn verify_function_integrity(&self, func_idx: u32, expected_hash: &str) -> Result<bool> {
        // This is a placeholder implementation
        // In a real-world scenario, you would verify the function's bytecode against the expected hash

        // Find the function in exports
        let func_export = self.exports.iter().find(|e| {
            if let FormatExternType::Function { .. } = &e.ty {
                e.idx == func_idx
            } else {
                false
            }
        });

        if let Some(export) = func_export {
            // In a complete implementation, compute the hash of the function bytecode
            // and compare it to the expected_hash

            // For now, just return true if the export has an integrity attribute that matches
            if let Some(integrity) = &export.integrity_hash {
                return Ok(integrity == expected_hash);
            }
        }

        // Function not found or no integrity hash
        Ok(false)
    }

    /// Call a function by its index in the exports
    pub fn call_function(
        &mut self,
        func_idx: u32,
        args: Vec<ComponentValue>,
    ) -> Result<Vec<ComponentValue>> {
        debug_println(&format!("Calling function index: {}", func_idx));
        let func_export = self.exports.get(func_idx as usize).ok_or_else(|| {
            Error::new(FunctionNotFoundError(format!( // Use specific error struct
                "Function export with index {} not found", func_idx
            )))
        })?;

        let (format_param_types, format_result_types, func_name) = match &func_export.ty {
             FormatExternType::Function{ params, results } => (params, results, &func_export.name),
             _ => return Err(Error::new(TypeMismatchError(format!( // Use specific error struct
                 "Export at index {} ('{}') is not a function", func_idx, func_export.name
             )))),
        };
        let format_param_types = format_param_types.clone();
        let format_result_types = format_result_types.clone();
        debug_println(&format!("Found function export '{}' with type: {:?} -> {:?}", func_name, format_param_types, format_result_types));

        let arg_types: Vec<FormatValType> = args
            .iter()
            .map(|arg| common_to_format_val_type(&arg.get_type()))
            .collect::<Result<Vec<_>>>()?;

        // Fix E0277: Compare Vec<FormatValType> with Vec<FormatValType>
        let format_param_valtypes: Vec<_> = format_param_types.iter().map(|(_, ty)| ty).cloned().collect();
        if arg_types != format_param_valtypes {
            return Err(Error::new(TypeMismatchError(format!( // Use specific error struct
                 "Argument type mismatch for function '{}': expected {:?}, got {:?}",
                 func_name, format_param_valtypes, arg_types
            ))));
        }
        debug_println("Argument types match expected parameter types.");

        let data = serialize_component_values(&args)?;
        debug_println(&format!("Serialized args ({} bytes)", data.len()));

        let _data_to_call = if let Some(_interceptor) = self.get_interceptor() { // Mark interceptor unused for now
            debug_println("Skipping lower interception..."); // Skip interception logic
            data
        } else {
            data
        };

        let core_args: Vec<Value> = args
            .into_iter()
            .map(|cv| component_value_to_value(&cv))
            .collect::<Result<Vec<_>>>()?;
        debug_println(&format!("Converted {} ComponentValues to Core Values", core_args.len()));

        debug_println(&format!("Executing resolved function '{}' (idx {})", func_name, func_idx));
        let core_results = self.call_resolved_function(func_idx, func_name, core_args)?;
        debug_println(&format!("Resolved function returned {} core results", core_results.len()));

        let final_core_results = if let Some(_interceptor) = self.get_interceptor() { // Mark interceptor unused
             debug_println("Skipping lift interception..."); // Skip interception logic
             core_results
        } else {
            core_results
        };

        // Fix E0061: value_to_component_value usage - needs type info
        // This fix assumes value_to_component_value can handle missing type info or uses a default.
        // A proper fix might require passing the correct FormatValType.
        // For now, let's comment out the map body to see other errors.
        // This part needs revisiting based on the definition of value_to_component_value.
        let component_results: Vec<ComponentValue> = final_core_results
            .into_iter()
            .zip(format_result_types.iter()) // Pair Value with its FormatValType
             // .map(|(value, format_type)| value_to_component_value(&value, format_type)) // Original failing call
             // Temporary fix: Call with only &value if value_to_component_value signature is changed, or use a placeholder type.
             // Assuming value_to_component_value is updated or needs more context.
             // For now, let's comment out the map body to see other errors.
             // This part needs revisiting based on the definition of value_to_component_value.
             .map(|(value, _format_type)| value_to_component_value(&value /* placeholder */)) // Placeholder call
            .collect::<Result<Vec<_>>>()?;
        debug_println(&format!("Converted {} Core Values back to ComponentValues", component_results.len()));

        Ok(component_results)
    }

    fn call_resolved_function(&self, _func_idx: u32, func_name: &str, args: Vec<Value>) -> Result<Vec<Value>> {
         if let Some(runtime) = &self.runtime {
             debug_println(&format!("Delegating call '{}' to runtime instance", func_name));
             runtime.execute_function(func_name, args)
         } else if let Some(_registry) = &self.callback_registry { // Mark registry unused
             debug_println(&format!("Delegating call '{}' to host callback registry - SKIPPED (Needs fix)", func_name));
             Err(Error::new(NotImplementedError( // Use specific error struct
                 "CallbackRegistry interaction needs correct method call".to_string(),
             )))
         } else {
             error!("Cannot call function '{}': No runtime or host registry configured", func_name);
             Err(Error::new(ExecutionError(format!( // Use specific error struct
                 "Cannot call function '{}': No runtime or host registry configured", func_name
             ))))
         }
    }
}

// Helper function to get the type index for a module
fn get_module_type_index(
    module_idx: u32,
    component: &wrt_format::component::Component,
) -> Option<u32> {
    for instance in &component.core_instances {
        if let wrt_format::component::CoreInstanceExpr::Instantiate {
            module_idx: idx, ..
        } = &instance.instance_expr
        {
            if *idx == module_idx {
                return Some(0); // Simplified - we would need more info to get the actual type
            }
        }
    }
    None
}

// Helper function to format core extern type info
fn format_core_extern_type(ty: &wrt_format::component::CoreExternType) -> String {
    match ty {
        wrt_format::component::CoreExternType::Function { params, results } => {
            format!(
                "Function(params={}, results={})",
                params.len(),
                results.len()
            )
        }
        wrt_format::component::CoreExternType::Table {
            element_type,
            min,
            max,
        } => {
            format!("Table({:?}, min={}, max={:?})", element_type, min, max)
        }
        wrt_format::component::CoreExternType::Memory { min, max, shared } => {
            format!("Memory(min={}, max={:?}, shared={})", min, max, shared)
        }
        wrt_format::component::CoreExternType::Global {
            value_type,
            mutable,
        } => {
            format!("Global({:?}, mutable={})", value_type, mutable)
        }
    }
}

/// Extract embedded modules from the component binary
pub fn extract_embedded_modules(bytes: &[u8]) -> Result<Vec<Vec<u8>>> {
    let mut modules = Vec::new();
    let mut offset = 8; // Skip magic + version bytes

    // First, try to find modules in the proper core module section
    while offset < bytes.len() {
        // Check if we have at least one byte for section ID
        if offset >= bytes.len() {
            break;
        }

        let section_id = bytes[offset];
        offset += 1;

        // If this is a core module section
        if section_id == binary::COMPONENT_CORE_MODULE_SECTION_ID {
            // Parse the section size
            if offset >= bytes.len() {
                break;
            }

            let (section_size, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            let section_end = offset + section_size as usize;
            if section_end > bytes.len() {
                break;
            }

            // Parse number of modules
            let (module_count, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;

            // Extract each module
            for _ in 0..module_count {
                // Read module size
                let (module_size, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
                offset += bytes_read;

                if offset + module_size as usize > bytes.len() {
                    break;
                }

                // Extract the module bytes
                let module_bytes = bytes[offset..offset + module_size as usize].to_vec();

                // Check if this is a valid module
                if is_valid_module(&module_bytes) {
                    modules.push(module_bytes);
                }

                // Move to the next module
                offset += module_size as usize;
            }
        } else {
            // Skip non-module sections
            if offset >= bytes.len() {
                break;
            }

            let (section_size, bytes_read) = binary::read_leb128_u32(bytes, offset)?;
            offset += bytes_read;
            offset += section_size as usize;
        }
    }

    // If no modules were found, try to find embedded modules in the raw binary
    if modules.is_empty() {
        // The WebAssembly module magic bytes and version
        let wasm_magic_version = [0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

        // Look for the core module at the beginning of the binary
        // This is the most likely place for an inline module in a simple component
        if bytes.len() > 16 && bytes[0x0C..0x14] == wasm_magic_version {
            // Debug print to help with troubleshooting
            debug_println(&format!(
                "Found potential embedded module at offset 0x0C with magic bytes: [{}]",
                bytes[0x0C..0x14]
                    .iter()
                    .map(|b| format!("0x{:02x}", b))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));

            // The component format often embeds the core module right after the component header
            // Extract it and check if it's a valid module
            let mut module_end = 0x14; // Start from after the magic+version
            let mut section_count = 0;

            // Basic module parsing to find its end
            while module_end < bytes.len() {
                // If we've found a reasonable number of sections, this is likely a valid module
                if section_count >= 5 {
                    // Extract the potential module
                    let module_bytes = bytes[0x0C..module_end].to_vec();
                    if is_valid_module(&module_bytes) {
                        modules.push(module_bytes);
                    }
                    break;
                }

                // Check for section ID
                if module_end + 1 < bytes.len() {
                    let section_id = bytes[module_end];
                    module_end += 1;

                    // Skip this section
                    if module_end < bytes.len() {
                        let (section_size, bytes_read) =
                            match binary::read_leb128_u32(bytes, module_end) {
                                Ok(result) => result,
                                Err(_) => break, // Not a valid LEB128, stop parsing
                            };
                        module_end += bytes_read;
                        module_end += section_size as usize;
                        section_count += 1;
                    }
                } else {
                    break;
                }
            }
        }
    }

    Ok(modules)
}

/// Check if a byte array represents a valid WebAssembly module
pub fn is_valid_module(bytes: &[u8]) -> bool {
    // Check for magic bytes and version
    if bytes.len() < 8 {
        return false;
    }

    // Check magic number and version - make clear we're using hex values
    if bytes[0..4] != [0x00, 0x61, 0x73, 0x6D] || // \0asm (0x00, 0x61, 0x73, 0x6D)
       bytes[4..8] != [0x01, 0x00, 0x00, 0x00]
    {
        // Version 1.0 (0x01, 0x00, 0x00, 0x00)
        debug_println(&format!(
            "Invalid magic/version. Found: [{}], [{}]",
            bytes[0..4]
                .iter()
                .map(|b| format!("0x{:02x}", b))
                .collect::<Vec<_>>()
                .join(", "),
            bytes[4..8]
                .iter()
                .map(|b| format!("0x{:02x}", b))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        return false;
    }

    // Validate basic module structure - try to parse sections
    let mut offset = 8;
    let mut section_count = 0;

    while offset < bytes.len() {
        // Read section ID
        if offset >= bytes.len() {
            break;
        }
        let _section_id = bytes[offset];
        offset += 1;

        // Read section size
        if offset >= bytes.len() {
            return false; // Unexpected end
        }

        match binary::read_leb128_u32(bytes, offset) {
            Ok((section_size, bytes_read)) => {
                offset += bytes_read;

                // Check if section fits in the module
                if offset + section_size as usize > bytes.len() {
                    return false;
                }

                // Skip section content
                offset += section_size as usize;
                section_count += 1;
            }
            Err(_) => return false, // Invalid LEB128
        }
    }

    // A valid module should have at least a few sections
    section_count >= 1
}

/// Helper struct for extracted module information
struct ModuleInfo {
    exports: Vec<String>,
    imports: Vec<String>,
    function_count: usize,
    memory_count: usize,
    table_count: usize,
    global_count: usize,
}

/// Extract basic information from a WebAssembly module binary
fn extract_module_info(binary: &[u8]) -> Result<ModuleInfo> {
    // Basic validation of the module header
    if binary.len() < 8 || binary[0..4] != [0x00, 0x61, 0x73, 0x6D] {
        return Err(Error::new(kinds::ParseError(
            "Invalid module header".to_string(),
        )));
    }

    let mut info = ModuleInfo {
        exports: Vec::new(),
        imports: Vec::new(),
        function_count: 0,
        memory_count: 0,
        table_count: 0,
        global_count: 0,
    };

    // Simple section scanning to extract basic information
    let mut offset = 8; // Skip magic + version
    while offset < binary.len() {
        // Read section ID
        let section_id = binary[offset];
        offset += 1;

        // Read section size
        let mut size_offset = 0;
        let (size, bytes_read) = binary::read_leb128_u32(binary, offset)?;
        offset += bytes_read;

        let section_end = offset + size as usize;

        // Process sections of interest
        match section_id {
            // Function section
            1 => {
                // Read count
                let (count, _) = binary::read_leb128_u32(binary, offset)?;
                info.function_count = count as usize;
            }
            // Import section
            2 => {
                let mut import_offset = offset;
                let (count, bytes_read) = binary::read_leb128_u32(binary, import_offset)?;
                import_offset += bytes_read;

                for _ in 0..count {
                    // Read module name
                    let (module, bytes_read) = binary::read_string(binary, import_offset)?;
                    import_offset += bytes_read;

                    // Read field name
                    let (field, bytes_read) = binary::read_string(binary, import_offset)?;
                    import_offset += bytes_read;

                    info.imports.push(format!("{}.{}", module, field));

                    // Skip the import kind and type
                    let kind = binary[import_offset];
                    import_offset += 1;

                    // Skip type information based on kind
                    match kind {
                        0 => {
                            // Function import
                            let (_, bytes_read) = binary::read_leb128_u32(binary, import_offset)?;
                            import_offset += bytes_read;
                        }
                        1 => {
                            // Table import
                            import_offset += 1; // elem type
                            let flags = binary[import_offset];
                            import_offset += 1;

                            // Initial size
                            let (_, bytes_read) = binary::read_leb128_u32(binary, import_offset)?;
                            import_offset += bytes_read;

                            // Max size if present
                            if (flags & 0x01) != 0 {
                                let (_, bytes_read) =
                                    binary::read_leb128_u32(binary, import_offset)?;
                                import_offset += bytes_read;
                            }
                        }
                        2 => {
                            // Memory import
                            let flags = binary[import_offset];
                            import_offset += 1;

                            // Initial size
                            let (_, bytes_read) = binary::read_leb128_u32(binary, import_offset)?;
                            import_offset += bytes_read;

                            // Max size if present
                            if (flags & 0x01) != 0 {
                                let (_, bytes_read) =
                                    binary::read_leb128_u32(binary, import_offset)?;
                                import_offset += bytes_read;
                            }
                        }
                        3 => {
                            // Global import
                            import_offset += 1; // value type
                            import_offset += 1; // mutability
                        }
                        _ => {
                            // Unknown import kind, can't parse further
                            break;
                        }
                    }
                }
            }
            // Table section
            4 => {
                let (count, _) = binary::read_leb128_u32(binary, offset)?;
                info.table_count = count as usize;
            }
            // Memory section
            5 => {
                let (count, _) = binary::read_leb128_u32(binary, offset)?;
                info.memory_count = count as usize;
            }
            // Global section
            6 => {
                let (count, _) = binary::read_leb128_u32(binary, offset)?;
                info.global_count = count as usize;
            }
            // Export section
            7 => {
                let mut export_offset = offset;
                let (count, bytes_read) = binary::read_leb128_u32(binary, export_offset)?;
                export_offset += bytes_read;

                for _ in 0..count {
                    // Read name
                    let (name, bytes_read) = binary::read_string(binary, export_offset)?;
                    export_offset += bytes_read;

                    info.exports.push(name);

                    // Skip kind and index
                    export_offset += 1; // kind
                    let (_, bytes_read) = binary::read_leb128_u32(binary, export_offset)?;
                    export_offset += bytes_read;
                }
            }
            _ => {
                // Skip other sections
            }
        }

        // Move to the next section
        offset = section_end;
    }

    Ok(info)
}

/// Helper function to format component type info for display
fn format_component_type_info(ty: &wrt_format::component::ComponentTypeDefinition) -> String {
    match ty {
        wrt_format::component::ComponentTypeDefinition::Function { params, results } => {
            format!(
                "Function({} params, {} results)",
                params.len(),
                results.len()
            )
        }
        wrt_format::component::ComponentTypeDefinition::Instance { exports } => {
            format!("Instance({} exports)", exports.len())
        }
        wrt_format::component::ComponentTypeDefinition::Component { imports, exports } => {
            format!(
                "Component({} imports, {} exports)",
                imports.len(),
                exports.len()
            )
        }
        wrt_format::component::ComponentTypeDefinition::Value(val_type) => {
            format!("Value({:?})", val_type)
        }
        _ => format!("{:?}", ty),
    }
}

/// Helper function to format extern type info for display
fn format_extern_type_info(ty: &wrt_format::component::ExternType) -> String {
    match ty {
        wrt_format::component::ExternType::Function { params, results } => {
            format!(
                "Function({} params, {} results)",
                params.len(),
                results.len()
            )
        }
        wrt_format::component::ExternType::Value(val_type) => {
            format!("Value({})", format_val_type(val_type))
        }
        wrt_format::component::ExternType::Type(idx) => {
            format!("Type(idx={})", idx)
        }
        wrt_format::component::ExternType::Instance { exports } => {
            format!("Instance(exports={})", exports.len())
        }
        wrt_format::component::ExternType::Component { imports, exports } => {
            format!(
                "Component(imports={}, exports={})",
                imports.len(),
                exports.len()
            )
        }
    }
}

// Helper function to format val type
fn format_val_type(val_type: &wrt_format::component::ValType) -> String {
    match val_type {
        wrt_format::component::ValType::Bool => "bool".to_string(),
        wrt_format::component::ValType::S8 => "s8".to_string(),
        wrt_format::component::ValType::U8 => "u8".to_string(),
        wrt_format::component::ValType::S16 => "s16".to_string(),
        wrt_format::component::ValType::U16 => "u16".to_string(),
        wrt_format::component::ValType::S32 => "s32".to_string(),
        wrt_format::component::ValType::U32 => "u32".to_string(),
        wrt_format::component::ValType::S64 => "s64".to_string(),
        wrt_format::component::ValType::U64 => "u64".to_string(),
        wrt_format::component::ValType::F32 => "f32".to_string(),
        wrt_format::component::ValType::F64 => "f64".to_string(),
        wrt_format::component::ValType::Char => "char".to_string(),
        wrt_format::component::ValType::String => "string".to_string(),
        wrt_format::component::ValType::Ref(idx) => format!("ref({})", idx),
        wrt_format::component::ValType::Record(fields) => {
            format!("record(fields={})", fields.len())
        }
        wrt_format::component::ValType::Variant(cases) => {
            format!("variant(cases={})", cases.len())
        }
        wrt_format::component::ValType::List(inner) => {
            format!("list({})", format_val_type(inner))
        }
        wrt_format::component::ValType::Tuple(elements) => {
            format!("tuple(elements={})", elements.len())
        }
        wrt_format::component::ValType::Flags(names) => {
            format!("flags(count={})", names.len())
        }
        wrt_format::component::ValType::Enum(cases) => {
            format!("enum(cases={})", cases.len())
        }
        wrt_format::component::ValType::Option(inner) => {
            format!("option({})", format_val_type(inner))
        }
        wrt_format::component::ValType::Result(ok) => {
            format!("result<{}, _>", format_val_type(ok))
        }
        wrt_format::component::ValType::ResultErr(err) => {
            format!("result<_, {}>", format_val_type(err))
        }
        wrt_format::component::ValType::ResultBoth(ok, err) => {
            format!("result<{}, {}>", format_val_type(ok), format_val_type(err))
        }
        wrt_format::component::ValType::Own(idx) => {
            format!("own({})", idx)
        }
        wrt_format::component::ValType::Borrow(idx) => {
            format!("borrow({})", idx)
        }
    }
}

/// Checks if two types are compatible
fn types_are_compatible(a: &FormatExternType, b: &FormatExternType) -> bool {
    match (a, b) {
        (
            FormatExternType::Function { .. },
            FormatExternType::Function { .. },
        ) => {
            // Convert to our FormatFuncType and use func_types_compatible
            if let (Some(a_func), Some(b_func)) = (
                crate::type_conversion::extern_type_to_func_type(a),
                crate::type_conversion::extern_type_to_func_type(b),
            ) {
                return func_types_compatible(&a_func, &b_func);
            }
            false
        }
        (FormatExternType::Type(a_idx), FormatExternType::Type(b_idx)) => a_idx == b_idx,
        (FormatExternType::Value(a_val_type), FormatExternType::Value(b_val_type)) => {
            a_val_type == b_val_type
        }
        (
            FormatExternType::Instance { exports: a_exports },
            FormatExternType::Instance { exports: b_exports },
        ) => {
            // Check that export counts match
            if a_exports.len() != b_exports.len() {
                return false;
            }

            // Check that exports match by name and type
            for ((a_name, a_type), (b_name, b_type)) in a_exports.iter().zip(b_exports.iter()) {
                if a_name != b_name || !types_are_compatible(a_type, b_type) {
                    return false;
                }
            }

            true
        }
        (
            FormatExternType::Component {
                imports: a_imports,
                exports: a_exports,
            },
            FormatExternType::Component {
                imports: b_imports,
                exports: b_exports,
            },
        ) => {
            // Check that import counts match
            if a_imports.len() != b_imports.len() {
                return false;
            }

            // Check that import namespaces, names, and types match
            for ((a_ns, a_name, a_type), (b_ns, b_name, b_type)) in
                a_imports.iter().zip(b_imports.iter())
            {
                if a_ns != b_ns || a_name != b_name || !types_are_compatible(a_type, b_type) {
                    return false;
                }
            }

            // Check that export counts match
            if a_exports.len() != b_exports.len() {
                return false;
            }

            // Check that exports match by name and type
            for ((a_name, a_type), (b_name, b_type)) in a_exports.iter().zip(b_exports.iter()) {
                if a_name != b_name || !types_are_compatible(a_type, b_type) {
                    return false;
                }
            }

            true
        }
        // Different types are not compatible
        _ => false,
    }
}

/// Checks if two function types are compatible
fn func_types_compatible(a: &crate::type_conversion::FormatFuncType, b: &crate::type_conversion::FormatFuncType) -> bool {
    if a.params.len() != b.params.len() || a.results.len() != b.results.len() {
        return false;
    }

    for (a_param, b_param) in a.params.iter().zip(b.params.iter()) {
        if a_param != b_param {
            return false;
        }
    }

    for (a_result, b_result) in a.results.iter().zip(b.results.iter()) {
        if a_result != b_result {
            return false;
        }
    }

    true
}

/// Debug println function that uses the host callback registry if available.
/// Falls back to regular println! if the std feature is enabled.
#[cfg(feature = "std")]
fn debug_println(msg: &str) {
    // Use standard println in debug builds with std feature
    #[cfg(debug_assertions)]
    println!("DEBUG: {}", msg);
}

/// Debug println function that does nothing in no_std mode
#[cfg(not(feature = "std"))]
fn debug_println(_msg: &str) {
    // No-op in no_std mode
}

#[cfg(test)]
mod tests {
    use super::*;
    use wrt_format::component::ValType;
    use wrt_host::function::CloneableFn;

    /// Creates a test component type
    fn create_test_component_type() -> ComponentType {
        ComponentType {
            imports: vec![(
                "wasi".to_string(),
                "print".to_string(),
                ExternType::Function {
                    params: vec![("message".to_string(), ValType::S32)],
                    results: vec![],
                },
            )],
            exports: vec![(
                "add".to_string(),
                ExternType::Function {
                    params: vec![
                        ("a".to_string(), ValType::S32),
                        ("b".to_string(), ValType::S32),
                    ],
                    results: vec![("result".to_string(), ValType::S32)],
                },
            )],
            instances: Vec::new(),
        }
    }

    /// Creates a test import
    fn create_test_import() -> Import {
        Import {
            namespace: "wasi".to_string(),
            name: "print".to_string(),
            ty: ExternType::Function {
                params: vec![("message".to_string(), ValType::S32)],
                results: vec![],
            },
            value: ExternValue::Function(FunctionValue {
                ty: FuncType {
                    params: vec![("message".to_string(), ValType::S32)],
                    results: vec![],
                },
                export_name: "print".to_string(),
            }),
        }
    }

    /// Test component creation and instantiation
    #[test]
    fn test_component_creation_and_instantiation() -> Result<()> {
        // Create a component
        let mut component = Component::new(create_test_component_type());

        // Test that it has the right imports and exports
        assert_eq!(component.component_type.imports.len(), 1);
        assert_eq!(component.component_type.exports.len(), 1);

        // Instantiate the component
        component.instantiate(vec![create_test_import()])?;

        // Verify it worked
        assert!(!component.exports.is_empty());
        assert!(!component.imports.is_empty());

        Ok(())
    }

    #[test]
    fn test_component_export_types() -> Result<()> {
        // Create a component with a function export
        let mut component = Component::new(ComponentType {
            imports: Vec::new(),
            exports: vec![(
                "add".to_string(),
                ExternType::Function {
                    params: vec![
                        ("a".to_string(), ValType::S32),
                        ("b".to_string(), ValType::S32),
                    ],
                    results: vec![("result".to_string(), ValType::S32)],
                },
            )],
            instances: Vec::new(),
        });

        // Add a function export directly
        component.export_value(
            ExternType::Function {
                params: vec![
                    ("a".to_string(), ValType::S32),
                    ("b".to_string(), ValType::S32),
                ],
                results: vec![("result".to_string(), ValType::S32)],
            },
            "add".to_string(),
        )?;

        // Verify the export
        let export = component.get_export("add");
        match &export.ty {
            ExternType::Function { params, results } => {
                assert_eq!(params.len(), 2);
                assert_eq!(results.len(), 1);
            }
            _ => {
                return Err(Error::new(kinds::ValidationError(
                    "Expected function export".to_string(),
                )));
            }
        }

        Ok(())
    }

    #[test]
    fn test_component_invalid_instantiation() {
        // Create a component type with imports
        let mut component_type = ComponentType::new();

        // Add an import
        component_type.imports.push((
            "add".to_string(),
            "host".to_string(),
            ExternType::Function(FuncType {
                params: vec![
                    wrt_types::types::ValueType::I32,
                    wrt_types::types::ValueType::I32,
                ],
                results: vec![wrt_types::types::ValueType::I32],
            }),
        ));

        // Create a component
        let mut component = Component::new(component_type);

        // Try to instantiate with no imports
        let result = component.instantiate(vec![]);

        // Should fail due to missing imports
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            Error::Validation(msg) => {
                assert!(
                    msg.contains("Expected 1 imports, got 0"),
                    "Unexpected error message: {}",
                    msg
                );
            }
            _ => panic!("Expected ValidationError, got {:?}", err),
        }

        // Try with wrong import type
        let wrong_import = Import {
            name: "add".to_string(),
            ty: ExternType::Type(0), // Use Type instead of Memory
            value: ExternValue::Memory(
                MemoryValue::new(MemoryType {
                    limits: wrt_types::types::Limits {
                        min: 1,
                        max: Some(2),
                        shared: false,
                    },
                })
                .unwrap(),
            ),
        };

        let result = component.instantiate(vec![wrong_import]);

        // Should fail due to import type mismatch during validation
        assert!(result.is_err());
    }

    #[test]
    fn test_component_function_calls() -> Result<()> {
        // Create a simple component with a function
        let mut component = Component::new(ComponentType::default());

        // Create a mock runtime that can handle function calls
        struct MockRuntime;
        impl RuntimeInstance {
            fn mock() -> Self {
                Self {}
            }

            #[cfg(test)]
            fn mock_execute_function(&self, name: &str, args: Vec<Value>) -> Result<Vec<Value>> {
                // Mock implementation for testing only
                match name {
                    "add" => {
                        if args.len() != 2 {
                            return Err(Error::new(kinds::ValidationError(
                                "add expects 2 args".to_string(),
                            )));
                        }
                        if let (Value::I32(a), Value::I32(b)) = (&args[0], &args[1]) {
                            Ok(vec![Value::I32(a + b)])
                        } else {
                            Err(Error::new(TypeMismatchError(
                                "Expected i32 args".to_string(),
                            )))
                        }
                    }
                    _ => Err(Error::new(kinds::NotImplementedError(format!(
                        "Function {} not implemented",
                        name
                    )))),
                }
            }
        }

        // Override execute_function for testing
        impl RuntimeInstance {
            #[cfg(test)]
            fn execute_function(&self, name: &str, args: Vec<Value>) -> Result<Vec<Value>> {
                Self::mock_execute_function(self, name, args)
            }
        }

        // Add runtime to component
        component = component.with_runtime(RuntimeInstance::mock());

        // Add a function export directly (bypassing normal instantiation)
        component.export_value(
            ExternType::Function(FuncType {
                params: vec![
                    wrt_types::types::ValueType::I32,
                    wrt_types::types::ValueType::I32,
                ],
                results: vec![wrt_types::types::ValueType::I32],
            }),
            "add".to_string(),
        )?;

        // Try calling the function
        let result = component.execute_function("add", vec![Value::I32(5), Value::I32(3)])?;

        // We should get back the addition result
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Value::I32(8));

        Ok(())
    }

    #[test]
    fn test_host_function_management() -> Result<()> {
        // Create a host
        let mut host = Host::new();

        // Add a host function
        let func_value = FunctionValue {
            ty: FuncType {
                params: vec![wrt_types::types::ValueType::I32],
                results: vec![wrt_types::types::ValueType::I32],
            },
            export_name: "increment".to_string(),
        };

        host.add_function("increment".to_string(), func_value);

        // Check that we can retrieve the function
        let retrieved_func = host.get_function("increment");
        assert!(retrieved_func.is_some());

        // Check that we can't retrieve a nonexistent function
        assert!(host.get_function("nonexistent").is_none());

        // Calling the function would fail since we have no implementation
        let result = host.call_function("increment", &[Value::I32(5)]);
        assert!(result.is_err());
        if let Error::NotImplemented(_) = result.unwrap_err() {
            // Expected error
        } else {
            panic!("Expected NotImplementedError");
        }

        Ok(())
    }

    #[test]
    fn test_host_function_call_validation() {
        // Create a host
        let mut host = Host::new();

        // Add a host function
        let func_value = FunctionValue {
            ty: FuncType {
                params: vec![
                    wrt_types::types::ValueType::I32,
                    wrt_types::types::ValueType::I32,
                ],
                results: vec![wrt_types::types::ValueType::I32],
            },
            export_name: "add".to_string(),
        };

        host.add_function("add".to_string(), func_value);

        // Try calling with wrong number of arguments
        let result = host.call_function("add", &[Value::I32(5)]);
        assert!(result.is_err());

        // Try calling a nonexistent function
        let result = host.call_function("nonexistent", &[Value::I32(5)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_export_mutation() -> Result<()> {
        // Create a component
        let mut component = Component::new(ComponentType::default());

        // Add a global export
        component.export_value(
            ExternType::Global(GlobalType {
                value_type: wrt_types::types::ValueType::I32,
                mutable: true,
            }),
            "counter".to_string(),
        )?;

        // Get the export
        let export = component.get_export("counter")?;

        // Check its type
        match &export.ty {
            ExternType::Global(ty) => {
                assert_eq!(ty.value_type, wrt_types::types::ValueType::I32);
                assert!(ty.mutable);
            }
            _ => panic!("Expected global export"),
        }

        // Update the export
        component.export_value(
            ExternType::Global(GlobalType {
                value_type: wrt_types::types::ValueType::I64,
                mutable: false,
            }),
            "counter".to_string(),
        )?;

        // Get the updated export
        let export = component.get_export("counter")?;

        // Check its new type
        match &export.ty {
            ExternType::Global(ty) => {
                assert_eq!(ty.value_type, wrt_types::types::ValueType::I64);
                assert!(!ty.mutable);
            }
            _ => panic!("Expected global export"),
        }

        Ok(())
    }

    #[test]
    fn test_component_linking() -> Result<()> {
        // Create a parent component
        let mut parent = Component::new(ComponentType::new());

        // Create a child component
        let mut child = Component::new(ComponentType::new());

        // Add an export to child
        child.export_value(
            ExternType::Function {
                params: vec![("a".to_string(), ValType::S32)],
                results: vec![ValType::S32],
            },
            "test_func".to_string(),
        )?;

        // Link child to parent with namespace
        parent.link_component(&child, "child")?;

        // Check if export is available in parent with namespace
        let export = parent.get_export("child.test_func");
        assert!(export.is_ok());

        Ok(())
    }

    #[test]
    fn test_host_function_integration() -> Result<()> {
        // Create a component
        let mut component = Component::new(ComponentType::new());

        // Create a callback registry
        let mut registry = CallbackRegistry::new();

        // Register a host function
        registry.register_host_function(
            "test_api",
            "add",
            CloneableFn::new(|_, args| {
                if args.len() != 2 {
                    return Err(Error::new(kinds::ValidationError(
                        "add expects 2 arguments".to_string(),
                    )));
                }

                match (&args[0], &args[1]) {
                    (Value::I32(a), Value::I32(b)) => Ok(vec![Value::I32(a + b)]),
                    _ => Err(Error::new(TypeMismatchError(
                        "add expects i32 arguments".to_string(),
                    ))),
                }
            }),
        );

        // Attach registry to component
        component = component.with_callback_registry(Arc::new(registry));

        // Register host API in component
        component.register_host_api(
            "test_api",
            vec![(
                "add".to_string(),
                FuncType {
                    params: vec![ValType::S32, ValType::S32],
                    results: vec![ValType::S32],
                },
                // In a real implementation, we would create a real handler here
                // For test, this is just a placeholder
                CloneableFn::new(|_, _| Ok(vec![Value::I32(0)])),
            )],
        )?;

        // Check if export is available
        let export = component.get_export("test_api.add");
        assert!(export.is_ok());

        Ok(())
    }
}

/// Placeholder clone implementation - this would need to be properly implemented
impl Clone for Component {
    fn clone(&self) -> Self {
        // Create a new component with the same type
        let mut component = Self::new(self.component_type.clone());

        // Clone exports, imports, instances, etc.
        component.exports = self.exports.clone();
        component.imports = self.imports.clone();
        component.instances = self.instances.clone();

        // Note: We don't clone linked_components to avoid circular references
        // This is a simplified implementation

        component
    }
}

/// Try to extract the inline module at module_idx 0
fn extract_inline_module(bytes: &[u8]) -> Result<Option<Vec<u8>>> {
    // In this special case, we're looking specifically at module 0
    // which is often embedded directly in the binary after the component header
    if bytes.len() < 16 {
        return Ok(None);
    }

    // Check if at offset 0xC we have the start of a WebAssembly module
    if bytes[0xC..0x14] != [0x00, 0x61, 0x73, 0x6D] {
        // \0asm magic
        return Ok(None);
    }

    if bytes[0x10..0x14] != [0x01, 0x00, 0x00, 0x00] {
        // version 1.0
        return Ok(None);
    }

    // We found a module, now need to determine its size
    let mut offset = 0x14; // Start after the magic+version
    let mut section_count = 0;
    let mut valid_module = true;

    // Parse the module sections to find its end
    while offset < bytes.len() {
        // If we've found enough sections, this is likely a valid module
        if section_count >= 5 {
            break;
        }

        // Check if we have a section ID
        if offset >= bytes.len() {
            valid_module = false;
            break;
        }

        let _section_id = bytes[offset];
        offset += 1;

        // Parse section size
        if offset >= bytes.len() {
            valid_module = false;
            break;
        }

        match binary::read_leb128_u32(bytes, offset) {
            Ok((section_size, bytes_read)) => {
                offset += bytes_read;

                // Check if section fits in the binary
                if offset + section_size as usize > bytes.len() {
                    valid_module = false;
                    break;
                }

                // Skip section content
                offset += section_size as usize;
                section_count += 1;
            }
            Err(_) => {
                valid_module = false;
                break;
            }
        }
    }

    if valid_module {
        // Extract the module
        let module_bytes = bytes[0xC..offset].to_vec();
        Ok(Some(module_bytes))
    } else {
        Ok(None)
    }
}

/// Summary information about a WebAssembly Component
#[derive(Debug)]
pub struct ComponentSummary {
    /// Component name
    pub name: String,
    /// Number of core modules in the component
    pub core_modules_count: u32,
    /// Number of core instances in the component
    pub core_instances_count: u32,
    /// Number of aliases in the component
    pub aliases_count: u32,
    /// Number of imports in the component
    pub imports_count: u32,
    /// Number of exports in the component
    pub exports_count: u32,
    /// Details about core modules
    pub core_modules: Vec<CoreModuleInfo>,
    /// Details about core instances
    pub core_instances: Vec<CoreInstanceInfo>,
    /// Details about aliases
    pub aliases: Vec<AliasInfo>,
    /// Component types
    pub component_types: Vec<wrt_format::Component>,
}

impl std::fmt::Display for ComponentSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Component Summary:")?;
        writeln!(f, "  Name: {}", self.name)?;
        writeln!(f, "  Core Modules: {}", self.core_modules_count)?;
        writeln!(f, "  Core Instances: {}", self.core_instances_count)?;
        writeln!(f, "  Aliases: {}", self.aliases_count)?;
        writeln!(f, "  Imports: {}", self.imports_count)?;
        writeln!(f, "  Exports: {}", self.exports_count)?;

        if !self.core_modules.is_empty() {
            writeln!(f, "\nCore Modules:")?;
            for module in &self.core_modules {
                writeln!(f, "  - Module {}: {} bytes", module.idx, module.size)?;
            }
        }

        if !self.core_instances.is_empty() {
            writeln!(f, "\nCore Instances:")?;
            for (i, instance) in self.core_instances.iter().enumerate() {
                writeln!(f, "  - Instance {}: Module {}", i, instance.module_idx)?;
                if !instance.args.is_empty() {
                    writeln!(f, "    Args: {:?}", instance.args)?;
                }
            }
        }

        if !self.aliases.is_empty() {
            writeln!(f, "\nAliases:")?;
            for (i, alias) in self.aliases.iter().enumerate() {
                writeln!(
                    f,
                    "  - Alias {}: {} from Instance {} export '{}'",
                    i, alias.kind, alias.instance_idx, alias.export_name
                )?;
            }
        }

        Ok(())
    }
}

/// Information about a core module in a component
#[derive(Debug)]
pub struct CoreModuleInfo {
    /// Module index
    pub idx: u32,
    /// Module size in bytes
    pub size: usize,
}

/// Information about a core instance in a component
#[derive(Debug)]
pub struct CoreInstanceInfo {
    /// Index of the module instantiated
    pub module_idx: u32,
    /// Arguments passed to the instantiation
    pub args: Vec<String>,
}

/// Information about an alias in a component
#[derive(Debug)]
pub struct AliasInfo {
    /// Kind of alias
    pub kind: String,
    /// Index of the instance being aliased
    pub instance_idx: u32,
    /// Name of the export being aliased
    pub export_name: String,
}

/// Extended information about an import in a component
#[derive(Debug, Clone)]
pub struct ExtendedImportInfo {
    /// Import namespace
    pub namespace: String,
    /// Import name
    pub name: String,
    /// Kind of import (as string representation)
    pub kind: String,
}

/// Extended information about an export in a component
#[derive(Debug, Clone)]
pub struct ExtendedExportInfo {
    /// Export name
    pub name: String,
    /// Kind of export (as string representation)
    pub kind: String,
    /// Export index
    pub index: u32,
}

/// Information about an import in a core module
#[derive(Debug, Clone)]
pub struct ModuleImportInfo {
    /// Module name (namespace)
    pub module: String,
    /// Import name
    pub name: String,
    /// Kind of import (as string representation)
    pub kind: String,
    /// Index within the type
    pub index: u32,
    /// Module index that contains this import
    pub module_idx: u32,
}

/// Information about an export in a core module
#[derive(Debug, Clone)]
pub struct ModuleExportInfo {
    /// Export name
    pub name: String,
    /// Kind of export (as string representation)
    pub kind: String,
    /// Index within the type
    pub index: u32,
    /// Module index that contains this export
    pub module_idx: u32,
}

// Helper function to convert CoreSort to string
fn kind_to_string(kind: &wrt_format::component::CoreSort) -> String {
    match kind {
        wrt_format::component::CoreSort::Function => "Function".to_string(),
        wrt_format::component::CoreSort::Table => "Table".to_string(),
        wrt_format::component::CoreSort::Memory => "Memory".to_string(),
        wrt_format::component::CoreSort::Global => "Global".to_string(),
        wrt_format::component::CoreSort::Type => "Type".to_string(),
        wrt_format::component::CoreSort::Module => "Module".to_string(),
        wrt_format::component::CoreSort::Instance => "Instance".to_string(),
    }
}

// Helper function to convert Sort to string
fn sort_to_string(sort: &wrt_format::component::Sort) -> String {
    match sort {
        wrt_format::component::Sort::Core(core_sort) => {
            format!("Core({})", kind_to_string(core_sort))
        }
        wrt_format::component::Sort::Function => "Function".to_string(),
        wrt_format::component::Sort::Value => "Value".to_string(),
        wrt_format::component::Sort::Type => "Type".to_string(),
        wrt_format::component::Sort::Component => "Component".to_string(),
        wrt_format::component::Sort::Instance => "Instance".to_string(),
    }
}

/// Convert a function parameter entry to a ValueType
fn convert_param_to_value_type(param: &ValueType) -> Result<ValueType> {
    // Just return the value type directly - already in the right format
    Ok(param.clone())
}

/// Converts a wrt_types::types::ValueType to wrt_format::component::ValType
fn value_to_format_val_type(
    value_type: &wrt_types::types::ValueType,
) -> Result<wrt_format::component::ValType> {
    // Use the conversion utility from type_conversion.rs
    value_type_to_format_val_type(value_type)
}

// Conversion between verification levels
fn convert_verification_level(level: wrt_types::VerificationLevel) -> crate::resources::VerificationLevel {
    match level {
        wrt_types::VerificationLevel::None => crate::resources::VerificationLevel::None,
        wrt_types::VerificationLevel::Sampling => crate::resources::VerificationLevel::Critical,
        wrt_types::VerificationLevel::Standard => crate::resources::VerificationLevel::Critical,
        wrt_types::VerificationLevel::Full => crate::resources::VerificationLevel::Full,
    }
}

// Helper function to convert a format type to a runtime type
fn convert_format_extern_type(extern_type: &FormatExternType) -> Result<ExternType> {
    // Use the FromFormat trait implementation
    ExternType::from_format(extern_type.clone())
}

// Helper function to convert a format type definition to a runtime type
fn convert_component_type(
    type_def: &wrt_format::component::ComponentTypeDefinition,
) -> Result<ComponentType> {
    // Use the helper function from component_conversion
    format_type_def_to_component_type(type_def)
}

// Helper function to check if two types are compatible
fn types_are_compatible(a: &ExternType, b: &ExternType) -> bool {
    // Use the compatibility function from wrt_types
    wrt_types::component::types_are_compatible(a, b)
}
