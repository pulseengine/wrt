use anyhow::{Context, Result};
use dagger_sdk::HostDirectoryOpts;
use tracing::info;

use crate::Query;

// Use official Kani Docker image for proper verification environment
const KANI_IMAGE: &str = "ghcr.io/model-checking/kani:latest";

pub async fn run(client: &Query) -> Result<()> {
    info!("Starting CI advanced tests pipeline (Kani, Miri, Coverage)...");

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

    // --- Kani Verification Pipeline ---
    info!("Running Kani verification suites...");
    
    let kani_container = client
        .container()
        .from(KANI_IMAGE)
        .with_mounted_directory("/src", src_dir.clone())
        .with_workdir("/src");

    // Run memory safety verification suite
    let memory_safety_results = kani_container
        .with_exec(vec![
            "cargo", "kani",
            "--package", "wrt-foundation",
            "--harness", "verify_bounded_collections_memory_safety",
            "--harness", "verify_safe_memory_bounds",
            "--harness", "verify_arithmetic_safety",
            "--output-format", "terse"
        ])
        .stdout().await
        .context("Failed to run memory safety verification")?;

    // Run concurrency safety verification suite
    let concurrency_results = kani_container
        .with_exec(vec![
            "cargo", "kani",
            "--package", "wrt-sync", 
            "--harness", "verify_mutex_no_data_races",
            "--harness", "verify_rwlock_concurrent_access",
            "--harness", "verify_atomic_operations_safety",
            "--output-format", "terse"
        ])
        .stdout().await
        .context("Failed to run concurrency verification")?;

    // Run type safety verification suite
    let type_safety_results = kani_container
        .with_exec(vec![
            "cargo", "kani",
            "--package", "wrt-component",
            "--harness", "verify_component_type_safety",
            "--harness", "verify_namespace_operations", 
            "--harness", "verify_import_export_consistency",
            "--output-format", "terse"
        ])
        .stdout().await
        .context("Failed to run type safety verification")?;

    // --- Miri Testing Pipeline ---
    info!("Running Miri undefined behavior detection...");
    
    let rust_container = client
        .container()
        .from("rust:latest")
        .with_exec(vec!["rustup", "toolchain", "install", "nightly"])
        .with_exec(vec!["rustup", "component", "add", "miri", "--toolchain", "nightly"])
        .with_mounted_directory("/src", src_dir.clone())
        .with_workdir("/src");

    // Run Miri on core synchronization primitives
    let miri_results = rust_container
        .with_exec(vec![
            "cargo", "+nightly", "miri", "test",
            "--package", "wrt-sync",
            "--package", "wrt-foundation",
            "--package", "wrt-error",
            "--lib"
        ])
        .stdout().await
        .context("Failed to run Miri tests")?;

    // --- Coverage Analysis Pipeline ---
    info!("Generating comprehensive code coverage...");
    
    let coverage_container = rust_container
        .with_exec(vec!["cargo", "install", "cargo-llvm-cov", "--locked"]);

    // Clean previous coverage runs
    let coverage_container = coverage_container
        .with_exec(vec!["cargo", "llvm-cov", "clean", "--workspace"]);

    // Generate comprehensive coverage report
    let coverage_container = coverage_container
        .with_exec(vec![
            "cargo", "llvm-cov",
            "--all-features",
            "--workspace",
            "--html",
            "--output-dir", "/src/target/coverage/html",
            "--lcov", "--output-path", "/src/target/coverage/lcov.info",
            "--json", "--output-path", "/src/target/coverage/coverage.json"
        ]);

    let coverage_artifacts_dir = coverage_container.directory("/src/target/coverage");

    // Execute coverage pipeline
    let _ = coverage_container.sync().await
        .context("Failed to execute coverage pipeline")?;

    // --- Results Processing ---
    info!("Processing verification results...");
    
    // Create verification summary
    let verification_summary = format!(
        "Kani Verification Results:\n\
         ========================\n\
         Memory Safety: {}\n\
         Concurrency Safety: {}\n\
         Type Safety: {}\n\
         \n\
         Miri Results:\n\
         =============\n\
         {}\n",
        if memory_safety_results.contains("VERIFICATION:- SUCCESSFUL") { "PASSED" } else { "REVIEW NEEDED" },
        if concurrency_results.contains("VERIFICATION:- SUCCESSFUL") { "PASSED" } else { "REVIEW NEEDED" },
        if type_safety_results.contains("VERIFICATION:- SUCCESSFUL") { "PASSED" } else { "REVIEW NEEDED" },
        if miri_results.contains("test result: ok") { "PASSED" } else { "REVIEW NEEDED" }
    );

    // Export verification results
    let results_file = client
        .directory()
        .with_new_file("verification_summary.txt", verification_summary);
    
    results_file
        .export("./target/verification_results")
        .await
        .context("Failed to export verification results")?;

    // Export coverage artifacts
    coverage_artifacts_dir
        .export("./target/coverage_report")
        .await
        .context("Failed to export coverage reports")?;

    info!("Advanced tests pipeline completed successfully.");
    info!("Verification results exported to ./target/verification_results/");
    info!("Coverage reports exported to ./target/coverage_report/");

    Ok(())
}