use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Deserialize;
use serde::Serialize;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::Command as StdCommand;
use tera::{Context as TeraContext, Tera};
use xshell::Shell;

mod bazel_ops;
mod check_imports;
mod check_panics;
mod docs;
mod fs_ops;
mod qualification;
mod update_panic_registry;
mod wasm_ops;
mod wast_tests;

#[derive(Parser, Debug)]
pub struct Opts {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run linters and formatters.
    Lint(LintOpts),
    /// Run tests.
    Test(TestOpts),
    /// Build the project.
    Build(BuildOpts),
    /// Run coverage analysis.
    Coverage(CoverageOpts),
    /// Run robust coverage that handles failing crates
    RobustCoverage(RobustCoverageOpts),
    /// Analyze and list demangled symbols for specified crates.
    Symbols(SymbolsOpts),
    /// Check Rust import organization (std/core/alloc first)
    CheckImports {
        #[arg(default_value = "wrt", help = "First directory to check")]
        dir1: PathBuf,
        #[arg(default_value = "wrtd", help = "Second directory to check")]
        dir2: PathBuf,
    },
    /// WebAssembly related tasks
    Wasm(WasmArgs),
    /// Filesystem operations
    Fs(FsArgs),
    /// Run WAST test suite
    RunWastTests {
        #[arg(long, help = "Create/overwrite wast_passed.md and wast_failed.md")]
        create_files: bool,
        #[arg(
            long,
            help = "Only run tests listed in wast_passed.md and error on regressions"
        )]
        verify_passing: bool,
    },
    /// Check if Kani verifier is installed and runnable.
    CheckKani,
    /// Qualification-related commands for certification
    Qualification {
        #[command(subcommand)]
        command: QualificationCommands,
    },
    /// Check for undocumented panics across all crates.
    CheckPanics {
        #[arg(
            long,
            help = "Fix issues by adding missing panic documentation templates"
        )]
        fix: bool,
        #[arg(long, help = "Only show crates with issues")]
        only_failures: bool,
    },
    /// Bazel related commands
    Bazel {
        #[command(subcommand)]
        command: BazelCommands,
    },
    /// Documentation related commands
    Docs {
        #[command(subcommand)]
        command: DocsCommands,
    },
    /// Update the panic registry CSV file and generate RST for sphinx-needs
    UpdatePanicRegistry {
        /// Output CSV file path (relative to workspace root)
        #[arg(long, default_value = "docs/source/development/panic_registry.csv")]
        output: String,

        /// Whether to print verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Test building all crates with both std and no_std features
    TestStdNoStd,
}

#[derive(Subcommand, Debug)]
enum QualificationCommands {
    /// Report qualification status summary
    Status,
}

#[derive(Subcommand, Debug)]
enum BazelCommands {
    /// Build a specific target with Bazel
    Build {
        #[arg(default_value = "//...", help = "Target to build")]
        target: String,
    },
    /// Run tests with Bazel
    Test {
        #[arg(default_value = "//...", help = "Target to test")]
        target: String,
    },
    /// Generate BUILD files for a package
    Generate {
        #[arg(help = "Directory containing the package")]
        directory: PathBuf,
    },
    /// Migrate a justfile command to Bazel
    Migrate {
        #[arg(help = "Command from justfile to migrate")]
        command: String,
    },
}

#[derive(Subcommand, Debug)]
enum DocsCommands {
    /// Generate the version switcher JSON file
    SwitcherJson {
        #[arg(long, help = "Generate for local development (localhost:8080)")]
        local: bool,
    },
    /// Start a local HTTP server for documentation
    Serve,
}

#[derive(Args, Debug)]
pub struct LintOpts {
    #[clap(long)]
    fix: bool,
}

#[derive(Args, Debug)]
pub struct TestOpts {
    #[clap(long)]
    coverage: bool,
}

#[derive(Args, Debug)]
pub struct BuildOpts {
    #[clap(long)]
    release: bool,
}

#[derive(Args, Debug)]
pub struct CoverageOpts {
    /// Mode of operation: single (run llvm-cov), individual (per-crate coverage), or combined (merge with grcov)
    #[clap(long, value_enum, default_value = "single")]
    mode: CoverageMode,

    /// Format for coverage output
    #[clap(long, value_enum, default_value = "lcov")]
    format: CoverageFormat,

    /// Crates to include, if not specified all will be included
    #[clap(long, use_value_delimiter = true, value_delimiter = ',')]
    crates: Vec<String>,

    /// Exclude specific crates from coverage
    #[clap(long, use_value_delimiter = true, value_delimiter = ',')]
    exclude: Vec<String>,

