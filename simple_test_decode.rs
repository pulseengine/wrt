use std::fs;
use wrt_decoder::decode_module;

fn main() -> wrt_error::Result<()> {
    println!("=== Simple WASM Decode Test ===";
    
    // Load our test WASM file
    let wasm_bytes = fs::read("test_add.wasm")?;
    println!("✓ Loaded test_add.wasm ({} bytes)", wasm_bytes.len(;
    
    // Try to decode it
    println!("→ Decoding module...";
    let module = decode_module(&wasm_bytes)?;
    println!("✓ Module decoded successfully";
    
    println!("Module info:";
    println!("  - Types: {}", module.types.len(;
    println!("  - Functions: {}", module.functions.len(;
    println!("  - Exports: {}", module.exports.len(;
    println!("  - Imports: {}", module.imports.len(;
    
    Ok(())
}