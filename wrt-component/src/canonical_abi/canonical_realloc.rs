//! Canonical ABI realloc function handling
//!
//! This module implements the realloc function support for the WebAssembly
//! Component Model's Canonical ABI, enabling dynamic memory allocation
//! during lifting and lowering operations.

#[cfg(not(feature = "std"))]
use std::sync::{Arc, Mutex};
#[cfg(feature = "std")]
use std::sync::{Arc, Mutex};

use wrt_foundation::{
    bounded_collections::{BoundedVec, MAX_GENERATIVE_TYPES},
    prelude::*,
};

use crate::{
    memory_layout::{Alignment, MemoryLayout},
    types::{ComponentError, ComponentInstanceId},
};

/// Binary std/no_std choice
pub type ReallocFn = fn(i32, i32, i32, i32) -> i32;

/// Binary std/no_std choice
#[derive(Debug, Clone)]
pub struct CanonicalOptionsWithRealloc {
    /// Memory for canonical operations
    pub memory: u32,
    /// Binary std/no_std choice
    pub realloc: Option<u32>,
    /// Post-return function index
    pub post_return: Option<u32>,
    /// String encoding
    pub string_encoding: StringEncoding,
    /// Component instance ID
    pub instance_id: ComponentInstanceId,
}

/// String encoding options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringEncoding {
    Utf8,
    Utf16Le,
    Utf16Be,
    Latin1,
}

/// Binary std/no_std choice
#[derive(Debug)]
pub struct ReallocManager {
    /// Binary std/no_std choice
    allocations: BTreeMap<ComponentInstanceId, InstanceAllocations>,
    /// Binary std/no_std choice
    metrics: AllocationMetrics,
    /// Binary std/no_std choice
    max_allocation_size: usize,
    /// Binary std/no_std choice
    max_instance_allocations: usize,
}

#[derive(Debug)]
struct InstanceAllocations {
    /// Binary std/no_std choice
    allocations: BoundedVec<Allocation, MAX_GENERATIVE_TYPES>,
    /// Binary std/no_std choice
    total_bytes: usize,
    /// Binary std/no_std choice
    realloc_fn: Option<ReallocFunction>,
}

#[derive(Debug, Clone)]
struct Allocation {
    /// Binary std/no_std choice
    ptr: i32,
    /// Binary std/no_std choice
    size: i32,
    /// Alignment requirement
    align: i32,
    /// Binary std/no_std choice
    active: bool,
}

#[derive(Debug, Clone)]
struct ReallocFunction {
    /// Function index in the component
    func_index: u32,
    /// Cached function reference for performance
    func_ref: Option<Arc<dyn Fn(i32, i32, i32, i32) -> i32 + Send + Sync>>,
}

#[derive(Debug, Default, Clone)]
struct AllocationMetrics {
    /// Binary std/no_std choice
    total_allocations: u64,
    /// Binary std/no_std choice
    total_deallocations: u64,
    /// Binary std/no_std choice
    total_bytes_allocated: u64,
    /// Binary std/no_std choice
    total_bytes_deallocated: u64,
    /// Peak memory usage
    peak_memory_usage: u64,
    /// Binary std/no_std choice
    failed_allocations: u64,
}

impl ReallocManager {
    pub fn new(max_allocation_size: usize, max_instance_allocations: usize) -> Self {
        Self {
            allocations: BTreeMap::new(),
            metrics: AllocationMetrics::default(),
            max_allocation_size,
            max_instance_allocations,
        }
    }

    /// Binary std/no_std choice
    pub fn register_realloc(
        &mut self,
        instance_id: ComponentInstanceId,
        func_index: u32,
    ) -> Result<(), ComponentError> {
        let instance_allocs = self.allocations.entry(instance_id).or_insert_with(|| {
            InstanceAllocations { allocations: BoundedVec::new(), total_bytes: 0, realloc_fn: None }
        });

        instance_allocs.realloc_fn = Some(ReallocFunction { func_index, func_ref: None });

        Ok(())
    }

    /// Binary std/no_std choice
    pub fn allocate(
        &mut self,
        instance_id: ComponentInstanceId,
        size: i32,
        align: i32,
    ) -> Result<i32, ComponentError> {
        // Binary std/no_std choice
        self.validate_allocation(size, align)?;

        let instance_allocs = self
            .allocations
            .get_mut(&instance_id)
            .ok_or(ComponentError::ResourceNotFound(instance_id.0))?;

        // Binary std/no_std choice
        if instance_allocs.allocations.len() >= self.max_instance_allocations {
            self.metrics.failed_allocations += 1;
            return Err(ComponentError::TooManyGenerativeTypes);
        }

        // Binary std/no_std choice
        let ptr = self.call_realloc(instance_allocs, 0, 0, align, size)?;

        // Binary std/no_std choice
        let allocation = Allocation { ptr, size, align, active: true };

        instance_allocs
            .allocations
            .push(allocation)
            .map_err(|_| ComponentError::TooManyGenerativeTypes)?;

        instance_allocs.total_bytes += size as usize;

        // Update metrics
        self.metrics.total_allocations += 1;
        self.metrics.total_bytes_allocated += size as u64;
        self.update_peak_memory();

        Ok(ptr)
    }

