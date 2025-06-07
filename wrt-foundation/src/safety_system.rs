//! ASIL-Aware Safety Primitives for WRT Foundation
//!
//! This module provides safety primitives that are aware of Automotive Safety
//! Integrity Level (ASIL) requirements. It implements compile-time and runtime
//! safety checks that adapt to the required safety level.
//!
//! # ASIL Levels
//!
//! - **QM (Quality Management)**: No safety requirements
//! - **ASIL A**: Lowest safety integrity level  
//! - **ASIL B**: Low safety integrity level
//! - **ASIL C**: Medium safety integrity level
//! - **ASIL D**: Highest safety integrity level
//!
//! # Design Principles
//!
//! - **Compile-Time Safety**: Safety levels are known at compile time when possible
//! - **Runtime Adaptation**: Safety checks can be enhanced at runtime
//! - **Zero-Cost Abstractions**: Safety primitives add minimal overhead
//! - **Fail-Safe Design**: All operations fail safely when safety violations occur
//! - **Audit Trail**: Safety-critical operations are logged for verification
//!
//! # Usage
//!
//! ```rust
//! use wrt_foundation::safety_system::{SafetyContext, AsilLevel};
//!
//! // Compile-time safety context
//! const SAFETY_CTX: SafetyContext = SafetyContext::new(AsilLevel::AsilC);
//!
//! // Runtime safety adaptation
//! let mut runtime_ctx = SAFETY_CTX;
//! runtime_ctx.upgrade_runtime_asil(AsilLevel::AsilD)?;
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

use core::sync::atomic::{AtomicU8, Ordering};

use crate::{Error, ErrorCategory, WrtResult, codes};

#[cfg(feature = "std")]
use std::time::{SystemTime, UNIX_EPOCH};

/// Automotive Safety Integrity Level (ASIL) classification
///
/// ASIL levels define the safety requirements for automotive systems.
/// Higher levels require more rigorous safety measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum AsilLevel {
    /// Quality Management - No safety requirements
    QM = 0,
    /// ASIL A - Lowest safety integrity level
    AsilA = 1,
    /// ASIL B - Low safety integrity level  
    AsilB = 2,
    /// ASIL C - Medium safety integrity level
    AsilC = 3,
    /// ASIL D - Highest safety integrity level
    AsilD = 4,
}

impl AsilLevel {
    /// Get the string representation of the ASIL level
    pub const fn as_str(&self) -> &'static str {
        match self {
            AsilLevel::QM => "QM",
            AsilLevel::AsilA => "ASIL-A",
            AsilLevel::AsilB => "ASIL-B", 
            AsilLevel::AsilC => "ASIL-C",
            AsilLevel::AsilD => "ASIL-D",
        }
    }

    /// Check if this ASIL level requires memory protection
    pub const fn requires_memory_protection(&self) -> bool {
        matches!(self, AsilLevel::AsilC | AsilLevel::AsilD)
    }

    /// Check if this ASIL level requires runtime verification
    pub const fn requires_runtime_verification(&self) -> bool {
        matches!(self, AsilLevel::AsilB | AsilLevel::AsilC | AsilLevel::AsilD)
    }

    /// Check if this ASIL level requires control flow integrity
    pub const fn requires_cfi(&self) -> bool {
        matches!(self, AsilLevel::AsilC | AsilLevel::AsilD)
    }

    /// Check if this ASIL level requires redundant computation
    pub const fn requires_redundancy(&self) -> bool {
        matches!(self, AsilLevel::AsilD)
    }

    /// Get the required verification frequency for this ASIL level
    pub const fn verification_frequency(&self) -> u32 {
        match self {
            AsilLevel::QM => 0,
            AsilLevel::AsilA => 1000,  // Every 1000 operations
            AsilLevel::AsilB => 100,   // Every 100 operations
            AsilLevel::AsilC => 10,    // Every 10 operations
            AsilLevel::AsilD => 1,     // Every operation
        }
    }

    /// Get the maximum allowed error rate for this ASIL level
    pub const fn max_error_rate(&self) -> f64 {
        match self {
            AsilLevel::QM => 1.0,       // No limit
            AsilLevel::AsilA => 0.1,   // 10%
            AsilLevel::AsilB => 0.01,  // 1%
            AsilLevel::AsilC => 0.001, // 0.1%
            AsilLevel::AsilD => 0.0001, // 0.01%
        }
    }
}

impl Default for AsilLevel {
    fn default() -> Self {
        AsilLevel::QM
    }
}

