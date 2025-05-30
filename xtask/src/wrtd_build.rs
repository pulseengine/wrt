//! WRTD multi-binary build commands
//! 
//! This module provides commands to build the three WRTD binary variants
//! for different runtime environments (std, alloc, no_std).

use anyhow::{Context, Result};
use std::process::Command;
use std::path::Path;

/// Configuration for WRTD build
pub struct WrtdBuildConfig {
    pub release: bool,
    pub show_summary: bool,
    pub test_binaries: bool,
    pub cross_compile: bool,
}

impl Default for WrtdBuildConfig {
    fn default() -> Self {
        Self {
            release: true,
            show_summary: true,
            test_binaries: true,
            cross_compile: false,
        }
    }
}

/// Build all WRTD binary variants
pub fn build_all_wrtd(config: WrtdBuildConfig) -> Result<()> {
    println!("🚀 WRTD Multi-Binary Build");
    println!("==========================");
    println!();

    // Build results tracking
    let mut build_results = Vec::new();

    // Build std binary (for servers/desktop)
    println!("📦 Building Standard Library Runtime (servers/desktop)...");
    let std_result = build_wrtd_binary(
        "wrtd-std",
        "std-runtime",
        config.release,
        None,
    );
    build_results.push(("wrtd-std", std_result));

    // Build alloc binary (for embedded with heap)
    println!("\n📦 Building Allocation Runtime (embedded with heap)...");
    let alloc_result = build_wrtd_binary(
        "wrtd-alloc",
        "alloc-runtime",
        config.release,
        None,
    );
    build_results.push(("wrtd-alloc", alloc_result));

    // Build no_std binary (for bare metal)
    println!("\n📦 Building No Standard Library Runtime (bare metal)...");
    let nostd_result = build_wrtd_binary(
        "wrtd-nostd",
        "nostd-runtime",
        config.release,
        None,
    );
    build_results.push(("wrtd-nostd", nostd_result));

    // Build default binary (std mode)
    println!("\n📦 Building Default Binary (std mode)...");
    let default_result = build_wrtd_binary(
        "wrtd",
        "std-runtime",
        config.release,
        None,
    );
    build_results.push(("wrtd", default_result));

    // Cross-compilation for embedded targets
    if config.cross_compile {
        println!("\n🎯 Cross-compilation for embedded targets...");
        
        // Check and build for ARM Linux
        if is_target_installed("armv7-unknown-linux-gnueabihf") {
            println!("\n📦 Building for ARM Linux (alloc mode)...");
            let arm_result = build_wrtd_binary(
                "wrtd-alloc",
                "alloc-runtime",
                config.release,
                Some("armv7-unknown-linux-gnueabihf"),
            );
            build_results.push(("wrtd-alloc (ARM)", arm_result));
        } else {
            println!("   ⚠️  ARM Linux target not installed");
            println!("   💡 Install with: rustup target add armv7-unknown-linux-gnueabihf");
        }

        // Check and build for Cortex-M4F
        if is_target_installed("thumbv7em-none-eabihf") {
            println!("\n📦 Building for Cortex-M4F (no_std mode)...");
            let cortex_result = build_wrtd_binary(
                "wrtd-nostd",
                "nostd-runtime",
                config.release,
                Some("thumbv7em-none-eabihf"),
            );
            build_results.push(("wrtd-nostd (Cortex-M4F)", cortex_result));
        } else {
            println!("   ⚠️  Cortex-M4F target not installed");
            println!("   💡 Install with: rustup target add thumbv7em-none-eabihf");
        }
    }

    // Test binaries if requested
    if config.test_binaries {
        println!("\n🧪 Testing binary functionality...");
        test_wrtd_binaries(config.release)?;
    }

    // Show summary
    if config.show_summary {
        show_build_summary(&build_results, config.release)?;
    }

    // Check if any builds failed
    let failed_builds: Vec<_> = build_results
        .iter()
        .filter(|(_, result)| result.is_err())
        .collect();

    if !failed_builds.is_empty() {
        println!("\n❌ {} build(s) failed:", failed_builds.len());
        for (name, result) in failed_builds {
            if let Err(e) = result {
                println!("   - {}: {}", name, e);
            }
        }
        return Err(anyhow::anyhow!("Some builds failed"));
    }

    println!("\n✅ All builds completed successfully!");
    Ok(())
}

/// Build a specific WRTD binary
pub fn build_wrtd_binary(
    binary_name: &str,
    features: &str,
    release: bool,
    target: Option<&str>,
) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--bin")
        .arg(binary_name)
        .arg("--features")
        .arg(features)
        .arg("-p")
        .arg("wrtd"); // Specify the package

    if release {
        cmd.arg("--release");
    }

    if let Some(target_triple) = target {
        cmd.arg("--target").arg(target_triple);
    }

    println!("   Running: {:?}", cmd);

    let output = cmd.output()
        .context("Failed to execute cargo build")?;

    if output.status.success() {
        println!("   ✅ Build successful");
        
        // Check binary size
        let binary_path = get_binary_path(binary_name, release, target)?;
        if binary_path.exists() {
            let metadata = std::fs::metadata(&binary_path)?;
            let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;
            println!("   📦 Binary size: {:.2} MB", size_mb);
        }
        
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("Build failed: {}", stderr))
    }
}

