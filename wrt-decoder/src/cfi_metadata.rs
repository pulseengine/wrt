// WRT - wrt-decoder
// Module: Control Flow Integrity Metadata Generation
// SW-REQ-ID: REQ_CFI_METADATA_001
//
// Copyright (c) 2025 The WRT Project Developers
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

//! CFI Metadata Generation for WebAssembly Modules
//!
//! This module analyzes WebAssembly modules and generates Control Flow
//! Integrity metadata that can be used by the runtime to protect against
//! ROP/JOP attacks.
//!
//! # Design Principles
//! - Generate CFI metadata from WASM analysis (not embedded in WASM)
//! - Compatible with existing WebAssembly modules
//! - Zero-cost when CFI is disabled
//! - Comprehensive control flow analysis

#![allow(dead_code)] // Allow during development

use wrt_format::types::{FuncType, ValueType};

use crate::prelude::*;

/// Control Flow Integrity metadata for a WebAssembly module
#[derive(Debug, Clone, Default)]
pub struct CfiMetadata {
    /// Function-level CFI information
    pub functions: Vec<FunctionCfiInfo>,
    /// Global CFI requirements
    pub global_requirements: GlobalCfiRequirements,
    /// Import/export CFI validation requirements
    pub imports: Vec<ImportCfiRequirement>,
    pub exports: Vec<ExportCfiRequirement>,
    /// Control flow graph for the entire module
    pub control_flow_graph: ControlFlowGraph,
}

/// CFI information for a single function
#[derive(Debug, Clone)]
pub struct FunctionCfiInfo {
    /// Function index in the module
    pub function_index: u32,
    /// Function type signature
    pub function_type: FuncType,
    /// All indirect call sites in this function
    pub indirect_calls: Vec<IndirectCallSite>,
    /// All return sites in this function
    pub return_sites: Vec<ReturnSite>,
    /// Landing pad requirements
    pub landing_pads: Vec<LandingPadRequirement>,
    /// Internal control flow (branches, blocks, loops)
    pub internal_control_flow: Vec<InternalControlFlow>,
}

/// Information about an indirect call site
#[derive(Debug, Clone)]
pub struct IndirectCallSite {
    /// Instruction offset within the function
    pub instruction_offset: u32,
    /// Function index containing this call
    pub function_index: u32,
    /// Type signature index for the call
    pub type_index: u32,
    /// Table index used for the call
    pub table_index: u32,
    /// Expected return landing pad location
    pub return_landing_pad: LandingPadLocation,
    /// Call site metadata for CFI validation
    pub call_metadata: CallSiteMetadata,
}

/// Information about a function return site
#[derive(Debug, Clone)]
pub struct ReturnSite {
    /// Instruction offset within the function
    pub instruction_offset: u32,
    /// Function index containing this return
    pub function_index: u32,
    /// Expected return values
    pub return_values: Vec<ValueType>,
    /// Shadow stack validation requirement
    pub requires_shadow_stack_check: bool,
}

/// Landing pad requirement for CFI protection
#[derive(Debug, Clone)]
pub struct LandingPadRequirement {
    /// Location where landing pad is needed
    pub location: LandingPadLocation,
    /// Type of control flow target
    pub target_type: ControlFlowTargetType,
    /// Hardware-specific protection instruction
    pub protection_instruction: Option<ProtectionInstruction>,
    /// Validation requirements
    pub validation_requirements: Vec<ValidationRequirement>,
}

/// Location of a landing pad
#[derive(Debug, Clone)]
pub struct LandingPadLocation {
    /// Function index
    pub function_index: u32,
    /// Instruction offset within function
    pub instruction_offset: u32,
}

/// Types of control flow targets that need protection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlFlowTargetType {
    /// Direct function call
    DirectCall,
    /// Indirect function call (via table)
    IndirectCall,
    /// Return from function
    Return,
    /// Branch within function (br, br_if)
    Branch,
    /// Block/loop entry point
    BlockEntry,
    /// Function entry point
    FunctionEntry,
}

