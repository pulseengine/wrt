//! Crate-Specific Memory Budget Definitions
//!
//! This module defines memory budgets for each WRT crate based on their
//! functionality and safety requirements.

use crate::memory_architecture::{CrateBudget, MemoryEnforcementLevel};
use crate::safety_system::SafetyLevel;
use core::sync::atomic::AtomicUsize;

/// Standard memory budget configurations for different deployment scenarios
pub struct StandardBudgets;

impl StandardBudgets {
    /// Embedded system budgets (total: ~8MB)
    pub fn embedded() -> &'static [CrateBudget] {
        &[
            // Core foundation crates
            CrateBudget {
                crate_name: "wrt-error",
                max_memory: 16 * 1024,        // 16KB - minimal error handling
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilD, // Errors are safety-critical
            },
            CrateBudget {
                crate_name: "wrt-foundation",
                max_memory: 512 * 1024,       // 512KB - core data structures
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilD, // Foundation is safety-critical
            },
            CrateBudget {
                crate_name: "wrt-sync",
                max_memory: 64 * 1024,        // 64KB - synchronization primitives
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilC, // Sync is important but not critical
            },
            CrateBudget {
                crate_name: "wrt-platform",
                max_memory: 256 * 1024,       // 256KB - platform abstraction
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilC, // Platform-specific code
            },

            // Format and parsing crates
            CrateBudget {
                crate_name: "wrt-format",
                max_memory: 1024 * 1024,      // 1MB - WebAssembly format parsing
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilB, // Format parsing is important
            },
            CrateBudget {
                crate_name: "wrt-decoder",
                max_memory: 512 * 1024,       // 512KB - binary decoding
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilB, // Decoder buffers
            },
            CrateBudget {
                crate_name: "wrt-instructions",
                max_memory: 768 * 1024,       // 768KB - instruction implementation
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilC, // Instructions affect execution
            },

            // Runtime execution crates  
            CrateBudget {
                crate_name: "wrt-runtime",
                max_memory: 2 * 1024 * 1024,  // 2MB - execution engine
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilD, // Runtime is safety-critical
            },
            CrateBudget {
                crate_name: "wrt-component",
                max_memory: 1024 * 1024,      // 1MB - component model
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilC, // Component management
            },
            CrateBudget {
                crate_name: "wrt-host",
                max_memory: 512 * 1024,       // 512KB - host functions
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilB, // Host interface
            },

            // Support and tooling crates
            CrateBudget {
                crate_name: "wrt-debug",
                max_memory: 256 * 1024,       // 256KB - debug information
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::QM, // Debug info is non-safety
            },
            CrateBudget {
                crate_name: "wrt-logging",
                max_memory: 128 * 1024,       // 128KB - logging infrastructure
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::QM, // Logging is non-safety
            },
            CrateBudget {
                crate_name: "wrt-intercept",
                max_memory: 256 * 1024,       // 256KB - function interception
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilA, // Monitoring/debugging
            },
            CrateBudget {
                crate_name: "wrt-math",
                max_memory: 64 * 1024,        // 64KB - mathematical operations
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilB, // Math operations matter
            },

            // Main integration crate
            CrateBudget {
                crate_name: "wrt",
                max_memory: 512 * 1024,       // 512KB - main integration
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilC, // Main interface
            },
        ]
        // Total: ~8.1MB for embedded systems
    }

    /// Desktop/server budgets (total: ~64MB)  
    pub fn desktop() -> &'static [CrateBudget] {
        &[
            // Core foundation crates (scaled up)
            CrateBudget {
                crate_name: "wrt-error",
                max_memory: 64 * 1024,        // 64KB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilC,
            },
            CrateBudget {
                crate_name: "wrt-foundation", 
                max_memory: 4 * 1024 * 1024,  // 4MB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilC,
            },
            CrateBudget {
                crate_name: "wrt-sync",
                max_memory: 256 * 1024,       // 256KB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilB,
            },
            CrateBudget {
                crate_name: "wrt-platform",
                max_memory: 1024 * 1024,      // 1MB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilB,
            },

            // Format and parsing (more generous)
            CrateBudget {
                crate_name: "wrt-format",
                max_memory: 8 * 1024 * 1024,  // 8MB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilA,
            },
            CrateBudget {
                crate_name: "wrt-decoder",
                max_memory: 4 * 1024 * 1024,  // 4MB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilA,
            },
            CrateBudget {
                crate_name: "wrt-instructions",
                max_memory: 4 * 1024 * 1024,  // 4MB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilB,
            },

            // Runtime (more memory for larger applications)
            CrateBudget {
                crate_name: "wrt-runtime",
                max_memory: 16 * 1024 * 1024, // 16MB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilC,
            },
            CrateBudget {
                crate_name: "wrt-component",
                max_memory: 8 * 1024 * 1024,  // 8MB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilB,
            },
            CrateBudget {
                crate_name: "wrt-host",
                max_memory: 4 * 1024 * 1024,  // 4MB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilA,
            },

            // Support and tooling (generous for development)
            CrateBudget {
                crate_name: "wrt-debug",
                max_memory: 4 * 1024 * 1024,  // 4MB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::QM,
            },
            CrateBudget {
                crate_name: "wrt-logging",
                max_memory: 2 * 1024 * 1024,  // 2MB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::QM,
            },
            CrateBudget {
                crate_name: "wrt-intercept",
                max_memory: 1024 * 1024,      // 1MB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::QM,
            },
            CrateBudget {
                crate_name: "wrt-math",
                max_memory: 256 * 1024,       // 256KB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilA,
            },

            // Main integration
            CrateBudget {
                crate_name: "wrt",
                max_memory: 4 * 1024 * 1024,  // 4MB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilB,
            },
        ]
        // Total: ~64MB for desktop/server systems
    }

    /// Ultra-conservative embedded budgets (total: ~2MB)
    pub fn ultra_embedded() -> &'static [CrateBudget] {
        &[
            CrateBudget {
                crate_name: "wrt-error",
                max_memory: 4 * 1024,         // 4KB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilD,
            },
            CrateBudget {
                crate_name: "wrt-foundation",
                max_memory: 256 * 1024,       // 256KB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilD,
            },
            CrateBudget {
                crate_name: "wrt-sync",
                max_memory: 16 * 1024,        // 16KB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilC,
            },
            CrateBudget {
                crate_name: "wrt-platform",
                max_memory: 64 * 1024,        // 64KB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilC,
            },
            CrateBudget {
                crate_name: "wrt-format",
                max_memory: 256 * 1024,       // 256KB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilB,
            },
            CrateBudget {
                crate_name: "wrt-decoder",
                max_memory: 128 * 1024,       // 128KB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilB,
            },
            CrateBudget {
                crate_name: "wrt-instructions",
                max_memory: 256 * 1024,       // 256KB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilC,
            },
            CrateBudget {
                crate_name: "wrt-runtime",
                max_memory: 512 * 1024,       // 512KB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilD,
            },
            CrateBudget {
                crate_name: "wrt-component",
                max_memory: 256 * 1024,       // 256KB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilC,
            },
            CrateBudget {
                crate_name: "wrt",
                max_memory: 128 * 1024,       // 128KB
                allocated: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
                safety_level: SafetyLevel::AsilC,
            },
        ]
        // Total: ~1.9MB for ultra-embedded systems
    }

    /// Get total budget for a configuration
    pub fn total_budget(budgets: &[CrateBudget]) -> usize {
        budgets.iter().map(|b| b.max_memory).sum()
    }

    /// Get safety-critical crates (ASIL-C or higher)
    pub fn safety_critical_crates(budgets: &[CrateBudget]) -> Vec<&'static str> {
        budgets.iter()
            .filter(|b| b.safety_level.asil_level() >= crate::safety_system::AsildLevel::C)
            .map(|b| b.crate_name)
            .collect()
    }
}

