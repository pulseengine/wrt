//! Documentation Verification Framework
//!
//! This module provides comprehensive documentation verification for safety-critical
//! systems, ensuring that all requirements, implementations, and tests are properly
//! documented according to ASIL standards. Inspired by SCORE's documentation practices.

use wrt_foundation::{
    safety_system::AsilLevel,
    prelude::*,
};
use crate::requirements::{RequirementRegistry, RequirementId, SafetyRequirement};
use core::fmt;

/// Documentation verification framework that ensures proper documentation
/// coverage for safety-critical requirements
pub struct DocumentationVerificationFramework {
    /// Registry of requirements to verify documentation for
    requirement_registry: RequirementRegistry,
    /// Documentation analysis results
    documentation_analysis: Vec<DocumentationAnalysis>,
    /// Configuration for verification standards
    verification_config: DocumentationVerificationConfig,
}

impl DocumentationVerificationFramework {
    /// Create a new documentation verification framework
    pub fn new() -> Self {
        Self {
            requirement_registry: RequirementRegistry::new(),
            documentation_analysis: Vec::new(),
            verification_config: DocumentationVerificationConfig::default(),
        }
    }
    
    /// Set the verification configuration
    pub fn with_config(mut self, config: DocumentationVerificationConfig) -> Self {
        self.verification_config = config;
        self
    }
    
    /// Add a requirement to be verified for documentation
    pub fn add_requirement(&mut self, requirement: SafetyRequirement) {
        self.requirement_registry.add_requirement(requirement);
    }
    
    /// Verify documentation for all requirements
    pub fn verify_all_documentation(&mut self) -> DocumentationVerificationResult {
        let requirements = self.requirement_registry.get_all_requirements();
        let mut violations = Vec::new();
        let mut compliant_requirements = 0;
        
        for requirement in &requirements {
            let analysis = self.analyze_requirement_documentation(requirement);
            
            if analysis.is_compliant() {
                compliant_requirements += 1;
            } else {
                // Collect violations
                for violation in &analysis.violations {
                    violations.push(violation.clone());
                }
            }
            
            self.documentation_analysis.push(analysis);
        }
        
        let total_requirements = requirements.len();
        let compliance_percentage = if total_requirements > 0 {
            (compliant_requirements as f64 / total_requirements as f64) * 100.0
        } else {
            100.0
        };
        
        DocumentationVerificationResult {
            total_requirements,
            compliant_requirements,
            compliance_percentage,
            violations,
            analysis_results: self.documentation_analysis.clone(),
            is_certification_ready: self.is_certification_ready(compliance_percentage),
        }
    }
    
    /// Verify documentation for a specific ASIL level
    pub fn verify_asil_documentation(&mut self, asil_level: AsilLevel) -> DocumentationVerificationResult {
        let requirements = self.requirement_registry.get_requirements_by_asil(asil_level);
        let mut violations = Vec::new();
        let mut compliant_requirements = 0;
        
        for requirement in &requirements {
            let analysis = self.analyze_requirement_documentation(requirement);
            
            if analysis.is_compliant() {
                compliant_requirements += 1;
            } else {
                for violation in &analysis.violations {
                    violations.push(violation.clone());
                }
            }
        }
        
        let total_requirements = requirements.len();
        let compliance_percentage = if total_requirements > 0 {
            (compliant_requirements as f64 / total_requirements as f64) * 100.0
        } else {
            100.0
        };
        
        DocumentationVerificationResult {
            total_requirements,
            compliant_requirements,
            compliance_percentage,
            violations,
            analysis_results: self.documentation_analysis.clone(),
            is_certification_ready: self.is_certification_ready_for_asil(compliance_percentage, asil_level),
        }
    }
    