/// Hardware-specific protection instructions
#[derive(Debug, Clone)]
pub enum ProtectionInstruction {
    /// ARM BTI instruction
    #[cfg(target_arch = "aarch64")]
    ArmBti {
        /// BTI mode
        mode: ArmBtiMode,
    },
    /// RISC-V CFI landing pad
    #[cfg(target_arch = "riscv64")]
    RiscVLandingPad {
        /// Landing pad label
        label: u32,
    },
    /// Software CFI check
    SoftwareCfi {
        /// Unique check identifier
        check_id: u32,
    },
}

#[cfg(target_arch = "aarch64")]
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

/// CFI validation requirements
#[derive(Debug, Clone)]
pub enum ValidationRequirement {
    /// Validate function signature matches expected
    TypeSignatureCheck { expected_type: u32 },
    /// Validate return address matches shadow stack
    ShadowStackCheck,
    /// Validate control flow target is valid
    ControlFlowTargetCheck { valid_targets: Vec<u32> },
    /// Validate calling convention
    CallingConventionCheck,
}

/// Metadata for call sites
#[derive(Debug, Clone)]
pub struct CallSiteMetadata {
    /// Expected function signature hash
    pub signature_hash: u64,
    /// Maximum allowed execution time (for timeout detection)
    pub max_execution_time: Option<u64>,
    /// Security level required for this call
    pub security_level: CallSecurityLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallSecurityLevel {
    /// No special security requirements
    None,
    /// Basic CFI protection
    Basic,
    /// Enhanced CFI with temporal validation
    Enhanced,
    /// Maximum security with all protections
    Maximum,
}

/// Internal control flow within a function
#[derive(Debug, Clone)]
pub struct InternalControlFlow {
    /// Source instruction offset
    pub source_offset: u32,
    /// Target instruction offset
    pub target_offset: u32,
    /// Type of control flow
    pub flow_type: InternalFlowType,
    /// Validation requirements
    pub validation: Vec<ValidationRequirement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InternalFlowType {
    /// Unconditional branch (br)
    Branch,
    /// Conditional branch (br_if)
    ConditionalBranch,
    /// Block entry
    BlockEntry,
    /// Loop entry
    LoopEntry,
    /// Block/loop exit
    BlockExit,
}

/// Global CFI requirements for the module
#[derive(Debug, Clone, Default)]
pub struct GlobalCfiRequirements {
    /// Required CFI protection level
    pub protection_level: CfiProtectionLevel,
    /// Hardware features required
    pub required_hardware_features: Vec<RequiredHardwareFeature>,
    /// Software CFI fallback allowed
    pub allow_software_fallback: bool,
    /// Maximum acceptable CFI overhead percentage
    pub max_overhead_percent: f32,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequiredHardwareFeature {
    /// ARM Branch Target Identification
    ArmBti,
    /// ARM Memory Tagging Extension
    ArmMte,
    /// RISC-V Control Flow Integrity
    RiscVCfi,
    /// RISC-V Physical Memory Protection
    RiscVPmp,
}

/// CFI requirements for imported functions
#[derive(Debug, Clone)]
pub struct ImportCfiRequirement {
    /// Import module name
    pub module_name: String,
    /// Import function name
    pub function_name: String,
    /// Required CFI protection level
    pub protection_level: CfiProtectionLevel,
    /// Additional validation requirements
    pub validation_requirements: Vec<ValidationRequirement>,
}

/// CFI requirements for exported functions
#[derive(Debug, Clone)]
pub struct ExportCfiRequirement {
    /// Export function name
    pub function_name: String,
    /// Function index
    pub function_index: u32,
    /// Required CFI protection level
    pub protection_level: CfiProtectionLevel,
    /// Entry point validation requirements
    pub entry_validation: Vec<ValidationRequirement>,
}

/// Control flow graph for the entire module
#[derive(Debug, Clone, Default)]
pub struct ControlFlowGraph {
    /// Function-level control flow information
    pub functions: Vec<FunctionControlFlow>,
    /// Cross-function call relationships
    pub call_graph: Vec<CallRelationship>,
    /// Function table relationships
    pub function_tables: Vec<FunctionTableInfo>,
}

/// Control flow information for a single function
#[derive(Debug, Clone)]
pub struct FunctionControlFlow {
    /// Function index
    pub function_index: u32,
    /// Entry points (normally just offset 0)
    pub entry_points: Vec<u32>,
    /// Exit points (return instructions)
    pub exit_points: Vec<u32>,
    /// Internal control flow edges
    pub internal_edges: Vec<ControlFlowEdge>,
    /// Indirect call sites
    pub indirect_calls: Vec<u32>,
    /// Basic blocks
    pub basic_blocks: Vec<BasicBlock>,
}

/// Control flow edge between instructions
#[derive(Debug, Clone)]
pub struct ControlFlowEdge {
    /// Source instruction offset
    pub source: u32,
    /// Target instruction offset
    pub target: u32,
    /// Edge type
    pub edge_type: EdgeType,
    /// Condition for conditional edges
    pub condition: Option<EdgeCondition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    /// Sequential execution
    Sequential,
    /// Unconditional jump
    Jump,
    /// Conditional jump
    ConditionalJump,
    /// Function call
    Call,
    /// Function return
    Return,
    /// Exception/trap
    Exception,
}

#[derive(Debug, Clone)]
pub enum EdgeCondition {
    /// Branch taken if top of stack is non-zero
    BranchIf,
    /// Branch taken if top of stack is zero
    BranchIfNot,
    /// Conditional based on comparison result
    ComparisonResult,
}

/// Basic block in control flow graph
#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Block identifier
    pub block_id: u32,
    /// Start instruction offset
    pub start_offset: u32,
    /// End instruction offset (exclusive)
    pub end_offset: u32,
    /// Predecessor blocks
    pub predecessors: Vec<u32>,
    /// Successor blocks
    pub successors: Vec<u32>,
}

/// Cross-function call relationship
#[derive(Debug, Clone)]
pub struct CallRelationship {
    /// Caller function index
    pub caller: u32,
    /// Caller instruction offset
    pub call_site: u32,
    /// Callee function index (None for indirect calls)
    pub callee: Option<u32>,
    /// Call type
    pub call_type: CallType,
    /// Call metadata
    pub metadata: CallSiteMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallType {
    /// Direct function call
    Direct,
    /// Indirect call via function table
    Indirect,
    /// Host function call (import)
    Host,
}

/// Function table information for CFI validation
#[derive(Debug, Clone)]
pub struct FunctionTableInfo {
    /// Table index
    pub table_index: u32,
    /// Functions that can be called via this table
    pub callable_functions: Vec<u32>,
    /// Type constraints for table entries
    pub type_constraints: Vec<u32>,
    /// CFI validation requirements for table calls
    pub validation_requirements: Vec<ValidationRequirement>,
}

/// CFI metadata generator for WebAssembly modules
pub struct CfiMetadataGenerator {
    /// Current function being analyzed
    current_function: Option<u32>,
    /// Generated metadata
    metadata: CfiMetadata,
    /// CFI protection configuration
    protection_config: CfiProtectionConfig,
    /// Hardware feature detection
    hardware_features: AvailableHardwareFeatures,
}

/// CFI protection configuration
#[derive(Debug, Clone)]
pub struct CfiProtectionConfig {
    /// Target protection level
    pub target_level: CfiProtectionLevel,
    /// Enable hardware-specific optimizations
    pub enable_hardware_optimizations: bool,
    /// Allow software fallback when hardware unavailable
    pub allow_software_fallback: bool,
    /// Maximum acceptable performance overhead
    pub max_overhead_percent: f32,
}

impl Default for CfiProtectionConfig {
    fn default() -> Self {
        Self {
            target_level: CfiProtectionLevel::Enhanced,
            enable_hardware_optimizations: true,
            allow_software_fallback: true,
            max_overhead_percent: 10.0, // 10% overhead acceptable
        }
    }
}

/// Available hardware features for CFI
#[derive(Debug, Clone, Default)]
pub struct AvailableHardwareFeatures {
    /// ARM BTI available
    pub arm_bti: bool,
    /// ARM MTE available
    pub arm_mte: bool,
    /// RISC-V CFI available
    pub riscv_cfi: bool,
    /// RISC-V PMP available
    pub riscv_pmp: bool,
}

impl CfiMetadataGenerator {
    /// Create new CFI metadata generator
    pub fn new(config: CfiProtectionConfig) -> Self {
        Self {
            current_function: None,
            metadata: CfiMetadata::default(),
            protection_config: config,
            hardware_features: Self::detect_hardware_features(),
        }
    }

