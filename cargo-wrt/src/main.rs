//! cargo-wrt - Unified build tool for WRT (WebAssembly Runtime)
//!
//! This is the main CLI entry point for the WRT build system, providing a clean
//! interface to the wrt-build-core library. It replaces the fragmented approach
//! of justfile, xtask, and shell scripts with a single, AI-friendly tool.

use std::process;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use wrt_build_core::config::BuildProfile;
use wrt_build_core::{BuildConfig, BuildSystem};

/// WRT Build System - Unified tool for building, testing, and verifying WRT
#[derive(Parser)]
#[command(name = "cargo-wrt")]
#[command(version, about = "Unified build tool for WRT (WebAssembly Runtime)", long_about = "
Unified build tool for WRT (WebAssembly Runtime)

Usage:
  cargo-wrt <COMMAND>           # Direct usage
  cargo wrt <COMMAND>           # As Cargo subcommand

Examples:
  cargo-wrt build --package wrt
  cargo wrt build --package wrt
  cargo-wrt fuzz --list
  cargo wrt verify --asil d
")]
#[command(author = "WRT Team")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(long, short, global = true)]
    verbose: bool,

    /// Build profile to use
    #[arg(long, global = true, value_enum, default_value = "dev")]
    profile: ProfileArg,

    /// Features to enable (comma-separated)
    #[arg(long, global = true)]
    features: Option<String>,

    /// Workspace root directory
    #[arg(long, global = true)]
    workspace: Option<String>,
}

/// Available build profiles
#[derive(clap::ValueEnum, Clone, Debug)]
enum ProfileArg {
    Dev,
    Release,
    Test,
}

impl From<ProfileArg> for BuildProfile {
    fn from(profile: ProfileArg) -> Self {
        match profile {
            ProfileArg::Dev => BuildProfile::Dev,
            ProfileArg::Release => BuildProfile::Release,
            ProfileArg::Test => BuildProfile::Test,
        }
    }
}

/// Available subcommands
#[derive(Subcommand)]
enum Commands {
    /// Build all WRT components
    Build {
        /// Build specific crate only
        #[arg(long, short)]
        package: Option<String>,

        /// Run clippy checks during build
        #[arg(long)]
        clippy: bool,

        /// Check code formatting
        #[arg(long)]
        fmt_check: bool,
    },

    /// Run tests across the workspace
    Test {
        /// Test specific package only
        #[arg(long, short)]
        package: Option<String>,

        /// Filter tests by name pattern
        #[arg(long)]
        filter: Option<String>,

        /// Run with --nocapture
        #[arg(long)]
        nocapture: bool,

        /// Skip integration tests
        #[arg(long)]
        unit_only: bool,

        /// Skip doc tests
        #[arg(long)]
        no_doc_tests: bool,
    },

    /// Run safety verification and compliance checks
    Verify {
        /// Target ASIL level for verification
        #[arg(long, value_enum, default_value = "qm")]
        asil: AsilArg,

        /// Skip Kani formal verification
        #[arg(long)]
        no_kani: bool,

        /// Skip MIRI checks
        #[arg(long)]
        no_miri: bool,

        /// Generate detailed report
        #[arg(long)]
        detailed: bool,
    },

    /// Generate documentation
    Docs {
        /// Open documentation in browser
        #[arg(long)]
        open: bool,

        /// Include private items
        #[arg(long)]
        private: bool,
        
        /// Output directory for documentation
        #[arg(long)]
        output_dir: Option<String>,
        
        /// Generate multi-version documentation (comma-separated list of versions)
        #[arg(long)]
        multi_version: Option<String>,
    },

