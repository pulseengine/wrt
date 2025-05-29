# Component Model Full Implementation Roadmap

## Priority 1: Async Support (Critical for MVP)

### 1.1 Async Types Module (`src/async_types.rs`)
```rust
// Core async types needed:
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

### 1.2 Task Manager Module (`src/task_manager.rs`)
```rust
// Task management system needed:
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

### 1.3 Async Canonical Module (`src/async_canonical.rs`)
```rust
// Canonical built-ins for async:
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

## Priority 2: Type System Enhancements

### 2.1 Generative Types Module (`src/generative_types.rs`)
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

### 2.2 Type Bounds Module (`src/type_bounds.rs`)
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

## Priority 3: WIT Support

### 3.1 WIT Parser Module (`src/wit/parser.rs`)
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

### 3.2 WIT to Component Module (`src/wit/converter.rs`)
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

## Priority 4: Binary Format Completion

### 4.1 Advanced Sections Module (`src/binary/advanced_sections.rs`)
```rust
// Missing section handlers:
pub fn parse_alias_section(reader: &mut BinaryReader) -> WrtResult<Vec<Alias>>;
pub fn parse_component_type_section(reader: &mut BinaryReader) -> WrtResult<Vec<ComponentType>>;
pub fn parse_start_section(reader: &mut BinaryReader) -> WrtResult<StartFunction>;
pub fn encode_alias_section(aliases: &[Alias]) -> WrtResult<Vec<u8>>;
pub fn encode_component_type_section(types: &[ComponentType]) -> WrtResult<Vec<u8>>;
```

### 4.2 Component Composition Module (`src/composition.rs`)
```rust
pub struct ComponentComposer {
    components: Vec<Component>,
    link_definitions: Vec<LinkDefinition>,
}

pub struct LinkDefinition {
    from_component: ComponentId,
    from_export: String,
    to_component: ComponentId,
    to_import: String,
}

impl ComponentComposer {
    pub fn compose(&self) -> WrtResult<Component>;
    pub fn validate_links(&self) -> WrtResult<()>;
    pub fn instantiate_composition(&self, imports: ImportValues) -> WrtResult<ComposedInstance>;
}
```

## Priority 5: Thread Support

### 5.1 Thread Manager Module (`src/thread_manager.rs`)
```rust
pub struct ThreadManager {
    #[cfg(feature = "std")]
    threads: Vec<std::thread::JoinHandle<()>>,
    #[cfg(not(feature = "std"))]
    thread_count: u32,
}

impl ThreadManager {
    pub fn spawn(&mut self, func: ComponentFunction) -> WrtResult<ThreadId>;
    pub fn hw_concurrency(&self) -> u32;
    pub fn current_thread_id(&self) -> ThreadId;
}
```

## Implementation Priority Order

1. **Week 1-2**: Async types and basic task management
2. **Week 3-4**: Async canonical built-ins and lifting/lowering
3. **Week 5-6**: Type system enhancements (generative types, bounds)
4. **Week 7-8**: WIT parser and basic conversion
5. **Week 9-10**: Advanced binary format support
6. **Week 11-12**: Component composition and threading

## Testing Strategy

Each module needs comprehensive tests across all environments:

### Async Tests
```rust
#[test]
fn test_stream_lifecycle() {
    let mut manager = TaskManager::new();
    let stream = manager.create_stream::<u32>().unwrap();
    manager.stream_write(stream, &[Value::U32(42)]).unwrap();
    let result = manager.stream_read(stream).unwrap();
    assert_eq!(result, AsyncReadResult::Value(Value::U32(42)));
}
```

### Cross-Environment Tests
```rust
#[cfg(feature = "std")]
#[test]
fn test_async_with_std() {
    // Test with std::future integration
}

#[cfg(all(not(feature = "std"), feature = "alloc"))]
#[test]
fn test_async_no_std_alloc() {
    // Test with custom async runtime
}

#[cfg(not(any(feature = "std", feature = "alloc")))]
#[test]
fn test_async_pure_no_std() {
    // Test with poll-based async
}
```

## Resource Requirements

### Development Resources
- 2-3 developers for 3 months
- Continuous integration for all environments
- Fuzzing infrastructure for parser testing
- Performance benchmarking setup

### Technical Requirements
- Rust 1.70+ for async trait support
- Optional: formal verification tools
- Test coverage > 90%
- Documentation for all public APIs

## Success Criteria

1. **Full MVP Compliance**: All features in the Component Model MVP spec implemented
2. **Cross-Environment Support**: Works in std, no_std+alloc, and pure no_std
3. **Performance**: Async operations within 10% of native performance
4. **Safety**: No unsafe code, all operations memory-safe
5. **Interoperability**: Can load and execute components from wasmtime/other runtimes
6. **Documentation**: Complete API docs and usage examples

This roadmap provides a clear path to achieving full Component Model MVP compliance while maintaining our cross-environment support and safety guarantees.