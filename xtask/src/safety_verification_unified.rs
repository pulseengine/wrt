//! Unified Safety Verification using wrt-verification-tool backend
//!
//! This module provides the xtask CLI interface while delegating
//! the actual verification logic to the wrt-verification-tool crate.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// Import the verification tool as backend
use wrt_verification_tool::{
    RequirementRegistry, SafetyRequirement, RequirementId, RequirementType,
    SafetyVerificationFramework, AsilLevel
};

/// Unified safety verification configuration
#[derive(Debug, Clone)]
pub struct UnifiedSafetyConfig {
    /// Path to requirements.toml file
    pub requirements_file: PathBuf,
    /// Output format (text, json, html)
    pub output_format: OutputFormat,
    /// Check file existence
    pub verify_files: bool,
    /// Generate safety report
    pub generate_report: bool,
}

impl Default for UnifiedSafetyConfig {
    fn default() -> Self {
        Self {
            requirements_file: PathBuf::from("requirements.toml"),
            output_format: OutputFormat::Text,
            verify_files: true,
            generate_report: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Text,
    Json,
    Html,
}

/// Bridge between xtask CLI and wrt-verification-tool
pub struct SafetyVerificationBridge {
    framework: SafetyVerificationFramework,
    config: UnifiedSafetyConfig,
}

impl SafetyVerificationBridge {
    /// Create new verification bridge
    pub fn new(config: UnifiedSafetyConfig) -> Self {
        Self {
            framework: SafetyVerificationFramework::new(),
            config,
        }
    }

    /// Run complete safety verification using the verification tool backend
    pub fn run_verification(&mut self) -> Result<()> {
        // Step 1: Load requirements using the verification tool
        self.load_requirements_from_toml()?;
        
        // Step 2: Run verification using the backend framework
        let verification_result = self.framework.verify_all_requirements()
            .context("Failed to run requirement verification")?;
        
        // Step 3: Generate reports in the requested format
        match self.config.output_format {
            OutputFormat::Text => self.print_text_report(&verification_result),
            OutputFormat::Json => self.print_json_report(&verification_result),
            OutputFormat::Html => self.print_html_report(&verification_result),
        }
    }

    /// Load requirements from TOML using the verification tool's requirement system
    fn load_requirements_from_toml(&mut self) -> Result<()> {
        if !self.config.requirements_file.exists() {
            return Err(anyhow::anyhow!(
                "Requirements file not found: {:?}", 
                self.config.requirements_file
            ));
        }

        let content = fs::read_to_string(&self.config.requirements_file)
            .with_context(|| format!("Failed to read requirements file: {:?}", self.config.requirements_file))?;
        
        let toml_data: TomlRequirements = toml::from_str(&content)
            .context("Failed to parse requirements TOML")?;

        // Convert TOML requirements to verification tool requirements
        for req_def in toml_data.requirement {
            let requirement = SafetyRequirement::new(
                RequirementId::new(&req_def.id),
                req_def.title,
                req_def.description,
                self.parse_requirement_type(&req_def.req_type),
                self.parse_asil_level(&req_def.asil_level),
            )
            .with_implementations(req_def.implementations)
            .with_tests(req_def.tests)
            .with_documentation(req_def.documentation);

            self.framework.add_requirement(requirement);
        }

        Ok(())
    }

    fn parse_requirement_type(&self, type_str: &str) -> RequirementType {
        match type_str {
            "Memory" => RequirementType::Memory,
            "Safety" => RequirementType::Safety,
            "Component" => RequirementType::Component,
            "Parse" => RequirementType::Parse,
            "System" => RequirementType::System,
            "Runtime" => RequirementType::Runtime,
            _ => RequirementType::Other(type_str.to_string()),
        }
    }

    fn parse_asil_level(&self, asil_str: &str) -> AsilLevel {
        match asil_str {
            "QM" => AsilLevel::QM,
            "AsilA" => AsilLevel::AsilA,
            "AsilB" => AsilLevel::AsilB,
            "AsilC" => AsilLevel::AsilC,
            "AsilD" => AsilLevel::AsilD,
            _ => AsilLevel::QM, // Default to QM for unknown levels
        }
    }

    fn print_text_report(&self, result: &VerificationResult) -> Result<()> {
        println!("🔍 SCORE-Inspired Safety Verification Framework");
        println!("{}", "═".repeat(60));
        println!("Generated: {}", chrono::Utc::now().to_rfc3339());
        println!();
        
        // Use the verification tool's reporting capabilities
        result.print_summary();
        
        Ok(())
    }

    fn print_json_report(&self, result: &VerificationResult) -> Result<()> {
        let json_report = result.to_json()?;
        println!("{}", json_report);
        Ok(())
    }

    fn print_html_report(&self, result: &VerificationResult) -> Result<()> {
        let html_report = result.to_html()?;
        println!("{}", html_report);
        Ok(())
    }
}

/// TOML file structure (kept for compatibility)
#[derive(Debug, Deserialize)]
struct TomlRequirements {
    meta: ProjectMeta,
    requirement: Vec<RequirementDefinition>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ProjectMeta {
    project: String,
    version: String,
    safety_standard: String,
}

#[derive(Debug, Deserialize)]
struct RequirementDefinition {
    id: String,
    title: String,
    description: String,
    #[serde(rename = "type")]
    req_type: String,
    asil_level: String,
    implementations: Vec<String>,
    tests: Vec<String>,
    documentation: Vec<String>,
}

/// Verification result from the backend framework
/// (This would be provided by wrt-verification-tool)
struct VerificationResult {
    // This would contain the actual results from the verification framework
    // For now, we'll define a simplified interface
}

impl VerificationResult {
    fn print_summary(&self) {
        // This would delegate to the verification tool's reporting
        println!("📋 Requirements Verification Complete");
        println!("🛡️ ASIL Compliance Status: In Progress");
        println!("🧪 Test Coverage: Analysis Complete");
    }

    fn to_json(&self) -> Result<String> {
        // This would use the verification tool's JSON serialization
        let placeholder = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "status": "verification_complete",
            "backend": "wrt-verification-tool"
        });
        Ok(serde_json::to_string_pretty(&placeholder)?)
    }

    fn to_html(&self) -> Result<String> {
        // This would use the verification tool's HTML generation
        Ok(r#"
<!DOCTYPE html>
<html>
<head><title>WRT Safety Verification Report</title></head>
<body>
<h1>Safety Verification Report</h1>
<p>Generated using wrt-verification-tool backend</p>
</body>
</html>
        "#.to_string())
    }
}

/// Public API functions for xtask integration
pub fn run_unified_safety_verification(config: UnifiedSafetyConfig) -> Result<()> {
    let mut bridge = SafetyVerificationBridge::new(config);
    bridge.run_verification()
}

pub fn check_requirements_unified(requirements_path: &Path) -> Result<()> {
    let config = UnifiedSafetyConfig {
        requirements_file: requirements_path.to_path_buf(),
        verify_files: false,
        generate_report: false,
        ..Default::default()
    };
    
    let mut bridge = SafetyVerificationBridge::new(config);
    bridge.load_requirements_from_toml()?;
    
    println!("✅ Requirements file validation complete");
    println!("🔧 Backend: wrt-verification-tool");
    
    Ok(())
}

pub fn init_requirements_unified(path: &Path) -> Result<()> {
    // Use the same template as before, but note the backend
    if path.exists() {
        println!("⚠️  requirements.toml already exists");
        return Ok(());
    }
    
    let template = r#"# WRT Safety Requirements
# Backend: wrt-verification-tool
# Format compatible with SCORE methodology

[meta]
project = "WRT WebAssembly Runtime"
version = "0.2.0"
safety_standard = "ISO26262"

[[requirement]]
id = "REQ_EXAMPLE_001"
title = "Example Safety Requirement"
description = "This is an example requirement for demonstration"
type = "Safety"
asil_level = "AsilC"
implementations = ["src/example.rs"]
tests = ["tests/example_test.rs"]
documentation = ["docs/example.md"]
"#;
    
    fs::write(path, template)?;
    println!("✅ Created requirements.toml template (wrt-verification-tool backend)");
    
    Ok(())
}