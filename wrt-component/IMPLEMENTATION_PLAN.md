# WRT-Component Implementation Plan

## Overview
This plan outlines the steps needed to complete the WebAssembly Component Model MVP implementation in wrt-component with full support for std, no_std+alloc, and pure no_std configurations.

## Phase 1: Fix Build Infrastructure (Week 1)

### 1.1 Fix Dependency Issues
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

### 1.2 Fix wrt-component Build Issues
- [ ] Add proper feature flags in Cargo.toml
- [ ] Conditionally compile all alloc-dependent code
- [ ] Replace all `format!` usage with no_std alternatives
- [ ] Fix all unused import warnings

## Phase 2: Complete Canonical ABI (Week 2-3)

### 2.1 String Operations
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

### 2.2 List Operations
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

### 2.3 Record Operations
```rust
impl CanonicalAbi {
    fn lift_record(&self, fields: &[(String, ValType)], addr: u32) -> Result<Value> {
        // Calculate field offsets
        // Read each field
        // Handle alignment and padding
    }
    
    fn lower_record(&self, fields: &[(String, Value)], addr: u32) -> Result<()> {
        // Calculate layout
        // Write each field
        // Add padding as needed
    }
}
```

### 2.4 Variant Operations
```rust
impl CanonicalAbi {
    fn lift_variant(&self, cases: &[(String, Option<ValType>)], addr: u32) -> Result<Value> {
        // Read discriminant
        // Read payload if present
        // Validate discriminant range
    }
    
    fn lower_variant(&self, case: &str, payload: Option<&Value>, addr: u32) -> Result<()> {
        // Find case index
        // Write discriminant
        // Write payload if present
    }
}
```

## Phase 3: Resource Management (Week 3-4)

### 3.1 Resource Table Implementation
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

### 3.2 Resource Lifecycle
- [ ] Implement drop handlers
- [ ] Add reference counting for borrows
- [ ] Validate resource ownership
- [ ] Handle resource transfer between components

## Phase 4: Type System Completion (Week 4)

### 4.1 Type Validation
```rust
impl ValType {
    fn validate(&self) -> Result<()>;
    fn is_subtype_of(&self, other: &ValType) -> bool;
    fn size_and_alignment(&self) -> (usize, usize);
}
```

### 4.2 Type Equality and Subtyping
- [ ] Implement structural equality
- [ ] Add subtyping rules
- [ ] Handle recursive types via ValTypeRef

## Phase 5: Component Operations (Week 5)

### 5.1 Component Instantiation
```rust
impl Component {
    fn instantiate(&self, imports: &ImportMap) -> Result<Instance>;
    fn validate_imports(&self, imports: &ImportMap) -> Result<()>;
    fn extract_exports(&self) -> ExportMap;
}
```

### 5.2 Component Linking
- [ ] Import resolution
- [ ] Export extraction
- [ ] Type checking at boundaries
- [ ] Value marshaling between components

## Phase 6: Testing and Documentation (Week 6)

### 6.1 Comprehensive Testing
- [ ] Unit tests for each canonical ABI operation
- [ ] Integration tests with real WASM components
- [ ] Property-based tests for type system
- [ ] Fuzzing for memory safety

### 6.2 Documentation
- [ ] API documentation for all public types
- [ ] Usage examples
- [ ] Migration guide from other implementations
- [ ] Performance considerations

## No_std Specific Considerations

### Memory Limits
```rust
// Define reasonable limits for no_std
const MAX_STRING_SIZE: usize = 4096;
const MAX_LIST_SIZE: usize = 1024;
const MAX_RECORD_FIELDS: usize = 64;
const MAX_VARIANT_CASES: usize = 256;
const MAX_RESOURCES: usize = 256;
const MAX_COMPONENTS: usize = 16;
```

### Error Handling
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

### Testing Strategy
1. Create shared test suite that runs on all configurations
2. Use conditional compilation for alloc-specific tests
3. Ensure feature parity across all modes
4. Benchmark memory usage in no_std mode

## Success Criteria

1. **Compilation**: Zero errors, zero warnings on all configurations
2. **Clippy**: Zero errors, zero warnings with pedantic lints
3. **Tests**: 100% of Component Model MVP features have tests
4. **Documentation**: All public APIs documented
5. **Performance**: No_std mode uses <64KB static memory
6. **Compatibility**: Can run official Component Model test suite

## Timeline

- Week 1: Fix build infrastructure
- Week 2-3: Complete Canonical ABI
- Week 3-4: Resource management
- Week 4: Type system
- Week 5: Component operations
- Week 6: Testing and documentation

Total: 6 weeks to full Component Model MVP compliance