# WRT Debug Capabilities Guide

This document provides a comprehensive overview of what debugging scenarios are now supported and what remains outside our current capabilities.

## ✅ What We CAN Debug Now

### 1. **Crash Analysis & Panic Debugging**
```rust
// When your WASM module crashes, we can tell you:
✓ Exact crash location: "src/server.rs:42:15"
✓ Function context: "in process_request(req: ptr, timeout: u32)"
✓ Whether code was inlined: "[inlined from validate() at server.rs:38]"
✓ Call stack at crash (with runtime support)
✓ Source file and line numbers
```

**Real-world scenario**: Production WASM module panics
- **Before**: "Module trapped at address 0x1234"
- **Now**: "Panic in process_request() at src/server.rs:42:15, parameter timeout=0"

### 2. **Function Call Analysis**
```rust
// We can analyze any function call:
✓ Function name and signature
✓ Parameter count and types
✓ Source location of definition
✓ Whether it's inlined or direct call
✓ Address ranges for profiling
```

**Use cases**:
- Performance profiling (which functions are hot?)
- Call graph analysis (what calls what?)
- Code coverage mapping
- Security auditing (function boundary checks)

### 3. **Source Code Mapping**
```rust
// Map any instruction address to source:
✓ Full file path: "src/handlers/auth.rs"
✓ Line and column: "line 42, column 8"
✓ Statement boundaries (for stepping)
✓ Basic block boundaries
✓ Multiple source files in project
```

**Use cases**:
- Breakpoint setting (by file:line)
- Error reporting with context
- Code coverage visualization
- Performance hot-spot identification

### 4. **Inline Function Transparency**
```rust
// Understand optimized code:
✓ Detect inlined functions
✓ Show inline call chains
✓ Map addresses to multiple logical functions
✓ Preserve call site information
```

**Example**: 
```
Address 0x1000 corresponds to:
- Directly: process_data() at data.rs:100
- Inlined: validate_input() called from data.rs:95
- Inlined: check_bounds() called from data.rs:90
```

### 5. **Multi-Module Projects**
```rust
// Handle real-world projects:
✓ Multiple compilation units
✓ Cross-module function calls
✓ Separate source files
✓ Library and application code
```

### 6. **Basic Type Information**
```rust
// Understand data types:
✓ Primitive types: i32, u64, f32, bool
✓ Pointer detection
✓ Function parameter types
✓ Basic array/struct recognition
```

## ❌ What We CANNOT Debug (Yet)

### 1. **Variable Values**
```rust
// We CANNOT:
✗ Read local variable values
✗ Inspect function parameters at runtime
✗ Show variable assignments
✗ Track variable lifetime/scope
```

**Why**: Requires DWARF location expressions and runtime memory access

### 2. **Complex Type Details**
```rust
// We CANNOT:
✗ Show struct field names/offsets
✗ Resolve enum variants
✗ Display array dimensions
✗ Show type hierarchies/inheritance
✗ Handle generic/template types
```

**Why**: Requires full DWARF type information parsing (DW_TAG_structure_type, etc.)

### 3. **Runtime State Inspection**
```rust
// We CANNOT:
✗ Read WebAssembly memory contents
✗ Inspect the operand stack
✗ Show current register values
✗ Display heap allocations
✗ Track memory usage
```

**Why**: Requires deep runtime integration, not just debug info

### 4. **Advanced Debugging Operations**
```rust
// We CANNOT:
✗ Set breakpoints (just identify locations)
✗ Single-step execution
✗ Modify variable values
✗ Conditional breakpoints
✗ Watchpoints on memory
```

**Why**: Requires runtime control and execution engine integration

### 5. **Call Stack Unwinding**
```rust
// We CANNOT:
✗ Full stack traces (only current frame)
✗ Show all function parameters in stack
✗ Display return addresses
✗ Unwind through exceptions
```

**Why**: Requires .debug_frame parsing and runtime stack access

### 6. **Source Code Display**
```rust
// We CANNOT:
✗ Show actual source code lines
✗ Syntax highlighting
✗ Show surrounding context
✗ Display comments
```

