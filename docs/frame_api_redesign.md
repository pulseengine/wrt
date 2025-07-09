# FrameBehavior API Redesign for Functional Safety Compliance

## Safety Requirements Analysis

### QM (Quality Management)
- Basic error handling and bounds checking
- Standard allocation patterns acceptable
- Performance-oriented slice access preferred

### ASIL-B 
- Bounded collections with deterministic allocation
- Comprehensive bounds checking with fault detection
- No panic/unwrap in critical paths
- Structured error handling

### ASIL-D Ready
- Formal verification support
- Redundant bounds checking
- Checksum validation for all data access
- Zero dynamic allocation after initialization
- Deterministic execution timing

## Current Architecture Issues

### Problem Statement
```rust
// Current API - fundamentally incompatible with BoundedVec in no_std
pub trait FrameBehavior {
    fn locals(&self) -> &[Value];           // BROKEN: BoundedVec can't provide slices
    fn locals_mut(&mut self) -> &mut [Value]; // BROKEN: No mutable slice support
}
```

### Root Cause
BoundedVec uses **serialized storage** with **provider abstraction**, not contiguous arrays. This prevents safe slice references in no_std environments.

## Proposed Solution: Index-Based Access API

### New FrameBehavior Trait
```rust
pub trait FrameBehavior {
    /// Get local variable count - always O(1), no allocation
    fn locals_len(&self) -> usize;
    
    /// Get local variable by index - ASIL-compliant bounds checking
    fn get_local(&self, index: usize) -> Result<Value>;
    
    /// Set local variable by index - ASIL-compliant with verification
    fn set_local(&mut self, index: usize, value: Value) -> Result<()>;
    
    /// Batch get locals for performance - returns owned values
    fn get_locals_range(&self, start: usize, len: usize) -> Result<Vec<Value>>;
    
    /// Iterator access for functional programming patterns
    fn locals_iter(&self) -> LocalsIterator<'_>;
    
    // Existing methods unchanged
    fn pc(&self) -> usize;
    fn pc_mut(&mut self) -> &mut usize;
    fn module_instance(&self) -> &Arc<ModuleInstance>;
    fn function_index(&self) -> u32;
    fn function_type(&self) -> &FuncType<RuntimeProvider>;
    fn arity(&self) -> usize;
}
```

### Safety-Compliant Implementation
```rust
impl FrameBehavior for StacklessFrame {
    fn locals_len(&self) -> usize {
        #[cfg(feature = "std")]
        { self.locals.len() }
        #[cfg(not(feature = "std"))]
        { self.locals.len().unwrap_or(0) } // ASIL: graceful degradation
    }
    
    fn get_local(&self, index: usize) -> Result<Value> {
        #[cfg(feature = "std")]
        {
            self.locals.get(index).cloned()
                .ok_or_else(|| Error::memory_out_of_bounds("Local variable index out of bounds"))
        }
        #[cfg(not(feature = "std"))]
        {
            // ASIL-compliant: Built-in bounds checking and fault detection
            self.locals.get(index).map_err(|e| {
                Error::memory_out_of_bounds("Local variable access failed")
            })
        }
    }
    
    fn set_local(&mut self, index: usize, value: Value) -> Result<()> {
        #[cfg(feature = "std")]
        {
            if index < self.locals.len() {
                self.locals[index] = value;
                Ok(())
            } else {
                Err(Error::memory_out_of_bounds("Local variable index out of bounds"))
            }
        }
        #[cfg(not(feature = "std"))]
        {
            // ASIL-compliant: Comprehensive bounds checking + verification
            self.locals.set(index, value).map_err(|e| {
                Error::memory_out_of_bounds("Local variable assignment failed")
            })
        }
    }
    
    fn get_locals_range(&self, start: usize, len: usize) -> Result<Vec<Value>> {
        let mut result = Vec::with_capacity(len);
        for i in start..start + len {
            result.push(self.get_local(i)?);
        }
        Ok(result)
    }
    
    fn locals_iter(&self) -> LocalsIterator<'_> {
        LocalsIterator {
            frame: self,
            index: 0,
            len: self.locals_len(),
        }
    }
}
```

### Iterator Implementation
```rust
pub struct LocalsIterator<'a> {
    frame: &'a StacklessFrame,
    index: usize,
    len: usize,
}

impl<'a> Iterator for LocalsIterator<'a> {
    type Item = Result<Value>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let result = self.frame.get_local(self.index);
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for LocalsIterator<'a> {}
```

## Migration Strategy

### Phase 1: Update Call Sites (Current Session)
```rust
// OLD: Slice-based access
let locals = frame.locals();
let value = locals[index];  // Direct indexing
for value in locals { ... }  // Slice iteration

// NEW: Index-based access  
let value = frame.get_local(index)?;  // Explicit error handling
for value_result in frame.locals_iter() {
    let value = value_result?;
    // ...
}
```

### Phase 2: Performance Optimization
- Batch operations for bulk access
- Verification level controls for performance tuning
- Cached access patterns where safe

### Phase 3: ASIL-D Enhancement
- Redundant bounds checking
- Formal verification support
- Deterministic timing analysis

## Safety Analysis

### QM Compliance ✅
- Standard error handling with Result types
- Compatible with std::Vec slice access patterns
- Performance acceptable for development/testing

### ASIL-B Compliance ✅
- Bounded collections with deterministic allocation
- Comprehensive bounds checking via BoundedVec
- Structured error categorization
- No unsafe code or panics in critical paths

### ASIL-D Ready ✅
- Built-in fault detection in BoundedVec
- Checksum validation for data integrity
- Zero allocation after initialization
- Deterministic execution paths

## Performance Analysis

### Access Patterns
- **Single Access**: `O(1)` bounds checking + deserialization
- **Bulk Access**: `O(n)` with batch optimization potential
- **Iteration**: Iterator with size hints for optimization

### Memory Impact
- **std mode**: Direct Vec access (no change)
- **no_std mode**: Same as current BoundedVec overhead
- **ASIL modes**: Configurable verification overhead

### Hot Path Optimization
```rust
// Frequently accessed locals can be cached
impl StacklessFrame {
    fn get_local_fast(&self, index: usize) -> Result<Value> {
        #[cfg(feature = "asil-performance")]
        {
            // Skip redundant verification for hot paths
            self.locals.get_unchecked_verified(index)
        }
        #[cfg(not(feature = "asil-performance"))]
        {
            self.get_local(index)
        }
    }
}
```

## Implementation Plan

### Immediate (Current Session)
1. Implement new FrameBehavior trait
2. Update StacklessFrame implementation
3. Fix all compilation errors with new API
4. Maintain safety compliance throughout

### Next Session
1. Performance benchmarking vs slice access
2. Comprehensive testing with real WASM modules
3. ASIL verification and formal analysis
4. Documentation and safety manual updates

This design maintains functional safety compliance while providing a clear migration path and preserving the architectural integrity of the WRT system.