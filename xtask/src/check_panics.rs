use anyhow::Result;
use colored::Colorize;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use xshell::Shell;

/// Run a scan across all crates to find undocumented panics
pub fn run(sh: &Shell, fix: bool, only_failures: bool) -> Result<()> {
    println!("Scanning for undocumented panics...");

    let crates = vec![
        "wrt",
        "wrtd",
        // "xtask", // Exclude xtask from panic search
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

    let mut overall_status = true;

    for crate_name in &crates {
        match check_crate(sh, crate_name, fix, only_failures) {
            Ok(status) => {
                if !status {
                    overall_status = false;
                }
            }
            Err(e) => {
                println!("Error checking crate {}: {}", crate_name, e);
                overall_status = false;
            }
        }
    }

    if overall_status {
        println!("✅ {}", "All panics are properly documented.".green());
    } else {
        println!("❌ {}", "Some panics are not properly documented.".red());
        println!("Run with --fix to add missing documentation templates.");
    }

    Ok(())
}

fn check_crate(sh: &Shell, crate_name: &str, fix: bool, only_failures: bool) -> Result<bool> {
    let crate_path = PathBuf::from(crate_name);
    if !crate_path.exists() {
        return Ok(true);
    }

    // Run clippy to look for missing panic docs
    let mut has_undocumented_panics = false;

    // Use a command that continues even if clippy fails
    let output = sh
        .cmd("cargo")
        .arg("clippy")
        .arg("--package")
        .arg(crate_name)
        .arg("--")
        .arg("-W")
        .arg("clippy::missing_panics_doc")
        .args(["--error-format", "json"])
        .ignore_status()
        .output()?;

    // Check for warnings/errors in clippy output
    let stderr = String::from_utf8_lossy(&output.stderr);

    for line in stderr.lines() {
        if line.contains("missing_panics_doc") {
            has_undocumented_panics = true;

            if let Some(json_start) = line.find('{') {
                let json_str = &line[json_start..];
                if let Ok(warning) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let (Some(file), Some(message)) = (
                        warning["spans"][0]["file_name"].as_str(),
                        warning["message"].as_str(),
                    ) {
                        if let Some(line_num) = warning["spans"][0]["line_start"].as_u64() {
                            println!(
                                "Found undocumented panic in {}:{} - {}",
                                file, line_num, message
                            );

                            if fix {
                                add_panic_doc_template(file, line_num as usize)?;
                            }
                        }
                    }
                }
            }
        }
    }

    // Check if panic documentation has Safety impact and Tracking fields
    if !has_undocumented_panics {
        // Get all Rust files
        let mut missing_fields = false;
        let src_dir = crate_path.join("src");
        if src_dir.exists() {
            let output = sh
                .cmd("find")
                .arg(src_dir.to_str().unwrap())
                .arg("-name")
                .arg("*.rs")
                .output()?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let rust_files: Vec<&str> = stdout.lines().collect();

            for file_path in rust_files {
                let has_missing = check_panic_doc_fields(file_path, fix)?;
                if has_missing {
                    missing_fields = true;
                }
            }
        }

        has_undocumented_panics = has_undocumented_panics || missing_fields;
    }

    if !only_failures || has_undocumented_panics {
        println!(
            "Crate {}: {}",
            crate_name,
            if has_undocumented_panics {
                "❌ Has undocumented panics or missing fields".red()
            } else {
                "✅ All panics properly documented".green()
            }
        );
    }

    Ok(!has_undocumented_panics)
}

