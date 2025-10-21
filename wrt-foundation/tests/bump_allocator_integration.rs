//! Integration tests for bump allocator with Vec
//!
//! These tests verify that the GlobalAlloc implementation works correctly
//! with standard Rust collections like Vec, and that scope-based memory
//! management functions as expected.

use wrt_foundation::{
    capabilities::MemoryFactory,
    budget_aware_provider::CrateId,
    verified_allocator::{global_allocators, MAX_MODULE_SIZE},
};

// Use our test allocator for these tests
#[global_allocator]
static GLOBAL: global_allocators::TestAllocator = global_allocators::TestAllocator;

#[test]
fn test_vec_basic_usage() {
    // Get the allocator for testing
    let allocator = global_allocators::get_crate_allocator(CrateId::Foundation);
    let initial_offset = allocator.current_offset();

    // Enter a scope
    let _scope = allocator.enter_scope(CrateId::Foundation, 4096).unwrap();

    // Create a Vec - should use our allocator
    let mut vec: Vec<u32> = Vec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);

    // Memory should have been allocated
    assert!(allocator.current_offset() > initial_offset);

    // Scope drops here, memory resets
    drop(_scope);

    // Memory should be reset to initial state
    assert_eq!(allocator.current_offset(), initial_offset);
}

#[test]
fn test_vec_with_capacity() {
    let allocator = global_allocators::get_crate_allocator(CrateId::Foundation);
    let initial_offset = allocator.current_offset();

    let _scope = allocator.enter_scope(CrateId::Foundation, 8192).unwrap();

    // Pre-allocate capacity
    let mut vec: Vec<u64> = Vec::with_capacity(100);
    let after_allocation = allocator.current_offset();

    // Should have allocated space for 100 u64s (800 bytes + overhead)
    assert!(after_allocation > initial_offset);
    assert!(after_allocation - initial_offset >= 800);

    // Fill the vector
    for i in 0..100 {
        vec.push(i);
    }

    // Should not have allocated much more (already had capacity)
    assert!(allocator.current_offset() <= after_allocation + 100);

    drop(_scope);
    assert_eq!(allocator.current_offset(), initial_offset);
}

#[test]
fn test_module_scope_basic() {
    let allocator = global_allocators::get_crate_allocator(CrateId::Foundation);
    let initial_offset = allocator.current_offset();

    // Use the high-level API (still uses Foundation allocator for Vec)
    {
        let _scope = MemoryFactory::enter_module_scope(CrateId::Foundation).unwrap();

        // Simulate module parsing
        let mut functions: Vec<String> = Vec::new();
        let mut imports: Vec<String> = Vec::new();
        let mut exports: Vec<String> = Vec::new();

        // Add some data
        for i in 0..10 {
            functions.push(format!("function_{}", i));
            imports.push(format!("import_{}", i));
            exports.push(format!("export_{}", i));
        }

        assert_eq!(functions.len(), 10);
        assert_eq!(imports.len(), 10);
        assert_eq!(exports.len(), 10);

        // Memory should be allocated
        assert!(allocator.current_offset() > initial_offset);
    }

    // After scope exit, memory should reset
    assert_eq!(allocator.current_offset(), initial_offset);
}

#[test]
fn test_scope_budget_enforcement() {
    let allocator = global_allocators::get_crate_allocator(CrateId::Runtime);
    let initial_offset = allocator.current_offset();

    // Create a scope with very small budget (1 KB)
    let _scope = allocator.enter_scope(CrateId::Runtime, 1024).unwrap();

    // Try to allocate a small Vec - should succeed
    let mut small_vec: Vec<u8> = Vec::with_capacity(100);
    for i in 0..100 {
        small_vec.push(i);
    }
    assert_eq!(small_vec.len(), 100);

    // Try to allocate a large Vec - will fail when budget exceeded
    // Note: Vec allocation failure typically panics, so we can't easily test
    // the failure case without catching panics. The allocator will return
    // null and Vec will handle it.

    drop(_scope);
    assert_eq!(allocator.current_offset(), initial_offset);
}