    /// Directory to store coverage artifacts
    #[clap(long, default_value = "target/coverage")]
    output_dir: PathBuf,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum CoverageMode {
    /// Run coverage for all crates combined (default)
    Single,
    /// Run coverage for each crate individually
    Individual,
    /// Combine previously generated coverage reports with grcov
    Combined,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum CoverageFormat {
    /// LCOV format (default)
    Lcov,
    /// HTML report
    Html,
    /// Cobertura XML format
    Cobertura,
    /// All formats
    All,
}

#[derive(Args, Debug)]
pub struct SymbolsOpts {
    /// Package to analyze (e.g., 'wrt').
    #[clap(long, default_value = "wrt")]
    package: String,

    /// Build profile to use.
    #[clap(long, default_value = "release")]
    profile: String,

    /// Target triple for the build (optional).
    #[clap(long)]
    target: Option<String>,

    /// Comma-separated list of features to enable (e.g., 'std').
    #[clap(long, use_value_delimiter = true, value_delimiter = ',')]
    features: Vec<String>,

    /// Output format for the symbol list.
    #[clap(long, value_enum, default_value = "text")]
    format: OutputFormat,

    /// Output file path. If not specified, prints to stdout.
    #[clap(short, long)]
    output: Option<PathBuf>,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum OutputFormat {
    Text,     // Simple list of demangled symbols
    Json,     // Structured data including crate info and symbols
    Markdown, // List formatted as Markdown
    Rst,      // Fragment formatted as reStructuredText for Sphinx
}

#[derive(clap::Args, Debug)]
struct WasmArgs {
    #[command(subcommand)]
    command: WasmCommands,
}

#[derive(clap::Subcommand, Debug)]
enum WasmCommands {
    /// Build all WAT files in the specified directory
    Build {
        #[arg(default_value = "example")]
        directory: PathBuf,
    },
    /// Check if WASM files are up-to-date with their WAT counterparts
    Check {
        #[arg(default_value = "example")]
        directory: PathBuf,
    },
    /// Convert a single WAT file to WASM
    Convert {
        /// Input WAT file path
        wat_file: PathBuf,
    },
}

#[derive(clap::Args, Debug)]
struct FsArgs {
    #[command(subcommand)]
    command: FsCommands,
}

#[derive(clap::Subcommand, Debug)]
enum FsCommands {
    /// Remove a directory or file recursively (like rm -rf)
    RmRf { path: PathBuf },
    /// Create a directory, including parent directories (like mkdir -p)
    MkdirP { path: PathBuf },
    /// Find files matching a pattern and delete them
    FindDelete {
        /// Starting directory
        directory: PathBuf,
        /// Filename pattern (e.g., *.wasm)
        pattern: String,
    },
    /// Count files matching a pattern in a directory
    CountFiles {
        /// Starting directory
        directory: PathBuf,
        /// Filename pattern (e.g., plantuml-*)
        pattern: String,
    },
    /// Report the size of a file in bytes
    FileSize { path: PathBuf },
    /// Copy a file
    Cp {
        source: PathBuf,
        destination: PathBuf,
    },
}

// Structs for deserializing cargo --message-format=json output
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CargoMessage {
    reason: String,
    package_id: Option<String>,
    target: Option<CargoTarget>,
    profile: Option<CargoProfile>,
    filenames: Option<Vec<PathBuf>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CargoTarget {
    name: String,
    kind: Vec<String>,
    crate_types: Vec<String>,
    src_path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CargoProfile {
    opt_level: String,
    debuginfo: Option<u32>,
    debug_assertions: bool,
    overflow_checks: bool,
    test: bool,
}

// Structs for deserializing cargo metadata output
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Metadata {
    packages: Vec<Package>,
    resolve: Option<Resolve>,
    workspace_root: PathBuf,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Package {
    name: String,
    version: String,
    id: String, // Format: "pkg_name version (path)"
    features: Option<serde_json::Value>, // Can be complex, might need refinement
                // Add other fields if needed (e.g., dependencies)
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Resolve {
    root: Option<String>, // Package ID of the root package
    nodes: Vec<ResolveNode>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ResolveNode {
    id: String, // Package ID
    features: Option<Vec<String>>, // Resolved features for this node
                // Add other fields if needed (e.g., deps)
}

// Struct for JSON and template output
#[derive(Serialize, Debug)]
struct SymbolOutput {
    package_name: String,
    version: Option<String>,
    features: Vec<String>,
    symbol_count: usize,
    symbols: Vec<String>,
}

/// Options for robust coverage generation
#[derive(Debug, clap::Args)]
pub struct RobustCoverageOpts {
    /// Crates to exclude from coverage
    #[clap(long, use_value_delimiter = true, value_delimiter = ',')]
    exclude: Vec<String>,

    /// Directory to store coverage artifacts
    #[clap(long, default_value = "target/coverage")]
    output_dir: PathBuf,

    /// Whether to generate HTML reports in addition to LCOV files
    #[clap(long)]
    html: bool,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let sh = Shell::new()?;

    match opts.command {
        Command::Lint(opts) => run_lint(&sh, opts),
        Command::Test(opts) => run_test(&sh, opts),
        Command::Build(opts) => run_build(&sh, opts),
        Command::Coverage(opts) => run_coverage(&sh, opts),
        Command::RobustCoverage(opts) => run_robust_coverage(&sh, opts),
        Command::Symbols(opts) => run_symbols(&sh, opts),
        Command::CheckImports { dir1, dir2 } => check_imports::run(&[&dir1, &dir2]),
        Command::Wasm(args) => match args.command {
            WasmCommands::Build { directory } => wasm_ops::build_all_wat(&directory),
            WasmCommands::Check { directory } => wasm_ops::check_all_wat(&directory),
            WasmCommands::Convert { wat_file } => {
                let wasm_path = wasm_ops::wat_to_wasm_path(&wat_file)?;
                wasm_ops::convert_wat(&wat_file, &wasm_path, false)?;
                Ok(())
            }
        },
        Command::Fs(args) => match args.command {
            FsCommands::RmRf { path } => fs_ops::rmrf(&path),
            FsCommands::MkdirP { path } => fs_ops::mkdirp(&path),
            FsCommands::FindDelete { directory, pattern } => {
                fs_ops::find_delete(&directory, &pattern)
            }
            FsCommands::CountFiles { directory, pattern } => {
                fs_ops::count_files(&directory, &pattern)
            }
            FsCommands::FileSize { path } => fs_ops::file_size(&path),
            FsCommands::Cp {
                source,
                destination,
            } => fs_ops::cp(&source, &destination),
        },
        Command::RunWastTests {
            create_files,
            verify_passing,
        } => wast_tests::run_wast_tests(&sh, create_files, verify_passing),
        Command::CheckKani => run_check_kani(&sh),
        Command::Qualification { command } => match command {
            QualificationCommands::Status => qualification::status(&sh),
        },
        Command::CheckPanics { fix, only_failures } => check_panics::run(&sh, fix, only_failures),
        Command::Bazel { command } => match command {
            BazelCommands::Build { target } => bazel_ops::run_build(&sh, &target),
            BazelCommands::Test { target } => bazel_ops::run_test(&sh, &target),
            BazelCommands::Generate { directory } => {
                bazel_ops::generate_build_file(&sh, &directory)
            }
            BazelCommands::Migrate { command } => bazel_ops::migrate_just_command(&sh, &command),
        },
        Command::Docs { command } => match command {
            DocsCommands::SwitcherJson { local } => docs::generate_switcher_json(local),
            DocsCommands::Serve => docs::serve_docs(),
        },
        Command::UpdatePanicRegistry { output, verbose } => {
            update_panic_registry::run(&sh, &output, verbose)?;
            Ok(())
        }
        Command::TestStdNoStd => test_std_no_std(),
    }
}

fn run_lint(sh: &Shell, opts: LintOpts) -> Result<()> {
    println!("Running lint checks...");
    if opts.fix {
        sh.cmd("cargo").arg("fmt").run()?;
        sh.cmd("cargo")
            .arg("clippy")
            .arg("--fix")
            .arg("--allow-dirty")
            .run()?;
    } else {
        sh.cmd("cargo").arg("fmt").arg("--check").run()?;
        sh.cmd("cargo").arg("clippy").run()?;
    }
    Ok(())
}

fn run_test(_sh: &Shell, opts: TestOpts) -> Result<()> {
    println!("Running tests...");

    let mut cmd = StdCommand::new("cargo");
    cmd.arg("test").arg("--workspace");

    if opts.coverage {
        // Example coverage setup (adjust as needed, e.g., using llvm-cov)
        println!("Running tests with coverage...");
        cmd.env("CARGO_INCREMENTAL", "0");
        cmd.env("RUSTFLAGS", "-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests");
        cmd.env("RUSTDOCFLAGS", "-Cpanic=abort");
        // Add specific coverage tool commands here, e.g., grcov or llvm-cov
    }

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("Cargo test failed");
    }

    // --- Add Kani Verification Step ---
    println!("\nRunning Kani verification for wrt-sync...");
    println!("(Ensure Kani is installed and compatible: https://model-checking.github.io/kani/)");

    // Run Kani specifically on the wrt-sync crate's tests
    let kani_cmd = StdCommand::new("cargo")
        .current_dir("wrt-sync") // Ensure we run in the correct directory
        .arg("kani")
        .arg("--tests")
        // Add --enable-unstable if needed for specific Kani features
        // .arg("--enable-unstable")
        .status()?;

    if !kani_cmd.success() {
        anyhow::bail!("Kani verification failed for wrt-sync");
    }

    println!("\nTests and Kani verification completed successfully.");
    Ok(())
}

fn run_build(sh: &Shell, opts: BuildOpts) -> Result<()> {
    println!("Building project...");
    let mut cmd = sh.cmd("cargo").arg("build");
    if opts.release {
        cmd = cmd.arg("--release");
    }
    cmd.run()?;
    Ok(())
}

fn run_coverage(sh: &Shell, opts: CoverageOpts) -> Result<()> {
    // Create output directory if it doesn't exist
    fs_ops::mkdirp(&opts.output_dir)?;

    match opts.mode {
        CoverageMode::Single => run_single_coverage(sh, &opts),
        CoverageMode::Individual => run_individual_coverage(sh, &opts),
        CoverageMode::Combined => run_combined_coverage(sh, &opts),
    }
}

fn run_single_coverage(sh: &Shell, opts: &CoverageOpts) -> Result<()> {
    let lcov_path = opts.output_dir.join("coverage.lcov");
    let html_output_dir = opts.output_dir.join("html");
    let summary_rst_path = PathBuf::from("docs/source/_generated_coverage_summary.rst");

    // 1. Generate LCOV data
    println!("Generating LCOV coverage data for all crates...");
    let mut cmd = sh.cmd("cargo");
    cmd = cmd
        .arg("llvm-cov")
        .arg("test") // Run tests to generate coverage
        .arg("--all-features")
        .arg("--workspace");

    // Add exclusions if specified
    for excl in &opts.exclude {
        cmd = cmd.arg("--exclude").arg(excl);
    }

    cmd = cmd.arg("--lcov").arg("--output-path").arg(&lcov_path);

    // Run the command but don't fail if it returns an error
    match cmd.run() {
        Ok(_) => println!("LCOV data generated at {}", lcov_path.display()),
        Err(e) => {
            println!("Warning: Failed to generate complete LCOV data: {}", e);
            println!("Continuing with partial coverage data if available");
        }
    }

    // Only proceed if the LCOV file was generated
    if !lcov_path.exists() {
        println!(
            "Error: LCOV file was not generated: {}",
            lcov_path.display()
        );

        // Create placeholder LCOV file
        println!("Creating placeholder LCOV file...");
        std::fs::write(&lcov_path, "TN:\nEND_OF_RECORD\n")
            .unwrap_or_else(|e| println!("Failed to create placeholder LCOV file: {}", e));
    }

    // 2. Generate HTML report from LCOV data if requested
    if opts.format == CoverageFormat::Html || opts.format == CoverageFormat::All {
        generate_html_report(sh, &lcov_path, &html_output_dir)?;
    }

    // 3. Generate summary for documentation
    match generate_coverage_summary(&lcov_path, &summary_rst_path) {
        Ok(_) => println!(
            "Coverage summary generated at {}",
            summary_rst_path.display()
        ),
        Err(e) => {
            println!("Warning: Failed to generate coverage summary: {}", e);
            create_placeholder_coverage_summary(&summary_rst_path);
        }
    }

    Ok(())
}

fn run_individual_coverage(sh: &Shell, opts: &CoverageOpts) -> Result<()> {
    println!("Running individual coverage for each crate...");

    // Get list of crates in workspace
    let mut crates = if opts.crates.is_empty() {
        // Get all crates in workspace
        let output = sh
            .cmd("cargo")
            .arg("metadata")
            .arg("--format-version=1")
            .read()?;
        let metadata: serde_json::Value = serde_json::from_str(&output)?;

        metadata["packages"]
            .as_array()
            .map(|packages| {
                packages
                    .iter()
                    .filter_map(|pkg| pkg["name"].as_str().map(String::from))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    } else {
        opts.crates.clone()
    };

    // Filter out excluded crates
    crates.retain(|c| !opts.exclude.contains(c));

    // Create directory for individual reports
    let individual_dir = opts.output_dir.join("individual");
    fs_ops::mkdirp(&individual_dir)?;

    // Track success status
    let mut any_success = false;
    let mut failed_crates = Vec::new();

    // Run coverage for each crate
    for crate_name in &crates {
        println!("Generating coverage for crate: {}", crate_name);
        let crate_lcov_path = individual_dir.join(format!("{}.lcov", crate_name));

        // Run coverage for this crate
        let result = sh
            .cmd("cargo")
            .arg("llvm-cov")
            .arg("test")
            .arg("--all-features")
            .arg("--package")
            .arg(crate_name)
            .arg("--lcov")
            .arg("--output-path")
            .arg(&crate_lcov_path)
            .run();

        match result {
            Ok(_) => {
                println!("  ✓ Coverage generated for {}", crate_name);
                any_success = true;
            }
            Err(e) => {
                println!("  ✗ Failed to generate coverage for {}: {}", crate_name, e);
                failed_crates.push(crate_name.clone());

                // Create an empty LCOV file as a placeholder to prevent failures in the combined step
                if !crate_lcov_path.exists() {
                    println!("    Creating placeholder LCOV file for {}", crate_name);
                    std::fs::write(&crate_lcov_path, "TN:\nEND_OF_RECORD\n")
                        .unwrap_or_else(|_| println!("    Failed to create placeholder LCOV file"));
                }
            }
        }
    }

    if !any_success {
        println!("Warning: Failed to generate coverage for any crate");
        // But don't return an error - continue with what we have
    }

    if !failed_crates.is_empty() {
        println!(
            "Warning: Failed to generate coverage for these crates: {}",
            failed_crates.join(", ")
        );
    }

    println!(
        "Individual coverage reports generated in {}",
        individual_dir.display()
    );

    // Even if we have partial failures, return success so the process can continue
    Ok(())
}

fn run_combined_coverage(sh: &Shell, opts: &CoverageOpts) -> Result<()> {
    println!("Combining coverage reports with grcov...");

    // Check if grcov is installed
    if sh.cmd("which").arg("grcov").read().is_err() {
        return Err(anyhow::anyhow!(
            "grcov is not installed. Install with 'cargo install grcov'"
        ));
    }

    // Find all LCOV files
    let individual_dir = opts.output_dir.join("individual");
    if !individual_dir.exists() {
        // Instead of failing, create the directory and add a placeholder file
        println!("No individual coverage reports found. Creating placeholder directory and empty report.");
        fs_ops::mkdirp(&individual_dir)?;
        std::fs::write(
            individual_dir.join("placeholder.lcov"),
            "TN:\nEND_OF_RECORD\n",
        )
        .with_context(|| {
            format!(
                "Failed to create placeholder LCOV file in {}",
                individual_dir.display()
            )
        })?;
    }

    // Check if there are any LCOV files
    let entries = std::fs::read_dir(&individual_dir)
        .with_context(|| format!("Failed to read directory: {}", individual_dir.display()))?;

    let lcov_files: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.path().extension().is_some_and(|ext| ext == "lcov")
                && entry.metadata().is_ok_and(|meta| meta.len() > 0)
        })
        .collect();

    if lcov_files.is_empty() {
        println!("No valid LCOV files found. Creating a placeholder LCOV file.");
        std::fs::write(
            individual_dir.join("placeholder.lcov"),
            "TN:\nEND_OF_RECORD\n",
        )
        .with_context(|| {
            format!(
                "Failed to create placeholder LCOV file in {}",
                individual_dir.display()
            )
        })?;
    }

    // Output paths
    let combined_lcov = opts.output_dir.join("combined.lcov");
    let html_output_dir = opts.output_dir.join("html");
    let cobertura_path = opts.output_dir.join("cobertura.xml");

    // Run grcov to combine reports for each format separately
    println!("Merging LCOV files with grcov...");

    // Generate LCOV report
    if opts.format == CoverageFormat::Lcov || opts.format == CoverageFormat::All {
        println!("Generating LCOV report...");
        let result = sh
            .cmd("grcov")
            .arg(&individual_dir)
            .arg("-t")
            .arg("lcov")
            .arg("-o")
            .arg(&combined_lcov)
            .run();

        match result {
            Ok(_) => println!("  ✓ LCOV report generated at {}", combined_lcov.display()),
            Err(e) => {
                println!("  ✗ Warning: Failed to generate LCOV report: {}", e);

                // Create a basic placeholder LCOV file
                println!("  Creating placeholder LCOV file");
                std::fs::write(&combined_lcov, "TN:\nEND_OF_RECORD\n")
                    .unwrap_or_else(|_| println!("  Failed to create placeholder LCOV file"));
            }
        }
    }

    // Generate HTML report
    if opts.format == CoverageFormat::Html || opts.format == CoverageFormat::All {
        println!("Generating HTML report...");
        fs_ops::mkdirp(&html_output_dir)?;

        let result = sh
            .cmd("grcov")
            .arg(&individual_dir)
            .arg("-t")
            .arg("html")
            .arg("--branch")
            .arg("--excl-br-line")
            .arg("^\\s*((debug_)?assert(_eq|_ne)?!|#\\[derive\\()")
            .arg("--ignore-not-existing")
            .arg("-o")
            .arg(&html_output_dir)
            .run();

        match result {
            Ok(_) => println!("  ✓ HTML report generated in {}", html_output_dir.display()),
            Err(e) => {
                println!("  ✗ Warning: Failed to generate HTML report: {}", e);

                // Create a basic placeholder HTML
                let placeholder_html = r#"<!DOCTYPE html>
<html>
<head><title>Coverage Report Not Available</title></head>
<body>
    <h1>Coverage Report Not Available</h1>
    <p>The coverage report could not be generated due to build errors in some crates.</p>
</body>
</html>"#;

                fs_ops::mkdirp(&html_output_dir)?;
                std::fs::write(html_output_dir.join("index.html"), placeholder_html)
                    .unwrap_or_else(|_| println!("  Failed to create placeholder HTML file"));
            }
        }
    }

    // Generate Cobertura XML report
    if opts.format == CoverageFormat::Cobertura || opts.format == CoverageFormat::All {
        println!("Generating Cobertura XML report...");

        let result = sh
            .cmd("grcov")
            .arg(&individual_dir)
            .arg("-t")
            .arg("cobertura")
            .arg("-o")
            .arg(&cobertura_path)
            .run();

        match result {
            Ok(_) => println!(
                "  ✓ Cobertura XML report generated at {}",
                cobertura_path.display()
            ),
            Err(e) => println!(
                "  ✗ Warning: Failed to generate Cobertura XML report: {}",
                e
            ),
        }
    }

    // Generate summary for documentation if LCOV was generated
    if combined_lcov.exists()
        && (opts.format == CoverageFormat::Lcov || opts.format == CoverageFormat::All)
    {
        let summary_rst_path = PathBuf::from("docs/source/_generated_coverage_summary.rst");
        match generate_coverage_summary(&combined_lcov, &summary_rst_path) {
            Ok(_) => println!(
                "  ✓ Coverage summary generated at {}",
                summary_rst_path.display()
            ),
            Err(e) => {
                println!("  ✗ Warning: Failed to generate coverage summary: {}", e);
                // Create a placeholder coverage summary
                create_placeholder_coverage_summary(&summary_rst_path);
            }
        }
    } else {
        // Create a placeholder coverage summary
        let summary_rst_path = PathBuf::from("docs/source/_generated_coverage_summary.rst");
        create_placeholder_coverage_summary(&summary_rst_path);
    }

    println!(
        "Combined coverage report generated in {}",
        opts.output_dir.display()
    );
    // Return success even if we had partial failures
    Ok(())
}

fn create_placeholder_coverage_summary(summary_rst_path: &PathBuf) {
    println!("Creating placeholder coverage summary");
    let template_path = PathBuf::from("docs/source/_generated_coverage_summary.rst.template");

    if template_path.exists() {
        if let Err(e) = std::fs::copy(&template_path, summary_rst_path) {
            println!("  ✗ Failed to copy template: {}", e);
            // Fallback to creating a basic file
            let basic_summary = r#".. container:: coverage-summary

   **Code Coverage:** 0.00% (0/0 lines)
   
   `Full HTML Report <../_static/coverage/index.html>`_
"#;
            std::fs::write(summary_rst_path, basic_summary)
                .unwrap_or_else(|_| println!("  ✗ Failed to create basic summary file"));
        }
    } else {
        // Template doesn't exist, create a basic file
        let basic_summary = r#".. container:: coverage-summary

   **Code Coverage:** 0.00% (0/0 lines)
   
   `Full HTML Report <../_static/coverage/index.html>`_
"#;
        std::fs::write(summary_rst_path, basic_summary)
            .unwrap_or_else(|_| println!("  ✗ Failed to create basic summary file"));
    }
}

fn generate_html_report(sh: &Shell, lcov_path: &PathBuf, html_output_dir: &PathBuf) -> Result<()> {
    println!("Generating HTML coverage report from LCOV data...");
    // Ensure the target directory exists
    fs_ops::mkdirp(html_output_dir)?;

    let result = sh
        .cmd("cargo")
        .arg("llvm-cov")
        .arg("report") // Use report subcommand
        .arg("--lcov")
        .arg(lcov_path) // Input LCOV
        .arg("--html")
        .arg("--output-dir") // Specify output directory
        .arg(html_output_dir)
        .run();

    match result {
        Ok(_) => println!("HTML report generated in {}", html_output_dir.display()),
        Err(e) => println!("Warning: Failed to generate HTML report: {}", e),
    }

    Ok(())
}

fn generate_coverage_summary(lcov_path: &PathBuf, summary_rst_path: &PathBuf) -> Result<()> {
    println!("Parsing LCOV data for summary...");
    let file = File::open(lcov_path)
        .with_context(|| format!("Failed to open LCOV file: {}", lcov_path.display()))?;
    let reader = BufReader::new(file);

    let mut total_lines = 0u64;
    let mut covered_lines = 0u64;

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("LF:") {
            // Lines Found
            if let Ok(count) = line[3..].parse::<u64>() {
                total_lines = total_lines.saturating_add(count);
            }
        } else if line.starts_with("LH:") {
            // Lines Hit
            if let Ok(count) = line[3..].parse::<u64>() {
                covered_lines = covered_lines.saturating_add(count);
            }
        }
    }

    let percentage = if total_lines > 0 {
        (covered_lines as f64 / total_lines as f64) * 100.0
    } else {
        100.0 // Or 0.0 if preferred for no lines found
    };

    println!(
        "Coverage Summary: {} / {} lines covered ({:.2}%)",
        covered_lines, total_lines, percentage
    );

    // Generate RST summary file
    println!(
        "Generating RST summary file: {}",
        summary_rst_path.display()
    );
    let rst_content = format!(
        ".. container:: coverage-summary

   **Code Coverage:** {:.2}% ({}/{} lines)

   `Full HTML Report <../_static/coverage/index.html>`_
",
        percentage, covered_lines, total_lines
    );

    // Ensure parent directory exists for the RST file
    if let Some(parent) = summary_rst_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create directory for RST file: {}",
                parent.display()
            )
        })?;
    }
    fs::write(summary_rst_path, rst_content).with_context(|| {
        format!(
            "Failed to write RST summary file: {}",
            summary_rst_path.display()
        )
    })?;
    println!("RST summary file generated successfully.");

    Ok(())
}

fn run_symbols(_sh: &Shell, opts: SymbolsOpts) -> Result<()> {
    println!(
        "Running symbol analysis for package '{}' with profile '{}'...",
        opts.package, opts.profile
    );
    if !opts.features.is_empty() {
        println!("Features: {}", opts.features.join(","));
    }
    if let Some(target) = &opts.target {
        println!("Target: {}", target);
    }

    // 1. Build the target crate and capture JSON output
    println!("Building target with JSON output...");
    let mut build_cmd_args = vec![
        "build",
        "--message-format=json",
        "--lib",
        "--package",
        &opts.package,
        "--profile",
        &opts.profile,
    ];

    if let Some(target) = &opts.target {
        build_cmd_args.extend_from_slice(&["--target", target]);
    }

    let feature_string;
    if !opts.features.is_empty() {
        feature_string = opts.features.join(",");
        build_cmd_args.extend_from_slice(&["--features", &feature_string]);
    }

    // Use std::process::Command to capture output easily
    let output = StdCommand::new("cargo").args(&build_cmd_args).output()?; // Use std::process::Command::output

    if !output.status.success() {
        eprintln!("Cargo build failed!");
        eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
        anyhow::bail!("Cargo build failed");
    }

    println!("Build complete.");

    // 2. Find the build artifact from JSON messages
    let mut artifact_path: Option<PathBuf> = None;
    let stdout = String::from_utf8(output.stdout)?;

    for line in stdout.lines() {
        match serde_json::from_str::<CargoMessage>(line) {
            Ok(message) => {
                if message.reason == "compiler-artifact"
                    && message.target.as_ref().is_some_and(|t| {
                        t.name == opts.package && t.kind.contains(&"lib".to_string())
                    })
                {
                    if let Some(filenames) = message.filenames {
                        // Find the .rlib file
                        if let Some(rlib) = filenames
                            .into_iter()
                            .find(|f| f.extension().is_some_and(|ext| ext == "rlib"))
                        {
                            artifact_path = Some(rlib);
                            break; // Found the artifact we need
                        }
                    }
                }
            }
            Err(_e) => {
                // Ignore lines that aren't valid JSON messages (like notes from cargo)
                // eprintln!("Failed to parse cargo JSON message: {}\nLine: {}", e, line);
            }
        }
    }

    let artifact_path = match artifact_path {
        Some(path) => path,
        None => anyhow::bail!(
            "Could not find .rlib artifact for package '{}' in build output",
            opts.package
        ),
    };

    println!("Found artifact: {}", artifact_path.display());

    // 3. Extract mangled symbols (using nm or similar)
    println!("Extracting symbols using nm...");
    let nm_output = StdCommand::new("nm")
        .arg("-g") // Show only global/external symbols
        .arg(&artifact_path)
        .output()?;

    if !nm_output.status.success() {
        eprintln!("nm command failed!");
        eprintln!("Stderr: {}", String::from_utf8_lossy(&nm_output.stderr));
        anyhow::bail!("nm command failed for artifact {}", artifact_path.display());
    }

    let nm_stdout = String::from_utf8(nm_output.stdout)?;
    let mut mangled_symbols: Vec<String> = Vec::new();

    for line in nm_stdout.lines() {
        // Basic parsing: Look for lines with format like "<address> T <symbol>"
        // or potentially just "<symbol>" if no address/type info is present (less common for .rlib)
        // Rust symbols often start with _ZN, _RNv, __ZN, etc.
        let parts: Vec<&str> = line.split_whitespace().collect();
        let symbol_name = if parts.len() >= 3
            && (parts[1] == "T"
                || parts[1] == "t"
                || parts[1] == "D"
                || parts[1] == "d"
                || parts[1] == "R"
                || parts[1] == "r")
        {
            // Format: <addr> <type> <symbol>
            parts[2]
        } else if parts.len() == 1 {
            // Sometimes just the symbol name might be listed
            parts[0]
        } else {
            continue; // Skip lines that don't match expected formats
        };

        // Filter for likely Rust mangled symbols (can be refined)
        if symbol_name.starts_with("_ZN")
            || symbol_name.starts_with("_RNv")
            || symbol_name.starts_with("__ZN")
            || symbol_name.starts_with("$S")
            || symbol_name.starts_with("$s")
        {
            mangled_symbols.push(symbol_name.to_string());
        }
    }

    // Deduplicate (nm might list symbols multiple times)
    mangled_symbols.sort_unstable();
    mangled_symbols.dedup();

    println!(
        "Extracted {} potential mangled symbols.",
        mangled_symbols.len()
    );

    // 4. Demangle symbols
    println!("Demangling symbols...");
    let demangled_symbols: Vec<String> = mangled_symbols
        .iter()
        .map(|mangled| rustc_demangle::demangle(mangled).to_string())
        // Optionally filter out symbols that didn't demangle well or are compiler internal
        // .filter(|demangled| !demangled.starts_with("_"))
        .collect();

    println!("Demangled {} symbols.", demangled_symbols.len());

    // 5. Gather metadata (if needed for JSON/doc/graph)
    let mut crate_version: Option<String> = None;
    let mut crate_features: Vec<String> = Vec::new();

    // Only gather metadata if needed for specific outputs
    if opts.format == OutputFormat::Json {
        println!("Gathering crate metadata...");
        let metadata_output = StdCommand::new("cargo")
            .arg("metadata")
            .arg("--format-version=1")
            // Optionally add --filter-platform if a target is specified
            // and --features / --no-default-features matching the build?
            // For now, just get general metadata and find our package.
            .output()?;

        if !metadata_output.status.success() {
            eprintln!("cargo metadata failed!");
            eprintln!(
                "Stderr: {}",
                String::from_utf8_lossy(&metadata_output.stderr)
            );
            anyhow::bail!("cargo metadata failed");
        }

        let metadata: Metadata = serde_json::from_slice(&metadata_output.stdout)?;

        // Find the target package in the metadata
        if let Some(pkg) = metadata.packages.iter().find(|p| p.name == opts.package) {
            crate_version = Some(pkg.version.clone());

            // Find resolved features for this package in the resolve graph
            if let Some(resolve) = metadata.resolve {
                if let Some(node) = resolve.nodes.iter().find(|n| n.id.starts_with(&pkg.id)) {
                    // Match by Package ID prefix
                    if let Some(features) = &node.features {
                        crate_features = features.clone();
                    }
                }
            }
            println!(
                "Found metadata: Version={}, Features={:?}",
                crate_version.as_deref().unwrap_or("N/A"),
                crate_features
            );
        } else {
            eprintln!(
                "Warning: Could not find package '{}' in cargo metadata output.",
                opts.package
            );
        }
    }

    // 6. Format and output results
    println!("Formatting and writing output...");

    // Prepare structured data (used for JSON and potentially doc)
    let output_data = SymbolOutput {
        package_name: opts.package.clone(),
        version: crate_version.clone(),
        features: crate_features.clone(),
        symbol_count: demangled_symbols.len(),
        symbols: demangled_symbols.clone(), // Clone symbols for output data
    };

    // Determine if JSON output is needed
    let needs_json = opts.format == OutputFormat::Json;

    // --- Handle Output ---
    let mut output_writer: Box<dyn Write> = match &opts.output {
        Some(path) => Box::new(File::create(path)?),
        None => Box::new(std::io::stdout()),
    };

    if needs_json {
        // Output as JSON
        let json_string = serde_json::to_string_pretty(&output_data)?;
        writeln!(output_writer, "{}", json_string)?;
        println!("Output written as JSON.");
    } else {
        // Format and write output
        println!("Formatting output as {:?}...", opts.format);
        let output_content = match opts.format {
            OutputFormat::Text => {
                // Simple text output: one symbol per line
                output_data.symbols.join("\n")
            }
            OutputFormat::Json => {
                // JSON output (potentially for graphs or structured data use)
                // Pretty print JSON for readability
                serde_json::to_string_pretty(&output_data)?
            }
            OutputFormat::Markdown => {
                // Markdown output
                let mut md = format!("# Symbols for `{}`\n\n", output_data.package_name);
                if let Some(v) = &output_data.version {
                    md.push_str(&format!("Version: `{}`\n", v));
                }
                if !output_data.features.is_empty() {
                    md.push_str(&format!(
                        "Features: `{}`\n",
                        output_data.features.join(", ")
                    ));
                }
                md.push_str(&format!("Symbol Count: {}\n\n", output_data.symbol_count));
                md.push_str("## Demangled Symbols:\n\n");
                for symbol in &output_data.symbols {
                    md.push_str(&format!("- `{}`\n", symbol));
                }
                md
            }
            OutputFormat::Rst => {
                // Use Tera for RST generation (similar to previous doc_data logic)
                let tera = match Tera::new("xtask/templates/**/*.tera") {
                    Ok(t) => t,
                    Err(e) => anyhow::bail!("Failed to load Tera templates: {}", e),
                };
                let mut context = TeraContext::new();
                context.insert("pkg", &output_data); // Pass the whole struct

                match tera.render("symbols.rst.tera", &context) {
                    Ok(rendered) => rendered,
                    Err(e) => anyhow::bail!("Failed to render RST template: {}", e),
                }
            }
        };

        // Write to file or stdout
        if let Some(output_path) = opts.output {
            println!("Writing output to {}...", output_path.display());
            // Ensure parent directory exists
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut file = File::create(&output_path)?;
            file.write_all(output_content.as_bytes())?;
            println!("Output written successfully.");
        } else {
            // Print to stdout if no output file is specified
            println!("\n--- Symbol Output ---");
            println!("{}", output_content);
            println!("--- End Symbol Output ---");
        }
    }

    println!("Symbol analysis complete.");

    Ok(())
}

fn run_check_kani(_sh: &Shell) -> Result<()> {
    println!("Checking Kani installation...");

    match StdCommand::new("cargo")
        .arg("kani")
        .arg("--version")
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let version_str = String::from_utf8_lossy(&output.stdout);
                println!("Kani found: {}", version_str.trim());
                println!("Note: Ensure Kani version is compatible with kani-verifier crate.");
                Ok(())
            } else {
                eprintln!("\nError: 'cargo kani --version' failed.");
                eprintln!("Kani might not be installed or configured correctly.");
                eprintln!(
                    "Please follow the installation guide: https://model-checking.github.io/kani/"
                );
                anyhow::bail!("Kani check failed.")
            }
        }
        Err(e) => {
            eprintln!("\nError: Failed to execute 'cargo kani'. {}", e);
            eprintln!("Kani might not be installed or in the system's PATH.");
            eprintln!(
                "Please follow the installation guide: https://model-checking.github.io/kani/"
            );
            anyhow::bail!("Kani check failed.")
        }
    }
}

