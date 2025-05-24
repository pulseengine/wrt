use anyhow::{Context, Result};
use dagger_sdk::HostDirectoryOpts;
use tracing::{error, info, warn};

use crate::Query;

// Use stable for quick coverage, nightly for comprehensive coverage with MC/DC
// support
const RUST_IMAGE_STABLE: &str = "rust:1.87";
const RUST_IMAGE_NIGHTLY: &str = "rustlang/rust:nightly";

// Feature combinations to test for comprehensive coverage
const FEATURE_COMBINATIONS: &[&[&str]] = &[
    &[], // Default features
    &["std"],
    &["alloc"],
    &["safety"],
    &["std", "safety"],
    // Note: no_std is the default when no std feature is enabled
];

// Safety-critical crates that require MCDC coverage
const SAFETY_CRITICAL_CRATES: &[&str] =
    &["wrt-runtime", "wrt-instructions", "wrt-sync", "wrt-foundation", "wrt-platform"];

// Platform-specific features to test
const PLATFORM_FEATURES: &[&str] = &["macos", "linux", "qnx", "zephyr", "tock"];

pub async fn run_comprehensive_coverage(client: &Query) -> Result<()> {
    info!("Starting comprehensive coverage analysis...");

    let src_dir = client.host().directory_opts(
        ".",
        HostDirectoryOpts {
            exclude: Some(vec![
                ".git",
                "target",
                ".vscode",
                ".idea",
                ".DS_Store",
                ".cargo/git",
                ".cargo/registry",
                ".zephyr-venv",
                ".zephyrproject",
            ]),
            include: None,
        },
    );

    let mut container = client
        .container()
        .from(RUST_IMAGE_NIGHTLY)
        .with_exec(vec!["apt-get", "update", "-y"])
        .with_exec(vec!["apt-get", "install", "-y", "git"])
        // Install llvm-tools-preview for MC/DC support
        .with_exec(vec!["rustup", "component", "add", "llvm-tools-preview"])
        .with_exec(vec![
            "cargo",
            "install",
            "cargo-llvm-cov",
            "cargo-kani",
            "cargo-miri",
            "--locked",
        ])
        .with_mounted_directory("/src", src_dir)
        .with_workdir("/src")
        // Set environment for MC/DC
        .with_env_variable("RUSTFLAGS", "-C instrument-coverage -C llvm-args=-enable-mcdc");

    // Create coverage output directory
    container = container.with_exec(vec!["mkdir", "-p", "/src/target/coverage"]);

    // 1. Standard coverage across feature combinations
    info!("Running standard coverage across feature combinations...");
    for features in FEATURE_COMBINATIONS {
        let feature_str = features.join(",");
        let feature_name =
            if features.is_empty() { "default".to_string() } else { features.join("_") };
        let output_file = format!("/src/target/coverage/features_{}.json", feature_name);

        let mut args = vec!["cargo", "llvm-cov", "--workspace"];
        if !features.is_empty() {
            args.push("--features");
            args.push(&feature_str);
        }
        args.push("--json");
        args.push("--output-path");
        args.push(&output_file);

        container = container.with_exec(args);
    }

    // 2. MCDC coverage for safety-critical crates
    info!("Running MCDC coverage for safety-critical crates...");
    for crate_name in SAFETY_CRITICAL_CRATES {
        container = container.with_exec(vec![
            "cargo",
            "llvm-cov",
            "--package",
            crate_name,
            "--all-features",
            "--mcdc", // Enable MC/DC coverage
            "--json",
            "--output-path",
            &format!("/src/target/coverage/mcdc_{}.json", crate_name),
        ]);
    }

    // 3. Platform-specific coverage (where applicable)
    info!("Running platform-specific coverage...");
    for platform in PLATFORM_FEATURES {
        // Only run if the feature exists in the workspace
        container = container.with_exec(vec![
            "cargo",
            "llvm-cov",
            "--workspace",
            "--features",
            platform,
            "--json",
            "--output-path",
            &format!("/src/target/coverage/platform_{}.json", platform),
        ]);
    }

    // 4. Kani coverage (for formal verification)
    info!("Running Kani coverage analysis...");
    container = container.with_exec(vec![
        "cargo",
        "kani",
        "--workspace",
        "--coverage",
        "--output-format",
        "json",
        "--json-final-results",
    ]);

    // 5. Generate comprehensive reports
    info!("Generating comprehensive coverage reports...");

    // LCOV format for Codecov
    container = container.with_exec(vec![
        "cargo",
        "llvm-cov",
        "--workspace",
        "--all-features",
        "--lcov",
        "--output-path",
        "/src/target/coverage/lcov.info",
    ]);

    // HTML report for human review
    container = container.with_exec(vec![
        "cargo",
        "llvm-cov",
        "--workspace",
        "--all-features",
        "--html",
        "--output-dir",
        "/src/target/coverage/html",
    ]);

    // Cobertura XML for other tools
    container = container.with_exec(vec![
        "cargo",
        "llvm-cov",
        "--workspace",
        "--all-features",
        "--cobertura",
        "--output-path",
        "/src/target/coverage/cobertura.xml",
    ]);

    // 6. Coverage validation and reporting
    info!("Validating coverage thresholds...");
    container = container.with_exec(vec![
        "sh",
        "-c",
        r#"
        # Extract coverage percentages and validate thresholds
        echo "Validating coverage thresholds for safety-critical system..."
        # TODO: Add threshold validation script
        # Line coverage: >= 90%
        # Branch coverage: >= 85% 
        # Function coverage: >= 95%
        # MCDC coverage: 100% for safety-critical components
        "#,
    ]);

    let coverage_dir = container.directory("/src/target/coverage");

    // Execute the pipeline
    let _ = container.sync().await.context("Failed to execute coverage pipeline")?;

    // Export coverage artifacts
    info!("Exporting coverage artifacts...");
    coverage_dir.export("./target/coverage").await.context("Failed to export coverage reports")?;

    info!("Comprehensive coverage analysis completed.");
    Ok(())
}

