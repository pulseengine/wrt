# WRT-Component Implementation Guide

This guide outlines the complete implementation plan for achieving WebAssembly Component Model MVP compliance in wrt-component with full support for std, no_std+alloc, and pure no_std configurations.

## Implementation Phases

### Phase 1: Fix Build Infrastructure (Week 1)

#### 1.1 Fix Dependency Issues
- [ ] **wrt-intercept**: Make builtins feature-gated behind alloc
  - Move `BuiltinInterceptor`, `BeforeBuiltinResult`, `BuiltinSerialization` behind `#[cfg(feature = "alloc")]`
  - Fix prelude imports to be conditional
  - Replace `format!` with static strings in no_std

- [ ] **wrt-format**: Complete trait implementations
  - Implement `ToBytes` for `Table`, `Memory`, `Element<P>`
  - Fix generic parameter bounds (add Clone, Default, PartialEq, Eq)
  - Fix remaining ~200 compilation errors

- [ ] **wrt-instructions**: Add missing types
  - Define `BranchTarget` type
  - Complete CFI control operations

#### 1.2 Fix wrt-component Build Issues
- [ ] Add proper feature flags in Cargo.toml
- [ ] Conditionally compile all alloc-dependent code
- [ ] Replace all `format!` usage with no_std alternatives
- [ ] Fix all unused import warnings

### Phase 2: Async Support Implementation (Week 2-4)

#### 2.1 Core Async Types (`src/async_types.rs`)
```rust
// Pure Component Model async (NO Rust futures dependency!)
pub enum AsyncValue {
    Stream(StreamHandle),
    Future(FutureHandle),
    ErrorContext(ErrorContextHandle),
}

pub struct Stream<T> {
    readable_end: StreamEnd,
    writable_end: StreamEnd,
    element_type: ValType,
}

pub struct Future<T> {
    readable_end: FutureEnd,
    writable_end: FutureEnd,
    value_type: ValType,
}

pub struct ErrorContext {
    id: u32,
    message: BoundedString<1024>,
    stack_trace: Option<StackTrace>,
}
```

#### 2.2 Task Manager (`src/task_manager.rs`)
```rust
pub struct TaskManager {
    tasks: TaskPool,
    waitables: WaitableSet,
    current_task: Option<TaskId>,
}

pub struct Task {
    id: TaskId,
    state: TaskState,
    borrowed_handles: BoundedVec<ResourceHandle, 32>,
    subtasks: BoundedVec<TaskId, 16>,
    context: TaskContext,
}

pub enum TaskState {
    Starting,
    Started,
    Returned,
    Cancelled,
}
```

#### 2.3 Async Canonical Built-ins (`src/async_canonical.rs`)
```rust
// Component Model canonical built-ins for async:
impl CanonicalAbi {
    pub fn stream_new(&mut self, element_type: &ValType) -> WrtResult<StreamHandle>;
    pub fn stream_read(&mut self, stream: StreamHandle) -> WrtResult<AsyncReadResult>;
    pub fn stream_write(&mut self, stream: StreamHandle, values: &[Value]) -> WrtResult<()>;
    pub fn future_new(&mut self, value_type: &ValType) -> WrtResult<FutureHandle>;
    pub fn future_read(&mut self, future: FutureHandle) -> WrtResult<AsyncReadResult>;
    pub fn task_return(&mut self, values: &[Value]) -> WrtResult<()>;
    pub fn task_wait(&mut self, waitables: &[Waitable]) -> WrtResult<u32>;
    pub fn task_poll(&mut self, waitables: &[Waitable]) -> WrtResult<Option<u32>>;
    pub fn task_yield(&mut self) -> WrtResult<()>;
}
```

#### 2.4 Manual Polling (No async/await)
```rust
// Component Model async.wait - no Rust futures needed!
loop {
    let store = self.async_store.lock().unwrap();
    
    match store.get_status(async_id) {
        Ok(AsyncStatus::Ready) => return store.get_result(async_id),
        Ok(AsyncStatus::Failed) => return store.get_result(async_id),
        Ok(AsyncStatus::Pending) => {
            drop(store);
            std::thread::sleep(Duration::from_millis(1));
            continue;
        }
        Err(e) => return Err(e),
    }
}
```

### Phase 3: Complete Canonical ABI (Week 5-6)

