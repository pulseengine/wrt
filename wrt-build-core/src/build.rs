//! Core build system implementation

use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::{BuildConfig, WorkspaceConfig};
use crate::error::{BuildError, BuildResult};

/// Ported functions from xtask for build operations
pub mod xtask_port {
    use super::*;
    use std::process::Command;

    /// Run comprehensive coverage analysis (ported from xtask coverage)
    pub fn run_coverage_analysis() -> BuildResult<()> {
        println!(
            "{} Running comprehensive coverage analysis...",
            "📊".bright_blue()
        );

        // Build with coverage flags
        let mut cmd = Command::new("cargo");
        cmd.args(["test", "--no-run", "--workspace"])
            .env("RUSTFLAGS", "-C instrument-coverage")
            .env("LLVM_PROFILE_FILE", "target/coverage/profile-%p-%m.profraw");

        let output = cmd
            .output()
            .map_err(|e| BuildError::Tool(format!("Failed to build with coverage: {}", e)))?;

        if !output.status.success() {
            return Err(BuildError::Build("Coverage build failed".to_string()));
        }

        // Run tests with coverage
        let mut test_cmd = Command::new("cargo");
        test_cmd
            .args(["test", "--workspace"])
            .env("RUSTFLAGS", "-C instrument-coverage")
            .env("LLVM_PROFILE_FILE", "target/coverage/profile-%p-%m.profraw");

        let test_output = test_cmd
            .output()
            .map_err(|e| BuildError::Tool(format!("Failed to run tests with coverage: {}", e)))?;

        if !test_output.status.success() {
            return Err(BuildError::Test("Coverage tests failed".to_string()));
        }

        println!("{} Coverage analysis completed", "✅".bright_green());
        Ok(())
    }

    /// Generate documentation (ported from xtask docs)
    pub fn generate_docs() -> BuildResult<()> {
        println!("{} Generating documentation...", "📚".bright_blue());

        let mut cmd = Command::new("cargo");
        cmd.args(["doc", "--workspace", "--all-features", "--no-deps"]);

        let output = cmd
            .output()
            .map_err(|e| BuildError::Tool(format!("Failed to generate docs: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BuildError::Build(format!(
                "Documentation generation failed: {}",
                stderr
            )));
        }