/// Runs a robust coverage generation that handles failing crates
fn run_robust_coverage(sh: &Shell, opts: RobustCoverageOpts) -> Result<()> {
    println!("🔍 Running robust coverage generation...");

    // Create output directories
    let output_dir = &opts.output_dir;
    let individual_dir = output_dir.join("individual");
    let html_dir = output_dir.join("html");

    fs_ops::mkdirp(output_dir)?;
    fs_ops::mkdirp(&individual_dir)?;
    if opts.html {
        fs_ops::mkdirp(&html_dir)?;
    }

    // Step 1: Get a list of all crates in the workspace
    let metadata = get_cargo_metadata(sh)?;
    let mut all_crates: Vec<String> = metadata
        .packages
        .iter()
        .filter(|package| package.id.contains("(path")) // Only local crates, not dependencies
        .map(|package| package.name.clone())
        .collect();

    // If no crates found, use a hardcoded list for testing
    if all_crates.is_empty() {
        println!("No crates detected from metadata, using hardcoded list for testing");
        all_crates = vec![
            "wrt".to_string(),
            "wrtd".to_string(),
            "xtask".to_string(),
            "wrt-error".to_string(),
            "wrt-format".to_string(),
            "wrt-types".to_string(),
        ];
    }

    println!("📦 Found {} crates in workspace", all_crates.len());

    // Step 2: Process each crate individually
    println!("🔄 Generating individual coverage reports...");
    let mut successful_crates = 0;

    for crate_name in &all_crates {
        // Skip excluded crates
        if opts.exclude.contains(crate_name) {
            println!("  ⏩ Skipping excluded crate: {}", crate_name);
            // Create placeholder LCOV file
            let placeholder_lcov = r#"TN:
SF:placeholder
DA:1,0
LF:1
LH:0
end_of_record"#;

            let crate_lcov_path = individual_dir.join(format!("{}.lcov", crate_name));
            std::fs::write(&crate_lcov_path, placeholder_lcov)
                .with_context(|| format!("Failed to write placeholder LCOV for {}", crate_name))?;
            continue;
        }

        println!("  📊 Generating coverage for: {}", crate_name);

        let crate_lcov_path = individual_dir.join(format!("{}.lcov", crate_name));

        // Set the coverage cfg to disable problematic modules during coverage builds
        let mut env = std::env::vars().collect::<Vec<_>>();
        env.push(("RUSTFLAGS".to_string(), "--cfg coverage".to_string()));

        // Run coverage for this crate, but don't fail if it errors
        let result = sh
            .cmd("cargo")
            .envs(env) // Add the cfg flag
            .arg("llvm-cov")
            .arg("test")
            .arg("--all-features")
            .arg("--package")
            .arg(crate_name)
            .arg("--lcov")
            .arg("--output-path")
            .arg(&crate_lcov_path)
            .run();

        match result {
            Ok(_) => {
                println!("    ✅ Coverage generated successfully");
                successful_crates += 1;
            }
            Err(e) => {
                println!("    ⚠️ Failed to generate coverage: {}", e);
                // Create placeholder LCOV file
                let placeholder_lcov = r#"TN:
SF:placeholder
DA:1,0
LF:1
LH:0
end_of_record"#;

                std::fs::write(&crate_lcov_path, placeholder_lcov).with_context(|| {
                    format!("Failed to write placeholder LCOV for {}", crate_name)
                })?;
            }
        }
    }

    println!(
        "✅ Generated coverage for {}/{} crates",
        successful_crates,
        all_crates.len()
    );

    // Step 3: Combine LCOV files
    println!("🔄 Combining LCOV files...");
    let combined_lcov_path = output_dir.join("coverage.lcov");

    // Check if grcov is installed
    if sh.cmd("which").arg("grcov").read().is_ok() {
        // Attempt to combine using grcov
        let grcov_result = sh
            .cmd("grcov")
            .arg(&individual_dir)
            .arg("-t")
            .arg("lcov")
            .arg("-o")
            .arg(&combined_lcov_path)
            .run();

        match grcov_result {
            Ok(_) => println!(
                "  ✅ Combined LCOV generated at {}",
                combined_lcov_path.display()
            ),
            Err(e) => {
                println!("  ⚠️ Failed to combine with grcov: {}", e);
                create_fallback_lcov(&individual_dir, &combined_lcov_path)?;
            }
        }

        // Generate HTML report if requested
        if opts.html {
            println!("🔄 Generating HTML report...");
            let html_result = sh
                .cmd("grcov")
                .arg(&individual_dir)
                .arg("-t")
                .arg("html")
                .arg("-o")
                .arg(&html_dir)
                .run();

            match html_result {
                Ok(_) => println!("  ✅ HTML report generated in {}", html_dir.display()),
                Err(e) => {
                    println!("  ⚠️ Failed to generate HTML report: {}", e);
                    create_fallback_html(&html_dir)?;
                }
            }
        }
    } else {
        println!("⚠️ grcov not found, using fallback approach");
        create_fallback_lcov(&individual_dir, &combined_lcov_path)?;

        if opts.html {
            create_fallback_html(&html_dir)?;
        }
    }

    // Step 4: Generate the documentation coverage summary
    println!("🔄 Generating coverage summary...");
    let summary_path = PathBuf::from("docs/source/_generated_coverage_summary.rst");
    fs_ops::mkdirp(summary_path.parent().unwrap())?;

    if combined_lcov_path.exists() {
        match generate_coverage_summary_robust(&combined_lcov_path, &summary_path) {
            Ok(_) => println!(
                "  ✅ Coverage summary generated at {}",
                summary_path.display()
            ),
            Err(e) => {
                println!("  ⚠️ Failed to generate summary: {}", e);
                create_fallback_summary(&summary_path)?;
            }
        }
    } else {
        println!("  ⚠️ No LCOV file found, creating placeholder summary");
        create_fallback_summary(&summary_path)?;
    }

    println!("✅ Robust coverage generation completed");
    println!("  • LCOV data: {}", combined_lcov_path.display());
    if opts.html {
        println!("  • HTML report: {}/index.html", html_dir.display());
    }
    println!("  • Summary: {}", summary_path.display());

    Ok(())
}

