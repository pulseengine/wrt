//! Demonstration of the Unified Execution Agent
//!
//! This example shows how to use the new unified agent system for various
//! WebAssembly execution scenarios.

use wrt_component::{
    AgentConfiguration, AgentCreationOptions, AgentRegistry, ExecutionMode, HybridModeFlags,
    PreferredAgentType, UnifiedExecutionAgent, UnifiedExecutionState, Value,
};

fn main() {
    println!("=== WRT Unified Execution Agent Demo ===\n");

    // Demo 1: Basic synchronous execution
    demo_synchronous_execution();

    // Demo 2: Async execution
    demo_async_execution();

    // Demo 3: Stackless execution
    demo_stackless_execution();

    // Demo 4: CFI-protected execution
    demo_cfi_protected_execution();

    // Demo 5: Hybrid mode execution
    demo_hybrid_execution();

    // Demo 6: Using the agent registry
    demo_agent_registry();

    // Demo 7: Migration from legacy agents
    demo_legacy_migration();
}

fn demo_synchronous_execution() {
    println!("1. Synchronous Execution Demo");
    println!("-----------------------------");

    // Create a unified agent with default synchronous mode
    let config = AgentConfiguration::default();
    let mut agent = UnifiedExecutionAgent::new(config);

    // Prepare function arguments
    let args = vec![Value::U32(42), Value::F64(3.14159), Value::Bool(true)];

    // Execute a function
    match agent.call_function(1, 100, &args) {
        Ok(result) => {
            println!("Function executed successfully!");
            println!("Result: {:?}", result);
            println!("State: {:?}", agent.state());
            println!("Statistics: {:?}", agent.statistics());
        }
        Err(e) => println!("Execution failed: {:?}", e),
    }

    println!();
}

fn demo_async_execution() {
    println!("2. Async Execution Demo");
    println!("----------------------");

    #[cfg(feature = "async")]
    {
        // Create agent configured for async execution
        let config = AgentConfiguration {
            execution_mode: ExecutionMode::Asynchronous,
            ..AgentConfiguration::default()
        };
        let mut agent = UnifiedExecutionAgent::new(config);

        // Execute async function
        let args = vec![Value::String("async_task".to_string())];

        match agent.call_function(2, 200, &args) {
            Ok(result) => {
                println!("Async function started!");
                println!("Result: {:?}", result);

                // In real usage, you would poll or await the async operation
                println!("Async operations tracked: {}", agent.statistics().async_operations);
            }
            Err(e) => println!("Async execution failed: {:?}", e),
        }
    }

    #[cfg(not(feature = "async"))]
    println!("Async feature not enabled. Compile with --features async");

    println!();
}

fn demo_stackless_execution() {
    println!("3. Stackless Execution Demo");
    println!("--------------------------");

    // Create agent for stackless execution (memory-constrained environments)
    let mut agent = UnifiedExecutionAgent::new_stackless();

    // Execute function without using system call stack
    let args = vec![Value::U32(1000)];

    match agent.call_function(3, 300, &args) {
        Ok(result) => {
            println!("Stackless execution successful!");
            println!("Result: {:?}", result);
            println!("Stackless frames: {}", agent.statistics().stackless_frames);
        }
        Err(e) => println!("Stackless execution failed: {:?}", e),
    }

    println!();
}

fn demo_cfi_protected_execution() {
    println!("4. CFI-Protected Execution Demo");
    println!("------------------------------");

    #[cfg(feature = "cfi")]
    {
        // Create agent with CFI protection enabled
        let mut agent = UnifiedExecutionAgent::new_cfi_protected();

        // Execute function with control flow integrity protection
        let args = vec![Value::U64(0xDEADBEEF)];

        match agent.call_function(4, 400, &args) {
            Ok(result) => {
                println!("CFI-protected execution successful!");
                println!("Result: {:?}", result);
                println!(
                    "CFI-protected instructions: {}",
                    agent.statistics().cfi_instructions_protected
                );
                println!("CFI violations detected: {}", agent.statistics().cfi_violations_detected);
            }
            Err(e) => println!("CFI-protected execution failed: {:?}", e),
        }
    }

    #[cfg(not(feature = "cfi"))]
    println!("CFI feature not enabled. Compile with --features cfi");

    println!();
}

