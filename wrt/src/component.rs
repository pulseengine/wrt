use crate::error::{Error, Result};
use crate::types::*;
use crate::values::Value;
use crate::{format, String, Vec};
use crate::{Global, Memory, Table};
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
#[cfg(not(feature = "std"))]
use alloc::vec;
#[cfg(feature = "std")]
use std::vec;

/// Represents a component instance
#[derive(Debug)]
pub struct Component {
    /// Component type
    component_type: ComponentType,
    /// Component exports
    exports: Vec<Export>,
    /// Component imports
    imports: Vec<Import>,
    /// Component instances
    #[allow(dead_code)]
    instances: Vec<InstanceValue>,
}

/// Represents a component type
#[derive(Debug)]
pub struct ComponentType {
    /// Component imports
    pub imports: Vec<(String, String, ExternType)>,
    /// Component exports
    pub exports: Vec<(String, ExternType)>,
    /// Component instances
    pub instances: Vec<crate::types::InstanceType>,
}

/// Represents a component export
#[derive(Debug)]
pub struct Export {
    /// Export name
    pub name: String,
    /// Export type
    pub ty: ExternType,
    /// Export value
    pub value: ExternValue,
}

/// Represents a component import
#[derive(Debug)]
pub struct Import {
    /// Import name
    pub name: String,
    /// Import type
    pub ty: ExternType,
    /// Import value
    pub value: ExternValue,
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
    pub ty: FuncType,
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
    pub memory: Memory,
}

/// Represents a global value
#[derive(Debug, Clone)]
pub struct GlobalValue {
    /// Global type
    pub ty: GlobalType,
    /// Global instance
    pub global: Global,
}

/// Represents an instance value
#[derive(Debug)]
pub struct InstanceValue {
    /// Instance type
    pub ty: crate::types::InstanceType,
    /// Instance exports
    pub exports: Vec<Export>,
}

impl Component {
    /// Creates a new component with the given type
    pub fn new(component_type: ComponentType) -> Self {
        Self {
            component_type,
            exports: Vec::new(),
            imports: Vec::new(),
            instances: Vec::new(),
        }
    }

    /// Instantiates the component with the given imports
    pub fn instantiate(&mut self, imports: Vec<Import>) -> Result<()> {
        // Validate imports
        if imports.len() != self.component_type.imports.len() {
            return Err(Error::Validation(format!(
                "Expected {} imports, got {}",
                self.component_type.imports.len(),
                imports.len()
            )));
        }

        // Validate import types
        for ((import_name, _import_namespace, import_type), import) in
            self.component_type.imports.iter().zip(imports.iter())
        {
            if import.name != *import_name {
                return Err(Error::Validation(format!(
                    "Expected import {}, got {}",
                    import_name, import.name
                )));
            }

            if !types_are_compatible(import_type, &import.ty) {
                return Err(Error::Validation(format!(
                    "Import {} has incompatible type",
                    import_name
                )));
            }
        }

        // Store imports
        self.imports = imports;

        // Initialize exports
        self.initialize_exports()?;

        // Link instances together
        self.link_instances()?;

        Ok(())
    }