#[test]
fn test_nested_scopes() {
    // Note: Vec uses Foundation allocator (index 0) due to #[global_allocator]
    let allocator = global_allocators::get_crate_allocator(CrateId::Foundation);
    let initial_offset = allocator.current_offset();

    // Outer scope
    let outer_scope = allocator.enter_scope(CrateId::Foundation, 16384).unwrap();
    let mut outer_vec: Vec<u32> = Vec::with_capacity(100);
    for i in 0..100 {
        outer_vec.push(i);
    }
    let after_outer = allocator.current_offset();
    assert!(after_outer > initial_offset);

    {
        // Inner scope
        let inner_scope = allocator.enter_scope(CrateId::Foundation, 4096).unwrap();
        let mut inner_vec: Vec<u32> = Vec::with_capacity(50);
        for i in 0..50 {
            inner_vec.push(i);
        }
        let after_inner = allocator.current_offset();
        assert!(after_inner > after_outer);

        // Inner scope exits
        drop(inner_scope);

        // Should reset to after_outer
        assert_eq!(allocator.current_offset(), after_outer);
    }

    // Outer scope still valid, can still use outer_vec
    assert_eq!(outer_vec.len(), 100);
    assert_eq!(outer_vec[50], 50);

    // Outer scope exits
    drop(outer_scope);

    // Should reset to initial
    assert_eq!(allocator.current_offset(), initial_offset);
}

#[test]
fn test_multiple_vecs_same_scope() {
    let allocator = global_allocators::get_crate_allocator(CrateId::Foundation);
    let initial_offset = allocator.current_offset();

    let _scope = allocator.enter_scope(CrateId::Foundation, MAX_MODULE_SIZE).unwrap();

    // Create multiple Vecs of different types
    let mut vec_u32: Vec<u32> = Vec::new();
    let mut vec_u64: Vec<u64> = Vec::new();
    let mut vec_string: Vec<String> = Vec::new();

    // Fill them
    for i in 0..50 {
        vec_u32.push(i);
        vec_u64.push(i as u64);
        vec_string.push(format!("item_{}", i));
    }

    assert_eq!(vec_u32.len(), 50);
    assert_eq!(vec_u64.len(), 50);
    assert_eq!(vec_string.len(), 50);

    // All should be allocated from the same scope
    assert!(allocator.current_offset() > initial_offset);

    drop(_scope);
    assert_eq!(allocator.current_offset(), initial_offset);
}

#[test]
fn test_scope_reuse() {
    let allocator = global_allocators::get_crate_allocator(CrateId::Foundation);
    let initial_offset = allocator.current_offset();

    // First scope
    {
        let _scope = allocator.enter_scope(CrateId::Foundation, 8192).unwrap();
        let mut vec1: Vec<u32> = Vec::with_capacity(100);
        for i in 0..100 {
            vec1.push(i);
        }
        let after_first = allocator.current_offset();
        assert!(after_first > initial_offset);
    }
    // First scope exits
    assert_eq!(allocator.current_offset(), initial_offset);

    // Second scope - should reuse the same memory
    {
        let _scope = allocator.enter_scope(CrateId::Foundation, 8192).unwrap();
        let mut vec2: Vec<u32> = Vec::with_capacity(100);
        for i in 0..100 {
            vec2.push(i * 2);
        }
        let after_second = allocator.current_offset();

        // Should allocate roughly the same amount
        assert!(after_second > initial_offset);
        // Memory is reused, so we're starting from the same offset
    }
    // Second scope exits
    assert_eq!(allocator.current_offset(), initial_offset);
}

