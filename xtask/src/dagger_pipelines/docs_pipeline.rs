// See: https://lib.rs/crates/dagger-sdk and https://github.com/dagger/dagger/blob/main/sdk/rust/crates/dagger-sdk/examples/first-pipeline/main.rs
use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Result as AnyhowResult};
use dagger_sdk::HostDirectoryOpts;
use tempfile::TempDir;
use tracing::{error, info, instrument, warn};

use crate::Query;

// const MDBOOK_IMAGE: &str = "localhost/obrunsm/mdbook-extended:latest";
// const NETLIFY_IMAGE: &str = "netlify/cli:latest";

// const OFFICIAL_MDBOOK_IMAGE: &str = "peaceiris/mdbook:latest";

#[instrument(name = "docs_pipeline", skip_all, err)]
pub async fn run_docs_pipeline(
    client: &Query,
    base_path: PathBuf,
    output_dir: PathBuf,
    versions: Vec<String>,
) -> AnyhowResult<()> {
    info!(
        "Starting docs pipeline. Base path: {:?}, Output dir: {:?}, Versions: {:?}",
        base_path, output_dir, versions
    );

    if versions.is_empty() {
        bail!("No versions specified for the documentation pipeline.");
    }

    anyhow::Context::context(
        fs::create_dir_all(&output_dir),
        format!("Failed to create output directory: {:?}", output_dir),
    )?;

    let mut build_futures = Vec::new();
    for (_i, version) in versions.iter().enumerate() {
        let client_clone = client.clone();
        let output_dir_clone = output_dir.clone();
        let base_path_clone = base_path.clone();
        let version_clone = version.clone();
        let is_main = version_clone == "main";

        build_futures.push(tokio::spawn(async move {
            run_docs_version_pipeline(
                &client_clone,
                &base_path_clone,
                &output_dir_clone,
                &version_clone,
                is_main,
            )
            .await
        }));
    }

    let mut all_successful = true;
    for future in build_futures {
        match future.await {
            Ok(Ok(_)) => { /* Version processed successfully */ }
            Ok(Err(e)) => {
                error!("A documentation version failed: {:?}", e);
                all_successful = false;
            }
            Err(e) => {
                error!("A documentation task panicked: {:?}", e);
                all_successful = false;
            }
        }
    }

    if !all_successful {
        bail!("One or more documentation versions failed to build/export. See logs for details.");
    }

    // After all versions are successfully built, copy the root_index.html
    let root_index_src_path = base_path.join("docs").join("source").join("root_index.html");
    let root_index_dest_path = output_dir.join("index.html");
    if root_index_src_path.exists() {
        anyhow::Context::context(
            fs::copy(&root_index_src_path, &root_index_dest_path),
            format!(
                "Failed to copy root_index.html from {:?} to {:?}",
                root_index_src_path, root_index_dest_path
            ),
        )?;
        info!("Copied root redirect index.html to {:?}", root_index_dest_path);
    } else {
        warn!("Root redirect index.html not found at {:?}, skipping copy.", root_index_src_path);
    }

    // Generate switcher.json
    // The `version_path_prefix` in conf.py is used by the theme to *construct* the
    // full URL to switcher.json. The `version` field within switcher.json
    // entries should be relative paths from where switcher.json is located. So,
    // if switcher.json is at the root of docs_artifact_final, then "./main/",
    // "./v0.1.0/" are correct.
    if let Err(e) = generate_switcher_json(&output_dir, &versions, "./") {
        warn!(
            "Failed to generate switcher.json: {:?}. Version switcher in docs might not work \
             correctly.",
            e
        );
        // Decide if this should be a hard error: bail!(e.context("Failed to
        // generate switcher.json"));
    }

    info!("All documentation versions built and exported successfully.");
    Ok(())
}

// Helper function to generate switcher.json
#[derive(serde::Serialize)]
struct SwitcherEntry {
    name: String,
    version: String, // This will be the plain version string, e.g., "local", "main"
    url: String,     // This will be the absolute URL from server root, e.g., "/local/", "/main/"
}

