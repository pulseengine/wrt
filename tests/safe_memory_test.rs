//! Tests for safe memory implementations
//!
//! These tests validate that our safe memory structures work as expected
//! and can replace Vec usage safely.

use wrt_runtime::memory::Memory;
use wrt_runtime::table::Table;
use wrt_runtime::types::{MemoryType, TableType};
use wrt_foundation::safe_memory::{SafeMemoryHandler, SafeStack, SafeSlice};
use wrt_foundation::verification::VerificationLevel;
use wrt_foundation::types::{Limits, ValueType};
use wrt_foundation::values::Value;
use wrt_error::Result;

#[test]
fn test_memory_safe_operations() -> Result<()> {
    // Create a memory type with 1 page (64KB)
    let mem_type = MemoryType {
        limits: Limits { min: 1, max: Some(2) },
    };
    
    // Create a memory instance
    let mut memory = Memory::new(mem_type)?;
    
    // Test data to write
    let test_data = [1, 2, 3, 4, 5];
    
    // Write the data
    memory.write(0, &test_data)?;
    
    // Read it back using safe slice
    let safe_slice = memory.get_safe_slice(0, test_data.len())?;
    let slice_data = safe_slice.data()?;
    assert_eq!(slice_data, test_data);
    
    // Verify integrity
    memory.verify_integrity()?;
    
    // Test with different verification levels
    memory.set_verification_level(VerificationLevel::Full);
    assert_eq!(memory.verification_level(), VerificationLevel::Full);
    
    // Write with full verification
    memory.write(10, &test_data)?;
    
    // Read it back
    let mut buffer = [0; 5];
    memory.read(10, &mut buffer)?;
    assert_eq!(buffer, test_data);
    
    Ok(())
}

#[test]
fn test_table_with_safe_stack() -> Result<()> {
    // Create a table type
    let table_type = TableType {
        element_type: ValueType::FuncRef,
        limits: Limits { min: 5, max: Some(10) },
    };
    
    // Create a table
    let mut table = Table::new(table_type, Value::func_ref(None))?;
    
    // Set verification level
    table.set_verification_level(VerificationLevel::Full);
    assert_eq!(table.verification_level(), VerificationLevel::Full);
    
    // Set some values
    table.set(1, Some(Value::func_ref(Some(42))))?;
    table.set(2, Some(Value::func_ref(Some(43))))?;
    
    // Get them back
    let val1 = table.get(1)?;
    let val2 = table.get(2)?;
    
    // Verify values
    assert_eq!(val1, Some(Value::func_ref(Some(42))));
    assert_eq!(val2, Some(Value::func_ref(Some(43))));
    
    // Test operations
    table.fill(3, 2, Some(Value::func_ref(Some(99))))?;
    assert_eq!(table.get(3)?, Some(Value::func_ref(Some(99))));
    assert_eq!(table.get(4)?, Some(Value::func_ref(Some(99))));
    
    // Copy operation
    table.copy(0, 3, 2)?;
    assert_eq!(table.get(0)?, Some(Value::func_ref(Some(99))));
    assert_eq!(table.get(1)?, Some(Value::func_ref(Some(99))));
    
    Ok(())
}

#[test]
fn test_direct_safe_memory_handler() -> Result<()> {
    // Create a SafeMemoryHandler directly
    let mut handler = SafeMemoryHandler::with_capacity(1024);
    
    // Set verification level
    handler.set_verification_level(VerificationLevel::Full);
    assert_eq!(handler.verification_level(), VerificationLevel::Full);
    
    // Add some data
    let test_data = [10, 20, 30, 40, 50];
    handler.add_data(&test_data);
    
    // Get a safe slice
    let slice = handler.get_slice(0, test_data.len())?;
    
    // Verify data
    let data = slice.data()?;
    assert_eq!(data, test_data);
    
    // Verify integrity
    handler.verify_integrity()?;
    
    // Convert to Vec and back
    let vec = handler.to_vec()?;
    assert_eq!(vec[0..test_data.len()], test_data);
    
    // Clear and rebuild
    handler.clear();
    handler.add_data(&[1, 2, 3]);
    
    // Verify new data
    let slice = handler.get_slice(0, 3)?;
    let data = slice.data()?;
    assert_eq!(data, [1, 2, 3]);
    
    Ok(())
}

#[test]
fn test_direct_safe_stack() -> Result<()> {
    // Create a SafeStack directly
    let mut stack: SafeStack<i32> = SafeStack::with_capacity(10);
    
    // Set verification level
    stack.set_verification_level(VerificationLevel::Full);
    
    // Push some values
    stack.push(10)?;
    stack.push(20)?;
    stack.push(30)?;
    
    // Get values
    assert_eq!(stack.get(0)?, 10);
    assert_eq!(stack.get(1)?, 20);
    assert_eq!(stack.get(2)?, 30);
    
    // Peek and pop
    assert_eq!(stack.peek()?, 30);
    assert_eq!(stack.pop()?, 30);
    assert_eq!(stack.peek()?, 20);
    
    // Convert to vec and back
    let vec = stack.to_vec()?;
    assert_eq!(vec, [10, 20]);
    
    // Clear and rebuild
    stack.clear();
    assert_eq!(stack.len(), 0);
    
    // Push new values
    stack.push(100)?;
    stack.push(200)?;
    
    // Verify new state
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.get(0)?, 100);
    assert_eq!(stack.get(1)?, 200);
    
    Ok(())
} 