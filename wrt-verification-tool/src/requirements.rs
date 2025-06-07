//! Requirements Traceability Framework
//!
//! This module provides a comprehensive requirements traceability system inspired by
//! SCORE's approach to safety-critical system verification. It links requirements to
//! implementation code, tests, and documentation for full accountability.

use wrt_foundation::{
    safety_system::AsilLevel,
    prelude::*,
};
use core::fmt;

/// Unique identifier for a requirement
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequirementId(String);

impl RequirementId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RequirementId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type of requirement based on safety standards
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequirementType {
    /// Functional requirement (what the system must do)
    Functional,
    /// Performance requirement (timing, throughput, etc.)
    Performance,
    /// Safety requirement (ASIL-related)
    Safety,
    /// Security requirement (protection against attacks)
    Security,
    /// Reliability requirement (availability, fault tolerance)
    Reliability,
    /// Qualification requirement (certification, standards)
    Qualification,
    /// Platform requirement (hardware/OS specific)
    Platform,
    /// Memory requirement (allocation, constraints)
    Memory,
}

/// Verification method for a requirement
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationMethod {
    /// Requirement verified through inspection/review
    Inspection,
    /// Requirement verified through analysis (static/dynamic)
    Analysis,
    /// Requirement verified through testing
    Test,
    /// Requirement verified through demonstration
    Demonstration,
    /// Requirement verified through simulation
    Simulation,
    /// Requirement verified through formal proof
    FormalProof,
}

/// Current status of requirement verification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationStatus {
    /// Verification not started
    NotStarted,
    /// Verification in progress
    InProgress,
    /// Verification completed successfully
    Verified,
    /// Verification failed
    Failed(String),
    /// Verification not applicable
    NotApplicable,
}

/// Coverage level for requirement testing
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CoverageLevel {
    /// No coverage
    None,
    /// Basic coverage (happy path)
    Basic,
    /// Comprehensive coverage (edge cases)
    Comprehensive,
    /// Complete coverage (all paths, formal verification)
    Complete,
}

/// Safety requirement definition
#[derive(Debug, Clone)]
pub struct SafetyRequirement {
    /// Unique identifier
    pub id: RequirementId,
    /// Human-readable title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Requirement type classification
    pub req_type: RequirementType,
    /// Required ASIL level
    pub asil_level: AsilLevel,
    /// Verification method
    pub verification_method: VerificationMethod,
    /// Current verification status
    pub status: VerificationStatus,
    /// Coverage level achieved
    pub coverage: CoverageLevel,
    /// Parent requirement (if this is derived)
    pub parent: Option<RequirementId>,
    /// Source document/standard
    pub source: String,
    /// Implementation references
    pub implementations: Vec<String>,
    /// Test references
    pub tests: Vec<String>,
    /// Documentation references
    pub documentation: Vec<String>,
}

impl SafetyRequirement {
    /// Create a new safety requirement
    pub fn new(
        id: RequirementId,
        title: String,
        description: String,
        req_type: RequirementType,
        asil_level: AsilLevel,
    ) -> Self {
        Self {
            id,
            title,
            description,
            req_type,
            asil_level,
            verification_method: VerificationMethod::Test,
            status: VerificationStatus::NotStarted,
            coverage: CoverageLevel::None,
            parent: None,
            source: String::new(),
            implementations: Vec::new(),
            tests: Vec::new(),
            documentation: Vec::new(),
        }
    }
    
    /// Add implementation reference
    pub fn add_implementation(&mut self, implementation: String) {
        self.implementations.push(implementation);
    }
    
    /// Add test reference
    pub fn add_test(&mut self, test: String) {
        self.tests.push(test);
    }
    
    /// Add documentation reference
    pub fn add_documentation(&mut self, doc: String) {
        self.documentation.push(doc);
    }
    
    /// Set verification status
    pub fn set_status(&mut self, status: VerificationStatus) {
        self.status = status;
    }
    
    /// Set coverage level
    pub fn set_coverage(&mut self, coverage: CoverageLevel) {
        self.coverage = coverage;
    }
    
    /// Check if requirement is fully verified
    pub fn is_verified(&self) -> bool {
        matches!(self.status, VerificationStatus::Verified) &&
        self.coverage >= CoverageLevel::Basic &&
        !self.implementations.is_empty()
    }
    
    /// Check if requirement needs implementation
    pub fn needs_implementation(&self) -> bool {
        self.implementations.is_empty()
    }
    
    /// Check if requirement needs testing
    pub fn needs_testing(&self) -> bool {
        self.tests.is_empty() || self.coverage < CoverageLevel::Basic
    }
    
