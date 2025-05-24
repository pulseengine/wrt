# Feature Flags

The `wrt-debug` crate uses feature flags to allow fine-grained control over which debug capabilities are included in your build. This is especially important for embedded and no_std environments where code size and memory usage are critical.

## Available Features

### Core Features

- **`line-info`** (default): Line number information support
  - Enables `.debug_line` section parsing
  - Maps instruction addresses to source file locations
  - Minimal overhead, suitable for most debugging scenarios

- **`abbrev`**: DWARF abbreviation table support
  - Required for parsing `.debug_abbrev` sections
  - Enables more efficient debug info parsing
  - Required by `debug-info` feature

- **`debug-info`**: Full debug information parsing
  - Enables `.debug_info` section parsing
  - Provides access to compilation units and DIEs
  - Automatically enables `abbrev` feature
  - Larger memory footprint

- **`function-info`**: Function discovery and mapping
  - Enables function boundary detection
  - Maps addresses to function ranges
  - Provides function metadata (source location, etc.)
  - Automatically enables `debug-info` feature

### Convenience Features

- **`full-debug`**: Enables all debug features
  - Equivalent to enabling `line-info`, `debug-info`, and `function-info`
  - Maximum debugging capability
  - Largest code and memory footprint

## Feature Dependencies

```
full-debug
├── line-info
├── debug-info
│   └── abbrev
└── function-info
    └── debug-info
        └── abbrev
```

## Usage Examples

### Minimal Configuration (Line Info Only)

```toml
[dependencies]
wrt-debug = { version = "0.1", default-features = false, features = ["line-info"] }
```

```rust
use wrt_debug::prelude::*;

let mut debug_info = DwarfDebugInfo::new(module_bytes);
debug_info.add_section(".debug_line", offset, size);

// This works with line-info feature
if let Ok(Some(line)) = debug_info.find_line_info(pc) {
    println!("Line: {}", line.line);
}
```

### No Debug Support

```toml
[dependencies]
wrt-debug = { version = "0.1", default-features = false }
```

```rust
use wrt_debug::prelude::*;

let mut debug_info = DwarfDebugInfo::new(module_bytes);
// Only section registration and basic queries work
debug_info.add_section(".debug_line", offset, size);
let has_debug = debug_info.has_debug_info(); // Always works
```

### Full Debug Support

```toml
[dependencies]
wrt-debug = { version = "0.1", features = ["full-debug"] }
```

```rust
use wrt_debug::prelude::*;

let mut debug_info = DwarfDebugInfo::new(module_bytes);
debug_info.add_section(".debug_line", offset, size);
debug_info.add_section(".debug_info", info_offset, info_size);
debug_info.add_section(".debug_abbrev", abbrev_offset, abbrev_size);

debug_info.init_info_parser()?;

// All features available
if let Ok(Some(line)) = debug_info.find_line_info(pc) { /* ... */ }
if let Some(func) = debug_info.find_function_info(pc) { /* ... */ }
```

## Integration with WRT Runtime

The `wrt-runtime` crate provides optional debug support:

```toml
[dependencies]
wrt-runtime = { version = "0.2", features = ["debug"] }      # Basic debug
wrt-runtime = { version = "0.2", features = ["debug-full"] } # Full debug
```

```rust
use wrt_runtime::ModuleInstance;

let mut instance = ModuleInstance::new(module, 0);

// Available with "debug" feature
instance.init_debug_info(module_bytes)?;
let line_info = instance.get_line_info(pc)?;

// Available with "debug-full" feature
let func_info = instance.get_function_info(pc);
```

## Memory and Code Size Impact

| Feature Configuration | Approximate Code Size | Memory Usage |
|----------------------|---------------------|-------------|
| No features          | ~2KB                | ~64 bytes   |
| `line-info`          | ~8KB                | ~1KB        |
| `debug-info`         | ~15KB               | ~4KB        |
| `full-debug`         | ~20KB               | ~6KB        |

*Note: Sizes are approximate and depend on the complexity of debug information.*

## Conditional Compilation

When writing code that may or may not have debug features, use conditional compilation:

```rust
#[cfg(feature = "line-info")]
fn debug_with_line_info(debug_info: &mut DwarfDebugInfo, pc: u32) {
    if let Ok(Some(line)) = debug_info.find_line_info(pc) {
        println!("At line {}", line.line);
    }
}

#[cfg(feature = "function-info")]
fn debug_with_function_info(debug_info: &DwarfDebugInfo, pc: u32) {
    if let Some(func) = debug_info.find_function_info(pc) {
        println!("In function {:x}-{:x}", func.low_pc, func.high_pc);
    }
}

// Always available
fn basic_debug(debug_info: &DwarfDebugInfo) {
    let has_debug = debug_info.has_debug_info();
    println!("Debug available: {}", has_debug);
}
```

## Recommendations

- **Embedded/Constrained**: Use no features or just `line-info`
- **Development**: Use `full-debug` for maximum debugging capability
- **Production**: Use `line-info` for basic error reporting
- **Library**: Allow users to choose features via optional dependencies