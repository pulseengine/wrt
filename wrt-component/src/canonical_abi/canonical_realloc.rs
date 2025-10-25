//! Canonical ABI realloc function handling
//!
//! This module implements the realloc function support for the WebAssembly
//! Component Model's Canonical ABI, enabling dynamic memory allocation
//! during lifting and lowering operations.

use wrt_error::{
    codes,
    Error,
    ErrorCategory,
};
use wrt_foundation::{
    bounded::MAX_COMPONENT_TYPES,
    budget_aware_provider::CrateId,
    collections::StaticVec as BoundedVec,
    safe_managed_alloc,
};
use wrt_error::Result;

// Type aliases for no_std compatibility
pub use crate::types::ComponentInstanceId;

/// Binary std/no_std choice
pub type ReallocFn = fn(i32, i32, i32, i32) -> i32;

/// Binary std/no_std choice
#[derive(Debug, Clone)]
pub struct CanonicalOptionsWithRealloc {
    /// Memory for canonical operations
    pub memory:          u32,
    /// Binary std/no_std choice
    pub realloc:         Option<u32>,
    /// Post-return function index
    pub post_return:     Option<u32>,
    /// String encoding
    pub string_encoding: StringEncoding,
    /// Component instance ID
    pub instance_id:     ComponentInstanceId,
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
    allocations:              BoundedVec<(ComponentInstanceId, InstanceAllocations), 32>,
    /// Binary std/no_std choice
    metrics:                  AllocationMetrics,
    /// Binary std/no_std choice
    max_allocation_size:      usize,
    /// Binary std/no_std choice
    max_instance_allocations: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InstanceAllocations {
    /// Binary std/no_std choice
    allocations: BoundedVec<Allocation, MAX_COMPONENT_TYPES>,
    /// Binary std/no_std choice
    total_bytes: usize,
    /// Binary std/no_std choice
    realloc_fn:  Option<ReallocFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Allocation {
    /// Binary std/no_std choice
    ptr:    i32,
    /// Binary std/no_std choice
    size:   i32,
    /// Alignment requirement
    align:  i32,
    /// Binary std/no_std choice
    active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Default)]
struct ReallocFunction {
    /// Function index in the component
    func_index:     u32,
    /// Cached function reference for performance (simplified for no_std)
    func_available: bool,
}

#[derive(Debug, Default, Clone)]
pub struct AllocationMetrics {
    /// Binary std/no_std choice
    total_allocations:       u64,
    /// Binary std/no_std choice
    total_deallocations:     u64,
    /// Binary std/no_std choice
    total_bytes_allocated:   u64,
    /// Binary std/no_std choice
    total_bytes_deallocated: u64,
    /// Peak memory usage
    peak_memory_usage:       u64,
    /// Binary std/no_std choice
    failed_allocations:      u64,
}

impl ReallocManager {
    pub fn new(max_allocation_size: usize, max_instance_allocations: usize) -> Result<Self> {
        Ok(Self {
            allocations: BoundedVec::new(),
            metrics: AllocationMetrics::default(),
            max_allocation_size,
            max_instance_allocations,
        })
    }

    /// Binary std/no_std choice
    pub fn register_realloc(
        &mut self,
        instance_id: ComponentInstanceId,
        func_index: u32,
    ) -> Result<()> {
        // Find existing instance or create new one
        let mut found = false;
        for (id, instance_allocs) in &mut self.allocations {
            if *id == instance_id {
                instance_allocs.realloc_fn = Some(ReallocFunction {
                    func_index,
                    func_available: true,
                });
                found = true;
                break;
            }
        }

        if !found {
            let instance_allocs = InstanceAllocations {
                allocations: BoundedVec::new(),
                total_bytes: 0,
                realloc_fn:  Some(ReallocFunction {
                    func_index,
                    func_available: true,
                }),
            };
            self.allocations
                .push((instance_id, instance_allocs))
                .map_err(|_| Error::capacity_exceeded("too_many_allocations"))?;
        }

        Ok(())
    }

