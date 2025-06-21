//! Component Model registry for streaming WebAssembly Component parsing
//!
//! This module provides a three-tier streaming registry for Component Model types
//! with ASIL-D compliant memory management and TypeRef-based cross-references.

use core::fmt;
use wrt_error::{Error, ErrorCategory, Result, codes};
use crate::bounded_types::{SimpleBoundedVec, SimpleBoundedString, SimpleBoundedBytes};
use crate::component_types::{
    ComponentType, ComponentValueType, TypeRef, TypeHash, 
    StreamingTypeIntern, ComponentMemoryBudget
};

/// Three-tier Component Model registry for streaming parsing
/// 
/// Provides hot/warm/cold storage tiers for optimal memory usage and
/// access patterns during streaming Component Model parsing.
#[derive(Debug)]
pub struct ComponentRegistry {
    /// Tier 1: Hot storage for frequently accessed types
    hot_storage: HotTypeStorage,
    
    /// Tier 2: Index registry for TypeRef management
    index_registry: TypeIndexRegistry,
    
    /// Tier 3: Cold storage for serialized type data
    cold_storage: ColdTypeStorage,
    
    /// Memory budget enforcement
    memory_budget: ComponentMemoryBudget,
    
    /// Type intern for deduplication
    type_intern: StreamingTypeIntern,
    
    /// Current parsing state
    parser_state: ComponentParserState,
}

/// Hot storage tier for frequently accessed types
#[derive(Debug)]
struct HotTypeStorage {
    /// Direct storage for primitive types
    primitives: SimpleBoundedVec<ComponentValueType, 256>,
    
    /// Recently accessed complex types (LRU-style)
    hot_types: SimpleBoundedVec<(TypeRef, ComponentType), 128>,
    
    /// Access frequency tracking
    access_counts: SimpleBoundedVec<(TypeRef, u32), 128>,
}

/// Type index registry for reference management
#[derive(Debug)]
struct TypeIndexRegistry {
    /// Maps external TypeRef to internal index
    ref_to_index: SimpleBoundedVec<(TypeRef, u32), 1024>,
    
    /// Reverse mapping for resolution
    index_to_ref: SimpleBoundedVec<TypeRef, 1024>,
    
    /// Type metadata for quick identification
    type_metadata: SimpleBoundedVec<TypeMetadata, 1024>,
}

/// Cold storage for types not immediately needed
#[derive(Debug)]
struct ColdTypeStorage {
    /// Serialized type data for later reconstruction
    serialized_types: SimpleBoundedVec<SimpleBoundedBytes<512>, 256>,
    
    /// Quick lookup metadata
    cold_metadata: SimpleBoundedVec<ColdTypeMetadata, 256>,
}

/// Type metadata for quick identification
#[derive(Debug, Clone)]
struct TypeMetadata {
    type_ref: TypeRef,
    type_kind: TypeKind,
    size_estimate: u32,
    dependencies: SimpleBoundedVec<TypeRef, 16>,
}

/// Cold storage metadata
#[derive(Debug, Clone)]
struct ColdTypeMetadata {
    type_ref: TypeRef,
    serialized_index: u32,
    size: u32,
    last_access: u32,
}

/// Type kind classification for fast categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeKind {
    Primitive,
    Function,
    Record,
    Variant, 
    List,
    Resource,
    Component,
    Instance,
    Module,
}

/// Component parser state tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentParserState {
    /// Parsing core WebAssembly sections
    Core,
    /// Parsing Component Model sections
    Component,
    /// Parsing nested components
    NestedComponent { depth: u8 },
    /// Parsing type definitions
    Types,
    /// Parsing imports/exports
    Interface,
    /// Completed parsing
    Complete,
}

impl ComponentRegistry {
    /// Create a new component registry with default settings
    pub fn new() -> Self {
        Self {
            hot_storage: HotTypeStorage::new(),
            index_registry: TypeIndexRegistry::new(),
            cold_storage: ColdTypeStorage::new(),
            memory_budget: ComponentMemoryBudget::new(),
            type_intern: StreamingTypeIntern::new(),
            parser_state: ComponentParserState::Core,
        }
    }
    
