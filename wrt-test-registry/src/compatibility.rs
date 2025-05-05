//! Compatibility tests between std and no_std environments
//!
//! This module provides tests that verify the proper functioning
//! of the WRT ecosystem in both std and no_std environments.

use crate::{test_case, TestRegistry, TestResult};

#[cfg(feature = "std")]
use std::println as debug_print;

#[cfg(not(feature = "std"))]
use core::println as debug_print;

/// Register all compatibility tests
pub fn register_compatibility_tests() {
    let registry = TestRegistry::global();

    // Test core dependency functionality
    registry.register(test_case!(
        name: "core_error_handling",
        features: ["std", "no_std"],
        test_fn: test_core_error_handling,
        description: "Tests error handling in both std and no_std environments"
    ));

    // Test type system compatibility
    registry.register(test_case!(
        name: "type_system_compatibility",
        features: ["std", "no_std"],
        test_fn: test_type_system_compatibility,
        description: "Tests type system compatibility across environments"
    ));

    // Test limits conversion
    registry.register(test_case!(
        name: "limits_conversion",
        features: ["std", "no_std"],
        test_fn: test_limits_conversion,
        description: "Tests proper limits conversion between format and types"
    ));

    // Test memory operations
    registry.register(test_case!(
        name: "memory_operations",
        features: ["std", "no_std"],
        test_fn: test_memory_operations,
        description: "Tests memory operations in both environments"
    ));

    // Test value operations
    registry.register(test_case!(
        name: "value_operations",
        features: ["std", "no_std"],
        test_fn: test_value_operations,
        description: "Tests value operations in both environments"
    ));

    // Add test for component model compatibility
    registry.register(test_case!(
        name: "component_model_compatibility",
        features: ["std", "no_std"],
        test_fn: test_component_model_compatibility,
        description: "Tests component model compatibility across environments"
    ));

    // Add comprehensive core compatibility test
    registry.register(test_case!(
        name: "core_std_nostd_compatibility",
        features: ["std", "no_std"],
        test_fn: test_core_std_nostd_compatibility,
        description: "Tests complete std/no_std compatibility for core WebAssembly functionality"
    ));

    // Add comprehensive component model compatibility test
    registry.register(test_case!(
        name: "component_std_nostd_compatibility",
        features: ["std", "no_std"],
        test_fn: test_component_std_nostd_compatibility,
        description: "Tests complete std/no_std compatibility for component model functionality"
    ));
}

/// Test core error handling
fn test_core_error_handling(_config: &crate::TestConfig) -> TestResult {
    // Create an error
    let error = wrt_error::Error::new(wrt_error::ErrorCategory::Core, 0, "Test error".to_string());

    // Verify category
    assert_eq!(error.category(), wrt_error::ErrorCategory::Core);

    // Create a Result with an error
    let result: wrt_error::Result<()> = Err(error);

    // Check that the is_err method works
    assert!(result.is_err());

    // Create a success result
    let ok_result: wrt_error::Result<u32> = Ok(42);

    // Check that the is_ok method works
    assert!(ok_result.is_ok());
    assert_eq!(ok_result.unwrap(), 42);

    // Test error conversion
    let parse_error = wrt_error::Error::new(
        wrt_error::ErrorCategory::Parse,
        1,
        "Parse error".to_string(),
    );

    // Test error displays properly in both environments
    let error_string = parse_error.to_string();
    assert!(error_string.contains("Parse error"));

    Ok(())
}

/// Test type system compatibility
fn test_type_system_compatibility(_config: &crate::TestConfig) -> TestResult {
    // Test ValueType functionality
    let i32_type = wrt_types::ValueType::I32;
    let i64_type = wrt_types::ValueType::I64;
    let f32_type = wrt_types::ValueType::F32;
    let f64_type = wrt_types::ValueType::F64;

    // Test equality
    assert_eq!(i32_type, wrt_types::ValueType::I32);
    assert_ne!(i32_type, i64_type);

    // Test binary conversion
    let i32_byte = wrt_types::conversion::val_type_to_binary(i32_type);
    let i64_byte = wrt_types::conversion::val_type_to_binary(i64_type);
    let f32_byte = wrt_types::conversion::val_type_to_binary(f32_type);
    let f64_byte = wrt_types::conversion::val_type_to_binary(f64_type);

    // Test conversion back
    assert_eq!(
        wrt_types::conversion::binary_to_val_type(i32_byte).unwrap(),
        i32_type
    );
    assert_eq!(
        wrt_types::conversion::binary_to_val_type(i64_byte).unwrap(),
        i64_type
    );
    assert_eq!(
        wrt_types::conversion::binary_to_val_type(f32_byte).unwrap(),
        f32_type
    );
    assert_eq!(
        wrt_types::conversion::binary_to_val_type(f64_byte).unwrap(),
        f64_type
    );

    // Test FuncType creation and equality
    #[cfg(feature = "std")]
    let param_types = vec![i32_type, i64_type];
    #[cfg(feature = "std")]
    let result_types = vec![f32_type];

    #[cfg(not(feature = "std"))]
    let param_types = alloc::vec![i32_type, i64_type];
    #[cfg(not(feature = "std"))]
    let result_types = alloc::vec![f32_type];

    let func_type = wrt_types::types::FuncType::new(param_types.clone(), result_types.clone());
    let func_type2 = wrt_types::types::FuncType::new(param_types, result_types);

    // Test equality
    assert_eq!(func_type, func_type2);

    Ok(())
}