#### 3.1 String Operations
```rust
// No_std compatible string operations
#[cfg(not(feature = "alloc"))]
type WasmString = BoundedString<MAX_STRING_SIZE>;

#[cfg(feature = "alloc")]
type WasmString = String;

impl CanonicalAbi {
    fn lift_string(&self, addr: u32, len: u32, memory: &[u8]) -> Result<WasmString> {
        // Validate UTF-8
        // Copy to bounded/allocated string
        // Handle encoding (UTF-8, UTF-16, Latin1)
    }
    
    fn lower_string(&self, s: &str, addr: u32, memory: &mut [u8]) -> Result<()> {
        // Write string bytes
        // Update length
        // Handle different encodings
    }
}
```

#### 3.2 List Operations
```rust
// Bounded list for no_std
#[cfg(not(feature = "alloc"))]
type WasmList<T> = BoundedVec<T, MAX_LIST_SIZE>;

#[cfg(feature = "alloc")]
type WasmList<T> = Vec<T>;

impl CanonicalAbi {
    fn lift_list(&self, elem_type: &ValType, addr: u32, len: u32) -> Result<Value> {
        // Read list elements
        // Handle alignment
        // Support both bounded and dynamic lists
    }
    
    fn lower_list(&self, list: &[Value], elem_type: &ValType, addr: u32) -> Result<()> {
        // Write list elements
        // Handle alignment
        // Update length
    }
}
```

#### 3.3 Record and Variant Operations
```rust
impl CanonicalAbi {
    fn lift_record(&self, fields: &[(String, ValType)], addr: u32) -> Result<Value> {
        // Calculate field offsets
        // Read each field
        // Handle alignment and padding
    }
    
    fn lift_variant(&self, cases: &[(String, Option<ValType>)], addr: u32) -> Result<Value> {
        // Read discriminant
        // Read payload if present
        // Validate discriminant range
    }
}
```

### Phase 4: WIT Support Implementation (Week 7-9)

#### 4.1 WIT Parser (`src/wit/parser.rs`)
```rust
pub struct WitParser {
    lexer: WitLexer,
    resolver: TypeResolver,
}

pub enum WitDocument {
    Package(WitPackage),
    Interface(WitInterface),
    World(WitWorld),
}

impl WitParser {
    pub fn parse_document(&mut self, source: &str) -> WrtResult<WitDocument>;
    pub fn parse_package(&mut self, source: &str) -> WrtResult<WitPackage>;
    pub fn resolve_imports(&mut self, deps: &[WitPackage]) -> WrtResult<()>;
}
```

#### 4.2 WIT to Component Converter (`src/wit/converter.rs`)
```rust
pub struct WitToComponentConverter {
    type_cache: TypeCache,
    interface_registry: InterfaceRegistry,
}

impl WitToComponentConverter {
    pub fn convert_world(&self, world: &WitWorld) -> WrtResult<Component>;
    pub fn convert_interface(&self, interface: &WitInterface) -> WrtResult<InstanceType>;
    pub fn convert_type(&self, wit_type: &WitType) -> WrtResult<ComponentType>;
}
```

### Phase 5: Advanced Type System (Week 10-11)

#### 5.1 Generative Types (`src/generative_types.rs`)
```rust
// Support for generative resource types:
pub struct GenerativeTypeRegistry {
    // Each component instance gets unique type IDs
    instance_types: HashMap<ComponentInstanceId, HashMap<LocalTypeId, GlobalTypeId>>,
    next_global_id: AtomicU32,
}

pub trait TypeGenerator {
    fn generate_type(&mut self, component_instance: ComponentInstanceId, local_type: &ResourceType) -> GlobalTypeId;
    fn resolve_type(&self, component_instance: ComponentInstanceId, local_id: LocalTypeId) -> Option<GlobalTypeId>;
}
```

#### 5.2 Type Bounds (`src/type_bounds.rs`)
```rust
// Type import bounds:
pub enum TypeBound {
    Eq(Box<ComponentType>),  // Type equality
    Sub(Box<ComponentType>), // Subtype bound
}

pub struct TypeImport {
    name: String,
    bound: TypeBound,
}

impl TypeChecker {
    pub fn check_type_bound(&self, provided: &ComponentType, bound: &TypeBound) -> WrtResult<()>;
    pub fn is_subtype(&self, sub: &ComponentType, super_: &ComponentType) -> bool;
}
```