    /// Create registry with custom memory budget
    pub fn with_memory_budget(budget: ComponentMemoryBudget) -> Self {
        Self {
            hot_storage: HotTypeStorage::new(),
            index_registry: TypeIndexRegistry::new(),
            cold_storage: ColdTypeStorage::new(),
            memory_budget: budget,
            type_intern: StreamingTypeIntern::new(),
            parser_state: ComponentParserState::Core,
        }
    }
    
    /// Register a new component type
    pub fn register_type(&mut self, component_type: ComponentType) -> Result<TypeRef> {
        // Check memory budget before allocation
        let estimated_size = self.estimate_type_size(&component_type);
        self.memory_budget.allocate(estimated_size)?;
        
        // Intern the type for deduplication
        let type_ref = self.type_intern.intern_type(component_type.clone())?;
        
        // Classify type for storage tier selection
        let type_kind = self.classify_type(&component_type);
        
        // Store in appropriate tier
        match type_kind {
            TypeKind::Primitive => {
                self.hot_storage.store_primitive(type_ref, &component_type)?;
            }
            TypeKind::Function | TypeKind::Record | TypeKind::Variant => {
                self.hot_storage.store_hot_type(type_ref, component_type)?;
            }
            _ => {
                self.cold_storage.store_cold_type(type_ref, &component_type)?;
            }
        }
        
        // Register in index
        self.index_registry.register_type_ref(type_ref, type_kind)?;
        
        Ok(type_ref)
    }
    
    /// Retrieve a type by its TypeRef
    pub fn get_type(&mut self, type_ref: TypeRef) -> Result<Option<ComponentType>> {
        // Try hot storage first
        if self.hot_storage.has_type(type_ref) {
            self.hot_storage.record_access(type_ref);
            if let Some(component_type) = self.hot_storage.get_type(type_ref) {
                return Ok(Some(component_type.clone()));
            }
        }
        
        // Try type intern
        if let Some(component_type) = self.type_intern.get_type(type_ref) {
            return Ok(Some(component_type.clone()));
        }
        
        // Try cold storage (with potential promotion to hot)
        if self.cold_storage.has_type(type_ref) {
            if let Some(component_type) = self.promote_from_cold(type_ref)? {
                return Ok(Some(component_type));
            }
        }
        
        Ok(None)
    }
    
    /// Get current parser state
    pub fn parser_state(&self) -> ComponentParserState {
        self.parser_state
    }
    
    /// Update parser state
    pub fn set_parser_state(&mut self, state: ComponentParserState) {
        self.parser_state = state;
    }
    
    /// Get memory budget status
    pub fn memory_usage(&self) -> (usize, usize) {
        (self.memory_budget.current_usage(), self.memory_budget.total_budget())
    }
    
    /// Get number of registered types
    pub fn type_count(&self) -> usize {
        self.type_intern.len()
    }
    
    /// Promote type from cold to hot storage
    fn promote_from_cold(&mut self, type_ref: TypeRef) -> Result<Option<ComponentType>> {
        if let Some(component_type) = self.cold_storage.retrieve_type(type_ref)? {
            self.hot_storage.store_hot_type(type_ref, component_type.clone())?;
            self.cold_storage.remove_type(type_ref);
            Ok(Some(component_type))
        } else {
            Ok(None)
        }
    }
    
    /// Estimate memory size for a type
    fn estimate_type_size(&self, component_type: &ComponentType) -> usize {
        // Simplified size estimation
        match &component_type.definition {
            crate::component_types::ComponentTypeDefinition::Function { params, results } => {
                64 + params.len() * 32 + results.len() * 8
            }
            crate::component_types::ComponentTypeDefinition::Component { imports, exports } => {
                128 + imports.len() * 64 + exports.len() * 64
            }
            crate::component_types::ComponentTypeDefinition::Instance { exports } => {
                64 + exports.len() * 64
            }
            crate::component_types::ComponentTypeDefinition::Value(_) => 32,
            crate::component_types::ComponentTypeDefinition::Resource { .. } => 48,
            crate::component_types::ComponentTypeDefinition::Module { imports, exports } => {
                96 + imports.len() * 48 + exports.len() * 48
            }
        }
    }
    