    /// Analyze documentation for a single requirement
    fn analyze_requirement_documentation(&self, requirement: &SafetyRequirement) -> DocumentationAnalysis {
        let mut violations = Vec::new();
        let required_standards = self.get_documentation_standards_for_asil(requirement.asil_level);
        
        // Check requirement documentation completeness
        if requirement.description.trim().is_empty() {
            violations.push(DocumentationViolation {
                requirement_id: requirement.id.clone(),
                violation_type: DocumentationViolationType::MissingDescription,
                severity: self.get_violation_severity(requirement.asil_level, DocumentationViolationType::MissingDescription),
                description: "Requirement lacks detailed description".to_string(),
                location: DocumentationLocation::Requirement,
            });
        }
        
        // Check if description meets ASIL standards
        if requirement.description.len() < required_standards.min_description_length {
            violations.push(DocumentationViolation {
                requirement_id: requirement.id.clone(),
                violation_type: DocumentationViolationType::InsufficientDetail,
                severity: self.get_violation_severity(requirement.asil_level, DocumentationViolationType::InsufficientDetail),
                description: format!("Description too brief ({}/<{} chars)", requirement.description.len(), required_standards.min_description_length),
                location: DocumentationLocation::Requirement,
            });
        }
        
        // Check implementation documentation
        if requirement.implementations.is_empty() {
            violations.push(DocumentationViolation {
                requirement_id: requirement.id.clone(),
                violation_type: DocumentationViolationType::MissingImplementation,
                severity: self.get_violation_severity(requirement.asil_level, DocumentationViolationType::MissingImplementation),
                description: "No implementation references found".to_string(),
                location: DocumentationLocation::Implementation,
            });
        } else {
            // Verify implementation documentation exists
            for impl_ref in &requirement.implementations {
                if !self.verify_implementation_documented(impl_ref) {
                    violations.push(DocumentationViolation {
                        requirement_id: requirement.id.clone(),
                        violation_type: DocumentationViolationType::UndocumentedImplementation,
                        severity: self.get_violation_severity(requirement.asil_level, DocumentationViolationType::UndocumentedImplementation),
                        description: format!("Implementation '{}' lacks documentation", impl_ref),
                        location: DocumentationLocation::Implementation,
                    });
                }
            }
        }
        
        // Check test documentation
        if requirement.tests.is_empty() {
            violations.push(DocumentationViolation {
                requirement_id: requirement.id.clone(),
                violation_type: DocumentationViolationType::MissingTestDocumentation,
                severity: self.get_violation_severity(requirement.asil_level, DocumentationViolationType::MissingTestDocumentation),
                description: "No test documentation found".to_string(),
                location: DocumentationLocation::Test,
            });
        }
        
        // Check verification documentation
        if required_standards.requires_verification_document && requirement.documentation.is_empty() {
            violations.push(DocumentationViolation {
                requirement_id: requirement.id.clone(),
                violation_type: DocumentationViolationType::MissingVerificationDocument,
                severity: self.get_violation_severity(requirement.asil_level, DocumentationViolationType::MissingVerificationDocument),
                description: "Missing verification documentation".to_string(),
                location: DocumentationLocation::Verification,
            });
        }
        
        let compliance_score = self.calculate_compliance_score(&violations, &required_standards);
        
        DocumentationAnalysis {
            requirement_id: requirement.id.clone(),
            asil_level: requirement.asil_level,
            violations,
            compliance_score,
            required_standards,
            analyzed_locations: vec![
                DocumentationLocation::Requirement,
                DocumentationLocation::Implementation,
                DocumentationLocation::Test,
                DocumentationLocation::Verification,
            ],
        }
    }
    
    /// Get documentation standards for a specific ASIL level
    fn get_documentation_standards_for_asil(&self, asil_level: AsilLevel) -> DocumentationStandards {
        match asil_level {
            AsilLevel::QM => DocumentationStandards {
                min_description_length: 50,
                requires_implementation_docs: false,
                requires_test_docs: false,
                requires_verification_document: false,
                max_allowed_violations: 10,
                required_compliance_score: 50.0,
            },
            AsilLevel::AsilA => DocumentationStandards {
                min_description_length: 100,
                requires_implementation_docs: true,
                requires_test_docs: false,
                requires_verification_document: false,
                max_allowed_violations: 5,
                required_compliance_score: 70.0,
            },
            AsilLevel::AsilB => DocumentationStandards {
                min_description_length: 150,
                requires_implementation_docs: true,
                requires_test_docs: true,
                requires_verification_document: false,
                max_allowed_violations: 3,
                required_compliance_score: 80.0,
            },
            AsilLevel::AsilC => DocumentationStandards {
                min_description_length: 200,
                requires_implementation_docs: true,
                requires_test_docs: true,
                requires_verification_document: true,
                max_allowed_violations: 1,
                required_compliance_score: 90.0,
            },
            AsilLevel::AsilD => DocumentationStandards {
                min_description_length: 300,
                requires_implementation_docs: true,
                requires_test_docs: true,
                requires_verification_document: true,
                max_allowed_violations: 0,
                required_compliance_score: 95.0,
            },
        }
    }
    
