# WRT Agent Unification and Migration Guide

This guide explains the unification of WebAssembly execution agents in the WRT (WebAssembly Runtime) project and provides migration instructions for transitioning from legacy agents to the unified agent system.

## Overview

The WRT project previously had multiple specialized execution agents:
- `ComponentExecutionEngine` - For WebAssembly Component Model execution
- `AsyncExecutionEngine` - For asynchronous WebAssembly operations  
- `StacklessEngine` - For memory-constrained stackless execution
- `CfiExecutionEngine` - For CFI-protected secure execution

These have been consolidated into a single `UnifiedExecutionAgent` that provides all capabilities through configurable execution modes.

## Benefits of Unification

### 1. **Reduced Code Duplication**
- Eliminates redundant execution state management
- Consolidates call stack handling
- Unifies resource management
- Shared statistics and error handling

### 2. **Improved Performance**
- Single agent instance reduces memory overhead
- Optimized execution paths
- Better resource utilization
- Reduced context switching

### 3. **Enhanced Flexibility**
- Hybrid execution modes combining multiple capabilities
- Runtime mode switching
- Configurable feature sets
- Better testing and debugging

### 4. **Simplified API**
- Single interface for all execution types
- Consistent error handling
- Unified statistics collection
- Easier integration

## Migration Path

### Phase 1: Legacy Support (Current)
- Legacy agents are still available but marked as deprecated
- New `UnifiedExecutionAgent` is available alongside legacy agents
- `AgentRegistry` provides migration utilities
- Full backward compatibility maintained

### Phase 2: Migration Period (Next Release)
- Legacy agents will emit deprecation warnings
- Migration tools will be provided
- Documentation will be updated
- Performance benefits will be highlighted

### Phase 3: Legacy Removal (Future Release)
- Legacy agents will be removed
- Only `UnifiedExecutionAgent` will be available
- Breaking changes will be clearly documented

## Code Migration Examples

### Basic Component Execution

**Before (Legacy):**
```rust
use wrt_component::ComponentExecutionEngine;

let mut engine = ComponentExecutionEngine::new();
let result = engine.call_function(instance_id, func_idx, &args)?;
```

**After (Unified):**
```rust
use wrt_component::{UnifiedExecutionAgent, AgentConfiguration, ExecutionMode};

let config = AgentConfiguration {
    execution_mode: ExecutionMode::Synchronous,
    ..AgentConfiguration::default()
};
let mut agent = UnifiedExecutionAgent::new(config);
let result = agent.call_function(instance_id, func_idx, &args)?;
```

### Async Execution

**Before (Legacy):**
```rust
use wrt_component::AsyncExecutionEngine;

let mut engine = AsyncExecutionEngine::new();
let execution_id = engine.start_execution(task_id, operation, None)?;
let result = engine.step_execution(execution_id)?;
```

**After (Unified):**
```rust
use wrt_component::{UnifiedExecutionAgent, AgentConfiguration, ExecutionMode};

let config = AgentConfiguration {
    execution_mode: ExecutionMode::Asynchronous,
    ..AgentConfiguration::default()
};
let mut agent = UnifiedExecutionAgent::new(config);
let result = agent.call_function(instance_id, func_idx, &args)?;
// Async operations are handled internally
```

### Stackless Execution

**Before (Legacy):**
```rust
use wrt_runtime::StacklessEngine;

let mut engine = StacklessEngine::new();
// Complex stackless setup...
```

**After (Unified):**
```rust
use wrt_component::{UnifiedExecutionAgent, AgentConfiguration, ExecutionMode};

let config = AgentConfiguration {
    execution_mode: ExecutionMode::Stackless,
    ..AgentConfiguration::default()
};
let mut agent = UnifiedExecutionAgent::new(config);
// Stackless execution is handled automatically
```

### CFI-Protected Execution

**Before (Legacy):**
```rust
use wrt_runtime::CfiExecutionEngine;

let protection = CfiControlFlowProtection::default();
let mut engine = CfiExecutionEngine::new(protection);
// CFI-specific setup...
```

**After (Unified):**
```rust
use wrt_component::{UnifiedExecutionAgent, AgentConfiguration, ExecutionMode};

let config = AgentConfiguration {
    execution_mode: ExecutionMode::CfiProtected,
    ..AgentConfiguration::default()
};
let mut agent = UnifiedExecutionAgent::new(config);
// CFI protection is enabled automatically
```

### Hybrid Execution (New Capability)

**New Unified Feature:**
```rust
use wrt_component::{UnifiedExecutionAgent, AgentConfiguration, ExecutionMode, HybridModeFlags};

let flags = HybridModeFlags {
    async_enabled: true,
    stackless_enabled: true,
    cfi_enabled: true,
};
let config = AgentConfiguration {
    execution_mode: ExecutionMode::Hybrid(flags),
    ..AgentConfiguration::default()
};
let mut agent = UnifiedExecutionAgent::new(config);
// Combines async, stackless, and CFI capabilities
```

## Using the Agent Registry

The `AgentRegistry` provides a centralized way to manage multiple agents and handle migration:

### Creating Agents Through Registry

