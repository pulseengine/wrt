use std::fs;
#[cfg(test)]
use std::path::{Path, PathBuf};
use std::sync::Once;
use wrt::{Engine, Error, Module, Value};

// Initialize the test suite once
static TESTSUITE_INIT: Once = Once::new();
static mut TESTSUITE_PATH: Option<PathBuf> = None;
static mut TESTSUITE_COMMIT: Option<String> = None;

/// Initialize the testsuite
fn init_testsuite() {
    TESTSUITE_INIT.call_once(|| {
        let testsuite_path = match std::env::var("WASM_TESTSUITE") {
            Ok(path) => PathBuf::from(path),
            Err(_) => {
                println!("WASM_TESTSUITE environment variable not set");
                println!("Using fallback path");
                Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/testsuite")
            }
        };

        let commit_hash =
            std::env::var("WASM_TESTSUITE_COMMIT").unwrap_or_else(|_| "main".to_string());

        unsafe {
            TESTSUITE_PATH = Some(testsuite_path);
            TESTSUITE_COMMIT = Some(commit_hash.clone());
        }

        println!("Initialized testsuite at commit: {}", commit_hash);
    });
}

/// Get path to the test file
fn get_test_file_path(subdir: &str, filename: &str) -> PathBuf {
    let testsuite_path =
        unsafe { TESTSUITE_PATH.as_ref() }.expect("Testsuite path not initialized");
    testsuite_path.join(subdir).join(filename)
}

// Define a Result type that uses wrt::Error
type Result<T> = std::result::Result<T, Error>;

#[test]
fn test_simple_arithmetic() -> Result<()> {
    // WAT code for a simple WebAssembly module that adds two numbers
    let wat_code = r#"
    (module
      (func $add (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.add
      )
      (export "add" (func $add))
    )
    "#;

    // Parse the WebAssembly text format
    let wasm_binary = wat::parse_str(wat_code)
        .map_err(|e| Error::Parse(format!("Failed to parse WAT: {}", e)))?;

    // Create a module
    let module = Module::new();
    let module = module.load_from_binary(&wasm_binary)?;

    // Create an engine
    let mut engine = Engine::new(module.clone());

    // Instantiate the module
    engine.instantiate(module)?;

    // Call the add function with test values: (5, 7)
    // Expected result: 5 + 7 = 12
    let args = vec![Value::I32(5), Value::I32(7)];

    let result = engine.execute(0, 0, args)?;

    // Check the result
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], Value::I32(12));

    println!("Basic arithmetic test passed successfully");
    Ok(())
}

/// Run the I32 arithmetic test directly from a WAT file
#[test]
fn test_i32_arithmetic_wat() -> Result<()> {
    // A simple WAT module defining i32 operations
    let wat_code = r#"
    (module
      (func $add (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.add
      )
      (func $sub (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.sub
      )
      (func $mul (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.mul
      )
      (export "add" (func $add))
      (export "sub" (func $sub))
      (export "mul" (func $mul))
    )
    "#;

    // Parse the WebAssembly text format
    let wasm_binary = wat::parse_str(wat_code)
        .map_err(|e| Error::Parse(format!("Failed to parse WAT: {}", e)))?;

    // Create and load the module
    let module = Module::new();
    let module = module.load_from_binary(&wasm_binary)?;

    // Create an engine
    let mut engine = Engine::new(module.clone());

    // Instantiate the module
    engine.instantiate(module)?;

    // Test add function: 5 + 7 = 12
    let args = vec![Value::I32(5), Value::I32(7)];
    let result = engine.execute(0, 0, args)?;
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], Value::I32(12));
    println!("add test: 5 + 7 = 12 ✅");

    // Test sub function: 10 - 3 = 7
    let args = vec![Value::I32(10), Value::I32(3)];
    let result = engine.execute(0, 1, args)?;
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], Value::I32(7));
    println!("sub test: 10 - 3 = 7 ✅");

    // Test mul function: 3 * 4 = 12
    let args = vec![Value::I32(3), Value::I32(4)];
    let result = engine.execute(0, 2, args)?;
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], Value::I32(12));
    println!("mul test: 3 * 4 = 12 ✅");

    println!("All i32 arithmetic tests passed successfully");
    Ok(())
}

/// Test WAST file by executing instructions from binary module
#[test]
fn test_binary_module_from_wast() -> Result<()> {
    // Create a simplified i32 arithmetic test in a temp file
    let temp_dir = tempfile::tempdir()
        .map_err(|e| Error::Parse(format!("Failed to create temp directory: {}", e)))?;

    // Create a simple WAT file with i32 operations
    let wat_content = r#"
    (module
      (func $add (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.add
      )
      (func $sub (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.sub
      )
      (export "add" (func $add))
      (export "sub" (func $sub))
    )
    "#;

    let wat_path = temp_dir.path().join("simple_module.wat");
    fs::write(&wat_path, wat_content)
        .map_err(|e| Error::Parse(format!("Failed to write WAT file: {}", e)))?;

    // Convert WAT to binary
    let wasm_binary = wat::parse_file(&wat_path)
        .map_err(|e| Error::Parse(format!("Failed to parse WAT file: {}", e)))?;

    // Create and load the module
    let module = Module::new();
    let module = module.load_from_binary(&wasm_binary)?;

    // Create an engine
    let mut engine = Engine::new(module.clone());

    // Instantiate the module
    engine.instantiate(module)?;

    // Test the add operation
    let args = vec![Value::I32(8), Value::I32(9)];
    let result = engine.execute(0, 0, args)?;
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], Value::I32(17));
    println!("add test: 8 + 9 = 17 ✅");

    // Test the sub operation
    let args = vec![Value::I32(20), Value::I32(5)];
    let result = engine.execute(0, 1, args)?;
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], Value::I32(15));
    println!("sub test: 20 - 5 = 15 ✅");

    println!("All binary module tests passed successfully");
    Ok(())
}

/// Tests for SIMD operations
#[test]
#[ignore = "SIMD tests need implementing"]
fn test_basic_simd_operations() -> Result<()> {
    // A simple WAT module defining SIMD operations
    let wat_code = r#"
    (module
      (func $i32x4_splat (param $x i32) (result v128)
        local.get $x
        i32x4.splat
      )
      (export "i32x4_splat" (func $i32x4_splat))
    )
    "#;

    // Parse the WebAssembly text format
    let wasm_binary = wat::parse_str(wat_code)
        .map_err(|e| Error::Parse(format!("Failed to parse WAT: {}", e)))?;

    // Create and load the module
    let module = Module::new();
    let module = module.load_from_binary(&wasm_binary)?;

    // Create an engine
    let mut engine = Engine::new(module.clone());

    // Instantiate the module
    engine.instantiate(module)?;

    println!("SIMD module loaded and instantiated successfully");

    // Note: We're not executing the SIMD operations yet, as they require more implementation

    Ok(())
}

/// Run all WAST tests - stub for future development
#[test]
#[ignore = "The WAST test infrastructure needs implementation"]
fn run_wast_tests() -> Result<()> {
    init_testsuite();

    println!("WAST test infrastructure will be implemented in a future update");

    Ok(())
}