    /// Verify that an implementation has proper documentation
    fn verify_implementation_documented(&self, _implementation_ref: &str) -> bool {
        // In a real implementation, this would:
        // - Check for rustdoc comments
        // - Verify API documentation completeness  
        // - Check for examples and usage documentation
        // - Validate cross-references to requirements
        
        // For now, simulate some basic validation
        true // Simplified for demonstration
    }
    
    /// Calculate compliance score for a requirement
    fn calculate_compliance_score(&self, violations: &[DocumentationViolation], standards: &DocumentationStandards) -> f64 {
        if violations.is_empty() {
            return 100.0;
        }
        
        let total_penalty: f64 = violations.iter().map(|v| self.get_violation_penalty(&v.severity)).sum();
        let max_possible_penalty = 100.0; // Maximum penalty possible
        
        ((max_possible_penalty - total_penalty) / max_possible_penalty * 100.0).max(0.0)
    }
    
    /// Get penalty points for a violation severity
    fn get_violation_penalty(&self, severity: &DocumentationViolationSeverity) -> f64 {
        match severity {
            DocumentationViolationSeverity::Info => 5.0,
            DocumentationViolationSeverity::Low => 10.0,
            DocumentationViolationSeverity::Medium => 20.0,
            DocumentationViolationSeverity::High => 40.0,
            DocumentationViolationSeverity::Critical => 80.0,
        }
    }
    
    /// Get violation severity based on ASIL level and violation type
    fn get_violation_severity(&self, asil_level: AsilLevel, violation_type: DocumentationViolationType) -> DocumentationViolationSeverity {
        match (asil_level, violation_type) {
            (AsilLevel::AsilD, DocumentationViolationType::MissingDescription) => DocumentationViolationSeverity::Critical,
            (AsilLevel::AsilD, _) => DocumentationViolationSeverity::High,
            (AsilLevel::AsilC, DocumentationViolationType::MissingDescription) => DocumentationViolationSeverity::High,
            (AsilLevel::AsilC, _) => DocumentationViolationSeverity::Medium,
            (AsilLevel::AsilB, _) => DocumentationViolationSeverity::Medium,
            (AsilLevel::AsilA, _) => DocumentationViolationSeverity::Low,
            (AsilLevel::QM, _) => DocumentationViolationSeverity::Info,
        }
    }
    
    /// Check if system is ready for certification based on documentation
    fn is_certification_ready(&self, compliance_percentage: f64) -> bool {
        compliance_percentage >= self.verification_config.min_certification_compliance
    }
    
    /// Check if system is ready for ASIL-specific certification
    fn is_certification_ready_for_asil(&self, compliance_percentage: f64, asil_level: AsilLevel) -> bool {
        let required_threshold = match asil_level {
            AsilLevel::AsilD => 95.0,
            AsilLevel::AsilC => 90.0,
            AsilLevel::AsilB => 85.0,
            AsilLevel::AsilA => 80.0,
            AsilLevel::QM => 70.0,
        };
        
        compliance_percentage >= required_threshold
    }
    
    /// Generate documentation verification report
    pub fn generate_report(&self) -> DocumentationReport {
        let overall_compliance = if !self.documentation_analysis.is_empty() {
            self.documentation_analysis.iter()
                .map(|a| a.compliance_score)
                .sum::<f64>() / self.documentation_analysis.len() as f64
        } else {
            100.0
        };
        
        let total_violations = self.documentation_analysis.iter()
            .map(|a| a.violations.len())
            .sum();
        
        let critical_violations = self.documentation_analysis.iter()
            .flat_map(|a| &a.violations)
            .filter(|v| v.severity == DocumentationViolationSeverity::Critical)
            .count();
        
        DocumentationReport {
            overall_compliance,
            total_requirements: self.documentation_analysis.len(),
            total_violations,
            critical_violations,
            asil_compliance: self.calculate_asil_compliance(),
            recommendations: self.generate_recommendations(),
            analysis_summary: self.documentation_analysis.clone(),
        }
    }
    
