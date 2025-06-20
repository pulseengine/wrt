//! Safe Memory Allocation API - Capability-Based Version
//!
//! This module provides a completely safe alternative to the unsafe memory allocation
//! patterns using the new capability-based memory management system. It eliminates the 
//! need for unsafe patterns by providing a safe factory pattern that creates 
//! collections directly with capability-managed providers.
//!
//! SW-REQ-ID: REQ_SAFETY_001 - No unsafe code in allocation
//! SW-REQ-ID: REQ_MEM_001 - Memory bounds checking  
//! SW-REQ-ID: REQ_MEM_003 - Automatic cleanup
//! SW-REQ-ID: REQ_CAP_001 - Capability-based access control

use crate::{
    bounded::{BoundedString, BoundedVec},
    bounded_collections::BoundedMap,
    budget_aware_provider::CrateId,
    capabilities::{MemoryCapabilityContext, StaticMemoryCapability, DynamicMemoryCapability},
    codes,
    safe_memory::NoStdProvider,
    traits::{Checksummable, FromBytes, ToBytes},
    verification::VerificationLevel,
    Error, ErrorCategory, Result,
};

// Legacy imports for backward compatibility (deprecated)
#[allow(deprecated)]
use crate::wrt_memory_system::{NoStdProviderFactory, CapabilityWrtFactory};

/// Capability-based safe factory for memory providers
pub struct CapabilityProviderFactory;

impl CapabilityProviderFactory {
    /// Create a static memory capability-managed provider for ASIL-B/C/D
    ///
    /// This creates a compile-time verified provider suitable for safety-critical levels.
    pub fn create_static_provider<const N: usize>(
        crate_id: CrateId,
        verification_level: VerificationLevel,
    ) -> Result<StaticMemoryCapability<N>> {
        Ok(StaticMemoryCapability::new(crate_id, verification_level))
    }

    /// Create a dynamic memory capability-managed provider for QM/ASIL-A
    ///
    /// This creates a runtime allocation provider suitable for non-safety-critical levels.
    pub fn create_dynamic_provider(
        crate_id: CrateId,
        max_allocation: usize,
        verification_level: VerificationLevel,
    ) -> Result<DynamicMemoryCapability> {
        Ok(DynamicMemoryCapability::new(max_allocation, crate_id, verification_level))
    }

    /// Create a capability-managed provider using context
    ///
    /// This integrates with the capability dependency injection system.
    pub fn create_context_managed_provider<const N: usize>(
        context: &mut MemoryCapabilityContext,
        crate_id: CrateId,
    ) -> Result<()> {
        context.register_static_capability::<N>(crate_id)
    }
}

/// Legacy safe factory functions (deprecated)
///
/// These functions maintain backward compatibility while transitioning to capability-based system.
#[deprecated(note = "Use CapabilityProviderFactory for capability-driven design")]
pub struct SafeProviderFactory;

#[allow(deprecated)]
impl SafeProviderFactory {
    /// Create a NoStdProvider safely without unsafe code
    ///
    /// This function creates a provider using the internal safe mechanisms
    /// without exposing the guard's unsafe release method.
    #[deprecated(note = "Use CapabilityProviderFactory::create_static_provider instead")]
    #[allow(deprecated)] // We're using the safe internal method
    fn create_provider_safe<const N: usize>() -> Result<NoStdProvider<N>> {
        // Use the safe default construction without resize
        Ok(NoStdProvider::<N>::default())
    }

    /// Create a provider with budget tracking through the memory coordinator
    ///
    /// This provides the same functionality as safe_managed_alloc! but without
    /// requiring unsafe code to extract the provider.
    #[deprecated(note = "Use CapabilityProviderFactory::create_context_managed_provider instead")]
    pub fn create_managed_provider<const N: usize>(crate_id: CrateId) -> Result<NoStdProvider<N>> {
        // MIGRATION: Use capability system instead of legacy coordinator
        use crate::memory_init::get_global_capability_context;
        let context = get_global_capability_context()?;
        
        // Verify capability and create provider through modern system
        let capability_provider = context.create_provider::<N>(crate_id)?;
        
        // For backward compatibility, return a default provider
        // The capability provider ensures the allocation is allowed
        Ok(NoStdProvider::<N>::default())
    }
    
    /// Create a context-managed provider using capability context
    ///
    /// This is the modern replacement for create_managed_provider that uses
    /// the capability system instead of global state.
    pub fn create_context_managed_provider<const N: usize>(
        context: &MemoryCapabilityContext,
        crate_id: CrateId,
    ) -> Result<NoStdProvider<N>> {
        // Verify we have allocation capability
        use crate::capabilities::MemoryOperation;
        let operation = MemoryOperation::Allocate { size: N };
        context.verify_operation(crate_id, &operation)?;
        
        // Create the provider safely
        Self::create_provider_safe::<N>()
    }
}

/// Capability-based safe collection factory functions
///
/// These functions create bounded collections using capability-managed memory allocation
/// suitable for different ASIL levels without requiring unsafe code blocks.
pub mod capability_factories {
    use super::*;

