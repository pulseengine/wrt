//! Core CFI Types and Structures Testing
//!
//! Tests fundamental CFI data structures and algorithms
//! independent of external dependencies.

use serde::{Deserialize, Serialize};

/// CFI Protection Level enumeration for testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CfiProtectionLevel {
    /// Hardware-only CFI protection
    Hardware,
    /// Software-only CFI protection  
    Software,
    /// Hybrid hardware + software CFI
    Hybrid,
}

impl Default for CfiProtectionLevel {
    fn default() -> Self {
        CfiProtectionLevel::Hybrid
    }
}

/// CFI Configuration for isolated testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CfiConfiguration {
    pub protection_level: CfiProtectionLevel,
    pub max_shadow_stack_depth: usize,
    pub landing_pad_timeout_ns: Option<u64>,
    pub enable_temporal_validation: bool,
    pub hardware_features: CfiHardwareFeatures,
}

/// CFI Hardware Features configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CfiHardwareFeatures {
    pub arm_bti: bool,
    pub riscv_cfi: bool,
    pub x86_cet: bool,
    pub auto_detect: bool,
}

/// CFI Violation Policy for testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CfiViolationPolicy {
    LogAndContinue,
    Terminate,
    ReturnError,
    AttemptRecovery,
}

impl Default for CfiViolationPolicy {
    fn default() -> Self {
        CfiViolationPolicy::ReturnError
    }
}

/// CFI Statistics for monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CfiStatistics {
    pub instructions_protected: u64,
    pub violations_detected: u64,
    pub violations_resolved: u64,
    pub shadow_stack_operations: u64,
    pub landing_pads_validated: u64,
    pub temporal_violations: u64,
}

impl Default for CfiConfiguration {
    fn default() -> Self {
        Self {
            protection_level: CfiProtectionLevel::Hybrid,
            max_shadow_stack_depth: 1024,
            landing_pad_timeout_ns: Some(1_000_000), // 1ms
            enable_temporal_validation: true,
            hardware_features: CfiHardwareFeatures {
                auto_detect: true,
                ..Default::default()
            },
        }
    }
}

/// CFI Shadow Stack Entry for testing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowStackEntry {
    pub return_address: (u32, u32), // (function_index, instruction_offset)
    pub signature_hash: u64,
    pub timestamp: u64,
    pub call_site_id: u32,
}

/// CFI Landing Pad information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LandingPad {
    pub function_index: u32,
    pub instruction_offset: u32,
    pub expected_signature: u64,
    pub hardware_instruction: Option<HardwareInstruction>,
    pub timeout_ns: Option<u64>,
}

/// Hardware CFI instruction types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareInstruction {
    ArmBti { mode: ArmBtiMode },
    RiscVLandingPad { label: u32 },
    X86Endbr,
}

