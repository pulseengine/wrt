use std::path::PathBuf;

use anyhow::{Context as _, Result};
use clap::Parser;
use dagger_sdk::{connect_opts, Config, Query};
use eyre::eyre;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use xshell::Shell;

// Valid module declarations based on list_dir output
mod ci_advanced_tests;
mod ci_integrity_checks;
mod ci_static_analysis;
mod coverage;
mod coverage_ci;
mod coverage_simple;
mod dagger_pipelines;
mod fmt_check;
mod fs_ops;
pub mod test_runner;
mod wasm_ops;
// The following seem to be single-file modules based on list_dir
mod check_imports;
mod check_panics;
mod docs; // Assuming docs.rs is a module
mod docs_preview;
mod docs_validation;
mod generate_changelog;
mod generate_coverage_summary;
mod generate_source_needs;
mod sftp_deploy;
mod no_std_verification;
mod qualification; // Assuming qualification.rs is a module, distinct from directory
mod update_panic_registry; // Added new module
mod wrtd_build;
mod safety_verification;
mod safety_verification_unified;
mod generate_safety_summary;

// Comment out install_ops and its usage due to missing file
// mod install_ops;
// use crate::install_ops::install_tools;

// pub mod dagger_pipelines {  // This block should be removed
//     pub mod docs_pipeline;
//     //Potentially more pipelines here
// }

#[derive(Debug, Parser)]
#[clap(name = "xtask", version = "0.1.0", about = "Workspace automation tasks")]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
    #[clap(long, default_value = "./", help = "Path to the workspace root")]
    pub workspace_root: PathBuf,
    #[clap(long, default_value = "info", help = "Logging level (trace, debug, info, warn, error)")]
    pub log_level: String,
}

#[derive(Debug, Parser)]
pub enum Command {
    // Keep commands that have corresponding existing modules
    Fs(FsArgs),
    Wasm(WasmArgs),
    PublishDocsDagger(PublishDocsDaggerArgs),
    CiStaticAnalysis,
    CiAdvancedTests,
    CiIntegrityChecks,
    Coverage,
    CoverageComprehensive,
    CoverageSimple,
    GenerateCoverageSummary,
    GenerateSafetySummary,
    CheckDocsStrict,
    FmtCheck,
    RunTests,
    GenerateSourceNeeds(generate_source_needs::GenerateSourceNeedsArgs),
    VerifyNoStd(VerifyNoStdArgs),
    PreviewDocs(PreviewDocsArgs),
    ValidateDocs,
    ValidateDocsComprehensive,
    GenerateChangelog(GenerateChangelogArgs),
    DeployDocsSftp(DeployDocsSftpArgs),
    WrtdBuild(WrtdBuildArgs),
    WrtdBuildAll,
    WrtdTest,
    // Safety verification commands
    CheckRequirements,
    VerifyRequirements(VerifyRequirementsArgs),
    VerifySafety(VerifySafetyArgs),
    SafetyReport(SafetyReportArgs),
    SafetyDashboard,
    InitRequirements,
}

// Args structs for existing commands
#[derive(Debug, Parser)]
pub struct WrtdBuildArgs {
    #[clap(long, help = "Build specific binary (wrtd-std, wrtd-alloc, wrtd-nostd)")]
    pub binary: Option<String>,
    #[clap(long, help = "Build in release mode")]
    pub release: bool,
    #[clap(long, help = "Show build summary")]
    pub show_summary: bool,
    #[clap(long, help = "Test binaries after building")]
    pub test_binaries: bool,
    #[clap(long, help = "Enable cross-compilation for embedded targets")]
    pub cross_compile: bool,
}

#[derive(Debug, Parser)]
pub struct PublishDocsDaggerArgs {
    #[clap(long, help = "Directory to output the generated documentation.")]
    pub output_dir: String,
    #[clap(
        long,
        help = "One or more versions (branches/tags) to build docs for (e.g., --versions main v0.1.0). Defaults to 'main' if none specified.",
        num_args = 1.., // Expect one or more values after --versions
        default_missing_value = "main" // If --versions is present but no values, or if not present at all (requires careful thought on clap's default_value_t)
                                      // A better approach for default might be to handle it post-parsing if versions vec is empty.
                                      // For now, let's ensure it takes multiple values.
                                      // Consider clap(default_values_t = vec!["main".to_string()]) if that's desired.
    )]
    pub versions: Vec<String>, // Changed to Vec<String>
}

