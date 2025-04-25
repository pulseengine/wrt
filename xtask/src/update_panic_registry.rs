use anyhow::Result;
use chrono::Local;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;
use xshell::Shell;

/// Extract panic documentation from Rust source files and update the panic registry CSV
pub fn run(sh: &Shell, output_path: &str, verbose: bool) -> Result<()> {
    println!("Scanning codebase for panic documentation...");
    let start_time = Instant::now();

    // List of all crates in the workspace to scan
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

    // Structure to hold panic information
    struct PanicInfo {
        file_path: String,
        function_name: String,
        line_number: usize,
        panic_condition: String,
        safety_impact: String,
        tracking_id: String,
        resolution_status: String,
        handling_strategy: String,
        last_updated: String,
    }

    // Load existing panic registry to preserve any manual updates
    let mut existing_entries = HashMap::new();
    if Path::new(output_path).exists() {
        let file = File::open(output_path)?;
        let reader = BufReader::new(file);

        // Skip header row
        for (i, line) in reader.lines().enumerate() {
            if let Ok(line) = line {
                if i == 0 {
                    // Skip header
                    continue;
                }

                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 7 {
                    let key = format!("{}:{}", parts[0].trim(), parts[1].trim());
                    let resolution_status = parts[6].trim().to_string();
                    let handling_strategy = if parts.len() >= 8 {
                        parts[7].trim().to_string()
                    } else {
                        String::new()
                    };

                    existing_entries.insert(key, (resolution_status, handling_strategy));
                }
            }
        }
    }

    let mut panic_infos = Vec::new();

    // Track which files we've already checked to avoid duplicates
    let mut processed_files = HashSet::new();

    for crate_name in &crates {
        let crate_path = PathBuf::from(crate_name);
        if !crate_path.exists() {
            if verbose {
                println!("Warning: Directory {} does not exist", crate_name);
            }
            continue;
        }

        let src_dir = crate_path.join("src");
        if !src_dir.exists() {
            if verbose {
                println!(
                    "Warning: Source directory {}/src does not exist",
                    crate_name
                );
            }
            continue;
        }

        // Find all Rust files in the crate
        let output = sh
            .cmd("find")
            .arg(src_dir.to_str().unwrap())
            .arg("-name")
            .arg("*.rs")
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let rust_files: Vec<&str> = stdout.lines().collect();

        for file_path in rust_files {
            if processed_files.contains(file_path) {
                continue;
            }
            processed_files.insert(file_path.to_string());

            if verbose {
                println!("Scanning {}", file_path);
            }

            let file = match File::open(file_path) {
                Ok(f) => f,
                Err(e) => {
                    println!("Error opening {}: {}", file_path, e);
                    continue;
                }
            };

            let reader = BufReader::new(file);
            let lines: Vec<String> = reader.lines().map(|l| l.unwrap_or_default()).collect();

            let mut line_idx = 0;
            while line_idx < lines.len() {
                let line = &lines[line_idx];

                // Look for panic documentation
                if line.contains("# Panics") {
                    // Extract function name
                    let mut function_name = String::new();
                    for i in (0..line_idx).rev() {
                        let prev_line = &lines[i];
                        if prev_line.contains("fn ") {
                            // Extract function name
                            if let Some(fn_idx) = prev_line.find("fn ") {
                                let substring = &prev_line[fn_idx + 3..];
                                if let Some(name_end) = substring.find(['(', '<']) {
                                    function_name = substring[0..name_end].trim().to_string();
                                    break;
                                }
                            }
                        }
                    }

                    if function_name.is_empty() {
                        line_idx += 1;
                        continue;
                    }

                    // Extract panic condition
                    let mut panic_condition = String::new();
                    let mut safety_impact = String::new();
                    let mut tracking_id = String::new();

                    // Look for panic condition, safety impact, and tracking ID
                    let mut i = line_idx + 1;
                    while i < lines.len() {
                        let l = &lines[i];
                        if l.contains("///") || l.trim().is_empty() {
                            if l.contains("Safety impact:") {
                                if let Some(impact_idx) = l.find("Safety impact:") {
                                    safety_impact = l[impact_idx + 14..].trim().to_string();
                                }
                            } else if l.contains("Tracking:") {
                                if let Some(track_idx) = l.find("Tracking:") {
                                    tracking_id = l[track_idx + 9..].trim().to_string();
                                }
                            } else if l.contains("///")
                                && !l.contains("# Panics")
                                && !panic_condition.is_empty()
                            {
                                // Append to existing panic condition
                                let content = l.trim_start().trim_start_matches("///").trim();
                                if !content.is_empty() {
                                    panic_condition.push(' ');
                                    panic_condition.push_str(content);
                                }
                            } else if l.contains("///")
                                && !l.contains("# Panics")
                                && panic_condition.is_empty()
                            {
                                // Start capturing panic condition
                                let content = l.trim_start().trim_start_matches("///").trim();
                                if !content.is_empty() {
                                    panic_condition = content.to_string();
                                }
                            }
                            i += 1;
                        } else {
                            break;
                        }
                    }

                    // Lookup existing resolution status and handling strategy
                    let relative_path = file_path.trim_start_matches("./");
                    let key = format!("{}:{}", relative_path, function_name);
                    let (resolution_status, handling_strategy) = existing_entries
                        .get(&key)
                        .cloned()
                        .unwrap_or((String::from("Todo"), String::new()));

                    panic_infos.push(PanicInfo {
                        file_path: relative_path.to_string(),
                        function_name,
                        line_number: line_idx + 1,
                        panic_condition,
                        safety_impact,
                        tracking_id,
                        resolution_status,
                        handling_strategy,
                        last_updated: Local::now().format("%Y-%m-%d").to_string(),
                    });
                }

                line_idx += 1;
            }
        }
    }

    // Write to CSV
    let mut file = File::create(output_path)?;

    // Write header
    writeln!(file, "File Path,Function Name,Line Number,Panic Condition,Safety Impact,Tracking ID,Resolution Status,Handling Strategy,Last Updated")?;

    // Get the count before consuming panic_infos
    let count = panic_infos.len();

    // Write entries
    for info in &panic_infos {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{}",
            info.file_path,
            info.function_name,
            info.line_number,
            info.panic_condition,
            info.safety_impact,
            info.tracking_id,
            info.resolution_status,
            info.handling_strategy,
            info.last_updated
        )?;
    }

    let elapsed = start_time.elapsed();
    println!("Successfully updated panic registry at {}", output_path);
    println!("Found {} panic documentation entries", count);
    println!("Time taken: {:.2?}", elapsed);

    Ok(())
}
