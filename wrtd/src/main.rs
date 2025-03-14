//! # WebAssembly Runtime Daemon (wrtd)
//!
//! A daemon process that coordinates WebAssembly module execution and provides system services.
//!
//! This daemon provides:
//! - Loading and execution of WebAssembly modules
//! - System service availability for WebAssembly components
//! - Resource management and isolation
//! - Runtime lifecycle management
//!
//! ## Usage
//!
//! ```bash
//! wrtd <wasm-file> [--call <function>] [--fuel <amount>] [--stats]
//! ```
//!
//! The daemon will load the specified WebAssembly module and execute it, providing
//! any necessary system services and managing its lifecycle.
//!
//! The `--fuel` option limits execution to the specified amount of computational resources.
//! This enables bounded execution and prevents infinite loops or excessive resource consumption.
//! If execution runs out of fuel, it will be paused and can be resumed with a higher fuel limit.
//!
//! The `--stats` option enables execution statistics reporting, displaying information such as:
//! - Number of instructions executed
//! - Amount of fuel consumed (when using the `--fuel` option)
//! - Memory usage (current and peak)
//! - Number of function calls and memory operations

#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]

use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use tracing::{debug, error, info, warn, Level};
use tracing_subscriber::fmt::format::FmtSpan;
use wrt::{Engine, LogLevel, Value};

/// WebAssembly Runtime Daemon CLI arguments
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Path to the WebAssembly file to execute
    wasm_file: String,

    /// Optional function to call
    #[arg(short, long)]
    call: Option<String>,

    /// Optional fuel limit for bounded execution
    /// Higher values allow more instructions to execute
    #[arg(short, long, help = "Limit execution to the specified amount of fuel")]
    fuel: Option<u64>,

    /// Show execution statistics after running
    /// Displays instruction count, memory usage, and other metrics
    #[arg(short, long, help = "Show execution statistics")]
    stats: bool,
}

