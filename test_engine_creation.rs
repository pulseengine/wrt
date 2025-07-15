fn main() {
    println!("Testing basic engine creation...");
    
    // Try to initialize memory first
    println!("Initializing memory system...");
    if let Err(e) = wrt_foundation::memory_init::MemoryInitializer::initialize() {
        println!("Warning: Failed to initialize memory system: {}", e);
    } else {
        println!("Memory system initialized successfully");
    }
    
    // Test creating a StacklessEngine
    println!("Creating StacklessEngine...");
    let _engine = wrt_runtime::stackless::engine::StacklessEngine::new();
    println!("StacklessEngine created successfully!");
    
    println!("Basic engine creation test passed!");
}