fn demo_hybrid_execution() {
    println!("5. Hybrid Mode Execution Demo");
    println!("----------------------------");

    // Create agent with multiple capabilities enabled
    let flags = HybridModeFlags {
        async_enabled: cfg!(feature = "async"),
        stackless_enabled: true,
        cfi_enabled: cfg!(feature = "cfi"),
    };

    let mut agent = UnifiedExecutionAgent::new_hybrid(flags);

    println!("Hybrid mode enabled with:");
    println!("  - Async: {}", flags.async_enabled);
    println!("  - Stackless: {}", flags.stackless_enabled);
    println!("  - CFI: {}", flags.cfi_enabled);

    // Execute function with combined capabilities
    let args = vec![Value::String("hybrid_test".to_string())];

    match agent.call_function(5, 500, &args) {
        Ok(result) => {
            println!("Hybrid execution successful!");
            println!("Result: {:?}", result);

            let stats = agent.statistics();
            println!("Combined statistics:");
            println!("  - Instructions: {}", stats.instructions_executed);
            println!("  - Stackless frames: {}", stats.stackless_frames);

            #[cfg(feature = "async")]
            println!("  - Async operations: {}", stats.async_operations);

            #[cfg(feature = "cfi")]
            println!("  - CFI protected: {}", stats.cfi_instructions_protected);
        }
        Err(e) => println!("Hybrid execution failed: {:?}", e),
    }

    println!();
}

fn demo_agent_registry() {
    println!("6. Agent Registry Demo");
    println!("--------------------");

    // Create a registry to manage multiple agents
    let mut registry = AgentRegistry::new();

    // Create multiple agents with different configurations
    let sync_agent_id = registry
        .create_unified_agent(AgentConfiguration::default())
        .expect("Failed to create sync agent");

    let stackless_config = AgentConfiguration {
        execution_mode: ExecutionMode::Stackless,
        max_memory: 64 * 1024, // 64KB for embedded
        ..AgentConfiguration::default()
    };
    let stackless_agent_id =
        registry.create_unified_agent(stackless_config).expect("Failed to create stackless agent");

    println!("Created {} agents in registry", registry.statistics().active_agents);

    // Execute functions on different agents
    let args = vec![Value::U32(777)];

    println!("\nExecuting on sync agent:");
    match registry.call_function(sync_agent_id, 1, 100, &args) {
        Ok(result) => println!("  Result: {:?}", result),
        Err(e) => println!("  Error: {:?}", e),
    }

    println!("\nExecuting on stackless agent:");
    match registry.call_function(stackless_agent_id, 1, 100, &args) {
        Ok(result) => println!("  Result: {:?}", result),
        Err(e) => println!("  Error: {:?}", e),
    }

    // Get agent information
    if let Some(info) = registry.get_agent_info(sync_agent_id) {
        println!("\nSync agent info:");
        println!("  Type: {:?}", info.agent_type);
        println!("  Migration status: {:?}", info.migration_status);
    }

    println!();
}

fn demo_legacy_migration() {
    println!("7. Legacy Agent Migration Demo");
    println!("-----------------------------");

    let mut registry = AgentRegistry::new();

    // Create a legacy agent (for demonstration)
    println!("Creating legacy component agent...");
    let legacy_id =
        registry.create_legacy_component_agent().expect("Failed to create legacy agent");

    // Check migration status
    let migration_status = registry.migration_status();
    println!("Pending migrations: {}", migration_status.pending_migrations.len());

    // Get agent info before migration
    if let Some(info) = registry.get_agent_info(legacy_id) {
        println!("\nBefore migration:");
        println!("  Agent type: {:?}", info.agent_type);
        println!("  Migration status: {:?}", info.migration_status);
    }

    // Migrate the agent
    println!("\nMigrating legacy agent to unified...");
    match registry.migrate_agent(legacy_id) {
        Ok(()) => {
            println!("Migration successful!");

            // Check status after migration
            if let Some(info) = registry.get_agent_info(legacy_id) {
                println!("\nAfter migration:");
                println!("  Agent type: {:?}", info.agent_type);
                println!("  Migration status: {:?}", info.migration_status);
            }

            println!("Completed migrations: {}", registry.migration_status().completed_migrations);
        }
        Err(e) => println!("Migration failed: {:?}", e),
    }

    // Test the migrated agent
    println!("\nTesting migrated agent:");
    let args = vec![Value::Bool(true)];
    match registry.call_function(legacy_id, 1, 100, &args) {
        Ok(result) => println!("  Execution successful: {:?}", result),
        Err(e) => println!("  Execution failed: {:?}", e),
    }

    println!();
}

// Helper function to print separator
fn print_separator() {
    println!("\n{}", "=".repeat(50));
    println!();
}
