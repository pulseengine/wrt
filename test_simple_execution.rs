use wrt::prelude::*;
use wrt::StacklessEngine;

fn main() {
    println!("Testing StacklessEngine real execution...\n");

    // Create a simple WASM module that adds two numbers
    let wasm = wat::parse_str(r#"
        (module
            (func $add (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
            (export "add" (func $add))
        )
    "#).expect("Failed to parse WAT");

    let engine = StacklessEngine::new();
    engine.load_module(Some("add_test"), &wasm).expect("Failed to load module");

    // Test addition
    match engine.call_function("add", &[Value::I32(5), Value::I32(3)]) {
        Ok(result) => {
            println!("✅ SUCCESS: 5 + 3 = {:?}", result[0]);
            if result[0] == Value::I32(8) {
                println!("✅ VERIFIED: StacklessEngine performs REAL execution!");
            } else {
                println!("❌ ERROR: Expected 8, got {:?}", result[0]);
            }
        }
        Err(e) => println!("❌ ERROR: {}", e),
    }
}