    /// Binary std/no_std choice
    pub fn allocate(
        &mut self,
        instance_id: ComponentInstanceId,
        size: i32,
        align: i32,
    ) -> Result<i32> {
        // Binary std/no_std choice
        self.validate_allocation(size, align)?;

        // Check limits first before getting mutable borrow
        {
            let instance_allocs_check = self
                .allocations
                .iter()
                .find(|(id, _)| *id == instance_id)
                .map(|(_, allocs)| allocs)
                .ok_or(Error::resource_not_found("resource_not_found"))?;

            if instance_allocs_check.allocations.len() >= self.max_instance_allocations {
                self.metrics.failed_allocations += 1;
                return Err(Error::capacity_exceeded("too_many_types"));
            }
        }

        // Get mutable borrow for allocation
        let instance_allocs = self
            .allocations
            .iter_mut()
            .find(|(id, _)| *id == instance_id)
            .map(|(_, allocs)| allocs)
            .ok_or(Error::resource_not_found("resource_not_found"))?;

        // Call realloc - extract function reference to avoid borrowing self
        let realloc_fn = instance_allocs
            .realloc_fn
            .as_ref()
            .ok_or(Error::resource_not_found("resource_not_found"))?;

        // Binary std/no_std choice - call realloc inline
        let ptr = if size == 0 {
            0 // Binary std/no_std choice
        } else {
            0x1000 + size // Dummy pointer calculation
        };

        // Binary std/no_std choice
        let allocation = Allocation {
            ptr,
            size,
            align,
            active: true,
        };

        instance_allocs
            .allocations
            .push(allocation)
            .map_err(|_| Error::capacity_exceeded("too_many_types"))?;

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
    ) -> Result<i32> {
        // Binary std/no_std choice
        self.validate_allocation(new_size, align)?;

        // Find allocation index first
        let alloc_index = {
            let instance_allocs_check = self
                .allocations
                .iter()
                .find(|(id, _)| *id == instance_id)
                .map(|(_, allocs)| allocs)
                .ok_or(Error::resource_not_found("resource_not_found"))?;

            instance_allocs_check
                .allocations
                .iter()
                .position(|a| a.ptr == old_ptr && a.size == old_size && a.active)
                .ok_or(Error::resource_not_found("resource_not_found"))?
        };

        // Get mutable borrow for realloc
        let instance_allocs = self
            .allocations
            .iter_mut()
            .find(|(id, _)| *id == instance_id)
            .map(|(_, allocs)| allocs)
            .ok_or(Error::resource_not_found("resource_not_found"))?;

        // Call realloc - extract function reference to avoid borrowing self
        let realloc_fn = instance_allocs
            .realloc_fn
            .as_ref()
            .ok_or(Error::resource_not_found("resource_not_found"))?;

        // Binary std/no_std choice - call realloc inline
        let new_ptr = if new_size == 0 {
            0 // Binary std/no_std choice
        } else if old_ptr == 0 {
            0x1000 + new_size // Dummy pointer calculation
        } else {
            old_ptr // In real impl, might return different pointer
        };

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
    ) -> Result<()> {
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
    ) -> Result<i32> {
        let realloc_fn = instance_allocs
            .realloc_fn
            .as_ref()
            .ok_or(Error::resource_not_found("resource_not_found"))?;

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
    fn validate_allocation(&self, size: i32, align: i32) -> Result<()> {
        if size < 0 {
            return Err(Error::validation_error("type_mismatch"));
        }

        if size as usize > self.max_allocation_size {
            return Err(Error::resource_not_found("resource_not_found"));
        }

        // Check alignment is power of 2
        if align <= 0 || (align & (align - 1)) != 0 {
            return Err(Error::validation_error("type_mismatch"));
        }

        Ok(())
    }

    /// Update peak memory usage
    fn update_peak_memory(&mut self) {
        let current_usage: u64 = self.allocations.iter().map(|(_, a)| a.total_bytes as u64).sum();

        if current_usage > self.metrics.peak_memory_usage {
            self.metrics.peak_memory_usage = current_usage;
        }
    }

    /// Binary std/no_std choice
    pub fn cleanup_instance(&mut self, instance_id: ComponentInstanceId) -> Result<()> {
        // Find and remove the instance
        if let Some(pos) = self.allocations.iter().position(|(id, _)| *id == instance_id) {
            let (_, instance_allocs) = self.allocations.remove(pos);
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

/// Memory layout for allocation calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryLayout {
    pub size:  usize,
    pub align: usize,
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
    pub fn calculate_allocation_size(layout: &MemoryLayout, count: usize) -> Result<usize> {
        let item_size = layout.size;
        let align = layout.align;

        // Check for overflow
        let total_size =
            item_size.checked_mul(count).ok_or(Error::validation_error("type_mismatch"))?;

        // Add alignment padding
        let aligned_size = align_size(total_size, align);

        Ok(aligned_size)
    }
}

impl ReallocManager {
    fn default() -> Result<Self> {
        Self::new(
            10 * 1024 * 1024, // Binary std/no_std choice
            1024,             // Binary std/no_std choice
        )
    }

}
