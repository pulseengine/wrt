//! Memory Enforcement Integration
//!
//! This module integrates the existing memory budget system with memory providers
//! to ensure strict memory limits are enforced after initialization.

use core::sync::atomic::{AtomicBool, Ordering};
use crate::{Result, Error, ErrorCategory, codes};
use crate::memory_budget::{global_budget, BudgetResult};
use crate::safety_system::SafetyLevel;
use crate::safe_memory::Provider;

/// Global initialization lock - once set, no new allocations allowed
static MEMORY_SYSTEM_LOCKED: AtomicBool = AtomicBool::new(false);

/// Enhanced Provider wrapper that enforces budget limits
#[derive(Debug)]
pub struct BudgetEnforcedProvider<P: Provider> {
    inner: P,
    component_id: u32,
    safety_level: SafetyLevel,
}

impl<P: Provider> BudgetEnforcedProvider<P> {
    /// Create a new budget-enforced provider
    pub fn new(inner: P, component_id: u32, safety_level: SafetyLevel) -> Self {
        Self {
            inner,
            component_id,
            safety_level,
        }
    }

    /// Check if allocation is allowed based on current system state
    fn check_allocation_allowed(&self, size: usize) -> Result<()> {
        // Check if system is locked post-initialization
        if MEMORY_SYSTEM_LOCKED.load(Ordering::Acquire) {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ERROR,
                "Memory allocation denied: system locked post-initialization"
            ));
        }

        // Check component budget
        match global_budget()?.allocate_for_component(self.component_id, size)? {
            BudgetResult::Approved => Ok(()),
            BudgetResult::InsufficientBudget => Err(Error::new(
                ErrorCategory::Memory,
                codes::CAPACITY_EXCEEDED,
                "Allocation denied: component budget exceeded"
            )),
            BudgetResult::PostInitializationLocked => Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ERROR,
                "Allocation denied: post-initialization lock active"
            )),
            BudgetResult::SafetyViolation => Err(Error::new(
                ErrorCategory::Safety,
                codes::SAFETY_VIOLATION,
                "Allocation denied: safety violation"
            )),
        }
    }
}

/// Lock the global memory system - no further allocations allowed
/// 
/// This should be called after all components are initialized and their
/// memory requirements are pre-allocated.
pub fn lock_memory_system() -> Result<()> {
    // Lock the global budget system
    global_budget()?.lock_all_budgets()?;
    
    // Set the global memory lock
    MEMORY_SYSTEM_LOCKED.store(true, Ordering::Release);
    
    Ok(())
}

/// Check if memory system is locked
pub fn is_memory_system_locked() -> bool {
    MEMORY_SYSTEM_LOCKED.load(Ordering::Acquire)
}

/// Pre-allocate all memory for a component during initialization
/// 
/// This ensures all memory is allocated upfront before the system is locked.
pub fn pre_allocate_component_memory(
    component_id: u32, 
    allocations: &[(usize, &str)], // (size, description)
    safety_level: SafetyLevel
) -> Result<Vec<Box<[u8]>>> {
    if MEMORY_SYSTEM_LOCKED.load(Ordering::Acquire) {
        return Err(Error::new(
            ErrorCategory::Memory,
            codes::MEMORY_ERROR,
            "Cannot pre-allocate: system already locked"
        ));
    }

    let mut allocated_blocks = Vec::with_capacity(allocations.len());
    let mut total_allocated = 0usize;

    for (size, description) in allocations {
        // Check budget before allocation
        match global_budget()?.allocate_for_component(component_id, *size)? {
            BudgetResult::Approved => {
                // Allocate the memory block
                let mut block = vec![0u8; *size].into_boxed_slice();
                total_allocated += *size;
                allocated_blocks.push(block);
                
                // Log allocation for audit trail
                #[cfg(feature = "std")]
                eprintln!("Pre-allocated {} bytes for component {} ({})", size, component_id, description);
            }
            _ => {
                return Err(Error::new(
                    ErrorCategory::Memory,
                    codes::CAPACITY_EXCEEDED,
                    &format!("Failed to pre-allocate {} bytes for component {}", size, component_id)
                ));
            }
        }
    }

    Ok(allocated_blocks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_budget::{initialize_global_budget};
    use crate::safety_system::SafetyLevel;

    #[test]
    fn test_memory_system_locking() {
        // Initialize budget system
        initialize_global_budget(1024 * 1024, SafetyLevel::default()).unwrap();
        
        // Should allow allocation before lock
        assert!(!is_memory_system_locked());
        
        // Lock the system
        lock_memory_system().unwrap();
        
        // Should be locked now
        assert!(is_memory_system_locked());
    }

    #[test]
    fn test_pre_allocation() {
        // Initialize budget system
        initialize_global_budget(1024 * 1024, SafetyLevel::default()).unwrap();
        
        let allocations = &[
            (1024, "Component buffer"),
            (2048, "Component workspace"),
        ];
        
        let result = pre_allocate_component_memory(
            1, 
            allocations, 
            SafetyLevel::default()
        );
        
        assert!(result.is_ok());
        let blocks = result.unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].len(), 1024);
        assert_eq!(blocks[1].len(), 2048);
    }
}