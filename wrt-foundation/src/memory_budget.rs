//! Memory Budget Enforcement System
//!
//! This module provides static memory budget allocation and enforcement mechanisms
//! to prevent dynamic allocation after system initialization.

use core::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use crate::safety_system::{SafetyLevel, AsildLevel};
use crate::verification::VerificationLevel;
use crate::{Result, Error, ErrorCategory, codes};

/// Global memory budget state
#[derive(Debug)]
pub struct MemoryBudget {
    /// Total allocated bytes across all components
    allocated_bytes: AtomicUsize,
    /// Maximum allowed allocation (hard limit)
    max_allocation: usize,
    /// Whether system is in post-initialization mode (no new allocations)
    post_initialization: AtomicBool,
    /// Safety level for budget enforcement
    safety_level: SafetyLevel,
    /// Verification level for budget checks
    verification_level: VerificationLevel,
}

/// Budget allocation result
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BudgetResult {
    /// Allocation approved
    Approved,
    /// Allocation denied - insufficient budget
    InsufficientBudget,
    /// Allocation denied - system locked post-initialization
    PostInitializationLocked,
    /// Allocation denied - safety violation
    SafetyViolation,
}

/// Memory budget registry for tracking component allocations
#[derive(Debug)]
pub struct BudgetRegistry {
    /// System-wide budget tracker
    global_budget: MemoryBudget,
    /// Per-component budget tracking
    component_budgets: crate::bounded::BoundedVec<ComponentBudget, 64, crate::safe_memory::NoStdProvider<4096>>,
}

/// Per-component memory budget
#[derive(Debug, Clone)]
pub struct ComponentBudget {
    /// Component identifier
    pub component_id: u32,
    /// Allocated bytes for this component
    pub allocated_bytes: usize,
    /// Maximum allowed for this component
    pub max_bytes: usize,
    /// Component safety level
    pub safety_level: SafetyLevel,
}

impl MemoryBudget {
    /// Create a new memory budget with specified limits
    pub const fn new(max_allocation: usize, safety_level: SafetyLevel) -> Self {
        Self {
            allocated_bytes: AtomicUsize::new(0),
            max_allocation,
            post_initialization: AtomicBool::new(false),
            safety_level,
            verification_level: VerificationLevel::default(),
        }
    }

    /// Reserve memory allocation (pre-initialization only)
    pub fn reserve_allocation(&self, size: usize) -> Result<BudgetResult> {
        // Check if post-initialization lock is active
        if self.post_initialization.load(Ordering::Acquire) {
            return Ok(BudgetResult::PostInitializationLocked);
        }

        // Perform safety-level appropriate verification
        if self.verification_level.should_verify(128) {
            self.verify_allocation_request(size)?;
        }

        // Atomic check-and-increment for thread safety
        let current = self.allocated_bytes.load(Ordering::Acquire);
        let new_total = current.checked_add(size)
            .ok_or_else(|| Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ERROR,
                "Memory allocation overflow"
            ))?;

        if new_total > self.max_allocation {
            return Ok(BudgetResult::InsufficientBudget);
        }

