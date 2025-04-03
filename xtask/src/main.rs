use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod check_imports;
mod fs_ops;
mod wasm_ops;
mod wast_tests;

#[derive(Parser, Debug)]
#[command(author, version, about = "Workspace tasks", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
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
        #[arg(default_value = "examples")]
        directory: PathBuf,
    },
    /// Check if WASM files are up-to-date with their WAT counterparts
    Check {
        #[arg(default_value = "examples")]
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

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::CheckImports { dir1, dir2 } => check_imports::run(&[&dir1, &dir2])?,
        Commands::Wasm(args) => match args.command {
            WasmCommands::Build { directory } => wasm_ops::build_all_wat(&directory)?,
            WasmCommands::Check { directory } => wasm_ops::check_all_wat(&directory)?,
            WasmCommands::Convert { wat_file } => {
                let wasm_file = wasm_ops::wat_to_wasm_path(&wat_file)?;
                wasm_ops::convert_wat(&wat_file, &wasm_file, false)?;
            }
        },
        Commands::Fs(args) => match args.command {
            FsCommands::RmRf { path } => fs_ops::rmrf(&path)?,
            FsCommands::MkdirP { path } => fs_ops::mkdirp(&path)?,
            FsCommands::FindDelete { directory, pattern } => {
                fs_ops::find_delete(&directory, &pattern)?
            }
            FsCommands::CountFiles { directory, pattern } => {
                fs_ops::count_files(&directory, &pattern)?
            }
            FsCommands::FileSize { path } => fs_ops::file_size(&path)?,
            FsCommands::Cp {
                source,
                destination,
            } => fs_ops::copy_file(&source, &destination)?,
        },
        Commands::RunWastTests {
            create_files,
            verify_passing,
        } => wast_tests::run(create_files, verify_passing)?,
    }

    Ok(())
}
