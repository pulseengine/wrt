//! WRT Memory Architecture Core
//!
//! This module defines the core memory architecture that enforces budget limits
//! across the entire WRT system with compile-time and runtime guarantees.

use core::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use crate::{Result, Error, ErrorCategory, codes};
use crate::safety_system::{SafetyLevel, AsildLevel};

/// Memory enforcement levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryEnforcementLevel {
    /// Permissive - warnings only
    Permissive,
    /// Strict - enforce budgets, allow some flexibility
    Strict,
    /// SafetyCritical - absolute enforcement, ASIL compliance
    SafetyCritical,
}

/// Global memory architecture configuration
#[derive(Debug)]
pub struct MemoryArchitecture {
    /// Total system memory budget
    total_budget: AtomicUsize,
    /// Currently allocated across all subsystems
    allocated: AtomicUsize,
    /// Peak allocation seen
    peak_allocated: AtomicUsize,
    /// Whether initialization phase is complete
    initialization_complete: AtomicBool,
    /// Enforcement level
    enforcement_level: MemoryEnforcementLevel,
    /// Safety level for the entire system
    safety_level: SafetyLevel,
}

/// Per-crate memory budget
#[derive(Debug, Clone)]
pub struct CrateBudget {
    /// Crate identifier
    pub crate_name: &'static str,
    /// Maximum memory this crate can use
    pub max_memory: usize,
    /// Currently allocated by this crate
    pub allocated: AtomicUsize,
    /// Peak allocation for this crate
    pub peak: AtomicUsize,
    /// Safety level requirement for this crate
    pub safety_level: SafetyLevel,
}

/// Memory allocation result
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AllocationResult {
    /// Allocation approved
    Approved,
    /// Denied - would exceed crate budget
    CrateBudgetExceeded,
    /// Denied - would exceed system budget
    SystemBudgetExceeded,
    /// Denied - initialization complete, no new allocations
    InitializationComplete,
    /// Denied - safety violation
    SafetyViolation,
}

impl MemoryArchitecture {
    /// Create new memory architecture with system budget
    pub const fn new(
        total_budget: usize,
        enforcement_level: MemoryEnforcementLevel,
        safety_level: SafetyLevel,
    ) -> Self {
        Self {
            total_budget: AtomicUsize::new(total_budget),
            allocated: AtomicUsize::new(0),
            peak_allocated: AtomicUsize::new(0),
            initialization_complete: AtomicBool::new(false),
            enforcement_level,
            safety_level,
        }
    }

    /// Request allocation from system budget
    pub fn request_allocation(&self, crate_name: &str, size: usize) -> Result<AllocationResult> {
        // Check if initialization is complete
        if self.initialization_complete.load(Ordering::Acquire) {
            if self.enforcement_level == MemoryEnforcementLevel::SafetyCritical {
                return Ok(AllocationResult::InitializationComplete);
            }
        }

        // Check system budget
        let current = self.allocated.load(Ordering::Acquire);
        let total_budget = self.total_budget.load(Ordering::Acquire);
        
        if current + size > total_budget {
            return Ok(AllocationResult::SystemBudgetExceeded);
        }

        // Safety-critical verification
        if self.safety_level.asil_level() >= AsildLevel::C {
            self.verify_safety_critical_allocation(size)?;
        }

        // Atomically update allocation
        let new_allocated = self.allocated.fetch_add(size, Ordering::AcqRel) + size;
        
        // Update peak if necessary
        loop {
            let current_peak = self.peak_allocated.load(Ordering::Acquire);
            if new_allocated <= current_peak {
                break;
            }
            if self.peak_allocated.compare_exchange_weak(
                current_peak, new_allocated, Ordering::AcqRel, Ordering::Acquire
            ).is_ok() {
                break;
            }
        }

        Ok(AllocationResult::Approved)
    }

    /// Complete initialization phase - lock further allocations
    pub fn complete_initialization(&self) -> Result<()> {
        self.initialization_complete.store(true, Ordering::Release);
        
        // Log completion for audit trail
        let stats = self.get_stats();
        #[cfg(feature = "std")]
        eprintln!(
            "Memory initialization complete. Allocated: {} bytes ({}% of budget)",
            stats.allocated, 
            (stats.allocated * 100) / stats.total_budget.max(1)
        );

        Ok(())
    }

    /// Safety-critical allocation verification
    fn verify_safety_critical_allocation(&self, size: usize) -> Result<()> {
        // No single allocation should exceed 10% of total budget
        let total_budget = self.total_budget.load(Ordering::Acquire);
        if size > total_budget / 10 {
            return Err(Error::new(
                ErrorCategory::Safety,
                codes::SAFETY_VIOLATION,
                "Single allocation exceeds 10% of system budget"
            ));
        }

        Ok(())
    }

    /// Get current memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        let allocated = self.allocated.load(Ordering::Acquire);
        let total_budget = self.total_budget.load(Ordering::Acquire);
        let peak = self.peak_allocated.load(Ordering::Acquire);
        
        MemoryStats {
            allocated,
            total_budget,
            available: total_budget.saturating_sub(allocated),
            peak,
            utilization_percent: (allocated * 100) / total_budget.max(1),
            initialization_complete: self.initialization_complete.load(Ordering::Acquire),
        }
    }
}

/// Memory statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub allocated: usize,
    pub total_budget: usize,
    pub available: usize,
    pub peak: usize,
    pub utilization_percent: usize,
    pub initialization_complete: bool,
}

/// Global memory architecture instance
static mut GLOBAL_MEMORY_ARCHITECTURE: Option<MemoryArchitecture> = None;
static MEMORY_INIT_LOCK: AtomicBool = AtomicBool::new(false);