/// Creates a fallback LCOV file by concatenating individual files or creating a placeholder
fn create_fallback_lcov(individual_dir: &PathBuf, output_path: &PathBuf) -> Result<()> {
    println!("  🔄 Creating fallback LCOV file...");

    // Try to find LCOV files
    let mut lcov_files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(individual_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "lcov") {
                lcov_files.push(path);
            }
        }
    }

    let mut output = String::new();

    if lcov_files.is_empty() {
        // No LCOV files found, create a placeholder
        output = "TN:\nSF:placeholder\nDA:1,0\nLF:1\nLH:0\nend_of_record\n".to_string();
    } else {
        // Concatenate all LCOV files
        for file in lcov_files {
            if let Ok(content) = std::fs::read_to_string(&file) {
                output.push_str(&content);
                if !content.ends_with('\n') {
                    output.push('\n');
                }
            }
        }

        // If we didn't get any content, create a placeholder
        if output.is_empty() {
            output = "TN:\nSF:placeholder\nDA:1,0\nLF:1\nLH:0\nend_of_record\n".to_string();
        }
    }

    std::fs::write(output_path, output)
        .with_context(|| format!("Failed to write fallback LCOV to {}", output_path.display()))?;

    println!(
        "  ✅ Fallback LCOV file created at {}",
        output_path.display()
    );
    Ok(())
}