        // Try to atomically update allocation
        match self.allocated_bytes.compare_exchange_weak(
            current,
            new_total,
            Ordering::AcqRel,
            Ordering::Acquire
        ) {
            Ok(_) => Ok(BudgetResult::Approved),
            Err(_) => {
                // Retry logic could be added here for high-contention scenarios
                Ok(BudgetResult::InsufficientBudget)
            }
        }
    }

    /// Lock the budget - no further allocations allowed
    pub fn lock_post_initialization(&self) -> Result<()> {
        // For safety-critical systems, this should be irreversible
        self.post_initialization.store(true, Ordering::Release);
        
        // Log the lock event for audit trail
        if self.safety_level.asil_level() >= AsildLevel::C {
            // In real implementation, this would log to safety-critical audit log
            // For now, we'll use a verification checkpoint
            self.verification_level.record_safety_event(
                "Memory budget locked post-initialization"
            );
        }
        
        Ok(())
    }

    /// Check if allocation is allowed (read-only check)
    pub fn can_allocate(&self, size: usize) -> BudgetResult {
        if self.post_initialization.load(Ordering::Acquire) {
            return BudgetResult::PostInitializationLocked;
        }

        let current = self.allocated_bytes.load(Ordering::Acquire);
        if current.saturating_add(size) > self.max_allocation {
            BudgetResult::InsufficientBudget
        } else {
            BudgetResult::Approved
        }
    }

    /// Get current allocation statistics
    pub fn allocation_stats(&self) -> AllocationStats {
        let current = self.allocated_bytes.load(Ordering::Acquire);
        AllocationStats {
            allocated_bytes: current,
            remaining_bytes: self.max_allocation.saturating_sub(current),
            max_allocation: self.max_allocation,
            is_locked: self.post_initialization.load(Ordering::Acquire),
            utilization_percent: (current * 100) / self.max_allocation.max(1),
        }
    }

    /// Safety verification for allocation requests
    fn verify_allocation_request(&self, size: usize) -> Result<()> {
        // Size reasonableness check
        if size == 0 {
            return Err(Error::new(
                ErrorCategory::Validation,
                codes::VALIDATION_ERROR,
                "Zero-size allocation request"
            ));
        }

        // Maximum single allocation check (prevent huge allocations)
        if size > self.max_allocation / 4 {
            return Err(Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ERROR,
                "Single allocation too large (>25% of budget)"
            ));
        }

        // ASIL-specific checks
        match self.safety_level.asil_level() {
            AsildLevel::C | AsildLevel::D => {
                // Stricter limits for highest safety levels
                if size > self.max_allocation / 8 {
                    return Err(Error::new(
                        ErrorCategory::Safety,
                        codes::SAFETY_VIOLATION,
                        "Allocation exceeds ASIL-C/D single allocation limit"
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }
}

/// Allocation statistics
#[derive(Debug, Clone, Copy)]
pub struct AllocationStats {
    pub allocated_bytes: usize,
    pub remaining_bytes: usize,
    pub max_allocation: usize,
    pub is_locked: bool,
    pub utilization_percent: usize,
}

impl BudgetRegistry {
    /// Create a new budget registry
    pub fn new(global_max: usize, safety_level: SafetyLevel) -> Result<Self> {
        Ok(Self {
            global_budget: MemoryBudget::new(global_max, safety_level),
            component_budgets: crate::bounded::BoundedVec::new(
                crate::safe_memory::NoStdProvider::<4096>::default()
            )?,
        })
    }

    /// Register a component with its memory budget
    pub fn register_component(&mut self, component_id: u32, max_bytes: usize, safety_level: SafetyLevel) -> Result<()> {
        let budget = ComponentBudget {
            component_id,
            allocated_bytes: 0,
            max_bytes,
            safety_level,
        };

        self.component_budgets.push(budget)
            .map_err(|_| Error::new(
                ErrorCategory::Resource,
                codes::CAPACITY_EXCEEDED,
                "Too many components registered"
            ))
    }

    /// Allocate memory for a specific component
    pub fn allocate_for_component(&mut self, component_id: u32, size: usize) -> Result<BudgetResult> {
        // First check global budget
        let global_result = self.global_budget.reserve_allocation(size)?;
        if global_result != BudgetResult::Approved {
            return Ok(global_result);
        }

        // Find and update component budget
        for budget in self.component_budgets.iter_mut() {
            if budget.component_id == component_id {
                if budget.allocated_bytes.saturating_add(size) > budget.max_bytes {
                    // Rollback global allocation
                    self.global_budget.allocated_bytes.fetch_sub(size, Ordering::AcqRel);
                    return Ok(BudgetResult::InsufficientBudget);
                }
                budget.allocated_bytes += size;
                return Ok(BudgetResult::Approved);
            }
        }

        // Component not found - rollback and error
        self.global_budget.allocated_bytes.fetch_sub(size, Ordering::AcqRel);
        Err(Error::new(
            ErrorCategory::Runtime,
            codes::INVALID_FUNCTION_INDEX,
            "Component not registered"
        ))
    }

    /// Lock all budgets post-initialization
    pub fn lock_all_budgets(&self) -> Result<()> {
        self.global_budget.lock_post_initialization()
    }

    /// Get system-wide allocation statistics
    pub fn system_stats(&self) -> SystemStats {
        let global_stats = self.global_budget.allocation_stats();
        let component_count = self.component_budgets.len();
        
        SystemStats {
            global: global_stats,
            component_count,
            total_registered_components: self.component_budgets.capacity(),
        }
    }
}

/// System-wide allocation statistics
#[derive(Debug)]
pub struct SystemStats {
    pub global: AllocationStats,
    pub component_count: usize,
    pub total_registered_components: usize,
}

/// Global budget registry instance (for singleton pattern)
static mut GLOBAL_BUDGET_REGISTRY: Option<BudgetRegistry> = None;
static BUDGET_INIT_LOCK: AtomicBool = AtomicBool::new(false);

/// Initialize the global budget system (call once at startup)
pub fn initialize_global_budget(max_memory: usize, safety_level: SafetyLevel) -> Result<()> {
    // Atomic check-and-set for initialization
    if BUDGET_INIT_LOCK.compare_exchange_weak(
        false, true, Ordering::AcqRel, Ordering::Acquire
    ).is_err() {
        return Err(Error::new(
            ErrorCategory::Runtime,
            codes::RUNTIME_ERROR,
            "Budget system already initialized"
        ));
    }

    unsafe {
        GLOBAL_BUDGET_REGISTRY = Some(BudgetRegistry::new(max_memory, safety_level)?);
    }
    
    Ok(())
}

/// Get reference to global budget registry
pub fn global_budget() -> Result<&'static mut BudgetRegistry> {
    unsafe {
        GLOBAL_BUDGET_REGISTRY.as_mut()
            .ok_or_else(|| Error::new(
                ErrorCategory::Runtime,
                codes::RUNTIME_ERROR,
                "Budget system not initialized"
            ))
    }
}

/// Lock the global budget system (call after initialization phase)
pub fn lock_global_budget() -> Result<()> {
    global_budget()?.lock_all_budgets()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_basic_allocation() {
        let budget = MemoryBudget::new(1024, SafetyLevel::default());
        
        // Should allow allocation within budget
        assert_eq!(budget.reserve_allocation(512).unwrap(), BudgetResult::Approved);
        assert_eq!(budget.reserve_allocation(400).unwrap(), BudgetResult::Approved);
        
        // Should deny allocation exceeding budget
        assert_eq!(budget.reserve_allocation(200).unwrap(), BudgetResult::InsufficientBudget);
    }

    #[test]
    fn test_post_initialization_lock() {
        let budget = MemoryBudget::new(1024, SafetyLevel::default());
        
        // Should allow allocation before lock
        assert_eq!(budget.reserve_allocation(512).unwrap(), BudgetResult::Approved);
        
        // Lock the budget
        budget.lock_post_initialization().unwrap();
        
        // Should deny allocation after lock
        assert_eq!(budget.reserve_allocation(100).unwrap(), BudgetResult::PostInitializationLocked);
    }

    #[test]
    fn test_component_budget_tracking() {
        let mut registry = BudgetRegistry::new(2048, SafetyLevel::default()).unwrap();
        
        // Register components
        registry.register_component(1, 1000, SafetyLevel::default()).unwrap();
        registry.register_component(2, 500, SafetyLevel::default()).unwrap();
        
        // Allocate within component limits
        assert_eq!(registry.allocate_for_component(1, 800).unwrap(), BudgetResult::Approved);
        assert_eq!(registry.allocate_for_component(2, 400).unwrap(), BudgetResult::Approved);
        
        // Should deny allocation exceeding component limit
        assert_eq!(registry.allocate_for_component(1, 300).unwrap(), BudgetResult::InsufficientBudget);
    }
}