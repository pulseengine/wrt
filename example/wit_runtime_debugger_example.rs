//! Example demonstrating WIT debugger integration with WRT runtime
//!
//! This example shows how to create a debuggable runtime with WIT support
//! and attach a WIT-aware debugger for source-level debugging.

#[cfg(feature = "wit-debug-integration")]
fn main() {
    use wrt_runtime::{
        DebuggableWrtRuntime, ComponentMetadata, FunctionMetadata, TypeMetadata, WitTypeKind,
        Breakpoint, BreakpointCondition, create_component_metadata, create_function_metadata,
        create_type_metadata, create_wit_enabled_runtime,
    };
    use wrt_debug::{SourceSpan, ComponentId, FunctionId, TypeId, BreakpointId, DebugAction};
    use wrt_error::Result;
    
    println!("WIT Runtime Debugger Integration Example");
    println!("========================================");
    
    // Create a debuggable runtime
    let mut runtime = create_wit_enabled_runtime();
    println!("✓ Created debuggable WRT runtime");
    
    // Create component metadata
    let comp_span = SourceSpan::new(0, 100, 0);
    let comp_meta = create_component_metadata("example-component", comp_span, 1000, 2000)
        .expect("Failed to create component metadata");
    
    // Create function metadata
    let func_span = SourceSpan::new(10, 50, 0);
    let func_meta = create_function_metadata("greet", func_span, 1200, false)
        .expect("Failed to create function metadata");
    
    // Create type metadata
    let type_span = SourceSpan::new(5, 15, 0);
    let type_meta = create_type_metadata("string", type_span, WitTypeKind::Primitive, Some(8))
        .expect("Failed to create type metadata");
    
    println!("✓ Created debug metadata");
    
    // Create WIT debugger
    let wit_debugger = DebuggableWrtRuntime::create_wit_debugger();
    
    // Attach debugger with metadata
    runtime.attach_wit_debugger_with_components(
        wit_debugger,
        vec![(ComponentId(1), comp_meta)],
        vec![(FunctionId(1), func_meta)],
        vec![(TypeId(1), type_meta)],
    );
    
    println!("✓ Attached WIT debugger to runtime");
    
    // Enable debug mode
    runtime.set_debug_mode(true);
    println!("✓ Enabled debug mode");
    
    // Add breakpoints
    let bp1 = Breakpoint {
        id: BreakpointId(0), // Will be assigned automatically
        address: 1200,
        file_index: Some(0),
        line: Some(10),
        condition: None,
        hit_count: 0,
        enabled: true,
    };
    
    let bp2 = Breakpoint {
        id: BreakpointId(0),
        address: 1300,
        file_index: Some(0),
        line: Some(15),
        condition: Some(BreakpointCondition::HitCount(2)),
        hit_count: 0,
        enabled: true,
    };
    
    runtime.add_breakpoint(bp1).expect("Failed to add breakpoint 1");
    runtime.add_breakpoint(bp2).expect("Failed to add breakpoint 2");
    println!("✓ Added breakpoints");
    
    // Simulate execution
    println!("\n--- Simulating Execution ---");
    
    // Set up some runtime state
    runtime.state_mut().set_pc(1200);
    runtime.state_mut().set_current_function(1);
    runtime.state_mut().add_local(42).expect("Failed to add local");
    runtime.state_mut().push_stack(123).expect("Failed to push stack");
    
    // Simulate memory
    let memory_data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    runtime.memory_mut().set_memory_data(&memory_data).expect("Failed to set memory");
    
    println!("✓ Set up runtime state and memory");
    
    // Enter function
    runtime.enter_function(1);
    println!("→ Entered function 1 (call depth: {})", runtime.call_depth());
    
    // Execute instructions with debugging
    let instructions = [1200, 1210, 1220, 1300, 1300, 1310]; // Repeat 1300 to test hit count
    
    for (i, &addr) in instructions.iter().enumerate() {
        println!("\nInstruction {}: PC=0x{:X}", i + 1, addr);
        
        match runtime.execute_instruction(addr) {
            Ok(action) => {
                println!("  Debug action: {:?}", action);
                match action {
                    DebugAction::Break => {
                        println!("  🛑 Breakpoint hit!");
                        // In a real debugger, you'd inspect state here
                        let state = runtime.get_state();
                        println!("    PC: 0x{:X}", state.pc());
                        if let Some(func) = state.current_function() {
                            println!("    Function: {}", func);
                        }
                        if let Some(local0) = state.read_local(0) {
                            println!("    Local[0]: {}", local0);
                        }
                        if let Some(stack0) = state.read_stack(0) {
                            println!("    Stack[0]: {}", stack0);
                        }
                    },
                    DebugAction::Continue => {
                        println!("  ✓ Continue execution");
                    },
                    _ => {
                        println!("  ⏯️  Debug step: {:?}", action);
                    }
                }
            },
            Err(e) => {
                println!("  ❌ Execution error: {:?}", e);
                runtime.handle_trap(1); // Generic trap code
            }
        }
        
        println!("  Instructions executed: {}", runtime.instruction_count());
    }
    
    // Exit function
    runtime.exit_function(1);
    println!("\n← Exited function 1 (call depth: {})", runtime.call_depth());
    
    // Test memory access
    println!("\n--- Memory Debugging ---");
    let memory = runtime.get_memory();
    
    println!("Memory is valid at 0x0: {}", memory.is_valid_address(0));
    println!("Memory is valid at 0x100: {}", memory.is_valid_address(0x100));
    
    if let Some(bytes) = memory.read_bytes(2, 4) {
        println!("Memory[2..6]: {:?}", bytes);
    }
    
    if let Some(u32_val) = memory.read_u32(0) {
        println!("Memory u32 at 0: 0x{:08X}", u32_val);
    }
    
    // Show execution statistics
    println!("\n--- Execution Statistics ---");
    println!("Total instructions executed: {}", runtime.instruction_count());
    println!("Maximum call depth reached: {}", runtime.call_depth());
    
    // Demonstrate debugger attachment/detachment
    println!("\n--- Debugger Management ---");
    println!("Debugger attached: {}", runtime.has_debugger());
    
    runtime.detach_debugger();
    println!("Debugger detached: {}", !runtime.has_debugger());
    
    println!("\n--- Integration Benefits ---");
    println!("1. Source-level debugging of WIT components");
    println!("2. Breakpoints at WIT source locations");
    println!("3. Variable inspection in WIT context");
    println!("4. Component boundary tracking");
    println!("5. Function call tracing");
    println!("6. Memory debugging with WIT type information");
    println!("7. Runtime state inspection");
    println!("8. Configurable debug modes and stepping");
    
    println!("\nWIT runtime debugger integration example completed!");
}

#[cfg(not(feature = "wit-debug-integration"))]
fn main() {
    println!("This example requires the 'wit-debug-integration' feature");
    println!("Run with: cargo run --example wit_runtime_debugger_example --features wit-debug-integration");
}