/// ARM BTI modes for testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArmBtiMode {
    Standard,
    CallOnly,
    JumpOnly, 
    CallAndJump,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfi_configuration_default() {
        let config = CfiConfiguration::default();
        assert_eq!(config.protection_level, CfiProtectionLevel::Hybrid);
        assert_eq!(config.max_shadow_stack_depth, 1024);
        assert_eq!(config.landing_pad_timeout_ns, Some(1_000_000));
        assert!(config.enable_temporal_validation);
        assert!(config.hardware_features.auto_detect);
    }

    #[test]
    fn test_cfi_configuration_serialization() {
        let config = CfiConfiguration::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: CfiConfiguration = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.protection_level, deserialized.protection_level);
        assert_eq!(config.max_shadow_stack_depth, deserialized.max_shadow_stack_depth);
    }

    #[test]
    fn test_cfi_protection_levels() {
        assert_eq!(CfiProtectionLevel::default(), CfiProtectionLevel::Hybrid);
        
        let levels = [
            CfiProtectionLevel::Hardware,
            CfiProtectionLevel::Software,
            CfiProtectionLevel::Hybrid,
        ];
        
        for level in levels {
            let json = serde_json::to_string(&level).unwrap();
            let deserialized: CfiProtectionLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(level, deserialized);
        }
    }

    #[test]
    fn test_cfi_violation_policy() {
        let policy = CfiViolationPolicy::default();
        assert_eq!(policy, CfiViolationPolicy::ReturnError);
        
        let policies = [
            CfiViolationPolicy::LogAndContinue,
            CfiViolationPolicy::Terminate,
            CfiViolationPolicy::ReturnError,
            CfiViolationPolicy::AttemptRecovery,
        ];
        
        for policy in policies {
            let json = serde_json::to_string(&policy).unwrap();
            let deserialized: CfiViolationPolicy = serde_json::from_str(&json).unwrap();
            assert_eq!(policy, deserialized);
        }
    }

    #[test]
    fn test_cfi_statistics() {
        let mut stats = CfiStatistics::default();
        assert_eq!(stats.instructions_protected, 0);
        assert_eq!(stats.violations_detected, 0);
        
        stats.instructions_protected = 1000;
        stats.violations_detected = 5;
        stats.violations_resolved = 3;
        
        assert_eq!(stats.instructions_protected, 1000);
        assert_eq!(stats.violations_detected, 5);
        assert_eq!(stats.violations_resolved, 3);
    }

    #[test]
    fn test_shadow_stack_entry() {
        let entry = ShadowStackEntry {
            return_address: (42, 100),
            signature_hash: 0xdeadbeef,
            timestamp: 1234567890,
            call_site_id: 0x1000,
        };
        
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: ShadowStackEntry = serde_json::from_str(&json).unwrap();
        
        assert_eq!(entry, deserialized);
    }

    #[test]
    fn test_landing_pad() {
        let landing_pad = LandingPad {
            function_index: 10,
            instruction_offset: 50,
            expected_signature: 0xcafebabe,
            hardware_instruction: Some(HardwareInstruction::ArmBti { 
                mode: ArmBtiMode::CallAndJump 
            }),
            timeout_ns: Some(500_000),
        };
        
        let json = serde_json::to_string(&landing_pad).unwrap();
        let deserialized: LandingPad = serde_json::from_str(&json).unwrap();
        
        assert_eq!(landing_pad, deserialized);
    }

    #[test]
    fn test_hardware_instructions() {
        let instructions = vec![
            HardwareInstruction::ArmBti { mode: ArmBtiMode::Standard },
            HardwareInstruction::RiscVLandingPad { label: 42 },
            HardwareInstruction::X86Endbr,
        ];
        
        for instruction in instructions {
            let json = serde_json::to_string(&instruction).unwrap();
            let deserialized: HardwareInstruction = serde_json::from_str(&json).unwrap();
            assert_eq!(instruction, deserialized);
        }
    }

    #[test]
    fn test_arm_bti_modes() {
        let modes = [
            ArmBtiMode::Standard,
            ArmBtiMode::CallOnly,
            ArmBtiMode::JumpOnly,
            ArmBtiMode::CallAndJump,
        ];
        
        for mode in modes {
            let json = serde_json::to_string(&mode).unwrap();
            let deserialized: ArmBtiMode = serde_json::from_str(&json).unwrap();
            assert_eq!(mode, deserialized);
        }
    }

    #[test]
    fn test_cfi_hardware_features() {
        let mut features = CfiHardwareFeatures::default();
        assert!(!features.arm_bti);
        assert!(!features.riscv_cfi);
        assert!(!features.x86_cet);
        assert!(features.auto_detect);
        
        features.arm_bti = true;
        features.riscv_cfi = true;
        features.auto_detect = false;
        
        let json = serde_json::to_string(&features).unwrap();
        let deserialized: CfiHardwareFeatures = serde_json::from_str(&json).unwrap();
        
        assert_eq!(features.arm_bti, deserialized.arm_bti);
        assert_eq!(features.riscv_cfi, deserialized.riscv_cfi);
        assert_eq!(features.auto_detect, deserialized.auto_detect);
    }
}