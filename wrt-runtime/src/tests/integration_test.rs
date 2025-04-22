#[cfg(test)]
mod tests {
    use wrt_error::Result;
    use wrt_types::{ComponentType, ExternType, FuncType, Value, ValueType};
    
    use crate::{ComponentRuntime, ComponentRuntimeImpl};

    #[test]
    fn test_basic_component_instantiation() -> Result<()> {
        // Create a simple component type with a function export
        let component_type = ComponentType {
            imports: Vec::new(),
            exports: vec![(
                "hello".to_string(),
                ExternType::Function(FuncType {
                    params: Vec::new(),
                    results: vec![ValueType::I32],
                }),
            )],
            instances: Vec::new(),
        };

        // Create a runtime
        let runtime = ComponentRuntimeImpl::new();
        
        // Instantiate the component
        let instance = runtime.instantiate(&component_type)?;
        
        // Call the function
        let result = instance.execute_function("hello", &[])?;
        
        // Check the result
        assert_eq!(result.len(), 1);
        match &result[0] {
            Value::I32(val) => assert_eq!(*val, 42), // Default implementation returns 42
            _ => panic!("Expected I32 result"),
        }
        
        Ok(())
    }

    #[test]
    fn test_host_function_registration() -> Result<()> {
        // Create a component type with a function export
        let component_type = ComponentType {
            imports: Vec::new(),
            exports: vec![(
                "add".to_string(),
                ExternType::Function(FuncType {
                    params: vec![ValueType::I32, ValueType::I32],
                    results: vec![ValueType::I32],
                }),
            )],
            instances: Vec::new(),
        };

        // Create a runtime
        let mut runtime = ComponentRuntimeImpl::new();
        
        // Register a host function
        runtime.register_host_function(
            "add",
            FuncType {
                params: vec![ValueType::I32, ValueType::I32],
                results: vec![ValueType::I32],
            },
            |args| {
                // Extract the arguments
                let a = match args[0] {
                    Value::I32(val) => val,
                    _ => return Err(wrt_error::Error::new("Expected I32 value".to_string())),
                };
                
                let b = match args[1] {
                    Value::I32(val) => val,
                    _ => return Err(wrt_error::Error::new("Expected I32 value".to_string())),
                };
                
                // Return the sum
                Ok(vec![Value::I32(a + b)])
            },
        )?;
        
        // Instantiate the component
        let instance = runtime.instantiate(&component_type)?;
        
        // Call the function
        let result = instance.execute_function("add", &[Value::I32(3), Value::I32(4)])?;
        
        // Check the result
        assert_eq!(result.len(), 1);
        match &result[0] {
            Value::I32(val) => assert_eq!(*val, 7), // 3 + 4 = 7
            _ => panic!("Expected I32 result"),
        }
        
        Ok(())
    }

    #[test]
    fn test_memory_operations() -> Result<()> {
        // Create a component type with a memory export
        let component_type = ComponentType {
            imports: Vec::new(),
            exports: vec![(
                "memory".to_string(),
                ExternType::Memory(wrt_types::MemoryType {
                    limits: wrt_types::Limits {
                        min: 1,
                        max: Some(2),
                    },
                    shared: false,
                }),
            )],
            instances: Vec::new(),
        };

        // Create a runtime
        let runtime = ComponentRuntimeImpl::new();
        
        // Instantiate the component
        let mut instance = runtime.instantiate(&component_type)?;
        
        // Write to memory
        let bytes = [1, 2, 3, 4, 5];
        instance.write_memory("memory", 100, &bytes)?;
        
        // Read from memory
        let read_bytes = instance.read_memory("memory", 100, 5)?;
        
        // Check the result
        assert_eq!(read_bytes, bytes);
        
        Ok(())
    }
} 