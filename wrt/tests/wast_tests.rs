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
        let testsuite_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/testsuite");
        let commit_hash = "main".to_string();

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

/// Tests for SIMD operations
#[test]
#[ignore = "The WAST test infrastructure needs updating"]
fn run_simd_tests() -> Result<()> {
    println!("SIMD tests are disabled");
    Ok(())
}

/// Run all WAST tests
#[test]
#[ignore = "The WAST test infrastructure needs updating"]
fn run_all_wast_tests() -> Result<()> {
    println!("All WAST tests are disabled");
    Ok(())
}