    /// Detect available hardware features
    fn detect_hardware_features() -> AvailableHardwareFeatures {
        AvailableHardwareFeatures {
            #[cfg(target_arch = "aarch64")]
            arm_bti: cfg!(target_feature = "bti"),
            #[cfg(not(target_arch = "aarch64"))]
            arm_bti: false,

            #[cfg(target_arch = "aarch64")]
            arm_mte: cfg!(target_feature = "mte"),
            #[cfg(not(target_arch = "aarch64"))]
            arm_mte: false,

            #[cfg(target_arch = "riscv64")]
            riscv_cfi: cfg!(target_feature = "zisslpcfi"),
            #[cfg(not(target_arch = "riscv64"))]
            riscv_cfi: false,

            #[cfg(target_arch = "riscv64")]
            riscv_pmp: true, // PMP is required by RISC-V spec
            #[cfg(not(target_arch = "riscv64"))]
            riscv_pmp: false,
        }
    }

    /// Generate CFI metadata for a WebAssembly module
    pub fn generate_metadata(&mut self, module: &crate::module::Module) -> Result<CfiMetadata> {
        // Reset metadata for new module
        self.metadata = CfiMetadata::default();

        // Set global CFI requirements
        self.metadata.global_requirements = GlobalCfiRequirements {
            protection_level: self.protection_config.target_level,
            required_hardware_features: self.determine_required_features(),
            allow_software_fallback: self.protection_config.allow_software_fallback,
            max_overhead_percent: self.protection_config.max_overhead_percent,
        };

        // Analyze each function in the module
        for (func_index, function) in module.functions.iter().enumerate() {
            self.current_function = Some(func_index as u32);
            let function_cfi = self.analyze_function(func_index as u32, function)?;
            self.metadata.functions.push(function_cfi);
        }

        // Build control flow graph
        self.metadata.control_flow_graph = self.build_control_flow_graph(module)?;

        // Analyze imports and exports
        self.analyze_imports_exports(module)?;

        Ok(self.metadata.clone())
    }