impl core::fmt::Display for AsilLevel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Safety context that tracks ASIL requirements and safety state
///
/// This structure maintains both compile-time and runtime safety information,
/// allowing for adaptive safety behavior based on current requirements.
#[derive(Debug)]
pub struct SafetyContext {
    /// ASIL level determined at compile time
    pub compile_time_asil: AsilLevel,
    /// ASIL level that may be upgraded at runtime
    runtime_asil: AtomicU8,
    /// Number of safety violations detected
    violation_count: AtomicU8,
    /// Operation counter for periodic verification
    operation_count: AtomicU8,
}

impl Clone for SafetyContext {
    fn clone(&self) -> Self {
        Self {
            compile_time_asil: self.compile_time_asil,
            runtime_asil: AtomicU8::new(self.runtime_asil.load(Ordering::SeqCst)),
            violation_count: AtomicU8::new(self.violation_count.load(Ordering::SeqCst)),
            operation_count: AtomicU8::new(self.operation_count.load(Ordering::SeqCst)),
        }
    }
}

impl SafetyContext {
    /// Create a new safety context with compile-time ASIL level
    ///
    /// # Arguments
    ///
    /// * `compile_time` - The ASIL level known at compile time
    ///
    /// # Examples
    ///
    /// ```rust
    /// use wrt_foundation::safety_system::{SafetyContext, AsilLevel};
    ///
    /// const SAFETY_CTX: SafetyContext = SafetyContext::new(AsilLevel::AsilC);
    /// ```
    pub const fn new(compile_time: AsilLevel) -> Self {
        Self {
            compile_time_asil: compile_time,
            runtime_asil: AtomicU8::new(compile_time as u8),
            violation_count: AtomicU8::new(0),
            operation_count: AtomicU8::new(0),
        }
    }

    /// Get the effective ASIL level (highest of compile-time and runtime)
    ///
    /// # Returns
    ///
    /// The effective ASIL level currently in effect.
    pub fn effective_asil(&self) -> AsilLevel {
        let runtime_level = self.runtime_asil.load(Ordering::Acquire);
        let compile_level = self.compile_time_asil as u8;
        
        let effective_level = runtime_level.max(compile_level);
        
        // Safe to unwrap because we only store valid ASIL values
        match effective_level {
            0 => AsilLevel::QM,
            1 => AsilLevel::AsilA,
            2 => AsilLevel::AsilB,
            3 => AsilLevel::AsilC,
            4 => AsilLevel::AsilD,
            _ => AsilLevel::AsilD, // Default to highest safety level for invalid values
        }
    }

    /// Upgrade the runtime ASIL level
    ///
    /// This allows increasing the safety requirements at runtime, but not
    /// decreasing them below the compile-time level.
    ///
    /// # Arguments
    ///
    /// * `new_level` - The new ASIL level to set
    ///
    /// # Errors
    ///
    /// Returns an error if attempting to downgrade below compile-time level.
    pub fn upgrade_runtime_asil(&self, new_level: AsilLevel) -> WrtResult<()> {
        let new_level_u8 = new_level as u8;
        let compile_level_u8 = self.compile_time_asil as u8;
        
        if new_level_u8 < compile_level_u8 {
            return Err(Error::new(
                ErrorCategory::Safety,
                codes::SAFETY_VIOLATION,
                "Cannot downgrade ASIL below compile-time level",
            ));
        }
        
        self.runtime_asil.store(new_level_u8, Ordering::Release);
        Ok(())
    }

    /// Record a safety violation
    ///
    /// This increments the violation counter and may trigger safety actions
    /// based on the current ASIL level.
    ///
    /// # Returns
    ///
    /// The new violation count after incrementing.
    pub fn record_violation(&self) -> u8 {
        let count = self.violation_count.fetch_add(1, Ordering::AcqRel) + 1;
        
        // Trigger safety actions based on ASIL level
        let effective = self.effective_asil();
        match effective {
            AsilLevel::QM => {
                // No action required
            }
            AsilLevel::AsilA | AsilLevel::AsilB => {
                // Log violation for audit
                #[cfg(feature = "std")]
                {
                    eprintln!("Safety violation #{} detected at {}", count, effective);
                }
            }
            AsilLevel::AsilC | AsilLevel::AsilD => {
                // For high ASIL levels, consider immediate protective actions
                #[cfg(feature = "std")]
                {
                    eprintln!("CRITICAL: Safety violation #{} detected at {}", count, effective);
                }
                
                // In a real implementation, this might trigger:
                // - System shutdown
                // - Failsafe mode activation
                // - Error reporting to safety monitor
            }
        }
        
        count
    }