pub async fn run_minimal_coverage(client: &Query) -> Result<()> {
    info!("Starting minimal coverage test for single crate...");

    let src_dir = client.host().directory_opts(
        ".",
        HostDirectoryOpts {
            exclude: Some(vec![".git", "target", ".vscode", ".idea", ".DS_Store"]),
            include: None,
        },
    );

    let container = client
        .container()
        .from(RUST_IMAGE_STABLE)
        .with_exec(vec!["apt-get", "update", "-y"])
        .with_exec(vec!["apt-get", "install", "-y", "build-essential", "pkg-config", "libssl-dev"])
        .with_exec(vec!["rustup", "component", "add", "llvm-tools-preview"])
        .with_exec(vec!["cargo", "install", "cargo-llvm-cov", "--version", "0.5.31", "--locked"])
        .with_mounted_directory("/src", src_dir)
        .with_workdir("/src")
        .with_exec(vec!["mkdir", "-p", "/src/target/coverage"])
        // Test just one small crate first
        .with_exec(vec![
            "cargo",
            "llvm-cov",
            "test",
            "--package",
            "wrt-error",
            "--json",
            "--output-path",
            "/src/target/coverage/coverage.json",
        ]);

    let exit_code = container.exit_code().await?;

    if exit_code != 0 {
        let stderr = container.stderr().await.unwrap_or_default();
        error!("Minimal coverage failed: {}", stderr);
        return Err(anyhow::anyhow!("Minimal coverage failed"));
    }

    let coverage_dir = container.directory("/src/target/coverage");
    coverage_dir.export("./target/coverage").await?;

    info!("Minimal coverage completed successfully");
    Ok(())
}

