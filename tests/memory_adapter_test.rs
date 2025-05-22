//! Tests for memory adapter implementation
//!
//! These tests validate that our memory adapter implementations work
//! correctly with the safe memory structures.

use std::sync::Arc;
use wrt_error::Result;
use wrt_runtime::memory::Memory;
use wrt_runtime::types::MemoryType;
use wrt_foundation::safe_memory::MemoryProvider;
use wrt_foundation::types::Limits;
use wrt_foundation::verification::VerificationLevel;

// Import memory adapters
use wrt::memory_adapter::{DefaultMemoryAdapter, MemoryAdapter, SafeMemoryAdapter};

#[test]
fn test_safe_memory_adapter() -> Result<()> {
    // Create a memory type with 1 page (64KB)
    let mem_type = MemoryType {
        limits: Limits { min: 1, max: Some(2) },
    };
    
    // Create a memory instance
    let memory = Memory::new(mem_type)?;
    let memory_arc = Arc::new(memory);
    
    // Create the safe memory adapter
    let adapter = SafeMemoryAdapter::new(memory_arc.clone())?;
    
    // Test data
    let test_data = [1, 2, 3, 4, 5];
    
    // Store data
    adapter.store(0, &test_data)?;
    
    // Load data
    let loaded_data = adapter.load(0, test_data.len())?;
    assert_eq!(loaded_data, test_data);
    
    // Get the size
    let size = adapter.size()?;
    assert_eq!(size, 65536); // 1 page = 64KB
    
    // Test alternate method name
    assert_eq!(adapter.byte_size()?, size);
    
    // Verify access check works
    adapter.memory_provider().verify_access(0, test_data.len())?;
    
    // Get the memory
    let mem = adapter.memory();
    assert_eq!(mem.size(), 1);
    
    Ok(())
}

#[test]
fn test_safe_memory_adapter_with_verification_level() -> Result<()> {
    // Create a memory type with 1 page (64KB)
    let mem_type = MemoryType {
        limits: Limits { min: 1, max: Some(2) },
    };
    
    // Create a memory instance
    let memory = Memory::new(mem_type)?;
    let memory_arc = Arc::new(memory);
    
    // Create the safe memory adapter with full verification
    let mut adapter = SafeMemoryAdapter::with_verification_level(
        memory_arc.clone(),
        VerificationLevel::Full
    )?;
    
    // Verify the verification level
    assert_eq!(adapter.verification_level(), VerificationLevel::Full);
    
    // Test data
    let test_data = [5, 10, 15, 20, 25];
    
    // Store data with full verification
    adapter.store(10, &test_data)?;
    
    // Load data with full verification
    let loaded_data = adapter.load(10, test_data.len())?;
    assert_eq!(loaded_data, test_data);
    
    // Grow memory
    let old_pages = adapter.grow(1)?;
    assert_eq!(old_pages, 1);
    
    // Verify new size
    assert_eq!(adapter.size()?, 65536 * 2); // Now 2 pages
    
    Ok(())
}

#[test]
fn test_default_memory_adapter_with_safety() -> Result<()> {
    // Create a memory type with 1 page (64KB)
    let mem_type = MemoryType {
        limits: Limits { min: 1, max: Some(2) },
    };
    
    // Create a memory instance
    let memory = Memory::new(mem_type)?;
    let memory_arc = Arc::new(memory);
    
    // Create the default memory adapter with safety features
    let adapter = DefaultMemoryAdapter::with_safety(memory_arc.clone())?;
    
    // Verify safety provider is available
    assert!(adapter.safety_provider().is_some());
    
    // Test data
    let test_data = [10, 20, 30, 40, 50];
    
    // Store data with safety checks
    adapter.store(20, &test_data)?;
    
    // Load data with safety checks
    let loaded_data = adapter.load(20, test_data.len())?;
    assert_eq!(loaded_data, test_data);
    
    // Get memory
    let mem = adapter.memory();
    assert_eq!(mem.size(), 1);
    
    // Grow memory
    let old_size = adapter.grow(1)?;
    assert_eq!(old_size, 1);
    
    Ok(())
}

#[test]
fn test_default_memory_adapter_without_safety() -> Result<()> {
    // Create a memory type with 1 page (64KB)
    let mem_type = MemoryType {
        limits: Limits { min: 1, max: Some(2) },
    };
    
    // Create a memory instance
    let memory = Memory::new(mem_type)?;
    let memory_arc = Arc::new(memory);
    
    // Create the default memory adapter without safety features
    let adapter = DefaultMemoryAdapter::new(memory_arc.clone());
    
    // Verify safety provider is not available
    assert!(adapter.safety_provider().is_none());
    
    // Test data
    let test_data = [15, 25, 35, 45, 55];
    
    // Store data without safety checks
    adapter.store(30, &test_data)?;
    
    // Load data without safety checks
    let loaded_data = adapter.load(30, test_data.len())?;
    assert_eq!(loaded_data, test_data);
    
    // Verify out of bounds checks still work
    let result = adapter.load(65536, 10);
    assert!(result.is_err());
    
    Ok(())
} 