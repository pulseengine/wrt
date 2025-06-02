# Missing Component Model Features

This document tracks the Component Model features that still need to be implemented in WRT.

## Status Legend
- ✅ Implemented
- 🚧 Partially implemented
- ❌ Not implemented
- 🔜 Planned for next phase

## Core Features

### Resource Management
- ✅ `resource.new` - Create new resource
- ✅ `resource.drop` - Drop resource
- ✅ `resource.rep` - Get resource representation
- ✅ Own/Borrow handle types
- ✅ Resource lifecycle tracking
- ✅ Drop handlers

### Async Operations
- 🚧 `stream.new` - Create new stream (partial)
- 🚧 `stream.read` - Read from stream (partial)
- 🚧 `stream.write` - Write to stream (partial)
- ✅ `stream.close-readable` - Close read end
- ✅ `stream.close-writable` - Close write end
- 🚧 `future.new` - Create future (partial)
- 🚧 `future.get` - Get future value (partial)
- ✅ `future.cancel` - Cancel future

### Context Management
- ✅ `context.get` - Get current async context
- ✅ `context.set` - Set async context
- ✅ Context switching for async operations

### Task Management
- ✅ `task.return` - Return from async task
- ✅ `task.cancel` - Cancel task (complete with built-ins)
- ✅ `task.status` - Get task status
- ✅ `task.start` - Start new task
- ✅ `task.wait` - Wait for task completion

### Waitable Operations
- ✅ `waitable-set.new` - Create waitable set (complete with built-ins)
- ✅ `waitable-set.wait` - Wait on set
- ✅ `waitable-set.add` - Add to set
- ✅ `waitable-set.remove` - Remove from set

### Error Context
- ✅ `error-context.new` - Create error context (complete with built-ins)
- ✅ `error-context.debug-message` - Get debug message
- ✅ `error-context.drop` - Drop error context

### Threading Built-ins
- ✅ `thread.available_parallelism` - Get parallelism info
- 🚧 `thread.spawn` - Basic thread spawn
- ✅ `thread.spawn_ref` - Spawn with function reference
- ✅ `thread.spawn_indirect` - Spawn with indirect call
- ✅ `thread.join` - Join thread
- ✅ Thread-local storage

### Type System Features
- ✅ Fixed-length lists
- ❌ Nested namespaces
- ❌ Package management
- 🚧 Generative types (partial)

### Canonical Operations
- ✅ `canon lift` - Basic lifting
- ✅ `canon lower` - Basic lowering
- 🚧 `canon lift` with `async` (partial)
- ❌ `canon callback` - Async callbacks
- ✅ `canon resource.new`
- ✅ `canon resource.drop`
- ✅ `canon resource.rep`

### Memory Features
- ❌ Shared memory support
- ❌ Memory64 support
- ❌ Custom page sizes
- ✅ Memory isolation

## Implementation Priority

### Phase 1: Complete Async Foundation (High Priority) ✅ COMPLETED
1. ✅ Implement context management built-ins
2. ✅ Complete task management built-ins
3. ✅ Implement waitable-set operations
4. ✅ Complete error-context built-ins

### Phase 2: Advanced Threading (Medium Priority) ✅ COMPLETED
1. ✅ Implement thread.spawn_ref
2. ✅ Implement thread.spawn_indirect
3. ✅ Add thread join operations
4. ✅ Add thread-local storage

### Phase 3: Type System Enhancements (Medium Priority) ✅ PARTIALLY COMPLETED
1. ✅ Add fixed-length list support
2. ❌ Implement nested namespaces
3. ❌ Add package management

### Phase 4: Future Features (Low Priority)
1. Shared memory support (when spec is ready)
2. Memory64 support
3. Custom page sizes

## Testing Requirements

Each feature implementation should include:
1. Unit tests for the built-in functions
2. Integration tests with the canonical ABI
3. Conformance tests from the official test suite
4. Performance benchmarks
5. Documentation and examples

## Specification References

- [Component Model MVP](https://github.com/WebAssembly/component-model/blob/main/design/mvp/Explainer.md)
- [Canonical ABI](https://github.com/WebAssembly/component-model/blob/main/design/mvp/CanonicalABI.md)
- [Binary Format](https://github.com/WebAssembly/component-model/blob/main/design/mvp/Binary.md)
- [WIT Format](https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md)