fn check_panic_doc_fields(file_path: &str, fix: bool) -> Result<bool> {
    let file = match File::open(file_path) {
        Ok(f) => f,
        Err(_) => return Ok(false),
    };

    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap_or_default()).collect();

    let mut line_idx = 0;
    let mut has_missing_fields = false;
    let mut need_to_fix = false;
    let mut fix_lines: Vec<(usize, String)> = Vec::new();

    while line_idx < lines.len() {
        let line = &lines[line_idx];

        if line.contains("# Panics") {
            let mut has_safety_impact = false;
            let mut has_tracking = false;
            let mut end_of_panic_doc = line_idx;
            let indentation = if let Some(idx) = line.find("///") {
                &line[0..idx]
            } else {
                ""
            };

            // Look for safety impact and tracking in subsequent lines
            let mut i = line_idx + 1;
            while i < lines.len() && (lines[i].contains("///") || lines[i].trim().is_empty()) {
                let l = &lines[i];
                if l.contains("Safety impact:") {
                    has_safety_impact = true;
                } else if l.contains("Tracking:") {
                    has_tracking = true;
                }
                end_of_panic_doc = i;
                i += 1;
            }

            if !has_safety_impact || !has_tracking {
                println!(
                    "{}:{} - Panic documentation is missing required fields: {}{}",
                    file_path,
                    line_idx + 1,
                    if !has_safety_impact {
                        "Safety impact"
                    } else {
                        ""
                    },
                    if !has_tracking {
                        if !has_safety_impact {
                            " and Tracking"
                        } else {
                            "Tracking"
                        }
                    } else {
                        ""
                    }
                );
                has_missing_fields = true;

                if fix {
                    need_to_fix = true;

                    // Add missing fields at the end of panic documentation
                    if !has_safety_impact {
                        let safety_template = format!("{}/// Safety impact: [LOW|MEDIUM|HIGH] - [Brief explanation of the safety implication]", indentation);
                        fix_lines.push((end_of_panic_doc + 1, safety_template));
                        end_of_panic_doc += 1;
                    }

                    if !has_tracking {
                        let tracking_template = format!(
                            "{}/// Tracking: WRTQ-XXX (qualification requirement tracking ID).",
                            indentation
                        );
                        fix_lines.push((end_of_panic_doc + 1, tracking_template));
                    }
                }
            }
        }

        line_idx += 1;
    }

    // Apply fixes if needed
    if need_to_fix {
        // Create a new file with fixes
        let mut new_content = String::new();
        let mut extra_lines_added = 0;

        for (i, line) in lines.iter().enumerate() {
            // Check if we need to insert any fixes before this line
            let current_pos = i + extra_lines_added;
            let fixes_at_pos: Vec<_> = fix_lines
                .iter()
                .filter(|(pos, _)| *pos == current_pos)
                .collect();

            // Insert fixes before the current line
            for (_, fix_line) in &fixes_at_pos {
                new_content.push_str(fix_line);
                new_content.push('\n');
                extra_lines_added += 1;
            }

            // Add the original line
            new_content.push_str(line);
            new_content.push('\n');
        }

        // Write the updated content back to the file
        let file_path = PathBuf::from(file_path);
        std::fs::write(&file_path, new_content)?;
        println!(
            "Added missing fields to panic documentation in {}",
            file_path.display()
        );
    }

    Ok(has_missing_fields)
}

/// Add a panic documentation template to the specified file and line
fn add_panic_doc_template(file: &str, line: usize) -> Result<()> {
    // First, read the file content
    let file_path = PathBuf::from(file);
    let file_content = std::fs::read_to_string(&file_path)?;
    let lines: Vec<&str> = file_content.lines().collect();

    // Find where to insert the documentation
    let mut insert_line = line;
    let mut indentation = "";

    // Move up to find the function definition
    for i in (0..line).rev() {
        let l = lines[i];
        if l.contains("fn ") {
            // Extract the indentation
            if let Some(idx) = l.find("fn") {
                indentation = &l[0..idx];
            }
            insert_line = i;
            break;
        }
    }

    // Prepare the template
    let doc_template = format!(
        "{}/// # Panics\n\
        {}///\n\
        {}/// This function will panic if [describe condition].\n\
        {}/// Safety impact: [LOW|MEDIUM|HIGH] - [Brief explanation of the safety implication]\n\
        {}///\n\
        {}/// Tracking: WRTQ-XXX (qualification requirement tracking ID).",
        indentation, indentation, indentation, indentation, indentation, indentation
    );

    // Insert the documentation
    let mut new_content = String::new();
    for (i, line) in lines.iter().enumerate() {
        if i == insert_line {
            new_content.push_str(&doc_template);
            new_content.push('\n');
        }
        new_content.push_str(line);
        new_content.push('\n');
    }

    // Write back to the file
    std::fs::write(&file_path, new_content)?;
    println!("Added panic documentation template to {}", file);

    Ok(())
}
