# Complete Debug Architecture for WRT

## 🏗️ Current Architecture: Static Debug Information

```
┌─────────────────────────┐
│   WebAssembly Module    │
├─────────────────────────┤
│  .debug_info section    │ ──┐
│  .debug_line section    │   │
│  .debug_str section     │   ├──► wrt-debug (static parsing)
│  .debug_abbrev section  │   │         ├── Line mapping
└─────────────────────────┘   │         ├── Function info
                              │         ├── Parameters
                              │         └── Inline detection
                              │
                              ▼
                    ┌──────────────────┐
                    │  Debug Info API  │
                    ├──────────────────┤
                    │ find_line_info() │ ◄── "What line is PC 0x1234?"
                    │ find_function()  │ ◄── "What function is this?"
                    │ get_parameters() │ ◄── "What are the params?"
                    └──────────────────┘
```

## 🚀 Proposed Architecture: Runtime-Integrated Debugging

```
┌─────────────────────────┐     ┌──────────────────────┐
│   WebAssembly Module    │     │   Runtime (WRT)      │
├─────────────────────────┤     ├──────────────────────┤
│  Code + Debug Sections  │ ──► │  Execution Engine    │
└─────────────────────────┘     │  ├── Interpreter    │
                                │  └── Future: AOT     │
              │                 ├──────────────────────┤
              │                 │  Runtime State       │
              │                 │  ├── PC/SP          │
              │                 │  ├── Locals         │
              │                 │  ├── Stack          │
              │                 │  └── Memory         │
              │                 └──────────────────────┘
              │                           │
              ▼                           ▼
    ┌───────────────────┐      ┌──────────────────────┐
    │  Static Debug     │      │  Runtime Debug API   │
    │  (current)        │      │  (proposed)          │
    ├───────────────────┤      ├──────────────────────┤
    │ • Line mapping    │      │ • Read variables     │
    │ • Function names  │      │ • Set breakpoints    │
    │ • Parameter types │      │ • Step execution     │
    │ • Inline info     │      │ • Inspect memory     │
    └───────────────────┘      │ • Stack traces       │
              │                └──────────────────────┘
              │                           │
              └───────────┬───────────────┘
                          ▼
                ┌─────────────────────┐
                │  Complete Debugger  │
                ├─────────────────────┤
                │ • Source locations  │
                │ • Variable values   │
                │ • Breakpoints       │
                │ • Stepping          │
                │ • Memory inspection │
                └─────────────────────┘
```

## 📦 Module Organization

### Current Modules (Static Only)
```
wrt-debug/
├── src/
│   ├── lib.rs           # Core API
│   ├── cursor.rs        # DWARF parsing
│   ├── line_info.rs     # Line number mapping
│   ├── info.rs          # Function/parameter parsing
│   ├── strings.rs       # String table access
│   ├── file_table.rs    # File path resolution
│   ├── parameter.rs     # Parameter/type info
│   └── stack_trace.rs   # Stack trace formatting
```

### Proposed Runtime Modules
```
wrt-debug/
├── src/
│   ├── runtime_api.rs       # Runtime interface traits
│   ├── runtime_vars.rs      # Variable inspection
│   ├── runtime_memory.rs    # Memory inspection
│   ├── runtime_break.rs     # Breakpoint management
│   ├── runtime_step.rs      # Stepping logic
│   ├── runtime_eval.rs      # Expression evaluation
│   └── runtime_bridge.rs    # WRT integration
```

## 🔌 Integration Points

### 1. **Interpreter Integration** (Natural fit)
```rust
impl WrtInterpreter {
    fn execute_instruction(&mut self, instr: Instruction) -> Result<()> {
        // Hook for debugger
        #[cfg(feature = "runtime-debug")]
        if let Some(debugger) = &mut self.debugger {
            match debugger.on_instruction(self.pc, &self.state) {
                DebugAction::Break => return Ok(()),
                DebugAction::StepLine => self.single_step = true,
                // ... handle other actions
            }
        }
        
        // Normal execution
        match instr {
            // ...
        }
    }
}
```

### 2. **AOT Integration** (More complex)

#### Option A: Debug-Instrumented Code
```rust
// During AOT compilation
fn emit_function(&mut self, func: &Function) {
    #[cfg(feature = "runtime-debug")]
    self.emit_debug_prologue(func.index);
    
    // Emit function body with debug hooks
    for (pc, instr) in func.instructions() {
        #[cfg(feature = "runtime-debug")]
        if self.is_line_boundary(pc) {
            self.emit_debug_checkpoint(pc);
        }
        
        self.emit_instruction(instr);
    }
}
```

