//! Budget-Aware Memory Provider Wrapper
//!
//! This module provides a wrapper around NoStdProvider that enforces
//! memory budget tracking and automatic cleanup via RAII.

use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicU32, Ordering};

use crate::{
    safe_memory::{NoStdProvider, Provider, Stats as MemoryStats},
    budget_aware_provider::{BudgetAwareProviderFactory, CrateId},
    WrtResult, Error, ErrorCategory, codes,
};

/// Unique allocation ID generator
static NEXT_ALLOCATION_ID: AtomicU32 = AtomicU32::new(1);

/// Budget-aware wrapper around NoStdProvider
/// 
/// This type automatically tracks memory allocations and returns
/// the memory to the budget when dropped (RAII pattern).
#[derive(Debug)]
pub struct BudgetProvider<const N: usize> {
    /// The underlying provider
    inner: NoStdProvider<N>,
    /// Unique allocation ID
    allocation_id: u32,
    /// Crate that owns this allocation
    crate_id: CrateId,
    /// Whether this allocation is still active
    active: bool,
}

impl<const N: usize> BudgetProvider<N> {
    /// Create a new budget-aware provider
    /// 
    /// This is the only way to create a provider that properly tracks budgets.
    /// The allocation will be automatically returned to the budget when dropped.
    pub fn new(crate_id: CrateId) -> WrtResult<Self> {
        // Use the factory to ensure budget compliance
        let inner = BudgetAwareProviderFactory::create_provider::<N>(crate_id)?;
        let allocation_id = NEXT_ALLOCATION_ID.fetch_add(1, Ordering::AcqRel);
        
        Ok(Self {
            inner,
            allocation_id,
            crate_id,
            active: true,
        })
    }
    
    /// Create a shared provider from the pool
    pub fn new_shared(crate_id: CrateId) -> WrtResult<Self> {
        // Use the factory to get from shared pool
        let inner = BudgetAwareProviderFactory::create_shared_provider::<N>(crate_id)?;
        let allocation_id = NEXT_ALLOCATION_ID.fetch_add(1, Ordering::AcqRel);
        
        Ok(Self {
            inner,
            allocation_id,
            crate_id,
            active: true,
        })
    }
    
    /// Get the allocation ID
    pub fn allocation_id(&self) -> u32 {
        self.allocation_id
    }
    
    /// Get the crate ID
    pub fn crate_id(&self) -> CrateId {
        self.crate_id
    }
    
    /// Check if the allocation is still active
    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl<const N: usize> Drop for BudgetProvider<N> {
    fn drop(&mut self) {
        if self.active {
            // Return the memory to the budget
            if let Err(e) = BudgetAwareProviderFactory::return_allocation(
                self.crate_id,
                self.allocation_id,
                N
            ) {
                // In no_std, we can't panic or print, so we'll increment an error counter
                #[cfg(feature = "std")]
                eprintln!("Failed to return memory allocation {}: {:?}", self.allocation_id, e);
            }
            self.active = false;
        }
    }
}

// Implement Deref to allow transparent access to the inner provider
impl<const N: usize> Deref for BudgetProvider<N> {
    type Target = NoStdProvider<N>;
    
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<const N: usize> DerefMut for BudgetProvider<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// Implement Provider trait by delegating to inner
impl<const N: usize> Provider for BudgetProvider<N> {
    type Allocator = <NoStdProvider<N> as Provider>::Allocator;
    
    fn borrow_slice(&self, offset: usize, len: usize) -> crate::safe_memory::Result<crate::safe_memory::Slice<'_>> {
        self.inner.borrow_slice(offset, len)
    }
    
    fn write_data(&mut self, offset: usize, data: &[u8]) -> crate::safe_memory::Result<()> {
        self.inner.write_data(offset, data)
    }
    
    fn verify_access(&self, offset: usize, len: usize) -> crate::safe_memory::Result<()> {
        self.inner.verify_access(offset, len)
    }
    
    fn size(&self) -> usize {
        self.inner.size()
    }
    
    fn capacity(&self) -> usize {
        self.inner.capacity()
    }
    
    fn verify_integrity(&self) -> crate::safe_memory::Result<()> {
        self.inner.verify_integrity()
    }
    
    fn set_verification_level(&mut self, level: crate::verification::VerificationLevel) {
        self.inner.set_verification_level(level)
    }
    
    fn verification_level(&self) -> crate::verification::VerificationLevel {
        self.inner.verification_level()
    }
    
    fn memory_stats(&self) -> MemoryStats {
        self.inner.memory_stats()
    }
    
    fn get_slice_mut(&mut self, offset: usize, len: usize) -> crate::safe_memory::Result<crate::safe_memory::SliceMut<'_>> {
        self.inner.get_slice_mut(offset, len)
    }
    
