use wrt_types::safe_memory::{SafeStack, StdMemoryProvider};
use wrt_types::verification::VerificationLevel;

#[derive(Debug, Clone, PartialEq)]
struct TestValue {
    id: u32,
    data: u32,
}

#[test]
fn test_safe_stack_as_vec_replacement() {
    // Create a new stack with default capacity
    let mut stack = SafeStack::<TestValue>::new();

    // Push values
    let value1 = TestValue { id: 1, data: 100 };
    let value2 = TestValue { id: 2, data: 200 };
    let value3 = TestValue { id: 3, data: 300 };

    stack.push(value1.clone()).unwrap();
    stack.push(value2.clone()).unwrap();
    stack.push(value3.clone()).unwrap();

    // Check length
    assert_eq!(stack.len(), 3);
    assert!(!stack.is_empty());

    // Pop values in reverse order (like a stack)
    let popped3 = stack.pop().unwrap();
    assert_eq!(popped3, value3);

    let popped2 = stack.pop().unwrap();
    assert_eq!(popped2, value2);

    // Get the top element without removing it
    let top = stack.peek().unwrap();
    assert_eq!(top, value1);

    // Still one element left
    assert_eq!(stack.len(), 1);

    // Get by index (like a Vec)
    let first = stack.get(0).unwrap();
    assert_eq!(first, value1);

    // Clear the stack
    stack.clear();
    assert_eq!(stack.len(), 0);
    assert!(stack.is_empty());
}

#[test]
fn test_safe_stack_with_different_verification_levels() {
    // Create stacks with different verification levels
    let mut stack_none = SafeStack::<u32>::new();
    stack_none.set_verification_level(VerificationLevel::None);

    let mut stack_standard = SafeStack::<u32>::new();
    stack_standard.set_verification_level(VerificationLevel::Standard);

    let mut stack_full = SafeStack::<u32>::new();
    stack_full.set_verification_level(VerificationLevel::Full);

    // Push values to all stacks
    for i in 0..10 {
        stack_none.push(i as u32).unwrap();
        stack_standard.push(i as u32).unwrap();
        stack_full.push(i as u32).unwrap();
    }

    // All stacks should have the same content
    for i in 0..10 {
        assert_eq!(stack_none.get(i).unwrap(), i as u32);
        assert_eq!(stack_standard.get(i).unwrap(), i as u32);
        assert_eq!(stack_full.get(i).unwrap(), i as u32);
    }

    // Pop values from all stacks
    for i in (0..10).rev() {
        assert_eq!(stack_none.pop().unwrap(), i as u32);
        assert_eq!(stack_standard.pop().unwrap(), i as u32);
        assert_eq!(stack_full.pop().unwrap(), i as u32);
    }

    // All stacks should be empty
    assert!(stack_none.is_empty());
    assert!(stack_standard.is_empty());
    assert!(stack_full.is_empty());
}

#[test]
fn test_safe_stack_to_vec_conversion() {
    let mut stack = SafeStack::<u32>::new();

    // Push values
    for i in 0..5 {
        stack.push(i).unwrap();
    }

    // Convert to Vec for interoperability with existing APIs
    let vec = stack.to_vec().unwrap();

    // Verify contents
    assert_eq!(vec, vec![0, 1, 2, 3, 4]);

    // Still has original content
    assert_eq!(stack.len(), 5);

    // Use split_off for more advanced manipulation
    let rest = stack.split_off(2).unwrap();

    // Original stack is truncated
    assert_eq!(stack.len(), 2);

    // Rest contains the remaining elements
    assert_eq!(rest, vec![2, 3, 4]);
}

#[test]
fn test_safe_stack_error_handling() {
    let mut stack = SafeStack::<u32>::new();

    // Test underflow error
    let pop_result = stack.pop();
    assert!(pop_result.is_err());

    // Test out of bounds error
    let get_result = stack.get(10);
    assert!(get_result.is_err());

    // Test split error
    let split_result = stack.split_off(10);
    assert!(split_result.is_err());

    // Filling and emptying
    for i in 0..5 {
        stack.push(i).unwrap();
    }

    for _ in 0..5 {
        stack.pop().unwrap();
    }

    // Should be empty again
    assert!(stack.is_empty());
}

