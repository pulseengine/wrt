use std::fs;
use std::path::Path;
use wrt::{Engine, Error as WrtError, Module, Result, Value};

/// Utility function to read a WebAssembly binary file
fn read_wasm_file(path: &str) -> Result<Vec<u8>> {
    match fs::read(path) {
        Ok(bytes) => Ok(bytes),
        Err(e) => Err(WrtError::Custom(format!(
            "Failed to read file {}: {}",
            path, e
        ))),
    }
}

/// Utility function to run a test with a WebAssembly module from a file
fn run_wasm_test(path: &str) -> Result<()> {
    println!("Running test from file: {}", path);

    // Read the WebAssembly binary
    let wasm_binary = read_wasm_file(path)?;

    // Load the module
    let empty_module = Module::new();
    let module = empty_module.load_from_binary(&wasm_binary)?;

    // Create an engine with the loaded module
    let mut engine = Engine::new(module.clone());

    // Instantiate the module
    engine.instantiate(module)?;

    // For simplicity, we'll just check that the module loads and instantiates without errors
    // In a real test, we would execute specific exported functions and check their results
    println!("✅ Successfully loaded and instantiated: {}", path);

    Ok(())
}

// This test is conditional because it requires access to the WebAssembly testsuite files
// It will be skipped if the WASM_TESTSUITE environment variable is not set
#[cfg(test)]
#[test]
fn test_simd_v128_load() -> Result<()> {
    // Get the test suite path from environment variable or skip the test
    let testsuite_path = match std::env::var("WASM_TESTSUITE") {
        Ok(path) => path,
        Err(_) => {
            println!("Skipping test: WASM_TESTSUITE environment variable not set");
            return Ok(());
        }
    };

    // Path to the test file
    let test_path = Path::new(&testsuite_path)
        .join("simd")
        .join("simd_load.wasm");

    // Check if the file exists
    if !test_path.exists() {
        println!("Skipping test: Test file not found at {:?}", test_path);
        return Ok(());
    }

    // Run the test
    run_wasm_test(test_path.to_str().unwrap())
}

#[cfg(test)]
#[test]
fn test_simd_v128_store() -> Result<()> {
    // Get the test suite path from environment variable or skip the test
    let testsuite_path = match std::env::var("WASM_TESTSUITE") {
        Ok(path) => path,
        Err(_) => {
            println!("Skipping test: WASM_TESTSUITE environment variable not set");
            return Ok(());
        }
    };

    // Path to the test file
    let test_path = Path::new(&testsuite_path)
        .join("simd")
        .join("simd_store.wasm");

    // Check if the file exists
    if !test_path.exists() {
        println!("Skipping test: Test file not found at {:?}", test_path);
        return Ok(());
    }

    // Run the test
    run_wasm_test(test_path.to_str().unwrap())
}

#[cfg(test)]
#[test]
fn test_simd_splat() -> Result<()> {
    // Get the test suite path from environment variable or skip the test
    let testsuite_path = match std::env::var("WASM_TESTSUITE") {
        Ok(path) => path,
        Err(_) => {
            println!("Skipping test: WASM_TESTSUITE environment variable not set");
            return Ok(());
        }
    };

    // Test i8x16_splat
    let test_path = Path::new(&testsuite_path)
        .join("simd")
        .join("simd_i8x16_splat.wasm");

    if test_path.exists() {
        run_wasm_test(test_path.to_str().unwrap())?;
    } else {
        println!("Skipping test: Test file not found at {:?}", test_path);
    }

    // Test i16x8_splat
    let test_path = Path::new(&testsuite_path)
        .join("simd")
        .join("simd_i16x8_splat.wasm");

    if test_path.exists() {
        run_wasm_test(test_path.to_str().unwrap())?;
    } else {
        println!("Skipping test: Test file not found at {:?}", test_path);
    }

    // Test i32x4_splat
    let test_path = Path::new(&testsuite_path)
        .join("simd")
        .join("simd_i32x4_splat.wasm");

    if test_path.exists() {
        run_wasm_test(test_path.to_str().unwrap())?;
    } else {
        println!("Skipping test: Test file not found at {:?}", test_path);
    }

    // Test i64x2_splat
    let test_path = Path::new(&testsuite_path)
        .join("simd")
        .join("simd_i64x2_splat.wasm");

    if test_path.exists() {
        run_wasm_test(test_path.to_str().unwrap())?;
    } else {
        println!("Skipping test: Test file not found at {:?}", test_path);
    }

    // Test f32x4_splat
    let test_path = Path::new(&testsuite_path)
        .join("simd")
        .join("simd_f32x4_splat.wasm");

    if test_path.exists() {
        run_wasm_test(test_path.to_str().unwrap())?;
    } else {
        println!("Skipping test: Test file not found at {:?}", test_path);
    }

    // Test f64x2_splat
    let test_path = Path::new(&testsuite_path)
        .join("simd")
        .join("simd_f64x2_splat.wasm");

    if test_path.exists() {
        run_wasm_test(test_path.to_str().unwrap())?;
    } else {
        println!("Skipping test: Test file not found at {:?}", test_path);
    }

    Ok(())
}

#[cfg(test)]
#[test]
fn test_simd_arithmetic() -> Result<()> {
    // Get the test suite path from environment variable or skip the test
    let testsuite_path = match std::env::var("WASM_TESTSUITE") {
        Ok(path) => path,
        Err(_) => {
            println!("Skipping test: WASM_TESTSUITE environment variable not set");
            return Ok(());
        }
    };

    // Test i32x4_add
    let test_path = Path::new(&testsuite_path)
        .join("simd")
        .join("simd_i32x4_arith.wasm");

    if test_path.exists() {
        run_wasm_test(test_path.to_str().unwrap())?;
    } else {
        println!("Skipping test: Test file not found at {:?}", test_path);
    }

    // Test f32x4_add
    let test_path = Path::new(&testsuite_path)
        .join("simd")
        .join("simd_f32x4_arith.wasm");

    if test_path.exists() {
        run_wasm_test(test_path.to_str().unwrap())?;
    } else {
        println!("Skipping test: Test file not found at {:?}", test_path);
    }

    Ok(())
}

#[cfg(test)]
#[test]
fn test_simd_bitwise() -> Result<()> {
    // Get the test suite path from environment variable or skip the test
    let testsuite_path = match std::env::var("WASM_TESTSUITE") {
        Ok(path) => path,
        Err(_) => {
            println!("Skipping test: WASM_TESTSUITE environment variable not set");
            return Ok(());
        }
    };

    // Test v128_and
    let test_path = Path::new(&testsuite_path)
        .join("simd")
        .join("simd_bitwise.wasm");

    if test_path.exists() {
        run_wasm_test(test_path.to_str().unwrap())?;
    } else {
        println!("Skipping test: Test file not found at {:?}", test_path);
    }

    Ok(())
}