    fn copy_within(&mut self, src_offset: usize, dst_offset: usize, len: usize) -> crate::safe_memory::Result<()> {
        self.inner.copy_within(src_offset, dst_offset, len)
    }
    
    fn ensure_used_up_to(&mut self, byte_offset: usize) -> crate::safe_memory::Result<()> {
        self.inner.ensure_used_up_to(byte_offset)
    }
    
    fn acquire_memory(&self, layout: core::alloc::Layout) -> WrtResult<*mut u8> {
        self.inner.acquire_memory(layout)
    }
}

// Implement Clone manually to track new allocations
impl<const N: usize> Clone for BudgetProvider<N> {
    fn clone(&self) -> Self {
        // Create a new provider through the factory to ensure budget tracking
        BudgetProvider::new(self.crate_id)
            .expect("Failed to clone BudgetProvider - budget exceeded")
    }
}

// Implement Default only for specific crates to prevent misuse
impl<const N: usize> Default for BudgetProvider<N> {
    /// Creates a default provider for the Foundation crate
    /// 
    /// # Panics
    /// Panics if the memory system is not initialized or budget is exceeded
    fn default() -> Self {
        BudgetProvider::new(CrateId::Foundation)
            .expect("Failed to create default BudgetProvider - ensure memory system is initialized")
    }
}

/// Type alias for backward compatibility during migration
pub type SafeProvider<const N: usize> = BudgetProvider<N>;

/// Budget-aware type aliases for common sizes
pub mod budget_types {
    use super::*;
    use crate::bounded::{BoundedVec, BoundedString};
    use crate::bounded_collections::BoundedMap;

    // Small providers (4KB)
    pub type SmallProvider = BudgetProvider<4096>;
    pub type SmallVec<T, const N: usize> = BoundedVec<T, N, SmallProvider>;
    pub type SmallString<const N: usize> = BoundedString<N, SmallProvider>;
    
    // Medium providers (64KB)
    pub type MediumProvider = BudgetProvider<65536>;
    pub type MediumVec<T, const N: usize> = BoundedVec<T, N, MediumProvider>;
    pub type MediumString<const N: usize> = BoundedString<N, MediumProvider>;
    pub type MediumMap<K, V, const N: usize> = BoundedMap<K, V, N, MediumProvider>;
    
    // Large providers (1MB)
    pub type LargeProvider = BudgetProvider<1048576>;
    pub type LargeVec<T, const N: usize> = BoundedVec<T, N, LargeProvider>;
    pub type LargeString<const N: usize> = BoundedString<N, LargeProvider>;
    pub type LargeMap<K, V, const N: usize> = BoundedMap<K, V, N, LargeProvider>;
}

/// Create a budget-aware provider for the current crate
/// 
/// This macro automatically detects the current crate and creates
/// an appropriately sized provider.
#[macro_export]
macro_rules! create_provider {
    ($size:expr) => {{
        use $crate::budget_provider::BudgetProvider;
        use $crate::provider_helpers::detect_current_crate;
        
        let crate_id = detect_current_crate();
        BudgetProvider::<$size>::new(crate_id)
    }};
    ($size:expr, $crate_id:expr) => {{
        use $crate::budget_provider::BudgetProvider;
        
        BudgetProvider::<$size>::new($crate_id)
    }};
}

/// Create a shared provider from the pool
#[macro_export]
macro_rules! create_shared_provider {
    ($size:expr) => {{
        use $crate::budget_provider::BudgetProvider;
        use $crate::provider_helpers::detect_current_crate;
        
        let crate_id = detect_current_crate();
        BudgetProvider::<$size>::new_shared(crate_id)
    }};
    ($size:expr, $crate_id:expr) => {{
        use $crate::budget_provider::BudgetProvider;
        
        BudgetProvider::<$size>::new_shared($crate_id)
    }};
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_budget_provider_drop() {
        // Initialize memory system for tests
        let _ = crate::memory_system_initializer::presets::test();
        
        // Create a provider
        let provider = BudgetProvider::<1024>::new(CrateId::Foundation).unwrap();
        let id = provider.allocation_id();
        assert!(provider.is_active());
        
        // Drop the provider
        drop(provider);
        
        // The allocation should have been returned
        // (In a real test, we'd check the budget tracker)
    }
    
    #[test]
    fn test_provider_macros() {
        let _ = crate::memory_system_initializer::presets::test();
        
        // Test the macros
        let _small = small_provider!().unwrap();
        let _medium = medium_provider!(CrateId::Runtime).unwrap();
        let _large = large_provider!().unwrap();
    }
    
    #[test]
    fn test_provider_deref() {
        let _ = crate::memory_system_initializer::presets::test();
        
        let provider = BudgetProvider::<1024>::new(CrateId::Foundation).unwrap();
        
        // Should be able to use provider methods through deref
        assert_eq!(provider.capacity(), 1024);
        assert_eq!(provider.allocated_memory(), 0);
    }
}