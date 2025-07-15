// Direct test of WastEngine execution to verify real execution
use anyhow::Result;
use wrt_build_core::wast_execution::WastEngine;
use wrt_foundation::values::Value;

fn main() -> Result<()> {
    println!("Testing WastEngine execution...");
    
    // Simple WASM module that adds two numbers
    let wasm_binary = wat::parse_str(r#"
        (module
          (func (export "add") (param i32) (param i32) (result i32)
            local.get 0
            local.get 1
            i32.add
          )
        )
    "#)?;
    
    println!("WASM binary created, {} bytes", wasm_binary.len());
    
    // Create engine and load module
    let mut engine = WastEngine::new()?;
    engine.load_module(Some("test"), &wasm_binary)?;
    println!("Module loaded successfully");
    
    // Test correct execution: 2 + 3 = 5
    println!("Testing correct execution: add(2, 3)");
    let args = vec![Value::I32(2), Value::I32(3)];
    let results = engine.invoke_function(None, "add", &args)?;
    println!("Results: {:?}", results);
    
    // Check if result is 5
    if let Some(Value::I32(result)) = results.first() {
        if *result == 5 {
            println!("✅ Correct execution: 2 + 3 = 5");
        } else {
            println!("❌ Wrong result: 2 + 3 = {} (expected 5)", result);
        }
    } else {
        println!("❌ No result or wrong type");
    }
    
    // Test that we can detect wrong expectations
    println!("\nTesting wrong expectation detection: add(2, 3) should NOT equal 999");
    if let Some(Value::I32(result)) = results.first() {
        if *result == 999 {
            println!("❌ FAKE EXECUTION DETECTED: 2 + 3 returned 999!");
        } else {
            println!("✅ Real execution: 2 + 3 = {} (not 999)", result);
        }
    }
    
    Ok(())
}