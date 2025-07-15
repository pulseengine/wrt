fn main() {
    println!("Testing CapabilityAwareEngine-based WastEngine...");
    
    // Initialize memory first like cargo-wrt does
    if let Err(e) = wrt_foundation::memory_init::MemoryInitializer::initialize() {
        println!("Warning: Failed to initialize memory system: {}", e);
    } else {
        println!("Memory system initialized successfully");
    }
    
    // Try to create WastEngine (now using CapabilityAwareEngine internally)
    println!("Creating WastEngine...");
    let mut engine = match wrt_build_core::wast_execution::WastEngine::new() {
        Ok(engine) => engine,
        Err(e) => {
            println!("Failed to create WastEngine: {}", e);
            return;
        }
    };
    println!("WastEngine created successfully!");
    
    // Try to create a simple WAT module for testing
    let simple_wat = r#"
        (module
            (func (export "add") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add))
    "#;
    
    println!("Converting WAT to WASM binary...");
    let wasm_binary = match wat::parse_str(simple_wat) {
        Ok(binary) => binary,
        Err(e) => {
            println!("Failed to parse WAT: {}", e);
            return;
        }
    };
    println!("WAT converted successfully, binary size: {}", wasm_binary.len());
    
    // Try to load the module
    println!("Loading WASM module...");
    match engine.load_module(Some("test"), &wasm_binary) {
        Ok(()) => println!("Module loaded successfully!"),
        Err(e) => {
            println!("Failed to load module: {}", e);
            return;
        }
    }
    
    println!("Basic CapabilityAwareEngine-based WastEngine test passed!");
}