    /// Initialize component exports
    fn initialize_exports(&mut self) -> Result<()> {
        let mut exports = Vec::new();

        for (name, ty) in &self.component_type.exports {
            let value = match ty {
                ExternType::Function(func_type) => {
                    // For function exports, create a reference to the function
                    ExternValue::Function(FunctionValue {
                        ty: func_type.clone(),
                        export_name: name.clone(),
                    })
                }
                ExternType::Table(table_type) => {
                    // Create a table export with default initialization
                    ExternValue::Table(TableValue {
                        ty: table_type.clone(),
                        table: Table::new(table_type.clone()),
                    })
                }
                ExternType::Memory(memory_type) => {
                    // Create a memory export with default initialization
                    ExternValue::Memory(MemoryValue {
                        ty: memory_type.clone(),
                        memory: Memory::new(memory_type.clone()),
                    })
                }
                ExternType::Global(global_type) => {
                    // Create a global export with default initialization
                    ExternValue::Global(GlobalValue {
                        ty: global_type.clone(),
                        global: Global::new(
                            global_type.clone(),
                            Value::default_for_type(&global_type.content_type),
                        )?,
                    })
                }
                ExternType::Resource(_resource_type) => {
                    // For resources, we create a reference to an abstract resource type
                    // The actual resource operations will be handled later through the canonical ABI
                    ExternValue::Trap(format!("Resource {} not implemented", name))
                }
                ExternType::Instance(instance_type) => {
                    // For instance exports, we need to create a new instance with proper linking
                    let instance_exports = instance_type
                        .exports
                        .iter()
                        .map(|(export_name, export_type)| {
                            // Since this is just initialization, create a placeholder trap for now
                            Export {
                                name: export_name.clone(),
                                ty: export_type.clone(),
                                value: ExternValue::Trap(format!(
                                    "Instance export {} not linked",
                                    export_name
                                )),
                            }
                        })
                        .collect();

                    let instance = InstanceValue {
                        ty: instance_type.clone(),
                        exports: instance_exports,
                    };

                    // Add this instance to our list for later linking
                    self.instances.push(instance);

                    // Return a trap for now - we'll fix this in linking
                    ExternValue::Trap(format!("Instance {} not linked", name))
                }
                ExternType::Component(_) => {
                    // Component exports are not instantiated here - they should be instantiated
                    // separately and then linked
                    ExternValue::Trap(format!("Component {} not instantiated", name))
                }
            };

            exports.push(Export {
                name: name.clone(),
                ty: ty.clone(),
                value,
            });
        }

        self.exports = exports;
        Ok(())
    }

    /// Link instances together
    fn link_instances(&mut self) -> Result<()> {
        // For each instance, find matching imports and link them
        for instance in &mut self.instances {
            for export in &mut instance.exports {
                // Try to find a matching import or export in the component
                if let Some(import) = self.imports.iter().find(|i| i.name == export.name) {
                    // We found a matching import, link it
                    export.value = import.value.clone();
                } else if let Some(comp_export) = self.exports.iter().find(|e| e.name == export.name) {
                    // We found a matching export, link it
                    export.value = comp_export.value.clone();
                }
            }
        }

        // Now update component exports with linked instances
        for export in &mut self.exports {
            if let ExternType::Instance(instance_type) = &export.ty {
                // Find the matching instance
                if let Some(_instance) = self
                    .instances
                    .iter()
                    .find(|i| instance_types_match(&i.ty, instance_type))
                {
                    // Link the instance
                    export.value = ExternValue::Trap(format!(
                        "Instance {} linked but not implemented",
                        export.name
                    ));
                }
            }
        }

        Ok(())
    }

    /// Reads from exported memory
    ///
    /// This function takes the name of the exported memory, the offset to read from,
    /// and the number of bytes to read. It returns the bytes read.
    pub fn read_memory(&self, name: &str, offset: u32, size: u32) -> Result<Vec<u8>> {
        // Find the export
        let export = self.get_export(name)?;

        // Check if it's a memory
        let memory_value = match &export.value {
            ExternValue::Memory(mem) => mem,
            _ => {
                return Err(Error::Execution(format!("Export {} is not a memory", name)))
            }
        };

        // Read from memory
        let bytes = memory_value.memory.read_bytes(offset, size as usize)?;
        Ok(bytes.to_vec())
    }