/// Runtime budget configuration selection
pub enum BudgetConfiguration {
    /// Ultra-conservative for minimal embedded systems
    UltraEmbedded,
    /// Standard embedded systems  
    Embedded,
    /// Desktop and server systems
    Desktop,
    /// Custom configuration
    Custom(&'static [CrateBudget]),
}

impl BudgetConfiguration {
    /// Get the crate budgets for this configuration
    pub fn budgets(&self) -> &'static [CrateBudget] {
        match self {
            Self::UltraEmbedded => StandardBudgets::ultra_embedded(),
            Self::Embedded => StandardBudgets::embedded(),
            Self::Desktop => StandardBudgets::desktop(),
            Self::Custom(budgets) => budgets,
        }
    }

    /// Get total memory budget
    pub fn total_budget(&self) -> usize {
        StandardBudgets::total_budget(self.budgets())
    }

    /// Get enforcement level recommendation
    pub fn recommended_enforcement_level(&self) -> MemoryEnforcementLevel {
        match self {
            Self::UltraEmbedded => MemoryEnforcementLevel::SafetyCritical,
            Self::Embedded => MemoryEnforcementLevel::Strict,
            Self::Desktop => MemoryEnforcementLevel::Strict,
            Self::Custom(_) => MemoryEnforcementLevel::Strict,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_configurations() {
        let ultra = BudgetConfiguration::UltraEmbedded;
        let embedded = BudgetConfiguration::Embedded;  
        let desktop = BudgetConfiguration::Desktop;

        assert!(ultra.total_budget() < embedded.total_budget());
        assert!(embedded.total_budget() < desktop.total_budget());

        println!("Ultra embedded: {} bytes", ultra.total_budget());
        println!("Embedded: {} bytes", embedded.total_budget()); 
        println!("Desktop: {} bytes", desktop.total_budget());
    }

    #[test]
    fn test_safety_critical_identification() {
        let budgets = StandardBudgets::embedded();
        let safety_critical = StandardBudgets::safety_critical_crates(budgets);
        
        assert!(safety_critical.contains(&"wrt-error"));
        assert!(safety_critical.contains(&"wrt-foundation"));
        assert!(safety_critical.contains(&"wrt-runtime"));
    }
}