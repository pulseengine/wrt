use std::result::Result as StdResult;
use wrt_decoder::{decode, Module};
use wrt_error::Error;

// Use a custom Result type for these tests
type Result<T> = StdResult<T, Box<dyn std::error::Error>>;

#[test]
fn test_basic_module_decoding() -> Result<()> {
    // Create a simple WebAssembly module using wat
    let wasm_bytes = wat::parse_str(
        r#"
        (module
          ;; Import a function
          (import "env" "log" (func $log (param i32)))
          
          ;; Define a memory
          (memory (export "memory") 1)
          
          ;; Define a global
          (global (export "global") (mut i32) (i32.const 42))
          
          ;; Define a function
          (func (export "add") (param i32 i32) (result i32)
            local.get 0
            local.get 1
            i32.add
          )
          
          ;; Define some data
          (data (i32.const 0) "Hello, WebAssembly!")
        )
        "#,
    )?;

    // Decode the module
    let module = decode(&wasm_bytes).map_err(Box::from)?;

    // Verify the module structure
    assert_eq!(module.imports.len(), 1);
    assert_eq!(module.exports.len(), 3); // memory, global, add
    assert_eq!(module.memories.len(), 1);
    assert_eq!(module.globals.len(), 1);
    assert_eq!(module.functions.len(), 1); // Not counting the imported function
    assert_eq!(module.data.len(), 1);

    Ok(())
}

#[test]
fn test_empty_module_decoding() -> Result<()> {
    // Create an empty WebAssembly module using wat
    let wasm_bytes = wat::parse_str(r#"(module)"#)?;

    // Decode the module
    let module = decode(&wasm_bytes).map_err(Box::from)?;

    // Verify that the module is empty
    assert_eq!(module.imports.len(), 0);
    assert_eq!(module.exports.len(), 0);
    assert_eq!(module.memories.len(), 0);
    assert_eq!(module.globals.len(), 0);
    assert_eq!(module.functions.len(), 0);
    assert_eq!(module.data.len(), 0);

    Ok(())
}

#[test]
fn test_invalid_module() {
    // Test with an invalid WebAssembly binary
    let invalid_bytes = vec![0, 1, 2, 3]; // Not a valid WASM binary
    let result = decode(&invalid_bytes);
    assert!(result.is_err(), "Expected an error for invalid binary");

    // Test with a truncated WebAssembly binary
    let wasm_bytes = wat::parse_str(r#"(module)"#).unwrap();
    let truncated = &wasm_bytes[0..4]; // Just the magic bytes
    let result = decode(truncated);
    assert!(result.is_err(), "Expected an error for truncated binary");
}

#[test]
fn test_module_roundtrip() -> Result<()> {
    // Create a simple WebAssembly module
    let wasm_bytes = wat::parse_str(
        r#"
        (module
          (func (export "answer") (result i32)
            i32.const 42
          )
        )
        "#,
    )?;

    // Decode the module
    let module = decode(&wasm_bytes).map_err(Box::from)?;

    // Verify the exports count
    assert_eq!(module.exports.len(), 1);

    // Encode the module back to binary
    let encoded = module.encode().map_err(Box::from)?;

    // Decode the encoded module
    let roundtrip_module = decode(&encoded).map_err(Box::from)?;

    // Verify the exports count after roundtrip
    assert_eq!(roundtrip_module.exports.len(), 1);

    Ok(())
}
