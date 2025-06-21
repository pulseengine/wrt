//! Component Model types for streaming WebAssembly Component parsing
//!
//! This module provides ASIL-D compliant Component Model type definitions
//! optimized for streaming parsing with fixed memory bounds and TypeRef-based
//! cross-references to avoid Box<T> allocations.

use core::fmt;
use wrt_error::{Error, ErrorCategory, Result, codes};
use wrt_foundation::{NoStdProvider, resource::ResourceRepresentation};
use crate::bounded_types::{SimpleBoundedVec, SimpleBoundedString};
use crate::types::ValueType;

/// Type reference for Component Model recursive types
/// 
/// Uses u32 indices instead of Box<T> to maintain ASIL-D compliance
/// and avoid heap allocations in recursive type definitions.
pub type TypeRef = u32;

/// Hash value for type deduplication
pub type TypeHash = u64;

/// Component Model value type with ASIL-D compliant memory management
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentValueType {
    // Primitive types (direct storage)
    Bool,
    S8,
    U8, 
    S16,
    U16,
    S32,
    U32,
    S64,
    U64,
    F32,
    F64,
    Char,
    String,
    
    // Complex types using TypeRef indices
    Record { 
        fields: SimpleBoundedVec<(SimpleBoundedString<64>, TypeRef), 64> 
    },
    Variant { 
        cases: SimpleBoundedVec<(SimpleBoundedString<64>, Option<TypeRef>), 64> 
    },
    List { 
        element_type: TypeRef 
    },
    FixedList { 
        element_type: TypeRef, 
        length: u32 
    },
    Tuple { 
        elements: SimpleBoundedVec<TypeRef, 32> 
    },
    Flags { 
        names: SimpleBoundedVec<SimpleBoundedString<64>, 64> 
    },
    Enum { 
        variants: SimpleBoundedVec<SimpleBoundedString<64>, 256> 
    },
    Option { 
        inner_type: TypeRef 
    },
    Result { 
        ok_type: Option<TypeRef>, 
        error_type: Option<TypeRef> 
    },
    
    // Resource types
    Own { resource_id: u32 },
    Borrow { resource_id: u32 },
    
    // Reference types
    Ref { type_index: u32 },
}

impl Default for ComponentValueType {
    fn default() -> Self {
        Self::Bool
    }
}

/// Component type definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentType {
    pub definition: ComponentTypeDefinition,
}

/// Component type definition variants
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentTypeDefinition {
    /// Function type with named parameters
    Function {
        params: SimpleBoundedVec<(SimpleBoundedString<64>, TypeRef), 32>,
        results: SimpleBoundedVec<TypeRef, 16>,
    },
    
    /// Component type with imports and exports
    Component {
        imports: SimpleBoundedVec<ComponentImport, 128>,
        exports: SimpleBoundedVec<ComponentExport, 128>,
    },
    
    /// Instance type with exports
    Instance {
        exports: SimpleBoundedVec<ComponentExport, 128>,
    },
    
    /// Value type wrapper
    Value(ComponentValueType),
    
    /// Resource type
    Resource {
        representation: ResourceRepresentation,
        nullable: bool,
    },
    
    /// Module type (core WebAssembly)
    Module {
        imports: SimpleBoundedVec<CoreImport, 128>,
        exports: SimpleBoundedVec<CoreExport, 128>,
    },
}

/// Component import definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentImport {
    pub namespace: SimpleBoundedString<128>,
    pub name: SimpleBoundedString<128>,
    pub ty: ExternType,
}

/// Component export definition  
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentExport {
    pub name: SimpleBoundedString<128>,
    pub ty: ExternType,
}

/// Core WebAssembly import (for module types)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreImport {
    pub module: SimpleBoundedString<128>,
    pub name: SimpleBoundedString<128>,
    pub ty: CoreExternType,
}

/// Core WebAssembly export (for module types)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreExport {
    pub name: SimpleBoundedString<128>,
    pub ty: CoreExternType,
}

/// External type for Component Model
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternType {
    Function {
        params: SimpleBoundedVec<(SimpleBoundedString<64>, TypeRef), 32>,
        results: SimpleBoundedVec<TypeRef, 16>,
    },
    Value(TypeRef),
    Type(TypeRef),
    Instance {
        exports: SimpleBoundedVec<ComponentExport, 64>,
    },
    Component {
        imports: SimpleBoundedVec<ComponentImport, 64>,
        exports: SimpleBoundedVec<ComponentExport, 64>,
    },
}