    /// Get the current violation count
    pub fn violation_count(&self) -> u8 {
        self.violation_count.load(Ordering::Acquire)
    }

    /// Check if periodic verification should be performed
    ///
    /// Based on the current ASIL level, this determines whether verification
    /// should be performed for the current operation.
    ///
    /// # Returns
    ///
    /// `true` if verification should be performed, `false` otherwise.
    pub fn should_verify(&self) -> bool {
        let effective = self.effective_asil();
        let frequency = effective.verification_frequency();
        
        if frequency == 0 {
            return false; // QM level - no verification required
        }
        
        let count = self.operation_count.fetch_add(1, Ordering::AcqRel) + 1;
        (count as u32) % frequency == 0
    }

    /// Reset the safety context (for testing or system restart)
    ///
    /// # Safety
    ///
    /// This should only be called during system initialization or controlled
    /// test scenarios.
    pub fn reset(&self) {
        self.runtime_asil.store(self.compile_time_asil as u8, Ordering::Release);
        self.violation_count.store(0, Ordering::Release);
        self.operation_count.store(0, Ordering::Release);
    }

    /// Check if the context is in a safe state
    ///
    /// A context is considered unsafe if it has too many violations relative
    /// to the ASIL requirements.
    pub fn is_safe(&self) -> bool {
        let violations = self.violation_count();
        let operations = self.operation_count.load(Ordering::Acquire);
        
        if operations == 0 {
            return true; // No operations yet
        }
        
        let error_rate = violations as f64 / operations as f64;
        let max_rate = self.effective_asil().max_error_rate();
        
        error_rate <= max_rate
    }
}

impl Default for SafetyContext {
    fn default() -> Self {
        Self::new(AsilLevel::default())
    }
}

/// Safety guard that ensures operations are performed within safety constraints
///
/// This guard automatically performs safety checks based on the current ASIL
/// level and can prevent unsafe operations from proceeding.
#[derive(Debug)]
pub struct SafetyGuard<'a> {
    context: &'a SafetyContext,
    operation_name: &'static str,
    #[cfg(feature = "std")]
    start_time: SystemTime,
}

impl<'a> SafetyGuard<'a> {
    /// Create a new safety guard for an operation
    ///
    /// # Arguments
    ///
    /// * `context` - The safety context to use
    /// * `operation_name` - Name of the operation for logging
    pub fn new(context: &'a SafetyContext, operation_name: &'static str) -> WrtResult<Self> {
        // Check if the context is in a safe state
        if !context.is_safe() {
            context.record_violation();
            return Err(Error::new(
                ErrorCategory::Safety,
                codes::SAFETY_VIOLATION,
                "Safety context is not in a safe state",
            ));
        }
        
        Ok(Self {
            context,
            operation_name,
            #[cfg(feature = "std")]
            start_time: SystemTime::now(),
        })
    }

    /// Get the safety context
    pub fn context(&self) -> &SafetyContext {
        self.context
    }

    /// Get the operation name
    pub fn operation_name(&self) -> &'static str {
        self.operation_name
    }

    /// Perform verification if required by the current ASIL level
    pub fn verify_if_required<F>(&self, verifier: F) -> WrtResult<()>
    where
        F: FnOnce() -> WrtResult<()>,
    {
        if self.context.should_verify() {
            verifier().map_err(|_| {
                self.context.record_violation();
                Error::new(
                    ErrorCategory::Safety,
                    codes::VERIFICATION_FAILED,
                    "Safety verification failed",
                )
            })?;
        }
        Ok(())
    }

    /// Complete the guarded operation successfully
    pub fn complete(self) -> WrtResult<()> {
        #[cfg(feature = "std")]
        {
            let duration = self.start_time.elapsed().unwrap_or_default();
            if self.context.effective_asil().requires_runtime_verification() {
                println!("Operation '{}' completed in {:?}", self.operation_name, duration);
            }
        }
        Ok(())
    }
}

impl<'a> Drop for SafetyGuard<'a> {
    fn drop(&mut self) {
        // If the guard is dropped without calling complete(), it's likely an error
        #[cfg(feature = "std")]
        {
            if std::thread::panicking() {
                self.context.record_violation();
                eprintln!("Safety guard for '{}' dropped during panic", self.operation_name);
            }
        }
        #[cfg(not(feature = "std"))]
        {
            // In no_std, we can't detect panicking, so we assume it might be an error
            // This is a conservative approach for safety-critical environments
            self.context.record_violation();
        }
    }
}

