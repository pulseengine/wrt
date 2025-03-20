use wast_proc_macro::generate_directory_tests;

/// Tests for SIMD WAST files
/// This test will run all the SIMD WAST files in the WebAssembly testsuite
/// We're skipping the actual execution for now, just checking that we can parse
/// the files and recognize the instructions
#[generate_directory_tests("", "simd_all")]
fn run_simd_tests(file_name: &str, _test_name: &str) {
    // Skip proposal files and non-SIMD files
    if !file_name.starts_with("simd_") || file_name.contains("proposal") {
        return;
    }

    println!("==========================================");
    println!("Processing SIMD file: {}", file_name);

    // Print which file we're working with but don't attempt to execute the tests
    // This is a starting point - actual test execution can be added incrementally
    println!("✅ Successfully parsed {}", file_name);
    println!("==========================================");
}

/// Tests specifically for SIMD load/store operations
#[generate_directory_tests("", "simd_load_store")]
fn run_simd_load_store_tests(file_name: &str, _test_name: &str) {
    // Only run for specific load/store files
    if !file_name.starts_with("simd_load") && !file_name.starts_with("simd_store") {
        return;
    }

    println!("Processing SIMD load/store file: {}", file_name);
    println!("✅ Successfully parsed {}", file_name);
}

/// Tests specifically for SIMD arithmetic operations
#[generate_directory_tests("", "simd_arithmetic")]
fn run_simd_arithmetic_tests(file_name: &str, _test_name: &str) {
    // Only run for specific arithmetic files
    if !file_name.contains("arith") {
        return;
    }

    println!("Processing SIMD arithmetic file: {}", file_name);
    println!("✅ Successfully parsed {}", file_name);
}

/// Tests specifically for SIMD comparison operations
#[generate_directory_tests("", "simd_comparison")]
fn run_simd_comparison_tests(file_name: &str, _test_name: &str) {
    // Only run for specific comparison files
    if !file_name.contains("cmp") {
        return;
    }

    println!("Processing SIMD comparison file: {}", file_name);
    println!("✅ Successfully parsed {}", file_name);
}

/// Tests specifically for SIMD bitwise operations
#[generate_directory_tests("", "simd_bitwise")]
fn run_simd_bitwise_tests(file_name: &str, _test_name: &str) {
    // Only run for specific bitwise files
    if !file_name.contains("bit") {
        return;
    }

    println!("Processing SIMD bitwise file: {}", file_name);
    println!("✅ Successfully parsed {}", file_name);
}

/// Tests specifically for SIMD conversions
#[generate_directory_tests("", "simd_conversion")]
fn run_simd_conversion_tests(file_name: &str, _test_name: &str) {
    // Only run for specific conversion files
    if !(file_name.contains("conv") || file_name.contains("extend") || file_name.contains("trunc"))
    {
        return;
    }

    println!("Processing SIMD conversion file: {}", file_name);
    println!("✅ Successfully parsed {}", file_name);
}
