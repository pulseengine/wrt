//! Agent Registry for Managing Execution Agents
//!
//! This module provides a centralized registry for managing different types of execution agents
//! and provides a migration path from the legacy individual agents to the unified agent system.

#[cfg(feature = "std")]
use std::{collections::HashMap, boxed::Box, sync::Arc};
#[cfg(not(feature = "std"))]
use core::marker::PhantomData;

use wrt_foundation::{
    bounded::{BoundedVec, BoundedString},
    prelude::*,
    traits::DefaultMemoryProvider,
};

use crate::{
    unified_execution_agent::{UnifiedExecutionAgent, AgentConfiguration, ExecutionMode, HybridModeFlags},
    execution_engine::ComponentExecutionEngine,
    types::Value,
};

use wrt_foundation::WrtResult;

// Re-export async types when available
#[cfg(feature = "async")]
use crate::async_::AsyncExecutionEngine;

/// Maximum number of registered agents in no_std
const MAX_AGENTS: usize = 32;

/// Agent registry for managing execution agents
pub struct AgentRegistry {
    /// Unified agents (recommended)
    #[cfg(feature = "std")]
    unified_agents: HashMap<AgentId, Box<UnifiedExecutionAgent>>,
    #[cfg(not(feature = "std"))]
    unified_agents: BoundedVec<(AgentId, UnifiedExecutionAgent), MAX_AGENTS, DefaultMemoryProvider>,
    
    /// Legacy agents (deprecated)
    #[cfg(feature = "std")]
    legacy_agents: HashMap<AgentId, Box<dyn LegacyExecutionAgent>>,
    #[cfg(not(feature = "std"))]
    legacy_agents: BoundedVec<(AgentId, LegacyAgentType), 16, DefaultMemoryProvider>,
    
    /// Next agent ID
    next_agent_id: u32,
    
    /// Registry statistics
    stats: RegistryStatistics,
    
    /// Migration tracking
    migration_status: MigrationStatus,
}

/// Unique identifier for agents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentId(pub u32);

/// Registry statistics
#[derive(Debug, Clone, Default)]
pub struct RegistryStatistics {
    /// Total unified agents created
    pub unified_agents_created: u32,
    /// Total legacy agents created  
    pub legacy_agents_created: u32,
    /// Total migrations performed
    pub migrations_performed: u32,
    /// Active agents count
    pub active_agents: u32,
}

/// Migration status tracking
#[derive(Debug, Clone)]
pub struct MigrationStatus {
    /// Agents pending migration
    #[cfg(feature = "std")]
    pub pending_migrations: Vec<AgentId>,
    #[cfg(not(feature = "std"))]
    pub pending_migrations: BoundedVec<AgentId, MAX_AGENTS, DefaultMemoryProvider>,
    
    /// Completed migrations
    pub completed_migrations: u32,
    
    /// Migration warnings
    #[cfg(feature = "std")]
    pub warnings: Vec<MigrationWarning>,
    #[cfg(not(feature = "std"))]
    pub warnings: BoundedVec<MigrationWarning, 16, DefaultMemoryProvider>,
}

/// Migration warning information
#[derive(Debug, Clone)]
pub struct MigrationWarning {
    pub agent_id: AgentId,
    pub warning_type: WarningType,
    pub message: BoundedString<256, DefaultMemoryProvider>,
}

/// Types of migration warnings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningType {
    /// Features not available in unified agent
    FeatureNotSupported,
    /// Performance implications
    PerformanceImpact,
    /// Configuration changes required
    ConfigurationRequired,
    /// API changes
    ApiChanges,
}

/// Legacy agent types for no_std environments
#[cfg(not(feature = "std"))]
#[derive(Debug)]
pub enum LegacyAgentType {
    Component(ComponentExecutionEngine),
    #[cfg(feature = "async")]
    Async(AsyncExecutionEngine),
    // Note: Stackless and CFI engines are not included as they're integrated into unified agent
}