#[derive(Debug, Parser)]
pub struct VerifyNoStdArgs {
    #[clap(long, help = "Continue on error instead of stopping")]
    pub continue_on_error: bool,
    #[clap(long, help = "Show verbose output")]
    pub verbose: bool,
    #[clap(long, help = "Show detailed summary table")]
    pub detailed: bool,
    #[clap(long, help = "Run partial verification")]
    pub partial: bool,
}

#[derive(Debug, Parser)]
pub struct PreviewDocsArgs {
    #[clap(long, default_value = "8000", help = "Port for the preview server")]
    pub port: u16,
    #[clap(long, default_value = "docs_output/local", help = "Documentation directory to serve")]
    pub docs_dir: String,
    #[clap(long, help = "Open browser automatically")]
    pub open_browser: bool,
}

#[derive(Debug, Parser)]
pub struct GenerateChangelogArgs {
    #[clap(long, default_value = "docs/source/changelog.md", help = "Output file path for the changelog")]
    pub output: String,
    #[clap(long, help = "Generate only unreleased changes")]
    pub unreleased: bool,
    #[clap(long, help = "Install git-cliff if not found")]
    pub install_if_missing: bool,
}

#[derive(Debug, Parser)]
pub struct DeployDocsSftpArgs {
    #[clap(long, help = "SFTP server hostname or IP address")]
    pub host: Option<String>,
    #[clap(long, help = "SSH username for SFTP hosting")]
    pub username: Option<String>,
    #[clap(long, default_value = "/htdocs", help = "Target directory on remote server")]
    pub target_dir: String,
    #[clap(long, default_value = "docs_output", help = "Local documentation directory")]
    pub docs_dir: String,
    #[clap(long, help = "Path to SSH private key file")]
    pub ssh_key_path: Option<String>,
    #[clap(long, help = "Build documentation before deployment")]
    pub build_docs: bool,
    #[clap(long, help = "Show what would be deployed without making changes")]
    pub dry_run: bool,
    #[clap(long, help = "Delete remote files not present locally")]
    pub delete_remote: bool,
    #[clap(long, default_value = "22", help = "SSH port")]
    pub port: u16,
}

#[derive(Debug, Parser)]
pub struct VerifyRequirementsArgs {
    #[clap(long, default_value = "requirements.toml", help = "Path to requirements TOML file")]
    pub requirements_file: String,
    #[clap(long, help = "Generate detailed report")]
    pub detailed: bool,
    #[clap(long, help = "Skip file existence verification")]
    pub skip_files: bool,
}

#[derive(Debug, Parser)]
pub struct VerifySafetyArgs {
    #[clap(long, default_value = "requirements.toml", help = "Path to requirements TOML file")]
    pub requirements_file: String,
    #[clap(long, help = "Output format (text, json, html)")]
    pub format: Option<String>,
    #[clap(long, help = "Save report to file instead of stdout")]
    pub output: Option<String>,
}

#[derive(Debug, Parser)]
pub struct SafetyReportArgs {
    #[clap(long, default_value = "safety-report.txt", help = "Output file for safety report")]
    pub output: String,
    #[clap(long, help = "Report format (text, json, html)")]
    pub format: Option<String>,
}

#[derive(Debug, Parser)]
pub struct FsArgs {
    #[clap(subcommand)]
    pub command: FsCommands,
}

#[derive(Debug, Parser)]
pub enum FsCommands {
    RmRf { path: PathBuf },
    MkdirP { path: PathBuf },
    FindDelete { directory: PathBuf, pattern: String },
    CountFiles { directory: PathBuf, pattern: String },
    Cp { source: PathBuf, destination: PathBuf },
}

#[derive(Debug, Parser)]
pub struct WasmArgs {
    #[clap(subcommand)]
    command: WasmCommands,
}

#[derive(Debug, Parser)]
pub enum WasmCommands {
    Build { directory: PathBuf },
    Check { directory: PathBuf },
    Convert { wat_file: PathBuf },
}

// Comment out InstallArgs as its module is missing
// #[derive(Debug, Parser)]
// pub struct InstallArgs {
//     #[clap(required = true, num_args = 1.., help = "List of tools to install
// (e.g., mdbook, cargo-nextest)")]     pub tools: Vec<String>,
// }

