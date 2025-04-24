use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Read;
#[cfg(feature = "wrt-integration")]
use std::io::Write;
use std::path::{Path, PathBuf};
#[cfg(feature = "wrt-integration")]
use std::time::Instant;
#[cfg(feature = "wrt-integration")]
use walkdir::WalkDir;
#[cfg(feature = "wrt-integration")]
use wat;
#[cfg(feature = "wrt-integration")]
use wrt::{Module, StacklessEngine};

// Assume testsuite is relative to the workspace root where xtask is run
#[allow(dead_code)]
const TEST_SUITE_PATH: &str = "wrt/testsuite";
// Output files in the workspace root
#[allow(dead_code)]
const PASSED_FILE: &str = "wast_passed.md";
#[allow(dead_code)]
const FAILED_FILE: &str = "wast_failed.md";

/// Process a single WebAssembly test file
#[cfg(feature = "wrt-integration")]
fn run_wast_test(path: &Path) -> Result<String> {
    let start = Instant::now();

    // Read the wast file
    let wast_content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read wast file: {}", path.display()))?;

    // Basic verification that this is a valid WAST file
    if !wast_content.contains("(module") {
        return Err(anyhow::anyhow!("No module found in WAST file"));
    }

    // Create a shared engine
    let module =
        Module::new().map_err(|e| anyhow::anyhow!("Failed to create empty module: {}", e))?;
    let mut shared_engine = StacklessEngine::new(module);

    // Extract and test all modules from the WAST file
    let mut module_idx = 0;
    let mut pos = 0;
    let mut modules_loaded = 0;

    while let Some(module_start_idx) = wast_content[pos..].find("(module") {
        let module_start_idx = pos + module_start_idx;
        let mut depth = 0;
        let mut end_pos = 0;

        // Find the matching closing parenthesis
        for (i, c) in wast_content[module_start_idx..].char_indices() {
            if c == '(' {
                depth += 1;
            } else if c == ')' {
                depth -= 1;
                if depth == 0 {
                    end_pos = module_start_idx + i + 1;
                    break;
                }
            }
        }

        if end_pos > 0 {
            let module_wat = &wast_content[module_start_idx..end_pos];

            // Skip modules with 'quote' as they're not standard WAT
            if module_wat.contains("quote") {
                pos = end_pos;
                module_idx += 1;
                continue;
            }

            // Try to parse the WAT
            match wat::parse_str(module_wat) {
                Ok(wasm_bytes) => {
                    // Try to load the module in our runtime
                    let mut wrt_module = Module::new()
                        .map_err(|e| anyhow::anyhow!("Failed to create module: {}", e))?;

                    match wrt_module.load_from_binary(&wasm_bytes) {
                        Ok(loaded_module) => {
                            // Try to instantiate the module
                            if shared_engine.instantiate(loaded_module.clone()).is_ok() {
                                modules_loaded += 1;
                            } else {
                                // Return error if instantiation fails for any module in the file
                                return Err(anyhow::anyhow!(
                                    "Failed to instantiate module index {}",
                                    module_idx
                                ));
                            }
                        }
                        Err(e) => {
                            return Err(anyhow::anyhow!(
                                "Failed to load module index {} from binary: {}",
                                module_idx,
                                e
                            ));
                        }
                    }
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Failed to parse WAT for module index {}: {}",
                        module_idx,
                        e
                    ));
                }
            }
        }

        // If no closing paren found for a module, something is wrong
        if end_pos == 0 {
            return Err(anyhow::anyhow!(
                "Malformed WAST file: Unmatched parenthesis for module starting near position {}",
                module_start_idx
            ));
        }

        pos = end_pos;
        module_idx += 1;
    }

    if modules_loaded == 0 {
        // If we parsed the file but found no loadable modules (e.g., all quotes)
        return Err(anyhow::anyhow!(
            "No modules were successfully loaded (skipped or failed)"
        ));
    }

    let duration = start.elapsed();
    Ok(format!(
        "Loaded {} modules in {:.2?}",
        modules_loaded, duration
    ))
}

#[cfg(not(feature = "wrt-integration"))]
#[allow(dead_code)]
fn run_wast_test(_path: &Path) -> Result<String> {
    // Stub implementation when wrt is not available
    Err(anyhow::anyhow!(
        "WAST tests are disabled - wrt-integration feature not enabled"
    ))
}

/// Load tests from a markdown file
#[allow(dead_code)]
fn load_tests_from_md(file_path: &Path) -> HashSet<PathBuf> {
    let mut tests = HashSet::new();

    // Return empty set if file doesn't exist
    if !file_path.exists() {
        return tests;
    }

    // Read file content
    let mut content = String::new();
    if let Ok(mut file) = File::open(file_path) {
        if file.read_to_string(&mut content).is_err() {
            eprintln!(
                "Warning: Failed to read markdown file: {}",
                file_path.display()
            );
            return tests;
        }
    } else {
        eprintln!(
            "Warning: Failed to open markdown file: {}",
            file_path.display()
        );
        return tests;
    }

    // Extract test paths from markdown file (format: "- `path/to/test.wast`")
    for line in content.lines() {
        if line.starts_with("- `") && line.contains("` - ") {
            if let Some(end_backtick) = line.find("` - ") {
                let path_str = line[3..end_backtick].trim();
                // Store paths relative to the workspace root
                tests.insert(PathBuf::from(path_str));
            }
        }
    }

    tests
}