    /// Determine required hardware features based on protection level
    fn determine_required_features(&self) -> Vec<RequiredHardwareFeature> {
        let mut features = Vec::new();

        match self.protection_config.target_level {
            CfiProtectionLevel::None => {
                // No features required
            }
            CfiProtectionLevel::Basic => {
                // Basic landing pad support
                if self.hardware_features.arm_bti {
                    features.push(RequiredHardwareFeature::ArmBti);
                }
                if self.hardware_features.riscv_cfi {
                    features.push(RequiredHardwareFeature::RiscVCfi);
                }
            }
            CfiProtectionLevel::Enhanced | CfiProtectionLevel::Maximum => {
                // All available features for maximum protection
                if self.hardware_features.arm_bti {
                    features.push(RequiredHardwareFeature::ArmBti);
                }
                if self.hardware_features.arm_mte {
                    features.push(RequiredHardwareFeature::ArmMte);
                }
                if self.hardware_features.riscv_cfi {
                    features.push(RequiredHardwareFeature::RiscVCfi);
                }
                if self.hardware_features.riscv_pmp {
                    features.push(RequiredHardwareFeature::RiscVPmp);
                }
            }
        }

        features
    }

    /// Analyze a single function for CFI requirements
    fn analyze_function(
        &mut self,
        func_index: u32,
        function: &crate::module::Function,
    ) -> Result<FunctionCfiInfo> {
        let mut function_cfi = FunctionCfiInfo {
            function_index: func_index,
            function_type: function.func_type.clone(),
            indirect_calls: Vec::new(),
            return_sites: Vec::new(),
            landing_pads: Vec::new(),
            internal_control_flow: Vec::new(),
        };

        // Analyze each instruction in the function
        for (instr_offset, instruction) in function.instructions.iter().enumerate() {
            match instruction {
                crate::instructions::Instruction::CallIndirect(type_idx, table_idx) => {
                    let call_site = self.analyze_indirect_call(
                        func_index,
                        instr_offset as u32,
                        *type_idx,
                        *table_idx,
                    )?;
                    function_cfi.indirect_calls.push(call_site);
                }

                crate::instructions::Instruction::Return => {
                    let return_site = self.analyze_return_site(
                        func_index,
                        instr_offset as u32,
                        &function.func_type,
                    )?;
                    function_cfi.return_sites.push(return_site);
                }

                crate::instructions::Instruction::Br(label_idx)
                | crate::instructions::Instruction::BrIf(label_idx) => {
                    let control_flow = self.analyze_branch(
                        func_index,
                        instr_offset as u32,
                        *label_idx,
                        matches!(instruction, crate::instructions::Instruction::BrIf(_)),
                    )?;
                    function_cfi.internal_control_flow.push(control_flow);
                }

                _ => {
                    // Other instructions don't require special CFI handling
                }
            }
        }

        // Generate landing pad requirements
        function_cfi.landing_pads = self.generate_landing_pad_requirements(&function_cfi)?;

        Ok(function_cfi)
    }

