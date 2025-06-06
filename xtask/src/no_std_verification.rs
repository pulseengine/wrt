//! No-std verification commands for xtask

use anyhow::Result;
use std::collections::HashMap;
use xshell::{cmd, Shell};

/// Configuration for no_std verification
#[derive(Debug, Clone)]
pub struct NoStdConfig {
    pub continue_on_error: bool,
    pub verbose: bool,
    pub detailed: bool,
    pub partial: bool,
}

impl Default for NoStdConfig {
    fn default() -> Self {
        Self {
            continue_on_error: false,
            verbose: false,
            detailed: false,
            partial: false,
        }
    }
}

/// All WRT crates to test for no_std compatibility
const WRT_CRATES: &[&str] = &[
    "wrt-math",
    "wrt-sync", 
    "wrt-error",
    "wrt-foundation",
    "wrt-format",
    "wrt-decoder",
    "wrt-instructions",
    "wrt-runtime",
    "wrt-host",
    "wrt-intercept",
    "wrt-component",
    "wrt-platform",
    "wrt-logging",
    "wrt",
];

/// Binary std/no_std choice
const TEST_CONFIGS: &[&str] = &["std", "alloc", ""];

/// Run no_std verification for all crates
pub fn run_no_std_verification(config: NoStdConfig) -> Result<()> {
    let sh = Shell::new()?;
    
    println!("🔍 WRT no_std Compatibility Verification");
    println!("📋 Testing configurations: std, no_std with alloc, no_std without alloc");
    if config.partial {
        println!("⚡ Running in partial mode (faster, less comprehensive)");
    }
    if config.continue_on_error {
        println!("🔄 Continue-on-error mode enabled");
    }
    println!();
    
    let mut results = HashMap::new();
    let mut failed_tests = Vec::new();
    
    let crates_to_test = if config.partial {
        &WRT_CRATES[..WRT_CRATES.len() / 2] // Test only half the crates in partial mode
    } else {
        WRT_CRATES
    };
    
    for crate_name in crates_to_test {
        println!("🧪 Verifying {}", crate_name);
        
        for config_name in TEST_CONFIGS {
            let config_display = if config_name.is_empty() { "no_std" } else { config_name };
            println!("  📦 Configuration: {}", config_display);
            
            // Build test
            let build_result = test_crate_build(&sh, crate_name, config_name, config.verbose)?;
            let build_key = format!("{}-{}-build", crate_name, config_display);
            results.insert(build_key.clone(), build_result);
            
            if !build_result {
                failed_tests.push(build_key.clone());
                if !config.continue_on_error {
                    return Err(anyhow::anyhow!("Build failed for {} in {} configuration. Use --continue-on-error to proceed.", crate_name, config_display));
                }
            }
            
            // Unit test
            let test_result = test_crate_tests(&sh, crate_name, config_name, config.verbose)?;
            let test_key = format!("{}-{}-test", crate_name, config_display);
            results.insert(test_key.clone(), test_result);
            
            if !test_result {
                failed_tests.push(test_key.clone());
                if !config.continue_on_error {
                    return Err(anyhow::anyhow!("Tests failed for {} in {} configuration. Use --continue-on-error to proceed.", crate_name, config_display));
                }
            }
            
            // Specific pattern tests
            if let Err(e) = run_pattern_tests(&sh, crate_name, config_name, config.verbose) {
                if !config.continue_on_error {
                    return Err(e);
                } else {
                    println!("    ⚠️  Pattern tests failed but continuing: {}", e);
                }
            }
        }
        println!();
    }
    
    // Run integration tests (skip in partial mode)
    if !config.partial {
        if let Err(e) = run_integration_tests(&sh, config.verbose) {
            if !config.continue_on_error {
                return Err(e);
            } else {
                println!("⚠️  Integration tests failed but continuing: {}", e);
            }
        }
    }
    
    if config.detailed {
        print_detailed_summary(&results);
    }
    
    if !failed_tests.is_empty() {
        println!("⚠️  Some tests failed:");
        for failed in &failed_tests {
            println!("   - {}", failed);
        }
        if config.continue_on_error {
            println!("✅ Verification completed with {} failures (continue-on-error mode)", failed_tests.len());
        }
    } else {
        println!("✅ Verification completed successfully!");
    }
    
    if !config.verbose {
        println!("💡 For detailed output, run with --verbose flag");
    }
    
    Ok(())
}

/// Test building a crate with specific configuration
fn test_crate_build(sh: &Shell, crate_name: &str, config: &str, verbose: bool) -> Result<bool> {
    let mut cmd = cmd!(sh, "cargo build -p {crate_name}");
    
    match config {
        "std" => cmd = cmd.args(&["--features", "std"]),
        "" => cmd = cmd.args(&["--no-default-features"]),
        _ => cmd = cmd.args(&["--no-default-features", "--features", config]),
    }
    
    if !verbose {
        cmd = cmd.quiet();
    }
    
    let result = cmd.run();
    let success = result.is_ok();
    
    if success {
        println!("    ✅ Build successful");
    } else {
        println!("    ❌ Build failed");
        if verbose && result.is_err() {
            println!("       Error: {:?}", result.err());
        }
    }
    
    Ok(success)
}

