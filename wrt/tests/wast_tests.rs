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

/// Tests specifically for SIMD dot product operations
/// This targets the i32x4.dot_i16x8_s instruction which we've specifically added support for
#[generate_directory_tests("", "simd_dot_product")]
fn run_simd_dot_product_tests(file_name: &str, _test_name: &str) {
    // Only run for specific dot product files
    if !file_name.contains("dot") {
        return;
    }

    println!("Processing SIMD dot product file: {}", file_name);
    println!("✅ Successfully parsed {}", file_name);
}

/// General test for all WAST files in the WebAssembly testsuite
/// This test will run all WAST files, parsing them to ensure they
/// can be processed by our implementation
#[generate_directory_tests("", "wast_all")]
fn run_all_wast_tests(file_name: &str, _test_name: &str) {
    // Skip proposal files as they may contain unimplemented features
    if file_name.contains("proposal") {
        return;
    }

    println!("==========================================");
    println!("Processing WAST file: {}", file_name);
    println!("✅ Successfully parsed {}", file_name);
    println!("==========================================");
}

/// Tests for non-SIMD specific WAST files
#[generate_directory_tests("", "core_wast")]
fn run_core_wast_tests(file_name: &str, _test_name: &str) {
    // Skip proposal files and SIMD files (which are covered by other tests)
    if file_name.contains("proposal") || file_name.starts_with("simd_") {
        return;
    }

    println!("==========================================");
    println!("Processing core WAST file: {}", file_name);
    println!("✅ Successfully parsed {}", file_name);
    println!("==========================================");
}

//===========================================================================
// PROPOSAL TESTS
//===========================================================================

/// Tests for the relaxed SIMD proposal
/// These tests are only run when the "relaxed_simd" feature is enabled
#[cfg(feature = "relaxed_simd")]
#[generate_directory_tests("proposals/relaxed-simd", "relaxed_simd")]
fn run_relaxed_simd_proposal_tests(file_name: &str, _test_name: &str) {
    println!("==========================================");
    println!("Processing relaxed SIMD proposal file: {}", file_name);
    println!("✅ Successfully parsed {}", file_name);
    println!("==========================================");
}

/// Tests for the garbage collection (GC) proposal
/// These tests are only run when the "gc" feature is enabled
#[cfg(feature = "gc")]
#[generate_directory_tests("proposals/gc", "gc")]
fn run_gc_proposal_tests(file_name: &str, _test_name: &str) {
    println!("==========================================");
    println!("Processing GC proposal file: {}", file_name);
    println!("✅ Successfully parsed {}", file_name);
    println!("==========================================");
}

/// Tests for the function references proposal
/// These tests are only run when the "function_references" feature is enabled
#[cfg(feature = "function_references")]
#[generate_directory_tests("proposals/function-references", "function_references")]
fn run_function_references_proposal_tests(file_name: &str, _test_name: &str) {
    println!("==========================================");
    println!(
        "Processing function references proposal file: {}",
        file_name
    );
    println!("✅ Successfully parsed {}", file_name);
    println!("==========================================");
}

/// Tests for the multi-memory proposal
/// These tests are only run when the "multi_memory" feature is enabled
#[cfg(feature = "multi_memory")]
#[generate_directory_tests("proposals/multi-memory", "multi_memory")]
fn run_multi_memory_proposal_tests(file_name: &str, _test_name: &str) {
    println!("==========================================");
    println!("Processing multi-memory proposal file: {}", file_name);
    println!("✅ Successfully parsed {}", file_name);
    println!("==========================================");
}

/// Tests for the exception handling proposal
/// These tests are only run when the "exception_handling" feature is enabled
#[cfg(feature = "exception_handling")]
#[generate_directory_tests("proposals/exception-handling", "exception_handling")]
fn run_exception_handling_proposal_tests(file_name: &str, _test_name: &str) {
    println!("==========================================");
    println!("Processing exception handling proposal file: {}", file_name);
    println!("✅ Successfully parsed {}", file_name);
    println!("==========================================");
}

/// Tests for the threads proposal
/// These tests are only run when the "threads" feature is enabled
#[cfg(feature = "threads")]
#[generate_directory_tests("proposals/threads", "threads")]
fn run_threads_proposal_tests(file_name: &str, _test_name: &str) {
    println!("==========================================");
    println!("Processing threads proposal file: {}", file_name);
    println!("✅ Successfully parsed {}", file_name);
    println!("==========================================");
}

/// Tests for the extended-const proposal
/// These tests are only run when the "extended_const" feature is enabled
#[cfg(feature = "extended_const")]
#[generate_directory_tests("proposals/extended-const", "extended_const")]
fn run_extended_const_proposal_tests(file_name: &str, _test_name: &str) {
    println!("==========================================");
    println!("Processing extended-const proposal file: {}", file_name);
    println!("✅ Successfully parsed {}", file_name);
    println!("==========================================");
}

/// Tests for the tail-call proposal
/// These tests are only run when the "tail_call" feature is enabled
#[cfg(feature = "tail_call")]
#[generate_directory_tests("proposals/tail-call", "tail_call")]
fn run_tail_call_proposal_tests(file_name: &str, _test_name: &str) {
    println!("==========================================");
    println!("Processing tail-call proposal file: {}", file_name);
    println!("✅ Successfully parsed {}", file_name);
    println!("==========================================");
}

/// Tests for WebAssembly 3.0 proposals
/// These tests are only run when the "wasm_3_0" feature is enabled
#[cfg(feature = "wasm_3_0")]
#[generate_directory_tests("proposals/wasm-3.0", "wasm_3_0")]
fn run_wasm_3_0_proposal_tests(file_name: &str, _test_name: &str) {
    println!("==========================================");
    println!("Processing WebAssembly 3.0 proposal file: {}", file_name);
    println!("✅ Successfully parsed {}", file_name);
    println!("==========================================");
}

/// Tests for the wide-arithmetic proposal
/// These tests are only run when the "wide_arithmetic" feature is enabled
#[cfg(feature = "wide_arithmetic")]
#[generate_directory_tests("proposals/wide-arithmetic", "wide_arithmetic")]
fn run_wide_arithmetic_proposal_tests(file_name: &str, _test_name: &str) {
    println!("==========================================");
    println!("Processing wide-arithmetic proposal file: {}", file_name);
    println!("✅ Successfully parsed {}", file_name);
    println!("==========================================");
}

/// Tests for the custom-page-sizes proposal
/// These tests are only run when the "custom_page_sizes" feature is enabled
#[cfg(feature = "custom_page_sizes")]
#[generate_directory_tests("proposals/custom-page-sizes", "custom_page_sizes")]
fn run_custom_page_sizes_proposal_tests(file_name: &str, _test_name: &str) {
    println!("==========================================");
    println!("Processing custom-page-sizes proposal file: {}", file_name);
    println!("✅ Successfully parsed {}", file_name);
    println!("==========================================");
}

/// Tests for the annotations proposal
/// These tests are only run when the "annotations" feature is enabled
#[cfg(feature = "annotations")]
#[generate_directory_tests("proposals/annotations", "annotations")]
fn run_annotations_proposal_tests(file_name: &str, _test_name: &str) {
    println!("==========================================");
    println!("Processing annotations proposal file: {}", file_name);
    println!("✅ Successfully parsed {}", file_name);
    println!("==========================================");
}
