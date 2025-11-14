# Component Model Import Linker - Implementation Plan

## Problem Statement

WebAssembly components declare imports as typed instance imports. For WASI Preview 2 support, we need to:
1. Match component imports (e.g., "wasi:cli/stdout@0.2.0") to providers
2. Validate type compatibility between imports and providers
3. Create runtime instances that satisfy the component's import requirements
4. Enable the component to call these imported functions

## Current State

**What Works:**
- ✅ Component decoder correctly parses all 13 WASI imports
- ✅ Imports are Type imports referencing instance type definitions
- ✅ Component exports are now callable (previous fix)
- ✅ WASI host functions registered (3 stub functions)

**What's Missing:**
- ❌ No import resolution/linking mechanism
- ❌ No WASI component instance providers
- ❌ No type validation for imports
- ❌ No resource handle management

## Architecture Design

### Phase 1: Minimal Viable Linker (MVP)

**Goal:** Link WASI imports to stub instances so component instantiation succeeds

**Components:**
1. `ComponentLinker` - Main linking orchestrator
2. `WasiInstanceProvider` - Creates stub WASI instances
3. `LinkedImport` - Represents a resolved import
4. Integration into `ComponentInstance::from_parsed`

**File Structure:**
```
wrt-component/src/
├── linker/
│   ├── mod.rs              # Public API
│   ├── component_linker.rs # Main linker logic
│   └── wasi_provider.rs    # WASI instance creation
```

### Phase 2: Type Validation (Future)

- Validate imported instance types match provided instances
- Check function signatures
- Verify resource type compatibility

### Phase 3: Full WASI Implementation (Future)

- Actual WASI function implementations
- Resource handle tables
- Canonical ABI lift/lower operations
- Memory sharing between component and host

## Implementation Plan - Phase 1

### Step 1: Create Linker Module Structure

**File:** `wrt-component/src/linker/mod.rs`

```rust
//! Component import linker
//!
//! Resolves component imports to providers and creates runtime instances

pub mod component_linker;
pub mod wasi_provider;

pub use component_linker::ComponentLinker;
pub use wasi_provider::WasiInstanceProvider;
```

### Step 2: Define Core Types

**File:** `wrt-component/src/linker/component_linker.rs`

```rust
use wrt_error::Result;
use wrt_format::component::Import;

/// Represents a successfully linked import
pub struct LinkedImport {
    /// Import name
    pub name: String,
    /// Instance ID that satisfies this import
    pub instance_id: u32,
}

/// Component linker that resolves imports to providers
pub struct ComponentLinker {
    /// WASI instance provider
    wasi_provider: WasiInstanceProvider,
}

impl ComponentLinker {
    pub fn new() -> Result<Self> {
        Ok(Self {
            wasi_provider: WasiInstanceProvider::new()?,
        })
    }

    /// Link component imports to providers
    pub fn link_imports(&mut self, imports: &[Import]) -> Result<Vec<LinkedImport>> {
        let mut linked = Vec::with_capacity(imports.len());

        for import in imports {
            let name = format!("{}:{}", import.name.namespace, import.name.name);

            // Check if this is a WASI import
            if name.starts_with("wasi:") {
                // Create stub WASI instance
                let instance_id = self.wasi_provider.create_instance(&name)?;
                linked.push(LinkedImport { name, instance_id });
            } else {
                // Unknown import - fail for now
                return Err(Error::validation_error(&format!("Unknown import: {}", name)));
            }
        }

        Ok(linked)
    }
}
```

### Step 3: WASI Instance Provider (Stubs)

**File:** `wrt-component/src/linker/wasi_provider.rs`

```rust
use wrt_error::Result;

/// Provides WASI component instances
pub struct WasiInstanceProvider {
    next_instance_id: u32,
}

impl WasiInstanceProvider {
    pub fn new() -> Result<Self> {
        Ok(Self { next_instance_id: 100 })
    }

    /// Create a stub WASI instance for the given interface
    pub fn create_instance(&mut self, interface_name: &str) -> Result<u32> {
        // For now, just allocate an ID
        // Future: actually create instance with proper exports
        let id = self.next_instance_id;
        self.next_instance_id += 1;

        #[cfg(feature = "std")]
        println!("[LINKER] Created stub instance {} for {}", id, interface_name);

        Ok(id)
    }
}
```

### Step 4: Integration

**In:** `wrt-component/src/components/component_instantiation.rs`

Add to `from_parsed`:
```rust
// After parsing imports, before building instance:
use crate::linker::ComponentLinker;

let mut linker = ComponentLinker::new()?;
let linked_imports = linker.link_imports(&parsed.imports)?;

// Store linked imports in instance
instance.linked_imports = linked_imports;
```

### Step 5: Test

Run component instantiation and verify:
- All 13 WASI imports are linked
- Instance IDs are assigned
- No "unknown import" errors

## Success Criteria

- [x] Component decoder parses all imports correctly
- [ ] Component linker successfully matches all 13 WASI imports
- [ ] Component instantiation completes without import errors
- [ ] Linked imports are accessible from ComponentInstance
- [ ] Ready for Phase 2 (actual WASI implementation)

## Notes

- This is a **stub implementation** - WASI functions don't actually work yet
- Focus is on getting the linking infrastructure in place
- Type validation and actual WASI logic come in later phases
- This unblocks component model development while WASI implementation proceeds in parallel
