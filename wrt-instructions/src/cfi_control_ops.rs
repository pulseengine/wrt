// WRT - wrt-instructions
// Module: CFI-Aware Control Flow Operations
// SW-REQ-ID: REQ_CFI_CONTROL_001
//
// Copyright (c) 2025 The WRT Project Developers
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! CFI-Aware Control Flow Operations for WebAssembly Instructions
//!
//! This module extends the basic control flow operations with Control Flow
//! Integrity support, providing protection against ROP/JOP attacks through
//! hardware and software CFI mechanisms.
//!
//! # Key Features
//! - Landing pad validation for indirect calls
//! - Shadow stack management for returns
//! - Branch target verification
//! - Hardware-specific CFI instruction integration
//! - Software CFI fallback support

#![allow(dead_code)] // Allow during development

// Remove unused imports

use crate::prelude::*;
use crate::control_ops::BranchTarget;

/// CFI-enhanced control flow protection configuration
#[derive(Debug, Clone)]
pub struct CfiControlFlowProtection {
    /// Enable CFI protection
    pub enabled: bool,
    /// CFI protection level
    pub protection_level: CfiProtectionLevel,
    /// Hardware CFI configuration
    pub hardware_config: Option<HardwareCfiConfig>,
    /// Software CFI fallback configuration
    pub software_config: SoftwareCfiConfig,
}

