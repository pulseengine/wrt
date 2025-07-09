use std::fs;
use std::sync::Arc;

// Import the core components we need
use wrt_runtime::stackless::StacklessEngine;
use wrt_runtime::module::Module;
use wrt_runtime::module_instance::ModuleInstance;
use wrt_decoder::decoder::decode_module;
use wrt_foundation::values::Value;

fn main() -> wrt_error::Result<()> {
    println!("=== WASM Execution Test ===");
    
    // Read the real WASM file
    let wasm_bytes = fs::read("test_add.wasm").map_err(|_| 
        wrt_error::Error::system_io_error("Failed to read test_add.wasm"))?;
    
    println!("1. Loaded {} bytes from test_add.wasm", wasm_bytes.len());
    
    // Decode the module
    let decoded = decode_module(&wasm_bytes)?;
    println!("2. Decoded module with {} functions", decoded.functions.len());
    
    // Convert to runtime module
    let runtime_module = Module::from_wrt_module(&decoded)?;
    println!("3. Converted to runtime module");
    
    // Check if we have parsed instructions
    if let Ok(function) = runtime_module.functions.get(0) {
        println!("4. Function 0 has {} instructions", function.body.len());
        if !function.body.is_empty() {
            println!("   ✓ Instructions successfully parsed!");
            
            // Create a stackless engine
            let mut engine = StacklessEngine::new()?;
            
            // Create module instance
            let instance = ModuleInstance::new(runtime_module.clone(), 0)?;
            let instance_arc = Arc::new(instance);
            
            // Set current module
            let _instance_idx = engine.set_current_module(instance_arc)?;
            
            // Try to execute function 0 with test arguments
            println!("5. Attempting execution...");
            let args = vec![Value::I32(5), Value::I32(3)];
            let results = engine.execute(0, 0, args)?;
            
            println!("6. ✓ Execution completed!");
            println!("   Results: {:?}", results);
            
        } else {
            println!("   ✗ No instructions found");
        }
    } else {
        println!("4. No functions found");
    }
    
    Ok(())
}