/// Trait for legacy execution agents (std only)
#[cfg(feature = "std")]
pub trait LegacyExecutionAgent: Send + Sync {
    /// Execute a function call
    fn call_function(&mut self, instance_id: u32, function_index: u32, args: &[Value]) -> WrtResult<Value>;
    
    /// Get agent type name
    fn agent_type(&self) -> &'static str;
    
    /// Check if agent can be migrated
    fn can_migrate(&self) -> bool;
    
    /// Get migration configuration
    fn migration_config(&self) -> AgentConfiguration;
}

/// Agent creation options
#[derive(Debug, Clone)]
pub struct AgentCreationOptions {
    /// Preferred agent type
    pub agent_type: PreferredAgentType,
    /// Configuration for the agent
    pub config: AgentConfiguration,
    /// Whether to use legacy agent if unified not available
    pub allow_legacy_fallback: bool,
}

/// Preferred agent type for creation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferredAgentType {
    /// Use unified agent (recommended)
    Unified,
    /// Use legacy component agent (deprecated)
    LegacyComponent,
    /// Use legacy async agent (deprecated)
    #[cfg(feature = "async")]
    LegacyAsync,
    /// Auto-select best available
    Auto,
}

impl AgentRegistry {
    /// Create a new agent registry
    pub fn new() -> Self {
        let provider = DefaultMemoryProvider::default();
        
        Self {
            #[cfg(feature = "std")]
            unified_agents: HashMap::new(),
            #[cfg(not(feature = "std"))]
            unified_agents: BoundedVec::new(provider.clone()).unwrap(),
            
            #[cfg(feature = "std")]
            legacy_agents: HashMap::new(),
            #[cfg(not(feature = "std"))]
            legacy_agents: BoundedVec::new(provider.clone()).unwrap(),
            
            next_agent_id: 1,
            stats: RegistryStatistics::default(),
            migration_status: MigrationStatus {
                #[cfg(feature = "std")]
                pending_migrations: Vec::new(),
                #[cfg(not(feature = "std"))]
                pending_migrations: BoundedVec::new(provider.clone()).unwrap(),
                completed_migrations: 0,
                #[cfg(feature = "std")]
                warnings: Vec::new(),
                #[cfg(not(feature = "std"))]
                warnings: BoundedVec::new(provider).unwrap(),
            },
        }
    }

    /// Create a new unified execution agent (recommended)
    pub fn create_unified_agent(&mut self, config: AgentConfiguration) -> WrtResult<AgentId> {
        let agent_id = AgentId(self.next_agent_id);
        self.next_agent_id += 1;

        let agent = UnifiedExecutionAgent::new(config);

        #[cfg(feature = "std")]
        {
            self.unified_agents.insert(agent_id, Box::new(agent));
        }
        #[cfg(not(feature = "std"))]
        {
            self.unified_agents.push((agent_id, agent)).map_err(|_| {
                wrt_foundation::WrtError::ResourceExhausted("Too many agents".into())
            })?;
        }

        self.stats.unified_agents_created += 1;
        self.stats.active_agents += 1;

        Ok(agent_id)
    }

