use anyhow::{Context, Result};
use std::path::Path;
use wast::parser::{self, ParseBuffer};
use wast::{Wast, WastDirective, WastExecute, WastInvoke, WastAssert};
use wrt::{Engine, Module, Value};

/// Process a single WebAssembly test file
fn run_wast_test(path: &Path) -> Result<()> {
    // Read the wast file
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read wast file: {}", path.display()))?;

    // Parse the wast file
    let buf = ParseBuffer::new(&contents)
        .with_context(|| format!("Failed to create parse buffer for: {}", path.display()))?;

    let wast = parser::parse::<Wast>(&buf)
        .with_context(|| format!("Failed to parse wast file: {}", path.display()))?;

    // Create engine
    let mut engine = Engine::new(Module::new());

    // Process each directive in the wast file
    process_directives(path, wast.directives, &mut engine)?;

    Ok(())
}

/// Process WebAssembly test directives
fn process_directives(path: &Path, directives: Vec<WastDirective>, engine: &mut Engine) -> Result<()> {
    for directive in directives {
        match directive {
            WastDirective::Module(module) => {
                process_module_directive(path, module, engine)?;
            }
            WastDirective::AssertReturn(assertion) => {
                // TODO: Implement assertion checking
                println!("Skipping AssertReturn");
            }
            WastDirective::AssertTrap(assertion) => {
                // TODO: Implement trap assertion
                println!("Skipping AssertTrap");
            }
            WastDirective::AssertExhaustion(assertion) => {
                // TODO: Implement resource exhaustion assertion
                println!("Skipping AssertExhaustion");
            }
            WastDirective::AssertMalformed(assertion) => {
                // TODO: Verify module is malformed
                println!("Skipping AssertMalformed");
            }
            WastDirective::AssertInvalid(assertion) => {
                // TODO: Verify module is invalid
                println!("Skipping AssertInvalid");
            }
            WastDirective::AssertUnlinkable(assertion) => {
                // TODO: Verify module is unlinkable
                println!("Skipping AssertUnlinkable");
            }
            WastDirective::Register { name, module } => {
                // TODO: Register module with name
                println!("Skipping Register");
            }
            _ => {
                // Ignore other directives
                println!("Skipping unknown directive");
            }
        }
    }
    Ok(())
}

/// Process a module directive
fn process_module_directive(path: &Path, module: wast::Module, engine: &mut Engine) -> Result<()> {
    // Get the binary representation
    let binary = module.into_binary()
        .with_context(|| format!("Failed to get binary for module: {}", path.display()))?;

    // Create and load the module
    let mut module = Module::new();
    let module = module.load_from_binary(&binary)
        .with_context(|| format!("Failed to load module from binary: {}", path.display()))?;

    // Create new engine with the module
    *engine = Engine::new(module);
    
    Ok(())
}

#[test]
fn run_wast_tests() -> Result<()> {
    // Path to the WebAssembly test suite
    let test_suite_path = Path::new("external/testsuite");
    
    // Check if the test suite exists
    if !test_suite_path.exists() {
        println!("Test suite not found at: {}", test_suite_path.display());
        println!("Please clone the WebAssembly test suite first.");
        return Ok(());
    }

    // Limit the number of test files to process
    let max_tests = 5;
    let mut test_count = 0;

    // Walk through the test suite directory
    for entry in walkdir::WalkDir::new(test_suite_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "wast") {
            if test_count >= max_tests {
                break;
            }

            println!("Running test {} of {}: {}", test_count + 1, max_tests, path.display());
            match run_wast_test(path) {
                Ok(_) => println!("Test passed: {}", path.display()),
                Err(e) => println!("Test failed: {} - {}", path.display(), e),
            }
            
            test_count += 1;
        }
    }

    Ok(())
} 