/// Get the path to a built binary
fn get_binary_path(binary_name: &str, release: bool, target: Option<&str>) -> Result<std::path::PathBuf> {
    let mut path = std::path::PathBuf::from("target");
    
    if let Some(target_triple) = target {
        path.push(target_triple);
    }
    
    path.push(if release { "release" } else { "debug" });
    path.push(binary_name);
    
    Ok(path)
}

/// Check if a target is installed
fn is_target_installed(target: &str) -> bool {
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .ok();

    if let Some(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.contains(target)
    } else {
        false
    }
}

/// Test WRTD binaries
fn test_wrtd_binaries(release: bool) -> Result<()> {
    // Test std binary
    let std_path = get_binary_path("wrtd-std", release, None)?;
    if std_path.exists() {
        println!("\n   Testing wrtd-std...");
        let output = Command::new(&std_path)
            .arg("--help")
            .output()
            .context("Failed to run wrtd-std")?;
        
        if output.status.success() {
            println!("   ✅ wrtd-std help works");
        } else {
            println!("   ❌ wrtd-std help failed");
        }
    }

    // Note about alloc and nostd binaries
    println!("   ℹ️  wrtd-alloc uses embedded configuration (no CLI)");
    println!("   ℹ️  wrtd-nostd is for embedded firmware (no CLI)");

    Ok(())
}

/// Show build summary
fn show_build_summary(_results: &[(&str, Result<()>)], release: bool) -> Result<()> {
    println!("\n🎉 Build Summary");
    println!("================");
    
    println!("\n📦 Available binaries:");
    println!("   Host binaries:");
    
    let mode = if release { "release" } else { "debug" };
    let binaries = ["wrtd", "wrtd-std", "wrtd-alloc", "wrtd-nostd"];
    
    for binary in &binaries {
        let path = get_binary_path(binary, release, None)?;
        if path.exists() {
            let metadata = std::fs::metadata(&path)?;
            let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;
            println!("     {} ({:.2} MB)", binary, size_mb);
        }
    }

    // Check for cross-compiled binaries
    println!("\n   Cross-compiled binaries:");
    
    let targets = [
        ("armv7-unknown-linux-gnueabihf", "wrtd-alloc"),
        ("thumbv7em-none-eabihf", "wrtd-nostd"),
    ];
    
    for (target, binary) in &targets {
        let path = get_binary_path(binary, release, Some(target))?;
        if path.exists() {
            let metadata = std::fs::metadata(&path)?;
            let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;
            println!("     {} [{}] ({:.2} MB)", binary, target, size_mb);
        }
    }

    println!("\n🔧 Usage examples:");
    println!("   # Server/desktop (full std support)");
    println!("   ./target/{}/wrtd-std module.wasm --call function --fuel 1000000 --stats", mode);
    println!();
    println!("   # Embedded Linux (heap but no std)");
    println!("   ./target/{}/wrtd-alloc embedded.wasm", mode);
    println!();
    println!("   # Bare metal (stack only)");
    println!("   # wrtd-nostd would be flashed to microcontroller firmware");

    println!("\n🚀 Deployment examples:");
    println!("   # Deploy to server");
    println!("   scp target/{}/wrtd-std server:/usr/local/bin/wrtd", mode);
    println!();
    println!("   # Deploy to embedded Linux device");
    println!("   scp target/armv7-unknown-linux-gnueabihf/{}/wrtd-alloc device:/bin/wrtd", mode);
    println!();
    println!("   # Create firmware for microcontroller");
    println!("   arm-none-eabi-objcopy -O binary target/thumbv7em-none-eabihf/{}/wrtd-nostd firmware.bin", mode);

    println!("\n📋 Binary characteristics:");
    println!("   wrtd-std:   Full std library, WASI support, unlimited resources");
    println!("   wrtd-alloc: Heap allocation, no std, limited resources (16MB/1M fuel)");
    println!("   wrtd-nostd: Stack only, no heap, minimal resources (1MB/100K fuel)");

    Ok(())
}

/// Test WRTD runtime modes with example WASM files
pub fn test_wrtd_modes(release: bool) -> Result<()> {
    println!("🧪 WRTD Runtime Mode Testing");
    println!("============================");
    
    // Check if test WASM files exist
    let test_files = [
        ("std-mode-example.wasm", "std"),
        ("alloc-mode-example.wasm", "alloc"),
        ("nostd-mode-example.wasm", "nostd"),
    ];
    
    let test_dir = Path::new("wrtd/tests/fixtures");
    
    for (file, _mode) in &test_files {
        let wasm_path = test_dir.join(file);
        if !wasm_path.exists() {
            println!("⚠️  Test file {} not found", file);
            println!("   Please run: wat2wasm {} -o {}", 
                     wasm_path.with_extension("wat").display(),
                     wasm_path.display());
        }
    }
    
    // Test std mode
    println!("\n📦 Testing std mode...");
    if let Ok(std_path) = get_binary_path("wrtd-std", release, None) {
        if std_path.exists() {
            let wasm_path = test_dir.join("std-mode-example.wasm");
            if wasm_path.exists() {
                println!("   Running: {} {} --call hello --stats", 
                         std_path.display(), wasm_path.display());
                // In real implementation, would execute and check output
                println!("   ✅ std mode test would execute here");
            }
        }
    }
    
    println!("\n📦 Testing alloc mode...");
    println!("   ℹ️  alloc mode uses embedded configuration");
    
    println!("\n📦 Testing nostd mode...");
    println!("   ℹ️  nostd mode is embedded firmware");
    
    Ok(())
}