    /// Get compliance score (0.0 to 1.0)
    pub fn compliance_score(&self) -> f64 {
        let mut score = 0.0;
        let mut total_points = 4.0;
        
        // Implementation coverage
        if !self.implementations.is_empty() {
            score += 1.0;
        }
        
        // Test coverage
        match self.coverage {
            CoverageLevel::None => {},
            CoverageLevel::Basic => score += 0.5,
            CoverageLevel::Comprehensive => score += 0.8,
            CoverageLevel::Complete => score += 1.0,
        }
        
        // Verification status
        match self.status {
            VerificationStatus::Verified => score += 1.0,
            VerificationStatus::InProgress => score += 0.3,
            _ => {},
        }
        
        // Documentation
        if !self.documentation.is_empty() {
            score += 1.0;
        }
        
        score / total_points
    }
}

/// Requirements registry for tracking all safety requirements
pub struct RequirementRegistry {
    requirements: Vec<SafetyRequirement>,
}

impl RequirementRegistry {
    /// Create a new requirements registry
    pub fn new() -> Self {
        Self {
            requirements: Vec::new(),
        }
    }
    
    /// Add a requirement to the registry
    pub fn add_requirement(&mut self, requirement: SafetyRequirement) {
        self.requirements.push(requirement);
    }
    
    /// Get requirement by ID
    pub fn get_requirement(&self, id: &RequirementId) -> Option<&SafetyRequirement> {
        self.requirements.iter().find(|r| r.id == *id)
    }
    
    /// Get mutable requirement by ID
    pub fn get_requirement_mut(&mut self, id: &RequirementId) -> Option<&mut SafetyRequirement> {
        self.requirements.iter_mut().find(|r| r.id == *id)
    }
    
    /// Get all requirements for a specific ASIL level
    pub fn get_requirements_by_asil(&self, asil_level: AsilLevel) -> Vec<&SafetyRequirement> {
        self.requirements.iter()
            .filter(|r| r.asil_level == asil_level)
            .collect()
    }
    
    /// Get all requirements of a specific type
    pub fn get_requirements_by_type(&self, req_type: RequirementType) -> Vec<&SafetyRequirement> {
        self.requirements.iter()
            .filter(|r| r.req_type == req_type)
            .collect()
    }
    
    /// Get unverified requirements
    pub fn get_unverified_requirements(&self) -> Vec<&SafetyRequirement> {
        self.requirements.iter()
            .filter(|r| !r.is_verified())
            .collect()
    }
    
    /// Get requirements needing implementation
    pub fn get_requirements_needing_implementation(&self) -> Vec<&SafetyRequirement> {
        self.requirements.iter()
            .filter(|r| r.needs_implementation())
            .collect()
    }
    
    /// Get requirements needing testing
    pub fn get_requirements_needing_testing(&self) -> Vec<&SafetyRequirement> {
        self.requirements.iter()
            .filter(|r| r.needs_testing())
            .collect()
    }
    
    /// Calculate overall compliance percentage
    pub fn overall_compliance(&self) -> f64 {
        if self.requirements.is_empty() {
            return 1.0; // 100% compliant if no requirements
        }
        
        let total_score: f64 = self.requirements.iter()
            .map(|r| r.compliance_score())
            .sum();
        
        total_score / self.requirements.len() as f64
    }
    
    /// Calculate ASIL-specific compliance
    pub fn asil_compliance(&self, asil_level: AsilLevel) -> f64 {
        let asil_requirements = self.get_requirements_by_asil(asil_level);
        
        if asil_requirements.is_empty() {
            return 1.0;
        }
        
        let total_score: f64 = asil_requirements.iter()
            .map(|r| r.compliance_score())
            .sum();
        
        total_score / asil_requirements.len() as f64
    }
    
    /// Generate compliance report
    pub fn generate_compliance_report(&self) -> ComplianceReport {
        ComplianceReport {
            total_requirements: self.requirements.len(),
            verified_requirements: self.requirements.iter().filter(|r| r.is_verified()).count(),
            overall_compliance: self.overall_compliance(),
            asil_compliance: [
                (AsilLevel::QM, self.asil_compliance(AsilLevel::QM)),
                (AsilLevel::ASIL_A, self.asil_compliance(AsilLevel::ASIL_A)),
                (AsilLevel::ASIL_B, self.asil_compliance(AsilLevel::ASIL_B)),
                (AsilLevel::ASIL_C, self.asil_compliance(AsilLevel::ASIL_C)),
                (AsilLevel::ASIL_D, self.asil_compliance(AsilLevel::ASIL_D)),
            ].into_iter().collect(),
            unverified_count: self.get_unverified_requirements().len(),
            missing_implementation_count: self.get_requirements_needing_implementation().len(),
            missing_testing_count: self.get_requirements_needing_testing().len(),
        }
    }
}