/// Core external type for module types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreExternType {
    Function {
        params: SimpleBoundedVec<ValueType, 32>,
        results: SimpleBoundedVec<ValueType, 16>,
    },
    Table {
        element_type: ValueType,
        min: u32,
        max: Option<u32>,
    },
    Memory {
        min: u32,
        max: Option<u32>,
        shared: bool,
    },
    Global {
        value_type: ValueType,
        mutable: bool,
    },
}

/// Streaming type intern for Component Model types
/// 
/// Provides type deduplication and memory management for component types
/// with ASIL-D compliant bounded storage.
#[derive(Debug)]
pub struct StreamingTypeIntern {
    /// Storage for type definitions
    types: SimpleBoundedVec<ComponentType, 1024>,
    
    /// Hash-based lookup for deduplication
    type_hashes: SimpleBoundedVec<(TypeHash, TypeRef), 1024>,
    
    /// Reference counters for memory management
    ref_counts: SimpleBoundedVec<u32, 1024>,
    
    /// Next available type reference
    next_ref: TypeRef,
}

impl StreamingTypeIntern {
    /// Create a new streaming type intern
    pub fn new() -> Self {
        Self {
            types: SimpleBoundedVec::new(),
            type_hashes: SimpleBoundedVec::new(),
            ref_counts: SimpleBoundedVec::new(),
            next_ref: 0,
        }
    }
    
    /// Intern a component type and return its TypeRef
    pub fn intern_type(&mut self, component_type: ComponentType) -> Result<TypeRef> {
        // Compute hash for deduplication
        let type_hash = self.compute_type_hash(&component_type);
        
        // Check if type already exists
        if let Some(existing_ref) = self.find_existing_type(type_hash) {
            self.increment_ref_count(existing_ref)?;
            return Ok(existing_ref);
        }
        
        // Check capacity before adding new type
        if self.types.len() >= self.types.capacity() {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::CAPACITY_EXCEEDED,
                "Component type storage capacity exceeded"
            ));
        }
        
        // Store new type
        let type_ref = self.next_ref;
        self.types.push(component_type).map_err(|_| Error::new(
            ErrorCategory::Memory,
            codes::CAPACITY_EXCEEDED,
            "Cannot store component type"
        ))?;
        
        self.type_hashes.push((type_hash, type_ref)).map_err(|_| Error::new(
            ErrorCategory::Memory,
            codes::CAPACITY_EXCEEDED,
            "Cannot store type hash"
        ))?;
        
        self.ref_counts.push(1).map_err(|_| Error::new(
            ErrorCategory::Memory,
            codes::CAPACITY_EXCEEDED,
            "Cannot store reference count"
        ))?;
        
        self.next_ref += 1;
        Ok(type_ref)
    }
    
    /// Get a type by its TypeRef
    pub fn get_type(&self, type_ref: TypeRef) -> Option<&ComponentType> {
        self.types.get(type_ref as usize)
    }
    
    /// Get the number of stored types
    pub fn len(&self) -> usize {
        self.types.len()
    }
    
    /// Check if the intern is empty
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }
    
    /// Find existing type by hash
    fn find_existing_type(&self, hash: TypeHash) -> Option<TypeRef> {
        for (stored_hash, type_ref) in self.type_hashes.iter() {
            if *stored_hash == hash {
                return Some(*type_ref);
            }
        }
        None
    }
    
    /// Increment reference count for a type
    fn increment_ref_count(&mut self, type_ref: TypeRef) -> Result<()> {
        if let Some(count) = self.ref_counts.get_mut(type_ref as usize) {
            *count = count.saturating_add(1);
            Ok(())
        } else {
            Err(Error::new(
                ErrorCategory::Memory,
                codes::INVALID_BINARY,
                "Invalid type reference for ref count increment"
            ))
        }
    }
    
    /// Compute hash for type deduplication
    /// 
    /// Simple hash implementation for type deduplication.
    /// In a full implementation, this would use a proper hash function.
    fn compute_type_hash(&self, component_type: &ComponentType) -> TypeHash {
        // Simplified hash - in practice would use proper hashing
        match &component_type.definition {
            ComponentTypeDefinition::Function { params, results } => {
                let mut hash = 0x1u64;
                hash = hash.wrapping_mul(31).wrapping_add(params.len() as u64);
                hash = hash.wrapping_mul(31).wrapping_add(results.len() as u64);
                hash
            }
            ComponentTypeDefinition::Component { imports, exports } => {
                let mut hash = 0x2u64;
                hash = hash.wrapping_mul(31).wrapping_add(imports.len() as u64);
                hash = hash.wrapping_mul(31).wrapping_add(exports.len() as u64);
                hash
            }
            ComponentTypeDefinition::Instance { exports } => {
                let mut hash = 0x3u64;
                hash = hash.wrapping_mul(31).wrapping_add(exports.len() as u64);
                hash
            }
            ComponentTypeDefinition::Value(_) => 0x4u64,
            ComponentTypeDefinition::Resource { nullable, .. } => {
                if *nullable { 0x5u64 } else { 0x6u64 }
            }
            ComponentTypeDefinition::Module { imports, exports } => {
                let mut hash = 0x7u64;
                hash = hash.wrapping_mul(31).wrapping_add(imports.len() as u64);
                hash = hash.wrapping_mul(31).wrapping_add(exports.len() as u64);
                hash
            }
        }
    }
}

