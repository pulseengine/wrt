# WebAssembly Component Model Async Features

This document provides a comprehensive guide to the async features implemented in WRT's Component Model support.

## Overview

The WRT Component Model implementation provides complete support for asynchronous operations as specified in the WebAssembly Component Model MVP. This includes context management, task orchestration, waitable sets, error handling, advanced threading, and fixed-length lists for type safety.

## Features

### 1. Async Context Management (`context.*`)

Thread-local context storage for async execution with automatic cleanup.

```rust
// Create and set a context
let context = AsyncContext::new();
AsyncContextManager::context_set(context)?;

// Store values in context
AsyncContextManager::set_context_value(
    ContextKey::new("user_id".to_string()),
    ContextValue::from_component_value(ComponentValue::I32(123))
)?;

// Retrieve values
let value = AsyncContextManager::get_context_value(&ContextKey::new("user_id"))?;

// Use scoped contexts
{
    let _scope = AsyncContextScope::enter_empty()?;
    // Context is automatically popped when scope ends
}
```

**Key Features:**
- Thread-local storage with stack-based contexts
- Automatic cleanup with RAII pattern
- Type-safe value storage
- Support for nested contexts
- Full no_std compatibility

### 2. Task Management (`task.*`)

Complete task lifecycle management with cancellation and metadata support.

```rust
// Initialize task system
TaskBuiltins::initialize()?;

// Start a task
let task_id = TaskBuiltins::task_start()?;

// Set task metadata
TaskBuiltins::set_task_metadata(task_id, "priority", ComponentValue::I32(5))?;

// Return from task
TaskBuiltins::task_return(task_id, TaskReturn::from_component_value(
    ComponentValue::Bool(true)
))?;

// Wait for completion
let result = TaskBuiltins::task_wait(task_id)?;

// Cancel a task
TaskBuiltins::task_cancel(task_id)?;
```

**Key Features:**
- Unique task IDs with atomic generation
- Task state tracking (Pending, Running, Completed, Cancelled, Failed)
- Metadata storage per task
- Integration with cancellation tokens
- Automatic cleanup of finished tasks

### 3. Waitable Sets (`waitable-set.*`)

Collective waiting on multiple async objects.

```rust
// Initialize waitable system
WaitableSetBuiltins::initialize()?;

// Create a waitable set
let set_id = WaitableSetBuiltins::waitable_set_new()?;

// Add waitables
let future = Future {
    handle: FutureHandle::new(),
    state: FutureState::Pending,
};
let waitable_id = WaitableSetBuiltins::waitable_set_add(set_id, Waitable::Future(future))?;

// Wait for any to be ready
let result = WaitableSetBuiltins::waitable_set_wait(set_id)?;
match result {
    WaitResult::Ready(entry) => { /* Handle ready waitable */ },
    WaitResult::Timeout => { /* No waitables ready */ },
    _ => { /* Handle other cases */ }
}

// Poll all ready waitables
let ready_list = WaitableSetBuiltins::waitable_set_poll_all(set_id)?;
```

**Key Features:**
- Support for futures, streams, and nested waitable sets
- Non-blocking polling
- Ready state detection
- Helper functions for common patterns
- Efficient storage with bounded collections in no_std

### 4. Error Context (`error-context.*`)

Rich error handling with stack traces and metadata.

```rust
// Initialize error system
ErrorContextBuiltins::initialize()?;

// Create error context
let context_id = ErrorContextBuiltins::error_context_new(
    "Database connection failed".to_string(),
    ErrorSeverity::Error
)?;

// Add stack frame
ErrorContextBuiltins::error_context_add_stack_frame(
    context_id,
    "connect_to_db".to_string(),
    Some("database.rs".to_string()),
    Some(142),
    Some(15)
)?;

// Add metadata
ErrorContextBuiltins::error_context_set_metadata(
    context_id,
    "database_url".to_string(),
    ComponentValue::String("postgres://localhost:5432".to_string())
)?;

// Get formatted stack trace
let stack_trace = ErrorContextBuiltins::error_context_stack_trace(context_id)?;
```

**Key Features:**
- Severity levels (Info, Warning, Error, Critical)
- Stack trace management
- Arbitrary metadata storage
- Error chaining support
- Helper functions for common error patterns

### 5. Advanced Threading (`thread.spawn_ref/indirect/join`)

Enhanced threading capabilities beyond basic spawn.

```rust
// Initialize threading system
AdvancedThreadingBuiltins::initialize()?;

// Create function reference
let func_ref = FunctionReference::new(
    "worker_function".to_string(),
    FunctionSignature {
        params: vec![ThreadValueType::I32],
        results: vec![ThreadValueType::I32],
    },
    0,  // module_index
    42  // function_index
);

// Configure thread
let config = ThreadSpawnConfig {
    stack_size: Some(65536),
    priority: Some(5),
};

// Spawn with function reference
let thread_id = AdvancedThreadingBuiltins::thread_spawn_ref(func_ref, config, None)?;

// Thread-local storage
AdvancedThreadingBuiltins::thread_local_set(
    thread_id,
    1, // key
    ComponentValue::String("thread_data".to_string()),
    Some(100) // optional destructor function index
)?;

// Join thread
let result = AdvancedThreadingBuiltins::thread_join(thread_id)?;
```