/// Safety-aware memory allocation wrapper
///
/// This wrapper ensures that memory allocations are performed according to
/// the current ASIL requirements, including verification and protection.
#[derive(Debug)]
pub struct SafeMemoryAllocation<'a> {
    data: &'a mut [u8],
    context: &'a SafetyContext,
    checksum: u32,
}

impl<'a> SafeMemoryAllocation<'a> {
    /// Create a new safe memory allocation
    ///
    /// # Arguments
    ///
    /// * `data` - The allocated memory slice
    /// * `context` - The safety context for verification
    pub fn new(data: &'a mut [u8], context: &'a SafetyContext) -> WrtResult<Self> {
        let checksum = Self::calculate_checksum(data);
        
        Ok(Self {
            data,
            context,
            checksum,
        })
    }

    /// Calculate checksum for memory protection
    fn calculate_checksum(data: &[u8]) -> u32 {
        data.iter().fold(0u32, |acc, &byte| {
            acc.wrapping_add(byte as u32)
        })
    }

    /// Verify memory integrity
    pub fn verify_integrity(&self) -> WrtResult<()> {
        if self.context.effective_asil().requires_memory_protection() {
            let current_checksum = Self::calculate_checksum(self.data);
            if current_checksum != self.checksum {
                self.context.record_violation();
                return Err(Error::new(
                    ErrorCategory::Safety,
                    codes::MEMORY_CORRUPTION_DETECTED,
                    "Memory corruption detected",
                ));
            }
        }
        Ok(())
    }

    /// Get access to the underlying data
    pub fn data(&self) -> &[u8] {
        self.data
    }

    /// Get mutable access to the underlying data
    pub fn data_mut(&mut self) -> WrtResult<&mut [u8]> {
        self.verify_integrity()?;
        Ok(self.data)
    }

    /// Update the checksum after modifying data
    pub fn update_checksum(&mut self) {
        if self.context.effective_asil().requires_memory_protection() {
            self.checksum = Self::calculate_checksum(self.data);
        }
    }
}

/// Macro for creating compile-time safety contexts
///
/// This macro ensures that safety contexts are created with the correct
/// ASIL level at compile time.
#[macro_export]
macro_rules! safety_context {
    (QM) => {
        $crate::safety_system::SafetyContext::new($crate::safety_system::AsilLevel::QM)
    };
    (AsilA) => {
        $crate::safety_system::SafetyContext::new($crate::safety_system::AsilLevel::AsilA)
    };
    (AsilB) => {
        $crate::safety_system::SafetyContext::new($crate::safety_system::AsilLevel::AsilB)
    };
    (AsilC) => {
        $crate::safety_system::SafetyContext::new($crate::safety_system::AsilLevel::AsilC)
    };
    (AsilD) => {
        $crate::safety_system::SafetyContext::new($crate::safety_system::AsilLevel::AsilD)
    };
}

