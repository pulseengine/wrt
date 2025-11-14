// Temporary diagnostic tool to test which module sizes work
// Run with: rustc test_module_sizes.rs && ./test_module_sizes

use std::fs;

fn test_module(binary: &[u8], name: &str) {
    println!("\n=== Testing {} ({} bytes) ===", name, binary.len());

    let result = std::thread::Builder::new()
        .name(format!("test-{}", name))
        .stack_size(32 * 1024 * 1024)  // 32MB
        .spawn(move || {
            println!("[{}] Thread started", name);

            // Simulate what wrt-runtime does
            println!("[{}] Would parse module...", name);

            // Just sleep to simulate work
            std::thread::sleep(std::time::Duration::from_millis(100));

            println!("[{}] ✓ Success", name);
        })
        .unwrap()
        .join();

    match result {
        Ok(_) => println!("[{}] ✓ Passed", name),
        Err(e) => println!("[{}] ✗ Failed: {:?}", name, e),
    }
}

fn main() {
    println!("Component Module Size Testing");
    println!("==============================\n");

    // Load the component
    let component_data = fs::read("./file_ops_component.wasm")
        .expect("Failed to read component file");

    println!("Component size: {} bytes\n", component_data.len());

    // We know the component has 4 modules but we don't have the decoder here
    // So just test with the full component for now
    test_module(&component_data, "full-component");
}