/// Test limits conversion
fn test_limits_conversion(_config: &crate::TestConfig) -> TestResult {
    // Create format limits
    let format_limits = wrt_format::Limits {
        min: 10,
        max: Some(20),
        memory64: false,
        shared: true,
    };

    // Convert to types limits
    let types_limits = wrt_decoder::conversion::format_limits_to_types_limits(format_limits);

    // Verify fields
    assert_eq!(types_limits.min, 10);
    assert_eq!(types_limits.max, Some(20));
    assert_eq!(types_limits.shared, true);

    // Convert back to format limits
    let format_limits2 = wrt_decoder::conversion::types_limits_to_format_limits(types_limits);

    // Verify fields
    assert_eq!(format_limits2.min, 10);
    assert_eq!(format_limits2.max, Some(20));
    assert_eq!(format_limits2.shared, true);
    assert_eq!(format_limits2.memory64, false);

    // Test with large values
    let large_format_limits = wrt_format::Limits {
        min: 0xFFFFFFFF + 1, // Exceeds u32::MAX
        max: Some(0xFFFFFFFF + 2),
        memory64: true,
        shared: false,
    };

    // Convert to types limits (should truncate)
    let truncated_types_limits =
        wrt_decoder::conversion::format_limits_to_types_limits(large_format_limits);

    // Verify fields are truncated correctly
    assert_eq!(truncated_types_limits.min, 0);
    assert_eq!(truncated_types_limits.max, Some(1));
    assert_eq!(truncated_types_limits.shared, false);

    // Test component model limits conversion
    let component_limits = wrt_types::component::Limits {
        min: 5,
        max: Some(10),
    };

    // Convert to format limits
    let component_format_limits =
        wrt_decoder::conversion::component_limits_to_format_limits(component_limits);

    // Verify fields
    assert_eq!(component_format_limits.min, 5);
    assert_eq!(component_format_limits.max, Some(10));

    Ok(())
}

/// Test memory operations
fn test_memory_operations(_config: &crate::TestConfig) -> TestResult {
    // Create memory type
    let mem_type = wrt_types::component::MemoryType {
        limits: wrt_types::component::Limits {
            min: 1,
            max: Some(2),
        },
        shared: false,
    };

    // Convert to runtime memory type
    let runtime_mem_type = wrt_runtime::MemoryType {
        minimum: 1,
        maximum: Some(2),
        shared: false,
    };

    // Create memory instance
    let memory = wrt_runtime::Memory::new(runtime_mem_type).unwrap();

    // Test initial size
    assert_eq!(memory.size(), 1);

    // Test growing memory
    assert!(memory.grow(1).is_ok());
    assert_eq!(memory.size(), 2);

    // Test growing beyond maximum fails
    assert!(memory.grow(1).is_err());

    Ok(())
}

/// Test value operations
fn test_value_operations(_config: &crate::TestConfig) -> TestResult {
    // Test Value functionality
    let i32_val = wrt_types::values::Value::I32(42);
    let i64_val = wrt_types::values::Value::I64(84);
    let f32_val = wrt_types::values::Value::F32(3.14);
    let f64_val = wrt_types::values::Value::F64(2.71828);

    // Test equality
    assert_eq!(i32_val, wrt_types::values::Value::I32(42));
    assert_ne!(i32_val, i64_val);

    // Test conversion
    assert_eq!(i32_val.get_type(), wrt_types::ValueType::I32);
    assert_eq!(i64_val.get_type(), wrt_types::ValueType::I64);
    assert_eq!(f32_val.get_type(), wrt_types::ValueType::F32);
    assert_eq!(f64_val.get_type(), wrt_types::ValueType::F64);

    // Test numeric operations
    match i32_val {
        wrt_types::values::Value::I32(v) => assert_eq!(v, 42),
        _ => return Err("Expected I32 value".to_string()),
    }

    match i64_val {
        wrt_types::values::Value::I64(v) => assert_eq!(v, 84),
        _ => return Err("Expected I64 value".to_string()),
    }

    match f32_val {
        wrt_types::values::Value::F32(v) => assert!((v - 3.14).abs() < 0.001),
        _ => return Err("Expected F32 value".to_string()),
    }

    match f64_val {
        wrt_types::values::Value::F64(v) => assert!((v - 2.71828).abs() < 0.00001),
        _ => return Err("Expected F64 value".to_string()),
    }

    Ok(())
}