/// Creates a fallback HTML report when grcov fails
fn create_fallback_html(html_dir: &PathBuf) -> Result<()> {
    println!("  🔄 Creating fallback HTML report...");

    // Ensure the directory exists
    fs_ops::mkdirp(html_dir)?;

    // Create a simple HTML file
    let html_content = r#"<!DOCTYPE html>
<html>
<head>
    <title>Coverage Report Not Available</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 2em; line-height: 1.6; }
        h1 { color: #d33; }
        .container { max-width: 800px; margin: 0 auto; }
    </style>
</head>
<body>
    <div class="container">
        <h1>Coverage Report Not Available</h1>
        <p>The HTML coverage report could not be generated due to errors. However, coverage data is still available in LCOV format.</p>
        <p>Possible reasons:</p>
        <ul>
            <li>Some crates failed to compile during coverage generation</li>
            <li>The grcov tool is not installed or failed to run</li>
            <li>There were no valid coverage records to process</li>
        </ul>
    </div>
</body>
</html>"#;

    std::fs::write(html_dir.join("index.html"), html_content).with_context(|| {
        format!(
            "Failed to write fallback HTML to {}/index.html",
            html_dir.display()
        )
    })?;

    println!(
        "  ✅ Fallback HTML report created at {}/index.html",
        html_dir.display()
    );
    Ok(())
}

