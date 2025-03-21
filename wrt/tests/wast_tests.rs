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
fn test_basic_simd_operations() -> Result<()> {
    // A WAT module defining various SIMD operations
    let wat = r#"
    (module
      (memory 1)
      (export "memory" (memory 0))
      
      ;; Load and store operations
      (func $test_load_store (export "test_load_store") (result v128)
        ;; Store a v128 constant in memory
        (v128.const i32x4 0x10203040 0x50607080 0x90A0B0C0 0xD0E0F0FF)
        (i32.const 0)  ;; address
        (v128.store)
        
        ;; Load it back
        (i32.const 0)  ;; address
        (v128.load)
      )
      
      ;; Splat operations - replicate a value to all lanes
      (func $i32x4_splat (export "i32x4_splat") (param i32) (result v128)
        (local.get 0)
        (i32x4.splat)
      )
      
      (func $i64x2_splat (export "i64x2_splat") (param i64) (result v128)
        (local.get 0)
        (i64x2.splat)
      )
      
      ;; Arithmetic operations
      (func $i32x4_add (export "i32x4_add") (result v128)
        (v128.const i32x4 1 2 3 4)
        (v128.const i32x4 5 6 7 8)
        (i32x4.add)
      )
      
      (func $i32x4_sub (export "i32x4_sub") (result v128)
        (v128.const i32x4 10 20 30 40)
        (v128.const i32x4 1 2 3 4)
        (i32x4.sub)
      )
      
      (func $i32x4_mul (export "i32x4_mul") (result v128)
        (v128.const i32x4 1 2 3 4)
        (v128.const i32x4 5 6 7 8)
        (i32x4.mul)
      )
      
      ;; Shuffle operation
      (func $i8x16_shuffle (export "i8x16_shuffle") (result v128)
        ;; First vector: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
        (v128.const i8x16 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15)
        
        ;; Second vector: [16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31]
        (v128.const i8x16 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31)
        
        ;; Shuffle: select lanes in reverse order, alternating between vectors
        (i8x16.shuffle 31 30 29 28 27 26 25 24 23 22 21 20 19 18 17 16)
      )
    )
    "#;

    // Parse the WebAssembly text format
    let wasm_binary =
        wat::parse_str(wat).map_err(|e| Error::Parse(format!("Failed to parse WAT: {}", e)))?;

    // Create and load the module
    let module = Module::new();
    let module = module.load_from_binary(&wasm_binary)?;

    // Create an engine
    let mut engine = Engine::new(module.clone());

    // Instantiate the module
    engine.instantiate(module)?;

    println!("SIMD module loaded and instantiated successfully");

    // Test load and store
    let result = engine.execute(0, 0, vec![])?;
    // Expected v128 value (0xD0E0F0FF_90A0B0C0_50607080_10203040 in little-endian representation)
    let expected = Value::V128(0xD0E0F0FF_90A0B0C0_50607080_10203040);
    assert_eq!(
        result,
        vec![expected.clone()],
        "v128.load/store failed: expected {:?}, got {:?}",
        expected,
        result
    );
    println!("✅ v128.load/store test passed");

    // Test i32x4.splat
    let result = engine.execute(0, 1, vec![Value::I32(0x12345678)])?;
    let expected = Value::V128(0x1234567812345678_1234567812345678);
    assert_eq!(
        result,
        vec![expected.clone()],
        "i32x4.splat failed: expected {:?}, got {:?}",
        expected,
        result
    );
    println!("✅ i32x4.splat test passed");

    // Test i64x2.splat
    let result = engine.execute(0, 2, vec![Value::I64(0x123456789ABCDEF0)])?;
    let expected = Value::V128(0x123456789ABCDEF0_123456789ABCDEF0);
    assert_eq!(
        result,
        vec![expected.clone()],
        "i64x2.splat failed: expected {:?}, got {:?}",
        expected,
        result
    );
    println!("✅ i64x2.splat test passed");

    // Test i32x4.add
    let result = engine.execute(0, 3, vec![])?;
    // Expected: [1+5, 2+6, 3+7, 4+8] = [6, 8, 10, 12]
    let expected = Value::V128(0x0000000C0000000A_0000000800000006);
    assert_eq!(
        result,
        vec![expected.clone()],
        "i32x4.add failed: expected {:?}, got {:?}",
        expected,
        result
    );
    println!("✅ i32x4.add test passed");

    // Test i32x4.sub
    let result = engine.execute(0, 4, vec![])?;
    // Expected: [10-1, 20-2, 30-3, 40-4] = [9, 18, 27, 36]
    let expected = Value::V128(0x0000002400000021_000000120000000A);
    assert_eq!(
        result,
        vec![expected.clone()],
        "i32x4.sub failed: expected {:?}, got {:?}",
        expected,
        result
    );
    println!("✅ i32x4.sub test passed");

    // Test i32x4.mul
    let result = engine.execute(0, 5, vec![])?;
    // Expected: [1*5, 2*6, 3*7, 4*8] = [5, 12, 21, 32]
    let expected = Value::V128(0x0000002000000015_000000070000000C);
    assert_eq!(
        result,
        vec![expected.clone()],
        "i32x4.mul failed: expected {:?}, got {:?}",
        expected,
        result
    );
    println!("✅ i32x4.mul test passed");

    // Test i8x16.shuffle
    let result = engine.execute(0, 6, vec![])?;
    // Expected shuffle result
    let expected = Value::V128(0x1011121314151617_18191A1B1C1D1E1F);
    assert_eq!(
        result,
        vec![expected.clone()],
        "i8x16.shuffle failed: expected {:?}, got {:?}",
        expected,
        result
    );
    println!("✅ i8x16.shuffle test passed");

    println!("All SIMD tests passed successfully!");

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

/// Test comprehensive i32 arithmetic operations
#[test]
fn test_i32_comprehensive_arithmetic() -> Result<()> {
    // A comprehensive WAT module defining all i32 arithmetic operations
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
      (func $div_s (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.div_s
      )
      (func $div_u (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.div_u
      )
      (func $rem_s (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.rem_s
      )
      (func $rem_u (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.rem_u
      )
      (func $and (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.and
      )
      (func $or (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.or
      )
      (func $xor (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.xor
      )
      (export "add" (func $add))
      (export "sub" (func $sub))
      (export "mul" (func $mul))
      (export "div_s" (func $div_s))
      (export "div_u" (func $div_u))
      (export "rem_s" (func $rem_s))
      (export "rem_u" (func $rem_u))
      (export "and" (func $and))
      (export "or" (func $or))
      (export "xor" (func $xor))
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

    // Define some test values
    let test_values = [
        (10, 5),   // Simple positive values
        (-10, 3),  // Signed negative and positive
        (7, 2),    // Non-divisible values
        (0, 5),    // Zero and positive
        (100, 10), // Large numbers
        (-8, -2),  // Both negative
    ];

    // Test all operations with the test values
    for (a, b) in test_values.iter() {
        if b == &0 {
            // Skip division by zero tests
            continue;
        }

        // Test add
        let args = vec![Value::I32(*a), Value::I32(*b)];
        let result = engine.execute(0, 0, args.clone())?;
        assert_eq!(result[0], Value::I32(a.wrapping_add(*b)));
        println!("add test: {} + {} = {} ✅", a, b, a.wrapping_add(*b));

        // Test sub
        let result = engine.execute(0, 1, args.clone())?;
        assert_eq!(result[0], Value::I32(a.wrapping_sub(*b)));
        println!("sub test: {} - {} = {} ✅", a, b, a.wrapping_sub(*b));

        // Test mul
        let result = engine.execute(0, 2, args.clone())?;
        assert_eq!(result[0], Value::I32(a.wrapping_mul(*b)));
        println!("mul test: {} * {} = {} ✅", a, b, a.wrapping_mul(*b));

        // Test div_s
        let result = engine.execute(0, 3, args.clone())?;
        assert_eq!(result[0], Value::I32(a.wrapping_div(*b)));
        println!("div_s test: {} / {} = {} ✅", a, b, a.wrapping_div(*b));

        // Test div_u
        let ua = *a as u32;
        let ub = *b as u32;
        let args = vec![Value::I32(*a), Value::I32(*b)];
        let result = engine.execute(0, 4, args.clone())?;
        assert_eq!(result[0], Value::I32((ua / ub) as i32));
        println!("div_u test: {} / {} = {} ✅", ua, ub, (ua / ub) as i32);

        // Test rem_s
        let result = engine.execute(0, 5, args.clone())?;
        assert_eq!(result[0], Value::I32(a % b));
        println!("rem_s test: {} % {} = {} ✅", a, b, a % b);

        // Test rem_u
        let result = engine.execute(0, 6, args.clone())?;
        assert_eq!(result[0], Value::I32((ua % ub) as i32));
        println!("rem_u test: {} % {} = {} ✅", ua, ub, (ua % ub) as i32);

        // Test and
        let result = engine.execute(0, 7, args.clone())?;
        assert_eq!(result[0], Value::I32(a & b));
        println!("and test: {} & {} = {} ✅", a, b, a & b);

        // Test or
        let result = engine.execute(0, 8, args.clone())?;
        assert_eq!(result[0], Value::I32(a | b));
        println!("or test: {} | {} = {} ✅", a, b, a | b);

        // Test xor
        let result = engine.execute(0, 9, args.clone())?;
        assert_eq!(result[0], Value::I32(a ^ b));
        println!("xor test: {} ^ {} = {} ✅", a, b, a ^ b);
    }

    println!("All i32 comprehensive arithmetic tests passed successfully");
    Ok(())
}

/// Tests for WebAssembly i32 compare operations
#[test]
fn test_i32_compare_operations() -> Result<()> {
    // A WAT module defining i32 comparison operations
    let wat_code = r#"
    (module
      (func $eq (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.eq
      )
      (func $ne (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.ne
      )
      (func $lt_s (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.lt_s
      )
      (func $lt_u (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.lt_u
      )
      (func $gt_s (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.gt_s
      )
      (func $gt_u (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.gt_u
      )
      (func $le_s (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.le_s
      )
      (func $le_u (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.le_u
      )
      (func $ge_s (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.ge_s
      )
      (func $ge_u (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.ge_u
      )
      (export "eq" (func $eq))
      (export "ne" (func $ne))
      (export "lt_s" (func $lt_s))
      (export "lt_u" (func $lt_u))
      (export "gt_s" (func $gt_s))
      (export "gt_u" (func $gt_u))
      (export "le_s" (func $le_s))
      (export "le_u" (func $le_u))
      (export "ge_s" (func $ge_s))
      (export "ge_u" (func $ge_u))
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

    // Define some test values
    let test_values = [
        (10, 10),   // Equal positive
        (10, 5),    // Greater positive
        (5, 10),    // Lesser positive
        (-10, -10), // Equal negative
        (-5, -10),  // Greater negative
        (-10, -5),  // Lesser negative
        (-10, 10),  // Negative and positive
        (0, 0),     // Both zero
    ];

    // Test all comparison operations with the test values
    for (a, b) in test_values.iter() {
        let args = vec![Value::I32(*a), Value::I32(*b)];

        // Test eq (equal)
        let result = engine.execute(0, 0, args.clone())?;
        let expected = Value::I32(if a == b { 1 } else { 0 });
        assert_eq!(result[0], expected);
        println!(
            "eq test: {} == {} = {} ✅",
            a,
            b,
            if a == b { 1 } else { 0 }
        );

        // Test ne (not equal)
        let result = engine.execute(0, 1, args.clone())?;
        let expected = Value::I32(if a != b { 1 } else { 0 });
        assert_eq!(result[0], expected);
        println!(
            "ne test: {} != {} = {} ✅",
            a,
            b,
            if a != b { 1 } else { 0 }
        );

        // Test lt_s (less than signed)
        let result = engine.execute(0, 2, args.clone())?;
        let expected = Value::I32(if a < b { 1 } else { 0 });
        assert_eq!(result[0], expected);
        println!(
            "lt_s test: {} < {} = {} ✅",
            a,
            b,
            if a < b { 1 } else { 0 }
        );

        // Test lt_u (less than unsigned)
        let ua = *a as u32;
        let ub = *b as u32;
        let result = engine.execute(0, 3, args.clone())?;
        let expected = Value::I32(if ua < ub { 1 } else { 0 });
        assert_eq!(result[0], expected);
        println!(
            "lt_u test: {} < {} = {} ✅",
            ua,
            ub,
            if ua < ub { 1 } else { 0 }
        );

        // Test gt_s (greater than signed)
        let result = engine.execute(0, 4, args.clone())?;
        let expected = Value::I32(if a > b { 1 } else { 0 });
        assert_eq!(result[0], expected);
        println!(
            "gt_s test: {} > {} = {} ✅",
            a,
            b,
            if a > b { 1 } else { 0 }
        );

        // Test gt_u (greater than unsigned)
        let result = engine.execute(0, 5, args.clone())?;
        let expected = Value::I32(if ua > ub { 1 } else { 0 });
        assert_eq!(result[0], expected);
        println!(
            "gt_u test: {} > {} = {} ✅",
            ua,
            ub,
            if ua > ub { 1 } else { 0 }
        );

        // Test le_s (less than or equal signed)
        let result = engine.execute(0, 6, args.clone())?;
        let expected = Value::I32(if a <= b { 1 } else { 0 });
        assert_eq!(result[0], expected);
        println!(
            "le_s test: {} <= {} = {} ✅",
            a,
            b,
            if a <= b { 1 } else { 0 }
        );

        // Test le_u (less than or equal unsigned)
        let result = engine.execute(0, 7, args.clone())?;
        let expected = Value::I32(if ua <= ub { 1 } else { 0 });
        assert_eq!(result[0], expected);
        println!(
            "le_u test: {} <= {} = {} ✅",
            ua,
            ub,
            if ua <= ub { 1 } else { 0 }
        );

        // Test ge_s (greater than or equal signed)
        let result = engine.execute(0, 8, args.clone())?;
        let expected = Value::I32(if a >= b { 1 } else { 0 });
        assert_eq!(result[0], expected);
        println!(
            "ge_s test: {} >= {} = {} ✅",
            a,
            b,
            if a >= b { 1 } else { 0 }
        );

        // Test ge_u (greater than or equal unsigned)
        let result = engine.execute(0, 9, args.clone())?;
        let expected = Value::I32(if ua >= ub { 1 } else { 0 });
        assert_eq!(result[0], expected);
        println!(
            "ge_u test: {} >= {} = {} ✅",
            ua,
            ub,
            if ua >= ub { 1 } else { 0 }
        );
    }

    println!("All i32 comparison tests passed successfully");
    Ok(())
}

/// Test basic WAST module functionality
#[test]
fn test_wast_basic_module() -> Result<()> {
    init_testsuite();

    // Create a basic WAST-style module with memory, imports and exports
    let wat_code = r#"
    (module
      (memory (export "memory") 1)
      (global $g (mut i32) (i32.const 0))
      
      (func $get_global (result i32)
        global.get $g
      )
      
      (func $set_global (param $value i32)
        local.get $value
        global.set $g
      )
      
      (func $add (param $a i32) (param $b i32) (result i32)
        local.get $a
        local.get $b
        i32.add
      )
      
      (export "get_global" (func $get_global))
      (export "set_global" (func $set_global))
      (export "add" (func $add))
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

    // Print export information
    println!("Module exports mapping:");
    // Process according to export ordering:
    // export[0] = get_global
    // export[1] = set_global
    // export[2] = add

    // Test get_global (export index 0) - no arguments
    let result = engine.execute(0, 0, vec![])?;
    assert_eq!(result[0], Value::I32(0));
    println!("get_global (initial): {} ✅", 0);

    // Test set_global (export index 1) - one argument
    let args = vec![Value::I32(42)];
    let _ = engine.execute(0, 1, args)?;

    // Verify the set_global worked by calling get_global again
    let result = engine.execute(0, 0, vec![])?;
    assert_eq!(result[0], Value::I32(42));
    println!("set_global and get_global: {} ✅", 42);

    // Test the add function (export index 2)
    let args = vec![Value::I32(7), Value::I32(8)];
    let result = engine.execute(0, 2, args)?;
    assert_eq!(result[0], Value::I32(15));
    println!("add test: 7 + 8 = 15 ✅");

    println!("All WAST basic module tests passed successfully");
    Ok(())
}

#[test]
fn test_i64_compare_operations() -> Result<()> {
    let wat = r#"
        (module
            (func (export "i64_eq") (param i64 i64) (result i32)
                local.get 0
                local.get 1
                i64.eq
            )
            (func (export "i64_ne") (param i64 i64) (result i32)
                local.get 0
                local.get 1
                i64.ne
            )
            (func (export "i64_lt_s") (param i64 i64) (result i32)
                local.get 0
                local.get 1
                i64.lt_s
            )
            (func (export "i64_lt_u") (param i64 i64) (result i32)
                local.get 0
                local.get 1
                i64.lt_u
            )
            (func (export "i64_gt_s") (param i64 i64) (result i32)
                local.get 0
                local.get 1
                i64.gt_s
            )
            (func (export "i64_gt_u") (param i64 i64) (result i32)
                local.get 0
                local.get 1
                i64.gt_u
            )
            (func (export "i64_le_s") (param i64 i64) (result i32)
                local.get 0
                local.get 1
                i64.le_s
            )
            (func (export "i64_le_u") (param i64 i64) (result i32)
                local.get 0
                local.get 1
                i64.le_u
            )
            (func (export "i64_ge_s") (param i64 i64) (result i32)
                local.get 0
                local.get 1
                i64.ge_s
            )
            (func (export "i64_ge_u") (param i64 i64) (result i32)
                local.get 0
                local.get 1
                i64.ge_u
            )
        )
    "#;

    // Parse the WebAssembly text format
    let wasm_binary =
        wat::parse_str(wat).map_err(|e| Error::Parse(format!("Failed to parse WAT: {}", e)))?;

    // Create and load the module
    let module = Module::new();
    let module = module.load_from_binary(&wasm_binary)?;

    // Create an engine
    let mut engine = Engine::new(module.clone());

    // Instantiate the module
    engine.instantiate(module)?;

    // Test i64.eq
    let args = vec![Value::I64(100), Value::I64(100)];
    let result = engine.execute(0, 0, args)?;
    assert_eq!(result, vec![Value::I32(1)]);

    let args = vec![Value::I64(100), Value::I64(101)];
    let result = engine.execute(0, 0, args)?;
    assert_eq!(result, vec![Value::I32(0)]);

    // Test i64.ne
    let args = vec![Value::I64(100), Value::I64(100)];
    let result = engine.execute(0, 1, args)?;
    assert_eq!(result, vec![Value::I32(0)]);

    let args = vec![Value::I64(100), Value::I64(101)];
    let result = engine.execute(0, 1, args)?;
    assert_eq!(result, vec![Value::I32(1)]);

    // Test i64.lt_s
    let args = vec![Value::I64(-100), Value::I64(100)];
    let result = engine.execute(0, 2, args)?;
    assert_eq!(result, vec![Value::I32(1)]);

    let args = vec![Value::I64(100), Value::I64(100)];
    let result = engine.execute(0, 2, args)?;
    assert_eq!(result, vec![Value::I32(0)]);

    // Test i64.lt_u
    let args = vec![Value::I64(100), Value::I64(200)];
    let result = engine.execute(0, 3, args)?;
    assert_eq!(result, vec![Value::I32(1)]);

    // Negative numbers are treated as large unsigned values
    let args = vec![Value::I64(-1), Value::I64(1)];
    let result = engine.execute(0, 3, args)?;
    assert_eq!(result, vec![Value::I32(0)]);

    // Test i64.gt_s
    let args = vec![Value::I64(100), Value::I64(-100)];
    let result = engine.execute(0, 4, args)?;
    assert_eq!(result, vec![Value::I32(1)]);

    let args = vec![Value::I64(100), Value::I64(100)];
    let result = engine.execute(0, 4, args)?;
    assert_eq!(result, vec![Value::I32(0)]);

    // Test i64.gt_u
    let args = vec![Value::I64(200), Value::I64(100)];
    let result = engine.execute(0, 5, args)?;
    assert_eq!(result, vec![Value::I32(1)]);

    // Negative numbers are treated as large unsigned values
    let args = vec![Value::I64(-1), Value::I64(1)];
    let result = engine.execute(0, 5, args)?;
    assert_eq!(result, vec![Value::I32(1)]);

    // Test i64.le_s
    let args = vec![Value::I64(-100), Value::I64(100)];
    let result = engine.execute(0, 6, args)?;
    assert_eq!(result, vec![Value::I32(1)]);

    let args = vec![Value::I64(100), Value::I64(100)];
    let result = engine.execute(0, 6, args)?;
    assert_eq!(result, vec![Value::I32(1)]);

    let args = vec![Value::I64(100), Value::I64(-100)];
    let result = engine.execute(0, 6, args)?;
    assert_eq!(result, vec![Value::I32(0)]);

    // Test i64.le_u
    let args = vec![Value::I64(100), Value::I64(200)];
    let result = engine.execute(0, 7, args)?;
    assert_eq!(result, vec![Value::I32(1)]);

    let args = vec![Value::I64(100), Value::I64(100)];
    let result = engine.execute(0, 7, args)?;
    assert_eq!(result, vec![Value::I32(1)]);

    // Negative numbers are treated as large unsigned values
    let args = vec![Value::I64(-1), Value::I64(1)];
    let result = engine.execute(0, 7, args)?;
    assert_eq!(result, vec![Value::I32(0)]);

    // Test i64.ge_s
    let args = vec![Value::I64(100), Value::I64(-100)];
    let result = engine.execute(0, 8, args)?;
    assert_eq!(result, vec![Value::I32(1)]);

    let args = vec![Value::I64(100), Value::I64(100)];
    let result = engine.execute(0, 8, args)?;
    assert_eq!(result, vec![Value::I32(1)]);

    let args = vec![Value::I64(-100), Value::I64(100)];
    let result = engine.execute(0, 8, args)?;
    assert_eq!(result, vec![Value::I32(0)]);

    // Test i64.ge_u
    let args = vec![Value::I64(200), Value::I64(100)];
    let result = engine.execute(0, 9, args)?;
    assert_eq!(result, vec![Value::I32(1)]);

    let args = vec![Value::I64(100), Value::I64(100)];
    let result = engine.execute(0, 9, args)?;
    assert_eq!(result, vec![Value::I32(1)]);

    // Negative numbers are treated as large unsigned values
    let args = vec![Value::I64(-1), Value::I64(1)];
    let result = engine.execute(0, 9, args)?;
    assert_eq!(result, vec![Value::I32(1)]);

    Ok(())
}