impl Default for RequirementRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Compliance report summarizing requirement verification status
#[derive(Debug)]
pub struct ComplianceReport {
    pub total_requirements: usize,
    pub verified_requirements: usize,
    pub overall_compliance: f64,
    pub asil_compliance: std::collections::HashMap<AsilLevel, f64>,
    pub unverified_count: usize,
    pub missing_implementation_count: usize,
    pub missing_testing_count: usize,
}

impl ComplianceReport {
    /// Check if the system meets minimum compliance threshold
    pub fn meets_compliance_threshold(&self, threshold: f64) -> bool {
        self.overall_compliance >= threshold
    }
    
    /// Get the lowest ASIL compliance level
    pub fn lowest_asil_compliance(&self) -> Option<(AsilLevel, f64)> {
        self.asil_compliance.iter()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(core::cmp::Ordering::Equal))
            .map(|(asil, compliance)| (*asil, *compliance))
    }
}

/// Macro for creating safety requirements with traceability
#[macro_export]
macro_rules! safety_requirement {
    (
        id: $id:literal,
        title: $title:literal,
        description: $desc:literal,
        type: $req_type:expr,
        asil: $asil:expr,
        verification: $verification:expr
    ) => {
        {
            let mut req = SafetyRequirement::new(
                RequirementId::new($id),
                $title.to_string(),
                $desc.to_string(),
                $req_type,
                $asil,
            );
            req.verification_method = $verification;
            req
        }
    };
}

/// Macro for linking tests to requirements
#[macro_export]
macro_rules! test_requirement {
    ($req_id:expr, $test_name:expr) => {
        #[cfg(test)]
        inventory::submit! {
            RequirementTestMapping {
                requirement_id: $req_id,
                test_name: $test_name,
                test_module: module_path!(),
            }
        }
    };
}

/// Mapping between requirements and tests for automated verification
#[derive(Debug)]
pub struct RequirementTestMapping {
    pub requirement_id: &'static str,
    pub test_name: &'static str,
    pub test_module: &'static str,
}

// Note: inventory crate would be used for collecting test mappings at runtime
// For now, we'll use a simpler approach

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_requirement_creation() {
        let req = SafetyRequirement::new(
            RequirementId::new("REQ_MEM_001"),
            "Memory Safety".to_string(),
            "All memory allocations must be bounded".to_string(),
            RequirementType::Memory,
            AsilLevel::ASIL_C,
        );
        
        assert_eq!(req.id.as_str(), "REQ_MEM_001");
        assert_eq!(req.asil_level, AsilLevel::ASIL_C);
        assert!(!req.is_verified());
        assert!(req.needs_implementation());
    }
    
    #[test]
    fn test_requirement_registry() {
        let mut registry = RequirementRegistry::new();
        
        let req = SafetyRequirement::new(
            RequirementId::new("REQ_SAFETY_001"),
            "Safety Context".to_string(),
            "Runtime must maintain safety context".to_string(),
            RequirementType::Safety,
            AsilLevel::ASIL_D,
        );
        
        registry.add_requirement(req);
        
        assert_eq!(registry.requirements.len(), 1);
        assert!(registry.get_requirement(&RequirementId::new("REQ_SAFETY_001")).is_some());
        
        let compliance = registry.overall_compliance();
        assert!(compliance < 1.0); // Should be less than 100% since not verified
    }
    
    #[test]
    fn test_compliance_calculation() {
        let mut req = SafetyRequirement::new(
            RequirementId::new("REQ_TEST_001"),
            "Test Requirement".to_string(),
            "Test description".to_string(),
            RequirementType::Functional,
            AsilLevel::ASIL_A,
        );
        
        // Initially no compliance
        assert_eq!(req.compliance_score(), 0.0);
        
        // Add implementation
        req.add_implementation("src/test.rs".to_string());
        assert!(req.compliance_score() > 0.0);
        
        // Add testing
        req.set_coverage(CoverageLevel::Comprehensive);
        assert!(req.compliance_score() > 0.5);
        
        // Mark as verified
        req.set_status(VerificationStatus::Verified);
        assert!(req.compliance_score() > 0.8);
    }
    
    #[test]
    fn test_safety_requirement_macro() {
        let req = safety_requirement! {
            id: "REQ_MACRO_001",
            title: "Macro Test",
            description: "Test the safety requirement macro",
            type: RequirementType::Functional,
            asil: AsilLevel::ASIL_B,
            verification: VerificationMethod::Test
        };
        
        assert_eq!(req.id.as_str(), "REQ_MACRO_001");
        assert_eq!(req.verification_method, VerificationMethod::Test);
    }
}