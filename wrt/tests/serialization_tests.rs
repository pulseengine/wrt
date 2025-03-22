#[cfg(feature = "serialization")]
mod serialization_tests {
    use wrt::error::Result;
    use wrt::execution::Engine;
    use wrt::module::Module;
    use wrt::serialization::Serializable;
    use wrt::values::Value;

    const SIMPLE_MODULE: &[u8] = &[
        // WebAssembly module header
        0x00, 0x61, 0x73, 0x6D, // magic
        0x01, 0x00, 0x00, 0x00, // version
        // Type section
        0x01, // section code
        0x07, // section size
        0x01, // number of types
        0x60, // func type
        0x01, // param count
        0x7F, // i32
        0x01, // return count
        0x7F, // i32
        // Function section
        0x03, // section code
        0x02, // section size
        0x01, // number of functions
        0x00, // type index
        // Export section
        0x07, // section code
        0x07, // section size
        0x01, // number of exports
        0x03, // export name length
        b'a', b'd', b'd', // "add"
        0x00, // export kind (function)
        0x00, // export index
        // Code section
        0x0A, // section code
        0x0A, // section size
        0x01, // number of code entries
        0x08, // code size
        0x01, // local declaration count
        0x01, // number of locals
        0x7F, // i32
        0x20, 0x00, // local.get 0
        0x41, 0x01, // i32.const 1
        0x6A, // i32.add
        0x0B, // end
    ];

    #[test]
    fn test_serialize_deserialize_idle_state() -> Result<()> {
        // Create a module and load it
        let mut module = Module::new();
        module.load_from_binary(SIMPLE_MODULE)?;

        // Create an engine in idle state
        let engine = Engine::new(module);

        // Serialize the engine to JSON
        let json = engine.to_json()?;

        // Deserialize the engine from JSON
        let deserialized_engine = Engine::from_json(&json)?;

        // Verify the state is idle
        assert_eq!(
            format!("{:?}", engine.state()),
            format!("{:?}", deserialized_engine.state())
        );

        // Verify the fuel setting is the same
        assert_eq!(
            engine.remaining_fuel(),
            deserialized_engine.remaining_fuel()
        );

        Ok(())
    }

    #[test]
    fn test_serialize_deserialize_with_binary_format() -> Result<()> {
        // Create a module and load it
        let mut module = Module::new();
        module.load_from_binary(SIMPLE_MODULE)?;

        // Create an engine
        let engine = Engine::new(module);

        // Serialize to binary
        let binary = engine.to_binary()?;

        // Deserialize from binary
        let deserialized_engine = Engine::from_binary(&binary)?;

        // Verify the binary format works
        assert_eq!(
            format!("{:?}", engine.state()),
            format!("{:?}", deserialized_engine.state())
        );

        Ok(())
    }

    #[test]
    fn test_execute_serialize_resume() -> Result<()> {
        // Create a module and load it
        let mut module = Module::new();
        module.load_from_binary(SIMPLE_MODULE)?;

        // Create an engine
        let mut engine = Engine::new(module);

        // Set a fuel limit to ensure we can pause execution
        engine.set_fuel(Some(100));

        // Get the function "add" from the module
        let instance_idx = 0;
        let function_idx = 0;

        // Start executing - this will execute until fuel runs out
        let args = vec![Value::I32(41)];
        let _result = match engine.execute(instance_idx, function_idx, args) {
            Ok(result) => result,
            Err(wrt::error::Error::FuelExhausted) => {
                // This is expected when out of fuel
                vec![]
            }
            Err(e) => return Err(e),
        };

        // Verify the state is paused
        assert!(matches!(
            engine.state(),
            wrt::execution::ExecutionState::Paused { .. }
        ));

        // Serialize the paused engine
        let json = engine.to_json()?;

        // On another "machine", deserialize the engine
        let mut resumed_engine = Engine::from_json(&json)?;

        // Add more fuel
        resumed_engine.set_fuel(Some(100));

        // Resume execution
        let result = resumed_engine.resume()?;

        // Check the result is correct (42)
        assert_eq!(result, vec![Value::I32(42)]);

        Ok(())
    }

    #[test]
    fn test_serializable_roundtrip() -> Result<()> {
        // Create a module and load it
        let mut module = Module::new();
        module.load_from_binary(SIMPLE_MODULE)?;

        // Create an engine
        let engine = Engine::new(module);

        // Convert to serializable state
        let serializable_state = engine.to_serializable()?;

        // Convert back to engine
        let restored_engine = Engine::from_serializable(serializable_state)?;

        // Verify the engines match
        assert_eq!(
            format!("{:?}", engine.state()),
            format!("{:?}", restored_engine.state())
        );

        Ok(())
    }
}
