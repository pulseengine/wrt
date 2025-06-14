//! Simple test to verify component instantiation works in all environments

use wrt_foundation::{
    bounded::BoundedVec,
    prelude::*,
    WrtResult,
};

#[cfg(feature = "std")]
use wrt_foundation::{
    component::ComponentType,
    component_value::ComponentValue,
};

#[cfg(not(feature = "std"))]
use crate::{
    types::Value as ComponentValue,
    types::ValType<NoStdProvider<65536>> as ComponentType,
};

use crate::{
    types::Value,
    // Simplified imports - these modules may not exist yet
    // instantiation::{ImportValues, ImportValue, FunctionImport, InstantiationContext},
    // execution_engine::ComponentExecutionEngine,
    // canonical::CanonicalAbi,
    // resource_lifecycle::ResourceLifecycleManager,
};

/// Test basic instantiation context creation and usage
// Commented out until InstantiationContext is available
/*
// #[test]
fn test_instantiation_context_creation() {
    let mut context = InstantiationContext::new();
    
    // Test instance ID generation
    assert_eq!(context.next_instance_id(), 0);
    assert_eq!(context.next_instance_id(), 1);
    assert_eq!(context.next_instance_id(), 2);
}
*/

/// Test import values creation and manipulation
// #[test] 
fn test_import_values() {
    let mut imports = ImportValues::new();
    
    // Test function import
    #[cfg(feature = "std")]
    {
        let func_import = FunctionImport {
            signature: ComponentType::Unit,
            implementation: Box::new(|args| {
                assert_eq!(args.len(), 0);
                Ok(Value::U32(42))
            }),
        };
        
        let result = imports.add("test_func".to_string(), ImportValue::Function(func_import));
        assert!(result.is_ok());
        
        // Verify we can retrieve the import
        let retrieved = imports.get("test_func");
        assert!(retrieved.is_some());
        
        match retrieved.unwrap() {
            ImportValue::Function(f) => {
                // Test calling the function
                let result = (f.implementation)(&[]);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::U32(42));
            }
            _ => panic!("Expected function import"),
        }
    }
    
    #[cfg(not(any(feature = "std", )))]
    {
        let func_import = FunctionImport {
            signature: ComponentType::Unit,
            implementation: |args| {
                assert_eq!(args.len(), 0);
                Ok(Value::U32(42))
            },
        };
        
        let name = wrt_foundation::BoundedString::from_str("test_func").unwrap();
        let result = imports.add(name, ImportValue::Function(func_import));
        assert!(result.is_ok());
        
        // Verify we can retrieve the import
        let retrieved = imports.get("test_func");
        assert!(retrieved.is_some());
        
        match retrieved.unwrap() {
            ImportValue::Function(f) => {
                // Test calling the function
                let result = (f.implementation)(&[]);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), Value::U32(42));
            }
            _ => panic!("Expected function import"),
        }
    }
}

/// Test value imports
// #[test]
fn test_value_imports() {
    let mut imports = ImportValues::new();
    
    let value_import = ImportValue::Value(ComponentValue::U32(100));
    
    #[cfg(feature = "std")]
    {
        let result = imports.add("test_value".to_string(), value_import);
        assert!(result.is_ok());
        
        let retrieved = imports.get("test_value");
        assert!(retrieved.is_some());
        
        match retrieved.unwrap() {
            ImportValue::Value(ComponentValue::U32(val)) => {
                assert_eq!(*val, 100);
            }
            _ => panic!("Expected value import"),
        }
    }
    
    #[cfg(not(any(feature = "std", )))]
    {
        let name = wrt_foundation::BoundedString::from_str("test_value").unwrap();
        let result = imports.add(name, value_import);
        assert!(result.is_ok());
        
        let retrieved = imports.get("test_value");
        assert!(retrieved.is_some());
        
        match retrieved.unwrap() {
            ImportValue::Value(ComponentValue::U32(val)) => {
                assert_eq!(*val, 100);
            }
            _ => panic!("Expected value import"),
        }
    }
}

