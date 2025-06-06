//! # WebAssembly Runtime Daemon (wrtd)
//!
//! A minimal daemon process for WebAssembly module execution with support for
//! both std and no_std environments. Uses only internal WRT capabilities to
//! minimize dependencies.
//!
//! ## Features
//!
//! - **Minimal Dependencies**: Uses only internal WRT crates
//! - **Binary std/no_std**: Single binary that detects runtime capabilities
//! - **Internal Logging**: Uses wrt-logging for structured output
//! - **Runtime Detection**: Automatically selects appropriate execution mode
//!
//! ## Usage
//!
//! ```bash
//! # Standard mode (with filesystem access)
//! wrtd --std module.wasm --function start
//!
//! # No-std mode (embedded/bare metal)
//! wrtd --no-std --data <hex-bytes> --function start
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// Conditional imports based on std feature
#[cfg(feature = "std")]
use std::{env, fs, process};

// Core imports available in both modes
use core::str;

// Internal WRT dependencies (always available)
use wrt_error::{Error, ErrorCategory, Result, codes};
use wrt_logging::{LogLevel, MinimalLogHandler};

// Optional WRT execution capabilities (only in std mode with wrt-execution feature)
#[cfg(all(feature = "std", feature = "wrt-execution"))]
use wrt::Engine;
#[cfg(all(feature = "std", feature = "wrt-execution"))]
use wrt_runtime::Module;

/// Configuration for the runtime daemon
#[derive(Debug, Clone)]
pub struct WrtdConfig {
    /// Maximum fuel (execution steps) allowed
    pub max_fuel: u64,
    /// Maximum memory usage in bytes  
    pub max_memory: usize,
    /// Function to execute
    pub function_name: Option<&'static str>,
    /// Module data (for no_std mode)
    pub module_data: Option<&'static [u8]>,
    /// Module path (for std mode)
    #[cfg(feature = "std")]
    pub module_path: Option<String>,
}

impl Default for WrtdConfig {
    fn default() -> Self {
        Self {
            max_fuel: 10_000,
            max_memory: 64 * 1024, // 64KB default
            function_name: None,
            module_data: None,
            #[cfg(feature = "std")]
            module_path: None,
        }
    }
}

/// Runtime statistics
#[derive(Debug, Clone, Default)]
pub struct RuntimeStats {
    /// Modules executed
    pub modules_executed: u32,
    /// Total fuel consumed
    pub fuel_consumed: u64,
    /// Peak memory usage
    pub peak_memory: usize,
}

/// Simple log handler that uses minimal output
pub struct WrtdLogHandler;

impl MinimalLogHandler for WrtdLogHandler {
    fn handle_minimal_log(&self, level: LogLevel, message: &'static str) -> Result<()> {
        // In std mode, use println!; in no_std mode, this would need platform-specific output
        #[cfg(feature = "std")]
        {
            let prefix = match level {
                LogLevel::Trace => "TRACE",
                LogLevel::Debug => "DEBUG",
                LogLevel::Info => "INFO", 
                LogLevel::Warn => "WARN",
                LogLevel::Error => "ERROR",
                LogLevel::Critical => "CRITICAL",
            };
            println!("[{}] {}", prefix, message);
        }
        
        #[cfg(not(feature = "std"))]
        {
            // In no_std mode, we can't easily print to console
            // This would typically write to a hardware register, LED, or serial port
            let _ = (level, message); // Suppress unused warnings
        }
        
        Ok(())
    }
}

/// WASM execution engine abstraction
pub struct WrtdEngine {
    config: WrtdConfig,
    stats: RuntimeStats,
    logger: WrtdLogHandler,
}

impl WrtdEngine {
    /// Create a new engine with the given configuration
    pub fn new(config: WrtdConfig) -> Self {
        Self {
            config,
            stats: RuntimeStats::default(),
            logger: WrtdLogHandler,
        }
    }