**Key Features:**
- Function reference and indirect call spawning
- Thread-local storage with destructors
- Parent-child thread relationships
- Advanced thread state management
- Thread join operations with result handling

### 6. Fixed-Length Lists

Type-safe fixed-length lists with compile-time size guarantees.

```rust
// Create fixed-length list type
let list_type = FixedLengthListType::new(ValueType::I32, 5);

// Create list instance
let mut list = FixedLengthList::new(list_type)?;

// Add elements
list.push(ComponentValue::I32(10))?;
list.push(ComponentValue::I32(20))?;

// Access elements
let value = list.get(0); // Some(&ComponentValue::I32(10))

// Use utility functions
let zeros = fixed_list_utils::zero_filled(ValueType::I32, 10)?;
let range = fixed_list_utils::from_range(0, 5)?;

// Type registry
let mut registry = FixedLengthListTypeRegistry::new();
let type_index = registry.register_type(list_type)?;
```

**Key Features:**
- Compile-time size validation
- Type-safe element access
- Mutable and immutable variants
- Utility functions (zero-fill, range, concatenate, slice)
- Component Model integration
- Type registry for reuse

## Integration Examples

### Async Context with Tasks

```rust
// Execute task within async context
let _scope = AsyncContextScope::enter_empty()?;
AsyncContextManager::set_context_value(
    ContextKey::new("operation_id".to_string()),
    ContextValue::from_component_value(ComponentValue::String("op_123".to_string()))
)?;

let task_id = TaskBuiltins::task_start()?;
// Task has access to context values
let op_id = AsyncContextManager::get_context_value(
    &ContextKey::new("operation_id")
)?;
```

### Error Handling with Tasks

```rust
let task_id = TaskBuiltins::task_start()?;

// If task fails, create detailed error context
let error_id = ErrorContextBuiltins::error_context_new(
    "Task execution failed".to_string(),
    ErrorSeverity::Error
)?;

ErrorContextBuiltins::error_context_set_metadata(
    error_id,
    "task_id".to_string(),
    ComponentValue::U64(task_id.as_u64())
)?;

TaskBuiltins::task_cancel(task_id)?;
```

### Waiting for Multiple Operations

```rust
let set_id = WaitableSetBuiltins::waitable_set_new()?;

// Add multiple futures
for future in futures {
    WaitableSetBuiltins::waitable_set_add(set_id, Waitable::Future(future))?;
}

// Wait for first to complete
match WaitableSetBuiltins::waitable_set_wait(set_id)? {
    WaitResult::Ready(entry) => {
        // Handle first ready future
    },
    _ => { /* Handle timeout or error */ }
}
```

## Environment Support

All features support three environments:

### 1. Standard (`std` feature)
- Full functionality with dynamic allocation
- Thread-local storage via `thread_local!`
- Unbounded collections

### 2. Allocation (`alloc` feature)
- Full functionality with `alloc` crate
- Global static storage for contexts
- Dynamic collections

### 3. No Standard Library (no features)
- Bounded collections with compile-time limits
- Static storage with fixed capacity
- All features available with size constraints

## Performance Considerations

- **Atomic Operations**: Task and thread IDs use atomic counters
- **Lock-Free Where Possible**: Registries use `AtomicRefCell` for minimal contention
- **Bounded Collections**: No_std mode uses fixed-size collections for predictability
- **Lazy Initialization**: Systems initialize on first use
- **Automatic Cleanup**: Finished tasks and threads are cleaned up periodically

## Testing

Comprehensive test coverage includes:
- Unit tests for each module (70+ tests)
- Integration tests across features
- Environment-specific tests (std/alloc/no_std)
- Cross-feature interaction tests
- Performance benchmarks (when enabled)

Run tests with:
```bash
# Standard tests
cargo test --features std

# Allocation-only tests  
cargo test --no-default-features --features alloc

# No_std tests
cargo test --no-default-features
```

## Future Enhancements

While the current implementation is complete for the Component Model MVP, future enhancements may include:

1. **Nested Namespaces**: Hierarchical organization of components
2. **Package Management**: Version resolution and dependency management
3. **Shared Memory Support**: When the specification is finalized
4. **Memory64 Support**: 64-bit memory addressing
5. **Custom Page Sizes**: Configurable memory page sizes

## Contributing

When contributing to async features:

1. Maintain `#![forbid(unsafe_code)]` - no unsafe code allowed
2. Ensure all features work in std/alloc/no_std environments
3. Add comprehensive tests for new functionality
4. Update documentation and examples
5. Follow existing patterns for consistency

## References

- [Component Model MVP](https://github.com/WebAssembly/component-model/blob/main/design/mvp/Explainer.md)
- [Canonical ABI](https://github.com/WebAssembly/component-model/blob/main/design/mvp/CanonicalABI.md)
- [Async Model](https://github.com/WebAssembly/component-model/blob/main/design/mvp/Async.md)