impl Default for StreamingTypeIntern {
    fn default() -> Self {
        Self::new()
    }
}

/// Component memory budget for ASIL-D compliance
/// 
/// Tracks memory usage for Component Model parsing with fixed bounds
/// to ensure deterministic memory usage in safety-critical environments.
#[derive(Debug)]
pub struct ComponentMemoryBudget {
    /// Maximum memory allocated for components (64KB default)
    max_memory: usize,
    
    /// Current memory usage tracking
    current_usage: usize,
    
    /// Reserved memory for critical operations (8KB default)
    reserved_memory: usize,
}

impl ComponentMemoryBudget {
    /// Create new memory budget with default limits
    pub fn new() -> Self {
        Self {
            max_memory: 64 * 1024,  // 64KB total budget
            current_usage: 0,
            reserved_memory: 8 * 1024,  // 8KB reserved
        }
    }
    
    /// Create memory budget with custom limits
    pub fn with_limits(max_memory: usize, reserved_memory: usize) -> Self {
        Self {
            max_memory,
            current_usage: 0,
            reserved_memory,
        }
    }
    
    /// Check if allocation is possible
    pub fn can_allocate(&self, size: usize) -> bool {
        self.current_usage + size + self.reserved_memory <= self.max_memory
    }
    
    /// Allocate memory from budget
    pub fn allocate(&mut self, size: usize) -> Result<()> {
        if !self.can_allocate(size) {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_LIMIT_EXCEEDED,
                "Component memory budget exceeded"
            ));
        }
        
        self.current_usage += size;
        Ok(())
    }
    
    /// Deallocate memory back to budget
    pub fn deallocate(&mut self, size: usize) {
        self.current_usage = self.current_usage.saturating_sub(size);
    }
    
    /// Get current memory usage
    pub fn current_usage(&self) -> usize {
        self.current_usage
    }
    
    /// Get available memory
    pub fn available_memory(&self) -> usize {
        self.max_memory.saturating_sub(self.current_usage + self.reserved_memory)
    }
    
    /// Get total memory budget
    pub fn total_budget(&self) -> usize {
        self.max_memory
    }
}

impl Default for ComponentMemoryBudget {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_streaming_type_intern_basic() {
        let mut intern = StreamingTypeIntern::new();
        
        let func_type = ComponentType {
            definition: ComponentTypeDefinition::Function {
                params: SimpleBoundedVec::new(),
                results: SimpleBoundedVec::new(),
            }
        };
        
        let type_ref = intern.intern_type(func_type.clone()).unwrap();
        assert_eq!(type_ref, 0);
        
        // Same type should return same reference
        let type_ref2 = intern.intern_type(func_type).unwrap();
        assert_eq!(type_ref, type_ref2);
        
        assert_eq!(intern.len(), 1);
    }
    
    #[test]
    fn test_component_memory_budget() {
        let mut budget = ComponentMemoryBudget::new();
        
        assert!(budget.can_allocate(1024));
        assert!(budget.allocate(1024).is_ok());
        assert_eq!(budget.current_usage(), 1024);
        
        // Try to allocate more than available
        let large_size = budget.available_memory() + 1;
        assert!(!budget.can_allocate(large_size));
        assert!(budget.allocate(large_size).is_err());
        
        budget.deallocate(512);
        assert_eq!(budget.current_usage(), 512);
    }
    
    #[test]
    fn test_type_ref_deduplication() {
        let mut intern = StreamingTypeIntern::new();
        
        // Create identical function types
        let func_type1 = ComponentType {
            definition: ComponentTypeDefinition::Function {
                params: SimpleBoundedVec::new(),
                results: SimpleBoundedVec::new(),
            }
        };
        
        let func_type2 = ComponentType {
            definition: ComponentTypeDefinition::Function {
                params: SimpleBoundedVec::new(),
                results: SimpleBoundedVec::new(),
            }
        };
        
        let ref1 = intern.intern_type(func_type1).unwrap();
        let ref2 = intern.intern_type(func_type2).unwrap();
        
        // Should get same reference due to deduplication
        assert_eq!(ref1, ref2);
        assert_eq!(intern.len(), 1);
    }
}