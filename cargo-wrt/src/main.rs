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
#[command(version, about = "Unified build tool for WRT (WebAssembly Runtime)", long_about = None)]
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Print header
    if cli.verbose {
        println!(
            "{} cargo-wrt v{}",
            "🚀".bright_blue(),
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
        Commands::Docs { open, private } => cmd_docs(&build_system, open, private).await,
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
async fn cmd_docs(build_system: &BuildSystem, open: bool, private: bool) -> Result<()> {
    println!("{} Generating documentation...", "📚".bright_blue());

    build_system.generate_docs().context("Documentation generation failed")?;

    if open {
        println!("{} Opening documentation in browser...", "🌐".bright_blue());
        // TODO: Implement browser opening
    }

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
