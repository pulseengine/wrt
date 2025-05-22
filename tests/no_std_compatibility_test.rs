//! Tests for no_std compatibility of the WRT ecosystem
//!
//! This file contains tests that ensure all crates in the WRT ecosystem
//! can be used in no_std environments.

// For testing in a no_std environment
#![cfg_attr(not(feature = "std"), no_std)]

// External crate imports
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(test)]
mod tests {
    // Import necessary types for no_std environment
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::{vec, vec::Vec, string::String, format};
    
    #[cfg(feature = "std")]
    use std::{vec, vec::Vec, string::String};
    
    // Import from wrt-error
    use wrt_error::{Error, ErrorCategory, Result};
    
    // Import from wrt-foundation
    use wrt_foundation::{
        values::Value,
        ValueType,
        types::FuncType,
        component::{MemoryType, Limits, TableType, RefType},
        bounded::{BoundedVec, BoundedStack},
        resource::ResourceId,
    };
    
    // Import from wrt-format
    use wrt_format::{
        module::Module as FormatModule,
        section::Section,
    };
    
    // Import from wrt-decoder
    use wrt_decoder::conversion::{
        format_limits_to_types_limits,
        types_limits_to_format_limits,
    };
    
    // Import from wrt-runtime
    use wrt_runtime::{Memory, Table, global::Global, MemoryType as RuntimeMemoryType};
    
    // Import from wrt-instructions
    use wrt_instructions::opcodes::Opcode;
    
    #[test]
    fn test_error_no_std_compatibility() {
        // Create an error in no_std environment
        let error = Error::new(
            ErrorCategory::Core,
            1,
            "No-std test error".to_string(),
        );
        
        // Verify error properties
        assert_eq!(error.category(), ErrorCategory::Core);
        assert_eq!(error.code(), 1);
        
        // Test result type
        let result: Result<()> = Err(error);
        assert!(result.is_err());
        
        let ok_result: Result<u32> = Ok(42);
        assert!(ok_result.is_ok());
        assert_eq!(ok_result.unwrap(), 42);
    }
    
    #[test]
    fn test_types_no_std_compatibility() {
        // Test ValueType
        let i32_type = ValueType::I32;
        let i64_type = ValueType::I64;
        
        // Test equality
        assert_eq!(i32_type, ValueType::I32);
        assert_ne!(i32_type, i64_type);
        
        // Test FuncType
        let params = vec![i32_type, i64_type];
        let results = vec![i32_type];
        
        let func_type = FuncType::new(params, results);
        
        assert_eq!(func_type.params().len(), 2);
        assert_eq!(func_type.results().len(), 1);
        
        // Test Value
        let i32_val = Value::I32(42);
        let i64_val = Value::I64(84);
        
        assert_eq!(i32_val.get_type(), ValueType::I32);
        assert_eq!(i64_val.get_type(), ValueType::I64);
    }
    
    #[test]
    fn test_bounded_containers_no_std() {
        // Test BoundedVec in no_std
        let mut vec = BoundedVec::<u32, 5>::new();
        assert!(vec.push(1).is_ok());
        assert!(vec.push(2).is_ok());
        assert_eq!(vec.len(), 2);
        
        // Test BoundedStack in no_std
        let mut stack = BoundedStack::<u32, 5>::new();
        assert!(stack.push(1).is_ok());
        assert!(stack.push(2).is_ok());
        assert_eq!(stack.pop(), Some(2));
    }
    
    #[test]
    fn test_resource_no_std() {
        // Test ResourceId in no_std
        let resource_id = ResourceId::new(42);
        assert_eq!(resource_id.get(), 42);
    }
    
    #[test]
    fn test_limits_conversion_no_std() {
        // Test limits conversion in no_std
        let format_limits = wrt_format::Limits {
            min: 1,
            max: Some(2),
            memory64: false,
            shared: false,
        };
        
        let types_limits = format_limits_to_types_limits(format_limits);
        
        assert_eq!(types_limits.min, 1);
        assert_eq!(types_limits.max, Some(2));
        assert_eq!(types_limits.shared, false);
        
        let format_limits2 = types_limits_to_format_limits(types_limits);
        
        assert_eq!(format_limits2.min, 1);
        assert_eq!(format_limits2.max, Some(2));
        assert_eq!(format_limits2.shared, false);
        assert_eq!(format_limits2.memory64, false);
    }
    
    #[test]
    fn test_memory_no_std() {
        // Create memory in no_std
        let mem_type = RuntimeMemoryType {
            minimum: 1,
            maximum: Some(2),
            shared: false,
        };
        
        let memory = Memory::new(mem_type).unwrap();
        
        // Write and read memory
        let data = [1, 2, 3, 4];
        assert!(memory.write(100, &data).is_ok());
        
        let mut buffer = [0; 4];
        assert!(memory.read(100, &mut buffer).is_ok());
        
        assert_eq!(buffer, data);
    }
    
    #[test]
    fn test_opcodes_no_std() {
        // Test opcodes in no_std
        let i32_const = Opcode::I32Const;
        let i32_add = Opcode::I32Add;
        
        assert_ne!(i32_const, i32_add);
    }
    
    #[test]
    fn test_global_no_std() {
        // Test Global in no_std
        let global = Global::new(ValueType::I32, true, Value::I32(42)).unwrap();
        
        assert_eq!(global.get(), Value::I32(42));
        
        // Test mutability
        assert!(global.set(Value::I32(100)).is_ok());
        assert_eq!(global.get(), Value::I32(100));
    }
} 