    /// Classify type for storage tier selection
    fn classify_type(&self, component_type: &ComponentType) -> TypeKind {
        match &component_type.definition {
            crate::component_types::ComponentTypeDefinition::Function { .. } => TypeKind::Function,
            crate::component_types::ComponentTypeDefinition::Component { .. } => TypeKind::Component,
            crate::component_types::ComponentTypeDefinition::Instance { .. } => TypeKind::Instance,
            crate::component_types::ComponentTypeDefinition::Value(val_type) => {
                match val_type {
                    ComponentValueType::Bool | ComponentValueType::S8 | ComponentValueType::U8 |
                    ComponentValueType::S16 | ComponentValueType::U16 | ComponentValueType::S32 |
                    ComponentValueType::U32 | ComponentValueType::S64 | ComponentValueType::U64 |
                    ComponentValueType::F32 | ComponentValueType::F64 | ComponentValueType::Char |
                    ComponentValueType::String => TypeKind::Primitive,
                    ComponentValueType::Record { .. } => TypeKind::Record,
                    ComponentValueType::Variant { .. } => TypeKind::Variant,
                    ComponentValueType::List { .. } => TypeKind::List,
                    ComponentValueType::Own { .. } | ComponentValueType::Borrow { .. } => TypeKind::Resource,
                    _ => TypeKind::Primitive,
                }
            }
            crate::component_types::ComponentTypeDefinition::Resource { .. } => TypeKind::Resource,
            crate::component_types::ComponentTypeDefinition::Module { .. } => TypeKind::Module,
        }
    }
}

impl HotTypeStorage {
    fn new() -> Self {
        Self {
            primitives: SimpleBoundedVec::new(),
            hot_types: SimpleBoundedVec::new(),
            access_counts: SimpleBoundedVec::new(),
        }
    }
    
    fn store_primitive(&mut self, _type_ref: TypeRef, _component_type: &ComponentType) -> Result<()> {
        // Store primitive type classification
        Ok(())
    }
    
    fn store_hot_type(&mut self, type_ref: TypeRef, component_type: ComponentType) -> Result<()> {
        self.hot_types.push((type_ref, component_type)).map_err(|_| Error::new(
            ErrorCategory::Memory,
            codes::CAPACITY_EXCEEDED,
            "Hot type storage capacity exceeded"
        ))?;
        Ok(())
    }
    
    fn has_type(&self, type_ref: TypeRef) -> bool {
        self.hot_types.iter().any(|(stored_ref, _)| *stored_ref == type_ref)
    }
    
    fn get_type(&self, type_ref: TypeRef) -> Option<&ComponentType> {
        for (stored_ref, component_type) in self.hot_types.iter() {
            if *stored_ref == type_ref {
                return Some(component_type);
            }
        }
        None
    }
    
    fn record_access(&mut self, type_ref: TypeRef) {
        // Update access count for LRU management
        for (stored_ref, count) in self.access_counts.iter_mut() {
            if *stored_ref == type_ref {
                *count = count.saturating_add(1);
                return;
            }
        }
        
        // Add new access record if space available
        let _ = self.access_counts.push((type_ref, 1));
    }
}

impl TypeIndexRegistry {
    fn new() -> Self {
        Self {
            ref_to_index: SimpleBoundedVec::new(),
            index_to_ref: SimpleBoundedVec::new(),
            type_metadata: SimpleBoundedVec::new(),
        }
    }
    
    fn register_type_ref(&mut self, type_ref: TypeRef, type_kind: TypeKind) -> Result<()> {
        let index = self.index_to_ref.len() as u32;
        
        self.ref_to_index.push((type_ref, index)).map_err(|_| Error::new(
            ErrorCategory::Memory,
            codes::CAPACITY_EXCEEDED,
            "Type reference registry capacity exceeded"
        ))?;
        
        self.index_to_ref.push(type_ref).map_err(|_| Error::new(
            ErrorCategory::Memory,
            codes::CAPACITY_EXCEEDED,
            "Type index registry capacity exceeded"
        ))?;
        
        let metadata = TypeMetadata {
            type_ref,
            type_kind,
            size_estimate: 64, // Default estimate
            dependencies: SimpleBoundedVec::new(),
        };
        
        self.type_metadata.push(metadata).map_err(|_| Error::new(
            ErrorCategory::Memory,
            codes::CAPACITY_EXCEEDED,
            "Type metadata registry capacity exceeded"
        ))?;
        
        Ok(())
    }
}

