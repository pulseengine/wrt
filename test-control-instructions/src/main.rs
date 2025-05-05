#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Tests module
mod tests;

// Import appropriate types based on environment
#[cfg(feature = "std")]
use std::process;

// Standard entry point
#[cfg(feature = "std")]
fn main() {
    println!("Running WebAssembly control instructions tests...");

    // Register all tests with the global registry
    tests::register_control_instruction_tests();

    // Run all tests
    let registry = wrt_test_registry::TestRegistry::global();
    let failed_count = registry.run_filtered_tests(None, Some("instruction-decoder"), true);

    if failed_count == 0 {
        println!("\n✅ All control instruction tests PASSED!");
    } else {
        println!("\n❌ Some control instruction tests FAILED!");
        process::exit(1);
    }
}

// No-std entry point
#[cfg(not(feature = "std"))]
fn main() -> ! {
    // Register all tests with the global registry
    tests::register_control_instruction_tests();

    // In a real no_std environment, we would need a custom way to report results
    // Here we just enter an idle loop
    loop {}
}
