#[cfg(test)]
mod resume_tests {
    use wrt::error::Result;
    use wrt::execution::{Engine, ExecutionState};
    use wrt::instructions::Instruction;
    use wrt::module::Function;
    use wrt::new_module;
    use wrt::types::{FuncType, ValueType};
    use wrt::values::Value;

    #[test]
    fn test_pause_on_fuel_exhaustion() -> Result<()> {
        // Create a simple module with a function that adds 1 to its input
        let mut module = new_module();

        // Create a function type for a function that takes an i32 and returns an i32
        let func_type = FuncType {
            params: vec![ValueType::I32],
            results: vec![ValueType::I32],
        };

        // Create a simple function that just returns a constant
        let body = vec![
            // Push constant 42
            Instruction::I32Const(42),
            // End function
            Instruction::End,
        ];

        // Create a function with this body and type
        let function = Function {
            type_idx: 0,
            locals: vec![],
            body,
        };

        // Add the type and function to the module
        module.types.push(func_type);
        module.functions.push(function);

        // Create an engine and instantiate the module
        let mut engine = Engine::new(module.clone());

        // Instantiate the module
        let instance_idx = engine.instantiate(module)?;

        // Manually set the engine state to paused
        engine.set_state(ExecutionState::Paused {
            instance_idx,
            func_idx: 0,
            pc: 0,
            expected_results: 1,
        });

        // Now resume execution
        let result = engine.resume()?;

        // The result should be 42
        assert_eq!(result, vec![Value::I32(42)]);

        // The engine state should be Finished
        assert!(matches!(engine.state(), ExecutionState::Finished));

        Ok(())
    }

    #[test]
    fn test_resume_functionality() -> Result<()> {
        // Create a simple module with a function that adds 1 to its input
        let mut module = new_module();

        // Create a function type for a function that takes an i32 and returns an i32
        let func_type = FuncType {
            params: vec![ValueType::I32],
            results: vec![ValueType::I32],
        };

        // Create a function with some instructions to create a loop that would run for many iterations
        let body = vec![
            // Load the parameter
            Instruction::LocalGet(0),
            // Push constant 1
            Instruction::I32Const(1),
            // Add param + 1
            Instruction::I32Add,
            // End function
            Instruction::End,
        ];

        // Create a function with this body and type
        let function = Function {
            type_idx: 0,
            locals: vec![],
            body,
        };

        // Add the type and function to the module
        module.types.push(func_type);
        module.functions.push(function);

        // Create an engine and instantiate the module
        let mut engine = Engine::new(module.clone());
        engine.instantiate(module)?;

        // Try to resume when the engine is not paused
        let result = engine.resume();

        // Should get an error with the message "Cannot resume: engine is not paused"
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "Execution error: Cannot resume: engine is not paused"
        );

        Ok(())
    }
}