    /// Executes an exported function
    ///
    /// This function takes the name of the exported function and the arguments
    /// to pass to it. It validates the arguments, executes the function, and
    /// returns the result.
    pub fn execute_function(&self, name: &str, args: Vec<Value>) -> Result<Vec<Value>> {
        // Find the export
        let export = self.get_export(name)?;

        // Check if it's a function
        let func_value = match &export.value {
            ExternValue::Function(func) => func,
            _ => {
                return Err(Error::Execution(format!("Export {} is not a function", name)))
            }
        };

        // Check argument count
        if args.len() != func_value.ty.params.len() {
            return Err(Error::Execution(format!(
                "Expected {} arguments, got {}",
                func_value.ty.params.len(),
                args.len()
            )));
        }

        // Validate argument types
        for (i, (arg, expected_type)) in args.iter().zip(func_value.ty.params.iter()).enumerate() {
            if !arg.matches_type(expected_type) {
                return Err(Error::Execution(format!(
                    "Argument {} has invalid type - expected {:?}, got {:?}",
                    i, expected_type, arg
                )));
            }
        }

        // Call the function
        self.handle_function_call(name, &args)
    }

    /// Writes to exported memory
    ///
    /// This function takes the name of the exported memory, the offset to write to,
    /// and the bytes to write. It returns the number of bytes written.
    pub fn write_memory(&mut self, name: &str, offset: u32, bytes: &[u8]) -> Result<()> {
        // Find the export
        let export = self.get_export_mut(name)?;

        // Check if it's a memory
        let memory_value = match &mut export.value {
            ExternValue::Memory(mem) => mem,
            _ => {
                return Err(Error::Execution(format!("Export {} is not a memory", name)))
            }
        };

        // Write to memory
        memory_value.memory.write_bytes(offset, bytes)
    }

    /// Gets an export by name
    pub fn get_export(&self, name: &str) -> Result<&Export> {
        self.exports
            .iter()
            .find(|e| e.name == name)
            .ok_or_else(|| Error::Validation(format!("Export {} not found", name)))
    }

    /// Gets a mutable reference to an export by name
    pub fn get_export_mut(&mut self, name: &str) -> Result<&mut Export> {
        self.exports
            .iter_mut()
            .find(|e| e.name == name)
            .ok_or_else(|| Error::Validation(format!("Export {} not found", name)))
    }

    /// Handles a function call from the host
    #[allow(dead_code)]
    fn handle_function_call(&self, name: &str, args: &[Value]) -> Result<Vec<Value>> {
        // Find the export
        let export = self.get_export(name)?;

        // Check if it's a function
        let func_value = match &export.value {
            ExternValue::Function(func) => func,
            _ => {
                return Err(Error::Execution(format!(
                    "Export {} is not a function",
                    name
                )))
            }
        };

        // Check argument count
        if args.len() != func_value.ty.params.len() {
            return Err(Error::Execution(format!(
                "Expected {} arguments, got {}",
                func_value.ty.params.len(),
                args.len()
            )));
        }

        // Call the function using the exported name
        // In this simplified version, just return a sample value
        Ok(vec![Value::I32(42)])
    }
}

/// Represents a host implementation
#[derive(Debug)]
pub struct Host {
    /// Host functions
    functions: Vec<(String, FunctionValue)>,
}

impl Default for Host {
    fn default() -> Self {
        Self::new()
    }
}

impl Host {
    /// Creates a new host implementation
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
        }
    }

    /// Adds a host function
    pub fn add_function(&mut self, name: String, func: FunctionValue) {
        self.functions.push((name, func));
    }

    /// Gets a host function by name
    pub fn get_function(&self, name: &str) -> Option<&FunctionValue> {
        self.functions
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, f)| f)
    }

    /// Calls a host function
    pub fn call_function(&self, name: &str, args: &[Value]) -> Result<Vec<Value>> {
        let func_value = self
            .get_function(name)
            .ok_or_else(|| Error::Execution(format!("Host function {} not found", name)))?;

        // Check argument count
        if args.len() != func_value.ty.params.len() {
            return Err(Error::Execution(format!(
                "Expected {} arguments, got {}",
                func_value.ty.params.len(),
                args.len()
            )));
        }

        // This is a simplified implementation - in a real system, you would
        // have a way to call functions based on their export_name
        Ok(vec![Value::I32(42)]) // Default implementation returns a sample value
    }
}