impl Default for CfiControlFlowProtection {
    fn default() -> Self {
        Self {
            enabled: true,
            protection_level: CfiProtectionLevel::Enhanced,
            hardware_config: None, // Will be detected at runtime
            software_config: SoftwareCfiConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CfiProtectionLevel {
    /// No CFI protection
    None,
    /// Basic CFI (landing pads only)
    Basic,
    /// Enhanced CFI (shadow stack + landing pads)
    Enhanced,
    /// Maximum CFI (all protections + temporal validation)
    Maximum,
}

/// Hardware-specific CFI configuration
#[derive(Debug, Clone)]
pub struct HardwareCfiConfig {
    /// Architecture type
    pub architecture: CfiArchitecture,
    /// Hardware-specific settings
    pub settings: HardwareCfiSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CfiArchitecture {
    /// ARM with BTI support
    ArmBti,
    /// RISC-V with CFI extension
    RiscVCfi,
    /// x86_64 with CET support
    X86Cet,
}

#[derive(Debug, Clone)]
pub enum HardwareCfiSettings {
    /// ARM BTI settings
    ArmBti {
        /// BTI mode
        mode: ArmBtiMode,
        /// Exception level
        exception_level: ArmBtiExceptionLevel,
    },
    /// RISC-V CFI settings
    RiscVCfi {
        /// Shadow stack enabled
        shadow_stack: bool,
        /// Landing pads enabled
        landing_pads: bool,
    },
    /// x86_64 CET settings
    X86Cet {
        /// Shadow stack enabled
        shadow_stack: bool,
        /// Indirect branch tracking
        indirect_branch_tracking: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmBtiMode {
    /// Standard BTI (bti)
    Standard,
    /// Call-specific BTI (bti c)
    CallOnly,
    /// Jump-specific BTI (bti j)
    JumpOnly,
    /// Both call and jump BTI (bti jc)
    CallAndJump,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmBtiExceptionLevel {
    /// User mode (EL0) only
    EL0,
    /// Kernel mode (EL1) only
    EL1,
    /// Both user and kernel modes
    Both,
}

/// Software CFI configuration
#[derive(Debug, Clone)]
pub struct SoftwareCfiConfig {
    /// Enable software shadow stack
    pub shadow_stack_enabled: bool,
    /// Maximum shadow stack depth
    pub max_shadow_stack_depth: usize,
    /// Enable landing pad simulation
    pub landing_pad_simulation: bool,
    /// Enable temporal validation
    pub temporal_validation: bool,
    /// Maximum function execution time (cycles)
    pub max_function_execution_time: u64,
}

impl Default for SoftwareCfiConfig {
    fn default() -> Self {
        Self {
            shadow_stack_enabled: true,
            max_shadow_stack_depth: 1024,
            landing_pad_simulation: true,
            temporal_validation: false, // Expensive, off by default
            max_function_execution_time: 1000000, // 1M cycles
        }
    }
}

/// CFI-enhanced branch target with protection metadata
#[derive(Debug, Clone)]
pub struct CfiProtectedBranchTarget {
    /// Base branch target information
    pub target: BranchTarget,
    /// CFI protection requirements
    pub protection: CfiTargetProtection,
    /// Validation requirements
    pub validation: Vec<CfiValidationRequirement>,
}

/// CFI protection for a specific control flow target
#[derive(Debug, Clone)]
pub struct CfiTargetProtection {
    /// Target type for CFI classification
    pub target_type: CfiTargetType,
    /// Required landing pad information
    pub landing_pad: Option<CfiLandingPad>,
    /// Shadow stack requirements
    pub shadow_stack_requirement: ShadowStackRequirement,
    /// Temporal validation settings
    pub temporal_validation: Option<TemporalValidation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CfiTargetType {
    /// Direct function call
    DirectCall,
    /// Indirect function call (via table)
    IndirectCall,
    /// Return from function
    Return,
    /// Branch within function
    Branch,
    /// Block/loop entry
    BlockEntry,
    /// Function entry point
    FunctionEntry,
}

/// CFI landing pad information
#[derive(Debug, Clone)]
pub struct CfiLandingPad {
    /// Landing pad identifier
    pub pad_id: u32,
    /// Hardware instruction to insert
    pub hardware_instruction: Option<CfiHardwareInstruction>,
    /// Software validation code
    pub software_validation: Option<CfiSoftwareValidation>,
    /// Expected predecessor types
    pub valid_predecessors: Vec<CfiTargetType>,
}

/// Hardware-specific CFI instructions
#[derive(Debug, Clone)]
pub enum CfiHardwareInstruction {
    /// ARM BTI instruction
    #[cfg(target_arch = "aarch64")]
    ArmBti { mode: ArmBtiMode },
    /// RISC-V landing pad instruction
    #[cfg(target_arch = "riscv64")]
    RiscVLandingPad { label: u32 },
    /// x86_64 CET instruction
    #[cfg(target_arch = "x86_64")]
    X86Endbr,
}

/// Software CFI validation code
#[derive(Debug, Clone)]
pub struct CfiSoftwareValidation {
    /// Validation check ID
    pub check_id: u32,
    /// Expected values to validate
    pub expected_values: Vec<CfiExpectedValue>,
    /// Validation function
    pub validation_function: SoftwareCfiFunction,
}

#[derive(Debug, Clone)]
pub enum CfiExpectedValue {
    /// Expected function signature hash
    FunctionSignatureHash(u64),
    /// Expected return address
    ReturnAddress(u64),
    /// Expected call site identifier
    CallSiteId(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoftwareCfiFunction {
    /// Validate function signature
    ValidateSignature,
    /// Validate return address
    ValidateReturnAddress,
    /// Validate call site
    ValidateCallSite,
    /// Validate branch target
    ValidateBranchTarget,
}

/// Shadow stack requirements
#[derive(Debug, Clone)]
pub enum ShadowStackRequirement {
    /// No shadow stack requirement
    None,
    /// Push return address to shadow stack
    Push { return_address: u64, function_signature: u64 },
    /// Pop and validate return address from shadow stack
    PopAndValidate { expected_address: u64 },
    /// Check shadow stack without modifying
    Check,
}

/// Temporal validation for CFI
#[derive(Debug, Clone)]
pub struct TemporalValidation {
    /// Maximum execution time allowed
    pub max_execution_time: u64,
    /// Timestamp when execution started
    pub start_timestamp: Option<u64>,
    /// Deadline for completion
    pub deadline: Option<u64>,
}

/// CFI validation requirements
#[derive(Debug, Clone)]
pub enum CfiValidationRequirement {
    /// Validate function signature matches expected
    TypeSignatureCheck { expected_type_index: u32, signature_hash: u64 },
    /// Validate return address matches shadow stack
    ShadowStackCheck,
    /// Validate control flow target is valid
    ControlFlowTargetCheck { valid_targets: Vec<u32> },
    /// Validate calling convention
    CallingConventionCheck,
    /// Validate temporal properties
    TemporalCheck { max_duration: u64 },
}

/// CFI-aware control flow operations trait
pub trait CfiControlFlowOps {
    /// Execute indirect call with CFI protection
    fn call_indirect_with_cfi(
        &self,
        type_idx: u32,
        table_idx: u32,
        cfi_protection: &CfiControlFlowProtection,
        context: &mut CfiExecutionContext,
    ) -> Result<CfiProtectedBranchTarget>;

    /// Execute return with CFI protection
    fn return_with_cfi(
        &self,
        cfi_protection: &CfiControlFlowProtection,
        context: &mut CfiExecutionContext,
    ) -> Result<()>;

    /// Execute branch with CFI protection
    fn branch_with_cfi(
        &self,
        label_idx: u32,
        conditional: bool,
        cfi_protection: &CfiControlFlowProtection,
        context: &mut CfiExecutionContext,
    ) -> Result<CfiProtectedBranchTarget>;

    /// Insert CFI landing pad
    fn insert_cfi_landing_pad(
        &self,
        target_type: CfiTargetType,
        cfi_protection: &CfiControlFlowProtection,
        context: &mut CfiExecutionContext,
    ) -> Result<CfiLandingPad>;

    /// Validate CFI requirements
    fn validate_cfi_requirements(
        &self,
        requirements: &[CfiValidationRequirement],
        context: &CfiExecutionContext,
    ) -> Result<()>;
}

/// CFI execution context for tracking state
#[derive(Debug, Clone)]
pub struct CfiExecutionContext {
    /// Current function index
    pub current_function: u32,
    /// Current instruction offset
    pub current_instruction: u32,
    /// Software shadow stack
    pub shadow_stack: Vec<ShadowStackEntry>,
    /// Active landing pad expectations
    pub landing_pad_expectations: Vec<LandingPadExpectation>,
    /// CFI violation count
    pub violation_count: u32,
    /// Performance metrics
    pub metrics: CfiMetrics,
}

impl Default for CfiExecutionContext {
    fn default() -> Self {
        Self {
            current_function: 0,
            current_instruction: 0,
            shadow_stack: Vec::new(),
            landing_pad_expectations: Vec::new(),
            violation_count: 0,
            metrics: CfiMetrics::default(),
        }
    }
}

/// Shadow stack entry for software CFI
#[derive(Debug, Clone)]
pub struct ShadowStackEntry {
    /// Return address (function index, instruction offset)
    pub return_address: (u32, u32),
    /// Function signature hash for validation
    pub signature_hash: u64,
    /// Timestamp when call was made
    pub timestamp: u64,
    /// Call site metadata
    pub call_site_id: u32,
}

/// Landing pad expectation for CFI validation
#[derive(Debug, Clone)]
pub struct LandingPadExpectation {
    /// Expected function index
    pub function_index: u32,
    /// Expected instruction offset
    pub instruction_offset: u32,
    /// Expected target type
    pub target_type: CfiTargetType,
    /// Deadline for landing pad (for timeout detection)
    pub deadline: Option<u64>,
    /// Associated metadata
    pub metadata: CfiLandingPad,
}

/// CFI performance metrics
#[derive(Debug, Clone, Default)]
pub struct CfiMetrics {
    /// Total indirect calls protected
    pub indirect_calls_protected: u64,
    /// Total returns protected
    pub returns_protected: u64,
    /// Total branches protected
    pub branches_protected: u64,
    /// Total CFI violations detected
    pub violations_detected: u64,
    /// Total CFI overhead (nanoseconds)
    pub total_overhead_ns: u64,
    /// Landing pads validated
    pub landing_pads_validated: u64,
    /// Shadow stack operations
    pub shadow_stack_operations: u64,
}

/// Default implementation of CFI control flow operations
pub struct DefaultCfiControlFlowOps;

impl CfiControlFlowOps for DefaultCfiControlFlowOps {
    fn call_indirect_with_cfi(
        &self,
        type_idx: u32,
        table_idx: u32,
        cfi_protection: &CfiControlFlowProtection,
        context: &mut CfiExecutionContext,
    ) -> Result<CfiProtectedBranchTarget> {
        if !cfi_protection.enabled {
            // CFI disabled, use regular branch target
            return Ok(CfiProtectedBranchTarget {
                target: BranchTarget {
                    label_idx: table_idx,
                    keep_values: 0, // Will be determined by type signature
                },
                protection: CfiTargetProtection {
                    target_type: CfiTargetType::IndirectCall,
                    landing_pad: None,
                    shadow_stack_requirement: ShadowStackRequirement::None,
                    temporal_validation: None,
                },
                validation: Vec::new(),
            });
        }

        // Create CFI-protected branch target
        let landing_pad =
            self.create_landing_pad_for_indirect_call(type_idx, cfi_protection, context)?;

        let shadow_stack_requirement = if matches!(
            cfi_protection.protection_level,
            CfiProtectionLevel::Enhanced | CfiProtectionLevel::Maximum
        ) {
            ShadowStackRequirement::Push {
                return_address: self.compute_return_address(context),
                function_signature: self.compute_signature_hash(type_idx),
            }
        } else {
            ShadowStackRequirement::None
        };

        let temporal_validation =
            if matches!(cfi_protection.protection_level, CfiProtectionLevel::Maximum) {
                Some(TemporalValidation {
                    max_execution_time: cfi_protection.software_config.max_function_execution_time,
                    start_timestamp: Some(self.get_current_timestamp()),
                    deadline: Some(
                        self.get_current_timestamp()
                            + cfi_protection.software_config.max_function_execution_time,
                    ),
                })
            } else {
                None
            };

        // Create validation requirements
        let validation_requirements = vec![
            CfiValidationRequirement::TypeSignatureCheck {
                expected_type_index: type_idx,
                signature_hash: self.compute_signature_hash(type_idx),
            },
            CfiValidationRequirement::ControlFlowTargetCheck {
                valid_targets: vec![table_idx], // Table entry validation
            },
        ];

        // Update metrics
        context.metrics.indirect_calls_protected += 1;

        Ok(CfiProtectedBranchTarget {
            target: BranchTarget {
                label_idx: table_idx,
                keep_values: 0, // Determined by function signature
            },
            protection: CfiTargetProtection {
                target_type: CfiTargetType::IndirectCall,
                landing_pad: Some(landing_pad),
                shadow_stack_requirement,
                temporal_validation,
            },
            validation: validation_requirements,
        })
    }

    fn return_with_cfi(
        &self,
        cfi_protection: &CfiControlFlowProtection,
        context: &mut CfiExecutionContext,
    ) -> Result<()> {
        if !cfi_protection.enabled {
            return Ok(());
        }

        // Validate shadow stack for enhanced/maximum protection
        if matches!(
            cfi_protection.protection_level,
            CfiProtectionLevel::Enhanced | CfiProtectionLevel::Maximum
        ) {
            self.validate_shadow_stack_return(context)?;
        }

        // Update metrics
        context.metrics.returns_protected += 1;

        Ok(())
    }

    fn branch_with_cfi(
        &self,
        label_idx: u32,
        conditional: bool,
        cfi_protection: &CfiControlFlowProtection,
        context: &mut CfiExecutionContext,
    ) -> Result<CfiProtectedBranchTarget> {
        if !cfi_protection.enabled {
            return Ok(CfiProtectedBranchTarget {
                target: BranchTarget { label_idx, keep_values: 0 },
                protection: CfiTargetProtection {
                    target_type: CfiTargetType::Branch,
                    landing_pad: None,
                    shadow_stack_requirement: ShadowStackRequirement::None,
                    temporal_validation: None,
                },
                validation: Vec::new(),
            });
        }

        // Create branch target validation
        let target_offset = self.resolve_label_to_offset(label_idx, context)?;

        let validation_requirements = vec![CfiValidationRequirement::ControlFlowTargetCheck {
            valid_targets: vec![target_offset],
        }];

        // Update metrics
        context.metrics.branches_protected += 1;

        Ok(CfiProtectedBranchTarget {
            target: BranchTarget { label_idx, keep_values: 0 },
            protection: CfiTargetProtection {
                target_type: CfiTargetType::Branch,
                landing_pad: None,
                shadow_stack_requirement: ShadowStackRequirement::None,
                temporal_validation: None,
            },
            validation: validation_requirements,
        })
    }

    fn insert_cfi_landing_pad(
        &self,
        target_type: CfiTargetType,
        cfi_protection: &CfiControlFlowProtection,
        context: &mut CfiExecutionContext,
    ) -> Result<CfiLandingPad> {
        let hardware_instruction = if let Some(ref hw_config) = cfi_protection.hardware_config {
            self.create_hardware_instruction(target_type, hw_config)?
        } else {
            None
        };

        let software_validation = if cfi_protection.software_config.landing_pad_simulation {
            Some(self.create_software_validation(target_type, context)?)
        } else {
            None
        };

        let landing_pad = CfiLandingPad {
            pad_id: self.generate_landing_pad_id(context),
            hardware_instruction,
            software_validation,
            valid_predecessors: self.determine_valid_predecessors(target_type),
        };

        // Update metrics
        context.metrics.landing_pads_validated += 1;

        Ok(landing_pad)
    }

    fn validate_cfi_requirements(
        &self,
        requirements: &[CfiValidationRequirement],
        context: &CfiExecutionContext,
    ) -> Result<()> {
        for requirement in requirements {
            match requirement {
                CfiValidationRequirement::TypeSignatureCheck {
                    expected_type_index,
                    signature_hash,
                } => {
                    self.validate_type_signature(*expected_type_index, *signature_hash, context)?;
                }
                CfiValidationRequirement::ShadowStackCheck => {
                    self.validate_shadow_stack(context)?;
                }
                CfiValidationRequirement::ControlFlowTargetCheck { valid_targets } => {
                    self.validate_control_flow_target(valid_targets, context)?;
                }
                CfiValidationRequirement::CallingConventionCheck => {
                    self.validate_calling_convention(context)?;
                }
                CfiValidationRequirement::TemporalCheck { max_duration } => {
                    self.validate_temporal_properties(*max_duration, context)?;
                }
            }
        }
        Ok(())
    }
}

impl DefaultCfiControlFlowOps {
    // Helper methods for CFI implementation

    fn create_landing_pad_for_indirect_call(
        &self,
        type_idx: u32,
        cfi_protection: &CfiControlFlowProtection,
        context: &mut CfiExecutionContext,
    ) -> Result<CfiLandingPad> {
        self.insert_cfi_landing_pad(CfiTargetType::IndirectCall, cfi_protection, context)
    }

    fn compute_return_address(&self, context: &CfiExecutionContext) -> u64 {
        // Combine function index and instruction offset into return address
        ((context.current_function as u64) << 32) | (context.current_instruction as u64)
    }

    fn compute_signature_hash(&self, type_idx: u32) -> u64 {
        // Simple hash for now - real implementation would use proper type information
        type_idx as u64 * 0x9e3779b97f4a7c15
    }

    fn get_current_timestamp(&self) -> u64 {
        // Software fallback - return 0 for deterministic behavior
        // Real implementation would use platform-specific timing APIs
        0
    }

    fn validate_shadow_stack_return(&self, context: &mut CfiExecutionContext) -> Result<()> {
        if let Some(shadow_entry) = context.shadow_stack.pop() {
            let expected_return = (context.current_function, context.current_instruction);
            if shadow_entry.return_address != expected_return {
                context.violation_count += 1;
                return Err(Error::new(
                    ErrorCategory::Runtime,
                    codes::CFI_VIOLATION,
                    "Shadow stack return address mismatch",
                ));
            }
            context.metrics.shadow_stack_operations += 1;
        } else {
            context.violation_count += 1;
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::CFI_VIOLATION,
                "Shadow stack underflow",
            ));
        }
        Ok(())
    }

    fn resolve_label_to_offset(
        &self,
        _label_idx: u32,
        _context: &CfiExecutionContext,
    ) -> Result<u32> {
        // TODO: Implement label resolution
        Ok(0)
    }

    fn create_hardware_instruction(
        &self,
        target_type: CfiTargetType,
        hw_config: &HardwareCfiConfig,
    ) -> Result<Option<CfiHardwareInstruction>> {
        match &hw_config.settings {
            #[cfg(target_arch = "aarch64")]
            HardwareCfiSettings::ArmBti { mode, .. } => {
                let bti_mode = match target_type {
                    CfiTargetType::IndirectCall => ArmBtiMode::CallOnly,
                    CfiTargetType::Return => ArmBtiMode::Standard,
                    CfiTargetType::Branch => ArmBtiMode::JumpOnly,
                    _ => *mode,
                };
                Ok(Some(CfiHardwareInstruction::ArmBti { mode: bti_mode }))
            }

            #[cfg(target_arch = "riscv64")]
            HardwareCfiSettings::RiscVCfi { landing_pads: true, .. } => {
                Ok(Some(CfiHardwareInstruction::RiscVLandingPad {
                    label: self.generate_riscv_label(),
                }))
            }

            #[cfg(target_arch = "x86_64")]
            HardwareCfiSettings::X86Cet { indirect_branch_tracking: true, .. } => {
                Ok(Some(CfiHardwareInstruction::X86Endbr))
            }

            _ => Ok(None),
        }
    }

    fn create_software_validation(
        &self,
        target_type: CfiTargetType,
        context: &CfiExecutionContext,
    ) -> Result<CfiSoftwareValidation> {
        let validation_function = match target_type {
            CfiTargetType::IndirectCall => SoftwareCfiFunction::ValidateSignature,
            CfiTargetType::Return => SoftwareCfiFunction::ValidateReturnAddress,
            CfiTargetType::Branch => SoftwareCfiFunction::ValidateBranchTarget,
            _ => SoftwareCfiFunction::ValidateCallSite,
        };

        Ok(CfiSoftwareValidation {
            check_id: self.generate_cfi_check_id(context),
            expected_values: Vec::new(), // Would be populated based on context
            validation_function,
        })
    }

    fn generate_landing_pad_id(&self, _context: &CfiExecutionContext) -> u32 {
        // Generate unique landing pad ID
        0 // Placeholder
    }

    fn generate_cfi_check_id(&self, _context: &CfiExecutionContext) -> u32 {
        // Generate unique CFI check ID
        0 // Placeholder
    }

    #[cfg(target_arch = "riscv64")]
    fn generate_riscv_label(&self) -> u32 {
        // Generate unique RISC-V landing pad label
        0 // Placeholder
    }

    fn determine_valid_predecessors(&self, target_type: CfiTargetType) -> Vec<CfiTargetType> {
        match target_type {
            CfiTargetType::IndirectCall => vec![CfiTargetType::IndirectCall],
            CfiTargetType::Return => vec![CfiTargetType::DirectCall, CfiTargetType::IndirectCall],
            CfiTargetType::Branch => vec![CfiTargetType::Branch],
            CfiTargetType::BlockEntry => vec![CfiTargetType::Branch],
            CfiTargetType::FunctionEntry => {
                vec![CfiTargetType::DirectCall, CfiTargetType::IndirectCall]
            }
            _ => Vec::new(),
        }
    }

    // Validation helper methods

    fn validate_type_signature(
        &self,
        _expected_type_index: u32,
        _signature_hash: u64,
        _context: &CfiExecutionContext,
    ) -> Result<()> {
        // TODO: Implement type signature validation
        Ok(())
    }

    fn validate_shadow_stack(&self, _context: &CfiExecutionContext) -> Result<()> {
        // TODO: Implement shadow stack validation
        Ok(())
    }

    fn validate_control_flow_target(
        &self,
        _valid_targets: &[u32],
        _context: &CfiExecutionContext,
    ) -> Result<()> {
        // TODO: Implement control flow target validation
        Ok(())
    }

    fn validate_calling_convention(&self, _context: &CfiExecutionContext) -> Result<()> {
        // TODO: Implement calling convention validation
        Ok(())
    }

    fn validate_temporal_properties(
        &self,
        _max_duration: u64,
        _context: &CfiExecutionContext,
    ) -> Result<()> {
        // TODO: Implement temporal validation
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfi_control_flow_protection_default() {
        let protection = CfiControlFlowProtection::default();
        assert!(protection.enabled);
        assert_eq!(protection.protection_level, CfiProtectionLevel::Enhanced);
        assert!(protection.software_config.shadow_stack_enabled);
    }

    #[test]
    fn test_cfi_execution_context_default() {
        let context = CfiExecutionContext::default();
        assert_eq!(context.current_function, 0);
        assert_eq!(context.current_instruction, 0);
        assert!(context.shadow_stack.is_empty());
        assert_eq!(context.violation_count, 0);
    }

    #[test]
    fn test_default_cfi_ops_call_indirect() {
        let ops = DefaultCfiControlFlowOps;
        let protection = CfiControlFlowProtection::default();
        let mut context = CfiExecutionContext::default();

        let result = ops.call_indirect_with_cfi(1, 0, &protection, &mut context);
        assert!(result.is_ok());

        let protected_target = result.unwrap();
        assert_eq!(protected_target.protection.target_type, CfiTargetType::IndirectCall);
        assert!(protected_target.protection.landing_pad.is_some());
        assert_eq!(context.metrics.indirect_calls_protected, 1);
    }

    #[test]
    fn test_cfi_disabled() {
        let ops = DefaultCfiControlFlowOps;
        let mut protection = CfiControlFlowProtection::default();
        protection.enabled = false;
        let mut context = CfiExecutionContext::default();

        let result = ops.call_indirect_with_cfi(1, 0, &protection, &mut context);
        assert!(result.is_ok());

        let protected_target = result.unwrap();
        assert!(protected_target.protection.landing_pad.is_none());
        assert!(matches!(
            protected_target.protection.shadow_stack_requirement,
            ShadowStackRequirement::None
        ));
    }

    #[test]
    fn test_software_cfi_config_default() {
        let config = SoftwareCfiConfig::default();
        assert!(config.shadow_stack_enabled);
        assert_eq!(config.max_shadow_stack_depth, 1024);
        assert!(config.landing_pad_simulation);
        assert!(!config.temporal_validation); // Off by default due to cost
    }
}