#### Option B: Hybrid Execution
```rust
enum ExecutionMode {
    /// Full-speed AOT execution
    Native,
    /// Interpreted with debug support
    Debug,
    /// AOT with minimal debug hooks
    NativeDebug,
}

impl Runtime {
    fn execute(&mut self) -> Result<()> {
        match self.mode {
            ExecutionMode::Native => self.execute_aot(),
            ExecutionMode::Debug => self.execute_interpreted(),
            ExecutionMode::NativeDebug => self.execute_aot_with_debug(),
        }
    }
}
```

## 🎛️ Feature Configuration

```toml
[features]
# Level 1: Static analysis only (current)
default = ["static-debug"]
static-debug = ["line-info", "function-info", "debug-info"]

# Level 2: Runtime inspection (proposed)
runtime-inspection = ["static-debug"]
runtime-vars = ["runtime-inspection"]
runtime-memory = ["runtime-inspection"]

# Level 3: Execution control (proposed)  
runtime-control = ["runtime-inspection"]
runtime-break = ["runtime-control"]
runtime-step = ["runtime-control"]

# Level 4: Advanced features (future)
runtime-eval = ["runtime-control"]
runtime-modify = ["runtime-eval"]

# Presets
minimal = ["line-info"]                    # Just crash locations
development = ["runtime-control"]          # Full debugging
production = ["static-debug"]              # Error reporting only
embedded = ["minimal", "runtime-memory"]   # Memory constrained
```

## 📊 Capability Matrix by Configuration

| Feature | Static | +Runtime Inspection | +Runtime Control | +AOT |
|---------|--------|-------------------|------------------|------|
| **Performance Impact** | 0% | 5-10% | 15-25% | Varies |
| **Memory Overhead** | 8KB | +16KB | +32KB | +Depends |
| **Crash Location** | ✅ | ✅ | ✅ | ✅ |
| **Function Names** | ✅ | ✅ | ✅ | ✅ |
| **Parameter Types** | ✅ | ✅ | ✅ | ✅ |
| **Variable Values** | ❌ | ✅ | ✅ | ⚠️¹ |
| **Memory Inspection** | ❌ | ✅ | ✅ | ✅ |
| **Breakpoints** | ❌ | ❌ | ✅ | ⚠️² |
| **Stepping** | ❌ | ❌ | ✅ | ⚠️³ |
| **Stack Unwinding** | ⚠️ | ✅ | ✅ | ⚠️⁴ |

¹ May need register mapping
² Requires code instrumentation
³ Requires debug build
⁴ Needs frame preservation

## 🎯 Use Case Alignment

### Production Deployment
```toml
features = ["static-debug"]  # Zero runtime overhead
```
- Crash reporting with full context
- Performance profiling
- Error diagnostics

### Development Environment
```toml
features = ["runtime-control"]  # Full debugging
```
- Interactive debugging
- Variable inspection
- Breakpoints and stepping

### Embedded Systems
```toml
features = ["minimal", "runtime-memory"]  # Selective features
```
- Minimal overhead
- Memory inspection for debugging
- No execution control

### Performance-Critical + Debuggable
```toml
# Use hybrid approach
features = ["static-debug", "runtime-inspection"]
```
```rust
// Normal execution: AOT
// On error: Switch to debug mode
runtime.set_mode(ExecutionMode::NativeDebug);
```

## 🔮 Future Considerations

### 1. **Debug Adapter Protocol (DAP)**
- Standardized IDE integration
- Would live above our debug API
- Enable VS Code, IntelliJ debugging

### 2. **Time-Travel Debugging**
- Record execution trace
- Step backwards through execution
- Requires significant memory

### 3. **Remote Debugging**
- Debug WASM running on different machine
- Minimal debug stub on target
- Full debugger on development machine

### 4. **Source-Level Debugging**
- Requires source file access
- Could integrate with source maps
- Show actual source code

## 📝 Summary

The proposed architecture cleanly separates:

1. **Static Debug Info** (what we have):
   - Zero runtime cost
   - Always available
   - Sufficient for production

2. **Runtime Debug API** (proposed):
   - Optional runtime integration
   - Enables full debugging
   - Pay-for-what-you-use

3. **Execution Strategy**:
   - Interpreter: Natural debugging
   - AOT: Multiple strategies available
   - Hybrid: Best of both worlds

This architecture provides a path from current basic debugging to full-featured debugging while maintaining flexibility for different deployment scenarios.