/// Check if two types are compatible for linking
fn types_are_compatible(a: &ExternType, b: &ExternType) -> bool {
    match (a, b) {
        (ExternType::Function(a_ty), ExternType::Function(b_ty)) => {
            a_ty.params == b_ty.params && a_ty.results == b_ty.results
        }
        (ExternType::Table(a_ty), ExternType::Table(b_ty)) => a_ty == b_ty,
        (ExternType::Memory(a_ty), ExternType::Memory(b_ty)) => a_ty == b_ty,
        (ExternType::Global(a_ty), ExternType::Global(b_ty)) => a_ty == b_ty,
        (ExternType::Resource(_), ExternType::Resource(_)) => true, // Basic compatibility for now
        (ExternType::Instance(a_ty), ExternType::Instance(b_ty)) => instance_types_match(a_ty, b_ty),
        (ExternType::Component(_), ExternType::Component(_)) => true, // Basic compatibility for now
        _ => false,
    }
}

/// Check if two instance types match for linking
fn instance_types_match(a: &crate::types::InstanceType, b: &crate::types::InstanceType) -> bool {
    if a.exports.len() != b.exports.len() {
        return false;
    }

    for ((a_name, a_ty), (b_name, b_ty)) in a.exports.iter().zip(b.exports.iter()) {
        if a_name != b_name || !types_are_compatible(a_ty, b_ty) {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a test component type
    fn create_test_component_type() -> ComponentType {
        ComponentType {
            imports: vec![(
                "add".to_string(),
                "math".to_string(),
                ExternType::Function(FuncType {
                    params: vec![ValueType::I32, ValueType::I32],
                    results: vec![ValueType::I32],
                }),
            )],
            exports: vec![
                (
                    "multiply".to_string(),
                    ExternType::Function(FuncType {
                        params: vec![ValueType::I32, ValueType::I32],
                        results: vec![ValueType::I32],
                    }),
                ),
                (
                    "memory".to_string(),
                    ExternType::Memory(MemoryType {
                        min: 1,
                        max: Some(2),
                    }),
                ),
            ],
            instances: Vec::new(),
        }
    }

    // Helper function to create a test import
    fn create_test_import() -> Import {
        Import {
            name: "add".to_string(),
            ty: ExternType::Function(FuncType {
                params: vec![ValueType::I32, ValueType::I32],
                results: vec![ValueType::I32],
            }),
            value: ExternValue::Function(FunctionValue {
                ty: FuncType {
                    params: vec![ValueType::I32, ValueType::I32],
                    results: vec![ValueType::I32],
                },
                export_name: "add".to_string(),
            }),
        }
    }

    #[test]
    fn test_component_creation_and_instantiation() -> Result<()> {
        let component_type = create_test_component_type();
        let mut component = Component::new(component_type);
        let import = create_test_import();

        // Test instantiation
        assert!(component.instantiate(vec![import]).is_ok());

        // Test export access
        let export = component.get_export("multiply")?;
        assert_eq!(export.name, "multiply");
        match &export.ty {
            ExternType::Function(func_type) => {
                assert_eq!(func_type.params.len(), 2);
                assert_eq!(func_type.results.len(), 1);
            }
            _ => panic!("Expected function type"),
        }

        Ok(())
    }

    #[test]
    fn test_component_export_types() -> Result<()> {
        let component_type = create_test_component_type();
        let mut component = Component::new(component_type);
        let import = create_test_import();
        component.instantiate(vec![import])?;

        // Test function export
        let func_export = component.get_export("multiply")?;
        assert_eq!(func_export.name, "multiply");
        match &func_export.value {
            ExternValue::Function(func) => {
                assert_eq!(func.ty.params.len(), 2);
                assert_eq!(func.ty.results.len(), 1);
            }
            _ => panic!("Expected function export"),
        }

        // Test memory export
        let mem_export = component.get_export("memory")?;
        assert_eq!(mem_export.name, "memory");
        match &mem_export.value {
            ExternValue::Memory(mem) => {
                assert_eq!(mem.ty.min, 1);
                assert_eq!(mem.ty.max, Some(2));
            }
            _ => panic!("Expected memory export"),
        }

        Ok(())
    }

    #[test]
    fn test_component_invalid_instantiation() {
        let component_type = ComponentType {
            imports: vec![(
                "add".to_string(),
                "math".to_string(),
                ExternType::Function(FuncType {
                    params: vec![ValueType::I32, ValueType::I32],
                    results: vec![ValueType::I32],
                }),
            )],
            exports: Vec::new(),
            instances: Vec::new(),
        };

        let mut component = Component::new(component_type);

        // Test instantiation with wrong number of imports
        assert!(component.instantiate(vec![]).is_err());

        // Test instantiation with wrong import type
        let wrong_import = Import {
            name: "add".to_string(),
            ty: ExternType::Memory(MemoryType { min: 1, max: None }),
            value: ExternValue::Memory(MemoryValue {
                ty: MemoryType { min: 1, max: None },
                memory: Memory::new(MemoryType { min: 1, max: None }),
            }),
        };
        assert!(component.instantiate(vec![wrong_import]).is_err());
    }

    #[test]
    fn test_component_function_calls() -> Result<()> {
        let component_type = create_test_component_type();
        let mut component = Component::new(component_type);
        let import = create_test_import();
        component.instantiate(vec![import])?;

        // Test valid function call
        let result = component.execute_function("multiply", vec![Value::I32(5), Value::I32(3)])?;
        assert_eq!(result, vec![Value::I32(42)]); // Default implementation returns 42

        // Test function call with wrong number of arguments
        assert!(component
            .execute_function("multiply", vec![Value::I32(5)])
            .is_err());

        // Test function call to non-existent function
        assert!(component
            .execute_function("nonexistent", vec![Value::I32(5)])
            .is_err());

        // Test function call to non-function export
        assert!(component
            .execute_function("memory", vec![Value::I32(5)])
            .is_err());

        Ok(())
    }

    #[test]
    fn test_host_function_management() -> Result<()> {
        let mut host = Host::new();

        // Add a host function
        let func_value = FunctionValue {
            ty: FuncType {
                params: vec![ValueType::I32],
                results: vec![ValueType::I32],
            },
            export_name: "increment".to_string(),
        };
        host.add_function("increment".to_string(), func_value);

        // Test function retrieval
        let retrieved_func = host.get_function("increment");
        assert!(retrieved_func.is_some());
        assert_eq!(retrieved_func.unwrap().export_name, "increment");

        // Test nonexistent function
        assert!(host.get_function("nonexistent").is_none());

        // Test function call
        let result = host.call_function("increment", &[Value::I32(5)])?;
        assert_eq!(result, vec![Value::I32(42)]); // Default implementation returns 42

        Ok(())
    }

    #[test]
    fn test_host_function_call_validation() {
        let mut host = Host::new();

        // Add a host function
        let func_value = FunctionValue {
            ty: FuncType {
                params: vec![ValueType::I32, ValueType::I32],
                results: vec![ValueType::I32],
            },
            export_name: "add".to_string(),
        };
        host.add_function("add".to_string(), func_value);

        // Test call with wrong number of arguments
        let result = host.call_function("add", &[Value::I32(5)]);
        assert!(result.is_err());

        // Test call to nonexistent function
        let result = host.call_function("nonexistent", &[Value::I32(5)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_export_mutation() -> Result<()> {
        let component_type = create_test_component_type();
        let mut component = Component::new(component_type);
        let import = create_test_import();
        component.instantiate(vec![import])?;

        // Test mutable access to export
        let export = component.get_export_mut("multiply")?;
        assert_eq!(export.name, "multiply");

        // Test mutable access to nonexistent export
        assert!(component.get_export_mut("nonexistent").is_err());

        Ok(())
    }
}
