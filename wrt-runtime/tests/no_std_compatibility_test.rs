//! Test no_std compatibility for wrt-runtime
//!
//! This file validates that the wrt-runtime crate works correctly in no_std environments.

// For testing in a no_std environment
#![cfg_attr(not(feature = "std"), no_std)]

// External crate imports
#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(test)]
mod tests {
    // Import necessary types for no_std environment
    #[cfg(all(not(feature = "std"), feature = "alloc"))]
    use alloc::{boxed::Box, format, string::String, vec, vec::Vec};

    #[cfg(feature = "std")]
    use std::{boxed::Box, string::String, vec, vec::Vec};

    // Import from wrt-runtime
    use wrt_runtime::{
        component_impl::{ComponentRuntimeImpl, DefaultHostFunctionFactory},
        component_traits::{
            ComponentInstance, ComponentRuntime, HostFunction, HostFunctionFactory,
        },
        global::Global,
        memory::Memory,
        table::Table,
        types::{
            GlobalType as RuntimeGlobalType, MemoryType as RuntimeMemoryType,
            TableType as RuntimeTableType,
        },
    };

    // Import from wrt-types
    use wrt_types::{
        safe_memory::{SafeMemoryHandler, SafeSlice},
        types::{Limits, ValueType},
        values::Value,
        verification::VerificationLevel,
    };

    #[test]
    fn test_memory_operations() {
        // Create memory
        let memory_type = RuntimeMemoryType {
            minimum: 1,
            maximum: Some(10),
            shared: false,
        };

        let memory = Memory::new(memory_type.clone()).unwrap();

        // Verify memory type
        let mem_type = memory.memory_type();
        assert_eq!(mem_type.minimum, 1);
        assert_eq!(mem_type.maximum, Some(10));
        assert_eq!(mem_type.shared, false);

        // Write memory
        let data = [1, 2, 3, 4];
        memory.write(0, &data).unwrap();

        // Read memory
        let mut buffer = [0; 4];
        memory.read(0, &mut buffer).unwrap();

        // Verify data
        assert_eq!(buffer, data);

        // Grow memory
        let old_pages = memory.grow(1).unwrap();
        assert_eq!(old_pages, 1); // Initial size was 1 page

        // Check new size
        assert_eq!(memory.size(), 2);
    }

    #[test]
    fn test_global_operations() {
        // Create global
        let global = Global::new(ValueType::I32, true, Value::I32(42)).unwrap();

        // Verify global value
        assert_eq!(global.get(), Value::I32(42));

        // Modify global
        global.set(Value::I32(100)).unwrap();

        // Verify new value
        assert_eq!(global.get(), Value::I32(100));

        // Verify global type
        let global_type = global.global_type();
        assert_eq!(global_type.value_type, ValueType::I32);
        assert_eq!(global_type.mutable, true);
    }

    #[test]
    fn test_table_operations() {
        // Create table
        let table_type = RuntimeTableType {
            element_type: ValueType::FuncRef,
            minimum: 10,
            maximum: Some(20),
        };

        let table = Table::new(table_type.clone()).unwrap();

        // Verify table type
        let tab_type = table.table_type();
        assert_eq!(tab_type.element_type, ValueType::FuncRef);
        assert_eq!(tab_type.minimum, 10);
        assert_eq!(tab_type.maximum, Some(20));

        // Set table element
        let func_ref = Value::FuncRef(5);
        table.set(0, func_ref.clone()).unwrap();

        // Get table element
        let element = table.get(0).unwrap();

        // Verify element
        assert_eq!(element, func_ref);

        // Grow table
        let old_size = table.grow(5, Value::FuncRef(0)).unwrap();
        assert_eq!(old_size, 10); // Initial size was 10

        // Check new size
        assert_eq!(table.size(), 15);
    }

    #[test]
    fn test_runtime_types() {
        // Test MemoryType
        let memory_type = RuntimeMemoryType {
            minimum: 1,
            maximum: Some(2),
            shared: false,
        };

        // Test GlobalType
        let global_type = RuntimeGlobalType {
            value_type: ValueType::I32,
            mutable: true,
        };

        // Test TableType
        let table_type = RuntimeTableType {
            element_type: ValueType::FuncRef,
            minimum: 10,
            maximum: Some(20),
        };

        // Verify different types
        assert_ne!(memory_type.minimum, table_type.minimum);
        assert_ne!(global_type.value_type, table_type.element_type);
    }
}
