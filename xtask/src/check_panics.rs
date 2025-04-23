use anyhow::{anyhow, Result};
use std::path::PathBuf;
use xshell::Shell;

/// Run the check_panics command to scan all crates for undocumented panics
pub fn run(sh: &Shell, fix: bool, only_failures: bool) -> Result<()> {
    println!("Checking for undocumented panics across all crates...");

    // List of all crates in the workspace
    let crates = vec![
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
        "wrt-common",
        "wrt-intercept",
    ];

    let mut failed_crates = Vec::new();

    for crate_name in &crates {
        let crate_path = PathBuf::from(crate_name);
        if !crate_path.exists() {
            println!("Warning: Directory {} does not exist", crate_name);
            continue;
        }

        // Run clippy with the missing_panics_doc lint enabled
        let output = sh
            .cmd("cargo")
            .arg("clippy")
            .arg("--manifest-path")
            .arg(format!("{}/Cargo.toml", crate_name))
            .arg("--")
            .arg("-W")
            .arg("clippy::missing_panics_doc")
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let has_missing_docs = stderr.contains("missing_panics_doc");

        if has_missing_docs {
            failed_crates.push(crate_name.to_string());

            if !only_failures {
                println!("❌ {} has undocumented panics:", crate_name);

                // Extract and display the specific warnings
                for line in stderr.lines() {
                    if line.contains("missing_panics_doc") {
                        println!("   {}", line);
                    }
                }
                println!();
            }

            // If fix option is enabled, add panic doc templates
            if fix {
                println!(
                    "Auto-fixing is not implemented yet. Please add panic documentation manually."
                );
                // TODO: Implement auto-fixing by parsing the output and adding templates
            }
        } else if !only_failures {
            println!("✅ {} - All panics documented", crate_name);
        }
    }

    // Always show summary of failed crates
    if !failed_crates.is_empty() {
        println!("\n{} crates have undocumented panics:", failed_crates.len());
        for crate_name in &failed_crates {
            println!("  - {}", crate_name);
        }

        println!("\nPlease add proper panic documentation using this format:");
        println!("/// # Panics");
        println!("///");
        println!("/// This function will panic if [describe condition]");
        println!("///");
        println!("/// Safety impact: [LOW|MEDIUM|HIGH]");
        println!("/// Tracking: [WRTQ-XXX]");

        println!("\nSee docs/PANIC_DOCUMENTATION.md for more details.");

        // Return an error to indicate failure
        return Err(anyhow!(
            "Found undocumented panics in {} crates",
            failed_crates.len()
        ));
    } else {
        println!("\n✅ All crates have properly documented panics!");
    }

    Ok(())
}
