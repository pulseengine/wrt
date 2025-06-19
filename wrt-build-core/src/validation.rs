//! Code validation utilities
//!
//! This module provides various code validation checks to ensure
//! code quality and organization standards.

use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{BuildError, BuildResult};

/// Validation results
#[derive(Debug)]
pub struct ValidationResults {
    /// Whether all validations passed
    pub success: bool,
    /// List of errors found
    pub errors: Vec<ValidationError>,
    /// List of warnings
    pub warnings: Vec<String>,
}

/// Validation error details
#[derive(Debug)]
pub struct ValidationError {
    /// Error category
    pub category: String,
    /// File path where error occurred
    pub file: PathBuf,
    /// Error message
    pub message: String,
}

/// Code validator
pub struct CodeValidator {
    workspace_root: PathBuf,
    verbose: bool,
}

impl CodeValidator {
    /// Create a new code validator
    pub fn new(workspace_root: PathBuf, verbose: bool) -> Self {
        Self {
            workspace_root,
            verbose,
        }
    }
    
    /// Check for test files in src/ directories
    pub fn check_no_test_files_in_src(&self) -> BuildResult<ValidationResults> {
        println!("{} Checking for test files in src/ directories...", "🔍".bright_blue());
        
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Walk through all Cargo.toml files to find crates
        let crates = self.find_workspace_crates()?;
        
        for crate_path in crates {
            let src_dir = crate_path.join("src");
            if src_dir.exists() {
                self.check_directory_for_test_files(&src_dir, &mut errors)?;
            }
        }
        
        let success = errors.is_empty();
        
        if success {
            println!("{} No test files found in src/ directories", "✅".bright_green());
        } else {
            println!("{} Found {} test files in src/ directories", "❌".bright_red(), errors.len());
            for error in &errors {
                println!("  - {}", error.file.display());
            }
        }
        
        Ok(ValidationResults {
            success,
            errors,
            warnings,
        })
    }
    
    /// Check module documentation coverage
    pub fn check_module_documentation(&self) -> BuildResult<ValidationResults> {
        println!("{} Checking module documentation coverage...", "📚".bright_blue());
        
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut total_modules = 0;
        let mut documented_modules = 0;
        
        // Walk through all Rust files
        let crates = self.find_workspace_crates()?;
        
        for crate_path in crates {
            let src_dir = crate_path.join("src");
            if src_dir.exists() {
                self.check_directory_documentation(&src_dir, &mut total_modules, &mut documented_modules, &mut warnings)?;
            }
        }
        
        let coverage_percent = if total_modules > 0 {
            (documented_modules as f64 / total_modules as f64) * 100.0
        } else {
            100.0
        };
        
        println!("{} Module documentation coverage: {:.1}%", "📊".bright_cyan(), coverage_percent);
        println!("  Documented: {}/{}", documented_modules, total_modules);
        
        if coverage_percent < 80.0 {
            errors.push(ValidationError {
                category: "documentation".to_string(),
                file: self.workspace_root.clone(),
                message: format!("Module documentation coverage {:.1}% is below 80% threshold", coverage_percent),
            });
        }
        
        Ok(ValidationResults {
            success: errors.is_empty(),
            errors,
            warnings,
        })
    }
    