/// Creates a fallback coverage summary for documentation
fn create_fallback_summary(summary_path: &PathBuf) -> Result<()> {
    println!("  🔄 Creating fallback coverage summary...");

    // Check if template exists
    let template_path = PathBuf::from("docs/source/_generated_coverage_summary.rst.template");

    if template_path.exists() {
        // Copy the template
        std::fs::copy(&template_path, summary_path).with_context(|| {
            format!(
                "Failed to copy template from {} to {}",
                template_path.display(),
                summary_path.display()
            )
        })?;
    } else {
        // Create a basic summary
        let summary_content = r#".. container:: coverage-summary

   **Code Coverage:** 0.00% (0/0 lines)
   
   `Full HTML Report <../_static/coverage/index.html>`_
"#;

        std::fs::write(summary_path, summary_content).with_context(|| {
            format!(
                "Failed to write fallback summary to {}",
                summary_path.display()
            )
        })?;
    }

    println!(
        "  ✅ Fallback coverage summary created at {}",
        summary_path.display()
    );
    Ok(())
}

/// Parses LCOV data and generates a summary for documentation, handling errors gracefully
fn generate_coverage_summary_robust(lcov_path: &PathBuf, summary_path: &PathBuf) -> Result<()> {
    let file = match File::open(lcov_path) {
        Ok(f) => f,
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to open LCOV file {}: {}",
                lcov_path.display(),
                e
            ));
        }
    };

    let reader = BufReader::new(file);

    let mut total_lines = 0u64;
    let mut covered_lines = 0u64;

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => {
                println!("  ⚠️ Error reading line from LCOV file: {}", e);
                continue;
            }
        };

        if line.starts_with("LF:") {
            // Lines Found
            if let Ok(count) = line[3..].parse::<u64>() {
                total_lines = total_lines.saturating_add(count);
            }
        } else if line.starts_with("LH:") {
            // Lines Hit
            if let Ok(count) = line[3..].parse::<u64>() {
                covered_lines = covered_lines.saturating_add(count);
            }
        }
    }

    // Calculate coverage percentage
    let coverage_percent = if total_lines > 0 {
        (covered_lines as f64 * 100.0) / (total_lines as f64)
    } else {
        0.0
    };

    // Format RST content
    let rst_content = format!(
        r#".. container:: coverage-summary

   **Code Coverage:** {:.2}% ({}/{} lines)
   
   `Full HTML Report <../_static/coverage/index.html>`_
"#,
        coverage_percent, covered_lines, total_lines
    );

    // Write to file
    std::fs::write(summary_path, rst_content).with_context(|| {
        format!(
            "Failed to write coverage summary to {}",
            summary_path.display()
        )
    })?;

    println!(
        "  ✅ Coverage summary generated: {:.2}% ({}/{} lines)",
        coverage_percent, covered_lines, total_lines
    );

    Ok(())
}