### Phase 6: Resource Management (Week 12)

#### 6.1 Resource Table Implementation
```rust
// No_std compatible resource table
#[cfg(not(feature = "alloc"))]
type ResourceMap = BoundedMap<u32, ResourceEntry, MAX_RESOURCES>;

#[cfg(feature = "alloc")]
type ResourceMap = HashMap<u32, ResourceEntry>;

struct ResourceTable {
    resources: ResourceMap,
    next_handle: u32,
}

impl ResourceTable {
    fn new_own<T>(&mut self, resource: T) -> Result<u32>;
    fn new_borrow<T>(&mut self, resource: &T) -> Result<u32>;
    fn drop_handle(&mut self, handle: u32) -> Result<()>;
    fn get<T>(&self, handle: u32) -> Result<&T>;
}
```

#### 6.2 Resource Lifecycle
- [ ] Implement drop handlers
- [ ] Add reference counting for borrows
- [ ] Validate resource ownership
- [ ] Handle resource transfer between components

### Phase 7: Component Operations (Week 13)

#### 7.1 Component Instantiation
```rust
impl Component {
    fn instantiate(&self, imports: &ImportMap) -> Result<Instance>;
    fn validate_imports(&self, imports: &ImportMap) -> Result<()>;
    fn extract_exports(&self) -> ExportMap;
}
```

#### 7.2 Component Linking
- [ ] Import resolution
- [ ] Export extraction
- [ ] Type checking at boundaries
- [ ] Value marshaling between components

### Phase 8: Testing and Documentation (Week 14)

#### 8.1 Comprehensive Testing
- [ ] Unit tests for each canonical ABI operation
- [ ] Integration tests with real WASM components
- [ ] Property-based tests for type system
- [ ] Fuzzing for memory safety

#### 8.2 Documentation
- [ ] API documentation for all public types
- [ ] Usage examples
- [ ] Migration guide from other implementations
- [ ] Performance considerations

## Key Design Principles

### Pure Component Model Async (No Rust Futures)
The implementation uses **only** WebAssembly Component Model async primitives:
- Component Model types (stream<T>, future<T>, error-context)
- Manual polling (no async/await)
- Task-based execution
- Canonical built-ins (stream.read/write, future.read/write, task.wait/yield)

### Cross-Environment Support
```rust
// Define reasonable limits for no_std
const MAX_STRING_SIZE: usize = 4096;
const MAX_LIST_SIZE: usize = 1024;
const MAX_RECORD_FIELDS: usize = 64;
const MAX_VARIANT_CASES: usize = 256;
const MAX_RESOURCES: usize = 256;
const MAX_COMPONENTS: usize = 16;
```

### No_std Error Handling
```rust
// No_std compatible error messages
#[cfg(not(feature = "alloc"))]
fn format_error(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::OutOfBounds => "out of bounds access",
        ErrorKind::InvalidUtf8 => "invalid UTF-8 string",
        ErrorKind::TypeMismatch => "type mismatch",
        // ... etc
    }
}
```

## Success Criteria

1. **Compilation**: Zero errors, zero warnings on all configurations
2. **Clippy**: Zero errors, zero warnings with pedantic lints
3. **Tests**: 100% of Component Model MVP features have tests
4. **Documentation**: All public APIs documented
5. **Performance**: No_std mode uses <64KB static memory
6. **Compatibility**: Can run official Component Model test suite
7. **MVP Compliance**: Full WebAssembly Component Model MVP implementation

## Timeline

- Week 1: Fix build infrastructure
- Week 2-4: Async support implementation
- Week 5-6: Complete Canonical ABI
- Week 7-9: WIT support implementation
- Week 10-11: Advanced type system
- Week 12: Resource management
- Week 13: Component operations
- Week 14: Testing and documentation

**Total: 14 weeks to full Component Model MVP compliance**

## Benefits of This Approach

1. **No External Dependencies**: Pure Component Model implementation
2. **Cross-Environment Support**: Works in std, no_std+alloc, and pure no_std
3. **Specification Compliant**: Follows Component Model MVP exactly
4. **Performance**: No overhead from Rust async machinery
5. **Deterministic**: Predictable execution without hidden state machines
6. **Safety**: No unsafe code, all operations memory-safe