    /// Analyze an indirect call instruction
    fn analyze_indirect_call(
        &self,
        func_index: u32,
        instr_offset: u32,
        type_idx: u32,
        table_idx: u32,
    ) -> Result<IndirectCallSite> {
        // Create return landing pad requirement
        let return_landing_pad = LandingPadLocation {
            function_index: func_index,
            instruction_offset: instr_offset + 1, // After the call
        };

        // Generate call site metadata
        let call_metadata = CallSiteMetadata {
            signature_hash: self.compute_signature_hash(type_idx),
            max_execution_time: self.compute_max_execution_time(),
            security_level: self.determine_call_security_level(),
        };

        Ok(IndirectCallSite {
            instruction_offset: instr_offset,
            function_index: func_index,
            type_index: type_idx,
            table_index: table_idx,
            return_landing_pad,
            call_metadata,
        })
    }

    /// Analyze a return instruction
    fn analyze_return_site(
        &self,
        func_index: u32,
        instr_offset: u32,
        func_type: &FuncType,
    ) -> Result<ReturnSite> {
        Ok(ReturnSite {
            instruction_offset: instr_offset,
            function_index: func_index,
            return_values: func_type.results.clone(),
            requires_shadow_stack_check: matches!(
                self.protection_config.target_level,
                CfiProtectionLevel::Enhanced | CfiProtectionLevel::Maximum
            ),
        })
    }

    /// Analyze a branch instruction
    fn analyze_branch(
        &self,
        func_index: u32,
        instr_offset: u32,
        label_idx: u32,
        is_conditional: bool,
    ) -> Result<InternalControlFlow> {
        // TODO: Resolve label to actual target offset
        let target_offset = self.resolve_label_offset(func_index, label_idx)?;

        Ok(InternalControlFlow {
            source_offset: instr_offset,
            target_offset,
            flow_type: if is_conditional {
                InternalFlowType::ConditionalBranch
            } else {
                InternalFlowType::Branch
            },
            validation: vec![ValidationRequirement::ControlFlowTargetCheck {
                valid_targets: vec![target_offset],
            }],
        })
    }

    /// Generate landing pad requirements for a function
    fn generate_landing_pad_requirements(
        &self,
        function_cfi: &FunctionCfiInfo,
    ) -> Result<Vec<LandingPadRequirement>> {
        let mut requirements = Vec::new();

        // Landing pads for indirect call returns
        for call_site in &function_cfi.indirect_calls {
            let protection_instruction =
                self.create_protection_instruction(ControlFlowTargetType::IndirectCall);

            requirements.push(LandingPadRequirement {
                location: call_site.return_landing_pad.clone(),
                target_type: ControlFlowTargetType::IndirectCall,
                protection_instruction,
                validation_requirements: vec![ValidationRequirement::TypeSignatureCheck {
                    expected_type: call_site.type_index,
                }],
            });
        }

        // Landing pads for function entries (if needed)
        if matches!(
            self.protection_config.target_level,
            CfiProtectionLevel::Enhanced | CfiProtectionLevel::Maximum
        ) {
            let protection_instruction =
                self.create_protection_instruction(ControlFlowTargetType::FunctionEntry);

            requirements.push(LandingPadRequirement {
                location: LandingPadLocation {
                    function_index: function_cfi.function_index,
                    instruction_offset: 0, // Function start
                },
                target_type: ControlFlowTargetType::FunctionEntry,
                protection_instruction,
                validation_requirements: vec![ValidationRequirement::CallingConventionCheck],
            });
        }

        Ok(requirements)
    }