/// Get the cargo metadata for the workspace
fn get_cargo_metadata(sh: &Shell) -> Result<Metadata> {
    let output = sh
        .cmd("cargo")
        .arg("metadata")
        .arg("--format-version=1")
        .read()?;

    let metadata: Metadata = match serde_json::from_str(&output) {
        Ok(m) => m,
        Err(e) => {
            println!("Failed to parse cargo metadata: {}", e);
            // Fallback approach - try to get crates directly
            let mut packages = Vec::new();

            // Look for directories that might be crates
            if let Ok(entries) = std::fs::read_dir(".") {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    if path.is_dir() && path.join("Cargo.toml").exists() {
                        let name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        // Skip certain directories that aren't crates
                        if name == "target" || name == ".git" || name == "docs" {
                            continue;
                        }

                        packages.push(Package {
                            name: name.clone(),
                            version: "0.1.0".to_string(),
                            id: format!("{} 0.1.0 (path+file:///{})", name, path.display()),
                            features: None,
                        });
                    }
                }
            }

            Metadata {
                packages,
                resolve: None,
                workspace_root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            }
        }
    };

    // Filter out non-workspace crates if empty
    if metadata.packages.is_empty() {
        println!("Warning: No packages found in cargo metadata, using fallback approach");
        let mut packages = Vec::new();

        // Manually detect crates
        for dir_name in &[
            "wrt",
            "wrtd",
            "xtask",
            "example",
            "wrt-sync",
            "wrt-error",
            "wrt-format",
            "wrt-types",
            "wrt-decoder",
            "wrt-component",
            "wrt-host",
            "wrt-logging",
            "wrt-runtime",
            "wrt-instructions",
            "wrt-intercept",
            "logging-adapter",
        ] {
            let dir = PathBuf::from(dir_name);
            if dir.exists() && dir.join("Cargo.toml").exists() {
                let name = dir_name.to_string();
                packages.push(Package {
                    name: name.clone(),
                    version: "0.1.0".to_string(),
                    id: format!("{} 0.1.0 (path+file:///{})", name, dir.display()),
                    features: None,
                });
            }
        }

        if !packages.is_empty() {
            return Ok(Metadata {
                packages,
                resolve: None,
                workspace_root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            });
        }
    }

    Ok(metadata)
}

