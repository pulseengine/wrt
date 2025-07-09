use wrt_runtime::engine::{CapabilityAwareEngine, EnginePreset};
use wrt_foundation::values::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing direct WASM execution with wrt-runtime...");
    
    // Check if simple_add.wasm exists
    if !std::path::Path::new("simple_add.wasm").exists() {
        println!("simple_add.wasm not found. Creating a simple add function WASM file...");
        
        // Create a simple WASM module with an add function
        let wasm_bytes = create_simple_add_wasm();
        std::fs::write("simple_add.wasm", wasm_bytes)?;
        println!("Created simple_add.wasm");
    }
    
    // Read the simple_add.wasm file
    let wasm_bytes = std::fs::read("simple_add.wasm")?;
    println!("Loaded WASM module: {} bytes", wasm_bytes.len());
    
    // Validate WASM header
    if wasm_bytes.len() >= 8 {
        let magic = &wasm_bytes[0..4];
        let version = &wasm_bytes[4..8];
        
        if magic == [0x00, 0x61, 0x73, 0x6D] {
            println!("✓ Valid WASM magic number");
            println!("  Version: {:?}", version);
        } else {
            println!("✗ Invalid WASM magic number: {:?}", magic);
            return Err("Invalid WASM file".into());
        }
    }
    
    // Create QM engine (most permissive)
    let mut engine = CapabilityAwareEngine::with_preset(EnginePreset::QM)?;
    println!("Created QM engine");
    
    // Load the module
    let module_handle = engine.load_module(&wasm_bytes)?;
    println!("Loaded module: {:?}", module_handle);
    
    // Instantiate the module
    let instance_handle = engine.instantiate(module_handle)?;
    println!("Created instance: {:?}", instance_handle);
    
    // Check if the add function exists
    if engine.has_function(instance_handle, "add")? {
        println!("Found 'add' function");
        
        // Call the add function with arguments 5 and 3
        let args = vec![Value::I32(5), Value::I32(3)];
        let results = engine.execute(instance_handle, "add", &args)?;
        
        println!("Execution result: {:?}", results);
        
        if let Some(Value::I32(result)) = results.first() {
            println!("✓ 5 + 3 = {}", result);
            if *result == 8 {
                println!("✓ Correct result!");
            } else {
                println!("✗ Unexpected result: expected 8, got {}", result);
            }
        }
    } else {
        println!("Function 'add' not found in exports");
        
        // List available exports
        println!("Available exports:");
        // Note: We would need to add this method to the engine
        // for now, we'll continue with what we have
    }
    
    Ok(())
}

// Create a simple WASM module with an add function
fn create_simple_add_wasm() -> Vec<u8> {
    // This is a hand-crafted WASM module with an add function
    // WASM magic number and version
    let mut wasm = vec![
        0x00, 0x61, 0x73, 0x6D, // magic "\0asm"
        0x01, 0x00, 0x00, 0x00, // version 1
    ];
    
    // Type section (function signatures)
    wasm.extend_from_slice(&[
        0x01, // section id: type
        0x07, // section size
        0x01, // number of types
        0x60, // function type
        0x02, // number of parameters
        0x7F, 0x7F, // i32, i32
        0x01, // number of results
        0x7F, // i32
    ]);
    
    // Function section (function type indices)
    wasm.extend_from_slice(&[
        0x03, // section id: function
        0x02, // section size
        0x01, // number of functions
        0x00, // function type index
    ]);
    
    // Export section
    wasm.extend_from_slice(&[
        0x07, // section id: export
        0x07, // section size
        0x01, // number of exports
        0x03, // export name length
        b'a', b'd', b'd', // export name "add"
        0x00, // export kind: function
        0x00, // function index
    ]);
    
    // Code section
    wasm.extend_from_slice(&[
        0x0A, // section id: code
        0x09, // section size
        0x01, // number of functions
        0x07, // function body size
        0x00, // number of locals
        0x20, 0x00, // local.get 0 (first parameter)
        0x20, 0x01, // local.get 1 (second parameter)
        0x6A, // i32.add
        0x0B, // end
    ]);
    
    wasm
}