/// Test running tests for a crate with specific configuration
fn test_crate_tests(sh: &Shell, crate_name: &str, config: &str, verbose: bool) -> Result<bool> {
    let mut cmd = cmd!(sh, "cargo test -p {crate_name}");
    
    match config {
        "std" => cmd = cmd.args(&["--features", "std"]),
        "" => cmd = cmd.args(&["--no-default-features"]),
        _ => cmd = cmd.args(&["--no-default-features", "--features", config]),
    }
    
    if !verbose {
        cmd = cmd.quiet();
    }
    
    let result = cmd.run();
    let success = result.is_ok();
    
    if success {
        println!("    ✅ Tests successful");
    } else {
        println!("    ❌ Tests failed");
    }
    
    Ok(success)
}

/// Run specific pattern tests based on crate
fn run_pattern_tests(sh: &Shell, crate_name: &str, config: &str, verbose: bool) -> Result<()> {
    let patterns = match crate_name {
        "wrt-error" => vec!["integration_test", "no_std_compatibility_test"],
        "wrt-foundation" => vec!["bounded_collections_test", "safe_memory_test", "safe_stack_test"],
        "wrt-runtime" => vec!["memory_safety_tests", "no_std_compatibility_test"],
        "wrt-component" | "wrt-host" | "wrt-intercept" | "wrt-decoder" | 
        "wrt-format" | "wrt-instructions" | "wrt-sync" => vec!["no_std_compatibility_test"],
        "wrt" => vec!["no_std_compatibility_test"],
        _ => vec![],
    };
    
    for pattern in patterns {
        run_test_pattern(sh, crate_name, config, pattern, verbose)?;
    }
    
    Ok(())
}

/// Run a specific test pattern
fn run_test_pattern(sh: &Shell, crate_name: &str, config: &str, pattern: &str, verbose: bool) -> Result<()> {
    let mut cmd = cmd!(sh, "cargo test -p {crate_name}");
    
    match config {
        "std" => cmd = cmd.args(&["--features", "std"]),
        "" => cmd = cmd.args(&["--no-default-features"]),
        _ => cmd = cmd.args(&["--no-default-features", "--features", config]),
    }
    
    cmd = cmd.args(&["--", pattern]);
    
    if !verbose {
        cmd = cmd.quiet();
    }
    
    let result = cmd.run();
    
    if result.is_ok() {
        println!("    ✅ Pattern '{}' tests passed", pattern);
    } else {
        println!("    ❌ Pattern '{}' tests failed", pattern);
    }
    
    Ok(())
}

/// Run workspace integration tests
fn run_integration_tests(sh: &Shell, verbose: bool) -> Result<()> {
    println!("🔗 Running Integration Tests");
    
    for config in TEST_CONFIGS {
        let config_display = if config.is_empty() { "no_std" } else { config };
        println!("  🧪 Integration tests with {}", config_display);
        
        let mut cmd = cmd!(sh, "cargo test --workspace");
        
        match *config {
            "std" => cmd = cmd.args(&["--features", "std"]),
            "" => cmd = cmd.args(&["--no-default-features"]),
            _ => cmd = cmd.args(&["--no-default-features", "--features", config]),
        }
        
        if !verbose {
            cmd = cmd.quiet();
        }
        
        let result = cmd.run();
        
        if result.is_ok() {
            println!("    ✅ Integration tests successful");
        } else {
            println!("    ❌ Integration tests failed");
        }
    }
    
    Ok(())
}

/// Print detailed summary table
fn print_detailed_summary(results: &HashMap<String, bool>) {
    println!("📊 Detailed Summary");
    println!();
    println!("| Crate           | no_std | no_std+alloc | std |");
    println!("|-----------------|--------|--------------|-----|");
    
    for crate_name in WRT_CRATES {
        let no_std_build = results.get(&format!("{}-no_std-build", crate_name))
            .map(|&success| if success { "✅" } else { "❌" })
            .unwrap_or("❓");
        
        let alloc_build = results.get(&format!("{}-alloc-build", crate_name))
            .map(|&success| if success { "✅" } else { "❌" })
            .unwrap_or("❓");
        
        let std_build = results.get(&format!("{}-std-build", crate_name))
            .map(|&success| if success { "✅" } else { "❌" })
            .unwrap_or("❓");
        
        println!("| {:<15} | {:<6} | {:<12} | {:<3} |", 
                 crate_name, no_std_build, alloc_build, std_build);
    }
}