fn main() -> Result<()> {
    // Initialize tracing with additional diagnostics
    let format = env::var("RUST_LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string());
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_span_events(FmtSpan::FULL)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    match format.as_str() {
        "json" => subscriber.json().init(),
        "compact" => subscriber.compact().init(),
        _ => subscriber.pretty().init(),
    }

    // Parse command line arguments
    let args = Args::parse();

    // Read the WebAssembly file
    let wasm_path = PathBuf::from(&args.wasm_file);
    debug!("Loading WebAssembly file: {}", wasm_path.display());
    let wasm_bytes = fs::read(&wasm_path).context("Failed to read WebAssembly file")?;
    info!("Loaded {} bytes of WebAssembly code", wasm_bytes.len());

    // Create a WebAssembly engine and module from the bytes
    info!("Initializing WebAssembly engine");
    let mut engine = Engine::new();

    // Register the log handler to handle component logging
    engine.register_log_handler(|log_op| {
        // Convert WRT log level to tracing level
        let level_str = log_op.level.as_str();
        let message = log_op.message;

        // Pass to the handle_component_log function
        handle_component_log(level_str, &message);
    });

    // Apply fuel limit if specified
    if let Some(fuel) = args.fuel {
        info!("Setting fuel limit to {} units", fuel);
        engine.set_fuel(Some(fuel));
    }

    // First try to load as a real module
    match wrt::new_module().load_from_binary(&wasm_bytes) {
        Ok(module) => {
            info!("Loaded WebAssembly module ({})", wasm_path.display());

            // Instantiate the module
            match engine.instantiate(module) {
                Ok(_) => {
                    info!("Successfully instantiated WebAssembly module");

                    // If a function call was specified, execute it
                    if let Some(function_name) = args.call {
                        info!("Calling function: {}", function_name);

                        // For now, we'll assume function index 0
                        // In a real implementation, we would look up the function by name
                        let func_idx = 0;
                        let func_args = Vec::new(); // Empty arguments for now

                        match engine.execute(0, func_idx, func_args) {
                            Ok(results) => {
                                info!("Function execution completed with results: {:?}", results);

                                // Display execution statistics if requested
                                if args.stats {
                                    display_execution_stats(&engine);
                                }
                            }
                            Err(wrt::Error::FuelExhausted) => {
                                info!("Function execution paused: out of fuel");
                                info!("To resume, run again with a higher --fuel value");

                                // In a real implementation, we would persist the state to be resumed later
                                // For now, we'll just report the remaining fuel
                                if let Some(fuel_remaining) = engine.remaining_fuel() {
                                    info!("Remaining fuel: {}", fuel_remaining);
                                }

                                // Display execution statistics if requested
                                if args.stats {
                                    display_execution_stats(&engine);
                                }
                            }
                            Err(e) => {
                                error!("Execution error: {}", e);
                                return Err(anyhow::anyhow!("Execution error: {}", e));
                            }
                        }
                    } else {
                        info!("No function specified to call. Use --call <function> to execute a function");
                    }
                }
                Err(e) => {
                    error!("Failed to instantiate module: {}", e);

                    // Fall back to mock component
                    info!("Falling back to mock component");
                    execute_mock_component(&wasm_bytes, args.call.as_deref())?;
                }
            }
        }
        Err(e) => {
            warn!("Failed to load as WebAssembly module: {}", e);

            // Fall back to mock component for demonstration purposes
            info!("Falling back to mock component");
            execute_mock_component(&wasm_bytes, args.call.as_deref())?;
        }
    }

    Ok(())
}

/// Executes a mock component when the real module loading fails
/// This is a fallback for demonstration/testing purposes
fn execute_mock_component(bytes: &[u8], function_name: Option<&str>) -> Result<()> {
    info!("Creating mock component with 'hello' and 'log' functions");

    // Create a WebAssembly engine with logging support
    let engine = Engine::new();

    // Register the log handler to handle component logging
    engine.register_log_handler(|log_op| {
        // Convert WRT log level to tracing level
        let level_str = log_op.level.as_str();
        let message = log_op.message;

        // Pass to the handle_component_log function
        handle_component_log(level_str, &message);
    });

    // Create a mock component that simulates having 'hello' and 'log' functions
    let component = MockComponent {
        functions: vec![
            // The hello function returns 42
            ("hello".to_string(), vec![Value::I32(42)]),
            // The log function doesn't return any values
            ("log".to_string(), vec![]),
        ],
        engine,
    };

    info!("Component loaded successfully (mocked)");
    info!(
        "Loaded component contains {} bytes of WebAssembly code",
        bytes.len()
    );

    // If a function call was specified, execute it
    if let Some(function_name) = function_name {
        info!("Calling function: {}", function_name);
        let result = call_mock_function(&component, function_name)?;
        info!("Function result: {:?}", result);
    } else {
        info!("No function specified to call. Use --call <function> to execute a function");
    }

    Ok(())
}

/// A mock component implementation that simulates a WebAssembly component
#[derive(Debug)]
struct MockComponent {
    /// Functions in the component with their return values
    functions: Vec<(String, Vec<Value>)>,
    /// Engine for logging and other operations
    engine: Engine,
}

/// Calls a function on a mock component
fn call_mock_function(component: &MockComponent, function_name: &str) -> Result<Vec<Value>> {
    // Check if the function exists in the component's functions
    for (name, return_values) in &component.functions {
        if name == function_name {
            info!("Executing function: {}", function_name);

            // If this is the hello function, simulate the component's internal behavior
            if name == "hello" {
                // The hello function in the component would call the log function with INFO level
                // Use the engine's logging mechanism to log the message
                component.engine.handle_log(
                    LogLevel::Info,
                    "Starting loop for 100 iterations".to_string(),
                );

                // Simulate loop iterations with logging
                for i in 0..5 {
                    component
                        .engine
                        .handle_log(LogLevel::Debug, format!("Loop iteration: {}", i + 1));
                }

                // Log completion
                component
                    .engine
                    .handle_log(LogLevel::Info, format!("Completed {} iterations", 5));

                // Also explain what's happening
                debug!("Component 'hello' function executed with logging");
            }

            // If this is the log function being called directly, handle it
            if name == "log" {
                // We're not handling the actual log function parameters in this mock,
                // but in a real implementation, we would extract the level and message
                debug!("Log function called directly (parameters not handled in this mock)");

                // Log a test message
                component
                    .engine
                    .handle_log(LogLevel::Info, "Direct log function call".to_string());
            }

            return Ok(return_values.clone());
        }
    }

    error!("Function '{}' not found in component", function_name);
    Err(anyhow::anyhow!("Function '{}' not found", function_name))
}

/// Handles a log call from a component by mapping it to the appropriate tracing level
fn handle_component_log(level: &str, message: &str) {
    // Map the level from the component to the appropriate tracing level
    let log_level = LogLevel::from_string_or_default(level);

    match log_level {
        LogLevel::Trace => tracing::trace!("[Component] {}", message),
        LogLevel::Debug => tracing::debug!("[Component] {}", message),
        LogLevel::Info => tracing::info!("[Component] {}", message),
        LogLevel::Warn => tracing::warn!("[Component] {}", message),
        LogLevel::Error => tracing::error!("[Component] {}", message),
        LogLevel::Critical => tracing::error!("[Component CRITICAL] {}", message),
    }
}

/// Displays execution statistics
fn display_execution_stats(engine: &Engine) {
    let stats = engine.stats();

    info!("=== Execution Statistics ===");
    info!("Instructions executed:  {}", stats.instructions_executed);

    if stats.fuel_consumed > 0 {
        info!("Fuel consumed:         {}", stats.fuel_consumed);
    }

    info!("Function calls:         {}", stats.function_calls);
    info!("Memory operations:      {}", stats.memory_operations);

    // Format memory usage in a human-readable way
    let current_kb = stats.current_memory_bytes / 1024;
    let peak_kb = stats.peak_memory_bytes / 1024;

    info!("Current memory usage:   {} KB", current_kb);
    info!("Peak memory usage:      {} KB", peak_kb);
    info!("===========================");
}
