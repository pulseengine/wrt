//! Budget Provider Compatibility Layer
//! 
//! This module provides compatibility for existing code that expects BudgetProvider
//! while we transition to the new generic memory system.

use crate::{
    safe_memory::NoStdProvider,
    budget_aware_provider::CrateId,
    wrt_memory_system::WrtProviderFactory,
    Error, Result,
};

/// Compatibility wrapper for budget-aware provider creation
pub struct BudgetProvider;

impl BudgetProvider {
    /// Create a new provider with budget tracking
    #[deprecated(
        since = "0.3.0",
        note = "Use WrtProviderFactory::create_provider() for new code"
    )]
    pub fn new<const N: usize>(crate_id: CrateId) -> Result<NoStdProvider<N>> {
        // Create through factory
        let guard = WrtProviderFactory::create_provider::<N>(crate_id)?;
        
        // For compatibility, we release the guard
        // This is not ideal but maintains backward compatibility
        #[allow(unsafe_code)] // Safe: guard.release() transfers ownership properly
        Ok(unsafe { guard.release() })
    }
    
    /// Create with explicit size (for compatibility)
    #[deprecated(
        since = "0.3.0", 
        note = "Use WrtProviderFactory::create_provider() for new code"
    )]
    pub fn with_size<const N: usize>(crate_id: CrateId, _size: usize) -> Result<NoStdProvider<N>> {
        // Create through factory
        let guard = WrtProviderFactory::create_provider::<N>(crate_id)?;
        
        // For compatibility, we release the guard
        // This is not ideal but maintains backward compatibility
        #[allow(unsafe_code)] // Safe: guard.release() transfers ownership properly
        Ok(unsafe { guard.release() })
    }
}

// create_provider macro is now defined in macros.rs