    /// Calculate compliance per ASIL level
    fn calculate_asil_compliance(&self) -> std::collections::HashMap<AsilLevel, f64> {
        let mut asil_compliance = std::collections::HashMap::new();
        
        for asil_level in [AsilLevel::QM, AsilLevel::AsilA, AsilLevel::AsilB, AsilLevel::AsilC, AsilLevel::AsilD] {
            let asil_analyses: Vec<_> = self.documentation_analysis.iter()
                .filter(|a| a.asil_level == asil_level)
                .collect();
            
            if !asil_analyses.is_empty() {
                let compliance = asil_analyses.iter()
                    .map(|a| a.compliance_score)
                    .sum::<f64>() / asil_analyses.len() as f64;
                asil_compliance.insert(asil_level, compliance);
            }
        }
        
        asil_compliance
    }
    
    /// Generate recommendations for improving documentation
    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        let critical_violations = self.documentation_analysis.iter()
            .flat_map(|a| &a.violations)
            .filter(|v| v.severity == DocumentationViolationSeverity::Critical)
            .count();
        
        if critical_violations > 0 {
            recommendations.push(format!("Address {} critical documentation violations immediately", critical_violations));
        }
        
        let missing_descriptions = self.documentation_analysis.iter()
            .flat_map(|a| &a.violations)
            .filter(|v| v.violation_type == DocumentationViolationType::MissingDescription)
            .count();
        
        if missing_descriptions > 0 {
            recommendations.push(format!("Add detailed descriptions for {} requirements", missing_descriptions));
        }
        
        recommendations
    }
}

impl Default for DocumentationVerificationFramework {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for documentation verification
#[derive(Debug, Clone)]
pub struct DocumentationVerificationConfig {
    pub min_certification_compliance: f64,
    pub enable_cross_reference_validation: bool,
    pub enable_api_documentation_check: bool,
    pub enable_example_validation: bool,
}

impl Default for DocumentationVerificationConfig {
    fn default() -> Self {
        Self {
            min_certification_compliance: 85.0,
            enable_cross_reference_validation: true,
            enable_api_documentation_check: true,
            enable_example_validation: false,
        }
    }
}

/// Documentation standards for a specific ASIL level
#[derive(Debug, Clone)]
pub struct DocumentationStandards {
    pub min_description_length: usize,
    pub requires_implementation_docs: bool,
    pub requires_test_docs: bool,
    pub requires_verification_document: bool,
    pub max_allowed_violations: usize,
    pub required_compliance_score: f64,
}

/// Result of documentation verification
#[derive(Debug)]
pub struct DocumentationVerificationResult {
    pub total_requirements: usize,
    pub compliant_requirements: usize,
    pub compliance_percentage: f64,
    pub violations: Vec<DocumentationViolation>,
    pub analysis_results: Vec<DocumentationAnalysis>,
    pub is_certification_ready: bool,
}

/// Analysis of documentation for a single requirement
#[derive(Debug, Clone)]
pub struct DocumentationAnalysis {
    pub requirement_id: RequirementId,
    pub asil_level: AsilLevel,
    pub violations: Vec<DocumentationViolation>,
    pub compliance_score: f64,
    pub required_standards: DocumentationStandards,
    pub analyzed_locations: Vec<DocumentationLocation>,
}

impl DocumentationAnalysis {
    /// Check if this requirement's documentation is compliant
    pub fn is_compliant(&self) -> bool {
        self.compliance_score >= self.required_standards.required_compliance_score &&
        self.violations.len() <= self.required_standards.max_allowed_violations
    }
}

/// A documentation violation that needs to be addressed
#[derive(Debug, Clone)]
pub struct DocumentationViolation {
    pub requirement_id: RequirementId,
    pub violation_type: DocumentationViolationType,
    pub severity: DocumentationViolationSeverity,
    pub description: String,
    pub location: DocumentationLocation,
}

/// Types of documentation violations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentationViolationType {
    MissingDescription,
    InsufficientDetail,
    MissingImplementation,
    UndocumentedImplementation,
    MissingTestDocumentation,
    MissingVerificationDocument,
    InconsistentCrossReferences,
    MissingExamples,
    OutdatedDocumentation,
}