    /// Create an agent with options
    pub fn create_agent(&mut self, options: AgentCreationOptions) -> WrtResult<AgentId> {
        match options.agent_type {
            PreferredAgentType::Unified => {
                self.create_unified_agent(options.config)
            }
            PreferredAgentType::LegacyComponent => {
                if options.allow_legacy_fallback {
                    self.create_legacy_component_agent()
                } else {
                    Err(wrt_foundation::WrtError::invalid_input("Invalid input")))
                }
            }
            #[cfg(feature = "async")]
            PreferredAgentType::LegacyAsync => {
                if options.allow_legacy_fallback {
                    self.create_legacy_async_agent()
                } else {
                    Err(wrt_foundation::WrtError::invalid_input("Invalid input")))
                }
            }
            PreferredAgentType::Auto => {
                // Always prefer unified agent
                self.create_unified_agent(options.config)
            }
        }
    }

    /// Create a legacy component agent (deprecated)
    pub fn create_legacy_component_agent(&mut self) -> WrtResult<AgentId> {
        let agent_id = AgentId(self.next_agent_id);
        self.next_agent_id += 1;

        let agent = ComponentExecutionEngine::new();

        #[cfg(feature = "std")]
        {
            self.legacy_agents.insert(agent_id, Box::new(agent));
        }
        #[cfg(not(feature = "std"))]
        {
            self.legacy_agents.push((agent_id, LegacyAgentType::Component(agent))).map_err(|_| {
                wrt_foundation::WrtError::ResourceExhausted("Too many legacy agents".into())
            })?;
        }

        self.stats.legacy_agents_created += 1;
        self.stats.active_agents += 1;

        // Add to pending migrations
        self.add_pending_migration(agent_id);

        Ok(agent_id)
    }

    /// Create a legacy async agent (deprecated)
    #[cfg(feature = "async")]
    pub fn create_legacy_async_agent(&mut self) -> WrtResult<AgentId> {
        let agent_id = AgentId(self.next_agent_id);
        self.next_agent_id += 1;

        let agent = AsyncExecutionEngine::new();

        #[cfg(feature = "std")]
        {
            self.legacy_agents.insert(agent_id, Box::new(agent));
        }
        #[cfg(not(feature = "std"))]
        {
            self.legacy_agents.push((agent_id, LegacyAgentType::Async(agent))).map_err(|_| {
                wrt_foundation::WrtError::ResourceExhausted("Too many legacy agents".into())
            })?;
        }

        self.stats.legacy_agents_created += 1;
        self.stats.active_agents += 1;

        // Add to pending migrations
        self.add_pending_migration(agent_id);

        Ok(agent_id)
    }

    /// Execute a function call on an agent
    pub fn call_function(
        &mut self, 
        agent_id: AgentId, 
        instance_id: u32, 
        function_index: u32, 
        args: &[Value]
    ) -> WrtResult<Value> {
        // Try unified agents first
        #[cfg(feature = "std")]
        {
            if let Some(agent) = self.unified_agents.get_mut(&agent_id) {
                return agent.call_function(instance_id, function_index, args);
            }
        }
        #[cfg(not(feature = "std"))]
        {
            for (id, agent) in &mut self.unified_agents {
                if *id == agent_id {
                    return agent.call_function(instance_id, function_index, args);
                }
            }
        }

        // Fallback to legacy agents
        #[cfg(feature = "std")]
        {
            if let Some(agent) = self.legacy_agents.get_mut(&agent_id) {
                return agent.call_function(instance_id, function_index, args);
            }
        }
        #[cfg(not(feature = "std"))]
        {
            for (id, agent) in &mut self.legacy_agents {
                if *id == agent_id {
                    return match agent {
                        LegacyAgentType::Component(engine) => {
                            engine.call_function(instance_id, function_index, args)
                        }
                        #[cfg(feature = "async")]
                        LegacyAgentType::Async(_engine) => {
                            // Async execution would require different API
                            Err(wrt_foundation::WrtError::InvalidOperation("Async agent requires different API".into()))
                        }
                    };
                }
            }
        }

        Err(wrt_foundation::WrtError::invalid_input("Invalid input")))
    }

    /// Migrate a legacy agent to unified agent
    pub fn migrate_agent(&mut self, agent_id: AgentId) -> WrtResult<()> {
        // Check if agent exists and is legacy
        #[cfg(feature = "std")]
        let migration_config = {
            if let Some(agent) = self.legacy_agents.get(&agent_id) {
                if !agent.can_migrate() {
                    return Err(wrt_foundation::WrtError::InvalidOperation("Agent cannot be migrated".into()));
                }
                agent.migration_config()
            } else {
                return Err(wrt_foundation::WrtError::invalid_input("Invalid input")));
            }
        };

        #[cfg(not(feature = "std"))]
        let migration_config = {
            let mut found = false;
            let mut config = AgentConfiguration::default();
            
            for (id, agent) in &self.legacy_agents {
                if *id == agent_id {
                    found = true;
                    config = match agent {
                        LegacyAgentType::Component(_) => AgentConfiguration {
                            execution_mode: ExecutionMode::Synchronous,
                            ..AgentConfiguration::default()
                        },
                        #[cfg(feature = "async")]
                        LegacyAgentType::Async(_) => AgentConfiguration {
                            execution_mode: ExecutionMode::Asynchronous,
                            ..AgentConfiguration::default()
                        },
                    };
                    break;
                }
            }
            
            if !found {
                return Err(wrt_foundation::WrtError::invalid_input("Invalid input")));
            }
            config
        };

        // Create new unified agent
        let unified_agent = UnifiedExecutionAgent::new(migration_config);

        // Replace legacy agent with unified agent
        #[cfg(feature = "std")]
        {
            self.legacy_agents.remove(&agent_id);
            self.unified_agents.insert(agent_id, Box::new(unified_agent));
        }
        #[cfg(not(feature = "std"))]
        {
            // Remove from legacy agents
            self.legacy_agents.retain(|(id, _)| *id != agent_id);
            // Add to unified agents
            self.unified_agents.push((agent_id, unified_agent)).map_err(|_| {
                wrt_foundation::WrtError::ResourceExhausted("Too many unified agents".into())
            })?;
        }

        // Update migration tracking
        self.remove_pending_migration(agent_id);
        self.migration_status.completed_migrations += 1;

        Ok(())
    }

    /// Get agent information
    pub fn get_agent_info(&self, agent_id: AgentId) -> Option<AgentInfo> {
        // Check unified agents
        #[cfg(feature = "std")]
        {
            if self.unified_agents.contains_key(&agent_id) {
                return Some(AgentInfo {
                    agent_id,
                    agent_type: AgentType::Unified,
                    migration_status: AgentMigrationStatus::NotRequired,
                });
            }
        }
        #[cfg(not(feature = "std"))]
        {
            for (id, _) in &self.unified_agents {
                if *id == agent_id {
                    return Some(AgentInfo {
                        agent_id,
                        agent_type: AgentType::Unified,
                        migration_status: AgentMigrationStatus::NotRequired,
                    });
                }
            }
        }

        // Check legacy agents
        #[cfg(feature = "std")]
        {
            if self.legacy_agents.contains_key(&agent_id) {
                return Some(AgentInfo {
                    agent_id,
                    agent_type: AgentType::Legacy,
                    migration_status: if self.is_pending_migration(agent_id) {
                        AgentMigrationStatus::Pending
                    } else {
                        AgentMigrationStatus::Available
                    },
                });
            }
        }
        #[cfg(not(feature = "std"))]
        {
            for (id, _) in &self.legacy_agents {
                if *id == agent_id {
                    return Some(AgentInfo {
                        agent_id,
                        agent_type: AgentType::Legacy,
                        migration_status: if self.is_pending_migration(agent_id) {
                            AgentMigrationStatus::Pending
                        } else {
                            AgentMigrationStatus::Available
                        },
                    });
                }
            }
        }

        None
    }

    /// Remove an agent from the registry
    pub fn remove_agent(&mut self, agent_id: AgentId) -> WrtResult<()> {
        let mut removed = false;

        // Try unified agents
        #[cfg(feature = "std")]
        {
            if self.unified_agents.remove(&agent_id).is_some() {
                removed = true;
            }
        }
        #[cfg(not(feature = "std"))]
        {
            let original_len = self.unified_agents.len();
            self.unified_agents.retain(|(id, _)| *id != agent_id);
            if self.unified_agents.len() < original_len {
                removed = true;
            }
        }

        // Try legacy agents
        #[cfg(feature = "std")]
        {
            if self.legacy_agents.remove(&agent_id).is_some() {
                removed = true;
                self.remove_pending_migration(agent_id);
            }
        }
        #[cfg(not(feature = "std"))]
        {
            let original_len = self.legacy_agents.len();
            self.legacy_agents.retain(|(id, _)| *id != agent_id);
            if self.legacy_agents.len() < original_len {
                removed = true;
                self.remove_pending_migration(agent_id);
            }
        }

        if removed {
            self.stats.active_agents = self.stats.active_agents.saturating_sub(1);
            Ok(())
        } else {
            Err(wrt_foundation::WrtError::invalid_input("Invalid input")))
        }
    }

    /// Get registry statistics
    pub fn statistics(&self) -> &RegistryStatistics {
        &self.stats
    }

    /// Get migration status
    pub fn migration_status(&self) -> &MigrationStatus {
        &self.migration_status
    }

    /// Migrate all eligible legacy agents
    pub fn migrate_all_agents(&mut self) -> WrtResult<u32> {
        let mut migrated_count = 0;
        
        // Get list of legacy agent IDs to avoid borrow conflicts
        #[cfg(feature = "std")]
        let legacy_ids: Vec<AgentId> = self.legacy_agents.keys().copied().collect();
        #[cfg(not(feature = "std"))]
        let legacy_ids: BoundedVec<AgentId, MAX_AGENTS, DefaultMemoryProvider> = {
            let mut ids = BoundedVec::new(DefaultMemoryProvider::default()).unwrap();
            for (id, _) in &self.legacy_agents {
                let _ = ids.push(*id);
            }
            ids
        };

        for agent_id in legacy_ids {
            if self.migrate_agent(agent_id).is_ok() {
                migrated_count += 1;
            }
        }

        Ok(migrated_count)
    }

    // Private helper methods

    fn add_pending_migration(&mut self, agent_id: AgentId) {
        let _ = self.migration_status.pending_migrations.push(agent_id);
    }

    fn remove_pending_migration(&mut self, agent_id: AgentId) {
        self.migration_status.pending_migrations.retain(|id| *id != agent_id);
    }

    fn is_pending_migration(&self, agent_id: AgentId) -> bool {
        self.migration_status.pending_migrations.iter().any(|id| *id == agent_id)
    }
}