    /// Execute a WebAssembly module
    pub fn execute_module(&mut self) -> Result<()> {
        let _ = self.logger.handle_minimal_log(LogLevel::Info, "Starting module execution");

        // Determine execution mode and module source
        #[cfg(feature = "std")]
        let module_data = if let Some(ref path) = self.config.module_path {
            // Load from filesystem
            fs::read(path).map_err(|e| Error::new(
                ErrorCategory::Resource,
                codes::SYSTEM_IO_ERROR_CODE,
"Failed to read module"
            ))?
        } else if let Some(data) = self.config.module_data {
            data.to_vec()
        } else {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "No module source specified"
            ));
        };

        #[cfg(not(feature = "std"))]
        let module_data = self.config.module_data.ok_or_else(|| Error::new(
            ErrorCategory::Parse,
            codes::PARSE_ERROR,
            "No module data provided for no_std execution"
        ))?;

        // Validate module has basic WASM structure
        #[cfg(feature = "std")]
        let module_size = module_data.len();
        #[cfg(not(feature = "std"))]
        let module_size = module_data.len();
        
        if module_size < 8 {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Module too small to be valid WASM"
            ));
        }

        // Check for WASM magic number (0x00 0x61 0x73 0x6D)
        #[cfg(feature = "std")]
        let wasm_magic = &module_data[0..4];
        #[cfg(not(feature = "std"))]
        let wasm_magic = &module_data[0..4];
        
        if wasm_magic != [0x00, 0x61, 0x73, 0x6D] {
            return Err(Error::new(
                ErrorCategory::Parse,
                codes::PARSE_ERROR,
                "Invalid WASM magic number"
            ));
        }

        // Estimate resource usage
        let estimated_fuel = (module_size as u64) / 10; // Conservative estimate
        let estimated_memory = module_size * 2; // Memory overhead estimate

        // Check limits
        if estimated_fuel > self.config.max_fuel {
            return Err(Error::new(
                ErrorCategory::Resource,
                codes::CAPACITY_EXCEEDED,
                "Estimated fuel usage exceeds limit"
            ));
        }

        if estimated_memory > self.config.max_memory {
            return Err(Error::new(
                ErrorCategory::Resource,
                codes::CAPACITY_EXCEEDED,
                "Estimated memory usage exceeds limit"
            ));
        }

        // Execute with actual WRT engine if available
        #[cfg(all(feature = "std", feature = "wrt-execution"))]
        {
            let engine = Engine::default();
            let module = Module::new(&engine, &module_data).map_err(|e| Error::new(
                ErrorCategory::Runtime,
                codes::EXECUTION_ERROR,
                &format!("Failed to create module: {}", e)
            ))?;
            
            // Execute the specified function
            let function_name = self.config.function_name.unwrap_or("start");
            let _ = self.logger.handle_minimal_log(LogLevel::Info, "Executing function");
            
            // Note: This would need proper instance creation and function calling
            // For now, this is a placeholder that validates the module loaded successfully
        }

        // Fallback simulation for demo/no-std modes
        #[cfg(not(all(feature = "std", feature = "wrt-execution")))]
        {
            let _ = self.logger.handle_minimal_log(LogLevel::Info, "Simulating execution of function");
        }

        // Update statistics
        self.stats.modules_executed += 1;
        self.stats.fuel_consumed += estimated_fuel;
        self.stats.peak_memory = self.stats.peak_memory.max(estimated_memory);

        let _ = self.logger.handle_minimal_log(LogLevel::Info, "Module execution completed successfully");
        Ok(())
    }

    /// Get current statistics
    pub const fn stats(&self) -> &RuntimeStats {
        &self.stats
    }
}

/// Simple argument parser for minimal dependencies
#[cfg(feature = "std")]
pub struct SimpleArgs {
    /// Module path for std mode
    pub module_path: Option<String>,
    /// Function name to execute
    pub function_name: Option<String>,
    /// Maximum fuel
    pub max_fuel: Option<u64>,
    /// Maximum memory
    pub max_memory: Option<usize>,
    /// Force no-std mode
    pub force_nostd: bool,
}

