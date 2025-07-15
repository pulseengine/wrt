//! Test program for the minimal WAST execution engine
//!
//! This demonstrates the basic functionality of the simplified WAST engine
//! using StacklessEngine directly for real execution.

use anyhow::Result;

fn main() -> Result<()> {
    println!("Testing Minimal WAST Execution Engine");
    println!("====================================");

    // Test 1: Simple constant function
    test_simple_constant()?;
    
    // Test 2: Basic arithmetic
    test_basic_arithmetic()?;
    
    println!("\nAll tests completed!");
    Ok(())
}

fn test_simple_constant() -> Result<()> {
    println!("\n1. Testing simple constant function...");
    
    let wast_content = r#"
        (module
          (func (export "get_five") (result i32)
            i32.const 5
          )
        )
        (assert_return (invoke "get_five") (i32.const 5))
    "#;

    match wrt_build_core::wast_execution::run_simple_wast_test(wast_content) {
        Ok(_) => println!("   ✓ Simple constant test PASSED"),
        Err(e) => println!("   ✗ Simple constant test FAILED: {}", e),
    }
    
    Ok(())
}

fn test_basic_arithmetic() -> Result<()> {
    println!("\n2. Testing basic arithmetic...");
    
    let wast_content = r#"
        (module
          (func (export "add") (param i32 i32) (result i32)
            local.get 0
            local.get 1
            i32.add
          )
        )
        (assert_return (invoke "add" (i32.const 3) (i32.const 4)) (i32.const 7))
    "#;

    match wrt_build_core::wast_execution::run_simple_wast_test(wast_content) {
        Ok(_) => println!("   ✓ Basic arithmetic test PASSED"),
        Err(e) => println!("   ✗ Basic arithmetic test FAILED: {}", e),
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let result = wrt_build_core::wast_execution::WastEngine::new();
        assert!(result.is_ok(), "Engine creation should succeed");
    }

    #[test] 
    fn test_value_conversion() {
        use wast::{WastArg, core::WastArgCore};
        use wrt_foundation::Value;

        let wast_arg = WastArg::Core(WastArgCore::I32(42));
        let result = wrt_build_core::wast_execution::convert_wast_arg_to_value(&wast_arg);
        
        assert!(result.is_ok(), "Value conversion should succeed");
        assert_eq!(result.unwrap(), Value::I32(42));
    }
}