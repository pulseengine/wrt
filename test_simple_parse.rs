use std::fs;

fn main() {
    println!("Simple WASM test...");
    
    // Read the test WASM file
    if let Ok(wasm_bytes) = fs::read("test_add.wasm") {
        println!("Loaded {} bytes from test_add.wasm", wasm_bytes.len());
        
        // Check WASM magic number
        if wasm_bytes.len() >= 4 && &wasm_bytes[0..4] == [0x00, 0x61, 0x73, 0x6D] {
            println!("✓ Valid WASM magic number found!");
        } else {
            println!("✗ Invalid WASM magic number");
        }
        
        // Check version
        if wasm_bytes.len() >= 8 && &wasm_bytes[4..8] == [0x01, 0x00, 0x00, 0x00] {
            println!("✓ Valid WASM version 1 found!");
        } else {
            println!("✗ Invalid WASM version");
        }
        
    } else {
        println!("Failed to read test_add.wasm");
    }
}