    /// Find all workspace crates
    fn find_workspace_crates(&self) -> BuildResult<Vec<PathBuf>> {
        let mut crates = vec![self.workspace_root.clone()];
        
        // Read workspace Cargo.toml
        let workspace_toml = self.workspace_root.join("Cargo.toml");
        if let Ok(content) = fs::read_to_string(&workspace_toml) {
            // Simple parsing - look for workspace members
            if let Some(members_start) = content.find("members = [") {
                if let Some(members_end) = content[members_start..].find(']') {
                    let members_section = &content[members_start..members_start + members_end];
                    for line in members_section.lines() {
                        let line = line.trim();
                        if line.starts_with('"') && line.ends_with('"') {
                            let member = line.trim_matches('"').trim_matches(',');
                            let member_path = self.workspace_root.join(member);
                            if member_path.exists() {
                                crates.push(member_path);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(crates)
    }
    
    /// Check a directory recursively for test files
    fn check_directory_for_test_files(&self, dir: &Path, errors: &mut Vec<ValidationError>) -> BuildResult<()> {
        for entry in fs::read_dir(dir)
            .map_err(|e| BuildError::Tool(format!("Failed to read directory {}: {}", dir.display(), e)))?
        {
            let entry = entry.map_err(|e| BuildError::Tool(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    // Check for test files
                    if file_name.ends_with("_test.rs") || 
                       file_name.ends_with("_tests.rs") ||
                       file_name == "test.rs" ||
                       file_name == "tests.rs" ||
                       (file_name.contains("test") && file_name.ends_with(".rs") && !file_name.contains("test_utils")) {
                        errors.push(ValidationError {
                            category: "test_file_location".to_string(),
                            file: path.clone(),
                            message: format!("Test file '{}' should be in tests/ directory, not in src/", file_name),
                        });
                    }
                }
            } else if path.is_dir() {
                // Skip target and .git directories
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name != "target" && dir_name != ".git" {
                        self.check_directory_for_test_files(&path, errors)?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Check directory for module documentation
    fn check_directory_documentation(
        &self, 
        dir: &Path, 
        total_modules: &mut usize, 
        documented_modules: &mut usize,
        warnings: &mut Vec<String>
    ) -> BuildResult<()> {
        for entry in fs::read_dir(dir)
            .map_err(|e| BuildError::Tool(format!("Failed to read directory {}: {}", dir.display(), e)))?
        {
            let entry = entry.map_err(|e| BuildError::Tool(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.ends_with(".rs") && file_name != "lib.rs" && file_name != "main.rs" {
                        *total_modules += 1;
                        
                        // Check if file has module documentation
                        if let Ok(content) = fs::read_to_string(&path) {
                            let lines: Vec<&str> = content.lines().collect();
                            let mut has_module_doc = false;
                            
                            // Look for //! at the start of the file
                            for line in lines.iter().take(10) {
                                let trimmed = line.trim();
                                if trimmed.starts_with("//!") {
                                    has_module_doc = true;
                                    break;
                                }
                                // Skip empty lines and attributes
                                if !trimmed.is_empty() && !trimmed.starts_with("#[") && !trimmed.starts_with("#!") {
                                    break;
                                }
                            }
                            
                            if has_module_doc {
                                *documented_modules += 1;
                            } else if self.verbose {
                                warnings.push(format!("Missing module documentation: {}", path.display()));
                            }
                        }
                    }
                }
            } else if path.is_dir() {
                // Skip certain directories
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if dir_name != "target" && dir_name != ".git" && dir_name != "tests" && dir_name != "benches" {
                        // Check mod.rs in subdirectories
                        let mod_file = path.join("mod.rs");
                        if mod_file.exists() {
                            *total_modules += 1;
                            
                            if let Ok(content) = fs::read_to_string(&mod_file) {
                                if content.trim_start().starts_with("//!") {
                                    *documented_modules += 1;
                                } else if self.verbose {
                                    warnings.push(format!("Missing module documentation: {}", mod_file.display()));
                                }
                            }
                        }
                        
                        self.check_directory_documentation(&path, total_modules, documented_modules, warnings)?;
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// Run all validation checks
pub fn run_all_validations(workspace_root: &Path, verbose: bool) -> BuildResult<bool> {
    let validator = CodeValidator::new(workspace_root.to_path_buf(), verbose);
    
    println!("{} Running code validation checks...", "🔍".bright_blue());
    println!();
    
    let mut all_passed = true;
    
    // Check for test files in src/
    let test_files_result = validator.check_no_test_files_in_src()?;
    if !test_files_result.success {
        all_passed = false;
    }
    
    println!();
    
    // Check module documentation
    let doc_result = validator.check_module_documentation()?;
    if !doc_result.success {
        all_passed = false;
    }
    
    println!();
    
    if all_passed {
        println!("{} All validation checks passed!", "✅".bright_green());
    } else {
        println!("{} Some validation checks failed", "❌".bright_red());
    }
    
    Ok(all_passed)
}