```rust
use wrt_component::{AgentRegistry, AgentCreationOptions, PreferredAgentType, AgentConfiguration};

let mut registry = AgentRegistry::new();

// Create unified agent (recommended)
let options = AgentCreationOptions {
    agent_type: PreferredAgentType::Unified,
    config: AgentConfiguration::default(),
    allow_legacy_fallback: false,
};
let agent_id = registry.create_agent(options)?;

// Execute functions
let result = registry.call_function(agent_id, instance_id, func_idx, &args)?;
```

### Migrating Legacy Agents

```rust
use wrt_component::{AgentRegistry, PreferredAgentType};

let mut registry = AgentRegistry::new();

// Create legacy agent (for compatibility)
let legacy_id = registry.create_legacy_component_agent()?;

// Migrate to unified agent
registry.migrate_agent(legacy_id)?;

// Or migrate all at once
let migrated_count = registry.migrate_all_agents()?;
println!("Migrated {} agents", migrated_count);
```

### Monitoring Migration Status

```rust
use wrt_component::{AgentRegistry, AgentType, AgentMigrationStatus};

let registry = AgentRegistry::new();

// Check agent status
if let Some(info) = registry.get_agent_info(agent_id) {
    match info.agent_type {
        AgentType::Unified => println!("Agent is already unified"),
        AgentType::Legacy => match info.migration_status {
            AgentMigrationStatus::Available => println!("Agent can be migrated"),
            AgentMigrationStatus::Pending => println!("Agent migration is pending"),
            _ => {}
        }
    }
}

// Get migration statistics
let stats = registry.migration_status();
println!("Completed migrations: {}", stats.completed_migrations);
println!("Pending migrations: {}", stats.pending_migrations.len());
```

## Configuration Migration

### Agent Configuration Mapping

| Legacy Agent | Unified Configuration |
|-------------|----------------------|
| `ComponentExecutionEngine` | `ExecutionMode::Synchronous` |
| `AsyncExecutionEngine` | `ExecutionMode::Asynchronous` |
| `StacklessEngine` | `ExecutionMode::Stackless` |
| `CfiExecutionEngine` | `ExecutionMode::CfiProtected` |

### Advanced Configuration

```rust
use wrt_component::{AgentConfiguration, ExecutionMode, HybridModeFlags};

let config = AgentConfiguration {
    max_call_depth: 1024,
    max_memory: 1024 * 1024, // 1MB
    execution_mode: ExecutionMode::Hybrid(HybridModeFlags {
        async_enabled: true,
        stackless_enabled: false,
        cfi_enabled: true,
    }),
    bounded_execution: true,
    initial_fuel: Some(10000),
    runtime_config: RuntimeBridgeConfig::default(),
};
```

## Breaking Changes

### API Changes
- Legacy agent constructors are deprecated
- Some legacy-specific methods are no longer available
- Error types have been unified
- Statistics collection is now unified

### Feature Changes
- Async execution API is simplified
- Stackless execution is automatic when enabled
- CFI protection is transparent
- Resource management is unified

### Performance Changes
- Memory usage may change due to unified data structures
- Execution overhead may be different
- Statistics collection is more comprehensive

## Migration Checklist

- [ ] **Inventory Legacy Usage**: Identify all uses of legacy agents in your codebase
- [ ] **Update Dependencies**: Ensure you have the latest WRT version with unified agents
- [ ] **Create Test Suite**: Set up tests for your migration
- [ ] **Migrate Configuration**: Convert legacy configurations to unified format
- [ ] **Update Code**: Replace legacy agent creation with unified agents
- [ ] **Test Functionality**: Verify that functionality is preserved
- [ ] **Performance Testing**: Compare performance before and after migration
- [ ] **Update Documentation**: Update your project documentation
- [ ] **Remove Legacy Code**: Clean up deprecated legacy agent usage
- [ ] **Monitor Production**: Watch for any issues in production

## Troubleshooting

### Common Migration Issues

**1. Feature Not Available**
```
Error: Feature not available in unified agent
```
**Solution**: Check if the feature is available through configuration or hybrid mode.

**2. Performance Regression**
```
Warning: Performance impact detected
```
**Solution**: Tune the agent configuration or use hybrid mode to optimize performance.

**3. API Changes**
```
Error: Method not found
```
**Solution**: Consult the API mapping table and update method calls.

### Getting Help

- Check the [API documentation](docs/api.md)
- Review [examples](examples/) for migration patterns
- Open an issue on the [WRT repository](https://github.com/wrt-project/wrt)
- Ask questions in the [community forum](https://forum.wrt-project.org)

## Future Roadmap

### Planned Improvements
- Enhanced hybrid mode capabilities
- Runtime mode switching
- Advanced performance monitoring
- Better debugging support

### Timeline
- **v2.1**: Legacy deprecation warnings
- **v2.2**: Migration utilities and documentation
- **v2.3**: Performance optimizations
- **v3.0**: Legacy agent removal

## Conclusion

The unification of WRT execution agents provides significant benefits in terms of performance, maintainability, and usability. While migration requires some code changes, the unified system offers enhanced capabilities and a more consistent development experience.

For questions or assistance with migration, please refer to the documentation or reach out to the WRT development team.