**Why**: Source code not embedded in DWARF (only references)

### 7. **Advanced DWARF Features**
```rust
// We CANNOT:
✗ Macro expansion info (.debug_macro)
✗ Split debug info (.dwo files)
✗ Compressed debug sections
✗ DWARF 5 features
✗ Location lists for optimized code
```

## 📊 Debugging Capability Matrix

| Feature | Supported | Limitation |
|---------|-----------|------------|
| **Crash Location** | ✅ Full | File:line:column precision |
| **Function Names** | ✅ Full | Including mangled names |
| **Parameter Types** | ✅ Basic | Types only, not values |
| **Inline Detection** | ✅ Full | All inline chains |
| **Source Mapping** | ✅ Full | Address ↔ location |
| **Variable Values** | ❌ None | Cannot read runtime values |
| **Breakpoints** | ⚠️ Partial | Can identify locations only |
| **Stack Traces** | ⚠️ Partial | Current frame + PC list |
| **Type Details** | ⚠️ Basic | Primitives only |
| **Memory Inspection** | ❌ None | No runtime memory access |

## 🎯 Practical Debugging Scenarios

### ✅ Scenarios We Handle Well

1. **"Where did my program crash?"**
   - Full source location with file:line:column
   - Function name and parameter types
   - Inline function context

2. **"What function is at this address?"**
   - Function name, boundaries, source location
   - Parameter signatures
   - Inline status

3. **"Generate an error report"**
   ```
   Error in process_request(req: ptr, timeout: u32)
     at src/server.rs:42:15
     inlined from validate_timeout() at src/server.rs:38:10
   ```

4. **"Profile hot functions"**
   - Map addresses to functions
   - Identify inline expansion
   - Source-level attribution

5. **"Set a breakpoint at file:line"**
   - Resolve file:line to address
   - Handle optimized/inlined code
   - Multiple breakpoint locations

### ❌ Scenarios We DON'T Handle

1. **"What's the value of variable X?"**
   - No variable value inspection
   - No runtime state access

2. **"Show me the call stack with all parameters"**
   - Only addresses, not full frames
   - No parameter values

3. **"Step through the code"**
   - No execution control
   - No single-stepping

4. **"What's in this struct?"**
   - No field names/offsets
   - No complex type layouts

5. **"Watch this memory location"**
   - No memory access
   - No watchpoints

## 🔧 Integration Requirements

### To Enable Our Debugging Features:
1. **Compile with debug info**: `rustc --emit=wasm -g` or `clang -g`
2. **Include debug sections**: .debug_info, .debug_line, .debug_str, .debug_abbrev
3. **Integrate with runtime**: Pass debug sections to wrt-debug
4. **Feature flags**: Enable desired features (line-info, debug-info, etc.)

### For Additional Debugging Capabilities, You'd Need:
1. **Runtime integration**: For variable values and memory inspection
2. **Execution control**: For breakpoints and stepping
3. **Stack walker**: For full stack traces
4. **Source server**: For displaying actual source code
5. **Debug protocol**: For IDE integration (DAP, etc.)

## 📈 Future Enhancement Path

To reach full debugging capabilities:

1. **Phase 1** (Current) ✅: Static debug info (locations, functions, types)
2. **Phase 2**: Runtime state (variables, memory, stack)
3. **Phase 3**: Execution control (breakpoints, stepping)
4. **Phase 4**: Advanced features (conditional breakpoints, expressions)
5. **Phase 5**: IDE integration (Debug Adapter Protocol)

## 🎓 Summary

**We CAN debug**:
- ✅ Crashes and panics with full context
- ✅ Function calls and parameters (types only)
- ✅ Source locations and inline functions
- ✅ Basic profiling and coverage

**We CANNOT debug**:
- ❌ Variable values or runtime state
- ❌ Complex types (structs, enums)
- ❌ Interactive debugging (stepping, breakpoints)
- ❌ Memory contents or heap state

**Best suited for**:
- Production crash analysis
- Performance profiling
- Error reporting
- Static code analysis

**Not suited for**:
- Interactive debugging sessions
- Runtime state inspection
- Memory leak detection
- Step-by-step debugging