//! Documentation validation commands for xtask

use anyhow::Result;
use std::path::Path;

/// Validate that required documentation files exist
pub fn validate_docs() -> Result<()> {
    println!("📚 Validating documentation files...");
    
    let required_docs = vec![
        "docs/conversion_audit.md",
        "docs/conversion_architecture.md", 
        "docs/conversion_review_complete.md",
    ];
    
    let mut all_exist = true;
    
    for doc_path in &required_docs {
        let path = Path::new(doc_path);
        if path.exists() {
            println!("  ✅ {}", doc_path);
        } else {
            println!("  ❌ {} (missing)", doc_path);
            all_exist = false;
        }
    }
    
    if all_exist {
        println!("✅ Documentation validation passed!");
        println!("📋 All required documentation files exist:");
        for doc_path in &required_docs {
            println!("   - {}", doc_path);
        }
        Ok(())
    } else {
        Err(anyhow::anyhow!("Some required documentation files are missing"))
    }
}

/// Run comprehensive documentation checks
pub fn check_docs_comprehensive() -> Result<()> {
    println!("🔍 Running comprehensive documentation checks...");
    
    // Basic file existence validation
    validate_docs()?;
    
    // Check for common documentation issues
    check_docs_structure()?;
    check_docs_links()?;
    
    println!("✅ Comprehensive documentation check completed!");
    Ok(())
}

/// Check documentation structure
fn check_docs_structure() -> Result<()> {
    println!("📁 Checking documentation structure...");
    
    let expected_dirs = vec![
        "docs/source",
        "docs/source/architecture",
        "docs/source/development", 
        "docs/source/examples",
        "docs/source/getting_started",
        "docs/source/overview",
        "docs/source/qualification",
        "docs/source/requirements",
        "docs/source/safety",
    ];
    
    for dir_path in &expected_dirs {
        let path = Path::new(dir_path);
        if path.exists() && path.is_dir() {
            println!("  ✅ {}/", dir_path);
        } else {
            println!("  ⚠️  {} (missing or not a directory)", dir_path);
        }
    }
    
    Ok(())
}

/// Basic check for broken internal links (simplified)
fn check_docs_links() -> Result<()> {
    println!("🔗 Checking documentation links...");
    
    // This is a simplified check - a full implementation would parse RST/MD files
    // and validate internal references
    
    let key_files = vec![
        "docs/source/index.rst",
        "docs/source/architecture/index.rst",
        "docs/source/development/index.rst",
        "docs/source/examples/index.rst",
    ];
    
    for file_path in &key_files {
        let path = Path::new(file_path);
        if path.exists() {
            println!("  ✅ {}", file_path);
        } else {
            println!("  ⚠️  {} (key file missing)", file_path);
        }
    }
    
    println!("💡 For detailed link checking, use: sphinx-build -b linkcheck");
    
    Ok(())
}