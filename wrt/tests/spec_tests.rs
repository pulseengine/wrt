use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use wrt::error::Result;
use wrt::execution::Engine;
use wrt::module::Module;
use wrt::values::Value;

// List of simple WAST tests to download and run first
const SIMPLE_TESTS: &[&str] = &[
    "memory.wast",            // Basic memory operations
    "memory_redundancy.wast", // Tests for memory redundancy checks
    "memory_size.wast",       // Tests for memory size operations
    "memory_trap.wast",       // Tests for memory access traps
    "address.wast",           // Tests for memory addressing
    "align.wast",             // Tests for alignment in memory operations
    "data.wast",              // Tests for data segments
];

// Test directory where downloaded WAST files will be stored
const TEST_DIR: &str = "wrt/tests/spec";

// URL of the WebAssembly spec test repository
const REPO_URL: &str = "https://raw.githubusercontent.com/WebAssembly/testsuite/main/";

fn ensure_test_dir() -> PathBuf {
    let path = PathBuf::from(TEST_DIR);
    if !path.exists() {
        fs::create_dir_all(&path).expect("Failed to create test directory");
    }
    path
}

fn download_test(test_name: &str) -> PathBuf {
    let test_dir = ensure_test_dir();
    let file_path = test_dir.join(test_name);

    if !file_path.exists() {
        println!("Downloading test: {}", test_name);
        let url = format!("{}{}", REPO_URL, test_name);

        // Use curl to download the file
        let output = Command::new("curl")
            .arg("--silent")
            .arg("-L")
            .arg(&url)
            .output()
            .expect("Failed to execute curl");

        if !output.status.success() {
            panic!(
                "Failed to download {}: {}",
                test_name,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let mut file = fs::File::create(&file_path).expect("Failed to create test file");
        file.write_all(&output.stdout)
            .expect("Failed to write test file");
    }

    file_path
}

fn extract_modules_from_wast(wast_path: &Path) -> Vec<(String, PathBuf)> {
    let wast_content = fs::read_to_string(wast_path).expect("Failed to read WAST file");

    // Extremely simplified extraction - in a real implementation,
    // we would use the wast crate to properly parse the file
    let mut modules = Vec::new();
    let mut in_module = false;
    let mut module_content = String::new();
    let mut module_count = 0;
    let mut module_name = String::new();

    let dir = wast_path.parent().unwrap();
    let base_name = wast_path.file_stem().unwrap().to_string_lossy();

    for line in wast_content.lines() {
        // Check for module name in form (module $name ...)
        if line.contains("(module") && !in_module {
            in_module = true;
            module_content.clear();
            module_content.push_str(line);
            module_content.push('\n');

            // Extract module name if present
            if let Some(name_start) = line.find("$") {
                if let Some(name_end) = line[name_start..].find(char::is_whitespace) {
                    module_name = line[name_start..name_start + name_end].to_string();
                } else {
                    module_name = format!("module_{}", module_count);
                }
            } else {
                module_name = format!("module_{}", module_count);
            }
        } else if in_module {
            module_content.push_str(line);
            module_content.push('\n');

            // Count balanced parentheses to determine end of module
            if line.ends_with(")") && !line.starts_with(";;") {
                // This is a simplistic approach - a proper implementation
                // would use the wast parser to extract modules

                let file_name = format!("{}_{}.wat", base_name, module_count);
                let module_path = dir.join(file_name);

                fs::write(&module_path, &module_content).expect("Failed to write module file");

                modules.push((module_name.clone(), module_path));
                module_count += 1;
                in_module = false;
            }
        }
    }

    modules
}

fn try_execute_module(wasm_binary: &[u8], func_name: &str) -> Result<Vec<Value>> {
    // Create a default module to hold the binary data
    let mut module = Module::default();

    // Load the module from binary data (this is based on the API we saw)
    module = module.load_from_binary(wasm_binary)?;

    // Create a new engine with the loaded module
    let mut engine = Engine::new(Module::default());

    // Instantiate the module by passing it as an argument
    engine.instantiate(module)?;

    // Find the function by name and execute it
    if engine.instance_count() > 0 {
        if let Some(instance) = engine.get_instance(0) {
            // Look for the exported function by name
            for export in &instance.module.exports {
                if export.name == func_name {
                    // Found the function, execute it
                    return engine.execute(0, export.index, vec![]);
                }
            }
        }
    }

    // Function not found or no instances created
    println!("Function '{}' not found in module", func_name);
    Ok(vec![])
}

#[test]
#[ignore] // This test is slow and requires network access
fn test_spec_memory() {
    for test_name in SIMPLE_TESTS {
        if test_name.contains("memory") {
            let test_path = download_test(test_name);
            let modules = extract_modules_from_wast(&test_path);

            for (module_name, module_path) in modules {
                println!("Testing module '{}': {:?}", module_name, module_path);

                // Read WAT content
                let wat_content =
                    fs::read_to_string(&module_path).expect("Failed to read WAT file");

                // Skip modules that contain features we don't support yet
                if wat_content.contains("(import") || 
                   wat_content.contains("(memory $") ||  // Named memories not supported
                   wat_content.contains("shared")
                {
                    // Shared memories not supported
                    println!("  Skipping: contains unsupported features");
                    continue;
                }

                // Extract exported function names
                let mut export_funcs = Vec::new();
                for line in wat_content.lines() {
                    if line.contains("(export") && line.contains("(func") {
                        if let Some(name_start) = line.find("\"") {
                            if let Some(name_end) = line[name_start + 1..].find("\"") {
                                let func_name = &line[name_start + 1..name_start + 1 + name_end];
                                export_funcs.push(func_name.to_string());
                            }
                        }
                    }
                }

                match wat::parse_str(&wat_content) {
                    Ok(wasm) => {
                        // Try to execute all exported functions
                        for func_name in export_funcs {
                            println!("  Executing function: {}", func_name);

                            match try_execute_module(&wasm, &func_name) {
                                Ok(results) => println!("    Success! Results: {:?}", results),
                                Err(e) => println!("    Error: {:?}", e),
                            }
                        }

                        println!("  Module testing complete");
                    }
                    Err(e) => {
                        println!("  Error parsing WAT: {:?}", e);
                    }
                }
            }
        }
    }
}

#[test]
#[ignore] // This test is slow and requires network access
fn test_spec_data() {
    // Test specifically for data segment tests
    let test_path = download_test("data.wast");
    let modules = extract_modules_from_wast(&test_path);

    for (module_name, module_path) in modules {
        println!("Testing data module '{}': {:?}", module_name, module_path);

        let wat_content = fs::read_to_string(&module_path).expect("Failed to read WAT file");

        // Skip modules that contain features we don't support yet
        if wat_content.contains("(import") || wat_content.contains("(memory $") {
            // Named memories not supported
            println!("  Skipping: contains unsupported features");
            continue;
        }

        // Extract exported function names
        let mut export_funcs = Vec::new();
        for line in wat_content.lines() {
            if line.contains("(export") && line.contains("(func") {
                if let Some(name_start) = line.find("\"") {
                    if let Some(name_end) = line[name_start + 1..].find("\"") {
                        let func_name = &line[name_start + 1..name_start + 1 + name_end];
                        export_funcs.push(func_name.to_string());
                    }
                }
            }
        }

        match wat::parse_str(&wat_content) {
            Ok(wasm) => {
                // Try to execute all exported functions
                for func_name in export_funcs {
                    println!("  Executing function: {}", func_name);

                    match try_execute_module(&wasm, &func_name) {
                        Ok(results) => println!("    Success! Results: {:?}", results),
                        Err(e) => println!("    Error: {:?}", e),
                    }
                }

                println!("  Module testing complete");
            }
            Err(e) => {
                println!("  Error parsing WAT: {:?}", e);
            }
        }
    }
}

#[test]
#[ignore] // This test is slow and requires network access
fn test_download_all_simple_tests() {
    // Just download all the test files but don't run them yet
    for test_name in SIMPLE_TESTS {
        let test_path = download_test(test_name);
        println!("Downloaded test: {:?}", test_path);
    }
}