/// Agent information
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub agent_id: AgentId,
    pub agent_type: AgentType,
    pub migration_status: AgentMigrationStatus,
}

/// Agent type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    /// Unified execution agent
    Unified,
    /// Legacy execution agent
    Legacy,
}

/// Agent migration status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentMigrationStatus {
    /// Migration not required (already unified)
    NotRequired,
    /// Migration available
    Available,
    /// Migration pending
    Pending,
    /// Migration completed
    Completed,
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AgentCreationOptions {
    fn default() -> Self {
        Self {
            agent_type: PreferredAgentType::Unified,
            config: AgentConfiguration::default(),
            allow_legacy_fallback: false,
        }
    }
}

// Implement LegacyExecutionAgent for ComponentExecutionEngine
#[cfg(feature = "std")]
impl LegacyExecutionAgent for ComponentExecutionEngine {
    fn call_function(&mut self, instance_id: u32, function_index: u32, args: &[Value]) -> WrtResult<Value> {
        ComponentExecutionEngine::call_function(self, instance_id, function_index, args)
    }

    fn agent_type(&self) -> &'static str {
        "ComponentExecutionEngine"
    }

    fn can_migrate(&self) -> bool {
        true
    }

    fn migration_config(&self) -> AgentConfiguration {
        AgentConfiguration {
            execution_mode: ExecutionMode::Synchronous,
            ..AgentConfiguration::default()
        }
    }
}

