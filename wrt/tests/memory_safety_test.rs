//! Tests for memory safety integration

use std::sync::Arc;

use wrt::{
    memory_adapter::{DefaultMemoryAdapter, MemoryAdapter, SafeMemoryAdapter},
    stackless::StacklessEngine,
    Error as WrtError, Module, Result,
};
use wrt_foundation::verification::VerificationLevel;
use wrt_runtime::Memory as RuntimeMemory;

#[test]
fn test_safe_memory_adapter() -> Result<()> {
    // Create a runtime memory
    let memory = Arc::new(RuntimeMemory::new(1)?); // 1 page

    // Create a safe memory adapter
    let adapter = SafeMemoryAdapter::new(memory.clone())?;

    // Test basic properties
    assert_eq!(adapter.size()?, 1);
    assert_eq!(adapter.byte_size()?, 65536);

    // Test store and load
    let data = vec![1, 2, 3, 4, 5];
    adapter.store(100, &data)?;

    let loaded = adapter.load(100, 5)?;
    assert_eq!(&*loaded, &[1, 2, 3, 4, 5]);

    // Test integrity verification
    assert!(adapter.verify_integrity().is_ok());

    // Test memory stats
    let stats = adapter.memory_stats()?;
    assert_eq!(stats.total_size, 65536);

    Ok(())
}

#[test]
fn test_safe_memory_adapter_out_of_bounds() -> Result<()> {
    // Create a runtime memory
    let memory = Arc::new(RuntimeMemory::new(1)?); // 1 page

    // Create a safe memory adapter
    let adapter = SafeMemoryAdapter::new(memory.clone())?;

    // Attempt out-of-bounds access
    let result = adapter.load(65530, 10);
    assert!(result.is_err());

    // Attempt out-of-bounds store
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let result = adapter.store(65530, &data);
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_adapter_verification_levels() -> Result<()> {
    // Create a runtime memory
    let memory = Arc::new(RuntimeMemory::new(1)?); // 1 page

    // Create adapters with different verification levels
    let none_adapter =
        SafeMemoryAdapter::with_verification_level(memory.clone(), VerificationLevel::None)?;
    let standard_adapter =
        SafeMemoryAdapter::with_verification_level(memory.clone(), VerificationLevel::Standard)?;
    let full_adapter =
        SafeMemoryAdapter::with_verification_level(memory.clone(), VerificationLevel::Full)?;

    // Test store and load with each adapter
    let data = vec![1, 2, 3, 4, 5];

    // None adapter
    none_adapter.store(100, &data)?;
    let loaded = none_adapter.load(100, 5)?;
    assert_eq!(&*loaded, &[1, 2, 3, 4, 5]);

    // Standard adapter
    standard_adapter.store(200, &data)?;
    let loaded = standard_adapter.load(200, 5)?;
    assert_eq!(&*loaded, &[1, 2, 3, 4, 5]);

    // Full adapter
    full_adapter.store(300, &data)?;
    let loaded = full_adapter.load(300, 5)?;
    assert_eq!(&*loaded, &[1, 2, 3, 4, 5]);

    Ok(())
}

#[test]
fn test_default_adapter() -> Result<()> {
    // Create a runtime memory
    let memory = Arc::new(RuntimeMemory::new(1)?); // 1 page

    // Create a default memory adapter
    let adapter = DefaultMemoryAdapter::new(memory.clone());

    // Test basic properties
    assert_eq!(adapter.size()?, 1);
    assert_eq!(adapter.byte_size()?, 65536);

    // Test store and load
    let data = vec![1, 2, 3, 4, 5];
    adapter.store(100, &data)?;

    let loaded = adapter.load(100, 5)?;
    assert_eq!(&*loaded, &[1, 2, 3, 4, 5]);

    // Test integrity verification (should always pass)
    assert!(adapter.verify_integrity().is_ok());

    Ok(())
}

#[test]
fn test_memory_adapter_with_wasm() -> Result<()> {
    // Create a WebAssembly module with memory operations
    let wat_code = r#"
    (module
      (memory (export "memory") 1)
      (func $store (export "store") (param i32 i32)
        (i32.store (local.get 0) (local.get 1)))
      (func $load (export "load") (param i32) (result i32)
        (i32.load (local.get 0)))
    )
    "#;

    // Parse the WebAssembly text format to binary
    let wasm = wat::parse_str(wat_code).unwrap();

    // Create a new module
    let module = Module::new()?.load_from_binary(&wasm)?;

    // Create an engine with standard verification
    let mut engine = StacklessEngine::with_verification_level(VerificationLevel::Standard);

    // Instantiate the module
    let instance_idx = engine.instantiate(module)?;

    // Call store function to write a value (store 42 at address 100)
    let store_args = vec![wrt::values::Value::I32(100), wrt::values::Value::I32(42)];
    engine.call_function(instance_idx as u32, 0, &store_args)?;

    // Call load function to read the value back
    let load_args = vec![wrt::values::Value::I32(100)];
    let result = engine.call_function(instance_idx as u32, 1, &load_args)?;

    // Verify the result
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], wrt::values::Value::I32(42));

    // Validate the engine state
    engine.validate()?;

    Ok(())
}