    /// Create hardware-specific protection instruction
    fn create_protection_instruction(
        &self,
        target_type: ControlFlowTargetType,
    ) -> Option<ProtectionInstruction> {
        #[cfg(target_arch = "aarch64")]
        if self.hardware_features.arm_bti {
            let mode = match target_type {
                ControlFlowTargetType::IndirectCall => ArmBtiMode::CallOnly,
                ControlFlowTargetType::Return => ArmBtiMode::Standard,
                ControlFlowTargetType::Branch => ArmBtiMode::JumpOnly,
                _ => ArmBtiMode::CallAndJump,
            };
            return Some(ProtectionInstruction::ArmBti { mode });
        }

        #[cfg(target_arch = "riscv64")]
        if self.hardware_features.riscv_cfi {
            return Some(ProtectionInstruction::RiscVLandingPad {
                label: self.generate_landing_pad_label(),
            });
        }

        // Software CFI fallback
        if self.protection_config.allow_software_fallback {
            return Some(ProtectionInstruction::SoftwareCfi {
                check_id: self.generate_cfi_check_id(),
            });
        }

        None
    }

    /// Helper functions
    fn compute_signature_hash(&self, type_idx: u32) -> u64 {
        // Simple hash for now - real implementation would use proper hashing
        type_idx as u64 * 0x9e3779b97f4a7c15
    }

    fn compute_max_execution_time(&self) -> Option<u64> {
        match self.protection_config.target_level {
            CfiProtectionLevel::Maximum => Some(10000), // 10000 cycles
            CfiProtectionLevel::Enhanced => Some(50000), // 50000 cycles
            _ => None,
        }
    }

    fn determine_call_security_level(&self) -> CallSecurityLevel {
        match self.protection_config.target_level {
            CfiProtectionLevel::None => CallSecurityLevel::None,
            CfiProtectionLevel::Basic => CallSecurityLevel::Basic,
            CfiProtectionLevel::Enhanced => CallSecurityLevel::Enhanced,
            CfiProtectionLevel::Maximum => CallSecurityLevel::Maximum,
        }
    }

    fn resolve_label_offset(&self, _func_index: u32, _label_idx: u32) -> Result<u32> {
        // TODO: Implement label resolution
        // For now, return a placeholder
        Ok(0)
    }

    #[cfg(target_arch = "riscv64")]
    fn generate_landing_pad_label(&self) -> u32 {
        // Generate unique landing pad label
        0 // Placeholder
    }

    fn generate_cfi_check_id(&self) -> u32 {
        // Generate unique CFI check ID
        0 // Placeholder
    }

    fn build_control_flow_graph(
        &self,
        _module: &crate::module::Module,
    ) -> Result<ControlFlowGraph> {
        // TODO: Implement control flow graph construction
        Ok(ControlFlowGraph::default())
    }

    fn analyze_imports_exports(&mut self, _module: &crate::module::Module) -> Result<()> {
        // TODO: Analyze imports and exports for CFI requirements
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfi_metadata_generator_creation() {
        let config = CfiProtectionConfig::default();
        let generator = CfiMetadataGenerator::new(config);

        assert!(matches!(generator.protection_config.target_level, CfiProtectionLevel::Enhanced));
        assert!(generator.protection_config.allow_software_fallback);
    }

    #[test]
    fn test_hardware_feature_detection() {
        let features = CfiMetadataGenerator::detect_hardware_features();

        // Test should work regardless of actual hardware availability
        #[cfg(target_arch = "aarch64")]
        {
            // ARM features might or might not be available
            let _ = features.arm_bti;
            let _ = features.arm_mte;
        }

        #[cfg(target_arch = "riscv64")]
        {
            // RISC-V features might or might not be available
            let _ = features.riscv_cfi;
            assert!(features.riscv_pmp); // PMP is required by spec
        }
    }

    #[test]
    fn test_protection_instruction_creation() {
        let config = CfiProtectionConfig::default();
        let generator = CfiMetadataGenerator::new(config);

        let instruction =
            generator.create_protection_instruction(ControlFlowTargetType::IndirectCall);

        // Should have some protection instruction (hardware or software)
        assert!(instruction.is_some());
    }
}