#[test]
fn test_custom_budget_scope() {
    let allocator = global_allocators::get_crate_allocator(CrateId::Foundation);
    let initial_offset = allocator.current_offset();

    // Use MemoryFactory with custom budget
    {
        let _scope = MemoryFactory::enter_scope(CrateId::Foundation, 2048).unwrap();

        let mut vec: Vec<u8> = Vec::with_capacity(1000);
        for i in 0..1000 {
            vec.push(i as u8);
        }

        assert_eq!(vec.len(), 1000);
        assert!(allocator.current_offset() > initial_offset);
    }

    assert_eq!(allocator.current_offset(), initial_offset);
}

#[test]
fn test_vec_growth() {
    let allocator = global_allocators::get_crate_allocator(CrateId::Foundation);
    let initial_offset = allocator.current_offset();

    let _scope = allocator.enter_scope(CrateId::Foundation, 16384).unwrap();

    // Start with small capacity
    let mut vec: Vec<u32> = Vec::with_capacity(10);
    let after_initial = allocator.current_offset();

    // Grow beyond initial capacity
    for i in 0..100 {
        vec.push(i);
    }

    // Should have allocated more memory for growth
    let after_growth = allocator.current_offset();
    assert!(after_growth > after_initial);
    assert_eq!(vec.len(), 100);

    drop(_scope);
    assert_eq!(allocator.current_offset(), initial_offset);
}

#[test]
fn test_empty_scope() {
    let allocator = global_allocators::get_crate_allocator(CrateId::Foundation);
    let initial_offset = allocator.current_offset();

    // Enter and exit scope without allocating
    {
        let _scope = allocator.enter_scope(CrateId::Foundation, 4096).unwrap();
        // Do nothing
    }

    // Should still be at initial offset
    assert_eq!(allocator.current_offset(), initial_offset);
}

#[test]
fn test_available_memory() {
    let allocator = global_allocators::get_crate_allocator(CrateId::Foundation);
    let initial_available = allocator.available();

    let _scope = allocator.enter_scope(CrateId::Foundation, 8192).unwrap();

    // Allocate some memory
    let mut vec: Vec<u64> = Vec::with_capacity(100);
    for i in 0..100 {
        vec.push(i);
    }

    // Available should decrease
    let after_allocation = allocator.available();
    assert!(after_allocation < initial_available);

    drop(_scope);

    // Available should return to initial
    assert_eq!(allocator.available(), initial_available);
}

/// Simulate a realistic module parsing scenario
#[test]
fn test_realistic_module_parsing() {
    let allocator = global_allocators::get_crate_allocator(CrateId::Foundation);
    let initial_offset = allocator.current_offset();

    // Enter module parsing scope
    let _scope = MemoryFactory::enter_module_scope(CrateId::Foundation).unwrap();

    // Simulate parsing different sections
    #[derive(Debug, Clone)]
    struct Function {
        name: String,
        params: Vec<String>,
        locals: Vec<String>,
    }

    #[derive(Debug, Clone)]
    struct Import {
        module: String,
        name: String,
    }

    let mut functions: Vec<Function> = Vec::new();
    let mut imports: Vec<Import> = Vec::new();
    let mut exports: Vec<String> = Vec::new();

    // Parse functions
    for i in 0..5 {
        let func = Function {
            name: format!("func_{}", i),
            params: vec![format!("param_0"), format!("param_1")],
            locals: vec![format!("local_0")],
        };
        functions.push(func);
    }

    // Parse imports
    for i in 0..3 {
        let import = Import {
            module: format!("module_{}", i),
            name: format!("import_{}", i),
        };
        imports.push(import);
    }

    // Parse exports
    for i in 0..2 {
        exports.push(format!("export_{}", i));
    }

    // Verify data
    assert_eq!(functions.len(), 5);
    assert_eq!(imports.len(), 3);
    assert_eq!(exports.len(), 2);
    assert_eq!(functions[0].params.len(), 2);

    // Memory should be allocated
    assert!(allocator.current_offset() > initial_offset);

    // Scope exits, all memory reclaimed
    drop(_scope);
    assert_eq!(allocator.current_offset(), initial_offset);
}