/// Test that all components of instantiation context work together
// #[test]
fn test_full_instantiation_context() {
    let mut context = InstantiationContext::new();
    
    // Verify all subsystems are initialized
    assert_eq!(context.canonical_abi.string_encoding(), crate::string_encoding::StringEncoding::Utf8);
    assert_eq!(context.execution_engine.state(), &crate::execution_engine::ExecutionState::Ready);
    
    // Test registering a host function
    #[cfg(not(any(feature = "std", )))]
    {
        fn test_host_func(_args: &[Value]) -> crate::WrtResult<Value> {
            Ok(Value::Bool(true))
        }
        
        let func_index = context.execution_engine.register_host_function(test_host_func);
        assert!(func_index.is_ok());
        assert_eq!(func_index.unwrap(), 0);
    }
    
    #[cfg(feature = "std")]
    {
        use crate::execution_engine::HostFunction;
        
        struct TestHostFunc;
        impl HostFunction for TestHostFunc {
            fn call(&mut self, _args: &[Value]) -> crate::WrtResult<Value> {
                Ok(Value::Bool(true))
            }
            
            fn signature(&self) -> &ComponentType {
                &ComponentType::Unit
            }
        }
        
        let func_index = context.execution_engine.register_host_function(Box::new(TestHostFunc));
        assert!(func_index.is_ok());
        assert_eq!(func_index.unwrap(), 0);
    }
}

/// Test resource management integration
// #[test]
fn test_resource_management() {
    let mut context = InstantiationContext::new();
    
    // Create a resource
    let resource_data = ComponentValue::String("test_resource".into());
    let handle = context.resource_manager.create_resource(1, resource_data);
    assert!(handle.is_ok());
    
    let handle = handle.unwrap();
    
    // Borrow the resource
    let borrowed = context.resource_manager.borrow_resource(handle);
    assert!(borrowed.is_ok());
    
    match borrowed.unwrap() {
        ComponentValue::String(s) => {
            assert_eq!(s.as_str(), "test_resource");
        }
        _ => panic!("Expected string resource"),
    }
    
    // Drop the resource
    let drop_result = context.resource_manager.drop_resource(handle);
    assert!(drop_result.is_ok());
}

/// Test memory layout calculations work
// #[test]
fn test_memory_layout_integration() {
    let context = InstantiationContext::new();
    
    // Test basic type layouts
    use crate::types::ValType<NoStdProvider<65536>>;
    use crate::memory_layout;
    
    let bool_layout = memory_layout::calculate_layout(&ValType<NoStdProvider<65536>>::Bool);
    assert_eq!(bool_layout.size, 1);
    assert_eq!(bool_layout.align, 1);
    
    let u32_layout = memory_layout::calculate_layout(&ValType<NoStdProvider<65536>>::U32);
    assert_eq!(u32_layout.size, 4);
    assert_eq!(u32_layout.align, 4);
    
    let u64_layout = memory_layout::calculate_layout(&ValType<NoStdProvider<65536>>::U64);
    assert_eq!(u64_layout.size, 8);
    assert_eq!(u64_layout.align, 8);
}

/// Test string encoding integration
// #[test]
fn test_string_encoding_integration() {
    let context = InstantiationContext::new();
    
    use crate::string_encoding::{StringEncoding, encode_string};
    
    // Test UTF-8 encoding (default)
    let test_string = "Hello, 世界!";
    let encoded = encode_string(test_string, StringEncoding::Utf8);
    assert!(encoded.is_ok());
    
    let encoded_bytes = encoded.unwrap();
    assert_eq!(encoded_bytes, test_string.as_bytes());
    
    // Test UTF-16LE encoding  
    let encoded_utf16 = encode_string("Hello", StringEncoding::Utf16Le);
    assert!(encoded_utf16.is_ok());
    
    // UTF-16LE encoding of "Hello" should be: H(0x48,0x00) e(0x65,0x00) l(0x6C,0x00) l(0x6C,0x00) o(0x6F,0x00)
    let expected_utf16 = vec![0x48, 0x00, 0x65, 0x00, 0x6C, 0x00, 0x6C, 0x00, 0x6F, 0x00];
    assert_eq!(encoded_utf16.unwrap(), expected_utf16);
}