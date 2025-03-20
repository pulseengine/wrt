use wrt::{Engine, ExportKind, Module, Result, Value};

/// Helper function to run a WebAssembly test using WAT format
fn run_wat_test(wat: &str, test_config: &[(&str, Vec<Value>, Option<Value>)]) -> Result<()> {
    // Parse WAT to binary WebAssembly
    let wasm = wat::parse_str(wat).expect("Failed to parse WAT");

    // Initialize the WRT engine
    let empty_module = Module::new();
    let module = empty_module
        .load_from_binary(&wasm)
        .expect("Failed to parse WASM");
    let mut engine = Engine::new(module.clone());

    // Instantiate the module
    engine.instantiate(module.clone())?;
    let instance_idx = 0;

    // Run each test case
    for (func_name, args, expected_result) in test_config {
        // Get the instance for each test to avoid borrowing issues
        let instance = engine
            .get_instance(instance_idx)
            .expect("No instance found");

        // Get the function index
        let func_idx = instance
            .module
            .exports
            .iter()
            .find(|export| export.name == *func_name)
            .map(|export| {
                if export.kind == ExportKind::Function {
                    export.index
                } else {
                    panic!("Expected function export for {}", func_name);
                }
            })
            .expect(&format!("Function {} not found", func_name));

        // Execute the function
        println!("Executing {}...", func_name);
        let result = engine.execute(instance_idx, func_idx, args.clone())?;

        // Verify the result
        if let Some(expected) = expected_result {
            if result.len() != 1 {
                panic!("Expected 1 result, got {}", result.len());
            }

            let actual = &result[0];
            assert_eq!(
                actual, expected,
                "Expected {:?}, got {:?}",
                expected, actual
            );
            println!("✅ {}: Got expected result: {:?}", func_name, actual);
        } else {
            // No return value expected
            assert!(result.is_empty(), "Expected no result, got {:?}", result);
            println!("✅ {}: Function executed successfully", func_name);
        }
    }

    println!("All tests passed!");
    Ok(())
}

#[test]
fn test_memory_load_store() -> Result<()> {
    // WebAssembly module with memory operations
    let wat = r#"
    (module
      (memory 1)
      (export "memory" (memory 0))

      ;; Store value 42 at address 0
      (func (export "store") (param i32 i32)
        local.get 0  ;; address
        local.get 1  ;; value
        i32.store)

      ;; Load value from address
      (func (export "load") (param i32) (result i32)
        local.get 0  ;; address
        i32.load)

      ;; Test store then load in a single function
      (func (export "test_store_load") (result i32)
        i32.const 100  ;; address
        i32.const 42   ;; value
        i32.store      ;; store value directly
        
        i32.const 100  ;; address
        i32.load)      ;; load value
    )
    "#;

    // Test configuration: (function_name, arguments, expected_result)
    let test_config = [
        // First store a value
        ("store", vec![Value::I32(100), Value::I32(42)], None),
        // Then load it back and verify
        ("load", vec![Value::I32(100)], Some(Value::I32(42))),
        // Test the combined function
        ("test_store_load", vec![], Some(Value::I32(42))),
    ];

    run_wat_test(wat, &test_config)
}

#[test]
fn test_memory_persistence() -> Result<()> {
    // WebAssembly module with memory operations
    let wat = r#"
    (module
      (memory 1)
      (export "memory" (memory 0))

      ;; Store a value at address
      (func (export "store") (param i32 i32)
        local.get 0  ;; address
        local.get 1  ;; value
        i32.store)

      ;; Load a value from address
      (func (export "load") (param i32) (result i32)
        local.get 0
        i32.load)
    )
    "#;

    // Test configuration: (function_name, arguments, expected_result)
    let test_config = [
        // Store value 42 at address 100
        ("store", vec![Value::I32(100), Value::I32(42)], None),
        // Load value from address 100 and verify it's 42
        ("load", vec![Value::I32(100)], Some(Value::I32(42))),
        // Store a different value 99 at address 200
        ("store", vec![Value::I32(200), Value::I32(99)], None),
        // Load first value again to verify memory persistence
        ("load", vec![Value::I32(100)], Some(Value::I32(42))),
        // Load second value to verify multiple values
        ("load", vec![Value::I32(200)], Some(Value::I32(99))),
    ];

    run_wat_test(wat, &test_config)
}

#[test]
fn test_memory_initialization() -> Result<()> {
    // WebAssembly module with memory initialized via data segments
    let wat = r#"
    (module
      (memory 1)
      (export "memory" (memory 0))
      
      ;; Initialize memory with data at offset 100
      (data (i32.const 100) "\01\02\03\04")
      
      ;; Load a word (4 bytes) from memory
      (func (export "load_word") (param i32) (result i32)
        local.get 0
        i32.load)
    )
    "#;

    // Test configuration: (function_name, arguments, expected_result)
    let test_config = [
        // Load the initialized data (should be 0x04030201 in little-endian)
        (
            "load_word",
            vec![Value::I32(100)],
            Some(Value::I32(0x04030201)),
        ),
    ];

    run_wat_test(wat, &test_config)
}

#[test]
fn test_memory_multiple_stores() -> Result<()> {
    // WebAssembly module with multiple stores to test memory updates
    let wat = r#"
    (module
      (memory 1)
      (export "memory" (memory 0))

      ;; Initialize some memory locations
      (func (export "init")
        ;; Store 0x11223344 at address 100
        i32.const 100
        i32.const 0x11223344
        i32.store
        
        ;; Store 0x55667788 at address 104
        i32.const 104
        i32.const 0x55667788
        i32.store)

      ;; Load a value from address
      (func (export "load") (param i32) (result i32)
        local.get 0
        i32.load)
        
      ;; Update a value at address  
      (func (export "update") (param i32 i32)
        local.get 0
        local.get 1
        i32.store)
    )
    "#;

    // Test configuration: (function_name, arguments, expected_result)
    let test_config = [
        // Initialize memory with test values
        ("init", vec![], None),
        // Test initial values
        ("load", vec![Value::I32(100)], Some(Value::I32(0x11223344))),
        ("load", vec![Value::I32(104)], Some(Value::I32(0x55667788))),
        // Update a value
        (
            "update",
            vec![Value::I32(100), Value::I32(0x7ABBCCDD)],
            None,
        ),
        // Verify the update worked
        ("load", vec![Value::I32(100)], Some(Value::I32(0x7ABBCCDD))),
        ("load", vec![Value::I32(104)], Some(Value::I32(0x55667788))),
    ];

    run_wat_test(wat, &test_config)
}
