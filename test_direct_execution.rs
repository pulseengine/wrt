use wrt_runtime::engine::{CapabilityAwareEngine, EnginePreset};
use wrt_foundation::values::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing direct WASM execution with QM mode...");
    
    // Read the simple_add.wasm file
    let wasm_bytes = std::fs::read("simple_add.wasm")?;
    println!("Loaded WASM module: {} bytes", wasm_bytes.len());
    
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
            println!("5 + 3 = {}", result);
        }
    } else {
        println!("Function 'add' not found in exports");
    }
    
    Ok(())
}