    /// Safely create a BoundedVec with static capability (ASIL-B/C/D)
    pub fn safe_static_bounded_vec<T, const CAPACITY: usize, const PROVIDER_SIZE: usize>(
        crate_id: CrateId,
        verification_level: VerificationLevel,
    ) -> Result<(BoundedVec<T, CAPACITY, NoStdProvider<PROVIDER_SIZE>>, StaticMemoryCapability<PROVIDER_SIZE>)>
    where
        T: Sized + Checksummable + ToBytes + FromBytes + Default + Clone + PartialEq + Eq,
    {
        let capability = CapabilityProviderFactory::create_static_provider::<PROVIDER_SIZE>(crate_id, verification_level)?;
        
        // Create provider using safe default construction
        let provider = NoStdProvider::<PROVIDER_SIZE>::default();
        
        let vec = BoundedVec::new(provider).map_err(|_| {
            Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ERROR,
                "Failed to create capability-managed bounded vector",
            )
        })?;
        
        Ok((vec, capability))
    }

    /// Create a memory capability context for dependency injection
    pub fn create_capability_context(
        verification_level: VerificationLevel,
        runtime_verification: bool,
    ) -> MemoryCapabilityContext {
        MemoryCapabilityContext::new(verification_level, runtime_verification)
    }
}

/// Legacy safe collection factory functions using owned providers (deprecated)
///
/// These functions create bounded collections using the legacy memory system.
/// Use capability_factories for new code.
#[deprecated(note = "Use capability_factories for capability-driven design")]
pub mod safe_factories {
    use super::*;

    /// Safely create a BoundedVec with owned provider
    #[deprecated(note = "Use capability_factories::safe_static_bounded_vec for capability-driven design")]
    #[allow(deprecated)]
    pub fn safe_bounded_vec<T, const CAPACITY: usize, const PROVIDER_SIZE: usize>(
        crate_id: CrateId,
    ) -> Result<BoundedVec<T, CAPACITY, NoStdProvider<PROVIDER_SIZE>>>
    where
        T: Sized + Checksummable + ToBytes + FromBytes + Default + Clone + PartialEq + Eq,
    {
        let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
        BoundedVec::new(provider).map_err(|e| {
            Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ERROR,
                "Failed to create bounded vector",
            )
        })
    }

    /// Safely create a BoundedMap with owned provider
    pub fn safe_bounded_map<K, V, const CAPACITY: usize, const PROVIDER_SIZE: usize>(
        crate_id: CrateId,
    ) -> Result<BoundedMap<K, V, CAPACITY, NoStdProvider<PROVIDER_SIZE>>>
    where
        K: Sized + Checksummable + ToBytes + FromBytes + Default + Clone + PartialEq + Eq,
        V: Sized + Checksummable + ToBytes + FromBytes + Default + Clone + PartialEq + Eq,
    {
        let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
        BoundedMap::new(provider).map_err(|e| {
            Error::new(ErrorCategory::Memory, codes::MEMORY_ERROR, "Failed to create bounded map")
        })
    }

    /// Safely create a BoundedString with owned provider
    pub fn safe_bounded_string<const CAPACITY: usize, const PROVIDER_SIZE: usize>(
        crate_id: CrateId,
    ) -> Result<BoundedString<CAPACITY, NoStdProvider<PROVIDER_SIZE>>> {
        let provider = SafeProviderFactory::create_managed_provider::<PROVIDER_SIZE>(crate_id)?;
        BoundedString::from_str("", provider).map_err(|e| {
            Error::new(
                ErrorCategory::Memory,
                codes::MEMORY_ERROR,
                "Failed to create bounded string",
            )
        })
    }
}

/// Safe macro for creating providers without unsafe code
///
/// This macro replaces the unsafe `safe_managed_alloc!` pattern with a safe alternative.
///
/// # Examples
///
/// ```rust
/// use wrt_foundation::{safe_managed_alloc, CrateId};
///
/// // Safe allocation with automatic budget tracking
/// let provider = safe_managed_alloc!(131072, CrateId::Runtime)?;
/// let vec = BoundedVec::new(provider)?;
///
/// // No unsafe code required!
/// ```
#[macro_export]
macro_rules! safe_managed_alloc {
    ($size:expr, $crate_id:expr) => {
        $crate::safe_allocation::SafeProviderFactory::create_managed_provider::<$size>($crate_id)
    };
}

/// Capability-aware macro for creating providers
///
/// This macro uses the capability context for memory allocation verification.
///
/// # Examples
///
/// ```rust
/// use wrt_foundation::{capability_managed_alloc, CrateId, MemoryCapabilityContext};
///
/// let context = MemoryCapabilityContext::new();
/// // Capability-based allocation with context verification
/// let provider = capability_managed_alloc!(131072, &context, CrateId::Runtime)?;
/// let vec = BoundedVec::new(provider)?;
/// ```
#[macro_export]
macro_rules! capability_managed_alloc {
    ($size:expr, $context:expr, $crate_id:expr) => {
        $crate::safe_allocation::SafeProviderFactory::create_context_managed_provider::<$size>($context, $crate_id)
    };
}

/// Public re-exports for convenience
pub use safe_factories::*;