    /// Binary std/no_std choice
    pub fn reallocate(
        &mut self,
        instance_id: ComponentInstanceId,
        old_ptr: i32,
        old_size: i32,
        align: i32,
        new_size: i32,
    ) -> Result<i32, ComponentError> {
        // Binary std/no_std choice
        self.validate_allocation(new_size, align)?;

        let instance_allocs = self
            .allocations
            .get_mut(&instance_id)
            .ok_or(ComponentError::ResourceNotFound(instance_id.0))?;

        // Binary std/no_std choice
        let alloc_index = instance_allocs
            .allocations
            .iter()
            .position(|a| a.ptr == old_ptr && a.size == old_size && a.active)
            .ok_or(ComponentError::ResourceNotFound(old_ptr as u32))?;

        // Binary std/no_std choice
        let new_ptr = self.call_realloc(instance_allocs, old_ptr, old_size, align, new_size)?;

        // Binary std/no_std choice
        if new_size == 0 {
            // Binary std/no_std choice
            instance_allocs.allocations[alloc_index].active = false;
            instance_allocs.total_bytes -= old_size as usize;
            self.metrics.total_deallocations += 1;
            self.metrics.total_bytes_deallocated += old_size as u64;
        } else {
            // Binary std/no_std choice
            instance_allocs.allocations[alloc_index].ptr = new_ptr;
            instance_allocs.allocations[alloc_index].size = new_size;
            instance_allocs.total_bytes =
                instance_allocs.total_bytes - (old_size as usize) + (new_size as usize);
            self.metrics.total_bytes_allocated += (new_size - old_size).max(0) as u64;
        }

        self.update_peak_memory();
        Ok(new_ptr)
    }

    /// Binary std/no_std choice
    pub fn deallocate(
        &mut self,
        instance_id: ComponentInstanceId,
        ptr: i32,
        size: i32,
        align: i32,
    ) -> Result<(), ComponentError> {
        self.reallocate(instance_id, ptr, size, align, 0)?;
        Ok(())
    }

    /// Binary std/no_std choice
    fn call_realloc(
        &self,
        instance_allocs: &InstanceAllocations,
        old_ptr: i32,
        old_size: i32,
        align: i32,
        new_size: i32,
    ) -> Result<i32, ComponentError> {
        let realloc_fn =
            instance_allocs.realloc_fn.as_ref().ok_or(ComponentError::ResourceNotFound(0))?;

        // In a real implementation, this would call the actual wasm function
        // Binary std/no_std choice
        if new_size == 0 {
            Ok(0) // Binary std/no_std choice
        } else if old_ptr == 0 {
            // Binary std/no_std choice
            Ok(0x1000 + new_size) // Dummy pointer calculation
        } else {
            // Binary std/no_std choice
            Ok(old_ptr) // In real impl, might return different pointer
        }
    }

    /// Binary std/no_std choice
    fn validate_allocation(&self, size: i32, align: i32) -> Result<(), ComponentError> {
        if size < 0 {
            return Err(ComponentError::TypeMismatch);
        }

        if size as usize > self.max_allocation_size {
            return Err(ComponentError::ResourceNotFound(0));
        }

        // Check alignment is power of 2
        if align <= 0 || (align & (align - 1)) != 0 {
            return Err(ComponentError::TypeMismatch);
        }

        Ok(())
    }

    /// Update peak memory usage
    fn update_peak_memory(&mut self) {
        let current_usage: u64 = self.allocations.values().map(|a| a.total_bytes as u64).sum();

        if current_usage > self.metrics.peak_memory_usage {
            self.metrics.peak_memory_usage = current_usage;
        }
    }

    /// Binary std/no_std choice
    pub fn cleanup_instance(
        &mut self,
        instance_id: ComponentInstanceId,
    ) -> Result<(), ComponentError> {
        if let Some(instance_allocs) = self.allocations.remove(&instance_id) {
            // Update metrics for cleanup
            for alloc in instance_allocs.allocations.iter() {
                if alloc.active {
                    self.metrics.total_deallocations += 1;
                    self.metrics.total_bytes_deallocated += alloc.size as u64;
                }
            }
        }
        Ok(())
    }

