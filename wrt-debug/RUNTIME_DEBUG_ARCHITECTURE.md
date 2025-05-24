# Runtime Debug Architecture & AOT Considerations

## 🎯 Runtime Debug Features Architecture

### Proposed Feature Structure
```toml
[features]
# Current static features (no runtime needed)
static-line-info = []
static-function-info = []
static-debug-info = ["static-line-info", "static-function-info"]

# NEW: Runtime-integrated features
runtime-inspection = ["static-debug-info"]
runtime-breakpoints = ["runtime-inspection"]
runtime-stepping = ["runtime-breakpoints"]
runtime-eval = ["runtime-inspection"]
runtime-memory = ["runtime-inspection"]

# Feature groups
static-debug = ["static-debug-info"]  # What we have now
runtime-debug = ["runtime-stepping", "runtime-memory", "runtime-eval"]  # New capabilities
full-debug = ["static-debug", "runtime-debug"]  # Everything
```

## 📦 New Modules for Runtime Debugging

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

### 2. **Breakpoint Manager** (`runtime_breakpoints.rs`)
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

## 🔄 Interpreter vs AOT: Debugging Implications

### Current Situation: Interpreter-based Execution

#### Advantages for Debugging:
```rust
// Easy to instrument
interpreter.set_trace_callback(|pc, instr| {
    debugger.on_instruction(pc, instr);
});

// Natural breakpoint support
if breakpoints.contains(pc) {
    return DebugAction::Break;
}

// Easy state inspection
let locals = frame.locals.clone();
let stack = interpreter.stack.clone();

// Single-stepping is trivial
interpreter.step_one_instruction();
```

#### Current Limitations:
- Performance overhead always present
- Limited optimization opportunities
- But debugging is "free"

### AOT Compilation Scenario

#### Debugging Challenges:
```rust
// Generated native code - no natural hook points
// Need to inject debug trampolines
fn compile_with_debug(wasm: &[u8]) -> NativeCode {
    let mut codegen = AotCodegen::new();
    
    // Insert debug checks at:
    // - Function entries/exits
    // - Line boundaries  
    // - Potential breakpoint sites
    codegen.insert_debug_trampoline(pc, |state| {
        if debug_enabled() {
            debugger.check_breakpoint(pc, state);
        }
    });
}
```

#### AOT Debug Strategies:

1. **Debug vs Release Builds**:
```rust
// Debug build: Full instrumentation
#[cfg(debug_assertions)]
fn emit_debug_prologue(&mut self) {
    // Save all registers
    // Call debug hook
    // Check breakpoints
}

// Release build: No overhead
#[cfg(not(debug_assertions))]
fn emit_debug_prologue(&mut self) { /* noop */ }
```

2. **Hybrid Approach**:
```rust
enum ExecutionMode {
    /// Fast AOT execution
    Native,
    /// Fall back to interpreter for debugging
    Interpreted,
    /// AOT with debug instrumentation
    NativeDebug,
}
```

3. **Deoptimization for Debugging**:
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

## 📊 Feature Comparison Table

| Capability | Static Only | + Runtime (Interp) | + Runtime (AOT) |
|------------|-------------|-------------------|-----------------|
| Crash Location | ✅ Full | ✅ Full | ✅ Full |
| Function Names | ✅ Full | ✅ Full | ✅ Full |
| Variable Values | ❌ | ✅ Full | ⚠️ Limited¹ |
| Breakpoints | ❌ | ✅ Trivial | ⚠️ Complex² |
| Single Step | ❌ | ✅ Natural | ⚠️ Emulated³ |
| Memory Inspect | ❌ | ✅ Direct | ✅ Direct |
| Stack Unwind | ❌ | ✅ Easy | ⚠️ Harder⁴ |
| Performance | ✅ No impact | ⚠️ Overhead | ⚠️ Configurable |

¹ Register allocation may hide variables
² Requires code instrumentation
³ May need instruction-level boundaries
⁴ Need to preserve frame pointers

## 🏗️ Implementation Strategy

### Phase 1: Runtime Interface Design
```rust
/// Core runtime debug trait
pub trait DebugRuntime {
    type State: RuntimeState;
    type Memory: MemoryInspector;
    
    fn attach_debugger(&mut self, debugger: Box<dyn Debugger>);
    fn get_state(&self) -> &Self::State;
    fn get_memory(&self) -> &Self::Memory;
}
```

### Phase 2: Interpreter Integration
```rust
impl DebugRuntime for WrtInterpreter {
    // Natural integration with interpreter
    // Full access to all state
}
```

### Phase 3: AOT Integration Options

#### Option A: Debug-Instrumented AOT
```rust
// Compile with debug hooks
let code = compile_wasm_with_debug(wasm, DebugLevel::Full);
// ~10-30% performance overhead
```

#### Option B: Hybrid Execution
```rust
// AOT for normal execution
// Interpreter for debugging
match execution_mode {
    Normal => execute_aot(pc),
    Debugging => execute_interpreted(pc),
}
```

#### Option C: JIT Deoptimization
```rust
// Start optimized, deoptimize on demand
// Like modern JavaScript engines
```

## 📈 Performance Impact Analysis

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

## 🎯 Recommended Architecture

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

## 📝 Summary

### For Interpreter-based WRT:
- Runtime debugging is **natural and low-cost**
- Add `runtime-*` features for full debugging
- Minimal performance impact
- Complete debugging experience possible

### For Future AOT-based WRT:
- Runtime debugging requires **careful design**
- Multiple strategies available:
  - Debug builds with instrumentation
  - Hybrid interpreter fallback
  - JIT-style deoptimization
- Trade-off between performance and debuggability

### Recommended Approach:
1. Implement runtime features for interpreter ✅
2. Design with AOT in mind 🎯
3. Use hybrid approach for AOT 🔄
4. Provide debug/release build options ⚙️

The key insight: **Debugging capabilities should be runtime features**, not just static analysis. The implementation strategy differs between interpreter and AOT, but the API can remain consistent.