#[test]
fn test_safe_stack_fixed() {
    // Create a stack with fixed capacity
    let mut stack_none = SafeStack::<u32>::with_capacity(20);
    let mut stack_standard = SafeStack::<u32>::with_capacity(20);
    let mut stack_full = SafeStack::<u32>::with_capacity(20);

    // Set verification levels
    stack_none.set_verification_level(VerificationLevel::None);
    stack_standard.set_verification_level(VerificationLevel::Standard);
    stack_full.set_verification_level(VerificationLevel::Full);

    // Push some values
    for i in 0..10 {
        stack_none.push(i).unwrap();
        stack_standard.push(i).unwrap();
        stack_full.push(i).unwrap();
    }

    // Check lengths
    assert_eq!(stack_none.len(), 10);
    assert_eq!(stack_standard.len(), 10);
    assert_eq!(stack_full.len(), 10);

    // Peek at the top
    assert_eq!(stack_none.peek().unwrap(), 9);
    assert_eq!(stack_standard.peek().unwrap(), 9);
    assert_eq!(stack_full.peek().unwrap(), 9);

    // Pop values
    assert_eq!(stack_none.pop().unwrap(), 9);
    assert_eq!(stack_standard.pop().unwrap(), 9);
    assert_eq!(stack_full.pop().unwrap(), 9);

    // Check lengths again
    assert_eq!(stack_none.len(), 9);
    assert_eq!(stack_standard.len(), 9);
    assert_eq!(stack_full.len(), 9);

    // Push more values to test resizing
    for i in 10..15 {
        stack_none.push(i).unwrap();
        stack_standard.push(i).unwrap();
        stack_full.push(i).unwrap();
    }

    // Convert to vector
    let vec_none = stack_none.to_vec().unwrap();
    let vec_standard = stack_standard.to_vec().unwrap();
    let vec_full = stack_full.to_vec().unwrap();

    // Check vector contents
    assert_eq!(
        vec_none,
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12, 13, 14]
    );
    assert_eq!(
        vec_standard,
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12, 13, 14]
    );
    assert_eq!(
        vec_full,
        vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12, 13, 14]
    );
}

#[test]
fn test_safe_stack_dynamic() {
    // Create stacks that will need to resize
    let mut stack_none = SafeStack::<u32>::new();
    let mut stack_standard = SafeStack::<u32>::new();
    let mut stack_full = SafeStack::<u32>::new();

    // Set verification levels
    stack_none.set_verification_level(VerificationLevel::None);
    stack_standard.set_verification_level(VerificationLevel::Standard);
    stack_full.set_verification_level(VerificationLevel::Full);

    // Push enough values to trigger resizing
    for i in 0..20 {
        stack_none.push(i).unwrap();
        stack_standard.push(i).unwrap();
        stack_full.push(i).unwrap();
    }

    // Check random access
    for i in 0..20 {
        assert_eq!(stack_none.get(i).unwrap(), i as u32);
        assert_eq!(stack_standard.get(i).unwrap(), i as u32);
        assert_eq!(stack_full.get(i).unwrap(), i as u32);
    }

    // Set values
    stack_none.set(5, 100).unwrap();
    stack_standard.set(5, 100).unwrap();
    stack_full.set(5, 100).unwrap();

    // Check updated values
    assert_eq!(stack_none.get(5).unwrap(), 100);
    assert_eq!(stack_standard.get(5).unwrap(), 100);
    assert_eq!(stack_full.get(5).unwrap(), 100);
}

#[test]
fn test_safe_stack_error_cases() {
    // Create a stack and push some values
    let mut stack = SafeStack::<u32>::with_capacity(5);
    for i in 0..5 {
        stack.push(i).unwrap();
    }

    // Test out of bounds access
    assert!(stack.get(10).is_err());

    // Test out of bounds set
    assert!(stack.set(10, 100).is_err());

    // Test clear
    stack.clear();
    assert_eq!(stack.len(), 0);
    assert!(stack.is_empty());

    // Test pop on empty stack
    assert!(stack.pop().is_err());
    assert!(stack.peek().is_err());
}

#[test]
fn test_safe_stack_conversions() {
    // Create a stack with values
    let mut stack = SafeStack::<u32>::with_capacity(10);
    for i in 0..10 {
        stack.push(i).unwrap();
    }

    // Convert to Vec
    let vec = stack.to_vec().unwrap();
    assert_eq!(vec, (0..10).collect::<Vec<_>>());

    // Get as slice
    let slice = stack.as_slice().unwrap();
    assert_eq!(slice, (0..10).collect::<Vec<_>>());
}
