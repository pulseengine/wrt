# Build-Time Memory Validation

WRT includes comprehensive build-time validation for memory budgets to ensure proper memory allocation planning before runtime.

## Overview

The build-time validation system:
- Validates total and per-crate memory budgets
- Checks minimum and maximum allocation limits
- Verifies crate memory system integration
- Generates compile-time constants for memory budgets
- Provides configuration flexibility through files and environment variables

## Configuration

### Default Budgets

The system includes sensible defaults for all WRT crates:

| Crate | Default Budget | Purpose |
|-------|----------------|---------|
| wrt-foundation | 8MB | Core memory management |
| wrt-runtime | 16MB | Execution engine |
| wrt-component | 12MB | Component Model support |
| wrt-decoder | 4MB | WebAssembly decoding |
| wrt-format | 2MB | Format handling |
| wrt-host | 4MB | Host integration |
| wrt-debug | 2MB | Debug tools |
| wrt-platform | 8MB | Platform abstraction |
| wrt-instructions | 4MB | Instruction execution |
| wrt-logging | 1MB | Logging infrastructure |
| wrt-intercept | 1MB | Interception/monitoring |
| wrt-panic | 512KB | Panic handling |
| wrt-sync | 1MB | Synchronization |
| wrt-math | 512KB | Mathematical operations |
| wrt-error | 256KB | Error handling |
| wrt-helper | 256KB | Helper utilities |

**Total Default Budget: ~65.5MB**

### Custom Configuration

#### Configuration File

Create `memory_budget.toml` in the workspace root:

```toml
# Custom memory budgets
"wrt-runtime" = "32MB"
"wrt-component" = "24MB"
"wrt-foundation" = "16MB"
```

Supports unit suffixes: `KB`, `MB`, `GB`

#### Environment Variables

Override individual crate budgets:

```bash
export WRT_BUDGET_WRT_RUNTIME=33554432    # 32MB
export WRT_BUDGET_WRT_FOUNDATION=16777216 # 16MB
```

Variable naming pattern: `WRT_BUDGET_{CRATE_NAME_UPPERCASE}`

## Validation Rules

### Total Budget Limits

- **Minimum**: 16MB (warning if below)
- **Maximum**: 128MB (error if exceeded)

### Per-Crate Limits

#### Minimum Budgets
- **Large crates** (runtime, component): 4MB minimum
- **Foundation crates** (foundation, platform): 2MB minimum  
- **Other crates**: 256KB minimum

#### Maximum Budgets
- **wrt-runtime**: 32MB maximum
- **wrt-component**: 24MB maximum
- **wrt-foundation**: 16MB maximum
- **wrt-platform**: 16MB maximum
- **Other crates**: 8MB maximum

### Crate Integration Checks

The validator checks for:
- Memory system integration in core crates
- `no_std` compatibility where required
- Proper memory initialization hooks

## Generated Constants

The build system generates compile-time constants:

```rust
// Auto-generated in target/*/build/*/out/memory_constants.rs

pub const TOTAL_MEMORY_BUDGET: usize = 68812800;

pub const BUDGET_WRT_FOUNDATION: usize = 8388608;
pub const BUDGET_WRT_RUNTIME: usize = 16777216;
// ... other crate budgets

pub const EMBEDDED_MAX_BUDGET: usize = 4194304;    // 4MB
pub const IOT_MAX_BUDGET: usize = 16777216;        // 16MB
pub const DESKTOP_MAX_BUDGET: usize = 134217728;   // 128MB
pub const SERVER_MAX_BUDGET: usize = 536870912;    // 512MB
```

## Platform-Specific Validation

### Embedded Systems
- Enforces 4MB total budget limit
- Requires strict memory accounting
- Validates no_std compatibility

### IoT Devices
- 16MB budget limit
- Optimized for low-power operation
- Efficient allocation strategies

### Desktop/Server
- Higher budget limits (128MB/512MB)
- Relaxed constraints for development
- Performance-optimized allocations

## Usage in Code

### Accessing Generated Constants

```rust
// Include generated constants
include!(concat!(env!("OUT_DIR"), "/memory_constants.rs"));

fn init_memory_system() {
    let total_budget = TOTAL_MEMORY_BUDGET;
    let foundation_budget = BUDGET_WRT_FOUNDATION;
    
    // Use constants for initialization
}
```

### Runtime Validation

```rust
use wrt_foundation::memory_system_initializer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Validate against build-time constants
    if requested_budget > TOTAL_MEMORY_BUDGET {
        return Err("Budget exceeds build-time limit".into());
    }
    
    memory_system_initializer::initialize_global_memory_system(
        SafetyLevel::Standard,
        MemoryEnforcementLevel::Strict,
        Some(TOTAL_MEMORY_BUDGET),
    )?;
    
    Ok(())
}
```

## Build Integration

### Cargo Integration

The validation runs automatically during `cargo build`:

```bash
cargo build
# Output:
# warning: ✅ Memory budget validation passed
# warning: ✅ Crate memory configuration validation passed
# warning: ✅ Memory constants generated
```

### Custom Build Scripts

For crates with custom build scripts, include validation:

```rust
// In your build.rs
fn main() {
    // Your existing build logic
    
    // Validate memory configuration
    if let Err(e) = validate_memory_budgets() {
        println!("cargo:warning=Memory validation failed: {}", e);
    }
}
```

## Error Messages

### Budget Exceeded
```
error: Budget for wrt-runtime (50331648 bytes) exceeds maximum (33554432 bytes)
```

### Total Budget Too High
```
error: Total budget allocation (268435456 bytes) exceeds maximum (134217728 bytes)
```

### Missing Integration
```
warning: wrt-component may need memory system integration
```

## CI/CD Integration

### GitHub Actions

```yaml
- name: Validate Memory Budgets
  run: cargo build --verbose
  env:
    WRT_BUDGET_WRT_RUNTIME: 16777216  # Override for CI
```

### Custom Validation

```bash
# Strict validation in CI
export WRT_STRICT_VALIDATION=1
cargo build
```

## Troubleshooting

### Common Issues

1. **Budget Too High**: Reduce individual crate budgets
2. **Missing Integration**: Add memory system imports to core crates
3. **No-std Violations**: Ensure proper conditional compilation

### Debug Mode

Enable verbose validation:

```bash
export WRT_DEBUG_VALIDATION=1
cargo build
```

## Future Enhancements

- Dynamic budget adjustment based on target platform
- Integration with cargo-deny for dependency budget analysis
- Memory profiling during build to suggest optimal budgets
- IDE integration for real-time budget feedback