// Make main async to support async Dagger tasks directly
#[tokio::main]
async fn main() -> Result<()> {
    let opts = Args::parse();

    // Check if we need to suppress logging for JSON output first
    let suppress_logging = matches!(&opts.command, 
        Command::VerifySafety(args) if args.format.as_deref() == Some("json"));

    if !suppress_logging {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(opts.log_level.parse::<Level>().unwrap_or(Level::INFO))
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .context("Failed to set global default tracing subscriber")?;
    } else {
        // For JSON output, set up silent logging
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::ERROR)
            .with_writer(|| std::io::empty())
            .without_time()
            .finish();
        tracing::subscriber::set_global_default(subscriber)
            .context("Failed to set silent tracing subscriber")?;
    }

    let sh = Shell::new().context("Failed to create xshell Shell")?;
    let workspace_root_for_shell = opts.workspace_root.clone();
    sh.change_dir(&workspace_root_for_shell);
    tracing::info!("Changed directory to workspace root: {:?}", workspace_root_for_shell);

    // Handle non-Dagger commands first
    match &opts.command {
        Command::GenerateSourceNeeds(args) => {
            return generate_source_needs::run_generate_source_needs(args.clone(), &sh);
        }
        Command::GenerateCoverageSummary => {
            let coverage_json = std::path::PathBuf::from("target/coverage/coverage.json");
            let output_rst =
                std::path::PathBuf::from("docs/source/_generated_coverage_summary.rst");

            if coverage_json.exists() {
                println!("Generating coverage summary from {:?}", coverage_json);
                if let Err(e) = generate_coverage_summary::generate_coverage_summary_rst(
                    &coverage_json,
                    &output_rst,
                ) {
                    eprintln!("Failed to generate coverage summary: {}", e);
                    println!("Generating placeholder instead");
                    generate_coverage_summary::generate_placeholder_coverage_summary(&output_rst)?;
                }
            } else {
                println!("No coverage data found, generating placeholder");
                generate_coverage_summary::generate_placeholder_coverage_summary(&output_rst)?;
            }
            return Ok(());
        }
        Command::GenerateSafetySummary => {
            let output_rst = std::path::PathBuf::from("docs/source/_generated_safety_summary.rst");
            
            println!("Generating safety verification summary...");
            if let Err(e) = generate_safety_summary::generate_safety_summary_rst(&output_rst) {
                eprintln!("Failed to generate safety summary: {}", e);
                println!("Generating placeholder instead");
                generate_safety_summary::generate_placeholder_safety_summary(&output_rst)?;
            }
            return Ok(());
        }
        Command::CoverageSimple => {
            // Generate simple coverage without Dagger
            coverage_simple::generate_simple_coverage()?;
            return Ok(());
        }
        Command::VerifyNoStd(args) => {
            let config = no_std_verification::NoStdConfig {
                continue_on_error: args.continue_on_error,
                verbose: args.verbose,
                detailed: args.detailed,
                partial: args.partial,
            };
            no_std_verification::run_no_std_verification(config)?;
            return Ok(());
        }
        Command::PreviewDocs(args) => {
            let config = docs_preview::DocsPreviewConfig {
                port: args.port,
                docs_dir: args.docs_dir.clone(),
                open_browser: args.open_browser,
                ..Default::default()
            };
            docs_preview::run_docs_preview(config)?;
            return Ok(());
        }
        Command::ValidateDocs => {
            docs_validation::validate_docs()?;
            return Ok(());
        }
        Command::ValidateDocsComprehensive => {
            docs_validation::check_docs_comprehensive()?;
            return Ok(());
        }
        Command::GenerateChangelog(args) => {
            let config = generate_changelog::ChangelogConfig {
                output_file: std::path::PathBuf::from(&args.output),
                unreleased_only: args.unreleased,
                install_if_missing: args.install_if_missing,
            };
            generate_changelog::generate_changelog(config)?;
            return Ok(());
        }
        Command::DeployDocsSftp(args) => {
            let config = sftp_deploy::SftpDeployConfig::from_env_and_args(
                args.host.clone(),
                args.username.clone(),
                Some(args.target_dir.clone()),
                Some(args.docs_dir.clone()),
                args.ssh_key_path.clone(),
                args.build_docs,
                args.dry_run,
                args.delete_remote,
                Some(args.port),
            )?;
            
            // Run async deployment
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(sftp_deploy::deploy_docs_sftp(config))?;
            return Ok(());
        }
        Command::WrtdBuild(args) => {
            let config = wrtd_build::WrtdBuildConfig {
                release: args.release,
                show_summary: args.show_summary,
                test_binaries: args.test_binaries,
                cross_compile: args.cross_compile,
            };
            
            if let Some(binary) = &args.binary {
                // Build specific binary
                match binary.as_str() {
                    "wrtd-std" => {
                        println!("📦 Building Standard Library Runtime (servers/desktop)...");
                        let result = wrtd_build::build_wrtd_binary(
                            "wrtd-std",
                            "std-runtime",
                            config.release,
                            None,
                        );
                        if let Err(e) = result {
                            return Err(e);
                        }
                    }
                    "wrtd-alloc" => {
                        println!("📦 Building Allocation Runtime (embedded with heap)...");
                        let result = wrtd_build::build_wrtd_binary(
                            "wrtd-alloc",
                            "alloc-runtime",
                            config.release,
                            None,
                        );
                        if let Err(e) = result {
                            return Err(e);
                        }
                    }
                    "wrtd-nostd" => {
                        println!("📦 Building No Standard Library Runtime (bare metal)...");
                        let result = wrtd_build::build_wrtd_binary(
                            "wrtd-nostd",
                            "nostd-runtime",
                            config.release,
                            None,
                        );
                        if let Err(e) = result {
                            return Err(e);
                        }
                    }
                    _ => {
                        return Err(anyhow::anyhow!("Unknown binary: {}. Valid options: wrtd-std, wrtd-alloc, wrtd-nostd", binary));
                    }
                }
            } else {
                // Build all binaries
                wrtd_build::build_all_wrtd(config)?;
            }
            return Ok(());
        }
        Command::WrtdBuildAll => {
            let config = wrtd_build::WrtdBuildConfig::default();
            wrtd_build::build_all_wrtd(config)?;
            return Ok(());
        }
        Command::WrtdTest => {
            wrtd_build::test_wrtd_modes(true)?;
            return Ok(());
        }
        // Safety verification commands
        Command::CheckRequirements => {
            let requirements_path = PathBuf::from("requirements.toml");
            safety_verification::check_requirements(&requirements_path)?;
            return Ok(());
        }
        Command::VerifyRequirements(args) => {
            let config = safety_verification::SafetyVerificationConfig {
                requirements_file: PathBuf::from(&args.requirements_file),
                verify_files: !args.skip_files,
                generate_report: true,
                ..Default::default()
            };
            
            // Use detailed flag for additional output
            if args.detailed {
                println!("🔍 Running detailed requirements verification...");
            }
            
            safety_verification::run_safety_verification(config)?;
            return Ok(());
        }
        Command::VerifySafety(args) => {
            let output_format = match args.format.as_deref() {
                Some("json") => safety_verification::OutputFormat::Json,
                Some("html") => safety_verification::OutputFormat::Html,
                _ => safety_verification::OutputFormat::Text,
            };
            
            
            let config = safety_verification::SafetyVerificationConfig {
                requirements_file: PathBuf::from(&args.requirements_file),
                output_format,
                ..Default::default()
            };
            
            if let Some(output_file) = &args.output {
                // Redirect stdout to file
                let _output = std::fs::File::create(output_file)?;
                let _guard = scopeguard::guard((), |_| {
                    // Restore stdout after writing
                });
                // Note: Actual redirection would need more complex handling
                safety_verification::run_safety_verification(config.clone())?;
            } else {
                safety_verification::run_safety_verification(config)?;
            }
            return Ok(());
        }
        Command::SafetyReport(args) => {
            let output_format = match args.format.as_deref() {
                Some("json") => safety_verification::OutputFormat::Json,
                Some("html") => safety_verification::OutputFormat::Html,
                _ => safety_verification::OutputFormat::Text,
            };
            
            let config = safety_verification::SafetyVerificationConfig {
                requirements_file: PathBuf::from("requirements.toml"),
                output_format,
                ..Default::default()
            };
            
            // Generate report and save to file
            let report_content = {
                use std::sync::Mutex;
                let _buffer = std::sync::Arc::new(Mutex::new(Vec::<u8>::new()));
                // Capture output - simplified version
                safety_verification::run_safety_verification(config.clone())?;
                // In real implementation, would capture stdout
                Vec::<u8>::new()
            };
            
            if !report_content.is_empty() {
                std::fs::write(&args.output, report_content)?;
                println!("✅ Safety report generated: {}", args.output);
            } else {
                // For now, just run the verification
                safety_verification::run_safety_verification(config)?;
            }
            return Ok(());
        }
        Command::SafetyDashboard => {
            // Run check-requirements
            println!("📋 Checking requirements traceability...");
            let requirements_path = PathBuf::from("requirements.toml");
            safety_verification::check_requirements(&requirements_path)?;
            
            println!();
            
            // Run verify-safety
            let config = safety_verification::SafetyVerificationConfig::default();
            safety_verification::run_safety_verification(config)?;
            
            return Ok(());
        }
        Command::InitRequirements => {
            let requirements_path = PathBuf::from("requirements.toml");
            safety_verification::init_requirements(&requirements_path)?;
            return Ok(());
        }
        _ => {
            // Continue to Dagger handling
        }
    }
    // Add other non-Dagger commands here if necessary, e.g.:
    // if let Command::Fs(args) = &opts.command { ... }
    // if let Command::Wasm(args) = &opts.command { ... }
    // etc.

    // Initialize Dagger client using connect_opts and a closure
    // This part should only run for Dagger-dependent commands
    let dagger_cfg = Config::default();
    let workspace_root_for_dagger = opts.workspace_root.clone(); // Separate clone for Dagger closure

    // Conditionally run Dagger connect if it's a Dagger command
    match &opts.command {
        // Match on opts.command again for Dagger commands
        Command::PublishDocsDagger(_)
        | Command::CiStaticAnalysis
        | Command::CiAdvancedTests
        | Command::CiIntegrityChecks
        | Command::Coverage
        | Command::CoverageComprehensive
        | Command::CheckDocsStrict
        | Command::FmtCheck
        | Command::RunTests => {
            connect_opts(dagger_cfg, move |query_client: Query| {
                let command_to_run = opts.command; // This will now use the moved opts.command
                let workspace_root_for_closure_dagger = workspace_root_for_dagger.clone(); // This will use the moved workspace_root_for_dagger

                async move {
                    let outcome = async {
                        match command_to_run {
                            Command::PublishDocsDagger(args) => {
                                dagger_pipelines::docs_pipeline::run_docs_pipeline(
                                    &query_client,
                                    workspace_root_for_closure_dagger,
                                    PathBuf::from(args.output_dir),
                                    if args.versions.is_empty() {
                                        vec!["main".to_string()]
                                    } else {
                                        args.versions.clone()
                                    },
                                )
                                .await
                            }
                            Command::CiStaticAnalysis => {
                                ci_static_analysis::run(&query_client).await
                            }
                            Command::CiAdvancedTests => ci_advanced_tests::run(&query_client).await,
                            Command::CiIntegrityChecks => {
                                ci_integrity_checks::run(&query_client).await
                            }
                            Command::Coverage => {
                                // Use CI-optimized coverage if we're in CI environment
                                if std::env::var("CI").is_ok() {
                                    info!("Detected CI environment, using CI-optimized coverage");
                                    coverage_ci::run_ci_coverage(&query_client).await
                                } else {
                                    // Add a timeout for coverage generation
                                    tokio::time::timeout(
                                        std::time::Duration::from_secs(300), // 5 minute timeout
                                        coverage::run_quick_coverage(&query_client),
                                    )
                                    .await
                                    .map_err(|_| {
                                        anyhow::anyhow!(
                                            "Coverage generation timed out after 5 minutes"
                                        )
                                    })?
                                }
                            }
                            Command::CoverageComprehensive => {
                                coverage::run_comprehensive_coverage(&query_client).await
                            }
                            Command::CheckDocsStrict => {
                                docs::check_docs_strict(&query_client).await
                            }
                            Command::FmtCheck => fmt_check::run(&query_client).await,
                            Command::RunTests => test_runner::run(&query_client).await,
                            // Other Dagger commands would go here
                            _ => Ok(()), // Should not happen if dispatch logic is correct
                        }
                    }
                    .await;
                    if let Err(e) = outcome {
                        tracing::error!(error = %e, "Dagger task failed");
                        let eyre_error = eyre!(e);
                        return Err(eyre_error.into());
                    }
                    Ok(())
                }
            })
            .await?;
        }
        Command::Fs(args) => {
            // Example: Handling Fs if it was non-Dagger
            match &args.command {
                FsCommands::RmRf { path } => fs_ops::rmrf(path)?,
                FsCommands::MkdirP { path } => fs_ops::mkdirp(path)?,
                FsCommands::FindDelete { directory, pattern } => {
                    fs_ops::find_delete(directory, pattern)?
                }
                FsCommands::CountFiles { directory, pattern } => {
                    fs_ops::count_files(directory, pattern)?
                }
                FsCommands::Cp { source, destination } => fs_ops::cp(source, destination)?,
            }
        }
        Command::Wasm(args) => {
            // Example: Handling Wasm if it was non-Dagger
            match &args.command {
                WasmCommands::Build { directory } => wasm_ops::build_all_wat(directory)?,
                WasmCommands::Check { directory } => wasm_ops::check_all_wat(directory)?,
                WasmCommands::Convert { wat_file } => {
                    let wasm_file = wasm_ops::wat_to_wasm_path(wat_file)?;
                    wasm_ops::convert_wat(wat_file, &wasm_file, false)?;
                }
            }
        }
        // Command::RunTests => test_runner::run_all_tests(&sh)?, // Removed old call
        // GenerateSourceNeeds already handled
        _ => {}
    }

    Ok(())
}
