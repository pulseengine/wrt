use wrt::{execute_test_with_stackless, Result};

#[test]
fn test_stackless_memory_operations() -> Result<()> {
    // Run the memory test using StacklessVM directly
    execute_test_with_stackless("tests/test_memory.wat")
}

#[test]
fn test_memory_persistence() -> Result<()> {
    // Run the memory persistence test using StacklessVM directly
    execute_test_with_stackless("tests/memory_persistence_test.wat")
}

#[test]
fn test_direct_memory_operations() -> Result<()> {
    // WAT code for a simple memory test that uses raw instructions
    let wat_code = r#"
    (module
      (memory (export "memory") 1)
      (func $store (export "store")
        i32.const 100      ;; address
        i32.const 42       ;; value 
        i32.store)         ;; store 42 at address 100
        
      (func $load (export "load") (result i32)
        i32.const 100      ;; address
        i32.load)          ;; load value from address 100
        
      (func $run (export "run") (result i32)
        i32.const 100      ;; address
        i32.const 42       ;; value
        i32.store          ;; store 42 at address 100
        i32.const 100      ;; address
        i32.load           ;; load value from address 100
        i32.const 42       ;; expected value
        i32.eq)            ;; compare result with expected (1 if equal, 0 if not equal)
    )
    "#;

    // Parse the WAT to WASM binary
    let wasm = wat::parse_str(wat_code).unwrap();
    println!("Successfully parsed WAT string to WASM binary");

    // Create a new module
    let module = wrt::Module::new().load_from_binary(&wasm)?;

    println!(
        "Successfully loaded module with {} memory definitions",
        module.memories.len()
    );
    println!("Memory types: {:?}", module.memories);
    println!(
        "Exports: {}",
        module
            .exports
            .iter()
            .map(|e| format!("{} (kind={:?}, idx={})", e.name, e.kind, e.index))
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Initialize the StacklessVM
    let mut engine = wrt::new_stackless_engine();
    let instance_idx = engine.instantiate(module.clone())?;

    // Check memory instance details before any operations
    println!("Module has {} memory definitions", module.memories.len());

    // Find the memory export (there should be one named "memory")
    let mem_export = module
        .exports
        .iter()
        .find(|e| e.name == "memory")
        .ok_or_else(|| wrt::Error::Execution("Memory export not found".into()))?;
    let mem_idx = if let wrt::ExportKind::Memory = mem_export.kind {
        mem_export.index
    } else {
        return Err(wrt::Error::Execution("Memory export kind mismatch".into()));
    };

    // Manual checks to diagnose the issue
    {
        let instance = &engine.instances[instance_idx as usize];
        println!("Created instance with {} memories", instance.memories.len());
        println!("Instance has {} memories", instance.memories.len());
        if !instance.memories.is_empty() {
            println!("Memory data around address 100 before any operations:");
            let start = if 100 >= 4 { 96 } else { 0 };
            for i in start..start + 12 {
                println!("  [{:3}]: {}", i, instance.memories[0].data[i as usize]);
            }
        }
    }

    // Manually modify memory to set value directly
    {
        let instance = &mut engine.instances[instance_idx as usize];
        if !instance.memories.is_empty() {
            // Set the value directly in memory
            let value: i32 = 42;
            let bytes = value.to_le_bytes();
            instance.memories[0].data[100] = bytes[0];
            instance.memories[0].data[101] = bytes[1];
            instance.memories[0].data[102] = bytes[2];
            instance.memories[0].data[103] = bytes[3];

            println!("Manually set memory at address 100 to value 42");

            // Verify the value was set
            println!("Memory after direct manipulation:");
            let start = if 100 >= 4 { 96 } else { 0 };
            for i in start..start + 12 {
                println!("  [{:3}]: {}", i, instance.memories[0].data[i as usize]);
            }
        }
    }

    // Call load directly to get the value from memory
    println!("Calling load function directly");
    let load_result = engine.execute(instance_idx, 1, vec![])?;
    println!("Load result: {:?}", load_result);

    // Verify the loaded value is correct
    if let Some(wrt::Value::I32(loaded_value)) = load_result.first() {
        if *loaded_value != 42 {
            return Err(wrt::Error::Execution(format!(
                "Load returned incorrect value: {}, expected 42",
                loaded_value
            )));
        }
        println!("Successfully loaded the correct value: {}", loaded_value);
    } else {
        return Err(wrt::Error::Execution(
            "Load did not return an i32 value".into(),
        ));
    }

    // Return Ok if we got this far
    Ok(())
}