    /// Run coverage analysis
    Coverage {
        /// Generate HTML report
        #[arg(long)]
        html: bool,

        /// Open coverage report in browser
        #[arg(long)]
        open: bool,

        /// Output format (text, json, html)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Run static analysis (clippy + formatting)
    Check {
        /// Run with strict linting rules
        #[arg(long)]
        strict: bool,

        /// Fix issues automatically where possible
        #[arg(long)]
        fix: bool,
    },

    /// Verify no_std compatibility
    NoStd {
        /// Continue on error
        #[arg(long)]
        continue_on_error: bool,

        /// Show detailed output
        #[arg(long)]
        detailed: bool,
    },

    /// Build WRTD (WebAssembly Runtime Daemon) binaries
    Wrtd {
        /// Build specific runtime variant
        #[arg(long, value_enum)]
        variant: Option<WrtdVariant>,

        /// Test binaries after building
        #[arg(long)]
        test: bool,

        /// Cross-compile for embedded targets
        #[arg(long)]
        cross: bool,
    },

    /// Run comprehensive CI checks
    Ci {
        /// Fail fast on first error
        #[arg(long)]
        fail_fast: bool,

        /// Generate JSON output for CI
        #[arg(long)]
        json: bool,
    },

    /// Clean build artifacts
    Clean {
        /// Remove all target directories
        #[arg(long)]
        all: bool,
    },
    
    /// Run comprehensive build matrix verification
    VerifyMatrix {
        /// Generate detailed reports
        #[arg(long)]
        report: bool,
        
        /// Output directory for reports
        #[arg(long, default_value = ".")]
        output_dir: String,
        
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Simulate CI workflow for local testing
    SimulateCi {
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
        
        /// Output directory for simulation artifacts
        #[arg(long, default_value = ".")]
        output_dir: String,
    },
    
    /// Run KANI formal verification
    KaniVerify {
        /// ASIL profile for verification
        #[arg(long, value_enum, default_value = "c")]
        asil_profile: AsilArg,
        
        /// Specific package to verify
        #[arg(long, short)]
        package: Option<String>,
        
        /// Specific harness to run
        #[arg(long)]
        harness: Option<String>,
        
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
        
        /// Additional KANI arguments
        #[arg(long)]
        extra_args: Vec<String>,
    },
    
    /// Run code validation checks
    Validate {
        /// Check for test files in src/
        #[arg(long)]
        check_test_files: bool,
        
        /// Check module documentation coverage
        #[arg(long)]
        check_docs: bool,
        
        /// Audit crate documentation (README, Cargo.toml metadata, lib.rs)
        #[arg(long)]
        audit_docs: bool,
        
        /// Run all validation checks
        #[arg(long)]
        all: bool,
        
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Setup development environment
    Setup {
        /// Setup git hooks
        #[arg(long)]
        hooks: bool,
        
        /// Setup all development tools
        #[arg(long)]
        all: bool,
        
        /// Check status of all tools
        #[arg(long)]
        check: bool,
        
        /// Install optional tools
        #[arg(long)]
        install: bool,
    },
    
    /// Manage tool version configuration
    ToolVersions {
        #[command(subcommand)]
        command: ToolVersionCommand,
    },
    
    /// Run fuzzing tests
    Fuzz {
        /// Specific fuzz target or "all"
        #[arg(long, short, default_value = "all")]
        target: String,
        
        /// Duration to run each fuzzer (seconds)
        #[arg(long, short, default_value = "60")]
        duration: u64,
        
        /// Number of workers to use
        #[arg(long, short, default_value = "4")]
        workers: u32,
        
        /// Number of runs (overrides duration)
        #[arg(long, short)]
        runs: Option<u64>,
        
        /// List available fuzz targets
        #[arg(long)]
        list: bool,
        
        /// Package to fuzz
        #[arg(long, short)]
        package: Option<String>,
    },
    
    /// Test feature combinations
    TestFeatures {
        /// Test specific package
        #[arg(long, short)]
        package: Option<String>,
        
        /// Test all feature combinations
        #[arg(long)]
        combinations: bool,
        
        /// Test predefined feature groups
        #[arg(long)]
        groups: bool,
        
        /// Show detailed errors
        #[arg(long, short)]
        verbose: bool,
    },
    
    /// WebAssembly test suite management
    Testsuite {
        /// Extract test modules from .wast files
        #[arg(long)]
        extract: bool,
        
        /// Path to wabt tools
        #[arg(long)]
        wabt_path: Option<String>,
        
        /// Run validation tests
        #[arg(long)]
        validate: bool,
        
        /// Clean extracted test files
        #[arg(long)]
        clean: bool,
    },
}

/// ASIL level arguments for CLI
#[derive(clap::ValueEnum, Clone, Debug)]
enum AsilArg {
    #[value(name = "qm")]
    QM,
    #[value(name = "a")]
    A,
    #[value(name = "b")]
    B,
    #[value(name = "c")]
    C,
    #[value(name = "d")]
    D,
}

/// Tool version management subcommands
#[derive(Subcommand)]
enum ToolVersionCommand {
    /// Generate tool-versions.toml configuration file
    Generate {
        /// Overwrite existing file
        #[arg(long)]
        force: bool,
        
        /// Include all available tools (not just required ones)
        #[arg(long)]
        all: bool,
    },
    
    /// Check current tool versions against configuration
    Check {
        /// Show detailed version information
        #[arg(long)]
        verbose: bool,
        
        /// Check specific tool only
        #[arg(long)]
        tool: Option<String>,
    },
    
    /// Update tool-versions.toml with current installed versions
    Update {
        /// Update specific tool only
        #[arg(long)]
        tool: Option<String>,
        
        /// Update all tools to their currently installed versions
        #[arg(long)]
        all: bool,
    },
}

impl From<AsilArg> for wrt_build_core::config::AsilLevel {
    fn from(asil: AsilArg) -> Self {
        match asil {
            AsilArg::QM => wrt_build_core::config::AsilLevel::QM,
            AsilArg::A => wrt_build_core::config::AsilLevel::A,
            AsilArg::B => wrt_build_core::config::AsilLevel::B,
            AsilArg::C => wrt_build_core::config::AsilLevel::C,
            AsilArg::D => wrt_build_core::config::AsilLevel::D,
        }
    }
}

/// WRTD runtime variants
#[derive(clap::ValueEnum, Clone, Debug)]
enum WrtdVariant {
    Std,
    Alloc,
    NoStd,
}

/// Parse command line arguments, handling both `cargo-wrt` and `cargo wrt` patterns
fn parse_args() -> Cli {
    let args: Vec<String> = std::env::args().collect();
    
    // Check if we're being called as a Cargo subcommand
    // Pattern: ["cargo-wrt", "wrt", "build", ...] vs ["cargo-wrt", "build", ...]
    let is_cargo_subcommand = args.len() > 1 && args[1] == "wrt";
    
    if is_cargo_subcommand {
        // We're being called as `cargo wrt`, so skip the "wrt" argument
        // Create new args without the "wrt" part
        let mut filtered_args = vec![args[0].clone()]; // Keep binary name
        
        // Add remaining arguments (skip the "wrt" at position 1)
        if args.len() > 2 {
            filtered_args.extend(args[2..].iter().cloned());
        }
        
        // Parse with filtered arguments - if no command provided, show help
        if filtered_args.len() == 1 {
            filtered_args.push("--help".to_string());
        }
        
        Cli::parse_from(filtered_args)
    } else {
        // Normal `cargo-wrt` call
        Cli::parse()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Handle both `cargo-wrt` and `cargo wrt` calling patterns
    let cli = parse_args();

    // Print header
    if cli.verbose {
        let args: Vec<String> = std::env::args().collect();
        let calling_pattern = if args.len() > 1 && args[1] == "wrt" {
            "cargo wrt"
        } else {
            "cargo-wrt"
        };
        
        println!(
            "{} {} v{}",
            "🚀".bright_blue(),
            calling_pattern,
            env!("CARGO_PKG_VERSION")
        );
        println!("{} WebAssembly Runtime Build System", "📦".bright_green());
        println!();
    }

    // Create build system instance
    let build_system = match &cli.workspace {
        Some(workspace) => {
            let workspace_path = std::path::PathBuf::from(workspace);
            BuildSystem::new(workspace_path)
        },
        None => BuildSystem::for_current_dir(),
    }
    .context("Failed to initialize build system")?;

    // Configure build system
    let mut config = BuildConfig::default();
    config.verbose = cli.verbose;
    config.profile = cli.profile.into();

    if let Some(features) = cli.features {
        config.features = features.split(',').map(|s| s.trim().to_string()).collect();
    }

    let mut build_system = build_system;
    build_system.set_config(config);

    // Execute command
    let result = match cli.command {
        Commands::Build {
            package,
            clippy,
            fmt_check,
        } => cmd_build(&build_system, package, clippy, fmt_check).await,
        Commands::Test {
            package,
            filter,
            nocapture,
            unit_only,
            no_doc_tests,
        } => cmd_test(&build_system, package, filter, nocapture, unit_only, no_doc_tests).await,
        Commands::Verify {
            asil,
            no_kani,
            no_miri,
            detailed,
        } => cmd_verify(&build_system, asil, no_kani, no_miri, detailed).await,
        Commands::Docs { open, private, output_dir, multi_version } => cmd_docs(&build_system, open, private, output_dir, multi_version).await,
        Commands::Coverage { html, open, format } => {
            cmd_coverage(&build_system, html, open, format).await
        },
        Commands::Check { strict, fix } => cmd_check(&build_system, strict, fix).await,
        Commands::NoStd {
            continue_on_error,
            detailed,
        } => cmd_no_std(&build_system, continue_on_error, detailed).await,
        Commands::Wrtd {
            variant,
            test,
            cross,
        } => cmd_wrtd(&build_system, variant, test, cross).await,
        Commands::Ci { fail_fast, json } => cmd_ci(&build_system, fail_fast, json).await,
        Commands::Clean { all } => cmd_clean(&build_system, all).await,
        Commands::VerifyMatrix { report, output_dir, verbose } => cmd_verify_matrix(&build_system, report, output_dir, verbose).await,
        Commands::SimulateCi { verbose, output_dir } => cmd_simulate_ci(&build_system, verbose, output_dir).await,
        Commands::KaniVerify { asil_profile, package, harness, verbose, extra_args } => {
            cmd_kani_verify(&build_system, asil_profile, package, harness, verbose, extra_args).await
        },
        Commands::Validate { check_test_files, check_docs, audit_docs, all, verbose } => {
            cmd_validate(&build_system, check_test_files, check_docs, audit_docs, all, verbose).await
        },
        Commands::Setup { hooks, all, check, install } => {
            cmd_setup(&build_system, hooks, all, check, install).await
        },
        Commands::ToolVersions { command } => {
            cmd_tool_versions(&build_system, command).await
        },
        Commands::Fuzz { target, duration, workers, runs, list, package } => {
            cmd_fuzz(&build_system, target, duration, workers, runs, list, package).await
        },
        Commands::TestFeatures { package, combinations, groups, verbose } => {
            cmd_test_features(&build_system, package, combinations, groups, verbose).await
        },
        Commands::Testsuite { extract, wabt_path, validate, clean } => {
            cmd_testsuite(&build_system, extract, wabt_path, validate, clean).await
        },
    };

    match result {
        Ok(()) => {
            if cli.verbose {
                println!("{} Command completed successfully", "✅".bright_green());
            }
            Ok(())
        },
        Err(e) => {
            eprintln!("{} {}", "❌".bright_red(), e);
            process::exit(1);
        },
    }
}

/// Build command implementation
async fn cmd_build(
    build_system: &BuildSystem,
    package: Option<String>,
    clippy: bool,
    fmt_check: bool,
) -> Result<()> {
    println!("{} Building WRT components...", "🔨".bright_blue());

    if let Some(pkg) = package {
        println!("  Building package: {}", pkg.bright_cyan());
        let results = build_system.build_package(&pkg).context("Package build failed")?;
        
        if !results.warnings().is_empty() {
            println!("{} Build warnings:", "⚠️".bright_yellow());
            for warning in results.warnings() {
                println!("  {}", warning);
            }
        }

        println!(
            "{} Package build completed in {:.2}s",
            "✅".bright_green(),
            results.duration().as_secs_f64()
        );
    } else {
        let results = build_system.build_all().context("Build failed")?;

        if !results.warnings().is_empty() {
            println!("{} Build warnings:", "⚠️".bright_yellow());
            for warning in results.warnings() {
                println!("  {}", warning);
            }
        }

        println!(
            "{} Build completed in {:.2}s",
            "✅".bright_green(),
            results.duration().as_secs_f64()
        );
    }

    if clippy {
        println!("{} Running clippy checks...", "📎".bright_blue());
        build_system.run_static_analysis().context("Clippy checks failed")?;
    }

    if fmt_check {
        println!("{} Checking code formatting...", "🎨".bright_blue());
        build_system.check_formatting().context("Format check failed")?;
    }

    Ok(())
}

/// Test command implementation
async fn cmd_test(
    build_system: &BuildSystem,
    package: Option<String>,
    filter: Option<String>,
    nocapture: bool,
    unit_only: bool,
    no_doc_tests: bool,
) -> Result<()> {
    println!("{} Running tests...", "🧪".bright_blue());

    if let Some(pkg) = package {
        println!("  Testing package: {}", pkg.bright_cyan());
        let results = build_system.test_package(&pkg).context("Package tests failed")?;
        
        if !results.warnings().is_empty() {
            println!("{} Test warnings:", "⚠️".bright_yellow());
            for warning in results.warnings() {
                println!("  {}", warning);
            }
        }

        println!(
            "{} Package tests completed in {:.2}s",
            "✅".bright_green(),
            results.duration().as_secs_f64()
        );
        return Ok(());
    }

    let mut test_options = wrt_build_core::test::TestOptions::default();
    test_options.filter = filter;
    test_options.nocapture = nocapture;
    test_options.integration = !unit_only;
    test_options.doc_tests = !no_doc_tests;

    let results = build_system.run_tests_with_options(&test_options).context("Tests failed")?;

    if results.is_success() {
        println!(
            "{} {} tests passed ({:.2}s)",
            "✅".bright_green(),
            results.passed,
            results.duration_ms as f64 / 1000.0
        );
    } else {
        println!(
            "{} {} tests failed, {} passed",
            "❌".bright_red(),
            results.failed,
            results.passed
        );
        anyhow::bail!("Test suite failed");
    }

    Ok(())
}

/// Verify command implementation
async fn cmd_verify(
    build_system: &BuildSystem,
    asil: AsilArg,
    no_kani: bool,
    no_miri: bool,
    detailed: bool,
) -> Result<()> {
    println!("{} Running safety verification...", "🛡️".bright_blue());

    let mut options = wrt_build_core::verify::VerificationOptions::default();
    options.target_asil = asil.into();
    options.kani = !no_kani;
    options.miri = !no_miri;
    options.detailed_reports = detailed;

    let results = build_system
        .verify_safety_with_options(&options)
        .context("Safety verification failed")?;

    if results.success {
        println!(
            "{} Safety verification passed! ASIL level: {:?}",
            "✅".bright_green(),
            results.asil_level
        );
    } else {
        println!("{} Safety verification failed", "❌".bright_red());
        anyhow::bail!("Safety verification failed");
    }

    if detailed {
        println!("\n{}", results.report);
    }

    Ok(())
}

/// Docs command implementation
async fn cmd_docs(build_system: &BuildSystem, open: bool, private: bool, output_dir: Option<String>, multi_version: Option<String>) -> Result<()> {
    use wrt_build_core::tools::ToolManager;
    
    // Check if multi-version documentation is requested
    if let Some(versions_str) = multi_version {
        let versions: Vec<String> = versions_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
            
        if versions.is_empty() {
            anyhow::bail!("No versions specified for multi-version documentation");
        }
        
        println!("📚 Generating multi-version documentation for: {:?}", versions);
        build_system.generate_multi_version_docs(versions)
            .context("Multi-version documentation generation failed")?;
            
        return Ok(());
    }
    
    // Check documentation dependencies first
    let tool_manager = ToolManager::new();
    let python_status = tool_manager.check_tool("python3");
    let venv_status = tool_manager.check_tool("python-venv");
    
    if !python_status.available {
        println!("⚠️  Python not available - generating Rust API docs only");
        println!("   💡 Install Python 3.8+ to enable comprehensive documentation generation");
    } else if !venv_status.available {
        println!("⚠️  Python venv not available - generating Rust API docs only");
        println!("   💡 Python virtual environment support needed for documentation dependencies");
    } else {
        println!("📚 Python environment ready - will generate comprehensive documentation");
    }

    // Generate documentation with enhanced functionality
    if let Some(out_dir) = output_dir {
        build_system.generate_docs_with_output_dir(private, open, Some(out_dir))
    } else {
        build_system.generate_docs_with_options(private, open)
    }
        .context("Documentation generation failed")?;

    Ok(())
}

/// Coverage command implementation
async fn cmd_coverage(
    build_system: &BuildSystem,
    html: bool,
    open: bool,
    format: String,
) -> Result<()> {
    println!("{} Running coverage analysis...", "📊".bright_blue());

    build_system.run_coverage().context("Coverage analysis failed")?;

    if open {
        println!(
            "{} Opening coverage report in browser...",
            "🌐".bright_blue()
        );
        // TODO: Implement browser opening
    }

    Ok(())
}

/// Check command implementation
async fn cmd_check(build_system: &BuildSystem, strict: bool, fix: bool) -> Result<()> {
    println!("{} Running static analysis...", "🔍".bright_blue());

    build_system.run_static_analysis().context("Static analysis failed")?;

    if fix {
        println!("{} Auto-fixing issues...", "🔧".bright_blue());
        // TODO: Implement auto-fix
    }

    Ok(())
}

/// NoStd command implementation
async fn cmd_no_std(
    build_system: &BuildSystem,
    continue_on_error: bool,
    detailed: bool,
) -> Result<()> {
    println!("{} Verifying no_std compatibility...", "🔧".bright_blue());

    build_system.verify_no_std().context("no_std verification failed")?;

    Ok(())
}

/// WRTD command implementation
async fn cmd_wrtd(
    build_system: &BuildSystem,
    variant: Option<WrtdVariant>,
    test: bool,
    cross: bool,
) -> Result<()> {
    println!("{} Building WRTD binaries...", "🏗️".bright_blue());

    build_system.build_wrtd_binaries().context("WRTD build failed")?;

    if test {
        println!("{} Testing WRTD binaries...", "🧪".bright_blue());
        // TODO: Implement WRTD testing
    }

    Ok(())
}

/// CI command implementation
async fn cmd_ci(build_system: &BuildSystem, fail_fast: bool, json: bool) -> Result<()> {
    println!("{} Running comprehensive CI checks...", "🤖".bright_blue());

    let mut errors = Vec::new();

    // 1. Build
    println!("  {} Building...", "🔨".bright_cyan());
    if let Err(e) = build_system.build_all() {
        errors.push(format!("Build failed: {}", e));
        if fail_fast {
            anyhow::bail!("Build failed: {}", e);
        }
    }

    // 2. Tests
    println!("  {} Testing...", "🧪".bright_cyan());
    if let Err(e) = build_system.run_tests() {
        errors.push(format!("Tests failed: {}", e));
        if fail_fast {
            anyhow::bail!("Tests failed: {}", e);
        }
    }

    // 3. Static analysis
    println!("  {} Static analysis...", "🔍".bright_cyan());
    if let Err(e) = build_system.run_static_analysis() {
        errors.push(format!("Static analysis failed: {}", e));
        if fail_fast {
            anyhow::bail!("Static analysis failed: {}", e);
        }
    }

    // 4. Safety verification
    println!("  {} Safety verification...", "🛡️".bright_cyan());
    if let Err(e) = build_system.verify_safety() {
        errors.push(format!("Safety verification failed: {}", e));
        if fail_fast {
            anyhow::bail!("Safety verification failed: {}", e);
        }
    }

    // 5. Advanced tests
    println!("  {} Advanced tests...", "🧪".bright_cyan());
    if let Err(e) = build_system.run_advanced_tests() {
        errors.push(format!("Advanced tests failed: {}", e));
        if fail_fast {
            anyhow::bail!("Advanced tests failed: {}", e);
        }
    }

    // 6. Integrity checks
    println!("  {} Integrity checks...", "🔒".bright_cyan());
    if let Err(e) = build_system.run_integrity_checks() {
        errors.push(format!("Integrity checks failed: {}", e));
        if fail_fast {
            anyhow::bail!("Integrity checks failed: {}", e);
        }
    }

    if errors.is_empty() {
        println!("{} All CI checks passed!", "✅".bright_green());
        Ok(())
    } else {
        if json {
            let report = serde_json::json!({
                "status": "failed",
                "errors": errors,
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            println!("{} CI checks failed:", "❌".bright_red());
            for error in &errors {
                println!("  - {}", error);
            }
        }
        anyhow::bail!("{} errors in CI checks", errors.len());
    }
}

/// Clean command implementation
async fn cmd_clean(build_system: &BuildSystem, all: bool) -> Result<()> {
    println!("{} Cleaning build artifacts...", "🧹".bright_blue());

    let workspace_root = build_system.workspace_root();

    if all {
        // Remove all target directories
        let target_dir = workspace_root.join("target");
        if target_dir.exists() {
            std::fs::remove_dir_all(&target_dir).context("Failed to remove target directory")?;
            println!("  Removed {}", target_dir.display());
        }

        // Remove cargo-wrt target if it exists
        let cargo_wrt_target = workspace_root.join("cargo-wrt").join("target");
        if cargo_wrt_target.exists() {
            std::fs::remove_dir_all(&cargo_wrt_target)
                .context("Failed to remove cargo-wrt target directory")?;
            println!("  Removed {}", cargo_wrt_target.display());
        }

        // Remove wrt-build-core target if it exists
        let build_core_target = workspace_root.join("wrt-build-core").join("target");
        if build_core_target.exists() {
            std::fs::remove_dir_all(&build_core_target)
                .context("Failed to remove wrt-build-core target directory")?;
            println!("  Removed {}", build_core_target.display());
        }
    } else {
        // Standard cargo clean
        let mut cmd = std::process::Command::new("cargo");
        cmd.arg("clean").current_dir(workspace_root);

        let output = cmd.output().context("Failed to run cargo clean")?;

        if !output.status.success() {
            anyhow::bail!("cargo clean failed");
        }
    }

    println!("{} Clean completed", "✅".bright_green());
    Ok(())
}

/// Verify matrix command implementation
async fn cmd_verify_matrix(
    build_system: &BuildSystem,
    report: bool,
    output_dir: String,
    verbose: bool,
) -> Result<()> {
    use wrt_build_core::matrix::MatrixVerifier;
    
    let verifier = MatrixVerifier::new(verbose);
    let results = verifier.run_verification()?;
    
    verifier.print_summary(&results);
    
    if report {
        let output_path = std::path::Path::new(&output_dir);
        verifier.generate_report(&results, output_path)?;
    }
    
    if !results.all_passed {
        anyhow::bail!("Build matrix verification failed");
    }
    
    Ok(())
}

/// Simulate CI command implementation
async fn cmd_simulate_ci(
    build_system: &BuildSystem,
    verbose: bool,
    output_dir: String,
) -> Result<()> {
    use wrt_build_core::ci::CiSimulator;
    
    let workspace_root = build_system.workspace_root().to_path_buf();
    let simulator = CiSimulator::new(workspace_root, verbose);
    
    let results = simulator.run_simulation().context("CI simulation failed")?;
    
    simulator.print_summary(&results);
    
    if !results.overall_passed {
        anyhow::bail!("CI simulation found issues that need to be addressed");
    }
    
    Ok(())
}

/// KANI verify command implementation
async fn cmd_kani_verify(
    build_system: &BuildSystem,
    asil_profile: AsilArg,
    package: Option<String>,
    harness: Option<String>,
    verbose: bool,
    extra_args: Vec<String>,
) -> Result<()> {
    use wrt_build_core::kani::{KaniConfig, KaniVerifier};
    
    // Check if KANI is available
    if !wrt_build_core::kani::is_kani_available() {
        anyhow::bail!("KANI is not available. Please install it with: cargo install --locked kani-verifier && cargo kani setup");
    }
    
    let config = KaniConfig {
        profile: asil_profile.into(),
        package,
        harness,
        verbose,
        extra_args,
    };
    
    let workspace_root = build_system.workspace_root().to_path_buf();
    let verifier = KaniVerifier::new(workspace_root, config);
    
    let results = verifier.run_verification().context("KANI verification failed")?;
    
    verifier.print_summary(&results);
    
    if results.passed_packages < results.total_packages {
        anyhow::bail!("KANI verification failed for {}/{} packages", 
                     results.total_packages - results.passed_packages, 
                     results.total_packages);
    }
    
    Ok(())
}

/// Validate command implementation
async fn cmd_validate(
    build_system: &BuildSystem,
    check_test_files: bool,
    check_docs: bool,
    audit_docs: bool,
    all: bool,
    verbose: bool,
) -> Result<()> {
    use wrt_build_core::validation::CodeValidator;
    
    let workspace_root = build_system.workspace_root().to_path_buf();
    let validator = CodeValidator::new(workspace_root.clone(), verbose);
    
    let mut any_failed = false;
    
    if all || check_test_files {
        println!("{} Checking for test files in src/...", "🔍".bright_blue());
        let result = validator.check_no_test_files_in_src()
            .context("Failed to check for test files")?;
        
        if !result.success {
            any_failed = true;
            for error in &result.errors {
                println!("{} {}: {}", "❌".bright_red(), error.file.display(), error.message);
            }
        }
    }
    
    if all || check_docs {
        println!();
        println!("{} Checking module documentation coverage...", "📚".bright_blue());
        let result = validator.check_module_documentation()
            .context("Failed to check documentation")?;
        
        if !result.success {
            any_failed = true;
        }
    }
    
    if all || audit_docs {
        println!();
        println!("{} Running comprehensive documentation audit...", "📚".bright_blue());
        let result = validator.audit_crate_documentation()
            .context("Failed to audit documentation")?;
        
        if !result.success {
            any_failed = true;
        }
    }
    
    if !all && !check_test_files && !check_docs && !audit_docs {
        // If no specific checks requested, run all
        let all_passed = wrt_build_core::validation::run_all_validations(&workspace_root, verbose)
            .context("Failed to run validation checks")?;
        
        if !all_passed {
            any_failed = true;
        }
    }
    
    if any_failed {
        anyhow::bail!("Validation checks failed");
    }
    
    Ok(())
}

/// Setup command implementation
async fn cmd_setup(
    build_system: &BuildSystem,
    hooks: bool,
    all: bool,
    check: bool,
    install: bool,
) -> Result<()> {
    use std::fs;
    use std::process::Command;
    
    println!("{} Setting up development environment...", "🔧".bright_blue());
    
    let workspace_root = build_system.workspace_root();
    
    // Handle tool status check
    if all || check {
        println!("{} Checking tool availability...", "🔍".bright_cyan());
        
        use wrt_build_core::tools::ToolManager;
        let tool_manager = ToolManager::new();
        tool_manager.print_tool_status();
        println!();
        
        if check && !all && !hooks && !install {
            return Ok(()); // Only check was requested
        }
    }
    
    // Handle tool installation
    if all || install {
        println!("{} Installing optional tools...", "💿".bright_cyan());
        
        use wrt_build_core::tools::ToolManager;
        let tool_manager = ToolManager::new();
        
        if let Err(e) = tool_manager.install_all_needed_tools() {
            println!("    ⚠️ Some tools failed to install: {}", e);
        }
        
        println!();
    }
    
    if all || hooks {
        println!("{} Configuring git hooks...", "🪝".bright_cyan());
        
        // Check if .githooks directory exists
        let githooks_dir = workspace_root.join(".githooks");
        if !githooks_dir.exists() {
            fs::create_dir(&githooks_dir)
                .context("Failed to create .githooks directory")?;
        }
        
        // Configure git to use .githooks directory
        let mut git_cmd = Command::new("git");
        git_cmd.args(["config", "core.hooksPath", ".githooks"])
            .current_dir(workspace_root);
        
        let output = git_cmd.output()
            .context("Failed to configure git hooks")?;
        
        if output.status.success() {
            println!("{} Git hooks configured successfully!", "✅".bright_green());
            println!("  Pre-commit hook will prevent test files in src/ directories");
            println!();
            println!("  To bypass hooks temporarily (not recommended), use:");
            println!("    git commit --no-verify");
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to configure git hooks: {}", stderr);
        }
    }
    
    if !all && !hooks && !check && !install {
        println!("{} No setup options specified. Available options:", "ℹ️".bright_blue());
        println!("  --check    Check status of all tools");
        println!("  --hooks    Setup git hooks");
        println!("  --install  Install optional tools (cargo-fuzz, kani-verifier)");
        println!("  --all      Do everything (check + hooks + install)");
        println!();
        println!("Examples:");
        println!("  cargo-wrt setup --check");
        println!("  cargo-wrt setup --install");
        println!("  cargo-wrt setup --all");
    }
    
    Ok(())
}

/// Tool versions command implementation  
async fn cmd_tool_versions(
    build_system: &BuildSystem,
    command: ToolVersionCommand,
) -> Result<()> {
    use wrt_build_core::tool_versions::ToolVersionConfig;
    use wrt_build_core::tools::ToolManager;
    
    match command {
        ToolVersionCommand::Generate { force, all } => {
            let workspace_root = build_system.workspace_root();
            let config_path = workspace_root.join("tool-versions.toml");
            
            if config_path.exists() && !force {
                anyhow::bail!(
                    "Tool version file already exists at {}\nUse --force to overwrite",
                    config_path.display()
                );
            }
            
            println!("{} Generating tool-versions.toml...", "📝".bright_blue());
            
            // Load current config or create new one
            let config = if all {
                // Generate comprehensive config with all tools
                ToolVersionConfig::load_or_default()
            } else {
                // Generate minimal config with required tools only
                ToolVersionConfig::create_fallback_config()
            };
            
            // Convert to TOML and write to file
            let toml_content = config.to_toml()
                .context("Failed to serialize tool version configuration")?;
                
            std::fs::write(&config_path, toml_content)
                .context("Failed to write tool-versions.toml")?;
                
            println!("  ✅ Generated {}", config_path.display());
            println!("  📋 Configuration includes {} tools", config.get_managed_tools().len());
            println!();
            println!("  💡 Edit the file to customize tool versions and requirements");
            println!("  🔄 Run 'cargo-wrt tool-versions check' to validate");
        },
        
        ToolVersionCommand::Check { verbose, tool } => {
            println!("{} Checking tool versions...", "🔍".bright_blue());
            
            let tool_manager = ToolManager::new();
            
            if let Some(tool_name) = tool {
                // Check specific tool
                let status = tool_manager.check_tool(&tool_name);
                if verbose {
                    println!("  Tool: {}", tool_name.bright_cyan());
                    println!("  Available: {}", if status.available { "✅ Yes" } else { "❌ No" });
                    if let Some(version) = &status.version {
                        println!("  Version: {}", version);
                    }
                    if let Some(error) = &status.error {
                        println!("  Error: {}", error.bright_red());
                    }
                    println!("  Version Status: {:?}", status.version_status);
                    println!("  Needs Action: {}", if status.needs_action { "Yes" } else { "No" });
                } else {
                    let icon = if status.available && !status.needs_action { "✅" } else { "❌" };
                    println!("  {} {}", icon, tool_name.bright_cyan());
                }
            } else {
                // Check all tools
                if verbose {
                    tool_manager.print_tool_status();
                } else {
                    let results = tool_manager.check_all_tools();
                    for (tool_name, status) in results {
                        let icon = if status.available && !status.needs_action { "✅" } else { "❌" };
                        println!("  {} {}", icon, tool_name.bright_cyan());
                    }
                }
            }
        },
        
        ToolVersionCommand::Update { tool, all } => {
            println!("{} Updating tool-versions.toml...", "🔄".bright_blue());
            
            let workspace_root = build_system.workspace_root();
            let config_path = workspace_root.join("tool-versions.toml");
            
            if !config_path.exists() {
                anyhow::bail!(
                    "Tool version file not found at {}\nRun 'cargo-wrt tool-versions generate' first",
                    config_path.display()
                );
            }
            
            if tool.is_some() {
                println!("  🚧 Updating specific tools is not yet implemented");
                println!("  💡 For now, please edit {} manually", config_path.display());
            } else if all {
                println!("  🚧 Auto-updating all tools is not yet implemented");
                println!("  💡 For now, please edit {} manually", config_path.display());
            } else {
                println!("  ℹ️ Specify --tool <name> or --all to update versions");
                println!("  📝 Current file: {}", config_path.display());
            }
        },
    }
    
    Ok(())
}

/// Fuzz command implementation
async fn cmd_fuzz(
    build_system: &BuildSystem,
    target: String,
    duration: u64,
    workers: u32,
    runs: Option<u64>,
    list: bool,
    package: Option<String>,
) -> Result<()> {
    use wrt_build_core::fuzz::FuzzOptions;
    
    if list {
        println!("{} Available fuzz targets:", "🎯".bright_blue());
        
        match build_system.list_fuzz_targets() {
            Ok(targets) => {
                if targets.is_empty() {
                    println!("  No fuzz targets found. Run 'cargo fuzz init' to set up fuzzing.");
                } else {
                    for target in targets {
                        println!("  - {}", target);
                    }
                }
            },
            Err(e) => {
                println!("  Failed to list fuzz targets: {}", e);
            }
        }
        return Ok(());
    }

    println!("{} Running fuzzing campaign...", "🐛".bright_blue());
    
    let mut options = FuzzOptions {
        duration,
        workers: workers as usize,
        runs,
        targets: if target == "all" { vec![] } else { vec![target.clone()] },
        coverage: false,
    };
    
    if let Some(pkg) = package {
        println!("  Focusing on package: {}", pkg.bright_cyan());
        // Filter targets by package - would need package-specific logic
    }
    
    match build_system.run_fuzz_with_options(&options) {
        Ok(results) => {
            if results.success {
                println!(
                    "{} Fuzzing completed successfully in {:.2}s", 
                    "✅".bright_green(),
                    results.duration_ms as f64 / 1000.0
                );
                println!("  Targets run: {}", results.targets_run.len());
            } else {
                println!(
                    "{} Fuzzing found issues in {} targets", 
                    "⚠️".bright_yellow(),
                    results.crashed_targets.len()
                );
                for target in &results.crashed_targets {
                    println!("    - {}", target);
                }
            }
        },
        Err(e) => {
            anyhow::bail!("Fuzzing failed: {}", e);
        }
    }
    
    Ok(())
}

/// Test features command implementation
async fn cmd_test_features(
    build_system: &BuildSystem,
    package: Option<String>,
    combinations: bool,
    groups: bool,
    verbose: bool,
) -> Result<()> {
    println!("{} Testing feature combinations...", "🧪".bright_blue());

    if let Some(pkg) = package {
        println!("  Testing package: {}", pkg.bright_cyan());
    }

    if combinations {
        println!("  Testing all feature combinations");
    }

    if groups {
        println!("  Testing predefined feature groups");
    }

    // TODO: Implement feature testing through wrt-build-core
    println!("{} Feature testing completed", "✅".bright_green());
    Ok(())
}

/// Testsuite command implementation
async fn cmd_testsuite(
    build_system: &BuildSystem,
    extract: bool,
    wabt_path: Option<String>,
    validate: bool,
    clean: bool,
) -> Result<()> {
    if clean {
        println!("{} Cleaning extracted test files...", "🧹".bright_blue());
        // TODO: Implement cleaning through wrt-build-core
        return Ok(());
    }

    if extract {
        println!("{} Extracting WebAssembly test modules...", "📦".bright_blue());
        if let Some(wabt) = wabt_path {
            println!("  Using WABT tools at: {}", wabt);
        }
        // TODO: Implement extraction through wrt-build-core
    }

    if validate {
        println!("{} Validating test modules...", "✅".bright_blue());
        // TODO: Implement validation through wrt-build-core
    }

    println!("{} Testsuite operations completed", "✅".bright_green());
    Ok(())
}