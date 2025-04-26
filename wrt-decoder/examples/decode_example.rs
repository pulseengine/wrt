use std::fs;
use wrt_decoder::decode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the WebAssembly binary file
    let wasm_bytes = fs::read("examples/example.wasm")?;

    // Decode the module
    let module = decode(&wasm_bytes)?;

    // Print some basic information about the module
    println!("Module version: {}", module.version);
    println!("Number of functions: {}", module.functions.len());
    println!("Number of memories: {}", module.memories.len());
    println!("Number of exports: {}", module.exports.len());

    Ok(())
}