/// Macro for performing safety-guarded operations
///
/// This macro automatically creates a safety guard and ensures proper
/// cleanup even if the operation fails.
#[macro_export]
macro_rules! safety_guarded {
    ($context:expr, $operation:expr, $block:block) => {{
        let guard = $crate::safety_system::SafetyGuard::new($context, $operation)?;
        let result = $block;
        guard.complete()?;
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asil_level_ordering() {
        assert!(AsilLevel::QM < AsilLevel::AsilA);
        assert!(AsilLevel::AsilA < AsilLevel::AsilB);
        assert!(AsilLevel::AsilB < AsilLevel::AsilC);
        assert!(AsilLevel::AsilC < AsilLevel::AsilD);
    }

    #[test]
    fn test_asil_level_properties() {
        assert!(!AsilLevel::QM.requires_memory_protection());
        assert!(!AsilLevel::AsilA.requires_memory_protection());
        assert!(!AsilLevel::AsilB.requires_memory_protection());
        assert!(AsilLevel::AsilC.requires_memory_protection());
        assert!(AsilLevel::AsilD.requires_memory_protection());

        assert!(!AsilLevel::QM.requires_cfi());
        assert!(!AsilLevel::AsilA.requires_cfi());
        assert!(!AsilLevel::AsilB.requires_cfi());
        assert!(AsilLevel::AsilC.requires_cfi());
        assert!(AsilLevel::AsilD.requires_cfi());

        assert!(!AsilLevel::QM.requires_redundancy());
        assert!(!AsilLevel::AsilA.requires_redundancy());
        assert!(!AsilLevel::AsilB.requires_redundancy());
        assert!(!AsilLevel::AsilC.requires_redundancy());
        assert!(AsilLevel::AsilD.requires_redundancy());
    }

    #[test]
    fn test_safety_context_creation() {
        let ctx = SafetyContext::new(AsilLevel::AsilC);
        assert_eq!(ctx.compile_time_asil, AsilLevel::AsilC);
        assert_eq!(ctx.effective_asil(), AsilLevel::AsilC);
        assert_eq!(ctx.violation_count(), 0);
    }

    #[test]
    fn test_safety_context_upgrade() {
        let ctx = SafetyContext::new(AsilLevel::AsilB);
        
        // Should be able to upgrade
        assert!(ctx.upgrade_runtime_asil(AsilLevel::AsilD).is_ok());
        assert_eq!(ctx.effective_asil(), AsilLevel::AsilD);
        
        // Should not be able to downgrade below compile-time level
        assert!(ctx.upgrade_runtime_asil(AsilLevel::AsilA).is_err());
        assert_eq!(ctx.effective_asil(), AsilLevel::AsilD); // Should remain unchanged
    }

    #[test]
    fn test_safety_context_violations() {
        let ctx = SafetyContext::new(AsilLevel::AsilA);
        
        assert_eq!(ctx.violation_count(), 0);
        assert!(ctx.is_safe());
        
        let count1 = ctx.record_violation();
        assert_eq!(count1, 1);
        assert_eq!(ctx.violation_count(), 1);
        
        let count2 = ctx.record_violation();
        assert_eq!(count2, 2);
        assert_eq!(ctx.violation_count(), 2);
    }

    #[test]
    fn test_safety_context_verification() {
        let ctx = SafetyContext::new(AsilLevel::AsilD);
        
        // AsilD requires verification every operation
        assert!(ctx.should_verify());
        assert!(ctx.should_verify());
        assert!(ctx.should_verify());
        
        let ctx_qm = SafetyContext::new(AsilLevel::QM);
        
        // QM requires no verification
        assert!(!ctx_qm.should_verify());
        assert!(!ctx_qm.should_verify());
        assert!(!ctx_qm.should_verify());
    }

    #[test]
    fn test_safety_guard() -> WrtResult<()> {
        let ctx = SafetyContext::new(AsilLevel::AsilB);
        
        let guard = SafetyGuard::new(&ctx, "test_operation")?;
        assert_eq!(guard.operation_name(), "test_operation");
        
        // Verify that verification works
        guard.verify_if_required(|| Ok(()))?;
        
        guard.complete()?;
        Ok(())
    }

    #[test]
    fn test_safe_memory_allocation() -> WrtResult<()> {
        let ctx = SafetyContext::new(AsilLevel::AsilC);
        let mut data = [1u8, 2u8, 3u8, 4u8];
        
        let mut allocation = SafeMemoryAllocation::new(&mut data, &ctx)?;
        
        // Should verify successfully initially
        allocation.verify_integrity()?;
        
        // Modify data and update checksum
        {
            let data_mut = allocation.data_mut()?;
            data_mut[0] = 10;
        }
        allocation.update_checksum();
        
        // Should still verify successfully
        allocation.verify_integrity()?;
        
        Ok(())
    }

    #[test]
    fn test_safety_context_macro() {
        let ctx = safety_context!(AsilC);
        assert_eq!(ctx.effective_asil(), AsilLevel::AsilC);
    }

    #[test]
    fn test_safety_guarded_macro() -> WrtResult<()> {
        let ctx = SafetyContext::new(AsilLevel::AsilA);
        
        let result = safety_guarded!(&ctx, "test_macro_operation", {
            42
        });
        
        assert_eq!(result, 42);
        Ok(())
    }

    #[test]
    fn test_asil_level_display() {
        assert_eq!(format!("{}", AsilLevel::QM), "QM");
        assert_eq!(format!("{}", AsilLevel::AsilA), "ASIL-A");
        assert_eq!(format!("{}", AsilLevel::AsilB), "ASIL-B");
        assert_eq!(format!("{}", AsilLevel::AsilC), "ASIL-C");
        assert_eq!(format!("{}", AsilLevel::AsilD), "ASIL-D");
    }
}