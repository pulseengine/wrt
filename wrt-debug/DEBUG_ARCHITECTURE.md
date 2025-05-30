# WRT Debug Architecture Complete Guide

This document outlines the complete debugging architecture for WRT, covering both current static capabilities and planned runtime integration.

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

### Current Module Organization
```
wrt-debug/
├── src/
│   ├── lib.rs           # Core API
│   ├── cursor.rs        # DWARF parsing utilities
│   ├── line_info.rs     # Line number mapping
│   ├── info.rs          # Function/parameter parsing
│   ├── strings.rs       # String table access
│   ├── file_table.rs    # File path resolution
│   ├── parameter.rs     # Parameter/type info
│   ├── stack_trace.rs   # Stack trace formatting
│   └── types.rs         # Core data structures
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

## 📦 Proposed Runtime Modules

### New Runtime Debug Modules
```
wrt-debug/
├── src/
│   ├── runtime_api.rs       # Runtime interface traits
│   ├── runtime_vars.rs      # Variable inspection
│   ├── runtime_memory.rs    # Memory inspection
│   ├── runtime_break.rs     # Breakpoint management
│   ├── runtime_step.rs      # Stepping logic
│   └── runtime_bridge.rs    # WRT integration
```

### 1. **Runtime Variable Inspector** (`runtime_vars.rs`)
```rust
/// Runtime variable inspection support
pub trait VariableInspector {
    /// Read local variable by DWARF location
    fn read_local(&self, location: DwarfLocation, frame: &StackFrame) -> Result<Value>;
    
    /// Read global variable
    fn read_global(&self, address: u32) -> Result<Value>;
    
    /// Evaluate DWARF expression for variable location
    fn eval_location(&self, expr: &[u8], frame: &StackFrame) -> Result<u32>;
}

pub struct RuntimeValue {
    pub raw_bytes: [u8; 8],
    pub type_info: BasicType,
    pub location: MemoryLocation,
}
```

### 2. **Breakpoint Manager** (`runtime_break.rs`)
```rust
pub trait BreakpointManager {
    /// Set breakpoint at address
    fn set_breakpoint(&mut self, addr: u32, condition: Option<Condition>) -> BreakpointId;
    
    /// Handle breakpoint hit
    fn on_breakpoint(&mut self, id: BreakpointId, state: &RuntimeState) -> DebugAction;
}

pub enum DebugAction {
    Continue,
    StepOver,
    StepInto,
    StepOut,
    Evaluate(String),
}
```

### 3. **Memory Inspector** (`runtime_memory.rs`)
```rust
pub trait MemoryInspector {
    /// Read memory range safely
    fn read_memory(&self, addr: u32, len: usize) -> Result<&[u8]>;
    
    /// Get heap allocation info
    fn heap_allocations(&self) -> Vec<AllocationInfo>;
    
    /// Stack frame analysis
    fn analyze_stack(&self, sp: u32) -> StackLayout;
}
```

### 4. **Runtime State Bridge** (`runtime_bridge.rs`)
```rust
/// Bridge between WRT runtime and debug system
pub trait RuntimeDebugBridge {
    /// Get current execution state
    fn get_state(&self) -> RuntimeState;
    
    /// Read register value
    fn read_register(&self, reg: Register) -> u32;
    
    /// Get current stack pointer
    fn get_sp(&self) -> u32;
    
    /// Get current frame pointer
    fn get_fp(&self) -> u32;
}
```

## 🔌 Runtime Integration Strategies

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
                DebugAction::Continue => {},
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

**Advantages**:
- Easy to instrument
- Natural breakpoint support
- Easy state inspection
- Single-stepping is trivial

**Current Limitations**:
- Performance overhead always present
- Limited optimization opportunities
- But debugging is "free"

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

#### Option C: Deoptimization for Debugging
```rust
// Start with optimized AOT
let native_code = compile_optimized(wasm);

