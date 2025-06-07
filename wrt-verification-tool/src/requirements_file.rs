use std::fs;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RequirementsFile {
    pub meta: ProjectMeta,
    pub requirement: Vec<RequirementDefinition>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectMeta {
    pub project: String,
    pub version: String,
    pub safety_standard: String,
}

#[derive(Debug, Deserialize)]
pub struct RequirementDefinition {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "type")]
    pub req_type: String,
    pub asil_level: String,
    pub implementations: Vec<String>,
    pub tests: Vec<String>,
    pub documentation: Vec<String>,
}

impl RequirementsFile {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let req_file: RequirementsFile = toml::from_str(&content)?;
        Ok(req_file)
    }
    
    pub fn verify_files_exist(&self) -> Vec<String> {
        let mut missing = Vec::new();
        
        for req in &self.requirement {
            for impl_file in &req.implementations {
                if !std::path::Path::new(impl_file).exists() {
                    missing.push(format!("Implementation: {}", impl_file));
                }
            }
            for test_file in &req.tests {
                if !std::path::Path::new(test_file).exists() {
                    missing.push(format!("Test: {}", test_file));
                }
            }
            for doc_file in &req.documentation {
                if !std::path::Path::new(doc_file).exists() {
                    missing.push(format!("Documentation: {}", doc_file));
                }
            }
        }
        
        missing
    }
    
    pub fn get_requirements_by_asil(&self, asil_level: &str) -> Vec<&RequirementDefinition> {
        self.requirement.iter()
            .filter(|req| req.asil_level == asil_level)
            .collect()
    }
    
    pub fn get_requirements_by_type(&self, req_type: &str) -> Vec<&RequirementDefinition> {
        self.requirement.iter()
            .filter(|req| req.req_type == req_type)
            .collect()
    }
    
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("📋 Requirements Report for {}\n", self.meta.project));
        report.push_str(&format!("Version: {}\n", self.meta.version));
        report.push_str(&format!("Safety Standard: {}\n", self.meta.safety_standard));
        report.push_str(&format!("Total Requirements: {}\n\n", self.requirement.len()));
        
        // ASIL breakdown
        report.push_str("🛡️  ASIL Level Breakdown:\n");
        let mut asil_counts = std::collections::HashMap::new();
        for req in &self.requirement {
            *asil_counts.entry(&req.asil_level).or_insert(0) += 1;
        }
        for (asil, count) in asil_counts {
            report.push_str(&format!("  {}: {} requirements\n", asil, count));
        }
        report.push_str("\n");
        
        // Type breakdown
        report.push_str("📂 Requirement Type Breakdown:\n");
        let mut type_counts = std::collections::HashMap::new();
        for req in &self.requirement {
            *type_counts.entry(&req.req_type).or_insert(0) += 1;
        }
        for (req_type, count) in type_counts {
            report.push_str(&format!("  {}: {} requirements\n", req_type, count));
        }
        report.push_str("\n");
        
        // File verification
        let missing_files = self.verify_files_exist();
        if missing_files.is_empty() {
            report.push_str("✅ All referenced files exist\n");
        } else {
            report.push_str("❌ Missing files:\n");
            for file in missing_files {
                report.push_str(&format!("  • {}\n", file));
            }
        }
        
        report
    }
}