    /// Binary std/no_std choice
    pub fn metrics(&self) -> &AllocationMetrics {
        &self.metrics
    }

    /// Reset metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = AllocationMetrics::default();
    }
}

/// Binary std/no_std choice
pub mod helpers {
    use super::*;

    /// Binary std/no_std choice
    pub fn align_size(size: usize, align: usize) -> usize {
        (size + align - 1) & !(align - 1)
    }

    /// Validate pointer alignment
    pub fn is_aligned(ptr: i32, align: i32) -> bool {
        (ptr & (align - 1)) == 0
    }

    /// Binary std/no_std choice
    pub fn calculate_allocation_size(
        layout: &MemoryLayout,
        count: usize,
    ) -> Result<usize, ComponentError> {
        let item_size = layout.size;
        let align = layout.align;

        // Check for overflow
        let total_size = item_size.checked_mul(count).ok_or(ComponentError::TypeMismatch)?;

        // Add alignment padding
        let aligned_size = align_size(total_size, align);

        Ok(aligned_size)
    }
}

impl Default for ReallocManager {
    fn default() -> Self {
        Self::new(
            10 * 1024 * 1024, // Binary std/no_std choice
            1024,             // Binary std/no_std choice
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_realloc_manager_creation() {
        let manager = ReallocManager::new(1024, 10);
        assert_eq!(manager.max_allocation_size, 1024);
        assert_eq!(manager.max_instance_allocations, 10);
    }

    #[test]
    fn test_register_realloc() {
        let mut manager = ReallocManager::new(1024, 10);
        let instance_id = ComponentInstanceId(1);

        assert!(manager.register_realloc(instance_id, 42).is_ok());
    }

    #[test]
    fn test_allocation() {
        let mut manager = ReallocManager::new(1024, 10);
        let instance_id = ComponentInstanceId(1);

        // Binary std/no_std choice
        manager.register_realloc(instance_id, 42).unwrap();

        // Allocate memory
        let ptr = manager.allocate(instance_id, 64, 8);
        assert!(ptr.is_ok());
        assert_ne!(ptr.unwrap(), 0);

        // Check metrics
        assert_eq!(manager.metrics.total_allocations, 1);
        assert_eq!(manager.metrics.total_bytes_allocated, 64);
    }

    #[test]
    fn test_reallocation() {
        let mut manager = ReallocManager::new(1024, 10);
        let instance_id = ComponentInstanceId(1);

        manager.register_realloc(instance_id, 42).unwrap();

        // Binary std/no_std choice
        let ptr = manager.allocate(instance_id, 64, 8).unwrap();

        // Binary std/no_std choice
        let new_ptr = manager.reallocate(instance_id, ptr, 64, 8, 128);
        assert!(new_ptr.is_ok());
    }

    #[test]
    fn test_deallocation() {
        let mut manager = ReallocManager::new(1024, 10);
        let instance_id = ComponentInstanceId(1);

        manager.register_realloc(instance_id, 42).unwrap();

        // Binary std/no_std choice
        let ptr = manager.allocate(instance_id, 64, 8).unwrap();
        assert!(manager.deallocate(instance_id, ptr, 64, 8).is_ok());

        // Check metrics
        assert_eq!(manager.metrics.total_deallocations, 1);
        assert_eq!(manager.metrics.total_bytes_deallocated, 64);
    }

    #[test]
    fn test_allocation_limits() {
        let mut manager = ReallocManager::new(100, 2);
        let instance_id = ComponentInstanceId(1);

        manager.register_realloc(instance_id, 42).unwrap();

        // Test size limit
        assert!(manager.allocate(instance_id, 200, 8).is_err());

        // Test count limit
        assert!(manager.allocate(instance_id, 10, 8).is_ok());
        assert!(manager.allocate(instance_id, 10, 8).is_ok());
        assert!(manager.allocate(instance_id, 10, 8).is_err()); // Binary std/no_std choice
    }

    #[test]
    fn test_helpers() {
        use helpers::*;

        // Test align_size
        assert_eq!(align_size(10, 8), 16);
        assert_eq!(align_size(16, 8), 16);
        assert_eq!(align_size(17, 8), 24);

        // Test is_aligned
        assert!(is_aligned(16, 8));
        assert!(!is_aligned(17, 8));
        assert!(is_aligned(0, 1));

        // Binary std/no_std choice
        let layout = MemoryLayout { size: 10, align: 8 };
        assert_eq!(calculate_allocation_size(&layout, 3).unwrap(), 32); // 30 rounded up to 32
    }
}
