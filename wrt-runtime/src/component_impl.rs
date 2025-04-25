use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wrt_error::{Error, Result};
use wrt_types::{ComponentType, ExternType, Vec};

// Import our local function type (not the component one)
use crate::func::FuncType;
use wrt_types::values::Value;

use crate::component_traits::{
    ComponentInstance, ComponentRuntime, HostFunction, HostFunctionFactory,
};

/// Type alias for function implementations
type HostFunctionImplementation = Arc<dyn Fn(&[Value]) -> Result<Vec<Value>> + Send + Sync>;

/// A host function implementation
pub struct HostFunctionImpl {
    /// Function type
    func_type: FuncType,
    /// Function implementation
    implementation: HostFunctionImplementation,
}

impl HostFunction for HostFunctionImpl {
    fn call(&self, args: &[Value]) -> Result<Vec<Value>> {
        (self.implementation)(args)
    }

    fn get_type(&self) -> FuncType {
        self.func_type.clone()
    }
}

/// A concrete implementation of a component instance
pub struct ComponentInstanceImpl {
    /// Component type
    component_type: ComponentType,
    /// Host functions
    host_functions: HashMap<String, Arc<dyn HostFunction + Send + Sync>>,
    /// Memory
    memory: HashMap<String, Vec<u8>>,
}

impl ComponentInstance for ComponentInstanceImpl {
    fn execute_function(&self, name: &str, args: &[Value]) -> Result<Vec<Value>> {
        // Try to find a host function with this name
        if let Some(host_func) = self.host_functions.get(name) {
            return host_func.call(args);
        }

        // Look for the function in the component type
        if let Some((_, ty)) = self.component_type.exports.iter().find(|(n, _)| n == name) {
            match ty {
                ExternType::Function(func_type) => {
                    // Validate argument count
                    if args.len() != func_type.params.len() {
                        return Err(Error::new(format!(
                            "Expected {} arguments, got {}",
                            func_type.params.len(),
                            args.len()
                        )));
                    }

                    // This is a placeholder implementation - in a real system, this would
                    // execute the actual component function
                    Ok(vec![Value::I32(42)])
                }
                _ => Err(Error::new(format!("Export {name} is not a function"))),
            }
        } else {
            Err(Error::new(format!("Function {name} not found")))
        }
    }

    fn read_memory(&self, name: &str, offset: u32, size: u32) -> Result<Vec<u8>> {
        if let Some(mem) = self.memory.get(name) {
            let start = offset as usize;
            let end = start + size as usize;

            if end <= mem.len() {
                Ok(mem[start..end].to_vec())
            } else {
                Err(Error::new("Memory access out of bounds".to_string()))
            }
        } else {
            Err(Error::new(format!("Memory {name} not found")))
        }
    }

    fn write_memory(&mut self, name: &str, offset: u32, bytes: &[u8]) -> Result<()> {
        let memory = self.memory.entry(name.to_string()).or_default();

        let start = offset as usize;
        let end = start + bytes.len();

        // Ensure memory is large enough
        if memory.len() < end {
            memory.resize(end, 0);
        }

        // Write the bytes
        memory[start..end].copy_from_slice(bytes);

        Ok(())
    }

    fn get_export_type(&self, name: &str) -> Result<ExternType> {
        // Find the export in the component type
        if let Some((_, ty)) = self.component_type.exports.iter().find(|(n, _)| n == name) {
            Ok(ty.clone())
        } else {
            Err(Error::new(format!("Export {name} not found")))
        }
    }
}

/// A concrete implementation of the component runtime
pub struct ComponentRuntimeImpl {
    /// Host function factories
    host_factories: Vec<Box<dyn HostFunctionFactory>>,
    /// Registered host functions
    host_functions: HashMap<String, Arc<dyn HostFunction + Send + Sync>>,
}

impl ComponentRuntime for ComponentRuntimeImpl {
    fn new() -> Self {
        Self {
            host_factories: Vec::new(),
            host_functions: HashMap::new(),
        }
    }

    fn register_host_factory(&mut self, factory: Box<dyn HostFunctionFactory>) {
        self.host_factories.push(factory);
    }

    fn instantiate(&self, component_type: &ComponentType) -> Result<Box<dyn ComponentInstance>> {
        // Create a new component instance
        let mut instance = ComponentInstanceImpl {
            component_type: component_type.clone(),
            host_functions: HashMap::new(),
            memory: HashMap::new(),
        };

        // Register host functions
        for (name, func) in &self.host_functions {
            instance
                .host_functions
                .insert(name.clone(), Arc::clone(func));
        }

        Ok(Box::new(instance))
    }

    fn register_host_function<F>(&mut self, name: &str, ty: FuncType, function: F) -> Result<()>
    where
        F: Fn(&[Value]) -> Result<Vec<Value>> + 'static + Send + Sync,
    {
        let host_func = HostFunctionImpl {
            func_type: ty,
            implementation: Arc::new(function),
        };

        self.host_functions
            .insert(name.to_string(), Arc::new(host_func));

        Ok(())
    }
}

/// A default host function factory for standard functions
pub struct DefaultHostFunctionFactory {
    /// Function implementations
    functions: HashMap<String, (FuncType, HostFunctionImplementation)>,
}

impl Default for DefaultHostFunctionFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultHostFunctionFactory {
    /// Create a new default host function factory
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    /// Register a function
    pub fn register_function<F>(&mut self, name: &str, ty: FuncType, function: F)
    where
        F: Fn(&[Value]) -> Result<Vec<Value>> + 'static + Send + Sync,
    {
        self.functions
            .insert(name.to_string(), (ty, Arc::new(function)));
    }
}

impl HostFunctionFactory for DefaultHostFunctionFactory {
    fn create_function(&self, name: &str, ty: &FuncType) -> Result<Box<dyn HostFunction>> {
        if let Some((func_type, implementation)) = self.functions.get(name) {
            // Convert our FuncType to a simpler compatibility check
            // Skip the conversion to component FuncType as they use different ValueType enums
            if func_type.params.len() == ty.params.len()
                && func_type.results.len() == ty.results.len()
            {
                // Simple parameter count check for compatibility
                let host_func = HostFunctionImpl {
                    func_type: func_type.clone(),
                    implementation: Arc::clone(implementation),
                };

                Ok(Box::new(host_func))
            } else {
                Err(Error::new(format!("Function {name} has incompatible type")))
            }
        } else {
            Err(Error::new(format!("Function {name} not found")))
        }
    }
}
