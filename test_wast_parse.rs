use std::fs;

fn main() {
    println!("Testing WAST parsing...");
    
    // First initialize memory like cargo-wrt does
    if let Err(e) = wrt_foundation::memory_init::MemoryInitializer::initialize() {
        println!("Warning: Failed to initialize memory system: {}", e);
    } else {
        println!("Memory system initialized successfully");
    }
    
    let content = fs::read_to_string("simple_test.wast").expect("Failed to read file");
    println!("File content read successfully, length: {}", content.len());
    
    // Try to parse with wast
    println!("Creating parse buffer...");
    let buf = wast::parser::ParseBuffer::new(&content).expect("Failed to create parse buffer");
    println!("Parse buffer created successfully");
    
    println!("Parsing WAST...");
    let _wast: wast::Wast = wast::parser::parse(&buf).expect("Failed to parse WAST");
    println!("WAST parsed successfully!");
}