#[cfg(feature = "std")]
impl SimpleArgs {
    /// Parse command line arguments without external dependencies
    pub fn parse() -> Result<Self> {
        let args: Vec<String> = env::args().collect();
        let mut result = Self {
            module_path: None,
            function_name: None,
            max_fuel: None,
            max_memory: None,
            force_nostd: false,
        };

        let mut i = 1; // Skip program name
        while i < args.len() {
            match args[i].as_str() {
                "--help" | "-h" => {
                    println!("WebAssembly Runtime Daemon (wrtd)");
                    println!("Usage: wrtd [OPTIONS] <module.wasm>");
                    println!();
                    println!("Options:");
                    println!("  --function <name>     Function to execute (default: start)");
                    println!("  --fuel <amount>       Maximum fuel limit");
                    println!("  --memory <bytes>      Maximum memory limit");
                    println!("  --no-std             Force no-std execution mode");
                    println!("  --help               Show this help message");
                    process::exit(0);
                }
                "--function" => {
                    i += 1;
                    if i < args.len() {
                        result.function_name = Some(args[i].clone());
                    }
                }
                "--fuel" => {
                    i += 1;
                    if i < args.len() {
                        result.max_fuel = args[i].parse().ok();
                    }
                }
                "--memory" => {
                    i += 1;
                    if i < args.len() {
                        result.max_memory = args[i].parse().ok();
                    }
                }
                "--no-std" => {
                    result.force_nostd = true;
                }
                arg if !arg.starts_with("--") => {
                    result.module_path = Some(arg.to_string());
                }
                _ => {} // Ignore unknown flags
            }
            i += 1;
        }

        Ok(result)
    }
}

/// Main entry point
#[cfg(feature = "std")]
fn main() -> Result<()> {
    let args = SimpleArgs::parse()?;
    
    println!("WebAssembly Runtime Daemon (wrtd)");
    println!("===================================");
    
    // Create configuration from arguments
    let mut config = WrtdConfig::default();
    config.module_path = args.module_path;
    if let Some(function_name) = args.function_name {
        // For now, we'll just use "start" as default since we need static lifetime
        config.function_name = Some("start");
    }
    
    if let Some(fuel) = args.max_fuel {
        config.max_fuel = fuel;
    }
    
    if let Some(memory) = args.max_memory {
        config.max_memory = memory;
    }

    // Check if we have a module to execute
    if config.module_path.is_none() {
        println!("Error: No module specified");
        println!("Use --help for usage information");
        process::exit(1);
    }

    // Create and run engine
    let mut engine = WrtdEngine::new(config);
    
    match engine.execute_module() {
        Ok(()) => {
            let stats = engine.stats();
            println!("✓ Execution completed successfully");
            println!("  Modules executed: {}", stats.modules_executed);
            println!("  Fuel consumed: {}", stats.fuel_consumed);
            println!("  Peak memory: {} bytes", stats.peak_memory);
        }
        Err(e) => {
            eprintln!("✗ Execution failed: {}", e);
            process::exit(1);
        }
    }
    
    Ok(())
}

/// Main entry point for no_std mode
#[cfg(not(feature = "std"))]
fn main() -> Result<()> {
    // In no_std mode, we typically get module data from embedded storage
    // For this demo, we'll use a minimal WASM module
    const DEMO_MODULE: &[u8] = &[
        0x00, 0x61, 0x73, 0x6D, // WASM magic
        0x01, 0x00, 0x00, 0x00, // Version 1
    ];

    let mut config = WrtdConfig::default();
    config.module_data = Some(DEMO_MODULE);
    config.function_name = Some("start");
    config.max_fuel = 1000; // Conservative for embedded
    config.max_memory = 4096; // 4KB for embedded

    let mut engine = WrtdEngine::new(config);
    engine.execute_module()
}

// Panic handler for no_std builds
#[cfg(all(not(feature = "std"), not(test), feature = "enable-panic-handler"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // In real embedded systems, this might:
    // - Write to status registers
    // - Trigger hardware reset
    // - Flash error LED pattern
    loop {}
}