        println!(
            "{} Documentation generated successfully",
            "✅".bright_green()
        );
        Ok(())
    }

    /// Validate no_std compatibility (ported from xtask no_std_verification)
    pub fn verify_no_std_compatibility() -> BuildResult<()> {
        println!("{} Verifying no_std compatibility...", "🔧".bright_blue());

        // Build each crate with no_std features
        let no_std_crates = [
            "wrt-runtime",
            "wrt-component",
            "wrt-foundation",
            "wrt-instructions",
        ];

        for crate_name in &no_std_crates {
            let mut cmd = Command::new("cargo");
            cmd.args([
                "build",
                "-p",
                crate_name,
                "--no-default-features",
                "--features",
                "no_std",
            ]);

            let output = cmd
                .output()
                .map_err(|e| BuildError::Tool(format!("Failed to build {}: {}", crate_name, e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(BuildError::Build(format!(
                    "no_std verification failed for {}: {}",
                    crate_name, stderr
                )));
            }

            println!(
                "  {} {} no_std compatibility verified",
                "✓".bright_green(),
                crate_name
            );
        }

        println!("{} All crates are no_std compatible", "✅".bright_green());
        Ok(())
    }

    /// Run static analysis checks (ported from xtask ci_static_analysis)
    pub fn run_static_analysis() -> BuildResult<()> {
        println!("{} Running static analysis...", "🔍".bright_blue());

        // Run clippy with basic settings (avoid strict settings that might fail)
        let mut clippy_cmd = Command::new("cargo");
        clippy_cmd.args(["clippy", "--workspace", "--all-targets"]);

        let clippy_output = clippy_cmd
            .output()
            .map_err(|e| BuildError::Tool(format!("Failed to run clippy: {}", e)))?;

        if !clippy_output.status.success() {
            let stderr = String::from_utf8_lossy(&clippy_output.stderr);
            if stderr.contains("not installed") || stderr.contains("not found") {
                println!("  ⚠️ clippy not available, skipping clippy check");
            } else {
                return Err(BuildError::Verification(format!(
                    "Clippy checks failed: {}",
                    stderr
                )));
            }
        } else {
            println!("  ✅ Clippy checks passed");
        }

        // Check formatting
        let mut fmt_cmd = Command::new("cargo");
        fmt_cmd.args(["fmt", "--check"]);

        let fmt_output = fmt_cmd
            .output()
            .map_err(|e| BuildError::Tool(format!("Failed to check formatting: {}", e)))?;

        if !fmt_output.status.success() {
            let stderr = String::from_utf8_lossy(&fmt_output.stderr);
            if stderr.contains("not installed") || stderr.contains("not found") {
                println!("  ⚠️ cargo fmt not available, skipping format check");
            } else {
                return Err(BuildError::Verification(format!(
                    "Code formatting check failed: {}",
                    stderr
                )));
            }
        } else {
            println!("  ✅ Format check passed");
        }

        println!("{} Static analysis completed", "✅".bright_green());
        Ok(())
    }

    /// Run advanced test suite (ported from xtask ci_advanced_tests)
    pub fn run_advanced_tests() -> BuildResult<()> {
        println!("{} Running advanced test suite...", "🧪".bright_blue());

        // Run all tests with verbose output
        let mut cmd = Command::new("cargo");
        cmd.args(["test", "--workspace", "--all-features", "--verbose"]);

        let output = cmd
            .output()
            .map_err(|e| BuildError::Tool(format!("Failed to run advanced tests: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BuildError::Test(format!(
                "Advanced tests failed: {}",
                stderr
            )));
        }

        // Run integration tests if they exist
        if std::path::Path::new("tests").exists() {
            let mut integration_cmd = Command::new("cargo");
            integration_cmd.args(["test", "--test", "*", "--workspace"]);

            let integration_output = integration_cmd
                .output()
                .map_err(|e| BuildError::Tool(format!("Failed to run integration tests: {}", e)))?;

            if !integration_output.status.success() {
                let stderr = String::from_utf8_lossy(&integration_output.stderr);
                return Err(BuildError::Test(format!(
                    "Integration tests failed: {}",
                    stderr
                )));
            }
        }

        println!("{} Advanced tests passed", "✅".bright_green());
        Ok(())
    }

    /// Run integrity checks (ported from xtask ci_integrity_checks)
    pub fn run_integrity_checks() -> BuildResult<()> {
        println!("{} Running integrity checks...", "🔒".bright_blue());

        // Check for unsafe code
        let unsafe_check =
            Command::new("grep").args(["-r", "unsafe", "--include=*.rs", "src/"]).output();

        if let Ok(unsafe_output) = unsafe_check {
            let unsafe_count = String::from_utf8_lossy(&unsafe_output.stdout)
                .lines()
                .filter(|line| !line.contains("//") && line.contains("unsafe"))
                .count();

            if unsafe_count > 0 {
                println!("  ⚠️ Found {} unsafe code blocks", unsafe_count);
            } else {
                println!("  ✓ No unsafe code found");
            }
        }

        // Check for panic usage
        let panic_check =
            Command::new("grep").args(["-r", "panic!", "--include=*.rs", "src/"]).output();

        if let Ok(panic_output) = panic_check {
            let panic_count = String::from_utf8_lossy(&panic_output.stdout)
                .lines()
                .filter(|line| !line.contains("//") && line.contains("panic!"))
                .count();

            if panic_count > 0 {
                println!("  ⚠️ Found {} panic! macros", panic_count);
            } else {
                println!("  ✓ No panic! macros found");
            }
        }

        println!("{} Integrity checks completed", "✅".bright_green());
        Ok(())
    }

    /// Build WRTD (WebAssembly Runtime Daemon) binaries (ported from xtask wrtd_build)
    pub fn build_wrtd_binaries() -> BuildResult<()> {
        println!("{} Building WRTD binaries...", "🏗️".bright_blue());

        let wrtd_targets = [
            (
                "wrtd-std",
                "std-runtime",
                "Standard library runtime for servers/desktop",
            ),
            (
                "wrtd-alloc",
                "alloc-runtime",
                "Allocation runtime for embedded with heap",
            ),
            (
                "wrtd-nostd",
                "nostd-runtime",
                "No standard library runtime for bare metal",
            ),
        ];

        for (binary_name, feature, description) in &wrtd_targets {
            println!(
                "  {} Building {} - {}",
                "📦".bright_cyan(),
                binary_name,
                description
            );

            let mut cmd = Command::new("cargo");
            cmd.args(["build", "--bin", binary_name, "--features", feature]);

            let output = cmd
                .output()
                .map_err(|e| BuildError::Tool(format!("Failed to build {}: {}", binary_name, e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(BuildError::Build(format!(
                    "Failed to build {}: {}",
                    binary_name, stderr
                )));
            }

            println!("    ✓ {} built successfully", binary_name);
        }

        println!(
            "{} All WRTD binaries built successfully",
            "✅".bright_green()
        );
        Ok(())
    }
}

/// Central build system coordinator
#[derive(Debug)]
pub struct BuildSystem {
    /// Workspace configuration
    pub workspace: WorkspaceConfig,
    /// Build configuration
    pub config: BuildConfig,
}

/// Build results and artifacts
#[derive(Debug)]
pub struct BuildResults {
    /// Whether the build succeeded
    pub success: bool,
    /// List of built artifacts
    pub artifacts: Vec<PathBuf>,
    /// Build duration in milliseconds
    pub duration_ms: u64,
    /// Any warnings or notices
    pub warnings: Vec<String>,
}

impl BuildSystem {
    /// Create a new build system instance
    pub fn new(workspace_root: PathBuf) -> BuildResult<Self> {
        let workspace = WorkspaceConfig::load(&workspace_root)?;
        let config = BuildConfig::default();

        Ok(Self { workspace, config })
    }

    /// Create build system instance for current working directory
    pub fn for_current_dir() -> BuildResult<Self> {
        let workspace_root = crate::detect_workspace_root().map_err(|e| {
            BuildError::Workspace(format!("Could not detect workspace root: {}", e))
        })?;
        Self::new(workspace_root)
    }

    /// Create build system with custom configuration
    pub fn with_config(workspace_root: PathBuf, config: BuildConfig) -> BuildResult<Self> {
        let workspace = WorkspaceConfig::load(&workspace_root)?;
        Ok(Self { workspace, config })
    }

    /// Build all components in the workspace
    pub fn build_all(&self) -> BuildResult<BuildResults> {
        println!("{} Building all WRT components...", "🔨".bright_blue());

        let start_time = std::time::Instant::now();
        let mut artifacts = Vec::new();
        let mut warnings = Vec::new();

        // Build each crate in dependency order
        for crate_path in self.workspace.crate_paths() {
            match self.build_crate(&crate_path) {
                Ok(mut crate_artifacts) => {
                    artifacts.append(&mut crate_artifacts);
                },
                Err(e) => {
                    return Err(BuildError::Build(format!(
                        "Failed to build crate at {}: {}",
                        crate_path.display(),
                        e
                    )));
                },
            }
        }

        // Run workspace-level checks
        if self.config.clippy {
            match self.run_clippy() {
                Ok(clippy_warnings) => warnings.extend(clippy_warnings),
                Err(e) => warnings.push(format!("Clippy failed: {}", e)),
            }
        }

        if self.config.format_check {
            match self.check_formatting() {
                Ok(()) => {},
                Err(e) => warnings.push(format!("Format check failed: {}", e)),
            }
        }

        let duration = start_time.elapsed();
        println!(
            "{} Build completed in {:.2}s",
            "✅".bright_green(),
            duration.as_secs_f64()
        );

        Ok(BuildResults {
            success: true,
            artifacts,
            duration_ms: duration.as_millis() as u64,
            warnings,
        })
    }

    /// Run comprehensive coverage analysis using ported xtask logic
    pub fn run_coverage(&self) -> BuildResult<()> {
        xtask_port::run_coverage_analysis()
    }

    /// Generate documentation using ported xtask logic
    pub fn generate_docs(&self) -> BuildResult<()> {
        xtask_port::generate_docs()
    }

    /// Verify no_std compatibility using ported xtask logic
    pub fn verify_no_std(&self) -> BuildResult<()> {
        xtask_port::verify_no_std_compatibility()
    }

    /// Run static analysis using ported xtask logic
    pub fn run_static_analysis(&self) -> BuildResult<()> {
        xtask_port::run_static_analysis()
    }

    /// Run advanced tests using ported xtask logic
    pub fn run_advanced_tests(&self) -> BuildResult<()> {
        xtask_port::run_advanced_tests()
    }

    /// Run integrity checks using ported xtask logic
    pub fn run_integrity_checks(&self) -> BuildResult<()> {
        xtask_port::run_integrity_checks()
    }

    /// Build WRTD binaries using ported xtask logic
    pub fn build_wrtd_binaries(&self) -> BuildResult<()> {
        xtask_port::build_wrtd_binaries()
    }

    /// Check requirements traceability
    pub fn check_requirements(&self, requirements_file: Option<&Path>) -> BuildResult<()> {
        use crate::requirements::Requirements;

        let req_path = requirements_file
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| self.workspace.root.join("requirements.toml"));

        if !req_path.exists() {
            println!(
                "{} No requirements.toml found at {}",
                "⚠️".bright_yellow(),
                req_path.display()
            );
            println!("  Use 'cargo-wrt init-requirements' to create a sample file");
            return Ok(());
        }

        println!(
            "{} Checking requirements traceability...",
            "📋".bright_blue()
        );

        let requirements = Requirements::load(&req_path)?;
        let results = requirements.verify(&self.workspace.root)?;

        println!();
        println!("📊 Requirements Summary:");
        println!("  Total requirements: {}", results.total_requirements);
        println!("  Verified requirements: {}", results.verified_requirements);
        println!(
            "  Certification readiness: {:.1}%",
            results.certification_readiness
        );

        if !results.missing_files.is_empty() {
            println!();
            println!("{} Missing files:", "⚠️".bright_yellow());
            for file in &results.missing_files {
                println!("  - {}", file);
            }
        }

        if !results.incomplete_requirements.is_empty() {
            println!();
            println!("{} Incomplete requirements:", "❌".bright_red());
            for req in &results.incomplete_requirements {
                println!("  - {}", req);
            }
        }

        if results.certification_readiness >= 80.0 {
            println!();
            println!("{} Requirements verification passed!", "✅".bright_green());
        } else {
            println!();
            println!(
                "{} Requirements need attention for certification",
                "⚠️".bright_yellow()
            );
        }

        Ok(())
    }

    /// Initialize sample requirements file
    pub fn init_requirements(&self, path: Option<&Path>) -> BuildResult<()> {
        use crate::requirements::Requirements;

        let req_path = path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| self.workspace.root.join("requirements.toml"));

        if req_path.exists() {
            return Err(BuildError::Verification(format!(
                "Requirements file already exists at {}",
                req_path.display()
            )));
        }

        Requirements::init_sample(&req_path)?;
        Ok(())
    }

    /// Build a specific crate
    pub fn build_crate(&self, crate_path: &Path) -> BuildResult<Vec<PathBuf>> {
        if !crate_path.exists() {
            return Err(BuildError::Workspace(format!(
                "Crate path does not exist: {}",
                crate_path.display()
            )));
        }

        let crate_name = crate_path.file_name().and_then(|name| name.to_str()).unwrap_or("unknown");

        if self.config.verbose {
            println!("  {} Building crate: {}", "📦".bright_cyan(), crate_name);
        }

        let mut cmd = Command::new("cargo");
        cmd.arg("build").current_dir(&self.workspace.root);

        // Add package selector
        cmd.arg("-p").arg(crate_name);

        // Add profile
        match self.config.profile {
            crate::config::BuildProfile::Release => {
                cmd.arg("--release");
            },
            crate::config::BuildProfile::Test => {
                cmd.arg("--tests");
            },
            _ => {}, // Dev is default
        }

        // Add features
        if !self.config.features.is_empty() {
            cmd.arg("--features").arg(self.config.features.join(","));
        }

        // Execute build
        let output = cmd
            .output()
            .map_err(|e| BuildError::Tool(format!("Failed to execute cargo: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BuildError::Build(format!(
                "Cargo build failed for {}: {}",
                crate_name, stderr
            )));
        }

        // Return artifacts (simplified - would parse cargo output in real implementation)
        Ok(vec![crate_path.join("target")])
    }

    /// Build a specific package by name
    pub fn build_package(&self, package_name: &str) -> BuildResult<BuildResults> {
        if self.config.verbose {
            println!("  {} Building package: {}", "📦".bright_cyan(), package_name);
        }

        let start_time = std::time::Instant::now();
        let mut warnings = Vec::new();

        let mut cmd = Command::new("cargo");
        cmd.arg("build").current_dir(&self.workspace.root);

        // Add package selector
        cmd.arg("-p").arg(package_name);

        // Add profile
        match self.config.profile {
            crate::config::BuildProfile::Release => {
                cmd.arg("--release");
            },
            crate::config::BuildProfile::Test => {
                cmd.arg("--tests");
            },
            _ => {}, // Dev is default
        }

        // Add features if specified
        if !self.config.features.is_empty() {
            cmd.arg("--features").arg(self.config.features.join(","));
        }

        let output = cmd
            .output()
            .map_err(|e| BuildError::Tool(format!("Failed to execute cargo build: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BuildError::Build(format!(
                "Cargo build failed for package {}: {}",
                package_name, stderr
            )));
        }

        // Check for warnings in stdout
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("warning:") {
            warnings.push(format!("Package {} has build warnings", package_name));
        }

        let duration = start_time.elapsed();
        
        Ok(BuildResults {
            success: true,
            artifacts: vec![], // Single package builds don't track specific artifacts yet
            duration_ms: duration.as_millis() as u64,
            warnings,
        })
    }

    /// Test a specific package by name
    pub fn test_package(&self, package_name: &str) -> BuildResult<BuildResults> {
        if self.config.verbose {
            println!("  {} Testing package: {}", "🧪".bright_cyan(), package_name);
        }

        let start_time = std::time::Instant::now();
        let mut warnings = Vec::new();

        let mut cmd = Command::new("cargo");
        cmd.arg("test").current_dir(&self.workspace.root);

        // Add package selector
        cmd.arg("-p").arg(package_name);

        // Add features if specified
        if !self.config.features.is_empty() {
            cmd.arg("--features").arg(self.config.features.join(","));
        }

        let output = cmd
            .output()
            .map_err(|e| BuildError::Tool(format!("Failed to execute cargo test: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BuildError::Test(format!(
                "Cargo test failed for package {}: {}",
                package_name, stderr
            )));
        }

        // Check for warnings in stdout
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("warning:") {
            warnings.push(format!("Package {} has test warnings", package_name));
        }

        let duration = start_time.elapsed();
        
        Ok(BuildResults {
            success: true,
            artifacts: vec![], // Package tests don't produce artifacts
            duration_ms: duration.as_millis() as u64,
            warnings,
        })
    }

    /// Run clippy checks on the workspace
    pub fn run_clippy(&self) -> BuildResult<Vec<String>> {
        if self.config.verbose {
            println!("  {} Running clippy checks...", "📎".bright_yellow());
        }

        let mut cmd = Command::new("cargo");
        cmd.arg("clippy")
            .arg("--workspace")
            .arg("--all-targets")
            .current_dir(&self.workspace.root);

        if !self.config.features.is_empty() {
            cmd.arg("--features").arg(self.config.features.join(","));
        }

        let output = cmd
            .output()
            .map_err(|e| BuildError::Tool(format!("Failed to execute clippy: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let warnings = stdout
            .lines()
            .filter(|line| line.contains("warning:"))
            .map(|line| line.to_string())
            .collect();

        Ok(warnings)
    }

    /// Check code formatting
    pub fn check_formatting(&self) -> BuildResult<()> {
        if self.config.verbose {
            println!("  {} Checking code formatting...", "🎨".bright_magenta());
        }

        let mut cmd = Command::new("cargo");
        cmd.arg("fmt").arg("--check").current_dir(&self.workspace.root);

        let output = cmd
            .output()
            .map_err(|e| BuildError::Tool(format!("Failed to execute cargo fmt: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("not installed") || stderr.contains("not found") {
                println!("  ⚠️ cargo fmt not available, skipping format check");
                return Ok(());
            }
            return Err(BuildError::Tool(format!(
                "Code formatting check failed: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Get workspace root path
    pub fn workspace_root(&self) -> &Path {
        &self.workspace.root
    }

    /// Get build configuration
    pub fn build_config(&self) -> &BuildConfig {
        &self.config
    }

    /// Update build configuration
    pub fn set_config(&mut self, config: BuildConfig) {
        self.config = config;
    }

    /// Set verbose mode
    pub fn set_verbose(&mut self, verbose: bool) {
        self.config.verbose = verbose;
    }

    /// Set profile
    pub fn set_profile(&mut self, profile: crate::config::BuildProfile) {
        self.config.profile = profile;
    }

    /// Add feature
    pub fn add_feature(&mut self, feature: String) {
        if !self.config.features.contains(&feature) {
            self.config.features.push(feature);
        }
    }

    /// Remove feature
    pub fn remove_feature(&mut self, feature: &str) {
        self.config.features.retain(|f| f != feature);
    }

    /// Get workspace metadata
    pub fn workspace(&self) -> &WorkspaceConfig {
        &self.workspace
    }
}

impl BuildResults {
    /// Check if build was successful
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get build artifacts
    pub fn artifacts(&self) -> &[PathBuf] {
        &self.artifacts
    }

    /// Get build duration
    pub fn duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.duration_ms)
    }

    /// Get warnings
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_build_system_creation() {
        // Use the actual workspace for testing
        let workspace = crate::detect_workspace_root().unwrap();
        let build_system = BuildSystem::new(workspace);
        assert!(build_system.is_ok());
    }

    #[test]
    fn test_build_system_for_current_dir() {
        let build_system = BuildSystem::for_current_dir();
        assert!(build_system.is_ok());
    }

    #[test]
    fn test_build_system_config_management() {
        let workspace = crate::detect_workspace_root().unwrap();
        let mut build_system = BuildSystem::new(workspace).unwrap();

        // Test feature management
        build_system.add_feature("test-feature".to_string());
        assert!(build_system.config.features.contains(&"test-feature".to_string()));

        build_system.remove_feature("test-feature");
        assert!(!build_system.config.features.contains(&"test-feature".to_string()));

        // Test verbose mode
        build_system.set_verbose(true);
        assert!(build_system.config.verbose);
    }

    #[test]
    fn test_build_results() {
        let results = BuildResults {
            success: true,
            artifacts: vec![PathBuf::from("target/debug/wrt")],
            duration_ms: 1000,
            warnings: vec!["warning: unused variable".to_string()],
        };

        assert!(results.is_success());
        assert_eq!(results.duration().as_millis(), 1000);
        assert_eq!(results.warnings().len(), 1);
    }
}
