//! Changelog generation using git-cliff

use anyhow::Result;
use std::path::{Path, PathBuf};
use xshell::{cmd, Shell};

/// Configuration for changelog generation
#[derive(Debug, Clone)]
pub struct ChangelogConfig {
    pub output_file: PathBuf,
    pub unreleased_only: bool,
    pub install_if_missing: bool,
}

impl Default for ChangelogConfig {
    fn default() -> Self {
        Self {
            output_file: PathBuf::from("docs/source/changelog.md"),
            unreleased_only: false,
            install_if_missing: true,
        }
    }
}

/// Generate changelog using git-cliff
pub fn generate_changelog(config: ChangelogConfig) -> Result<()> {
    let sh = Shell::new()?;
    
    println!("📝 Generating changelog using git-cliff...");
    
    // Check if git-cliff is available
    if !is_git_cliff_installed(&sh)? {
        if config.install_if_missing {
            println!("📦 git-cliff not found, installing...");
            install_git_cliff(&sh)?;
        } else {
            return Err(anyhow::anyhow!(
                "git-cliff is not installed. Install it with: cargo install git-cliff"
            ));
        }
    }
    
    // Check if we're in a git repository
    check_git_repository(&sh)?;
    
    // Check if cliff.toml exists
    if !Path::new("cliff.toml").exists() {
        return Err(anyhow::anyhow!(
            "cliff.toml configuration file not found in workspace root"
        ));
    }
    
    // Create output directory if it doesn't exist
    if let Some(parent) = config.output_file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Check if working directory has uncommitted changes
    let has_changes = !cmd!(sh, "git diff --quiet HEAD").run().is_ok();
    
    // Generate the changelog
    println!("📄 Generating changelog to: {}", config.output_file.display());
    
    let mut cliff_cmd = cmd!(sh, "git-cliff");
    
    if config.unreleased_only || has_changes {
        if has_changes {
            println!("⚠️  Working directory has changes, generating unreleased changelog...");
        }
        cliff_cmd = cliff_cmd.arg("--unreleased");
    } else {
        println!("✨ Generating full changelog...");
    }
    
    cliff_cmd = cliff_cmd.args(&["--output", config.output_file.to_str().unwrap()]);
    cliff_cmd.run()?;
    
    println!("✅ Changelog generated successfully!");
    
    // Show preview
    show_changelog_preview(&config.output_file)?;
    
    Ok(())
}

/// Check if git-cliff is installed
fn is_git_cliff_installed(sh: &Shell) -> Result<bool> {
    Ok(cmd!(sh, "which git-cliff").run().is_ok() || 
       cmd!(sh, "where git-cliff").run().is_ok())
}

/// Install git-cliff using cargo
fn install_git_cliff(sh: &Shell) -> Result<()> {
    println!("🔧 Installing git-cliff...");
    cmd!(sh, "cargo install git-cliff").run()?;
    println!("✅ git-cliff installed successfully!");
    Ok(())
}

/// Check if we're in a git repository
fn check_git_repository(sh: &Shell) -> Result<()> {
    cmd!(sh, "git rev-parse --git-dir")
        .quiet()
        .run()
        .map_err(|_| anyhow::anyhow!("Not in a git repository"))?;
    Ok(())
}

/// Show preview of generated changelog
fn show_changelog_preview(changelog_path: &Path) -> Result<()> {
    if changelog_path.exists() {
        println!("\n📋 Preview (first 10 lines):");
        println!("─────────────────────────────");
        
        let content = std::fs::read_to_string(changelog_path)?;
        let lines: Vec<&str> = content.lines().take(10).collect();
        
        for line in lines {
            println!("{}", line);
        }
        
        if content.lines().count() > 10 {
            println!("... (truncated)");
        }
    }
    
    Ok(())
}