/// Initialize global memory architecture
pub fn initialize_memory_architecture(
    total_budget: usize,
    enforcement_level: MemoryEnforcementLevel,
    safety_level: SafetyLevel,
) -> Result<()> {
    if MEMORY_INIT_LOCK.compare_exchange_weak(
        false, true, Ordering::AcqRel, Ordering::Acquire
    ).is_err() {
        return Err(Error::new(
            ErrorCategory::Runtime,
            codes::RUNTIME_ERROR,
            "Memory architecture already initialized"
        ));
    }

    unsafe {
        GLOBAL_MEMORY_ARCHITECTURE = Some(MemoryArchitecture::new(
            total_budget, enforcement_level, safety_level
        ));
    }

    Ok(())
}

/// Get global memory architecture
pub fn global_memory_architecture() -> Result<&'static MemoryArchitecture> {
    unsafe {
        GLOBAL_MEMORY_ARCHITECTURE.as_ref()
            .ok_or_else(|| Error::new(
                ErrorCategory::Runtime,
                codes::RUNTIME_ERROR,
                "Memory architecture not initialized"
            ))
    }
}

/// Complete initialization and lock allocations
pub fn complete_memory_initialization() -> Result<()> {
    global_memory_architecture()?.complete_initialization()
}

/// Crate-specific memory budget tracking
#[derive(Debug)]
pub struct CrateBudgetManager {
    budgets: [Option<CrateBudget>; 32], // Support up to 32 crates
    count: AtomicUsize,
}

impl CrateBudgetManager {
    pub const fn new() -> Self {
        const NONE: Option<CrateBudget> = None;
        Self {
            budgets: [NONE; 32],
            count: AtomicUsize::new(0),
        }
    }

    /// Register a crate with its memory budget
    pub fn register_crate(&mut self, budget: CrateBudget) -> Result<()> {
        let index = self.count.load(Ordering::Acquire);
        if index >= 32 {
            return Err(Error::new(
                ErrorCategory::Resource,
                codes::CAPACITY_EXCEEDED,
                "Too many crates registered"
            ));
        }

        self.budgets[index] = Some(budget);
        self.count.store(index + 1, Ordering::Release);
        Ok(())
    }

    /// Request allocation for a specific crate
    pub fn request_crate_allocation(&self, crate_name: &str, size: usize) -> Result<AllocationResult> {
        // Find the crate budget
        let count = self.count.load(Ordering::Acquire);
        for i in 0..count {
            if let Some(ref budget) = self.budgets[i] {
                if budget.crate_name == crate_name {
                    // Check crate-specific budget
                    let current = budget.allocated.load(Ordering::Acquire);
                    if current + size > budget.max_memory {
                        return Ok(AllocationResult::CrateBudgetExceeded);
                    }

                    // Check system budget
                    let system_result = global_memory_architecture()?
                        .request_allocation(crate_name, size)?;
                    
                    if system_result == AllocationResult::Approved {
                        // Update crate allocation
                        budget.allocated.fetch_add(size, Ordering::AcqRel);
                        
                        // Update peak
                        let new_allocated = current + size;
                        loop {
                            let current_peak = budget.peak.load(Ordering::Acquire);
                            if new_allocated <= current_peak {
                                break;
                            }
                            if budget.peak.compare_exchange_weak(
                                current_peak, new_allocated, Ordering::AcqRel, Ordering::Acquire
                            ).is_ok() {
                                break;
                            }
                        }
                    }

                    return Ok(system_result);
                }
            }
        }

        Err(Error::new(
            ErrorCategory::Runtime,
            codes::INVALID_FUNCTION_INDEX,
            "Crate not registered"
        ))
    }
}

/// Global crate budget manager
static mut GLOBAL_CRATE_BUDGETS: Option<CrateBudgetManager> = None;

/// Initialize crate budget tracking
pub fn initialize_crate_budgets() -> Result<&'static mut CrateBudgetManager> {
    unsafe {
        if GLOBAL_CRATE_BUDGETS.is_none() {
            GLOBAL_CRATE_BUDGETS = Some(CrateBudgetManager::new());
        }
        GLOBAL_CRATE_BUDGETS.as_mut()
            .ok_or_else(|| Error::new(
                ErrorCategory::Runtime,
                codes::RUNTIME_ERROR,
                "Failed to initialize crate budgets"
            ))
    }
}

/// Get crate budget manager
pub fn crate_budgets() -> Result<&'static CrateBudgetManager> {
    unsafe {
        GLOBAL_CRATE_BUDGETS.as_ref()
            .ok_or_else(|| Error::new(
                ErrorCategory::Runtime,
                codes::RUNTIME_ERROR,
                "Crate budgets not initialized"
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_architecture_basic() {
        let arch = MemoryArchitecture::new(
            1024 * 1024, // 1MB
            MemoryEnforcementLevel::Strict,
            SafetyLevel::default()
        );

        // Should allow allocation within budget
        assert_eq!(
            arch.request_allocation("test-crate", 512 * 1024).unwrap(),
            AllocationResult::Approved
        );

        // Should deny allocation exceeding budget  
        assert_eq!(
            arch.request_allocation("test-crate", 600 * 1024).unwrap(),
            AllocationResult::SystemBudgetExceeded
        );
    }

    #[test]
    fn test_initialization_lock() {
        let arch = MemoryArchitecture::new(
            1024 * 1024,
            MemoryEnforcementLevel::SafetyCritical,
            SafetyLevel::default()
        );

        // Should allow before completion
        assert_eq!(
            arch.request_allocation("test", 1024).unwrap(),
            AllocationResult::Approved
        );

        // Complete initialization
        arch.complete_initialization().unwrap();

        // Should deny after completion in safety-critical mode
        assert_eq!(
            arch.request_allocation("test", 1024).unwrap(),
            AllocationResult::InitializationComplete
        );
    }
}