impl ColdTypeStorage {
    fn new() -> Self {
        Self {
            serialized_types: SimpleBoundedVec::new(),
            cold_metadata: SimpleBoundedVec::new(),
        }
    }
    
    fn store_cold_type(&mut self, type_ref: TypeRef, _component_type: &ComponentType) -> Result<()> {
        // Serialize type for cold storage (simplified)
        let serialized = SimpleBoundedVec::new();
        let index = self.serialized_types.len() as u32;
        
        self.serialized_types.push(serialized).map_err(|_| Error::new(
            ErrorCategory::Memory,
            codes::CAPACITY_EXCEEDED,
            "Cold storage capacity exceeded"
        ))?;
        
        let metadata = ColdTypeMetadata {
            type_ref,
            serialized_index: index,
            size: 64, // Placeholder size
            last_access: 0,
        };
        
        self.cold_metadata.push(metadata).map_err(|_| Error::new(
            ErrorCategory::Memory,
            codes::CAPACITY_EXCEEDED,
            "Cold metadata capacity exceeded"
        ))?;
        
        Ok(())
    }
    
    fn has_type(&self, type_ref: TypeRef) -> bool {
        self.cold_metadata.iter().any(|meta| meta.type_ref == type_ref)
    }
    
    fn retrieve_type(&mut self, _type_ref: TypeRef) -> Result<Option<ComponentType>> {
        // Deserialize type from cold storage (simplified)
        // In full implementation, would deserialize from serialized_types
        Ok(None)
    }
    
    fn remove_type(&mut self, type_ref: TypeRef) {
        // Remove type from cold storage (simplified)
        // In full implementation, would compact storage
        self.cold_metadata.retain(|meta| meta.type_ref != type_ref);
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ComponentParserState {
    fn default() -> Self {
        Self::Core
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component_types::{ComponentTypeDefinition, ComponentValueType};
    
    #[test]
    fn test_component_registry_basic() {
        let mut registry = ComponentRegistry::new();
        
        let func_type = ComponentType {
            definition: ComponentTypeDefinition::Function {
                params: SimpleBoundedVec::new(),
                results: SimpleBoundedVec::new(),
            }
        };
        
        let type_ref = registry.register_type(func_type).unwrap();
        assert_eq!(type_ref, 0);
        assert_eq!(registry.type_count(), 1);
    }
    
    #[test]
    fn test_memory_budget_enforcement() {
        let small_budget = ComponentMemoryBudget::with_limits(1024, 256);
        let mut registry = ComponentRegistry::with_memory_budget(small_budget);
        
        // Create a large component type that should exceed budget
        let large_component = ComponentType {
            definition: ComponentTypeDefinition::Component {
                imports: SimpleBoundedVec::new(),
                exports: SimpleBoundedVec::new(),
            }
        };
        
        // First registration should succeed
        assert!(registry.register_type(large_component.clone()).is_ok());
        
        // Eventually should hit memory budget limits with enough registrations
        let mut registrations = 0;
        while registrations < 100 {
            if registry.register_type(large_component.clone()).is_err() {
                break;
            }
            registrations += 1;
        }
        
        // Should have hit budget limit before 100 registrations
        assert!(registrations < 100);
    }
    
    #[test]
    fn test_parser_state_management() {
        let mut registry = ComponentRegistry::new();
        
        assert_eq!(registry.parser_state(), ComponentParserState::Core);
        
        registry.set_parser_state(ComponentParserState::Component);
        assert_eq!(registry.parser_state(), ComponentParserState::Component);
        
        registry.set_parser_state(ComponentParserState::NestedComponent { depth: 2 });
        assert_eq!(registry.parser_state(), ComponentParserState::NestedComponent { depth: 2 });
    }
}