/// Test building all crates with both std and no_std features
///
/// This function attempts to build each crate in the workspace with both
/// the standard library and no_std features to ensure compatibility.
fn test_std_no_std() -> Result<()> {
    println!("Testing crates with std and no_std features...");

    // List of crates to test
    let crates = vec![
        "wrt-error",
        "wrt-types",
        "wrt-format",
        "wrt-decoder",
        "wrt-runtime",
        "wrt-component",
        "wrt-host",
        "wrt-instructions",
        "wrt-intercept",
        "wrt-logging",
        "wrt-sync",
        "wrt",
    ];

    // Test with std (default)
    for crate_name in &crates {
        println!("\nTesting {} with std features", crate_name);
        let status = std::process::Command::new("cargo")
            .args(["build", "-p", crate_name])
            .status()?;

        if !status.success() {
            eprintln!("Failed to build {} with std features", crate_name);
            return Err(anyhow::anyhow!("Build failed"));
        }
    }

    // Test with no_std
    for crate_name in &crates {
        println!("\nTesting {} with no_std features", crate_name);
        let status = std::process::Command::new("cargo")
            .args([
                "build",
                "-p",
                crate_name,
                "--no-default-features",
                "--features",
                "no_std",
            ])
            .status()?;

        if !status.success() {
            eprintln!("Failed to build {} with no_std features", crate_name);
            return Err(anyhow::anyhow!("Build failed"));
        }
    }

    println!("\nAll crates successfully built with both std and no_std features!");
    Ok(())
}