/// Test component model compatibility
fn test_component_model_compatibility(_config: &crate::TestConfig) -> TestResult {
    // Test ComponentType functionality
    let component_type = wrt_types::component::ComponentType::Interface;

    // Assert equality
    assert_eq!(
        component_type,
        wrt_types::component::ComponentType::Interface
    );
    assert_ne!(component_type, wrt_types::component::ComponentType::Module);

    // Test ExternType functionality
    let mem_type = wrt_types::component::MemoryType {
        limits: wrt_types::component::Limits {
            min: 1,
            max: Some(2),
        },
        shared: false,
    };

    let mem_extern_type = wrt_types::component::ExternType::Memory(mem_type);

    // Test matching
    match mem_extern_type {
        wrt_types::component::ExternType::Memory(mt) => {
            assert_eq!(mt.limits.min, 1);
            assert_eq!(mt.limits.max, Some(2));
            assert_eq!(mt.shared, false);
        }
        _ => return Err("Expected Memory extern type".to_string()),
    }

    Ok(())
}

/// Test for complete std and no_std feature compatibility in core functionality
fn test_core_std_nostd_compatibility(config: &crate::TestConfig) -> TestResult {
    debug_print!(
        "Testing core std/no_std compatibility with config: is_std={}",
        config.is_std
    );

    // Create a basic WebAssembly module
    let memory_type = wrt_types::component::MemoryType {
        limits: wrt_types::component::Limits {
            min: 1,
            max: Some(2),
        },
        shared: false,
    };

    // Convert to runtime memory type
    let runtime_mem_type = wrt_runtime::MemoryType {
        minimum: 1,
        maximum: Some(2),
        shared: false,
    };

    // Create a memory instance
    let memory = wrt_runtime::Memory::new(runtime_mem_type).unwrap();

    // Write data to memory and read it back
    let offset = 100;
    let data = [1, 2, 3, 4, 5];

    // Write to memory
    memory.write(offset, &data).unwrap();

    // Read from memory
    let mut buffer = [0; 5];
    memory.read(offset, &mut buffer).unwrap();

    // Verify data
    for i in 0..5 {
        assert_eq!(
            buffer[i], data[i],
            "Memory read/write mismatch at index {}",
            i
        );
    }

    // Create a simple module using the main WRT API
    let module = wrt::new_module().unwrap();
    assert!(
        module.exports().is_empty(),
        "New module should have no exports"
    );

    // Test error handling
    let error = wrt_error::Error::new(
        wrt_error::ErrorCategory::Core,
        1,
        "Core test error".to_string(),
    );
    assert_eq!(error.category(), wrt_error::ErrorCategory::Core);

    Ok(())
}

/// Test for complete std and no_std feature compatibility in component model functionality
fn test_component_std_nostd_compatibility(config: &crate::TestConfig) -> TestResult {
    debug_print!(
        "Testing component model std/no_std compatibility with config: is_std={}",
        config.is_std
    );

    // Create a memory type
    let mem_type = wrt_types::component::MemoryType {
        limits: wrt_types::component::Limits {
            min: 1,
            max: Some(2),
        },
        shared: false,
    };

    // Create a table type
    let table_type = wrt_types::component::TableType {
        element_type: wrt_types::component::RefType::Funcref,
        limits: wrt_types::component::Limits {
            min: 1,
            max: Some(10),
        },
    };

    // Create a global type
    let global_type = wrt_types::component::GlobalType {
        value_type: wrt_types::ValueType::I32,
        mutable: true,
    };

    // Create extern types
    let mem_extern = wrt_types::component::ExternType::Memory(mem_type);
    let table_extern = wrt_types::component::ExternType::Table(table_type);
    let global_extern = wrt_types::component::ExternType::Global(global_type);

    // Test matching
    match mem_extern {
        wrt_types::component::ExternType::Memory(mt) => {
            assert_eq!(mt.limits.min, 1);
            assert_eq!(mt.limits.max, Some(2));
            assert_eq!(mt.shared, false);
        }
        _ => return Err("Expected Memory extern type".to_string()),
    }

    match table_extern {
        wrt_types::component::ExternType::Table(tt) => {
            assert_eq!(tt.element_type, wrt_types::component::RefType::Funcref);
            assert_eq!(tt.limits.min, 1);
            assert_eq!(tt.limits.max, Some(10));
        }
        _ => return Err("Expected Table extern type".to_string()),
    }

    match global_extern {
        wrt_types::component::ExternType::Global(gt) => {
            assert_eq!(gt.value_type, wrt_types::ValueType::I32);
            assert_eq!(gt.mutable, true);
        }
        _ => return Err("Expected Global extern type".to_string()),
    }

    // Test component model resource functionality
    let resource_id = wrt_types::resource::ResourceId::new(1);
    assert_eq!(resource_id.get(), 1);

    Ok(())
}