// Implement LegacyExecutionAgent for AsyncExecutionEngine
#[cfg(all(feature = "std", feature = "async"))]
impl LegacyExecutionAgent for AsyncExecutionEngine {
    fn call_function(&mut self, _instance_id: u32, _function_index: u32, _args: &[Value]) -> WrtResult<Value> {
        // Async engines need different API - this is just a placeholder
        Err(wrt_foundation::WrtError::InvalidOperation("Async agent requires different API".into()))
    }

    fn agent_type(&self) -> &'static str {
        "AsyncExecutionEngine"
    }

    fn can_migrate(&self) -> bool {
        true
    }

    fn migration_config(&self) -> AgentConfiguration {
        AgentConfiguration {
            execution_mode: ExecutionMode::Asynchronous,
            ..AgentConfiguration::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = AgentRegistry::new();
        assert_eq!(registry.stats.active_agents, 0);
        assert_eq!(registry.stats.unified_agents_created, 0);
        assert_eq!(registry.stats.legacy_agents_created, 0);
    }

    #[test]
    fn test_unified_agent_creation() {
        let mut registry = AgentRegistry::new();
        let config = AgentConfiguration::default();
        
        let agent_id = registry.create_unified_agent(config).unwrap();
        assert_eq!(agent_id.0, 1);
        assert_eq!(registry.stats.unified_agents_created, 1);
        assert_eq!(registry.stats.active_agents, 1);
    }

    #[test]
    fn test_legacy_agent_creation() {
        let mut registry = AgentRegistry::new();
        
        let agent_id = registry.create_legacy_component_agent().unwrap();
        assert_eq!(agent_id.0, 1);
        assert_eq!(registry.stats.legacy_agents_created, 1);
        assert_eq!(registry.stats.active_agents, 1);
        
        // Should be added to pending migrations
        assert!(registry.is_pending_migration(agent_id));
    }

    #[test]
    fn test_agent_migration() {
        let mut registry = AgentRegistry::new();
        
        // Create legacy agent
        let agent_id = registry.create_legacy_component_agent().unwrap();
        assert!(registry.is_pending_migration(agent_id));
        
        // Migrate to unified
        registry.migrate_agent(agent_id).unwrap();
        assert!(!registry.is_pending_migration(agent_id));
        assert_eq!(registry.migration_status.completed_migrations, 1);
        
        // Should now be a unified agent
        let info = registry.get_agent_info(agent_id).unwrap();
        assert_eq!(info.agent_type, AgentType::Unified);
        assert_eq!(info.migration_status, AgentMigrationStatus::NotRequired);
    }

    #[test]
    fn test_agent_creation_options() {
        let mut registry = AgentRegistry::new();
        
        let options = AgentCreationOptions {
            agent_type: PreferredAgentType::Unified,
            config: AgentConfiguration::default(),
            allow_legacy_fallback: false,
        };
        
        let agent_id = registry.create_agent(options).unwrap();
        let info = registry.get_agent_info(agent_id).unwrap();
        assert_eq!(info.agent_type, AgentType::Unified);
    }

    #[test]
    fn test_function_execution() {
        let mut registry = AgentRegistry::new();
        let config = AgentConfiguration::default();
        
        let agent_id = registry.create_unified_agent(config).unwrap();
        let args = [Value::U32(42), Value::Bool(true)];
        
        let result = registry.call_function(agent_id, 1, 2, &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_agent_removal() {
        let mut registry = AgentRegistry::new();
        let config = AgentConfiguration::default();
        
        let agent_id = registry.create_unified_agent(config).unwrap();
        assert_eq!(registry.stats.active_agents, 1);
        
        registry.remove_agent(agent_id).unwrap();
        assert_eq!(registry.stats.active_agents, 0);
        
        let info = registry.get_agent_info(agent_id);
        assert!(info.is_none());
    }
}