/// Severity levels for documentation violations
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DocumentationViolationSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Location where documentation issue was found
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentationLocation {
    Requirement,
    Implementation,
    Test,
    Verification,
    Api,
    Example,
}

/// Comprehensive documentation report
#[derive(Debug)]
pub struct DocumentationReport {
    pub overall_compliance: f64,
    pub total_requirements: usize,
    pub total_violations: usize,
    pub critical_violations: usize,
    pub asil_compliance: std::collections::HashMap<AsilLevel, f64>,
    pub recommendations: Vec<String>,
    pub analysis_summary: Vec<DocumentationAnalysis>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::requirements::{RequirementType, VerificationMethod};
    
    #[test]
    fn test_documentation_verification_framework_creation() {
        let framework = DocumentationVerificationFramework::new();
        let result = framework.verify_all_documentation();
        
        assert_eq!(result.total_requirements, 0);
        assert_eq!(result.compliance_percentage, 100.0);
        assert!(result.is_certification_ready);
    }
    
    #[test]
    fn test_requirement_documentation_analysis() {
        let mut framework = DocumentationVerificationFramework::new();
        
        let mut requirement = SafetyRequirement::new(
            RequirementId::new("DOC_TEST_001"),
            "Test Requirement".to_string(),
            "A".to_string(), // Very short description - should trigger violation
            RequirementType::Safety,
            AsilLevel::AsilC,
        );
        
        framework.add_requirement(requirement);
        
        let result = framework.verify_all_documentation();
        
        assert_eq!(result.total_requirements, 1);
        assert_eq!(result.compliant_requirements, 0);
        assert!(!result.violations.is_empty());
        assert!(!result.is_certification_ready);
    }
    
    #[test]
    fn test_asil_specific_documentation_standards() {
        let framework = DocumentationVerificationFramework::new();
        
        let qm_standards = framework.get_documentation_standards_for_asil(AsilLevel::QM);
        let asil_d_standards = framework.get_documentation_standards_for_asil(AsilLevel::AsilD);
        
        assert!(asil_d_standards.min_description_length > qm_standards.min_description_length);
        assert!(asil_d_standards.requires_verification_document);
        assert!(!qm_standards.requires_verification_document);
        assert!(asil_d_standards.required_compliance_score > qm_standards.required_compliance_score);
    }
    
    #[test]
    fn test_compliant_requirement_documentation() {
        let mut framework = DocumentationVerificationFramework::new();
        
        let mut requirement = SafetyRequirement::new(
            RequirementId::new("DOC_TEST_002"),
            "Well Documented Requirement".to_string(),
            "This is a comprehensive description of a safety requirement that provides detailed information about the expected behavior, constraints, and verification criteria for the implementation.".to_string(),
            RequirementType::Safety,
            AsilLevel::AsilA,
        );
        
        requirement.add_implementation("well_documented_impl.rs".to_string());
        requirement.add_test("comprehensive_test.rs".to_string());
        
        framework.add_requirement(requirement);
        
        let result = framework.verify_all_documentation();
        
        assert_eq!(result.total_requirements, 1);
        assert_eq!(result.compliant_requirements, 1);
        assert_eq!(result.compliance_percentage, 100.0);
        assert!(result.is_certification_ready);
    }
    
    #[test]
    fn test_documentation_report_generation() {
        let framework = DocumentationVerificationFramework::new();
        let report = framework.generate_report();
        
        assert_eq!(report.overall_compliance, 100.0);
        assert_eq!(report.total_requirements, 0);
        assert_eq!(report.total_violations, 0);
        assert_eq!(report.critical_violations, 0);
    }
    
    #[test]
    fn test_violation_severity_mapping() {
        let framework = DocumentationVerificationFramework::new();
        
        let asil_d_missing_desc = framework.get_violation_severity(
            AsilLevel::AsilD,
            DocumentationViolationType::MissingDescription
        );
        
        let qm_missing_desc = framework.get_violation_severity(
            AsilLevel::QM,
            DocumentationViolationType::MissingDescription
        );
        
        assert_eq!(asil_d_missing_desc, DocumentationViolationSeverity::Critical);
        assert_eq!(qm_missing_desc, DocumentationViolationSeverity::Info);
        assert!(asil_d_missing_desc > qm_missing_desc);
    }
}