use wrt_runtime::{Memory, MemoryType, Table, TableType};
use wrt_foundation::safe_memory::{SafeMemoryHandler, SafeStack};
use wrt_foundation::types::{ExternType, Limits, ValType};
use wrt_foundation::values::Value;
use wrt_foundation::verification::VerificationLevel;
use wrt_error::Result;

// Integration test to verify safe memory implementation works across components
#[test]
fn test_safe_memory_integration() -> Result<()> {
    // Create memory with full verification
    let mem_type = MemoryType {
        limits: Limits { min: 1, max: Some(2) },
    };
    let mut memory = Memory::new(mem_type)?;
    memory.set_verification_level(VerificationLevel::Full);
    
    // Create table with full verification
    let table_type = TableType {
        element_type: ExternType::Func,
        limits: Limits { min: 10, max: Some(20) },
    };
    let mut table = Table::new(table_type)?;
    table.set_verification_level(VerificationLevel::Full);
    
    // Test memory operations
    let data = [1, 2, 3, 4, 5];
    memory.write(0, &data)?;
    
    let mut read_buf = [0; 5];
    memory.read(0, &mut read_buf)?;
    assert_eq!(read_buf, data);
    
    // Test table operations
    let value = Some(Value::func_ref(Some(42)));
    table.init_element(0, value.clone())?;
    assert_eq!(table.get(0)?, value);
    
    // Fill table with a value
    table.fill_elements(1, value.clone(), 3)?;
    assert_eq!(table.get(1)?, value);
    assert_eq!(table.get(2)?, value);
    assert_eq!(table.get(3)?, value);
    
    // Memory grow operation
    let old_pages = memory.grow(1)?;
    assert_eq!(old_pages, 1);
    assert_eq!(memory.size(), 2);
    
    // Memory fill operation
    memory.fill(100, 0xAA, 10)?;
    
    let mut read_buf = [0; 10];
    memory.read(100, &mut read_buf)?;
    assert_eq!(read_buf, [0xAA; 10]);
    
    // Print safety statistics
    println!("Memory Safety:\n{}", memory.safety_stats());
    println!("Table Safety:\n{}", table.safety_stats());
    
    Ok(())
}

// Test that memory and table correctly handle errors
#[test]
fn test_safe_memory_error_handling() -> Result<()> {
    // Create memory and table with minimal pages
    let mem_type = MemoryType {
        limits: Limits { min: 1, max: Some(1) },
    };
    let mut memory = Memory::new(mem_type)?;
    
    let table_type = TableType {
        element_type: ExternType::Func,
        limits: Limits { min: 5, max: Some(5) },
    };
    let mut table = Table::new(table_type)?;
    
    // Test memory out of bounds
    let result = memory.read(65537, &mut [0; 10]);
    assert!(result.is_err());
    
    // Test memory grow beyond max
    let result = memory.grow(1);
    assert!(result.is_err());
    
    // Test table out of bounds
    let result = table.get(5);
    assert!(result.is_err());
    
    // Test table fill out of bounds
    let result = table.fill_elements(3, Some(Value::func_ref(None)), 3);
    assert!(result.is_err());
    
    Ok(())
} 