fn generate_switcher_json(
    output_dir: &Path,
    built_versions: &[String],
    _base_url_prefix: &str, /* No longer used for constructing the URL path itself, but kept for
                             * signature stability if needed elsewhere. */
) -> AnyhowResult<()> {
    if built_versions.is_empty() {
        info!("No versions provided to generate switcher.json, skipping.");
        return Ok(());
    }

    // Identify the latest semver tag to mark as stable
    let mut latest_semver_tag: Option<semver::Version> = None;
    for v_str in built_versions {
        if v_str.starts_with('v') {
            if let Ok(parsed_ver) = semver::Version::parse(&v_str[1..]) {
                if latest_semver_tag.as_ref().map_or(true, |latest| parsed_ver > *latest) {
                    latest_semver_tag = Some(parsed_ver);
                }
            }
        }
    }
    let latest_semver_tag_str = latest_semver_tag.as_ref().map(|v| format!("v{}", v));

    let mut entries = Vec::new();
    for version_str in built_versions {
        let name = if version_str == "local" {
            // Handle "local" version
            "Local (uncommitted)".to_string()
        } else if version_str == "main" {
            "main (development)".to_string()
        } else if Some(version_str.clone()) == latest_semver_tag_str {
            format!("{} (stable)", version_str)
        } else {
            version_str.clone()
        };

        let plain_version = version_str.trim_end_matches('/').to_string();
        // Construct the absolute URL path from server root, ensuring it ends with a
        // slash.
        let absolute_url = format!("/{}/", plain_version);

        entries.push(SwitcherEntry { name, version: plain_version, url: absolute_url });
    }

    // Sort entries: "Local" first, then "main", then semver tags in descending
    // order, then others alphabetically.
    entries.sort_by(|a, b| {
        let a_is_local = a.name.starts_with("Local");
        let b_is_local = b.name.starts_with("Local");
        let a_is_main = a.name.starts_with("main");
        let b_is_main = b.name.starts_with("main");

        if a_is_local && !b_is_local {
            return std::cmp::Ordering::Less;
        } // Local first
        if !a_is_local && b_is_local {
            return std::cmp::Ordering::Greater;
        }

        if a_is_main && !b_is_main {
            return std::cmp::Ordering::Less;
        } // Then main
        if !a_is_main && b_is_main {
            return std::cmp::Ordering::Greater;
        }

        // Extract version string for semver parsing (e.g., from "v1.2.3 (stable)" ->
        // "1.2.3")
        let re = regex::Regex::new(r"v?(\d+\.\d+\.\d+)").unwrap(); // Simple regex for semver part

        let a_semver_str = re.captures(&a.name).and_then(|caps| caps.get(1)).map(|m| m.as_str());
        let b_semver_str = re.captures(&b.name).and_then(|caps| caps.get(1)).map(|m| m.as_str());

        if let (Some(a_sv_str), Some(b_sv_str)) = (a_semver_str, b_semver_str) {
            if let (Ok(a_ver), Ok(b_ver)) =
                (semver::Version::parse(a_sv_str), semver::Version::parse(b_sv_str))
            {
                return b_ver.cmp(&a_ver); // Descending for versions
            }
        }
        a.name.cmp(&b.name) // Fallback to alphabetical
    });

    let switcher_file_path = output_dir.join("switcher.json");
    let file = anyhow::Context::context(
        fs::File::create(&switcher_file_path),
        format!("Failed to create switcher.json at {:?}", switcher_file_path),
    )?;
    anyhow::Context::context(
        serde_json::to_writer_pretty(file, &entries),
        "Failed to write switcher.json",
    )?;

    info!("Generated switcher.json at {:?}", switcher_file_path);
    Ok(())
}

