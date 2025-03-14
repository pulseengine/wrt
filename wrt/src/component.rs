use crate::error::{Error, Result};
use crate::types::*;
use crate::values::Value;
use crate::{format, String, Vec};
use crate::{Global, Memory, Table};
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
    pub instances: Vec<InstanceType>,
}

/// Represents an instance type
#[derive(Debug)]
pub struct InstanceType {
    /// Instance exports
    pub exports: Vec<(String, ExternType)>,
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
#[derive(Debug)]
pub enum ExternValue {
    /// Function value
    Function(FunctionValue),
    /// Table value
    Table(TableValue),
    /// Memory value
    Memory(MemoryValue),
    /// Global value
    Global(GlobalValue),
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
#[derive(Debug)]
pub struct TableValue {
    /// Table type
    pub ty: TableType,
    /// Table instance
    pub table: Table,
}

/// Represents a memory value
#[derive(Debug)]
pub struct MemoryValue {
    /// Memory type
    pub ty: MemoryType,
    /// Memory instance
    pub memory: Memory,
}

/// Represents a global value
#[derive(Debug)]
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
    pub ty: InstanceType,
    /// Instance exports
    pub exports: Vec<Export>,
}

impl Component {
    /// Creates a new component instance
    pub fn new(component_type: ComponentType) -> Self {
        Self {
            component_type,
            exports: Vec::new(),
            imports: Vec::new(),
            instances: Vec::new(),
        }
    }

    /// Instantiates a component
    pub fn instantiate(&mut self, imports: Vec<Import>) -> Result<()> {
        // Validate imports
        if imports.len() != self.component_type.imports.len() {
            return Err(Error::Validation(format!(
                "Expected {} imports, got {}",
                self.component_type.imports.len(),
                imports.len()
            )));
        }

        // Check import types
        for ((expected_name, _expected_module, expected_type), import) in
            self.component_type.imports.iter().zip(imports.iter())
        {
            if import.name != *expected_name || import.ty != *expected_type {
                return Err(Error::Validation(format!(
                    "Import {} has invalid type",
                    import.name
                )));
            }
        }

        // Store imports
        self.imports = imports;

        // Initialize exports
        for (name, ty) in &self.component_type.exports {
            let value = match ty {
                ExternType::Function(func_type) => {
                    // Create a named function instead of a closure to avoid self capture
                    let export_name = name.clone();
                    ExternValue::Function(FunctionValue {
                        ty: func_type.clone(),
                        // Use an unbound function reference instead of a closure
                        export_name,
                    })
                }
                ExternType::Table(table_type) => ExternValue::Table(TableValue {
                    ty: table_type.clone(),
                    table: Table::new(table_type.clone()),
                }),
                ExternType::Memory(memory_type) => ExternValue::Memory(MemoryValue {
                    ty: memory_type.clone(),
                    memory: Memory::new(memory_type.clone()),
                }),
                ExternType::Global(global_type) => ExternValue::Global(GlobalValue {
                    ty: global_type.clone(),
                    global: Global::new(
                        global_type.clone(),
                        Value::default_for_type(&global_type.content_type),
                    )?,
                }),
            };

            self.exports.push(Export {
                name: name.clone(),
                ty: ty.clone(),
                value,
            });
        }

        Ok(())
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