// On breakpoint/debug request:
// 1. Capture current state
// 2. Switch to interpreter
// 3. Continue execution with full debug
fn deoptimize_for_debug(pc: u32, state: RuntimeState) {
    let interpreter = restore_to_interpreter(state);
    interpreter.continue_with_debug(pc);
}
```

## 🎛️ Feature Configuration Architecture

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

| Feature | Static | +Runtime Inspection | +Runtime Control | AOT Support |
|---------|--------|-------------------|------------------|-------------|
| **Performance Impact** | 0% | 5-10% | 15-25% | Varies |
| **Memory Overhead** | 8KB | +16KB | +32KB | Depends |
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
³ May need instruction-level boundaries  
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

## 📈 Performance Analysis

### Interpreter + Runtime Debug
- Base interpreter: ~10-50x slower than native
- Debug overhead: +10-20% on interpreter  
- Total: ~11-60x slower than native
- **Verdict**: Debug overhead negligible

### AOT + Runtime Debug
- Base AOT: ~1-2x slower than native
- Debug instrumentation: +10-30% on AOT
- Total: ~1.1-2.6x slower than native
- **Verdict**: Significant but acceptable

### Hybrid Approach
- Normal execution: Full AOT speed
- Debug execution: Falls back to interpreter
- **Verdict**: Best of both worlds

## 🎨 Example Integrations

### Complete Runtime Debugger
```rust
pub struct CompleteDebugger {
    static_info: DwarfDebugInfo<'static>,
    var_inspector: VariableInspector<'static>,
    mem_inspector: MemoryInspector<'static>,
    bp_manager: BreakpointManager,
    stepper: SteppingController,
}

impl CompleteDebugger {
    pub fn on_break(&mut self, pc: u32, state: &dyn RuntimeState) {
        // Show location
        if let Some(line_info) = self.static_info.find_line_info(pc).ok().flatten() {
            println!("Stopped at {}:{}", line_info.file_index, line_info.line);
        }
        
        // Show function context
        if let Some(func) = self.static_info.find_function_info(pc) {
            println!("In function: {}", func.name.unwrap_or("<unknown>"));
        }
        
        // Show variables
        let vars = self.var_inspector.get_live_variables(pc, state);
        for var in vars.iter() {
            if let Some(value) = &var.value {
                println!("  {}: {:?}", var.name.unwrap_or("?"), value);
            }
        }
    }
}
```

### Feature-Gated Runtime Integration
```rust
// Feature-gated runtime debug support
#[cfg(feature = "runtime-debug")]
pub struct DebugCapableRuntime<R: Runtime> {
    runtime: R,
    debugger: Option<Box<dyn Debugger>>,
    mode: ExecutionMode,
}

impl<R: Runtime> DebugCapableRuntime<R> {
    /// Execute with optional debugging
    pub fn execute(&mut self) -> Result<()> {
        match self.mode {
            ExecutionMode::Normal => self.runtime.execute(),
            ExecutionMode::Debug => self.execute_with_debug(),
        }
    }
}
```

## 🔮 Future Architecture Considerations

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

## 📝 Implementation Roadmap

### Phase 1: Runtime Interface Design ✅
- Define core runtime debug traits
- Establish feature flag structure
- Design integration points

### Phase 2: Interpreter Integration 🔄
```rust
impl DebugRuntime for WrtInterpreter {
    // Natural integration with interpreter
    // Full access to all state
}
```

### Phase 3: Basic Runtime Features 🎯
- Variable inspection
- Memory inspection  
- Basic breakpoints

### Phase 4: Advanced Runtime Features 🔄
- Stepping control
- Conditional breakpoints
- Expression evaluation

### Phase 5: AOT Integration Options 🔮
- Debug-instrumented compilation
- Hybrid execution strategies
- JIT-style deoptimization

## 🏆 Architecture Summary

### Current Strengths
1. **Static Analysis**: Complete and production-ready
2. **Zero Overhead**: No runtime impact for static features
3. **Cross-Platform**: Works in all environments
4. **Modular Design**: Pay-for-what-you-use features

### Planned Enhancements
1. **Runtime Integration**: Full debugging capabilities
2. **Hybrid Execution**: Optimal performance with debug capability
3. **Feature Granularity**: Fine-grained control over capabilities
4. **Future-Proof**: Designed for both interpreter and AOT

### Key Design Principles
1. **Feature-Gated**: All runtime features are optional
2. **Performance-Conscious**: Minimal overhead by default
3. **Flexible Integration**: Works with multiple runtime strategies
4. **Backward Compatible**: Static features always available

The proposed architecture provides a clear evolution path from current static debugging to full runtime debugging capabilities while maintaining the flexibility to support different execution strategies and performance requirements.