#[instrument(name = "docs_version_pipeline", skip_all, fields(version = % version), err)]
async fn run_docs_version_pipeline(
    client: &Query,
    base_path: &Path,
    output_dir: &Path,
    version: &str,
    _is_main_branch: bool,
) -> AnyhowResult<()> {
    info!("Running docs pipeline for version: {}", version);
    let version_docs_output_path = output_dir.join(version);
    anyhow::Context::context(
        fs::create_dir_all(&version_docs_output_path),
        format!("Failed to create version output directory for {}", version),
    )?;

    let docs_src_host_path_str: String;
    // This guard will ensure TempDir is cleaned up when it goes out of scope if a
    // worktree was created.
    let _worktree_temp_dir_guard: Option<TempDir> = if version == "local" {
        let local_docs_dir = base_path.join("docs");
        info!("Using local docs directory for version 'local' from: {:?}", local_docs_dir);
        docs_src_host_path_str = local_docs_dir
            .to_str()
            .ok_or_else(|| anyhow!("Local docs path is not valid UTF-8: {:?}", local_docs_dir))?
            .to_string();
        None // No TempDir needed for "local" version
    } else {
        // Existing git worktree logic for actual git versions (main, tags, etc.)
        let temp_dir = anyhow::Context::context(
            TempDir::new(),
            "Failed to create temporary directory for git worktree",
        )?;
        info!("Created temporary worktree at: {:?} for version: {}", temp_dir.path(), version);
        let worktree_path_str = temp_dir.path().to_str().ok_or_else(|| {
            anyhow!("Temporary worktree path is not valid UTF-8: {:?}", temp_dir.path())
        })?;

        // Verify the version exists as a git ref before attempting to create a worktree
        info!("Verifying git version/ref: {}", version);
        let verify_version_cmd_args = ["rev-parse", "--verify", version];
        let verify_output = anyhow::Context::context(
            std::process::Command::new("git").args(&verify_version_cmd_args).output(),
            format!("Failed to execute git rev-parse command for version {}", version),
        )?;

        if !verify_output.status.success() {
            let stderr = String::from_utf8_lossy(&verify_output.stderr);
            error!(
                "Git rev-parse command failed for version {}: {}\\nStdout: {}\\nStderr: {}",
                version,
                verify_output.status,
                String::from_utf8_lossy(&verify_output.stdout),
                stderr
            );
            bail!(
                "Git rev-parse --verify failed for version: {}. Is it a valid git reference \
                 (branch, tag, commit hash)? Stderr: {}",
                version,
                stderr
            );
        }
        info!("Successfully verified git version/ref: {}", version);

        let git_checkout_cmd_args = ["worktree", "add", "--detach", worktree_path_str, version];

        info!("Running git command: git {:?}", git_checkout_cmd_args);
        let checkout_output = anyhow::Context::context(
            std::process::Command::new("git").args(&git_checkout_cmd_args).output(),
            format!("Failed to execute git worktree command for version {}", version),
        )?;

        if !checkout_output.status.success() {
            let stderr = String::from_utf8_lossy(&checkout_output.stderr);
            error!(
                "Git worktree command failed for version {}: {}\\nStdout: {}\\nStderr: {}",
                version,
                checkout_output.status,
                String::from_utf8_lossy(&checkout_output.stdout),
                stderr
            );
            bail!(
                "Git worktree add command failed with status {} for version: {}. Stderr: {}",
                checkout_output.status,
                version,
                stderr
            );
        }
        info!("Successfully checked out version {} to {:?}", version, temp_dir.path());

        let worktree_docs_dir = temp_dir.path().join("docs");
        docs_src_host_path_str = worktree_docs_dir
            .to_str()
            .ok_or_else(|| {
                anyhow!("Worktree docs path is not valid UTF-8: {:?}", worktree_docs_dir)
            })?
            .to_string();
        Some(temp_dir) // temp_dir is now owned by the guard and will be cleaned
                       // up on drop.
    };

    // Generate coverage summary before building docs
    {
        let docs_path = Path::new(&docs_src_host_path_str);
        let coverage_json_path = docs_path
            .parent()
            .map(|p| p.join("target/coverage/coverage.json"))
            .unwrap_or_else(|| PathBuf::from("target/coverage/coverage.json"));
        let coverage_summary_path = docs_path.join("source/_generated_coverage_summary.rst");

        if coverage_json_path.exists() {
            info!("Generating coverage summary from {:?}", coverage_json_path);
            if let Err(e) = crate::generate_coverage_summary::generate_coverage_summary_rst(
                &coverage_json_path,
                &coverage_summary_path,
            ) {
                warn!("Failed to generate coverage summary: {}", e);
                // Generate placeholder instead
                let _ = crate::generate_coverage_summary::generate_placeholder_coverage_summary(
                    &coverage_summary_path,
                );
            }
        } else {
            info!("No coverage data found, generating placeholder");
            let _ = crate::generate_coverage_summary::generate_placeholder_coverage_summary(
                &coverage_summary_path,
            );
        }
    }

    // Generate changelog using git cliff
    {
        let docs_path = Path::new(&docs_src_host_path_str);
        let changelog_path = docs_path.join("source/changelog.md");

        info!("Generating changelog for version: {}", version);

        // Generate changelog in the main repository, not in the worktree
        let temp_changelog = base_path.join(format!("changelog_{}.md", version.replace("/", "_")));

        // Determine git cliff arguments based on version
        let cliff_args = if version == "local" {
            // For local builds, generate unreleased changes
            vec!["cliff", "--unreleased", "--output", temp_changelog.to_str().unwrap()]
        } else {
            // For any other version, generate the full changelog
            // Git cliff will include all commits up to HEAD in the main repo
            vec!["cliff", "--output", temp_changelog.to_str().unwrap()]
        };

        // Run git cliff in the main repository
        let cliff_output =
            std::process::Command::new("git").args(&cliff_args).current_dir(base_path).output();

        match cliff_output {
            Ok(output) => {
                if !output.status.success() {
                    warn!("git cliff failed: {}", String::from_utf8_lossy(&output.stderr));
                    // Create a minimal changelog if git cliff fails
                    let fallback_content = format!(
                        "# Changelog\n\n## Version: {}\n\nChangelog generation failed. Please \
                         check git cliff configuration.\n",
                        version
                    );
                    if let Err(e) = fs::write(&changelog_path, fallback_content) {
                        warn!("Failed to write fallback changelog: {}", e);
                    }
                } else {
                    info!("Successfully generated changelog for version {}", version);
                    // Copy the generated changelog to the docs directory
                    if temp_changelog.exists() {
                        if let Err(e) = fs::copy(&temp_changelog, &changelog_path) {
                            warn!("Failed to copy changelog to docs: {}", e);
                        } else {
                            // Clean up temp file
                            let _ = fs::remove_file(&temp_changelog);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to run git cliff: {}", e);
                // Create a minimal changelog if git cliff is not available
                let fallback_content = format!(
                    "# Changelog\n\n## Version: {}\n\ngit cliff not available. Install with: \
                     `cargo install git-cliff`\n",
                    version
                );
                if let Err(e) = fs::write(&temp_changelog, fallback_content) {
                    warn!("Failed to write fallback changelog: {}", e);
                } else {
                    // Copy to docs directory
                    if let Err(e) = fs::copy(&temp_changelog, &changelog_path) {
                        warn!("Failed to copy fallback changelog to docs: {}", e);
                    }
                    let _ = fs::remove_file(&temp_changelog);
                }
            }
        }
    }

    // Dagger directory for WORKTREE/docs or local/docs
    let docs_dagger_dir = client.host().directory_opts(
        &docs_src_host_path_str, // This is now correctly sourced based on 'version'
        HostDirectoryOpts { exclude: None, include: None }, /* This will include both 'source'
                                  * and 'requirements.txt' */
    );

    let docs_container = client
        .container()
        .from("sphinxdoc/sphinx:latest")
        // Mount WORKTREE/docs to /mounted_docs first, then set workdir
        .with_mounted_directory("/mounted_docs", docs_dagger_dir)
        .with_workdir("/mounted_docs/source")
        // Pass the current version to Sphinx so it can set version_match correctly
        .with_env_variable("DOCS_VERSION", version)
        // Set prefix to / for local serving from docs_artifact_final root. Adjust if serving from a
        // subpath like /wrt/.
        .with_env_variable("DOCS_VERSION_PATH_PREFIX", "/")
        // Install build-essential (for linker cc), curl, then rustup, source its env, and then run
        // pip install.
        .with_exec(vec![
            "sh",
            "-c",
            "\
            apt-get update && apt-get install -y build-essential curl default-jre graphviz && \
            curl -L -o /usr/local/bin/plantuml.jar https://github.com/plantuml/plantuml/releases/download/v1.2024.0/plantuml-1.2024.0.jar && \
            echo '#!/bin/sh\njava -jar /usr/local/bin/plantuml.jar \"$@\"' > /usr/local/bin/plantuml && \
            chmod +x /usr/local/bin/plantuml && \
            export CARGO_HOME=/root/.cargo && \
            export RUSTUP_HOME=/root/.rustup && \
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
            . $CARGO_HOME/env && \
            pip install -r ../requirements.txt",
        ])
        // Similarly, ensure PlantUML is available and run sphinx-build.
        .with_exec(vec![
            "sh",
            "-c",
            "\
            export CARGO_HOME=/root/.cargo && \
            export RUSTUP_HOME=/root/.rustup && \
            . $CARGO_HOME/env && \
            export PATH=/usr/local/bin:$PATH && \
            sphinx-build -vv -b html . ../_build/html",
        ]);

    let exit_code = anyhow::Context::context(
        docs_container.exit_code().await,
        format!("Failed to get Sphinx build exit code for version {}", version),
    )?;

    if exit_code != 0 {
        let stderr = docs_container
            .stderr()
            .await
            .unwrap_or_else(|e| format!("Failed to get Sphinx stderr: {:?}", e));
        error!(
            "Sphinx build failed for version {} with exit code {}. Stderr: {}",
            version, exit_code, stderr
        );
        bail!(
            "Sphinx build failed for version {} with exit code {}. Stderr: {}",
            version,
            exit_code,
            stderr
        );
    }
    info!("Sphinx build successful for version: {}", version);

    // The relative path "../_build/html" is from the workdir
    // "/mounted_docs/source", so it correctly points to
    // "/mounted_docs/_build/html" in the container.
    let built_docs_dir = docs_container.directory("../_build/html");
    let export_path_str = version_docs_output_path
        .to_str()
        .ok_or_else(|| anyhow!("Invalid version_docs_path: not valid UTF-8"))?;

    anyhow::Context::context(
        built_docs_dir.export(export_path_str).await,
        format!(
            "Failed to export generated docs from container for version {} to {}",
            version, export_path_str
        ),
    )?;
    info!("Exported docs for version {} to {}", version, export_path_str);

    // No explicit cleanup of _worktree_temp_dir_guard needed here as Drop will
    // handle it. The previous explicit .close() call for worktree_path is
    // removed. If detailed error reporting on cleanup is needed later, .close()
    // can be called on the Option<TempDir>.

    Ok(())
}

#[instrument(name = "check_docs_strict_pipeline", skip_all, err)]
pub async fn check_docs_strict_pipeline(client: &Query) -> AnyhowResult<()> {
    info!("Starting Daggerized strict documentation check pipeline...");

    // Get the project root directory, canonicalized
    let current_dir = std::env::current_dir()?;
    let host_dir_path = current_dir.canonicalize()?;
    let host_dir_path_str = host_dir_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to convert workspace root path to string"))?;

    let project_src_dir = client.host().directory_opts(
        host_dir_path_str, // Use canonicalized absolute path
        HostDirectoryOpts {
            exclude: Some(vec!["./target", "./.git", "./.cargo"]),
            include: Some(vec!["./wrt*", "./docs", "./Cargo.toml", "./Cargo.lock"]),
        },
    );

    let python_image = "python:3.11-slim";
    let python_container = client
        .container()
        .from(python_image)
        // Install build tools, then Rust/Cargo
        .with_exec(vec!["apt-get", "update"])
        .with_exec(vec!["apt-get", "install", "-y", "curl", "build-essential", "gcc"])
        // Ensure sh can find curl and gcc for the rustup script
        .with_env_variable("PATH", "/usr/bin:/usr/local/bin:$PATH")
        .with_exec(vec![
            "sh",
            "-c",
            "curl --proto =https --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
        ])
        // Add Cargo to PATH, ensure Python's bin and system bins are still there, and set CC
        .with_env_variable("PATH", "/root/.cargo/bin:/usr/local/bin:/usr/bin:$PATH")
        .with_env_variable("CC", "gcc")
        .with_env_variable("RUST_BACKTRACE", "1")
        .with_mounted_directory("/wrt", project_src_dir.id().await?)
        .with_workdir("/wrt")
        .with_exec(vec!["pip", "install", "-r", "docs/requirements.txt"])
        .with_exec(vec![
            "sphinx-build",
            "-E",
            "-a",
            "-b",
            "html",
            "docs/source",
            "docs/_build/html_strict_check",
        ]);

    let exit_code_result = python_container.exit_code().await;

    match exit_code_result {
        Ok(exit_code) => {
            if exit_code == 0 {
                info!("Sphinx strict check passed.");
                Ok(())
            } else {
                let stderr_result = python_container.stderr().await;
                match stderr_result {
                    Ok(stderr_output) => {
                        error!(
                            "Sphinx strict check failed with exit code {}. Stderr: {}",
                            exit_code, stderr_output
                        );
                        bail!(
                            "Sphinx strict check failed. Exit code: {}. Stderr: {}",
                            exit_code,
                            stderr_output
                        )
                    }
                    Err(e) => {
                        error!(
                            "Sphinx strict check failed with exit code {}. Additionally, failed \
                             to fetch stderr: {:?}",
                            exit_code, e
                        );
                        bail!(
                            "Sphinx strict check failed with exit code {}. Failed to fetch \
                             stderr: {:?}",
                            exit_code,
                            e
                        )
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to get exit code from Sphinx strict check container: {:?}", e);
            // Attempt to get stdout/stderr even if exit_code failed, as they might contain
            // clues
            let stdout_dbg = python_container
                .stdout()
                .await
                .unwrap_or_else(|dbg_e| format!("Failed to get stdout for debugging: {:?}", dbg_e));
            let stderr_dbg = python_container
                .stderr()
                .await
                .unwrap_or_else(|dbg_e| format!("Failed to get stderr for debugging: {:?}", dbg_e));
            error!(
                "Attempted to get container logs for debugging. Stdout: {}. Stderr: {}",
                stdout_dbg, stderr_dbg
            );
            // Use anyhow::Error::new(e) to wrap the original error and add context.
            Err(anyhow::Error::new(e).context(
                "Failed to get exit code from Sphinx strict check container (outer error)",
            ))
        }
    }
}

// Helper function to manage temporary git worktree removal
// Not strictly necessary with TempDir.close() but can be expanded for more
// robust cleanup fn cleanup_worktree(worktree_path: &Path, version: &str) ->
// AnyhowResult<()> {     info!(\"Cleaning up worktree for version {}: {:?}\",
// version, worktree_path);     std::process::Command::new(\"git\")
//         .args(&[\"worktree\", \"remove\", \"--force\",
// worktree_path.to_str().context(\"Invalid worktree path for cleanup\")?])
//         .status()
//         .context(format!(\"Failed to cleanup git worktree at {:?}\",
// worktree_path))         .and_then(|status| {
//             if status.success() {
//                 Ok(())
//             } else {
//                 Err(anyhow!(\"Git worktree remove command failed for {:?}
// with status {}\", worktree_path, status))             }
//         })?;
//     // TempDir will also attempt to remove its directory when it goes out of
// scope.     Ok(())
// }

// TODO: Add unit tests for this pipeline
// TODO: Add logic for docs coverage (if applicable/desired)

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    #[ignore] // Test requires Docker/Dagger, network access for git clone, and a valid
              // repo_url.
    async fn test_run_docs_pipeline_main_branch_mocked_env() {
        let temp_output_dir = tempdir().unwrap();
        let output_dir = temp_output_dir.path().to_path_buf();

        let temp_base_path_dir = tempdir().unwrap();
        let base_path = temp_base_path_dir.path().to_path_buf();

        let dummy_docs_dir = base_path.join("docs").join("_build").join("html");
        fs::create_dir_all(&dummy_docs_dir).unwrap();
        let dummy_html_content = "dummy main content for test";
        fs::write(dummy_docs_dir.join("index.html"), dummy_html_content).unwrap();

        // Create a justfile that copies the pre-made dummy content
        fs::write(
            base_path.join("justfile"),
            format!(
                "docs:\n    @echo 'Building dummy docs for main branch'\n    @mkdir -p \
                 docs/_build/html\n    @cp -r {}/* docs/_build/html/",
                dummy_docs_dir.parent().unwrap().to_str().unwrap()
            ),
        )
        .unwrap();

        let versions = vec!["main".to_string()];

        // Ensure DOCKER_HOST is set if not using default, e.g., for Colima:
        // std::env::set_var("DOCKER_HOST",
        // "unix:///Users/YOUR_USER/.colima/default/docker.sock");

        let result = run_docs_pipeline(base_path.clone(), output_dir.clone(), versions).await;

        if result.is_ok() {
            let main_output_html_path = output_dir.join("main").join("index.html");
            assert!(
                main_output_html_path.exists(),
                "Output file for main branch not found: {:?}",
                main_output_html_path
            );
            let content = fs::read_to_string(main_output_html_path).unwrap();
            assert!(content.contains(dummy_html_content), "Output HTML content mismatch");
            println!("Test test_run_docs_pipeline_main_branch_mocked_env PASSED (with Dagger)");
        } else {
            let err_msg = format!(
                "Test test_run_docs_pipeline_main_branch_mocked_env failed (Dagger connection or \
                 build error): {:?}",
                result.err().unwrap()
            );
            eprintln!("{}", err_msg);
            // For CI, we'd want this to panic to fail the test run clearly.
            panic!("{}", err_msg);
        }
    }

    // TODO: Add tests for specific versions (git clone branch) using a mock git
    // server or a public test repo. TODO: Add tests for error handling
    // (e.g., git clone fails, Dagger export fails)
}
