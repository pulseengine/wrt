//! Memory search test program
//! This test demonstrates using the memory search functionality.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wrt::execution::{Engine, MemoryAddr, ModuleInstance};
use wrt::memory::Memory;
use wrt::module::Module;
use wrt::types::MemoryType;

/// A simple memory search test
///
/// This test:
/// 1. Creates a memory instance
/// 2. Populates it with some sample data including a "Completed 5 iterations" string
/// 3. Demonstrates using the memory search functionality
#[test]
fn test_memory_search() {
    // Create a memory instance with default settings
    let mem_type = MemoryType {
        min: 1,
        max: Some(2),
    };
    let mut memory = Memory::new(mem_type);

    // Sample data to mimic real WebAssembly memory
    let sample_data = b"Hello, world!\0This is a test\0Completed 5 iterations\0";

    // Write sample data to memory at different addresses
    memory.write_bytes(1000, sample_data).unwrap();
    memory.write_bytes(2000, b"Another test string\0").unwrap();
    memory
        .write_bytes(0xFFFFFFE0 as u32, b"Negative offset string\0")
        .unwrap();

    // Test the memory search function directly
    let results = memory.search_memory("Completed", false);

    println!("Memory search results:");
    println!("Found {} occurrences of 'Completed'", results.len());

    for (i, (addr, string)) in results.iter().enumerate() {
        println!(
            "Result #{}: Address: {:#x} - String: '{}'",
            i + 1,
            addr,
            string
        );
    }

    // Verify that we found at least one occurrence
    assert!(!results.is_empty(), "No search results found");

    // Also search for something that doesn't exist
    let empty_results = memory.search_memory("NonExistentString", false);
    assert!(empty_results.is_empty(), "Found unexpected results");

    // Test the Engine with memory search capability
    // Create a basic engine to test the execution context
    let mem_addr = MemoryAddr {
        instance_idx: 0,
        memory_idx: 0,
    };

    // Create an empty module
    let module = Module::default();

    // Create a basic engine with our memory
    let mut engine = Engine::new();

    // Create a module instance with our memory
    let instance = ModuleInstance {
        module_idx: 0,
        module,
        func_addrs: vec![],
        table_addrs: vec![],
        memory_addrs: vec![mem_addr.clone()],
        global_addrs: vec![],
        memories: vec![memory],
    };
    engine.instances.push(instance);

    // Now test the Engine's memory search function
    println!("\nTesting engine's memory search capability:");
    engine
        .search_memory_for_pattern(&mem_addr, "Completed", false)
        .unwrap();

    // Also try to search for iterations
    println!("\nSearching for 'iterations':");
    engine
        .search_memory_for_pattern(&mem_addr, "iterations", false)
        .unwrap();

    println!("\nMemory search test complete!");
}