/// Update the markdown files with test results
#[allow(dead_code)]
fn update_md_files(
    passed: &HashMap<PathBuf, String>,
    failed: &HashMap<PathBuf, String>,
    create_files: bool,
) -> Result<()> {
    if create_files || !passed.is_empty() {
        // Write passed tests
        let mut passed_content = "# Passing WAST Tests\n\n".to_string();
        // Sort for consistent output
        let mut sorted_passed: Vec<_> = passed.iter().collect();
        sorted_passed.sort_by_key(|(k, _)| *k);
        for (path, info) in sorted_passed {
            passed_content.push_str(&format!("- `{}` - {}\n", path.display(), info));
        }

        fs::write(PASSED_FILE, passed_content)
            .context(format!("Failed to write {}", PASSED_FILE))?;
        println!("Written passing tests to {}", PASSED_FILE);
    }

    if create_files || !failed.is_empty() {
        // Write failed tests
        let mut failed_content = "# Failed WAST Tests\n\n".to_string();
        // Sort for consistent output
        let mut sorted_failed: Vec<_> = failed.iter().collect();
        sorted_failed.sort_by_key(|(k, _)| *k);
        for (path, error) in sorted_failed {
            failed_content.push_str(&format!("- `{}` - Error: {}\n", path.display(), error));
        }

        fs::write(FAILED_FILE, failed_content)
            .context(format!("Failed to write {}", FAILED_FILE))?;
        println!("Written failed tests to {}", FAILED_FILE);
    }

    Ok(())
}

// Main function to be called by xtask
pub fn run(_create_files: bool, _verify_passing: bool) -> Result<()> {
    #[cfg(not(feature = "wrt-integration"))]
    {
        println!("WAST test runner is disabled because the wrt-integration feature is not enabled");
        println!("To enable, build with --features=wrt-integration");
        Ok(())
    }

    #[cfg(feature = "wrt-integration")]
    {
        // Path to the WebAssembly test suite
        let test_suite_path = Path::new(TEST_SUITE_PATH);

        // Check if the test suite exists
        if !test_suite_path.exists() {
            println!("Test suite not found at: {}", test_suite_path.display());
            println!(
                "Please make sure the WebAssembly test suite submodule is initialized and updated:"
            );
            println!("  git submodule update --init --recursive");
            // Return Ok, as this isn't a failure of the runner itself
            return Ok(());
        }

        // Load existing test results
        let passed_file_path = Path::new(PASSED_FILE);
        let failed_file_path = Path::new(FAILED_FILE);

        let known_passed = load_tests_from_md(passed_file_path);
        let known_failed = load_tests_from_md(failed_file_path);

        // If --verify-passing is specified but wast_passed.md doesn't exist, exit
        if _verify_passing && !passed_file_path.exists() {
            println!("No {} file found to verify against.", PASSED_FILE);
            return Ok(());
        }

        // Track test results
        let mut passed_tests = HashMap::new();
        let mut failed_tests = HashMap::new();
        let mut count = 0;
        let mut run_count = 0;

        // If not creating files and not verifying, check if files exist
        if !_create_files
            && !_verify_passing
            && !passed_file_path.exists()
            && !failed_file_path.exists()
        {
            println!("No existing test results found. Run with --create-files to initialize.");
            return Ok(());
        }

        // Only if we are verifying passing, we'll focus on known passing tests
        let tests_to_run = if _verify_passing {
            known_passed.iter().cloned().collect::<Vec<_>>()
        } else {
            // Otherwise, discover all WAST files in the test suite
            WalkDir::new(test_suite_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let path = e.path();
                    path.is_file() && path.extension().map_or(false, |ext| ext == "wast")
                })
                .map(|e| e.path().to_path_buf())
                .collect::<Vec<_>>()
        };

        let total_tests = tests_to_run.len();
        println!("Found {} WAST tests to run", total_tests);

        // Run the tests
        for path in tests_to_run {
            count += 1;
            if count % 10 == 0 || count == total_tests {
                print!("\rRunning test {}/{}", count, total_tests);
                std::io::stdout().flush().unwrap();
            }

            // Skip known failed tests unless verify_passing mode
            if !_verify_passing && known_failed.contains(&path) {
                continue;
            }

            run_count += 1;
            let relative_path = path
                .strip_prefix(std::env::current_dir().unwrap())
                .unwrap_or(&path);

            match run_wast_test(&path) {
                Ok(info) => {
                    passed_tests.insert(relative_path.to_path_buf(), info);
                }
                Err(e) => {
                    // In verify mode, check if this was a previously passing test
                    if _verify_passing && known_passed.contains(relative_path) {
                        println!(
                            "\n❌ ERROR: Regression in previously passing test: {}",
                            relative_path.display()
                        );
                        println!("  Error: {}", e);
                        return Err(anyhow::anyhow!(
                            "Regression detected in previously passing test: {}",
                            relative_path.display()
                        ));
                    }

                    failed_tests.insert(relative_path.to_path_buf(), e.to_string());
                }
            }
        }

        println!("\nRan {} tests", run_count);
        println!(
            "Passed: {}, Failed: {}",
            passed_tests.len(),
            failed_tests.len()
        );

        // Update the output files if requested
        if _create_files {
            update_md_files(&passed_tests, &failed_tests, true)?;
        } else if !_verify_passing {
            // In normal mode, update with any newly passed/failed tests
            update_md_files(&passed_tests, &failed_tests, false)?;
        }

        Ok(())
    }
}

// Add a public wrapper for run function to maintain backward compatibility with the old name
pub fn run_wast_tests(_sh: &xshell::Shell, create_files: bool, verify_passing: bool) -> Result<()> {
    // We don't use the Shell parameter in the implementation
    run(create_files, verify_passing)
}