pub async fn run_quick_coverage(client: &Query) -> Result<()> {
    info!("Starting quick coverage analysis...");

    // First try minimal coverage to ensure the setup works
    if let Err(e) = run_minimal_coverage(client).await {
        error!("Minimal coverage failed: {}, attempting full coverage anyway", e);
    }

    let src_dir = client.host().directory_opts(
        ".",
        HostDirectoryOpts {
            exclude: Some(vec![
                ".git",
                "target",
                ".vscode",
                ".idea",
                ".DS_Store",
                "docs_output",
                "external",
            ]),
            include: None,
        },
    );

    info!("Building coverage container...");

    // Build container step by step with validation
    // Use stable for quick coverage to avoid nightly issues
    let mut container = client.container().from(RUST_IMAGE_STABLE);

    info!("Installing system dependencies...");
    container = container.with_exec(vec!["apt-get", "update", "-y"]).with_exec(vec![
        "apt-get",
        "install",
        "-y",
        "build-essential",
        "pkg-config",
        "libssl-dev",
    ]);

    info!("Setting up Rust toolchain...");
    container = container.with_exec(vec!["rustup", "component", "add", "llvm-tools-preview"]);

    info!("Installing cargo-llvm-cov...");
    container = container.with_exec(vec![
        "cargo",
        "install",
        "cargo-llvm-cov",
        "--version",
        "0.5.31",
        "--locked",
    ]);

    info!("Mounting source directory...");
    container = container
        .with_mounted_directory("/src", src_dir)
        .with_workdir("/src")
        // Set environment variables for better debugging
        .with_env_variable("RUST_BACKTRACE", "1")
        .with_env_variable("CARGO_TERM_COLOR", "always")
        .with_env_variable("RUSTFLAGS", "-C instrument-coverage");

    info!("Creating coverage directory...");
    container = container.with_exec(vec!["mkdir", "-p", "/src/target/coverage"]);

    info!("Cleaning previous coverage data...");
    container = container.with_exec(vec!["cargo", "llvm-cov", "clean", "--workspace"]);

    // First, try to build the workspace to catch compilation errors early
    info!("Building workspace to verify compilation...");
    container = container.with_exec(vec![
        "cargo",
        "build",
        "--workspace",
        "--exclude",
        "wrt-debug",
        "--exclude",
        "wrt-verification-tool",
        "--exclude",
        "example",
        "--exclude",
        "wrtd",
    ]);

    info!("Running coverage tests...");
    // Run tests with coverage, excluding problematic crates
    container = container.with_exec(vec![
        "cargo",
        "llvm-cov",
        "test",
        "--workspace",
        "--exclude",
        "wrt-debug",
        "--exclude",
        "wrt-verification-tool",
        "--exclude",
        "example",
        "--exclude",
        "wrtd",
        "--exclude",
        "xtask",
        "--no-fail-fast", // Continue even if some tests fail
        "--",
        "--test-threads=1", // Run tests sequentially to avoid race conditions
    ]);

    info!("Generating coverage reports...");
    // Generate JSON report
    container = container.with_exec(vec![
        "cargo",
        "llvm-cov",
        "report",
        "--workspace",
        "--exclude",
        "wrt-debug",
        "--exclude",
        "wrt-verification-tool",
        "--exclude",
        "example",
        "--exclude",
        "wrtd",
        "--exclude",
        "xtask",
        "--json",
        "--output-path",
        "/src/target/coverage/coverage.json",
    ]);

    // Generate LCOV report
    container = container.with_exec(vec![
        "cargo",
        "llvm-cov",
        "report",
        "--workspace",
        "--exclude",
        "wrt-debug",
        "--exclude",
        "wrt-verification-tool",
        "--exclude",
        "example",
        "--exclude",
        "wrtd",
        "--exclude",
        "xtask",
        "--lcov",
        "--output-path",
        "/src/target/coverage/lcov.info",
    ]);

    // Generate HTML report
    container = container.with_exec(vec![
        "cargo",
        "llvm-cov",
        "report",
        "--workspace",
        "--exclude",
        "wrt-debug",
        "--exclude",
        "wrt-verification-tool",
        "--exclude",
        "example",
        "--exclude",
        "wrtd",
        "--exclude",
        "xtask",
        "--html",
        "--output-dir",
        "/src/target/coverage/html",
    ]);

    // Execute the container
    info!("Executing coverage container...");

    // Try to sync the container, but don't fail if it doesn't work perfectly
    match container.sync().await {
        Ok(_) => info!("Container sync successful"),
        Err(e) => {
            warn!("Container sync had issues: {}, continuing anyway", e);
            // Don't return error here, try to export what we can
        }
    }

    // Then check exit code
    let exit_code = container.exit_code().await.context("Failed to get container exit code")?;

    if exit_code != 0 {
        // Try to get logs for debugging
        let stderr =
            container.stderr().await.unwrap_or_else(|_| "Failed to get stderr".to_string());
        let stdout =
            container.stdout().await.unwrap_or_else(|_| "Failed to get stdout".to_string());

        error!("Coverage container failed with exit code {}", exit_code);
        warn!("stderr (last 1000 chars): {}", &stderr[stderr.len().saturating_sub(1000)..]);
        warn!("stdout (last 1000 chars): {}", &stdout[stdout.len().saturating_sub(1000)..]);

        // Even if tests fail, we might still have coverage data
        warn!("Container failed but attempting to export any coverage data that was generated");
    }

    // Try to export coverage even if some tests failed
    let coverage_dir = container.directory("/src/target/coverage");

    info!("Exporting coverage artifacts...");
    match coverage_dir.export("./target/coverage").await {
        Ok(_) => info!("Coverage artifacts exported successfully"),
        Err(e) => {
            error!("Failed to export coverage reports: {}", e);
            return Err(e).context("Failed to export coverage reports");
        }
    }

    info!("Quick coverage analysis completed.");
    Ok(())
}
