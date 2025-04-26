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
use log::{debug, error, info, trace, warn};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use wrt_error::{kinds, kinds::*, Error, Result};
use wrt_types::ComponentValue;

// Use direct imports from wrt-types
use wrt_types::{
    builtin::BuiltinType, ComponentType, ExportType, ExternType as TypesExternType, FuncType,
    GlobalType, InstanceType, Limits, MemoryType as TypesMemoryType, Namespace, ResourceType,
    TableType as TypesTableType, TypeIndex, ValueType,
};

// Use format types with explicit namespacing
use wrt_format::component::{
    ComponentSection, ComponentTypeDefinition as FormatComponentTypeDefinition,
    ComponentTypeSection, ExportSection, ExternType as FormatExternType, ImportSection,
    InstanceSection, ResourceOperation as FormatResourceOperation, ValType as FormatValType,
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
    common_to_format_val_type, extern_type_to_func_type, format_to_common_val_type,
    format_to_types_extern_type, format_val_type_to_types_valtype, format_val_type_to_value_type,
    types_to_format_extern_type, types_valtype_to_format_valtype, value_type_to_format_val_type,
};

use crate::debug_println;

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
    /// Built-in requirements
    pub(crate) built_in_requirements: Option<BuiltinRequirements>,
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
    /// Creates a new component with the given type
    pub fn new(component_type: ComponentType) -> Self {
        Self {
            component_type,
            exports: Vec::new(),
            imports: Vec::new(),
            instances: Vec::new(),
            linked_components: HashMap::new(),
            callback_registry: None,
            runtime: None,
            interceptor: None,
            resource_table: ResourceTable::new(),
            built_in_requirements: Some(BuiltinRequirements::new()),
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
                    debug_println!(&format!(
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
                        debug_println!(&format!(
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
                debug_println!(&format!("Executing function: {}", func_name));
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
            let format_extern_type =
                types_to_format_extern_type(&TypesExternType::Function(func_type.clone()))?;
            let extern_value = ExternValue::Function(host_func);

            // Use Export::new
            host_namespace.add_export(Export::new(
                func_name.clone(),
                format_extern_type,
                extern_value,
            ));

            // Register with callback registry if available
            if let Some(_registry) = &self.callback_registry {
                // Mark registry unused
                let _ = handler; // Avoid unused warning
                debug_println!(&format!(
                    "Skipping callback registration for {}/{} (needs CallbackType fix)",
                    api_name, func_name
                ));
            }
        }
        Ok(())
    }

    /// Instantiate a component from binary bytes
    pub fn instantiate_component(&mut self, bytes: &[u8], imports: Vec<Import>) -> Result<()> {
        use wrt_decoder::{Component as DecodedComponent, ComponentDecoder};

        // Parse the component binary
        let component = match ComponentDecoder::new().parse(bytes) {
            Ok(component) => component,
            Err(err) => {
                return Err(Error::new(format!(
                    "Failed to parse component binary: {}",
                    err
                )))
            }
        };

        // Detect built-in requirements before instantiation
        let requirements = scan_builtins(bytes)?;

        // Validate that all required built-ins are available in the current configuration
        if !requirements.is_supported() {
            let unsupported_builtins = requirements.get_unsupported();
            return Err(Error::new(format!(
                "Component requires unsupported built-ins: {:?}",
                unsupported_builtins
            )));
        }

        // If we have a callback registry, validate that it can satisfy all requirements
        if let Some(registry) = &self.callback_registry {
            let available_builtins = registry.get_available_builtins();
            let mut missing_builtins = vec![];

            for builtin in requirements.get_requirements() {
                if !available_builtins.contains(&builtin) {
                    missing_builtins.push(builtin);
                }
            }

            if !missing_builtins.is_empty() {
                return Err(Error::new(format!(
                    "Component requires built-ins not provided by the host: {:?}",
                    missing_builtins
                )));
            }
        }

        // Continue with the normal instantiation process...
        self.imports = imports;

        // Extract the component type
        let component_type = Self::extract_component_type(&component)?;
        self.component_type = component_type;

        // Instantiate the component
        self.instantiate(self.imports.clone())?;

        Ok(())
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
                    .map(|(name, val_type)| Ok(format_val_type_to_value_type(val_type)?))
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
                let resource_type = crate::type_conversion::format_resource_rep_to_resource_type(
                    representation,
                    *nullable,
                );
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
                debug_println!(&format!(
                    "Extracting exports from instantiated component {}",
                    component_idx
                ));

                // Process instantiation arguments which often carry import/export mappings
                for arg in args {
                    match arg {
                        // The actual enum variants may differ, so we'll use match guards instead
                        arg if Self::is_instance_arg(arg) => {
                            let (name, idx) = Self::get_instance_arg_info(arg);
                            debug_println!(&format!(
                                "Found instance arg {} -> instance {}",
                                name, idx
                            ));
                        }
                        arg if Self::is_core_arg(arg) => {
                            let (name, idx) = Self::get_core_arg_info(arg);
                            debug_println!(&format!(
                                "Found core arg {} -> core instance {}",
                                name, idx
                            ));
                        }
                        arg if Self::is_function_arg(arg) => {
                            let (name, idx) = Self::get_function_arg_info(arg);
                            debug_println!(&format!(
                                "Found function arg {} -> function {}",
                                name, idx
                            ));
                        }
                        // Add other argument types as needed
                        _ => {
                            debug_println!("Found other instantiate arg type");
                        }
                    }
                }

                exports
            }
            // Check for other instance expression types
            _ => {
                debug_println!("Unknown instance expression type");
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
                    integrity_hash: None,       // Ensure integrity_hash field exists
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
        self.resource_table
            .set_verification_level(handle, resources_level)
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
        debug_println!(&format!("Calling function index: {}", func_idx));
        let func_export = self.exports.get(func_idx as usize).ok_or_else(|| {
            Error::new(FunctionNotFoundError(format!(
                // Use specific error struct
                "Function export with index {} not found",
                func_idx
            )))
        })?;

        let (format_param_types, format_result_types, func_name) = match &func_export.ty {
            FormatExternType::Function { params, results } => (params, results, &func_export.name),
            _ => {
                return Err(Error::new(TypeMismatchError(format!(
                    // Use specific error struct
                    "Export at index {} ('{}') is not a function",
                    func_idx, func_export.name
                ))));
            }
        };
        let format_param_types = format_param_types.clone();
        let format_result_types = format_result_types.clone();
        debug_println!(&format!(
            "Found function export '{}' with type: {:?} -> {:?}",
            func_name, format_param_types, format_result_types
        ));

        let arg_types: Vec<FormatValType> = args
            .iter()
            .map(|arg| common_to_format_val_type(&arg.get_type()))
            .collect::<Result<Vec<_>>>()?;

        // Fix E0277: Compare Vec<FormatValType> with Vec<FormatValType>
        let format_param_valtypes: Vec<_> = format_param_types
            .iter()
            .map(|(_, ty)| ty)
            .cloned()
            .collect();
        if arg_types != format_param_valtypes {
            return Err(Error::new(TypeMismatchError(format!(
                // Use specific error struct
                "Argument type mismatch for function '{}': expected {:?}, got {:?}",
                func_name, format_param_valtypes, arg_types
            ))));
        }
        debug_println!("Argument types match expected parameter types.");

        let data = serialize_component_values(&args)?;
        debug_println!(&format!("Serialized args ({} bytes)", data.len()));

        let _data_to_call = if let Some(_interceptor) = self.get_interceptor() {
            // Mark interceptor unused for now
            debug_println!("Skipping lower interception..."); // Skip interception logic
            data
        } else {
            data
        };

        let core_args: Vec<Value> = args
            .into_iter()
            .map(|cv| component_value_to_value(&cv))
            .collect::<Result<Vec<_>>>()?;
        debug_println!(&format!(
            "Converted {} ComponentValues to Core Values",
            core_args.len()
        ));

        debug_println!(&format!(
            "Executing resolved function '{}' (idx {})",
            func_name, func_idx
        ));
        let core_results = self.call_resolved_function(func_idx, func_name, core_args)?;
        debug_println!(&format!(
            "Resolved function returned {} core results",
            core_results.len()
        ));

        let final_core_results = if let Some(_interceptor) = self.get_interceptor() {
            // Mark interceptor unused
            debug_println!("Skipping lift interception..."); // Skip interception logic
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
        debug_println!(&format!(
            "Converted {} Core Values back to ComponentValues",
            component_results.len()
        ));

        Ok(component_results)
    }

    fn call_resolved_function(
        &self,
        _func_idx: u32,
        func_name: &str,
        args: Vec<Value>,
    ) -> Result<Vec<Value>> {
        if let Some(runtime) = &self.runtime {
            debug_println!(&format!(
                "Delegating call '{}' to runtime instance",
                func_name
            ));
            runtime.execute_function(func_name, args)
        } else if let Some(_registry) = &self.callback_registry {
            // Mark registry unused
            debug_println!(&format!(
                "Delegating call '{}' to host callback registry - SKIPPED (Needs fix)",
                func_name
            ));
            Err(Error::new(NotImplementedError(
                // Use specific error struct
                "CallbackRegistry interaction needs correct method call".to_string(),
            )))
        } else {
            error!(
                "Cannot call function '{}': No runtime or host registry configured",
                func_name
            );
            Err(Error::new(ExecutionError(format!(
                // Use specific error struct
                "Cannot call function '{}': No runtime or host registry configured",
                func_name
            ))))
        }
    }

    /// Extract component type information from a decoded component
    ///
    /// # Arguments
    ///
    /// * `component` - The decoded component
    ///
    /// # Returns
    ///
    /// A `Result` containing the extracted `ComponentType` or an error
    fn extract_component_type(component: &wrt_decoder::Component) -> Result<ComponentType> {
        let mut component_type = ComponentType::new();

        // Process imports from the decoded component
        for import in component.import_section() {
            let namespace = import.module.clone();
            let name = import.name.clone();

            // Convert the import type to our internal type representation
            let extern_type = match &import.import_type {
                wrt_decoder::ImportType::Func(_) => TypesExternType::Func(FuncType::default()),
                wrt_decoder::ImportType::Table(_) => {
                    TypesExternType::Table(TypesTableType::default())
                }
                wrt_decoder::ImportType::Memory(_) => {
                    TypesExternType::Memory(TypesMemoryType::default())
                }
                wrt_decoder::ImportType::Global(_) => {
                    TypesExternType::Global(GlobalType::default())
                }
                wrt_decoder::ImportType::Tag(_) => {
                    return Err(Error::new(kinds::ValidationError(
                        "Tag imports are not supported".to_string(),
                    )));
                }
            };

            component_type.imports.push((name, namespace, extern_type));
        }

        // Process exports from the decoded component
        for export in component.export_section() {
            let name = export.name.clone();

            // Convert the export type to our internal type representation
            let extern_type = match export.export_type {
                wrt_decoder::ExportType::Func(_) => TypesExternType::Func(FuncType::default()),
                wrt_decoder::ExportType::Table(_) => {
                    TypesExternType::Table(TypesTableType::default())
                }
                wrt_decoder::ExportType::Memory(_) => {
                    TypesExternType::Memory(TypesMemoryType::default())
                }
                wrt_decoder::ExportType::Global(_) => {
                    TypesExternType::Global(GlobalType::default())
                }
                wrt_decoder::ExportType::Tag(_) => {
                    return Err(Error::new(kinds::ValidationError(
                        "Tag exports are not supported".to_string(),
                    )));
                }
            };

            component_type.exports.push((name, extern_type));
        }

        // Process component instances
        for instance in component.instance_section() {
            // For now, create a placeholder type definition
            let instance_type =
                FormatComponentTypeDefinition::Instance(wrt_format::component::InstanceType {
                    exports: Vec::new(),
                });
            component_type.instances.push(instance_type);
        }

        Ok(component_type)
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

/// A struct to track the built-in requirements of a component
#[derive(Debug, Clone)]
pub struct BuiltinRequirements {
    /// The set of built-in types required by this component
    requirements: HashSet<BuiltinType>,
}

impl BuiltinRequirements {
    /// Create a new, empty set of requirements
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

impl Default for BuiltinRequirements {
    fn default() -> Self {
        Self::new()
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
    let component = match wrt_decoder::ComponentDecoder::new().parse(bytes) {
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
    // Parse the module
    let module = match wasmparser::Parser::new(0).parse_all(module) {
        Ok(module) => module,
        Err(err) => {
            return Err(Error::new(kinds::DecodingError(format!(
                "Failed to parse embedded module during built-in scan: {}",
                err
            ))));
        }
    };

    // Scan for imports from the "wasi_builtin" module
    for payload in module {
        if let Ok(wasmparser::Payload::ImportSection(imports)) = payload {
            for import in imports {
                match import {
                    Ok(import) => {
                        if import.module == "wasi_builtin" {
                            // Map the import name to the corresponding built-in type
                            if let Some(builtin_type) = map_import_to_builtin(import.name) {
                                requirements.add_requirement(builtin_type);
                            }
                        }
                    }
                    Err(err) => {
                        return Err(Error::new(kinds::DecodingError(format!(
                            "Failed to parse import during built-in scan: {}",
                            err
                        ))));
                    }
                }
            }
        }
    }

    Ok(())
}

/// Map an import name to a built-in type
fn map_import_to_builtin(import_name: &str) -> Option<BuiltinType> {
    match import_name {
        "http_fetch" => Some(BuiltinType::Http),
        "random_get_bytes" => Some(BuiltinType::Random),
        "clock_now" => Some(BuiltinType::Clock),
        "stdin_read" => Some(BuiltinType::Stdin),
        "stdout_write" => Some(BuiltinType::Stdout),
        "stderr_write" => Some(BuiltinType::Stderr),
        "fs_read_file" | "fs_write_file" => Some(BuiltinType::FileSystem),
        _ => None,
    }
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
    component: &wrt_decoder::Component,
    requirements: &mut BuiltinRequirements,
) -> Result<()> {
    // Check for resource types which indicate built-in usage
    for type_def in component.types() {
        if let wrt_decoder::TypeDef::Resource(_) = type_def {
            // Resources typically require the Resource built-in
            requirements.add_requirement(BuiltinType::Resource);
        }
    }

    // Check for async functions which require the Future built-in
    for func in component.functions() {
        if func.is_async() {
            requirements.add_requirement(BuiltinType::Future);
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
    let component = match wrt_decoder::ComponentDecoder::new().parse(bytes) {
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
    use wrt_intercept::Value;
    use wrt_types::ComponentValue;

    match component_value {
        ComponentValue::Bool(v) => Value::I32(if *v { 1 } else { 0 }),
        ComponentValue::S8(v) => Value::I32(*v as i32),
        ComponentValue::U8(v) => Value::I32(*v as i32),
        ComponentValue::S16(v) => Value::I32(*v as i32),
        ComponentValue::U16(v) => Value::I32(*v as i32),
        ComponentValue::S32(v) => Value::I32(*v),
        ComponentValue::U32(v) => Value::I32(*v as i32),
        ComponentValue::S64(v) => Value::I64(*v),
        ComponentValue::U64(v) => Value::I64(*v as i64),
        ComponentValue::F32(v) => Value::F32(*v),
        ComponentValue::F64(v) => Value::F64(*v),
        ComponentValue::Handle(v) => Value::I32(*v as i32),
        ComponentValue::Borrow(v) => Value::I32(*v as i32),
        // For complex types, we need to serialize them properly
        // This is a simplified implementation
        _ => Value::I32(0),
    }
}

/// Convert a runtime value to a component value
pub fn value_to_component_value(value: &wrt_intercept::Value) -> wrt_types::ComponentValue {
    use wrt_intercept::Value;
    use wrt_types::ComponentValue;

    match value {
        Value::I32(v) => ComponentValue::S32(*v),
        Value::I64(v) => ComponentValue::S64(*v),
        Value::F32(v) => ComponentValue::F32(*v),
        Value::F64(v) => ComponentValue::F64(*v),
        Value::ExternRef(_) => ComponentValue::Void,
        Value::FuncRef(_) => ComponentValue::Void,
        Value::V128(_) => ComponentValue::Void,
    }
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
