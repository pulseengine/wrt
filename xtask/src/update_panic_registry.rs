use anyhow::Result;
use chrono::Local;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;
use xshell::Shell;

/// Structure representing a panic entry with safety impact and tracking ID
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

/// Extract panic documentation from Rust source files and update the panic registry CSV
/// and generate an RST file for sphinx-needs integration
pub fn run(sh: &Shell, output_path: &str, verbose: bool) -> Result<()> {
    println!("Scanning codebase for panic documentation...");
    let start_time = Instant::now();

    // List of all crates in the workspace to scan
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
                            } else if l.contains("///") && !l.contains("# Panics") {
                                // Extract the content after ///
                                let content = l.trim_start().trim_start_matches("///").trim();

                                if !content.is_empty() {
                                    // If we already have content, append; otherwise set as new content
                                    if !panic_condition.is_empty() {
                                        panic_condition.push(' ');
                                        panic_condition.push_str(content);
                                    } else {
                                        panic_condition = content.to_string();
                                    }
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

    // Generate RST file from the CSV
    generate_rst_file(&panic_infos, output_path)?;

    let elapsed = start_time.elapsed();
    println!("Successfully updated panic registry at {}", output_path);
    println!("Found {} panic documentation entries", count);
    println!("Time taken: {:?}", elapsed);

    Ok(())
}

/// Generate an RST file for sphinx-needs from the panic registry data
fn generate_rst_file(panic_infos: &[PanicInfo], csv_path: &str) -> Result<()> {
    // Create RST path in qualification directory instead of the same location as CSV
    let qualification_dir = Path::new("docs/source/qualification");

    // Create qualification directory if it doesn't exist
    if !qualification_dir.exists() {
        std::fs::create_dir_all(qualification_dir)?;
    }

    let rst_file_name = Path::new(csv_path).file_name().unwrap();
    let rst_path = qualification_dir.join(rst_file_name).with_extension("rst");
    let mut file = File::create(&rst_path)?;

    // Write RST header
    writeln!(file, ".. _panic-registry:")?;
    writeln!(file)?;
    writeln!(file, "Panic Registry")?;
    writeln!(file, "==============")?;
    writeln!(file)?;
    writeln!(
        file,
        "This document contains all documented panic conditions in the WRT codebase."
    )?;
    writeln!(
        file,
        "Each panic is tracked as a qualification requirement using sphinx-needs."
    )?;
    writeln!(file)?;
    writeln!(file, ".. contents:: Table of Contents")?;
    writeln!(file, "   :local:")?;
    writeln!(file, "   :depth: 2")?;
    writeln!(file)?;
    writeln!(file, "Summary")?;
    writeln!(file, "-------")?;
    writeln!(file)?;
    writeln!(file, "* Total panic points: {}", panic_infos.len())?;

    // Count by status
    let mut todo_count = 0;
    let mut in_progress_count = 0;
    let mut resolved_count = 0;

    for info in panic_infos {
        match info.resolution_status.as_str() {
            "Todo" => todo_count += 1,
            "In Progress" => in_progress_count += 1,
            "Resolved" => resolved_count += 1,
            _ => todo_count += 1,
        }
    }

    writeln!(file, "* Status:")?;
    writeln!(file, "  * Todo: {}", todo_count)?;
    writeln!(file, "  * In Progress: {}", in_progress_count)?;
    writeln!(file, "  * Resolved: {}", resolved_count)?;
    writeln!(file)?;
    writeln!(
        file,
        "The original CSV version of this registry is maintained at:"
    )?;
    writeln!(file, "{}", csv_path)?;
    writeln!(file)?;
    writeln!(file, ".. csv-table:: Panic Registry CSV")?;
    writeln!(
        file,
        "   :file: {}",
        Path::new(csv_path).file_name().unwrap().to_string_lossy()
    )?;
    writeln!(file, "   :header-rows: 1")?;
    writeln!(file, "   :widths: 20, 15, 5, 20, 5, 10, 10, 15")?;
    writeln!(file)?;
    writeln!(file, "Panic Details")?;
    writeln!(file, "------------")?;
    writeln!(file)?;

    // Add entries as needs
    for (i, info) in panic_infos.iter().enumerate() {
        // Create ID from tracking ID if available, otherwise generate one
        let id = if !info.tracking_id.is_empty() {
            info.tracking_id.clone()
        } else {
            format!("WRTQ-{:04}", i + 1)
        };

        // Determine safety level
        let safety_level = if info.safety_impact.starts_with("LOW") {
            "LOW"
        } else if info.safety_impact.starts_with("MEDIUM") {
            "MEDIUM"
        } else if info.safety_impact.starts_with("HIGH") {
            "HIGH"
        } else {
            "UNKNOWN"
        };

        // Clean up panic condition and safety impact for RST format
        let panic_condition = info.panic_condition.replace('\n', " ");

        writeln!(file, ".. qual:: {}", info.function_name)?;
        writeln!(file, "   :id: {}", id)?;
        writeln!(file, "   :status: {}", info.resolution_status)?;
        writeln!(file, "   :implementation: {}", info.handling_strategy)?;
        writeln!(file, "   :tags: panic, {}", safety_level.to_lowercase())?;
        writeln!(file)?;
        writeln!(file, "   **File:** {}", info.file_path)?;
        writeln!(file, "   **Line:** {}", info.line_number)?;
        writeln!(file, "   **Function:** {}", info.function_name)?;
        writeln!(file, "   **Safety Impact:** {}", info.safety_impact)?;
        writeln!(file, "   **Last Updated:** {}", info.last_updated)?;
        writeln!(file)?;
        writeln!(file, "   {}", panic_condition)?;
        writeln!(file)?;
    }

    println!(
        "Successfully generated sphinx-needs RST file at {}",
        rst_path.display()
    );
    Ok(())
}
