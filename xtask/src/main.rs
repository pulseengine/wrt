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
    CheckDocsStrict,
    FmtCheck,
    RunTests,
    GenerateSourceNeeds(generate_source_needs::GenerateSourceNeedsArgs),
    VerifyNoStd(VerifyNoStdArgs),
    PreviewDocs(PreviewDocsArgs),
    ValidateDocs,
    ValidateDocsComprehensive,
    GenerateChangelog(GenerateChangelogArgs),
    DeployDocsSftp(DeployDocsSftpArgs), /* Added new command
                                                                          * Comment out
                                                                          * commands whose
                                                                          * modules are missing
                                                                          * or commented out
                                                                          * Install(InstallArgs),
                                                                          * Lint(rust_ops::LintOpts), // rust_ops missing
                                                                          * Test(rust_ops::TestOpts),  // rust_ops missing
                                                                          * Build(rust_ops::BuildOpts), // rust_ops missing
                                                                          * Ci(ci_ops::CiArgs),
                                                                          * // ci_ops missing
                                                                          * UpdateManifest(manifest_ops::UpdateManifestArgs), // manifest_ops missing
                                                                          * Coverage(cobertura_ops::CoverageArgs), // cobertura_ops missing
                                                                          * CoverageClean(cobertura_ops::CoverageCleanArgs), // cobertura_ops missing
                                                                          * LicheDown(lichedown_ops::LicheDownArgs), // lichedown_ops missing
                                                                          * Apps(apps_ops::AppsArgs), // apps_ops missing */
}

// Args structs for existing commands
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

    let subscriber = FmtSubscriber::builder()
        .with_max_level(opts.log_level.parse::<Level>().unwrap_or(Level::INFO))
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set global default tracing subscriber")?;

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
