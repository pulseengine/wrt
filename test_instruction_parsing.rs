use std::fs;
use wrt_runtime::{instruction_parser, type_conversion, module::Module};
use wrt_decoder::decoder::decode_module;

fn main() -> wrt_error::Result<()> {
    println!("Testing instruction parsing...");
    
    // Read the test WASM file
    let wasm_bytes = fs::read("test_add.wasm").map_err(|_| 
        wrt_error::Error::system_io_error("Failed to read test_add.wasm"))?;
    
    println!("Loaded {} bytes from test_add.wasm", wasm_bytes.len());
    
    // Decode the module
    let decoded = decode_module(&wasm_bytes)?;
    println!("Decoded module with {} functions", decoded.functions.len());
    
    // Convert to runtime module
    let runtime_module = Module::from_wrt_module(&decoded)?;
    println!("Converted to runtime module");
    
    // Check the first function's instructions
    if let Ok(function) = runtime_module.functions.get(0) {
        println!("First function has {} instructions", function.body.len());
        if !function.body.is_empty() {
            println!("✓ Function body successfully parsed with instructions!");
        } else {
            println!("✗ Function body is empty");
        }
    } else {
        println!("No functions found in runtime module");
    }
    
    Ok(())
}