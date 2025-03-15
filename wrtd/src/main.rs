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
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::Parser;
use tracing::{debug, error, info, warn, Level};
use tracing_subscriber::fmt::format::FmtSpan;
use wrt::{Engine, ExportKind, ExternType, LogLevel, Module, Value};

/// WebAssembly Runtime Daemon CLI arguments
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Path to the WebAssembly Component file to execute
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

/// Performance timing information for executing a WebAssembly module
struct ExecutionTiming {
    /// Time to load the WebAssembly file
    load_time: Duration,
    /// Time to parse the WebAssembly module
    parse_time: Duration,
    /// Time to instantiate the WebAssembly module
    instantiate_time: Duration,
    /// Time to execute the WebAssembly function
    execution_time: Duration,
    /// Total elapsed time
    total_time: Duration,
}

fn main() -> Result<()> {
    // Initialize tracing
    initialize_tracing();

    // Parse command line arguments
    let args = Args::parse();

    // Setup timings for performance measurement
    let _start_time = Instant::now();

    // Read the WebAssembly file
    let (wasm_path, wasm_bytes, _load_time) = load_wasm_file(&args.wasm_file)?;

    // Create a WebAssembly engine
    info!("Initializing WebAssembly Component engine");
    let mut engine = create_engine(args.fuel);

    // Load and execute as WebAssembly Component Model only
    if let Err(e) = load_component(
        &mut engine,
        &wasm_bytes,
        args.call.as_deref(),
        wasm_path.display().to_string(),
    ) {
        error!("Failed to load WebAssembly Component: {}", e);
        return Err(anyhow::anyhow!(
            "Failed to load WebAssembly Component: {}",
            e
        ));
    }

    if args.stats {
        display_execution_stats(&engine);
    }

    Ok(())
}

/// Initialize the tracing system for logging
fn initialize_tracing() {
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
}

/// Create a WebAssembly Component engine with the specified fuel limit
fn create_engine(fuel: Option<u64>) -> Engine {
    let mut engine = Engine::new();

    // Register the log handler to handle component logging
    engine.register_log_handler(|log_op| {
        // Get context from the component_id if available, otherwise use a default
        let context = log_op.component_id.as_deref().unwrap_or("component");

        // Handle the log operation with proper context included
        handle_component_log(
            log_op.level.as_str(),
            &format!("{}: {}", context, log_op.message),
        );

        // Also print directly to stdout for debugging
        println!(
            "[Handler] {} log from {}: {}",
            log_op.level.as_str(),
            context,
            log_op.message
        );
    });

    // Apply fuel limit if specified
    if let Some(fuel) = fuel {
        info!("Setting fuel limit to {} units", fuel);
        engine.set_fuel(Some(fuel));
    } else {
        // Default fuel for components to prevent infinite loops
        info!("Setting default fuel limit of 1000000 units");
        engine.set_fuel(Some(1000000));
    }

    engine
}

/// Load a WebAssembly file from disk
fn load_wasm_file(file_path: &str) -> Result<(PathBuf, Vec<u8>, Duration)> {
    let wasm_path = PathBuf::from(file_path);
    debug!("Loading WebAssembly file: {}", wasm_path.display());

    let load_start = Instant::now();
    let wasm_bytes = fs::read(&wasm_path).context("Failed to read WebAssembly file")?;
    let load_time = load_start.elapsed();

    info!(
        "Loaded {} bytes of WebAssembly code in {:?}",
        wasm_bytes.len(),
        load_time
    );

    Ok((wasm_path, wasm_bytes, load_time))
}

/// Load and execute a WebAssembly module
fn load_and_execute_module(
    engine: &mut Engine,
    wasm_bytes: &[u8],
    wasm_path: &std::path::Path,
    function_name: Option<&str>,
    total_start_time: Instant,
    show_stats: bool,
) -> Result<()> {
    let parse_start = Instant::now();

    // Try to load the module
    let module = wrt::new_module()
        .load_from_binary(wasm_bytes)
        .context("Failed to load WebAssembly module")?;

    let parse_time = parse_start.elapsed();
    info!(
        "Loaded WebAssembly module ({}) in {:?}",
        wasm_path.display(),
        parse_time
    );

    // Instantiate the module
    let inst_start = Instant::now();
    engine
        .instantiate(module)
        .context("Failed to instantiate WebAssembly module")?;

    let instantiate_time = inst_start.elapsed();
    info!(
        "Successfully instantiated WebAssembly module in {:?}",
        instantiate_time
    );

    // If a function call was specified, execute it
    if let Some(function_name) = function_name {
        info!("Calling function: {}", function_name);

        // Find the function to call
        let instance_idx = 0; // First instance
        let (func_idx, func_args) = find_function_to_call(engine, instance_idx, function_name)?;

        // Execute the function
        let exec_start = Instant::now();
        info!(
            "Executing function with {} arguments: {:?}",
            func_args.len(),
            func_args
        );

        match engine.execute(instance_idx as u32, func_idx, func_args) {
            Ok(results) => {
                let execution_time = exec_start.elapsed();
                info!(
                    "Function execution completed in {:?} with results: {:?}",
                    execution_time, results
                );

                // Print for tests to check
                println!("Function result: {:?}", results);

                // Display execution statistics if requested
                if show_stats {
                    display_execution_stats(engine);
                }

                // Display detailed timing breakdown
                display_timing_breakdown(&ExecutionTiming {
                    load_time: Duration::default(), // Not tracked here
                    parse_time,
                    instantiate_time,
                    execution_time,
                    total_time: total_start_time.elapsed(),
                });
            }
            Err(wrt::Error::FuelExhausted) => {
                let execution_time = exec_start.elapsed();
                handle_fuel_exhaustion(engine, execution_time, show_stats);
            }
            Err(e) => {
                let execution_time = exec_start.elapsed();
                error!("Execution error after {:?}: {}", execution_time, e);
                return Err(anyhow::anyhow!("Execution error: {}", e));
            }
        }
    } else {
        info!("No function specified to call. Use --call <function> to execute a function");
    }

    Ok(())
}

/// Handle the case where execution ran out of fuel
fn handle_fuel_exhaustion(engine: &Engine, execution_time: Duration, show_stats: bool) {
    info!(
        "Function execution paused after {:?}: out of fuel",
        execution_time
    );
    info!("To resume, run again with a higher --fuel value");

    // Report the remaining fuel
    if let Some(fuel_remaining) = engine.remaining_fuel() {
        info!("Remaining fuel: {}", fuel_remaining);
    }

    // Display execution statistics if requested
    if show_stats {
        display_execution_stats(engine);
    }
}

/// Find a function to call in a module by name or index
fn find_function_to_call(
    engine: &Engine,
    instance_idx: usize,
    function_name: &str,
) -> Result<(u32, Vec<Value>)> {
    let instance = &engine.instances[instance_idx];
    let module = &instance.module;
    let mut func_idx = 0; // Default to function index 0
    let mut is_found = false;

    // Display module imports
    display_module_imports(module);

    // Display module exports and find the function
    display_module_exports(module, function_name, &mut func_idx, &mut is_found);

    // Detect if this is a component model module
    let is_component = module
        .custom_sections
        .iter()
        .any(|s| s.name == "component-model-info");

    if is_component {
        info!("Detected WebAssembly Component Model module");
    }

    if !is_found {
        warn!(
            "Export '{}' not found, using function index {}",
            function_name, func_idx
        );
    }

    // Create arguments based on function type
    let expected_args = get_expected_argument_count(module, function_name, func_idx);

    // Create default arguments (zeros) for each parameter
    let mut func_args = Vec::new();
    for _ in 0..expected_args {
        func_args.push(Value::I32(42)); // Default argument value for testing
    }

    if expected_args > 0 {
        info!(
            "Adding {} default arguments (value: 42) for function call",
            expected_args
        );
    }

    Ok((func_idx, func_args))
}

/// Display information about a module's imports
fn display_module_imports(module: &Module) {
    if !module.imports.is_empty() {
        info!("Module imports:");
        for import in &module.imports {
            let import_desc = format_extern_type(&import.ty);
            info!("  - {}.{} -> {}", import.module, import.name, import_desc);
        }
    }
}

/// Represents an export entry
#[derive(Clone)]
struct ExportEntry {
    /// Name of the export
    #[allow(dead_code)]
    name: String,
    /// Kind of the export (Function, Table, Memory, Global)
    #[allow(dead_code)]
    kind: ExportKind,
    /// Index of the exported item
    index: u32,
}

/// Display information about a module's exports
fn display_module_exports(
    module: &Module,
    function_name: &str,
    func_idx: &mut u32,
    is_found: &mut bool,
) {
    info!("Module exports:");
    for export in &module.exports {
        info!("  - {} ({:?})", export.name, export.kind);

        if matches!(export.kind, ExportKind::Function) {
            display_function_export_details(
                module,
                &ExportEntry {
                    name: export.name.clone(),
                    kind: export.kind.clone(),
                    index: export.index,
                },
            );
        }

        if export.name == function_name && matches!(export.kind, ExportKind::Function) {
            *func_idx = export.index;
            *is_found = true;
            info!(
                "Found export '{}' at function index {}",
                function_name, *func_idx
            );
        }
    }
}

/// Display detailed information about a function export
fn display_function_export_details(module: &Module, export: &ExportEntry) {
    // Get more details about the exported function type
    let type_info = get_function_type_info(module, export);
    info!("    Type info: {}", type_info);

    // Display function details if available
    display_function_body_preview(module, export);
}

/// Get function type information
fn get_function_type_info(module: &Module, export: &ExportEntry) -> String {
    if export.index < module.imports.len() as u32 {
        // For imported functions that are exported
        let import = &module.imports[export.index as usize];
        if let wrt::ExternType::Function(func_type) = &import.ty {
            let params = format_value_types(&func_type.params);
            let results = format_value_types(&func_type.results);
            format!(
                " (Re-export of imported function. Type: params: [{}], results: [{}])",
                params, results
            )
        } else {
            String::from(" (Re-export of unknown import type)")
        }
    } else {
        // For normal functions
        let import_func_count = count_imported_functions(module);
        let adjusted_idx = export
            .index
            .checked_sub(import_func_count as u32)
            .unwrap_or(export.index);

        if adjusted_idx as usize >= module.functions.len() {
            format!(" (Invalid function index: {})", adjusted_idx)
        } else {
            let func = &module.functions[adjusted_idx as usize];
            let func_type = &module.types[func.type_idx as usize];

            let params = format_value_types(&func_type.params);
            let results = format_value_types(&func_type.results);

            format!(" (Type: params: [{}], results: [{}])", params, results)
        }
    }
}

/// Display a preview of the function body (instructions)
fn display_function_body_preview(module: &Module, export: &ExportEntry) {
    let import_func_count = count_imported_functions(module);
    let adjusted_idx = export.index as usize;

    if adjusted_idx >= import_func_count
        && (adjusted_idx - import_func_count) < module.functions.len()
    {
        let func_idx = adjusted_idx - import_func_count;
        let func = &module.functions[func_idx];

        info!("    Function details:");
        info!("      - Type index: {}", func.type_idx);
        info!("      - Locals: {} variables", func.locals.len());
        info!("      - Body: {} instructions", func.body.len());

        // Display a preview of the function body (first few instructions)
        if !func.body.is_empty() {
            let preview_count = std::cmp::min(5, func.body.len());
            info!("      - Instruction preview:");
            for (i, instr) in func.body.iter().take(preview_count).enumerate() {
                info!("        {}. {:?}", i, instr);
            }
            if func.body.len() > preview_count {
                info!(
                    "        ... and {} more instructions",
                    func.body.len() - preview_count
                );
            }
        }
    }
}

/// Get the number of arguments expected by a function
fn get_expected_argument_count(module: &Module, function_name: &str, func_idx: u32) -> usize {
    // First, get comprehensive info on the function we want to call
    info!(
        "Looking up function signature for '{}' (index {})",
        function_name, func_idx
    );

    // Count number of imported functions
    let import_func_count = count_imported_functions(module);
    info!("Module has {} imported functions", import_func_count);

    // First check if the function name is explicitly exported
    if let Some(export) = module.exports.iter().find(|e| e.name == function_name) {
        if matches!(export.kind, ExportKind::Function) {
            // Update the function index to use the export's index
            info!(
                "Found function '{}' as export with index {}",
                function_name, export.index
            );

            if (export.index as usize) < import_func_count {
                // It's a re-export of an imported function
                get_imported_function_param_count(module, export.index as usize)
            } else {
                // It's a regular function, but we need to adjust the index
                get_regular_function_param_count(module, export.index as usize, import_func_count)
            }
        } else {
            info!("Export '{}' is not a function", function_name);
            0
        }
    } else {
        // Function name not found as export, try using the raw function index
        info!(
            "No export found with name '{}', using raw function index {}",
            function_name, func_idx
        );

        if (func_idx as usize) < import_func_count {
            // It's an imported function
            get_imported_function_param_count(module, func_idx as usize)
        } else {
            // It's a regular function with adjusted index
            get_regular_function_param_count(module, func_idx as usize, import_func_count)
        }
    }
}

/// Get the parameter count of an imported function
fn get_imported_function_param_count(module: &Module, import_idx: usize) -> usize {
    if let ExternType::Function(func_type) = &module.imports[import_idx].ty {
        info!("Function is an imported function at index {}", import_idx);
        info!(
            "Function type: params: {}, results: {}",
            func_type.params.len(),
            func_type.results.len()
        );
        func_type.params.len()
    } else {
        info!("Import at index {} is not a function", import_idx);
        0
    }
}

/// Get the parameter count of a regular (non-imported) function
fn get_regular_function_param_count(
    module: &Module,
    func_idx: usize,
    import_func_count: usize,
) -> usize {
    let adjusted_idx = func_idx - import_func_count;
    info!("Function is at adjusted index {}", adjusted_idx);

    if adjusted_idx < module.functions.len() {
        let func = &module.functions[adjusted_idx];
        let func_type = &module.types[func.type_idx as usize];
        info!(
            "Function type_idx: {}, params: {}, results: {}",
            func.type_idx,
            func_type.params.len(),
            func_type.results.len()
        );
        func_type.params.len()
    } else {
        info!(
            "Adjusted index {} is out of bounds (module has {} functions)",
            adjusted_idx,
            module.functions.len()
        );
        0
    }
}

/// Count the number of imported functions in a module
fn count_imported_functions(module: &Module) -> usize {
    module
        .imports
        .iter()
        .filter(|import| matches!(import.ty, ExternType::Function(_)))
        .count()
}

/// Format a list of value types as a string
fn format_value_types(types: &[wrt::ValueType]) -> String {
    types
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Format an extern type as a string description
fn format_extern_type(ty: &ExternType) -> String {
    match ty {
        ExternType::Function(func_type) => {
            let params = format_value_types(&func_type.params);
            let results = format_value_types(&func_type.results);
            format!("Function(params: [{}], results: [{}])", params, results)
        }
        ExternType::Table(table) => {
            format!(
                "Table({:?}, min: {}, max: {:?})",
                table.element_type, table.min, table.max
            )
        }
        ExternType::Memory(mem) => {
            format!("Memory(min: {}, max: {:?})", mem.min, mem.max)
        }
        ExternType::Global(global) => {
            format!(
                "Global({:?}, mutable: {})",
                global.content_type, global.mutable
            )
        }
    }
}

/// Load and execute a WebAssembly Component Model module
fn load_component(
    engine: &mut Engine,
    bytes: &[u8],
    function_name: Option<&str>,
    _file_path: String,
) -> Result<()> {
    // Load the component
    let parse_start = Instant::now();

    // Load the module from binary
    let module = wrt::new_module()
        .load_from_binary(bytes)
        .context("Failed to load WebAssembly component")?;

    let parse_time = parse_start.elapsed();
    info!("Loaded WebAssembly Component in {:?}", parse_time);

    // Store a copy of exports to display available functions
    let mut available_exports = Vec::new();
    for export in &module.exports {
        if matches!(export.kind, ExportKind::Function) {
            available_exports.push(export.name.clone());
        }
    }

    // Parse interface information from the module
    analyze_component_interfaces(&module);

    // Instantiate the module
    let inst_start = Instant::now();
    engine
        .instantiate(module)
        .context("Failed to instantiate component")?;

    let instantiate_time = inst_start.elapsed();
    info!("Component instantiated in {:?}", instantiate_time);

    // Execute the component's function if specified
    if let Some(func_name) = function_name {
        // For component model, we'll use instance index 0
        let instance_idx = 0;

        execute_component_function(engine, instance_idx, func_name)?;
    } else {
        info!("No function specified to call. Use --call <function> to execute a function");
        // List available exports to help the user
        info!("Available exported functions:");
        for name in &available_exports {
            info!("  - {}", name);
        }
    }

    Ok(())
}

/// Analyze and display component interfaces
fn analyze_component_interfaces(module: &Module) {
    info!("Component interfaces:");

    // Examine module imports and exports to identify interfaces
    let function_imports = module
        .imports
        .iter()
        .filter(|import| matches!(import.ty, ExternType::Function(_)))
        .collect::<Vec<_>>();

    // Log the interfaces
    for import in &function_imports {
        let function_name = &import.name;
        info!("  - Import: {}.{}", import.module, function_name);

        // Check if this is a logging interface
        if import.module.contains("logging") || function_name == "log" {
            info!("    Detected logging interface import - will provide implementation");
        }

        // Display function signature
        if let ExternType::Function(func_type) = &import.ty {
            let params = format_value_types(&func_type.params);
            let results = format_value_types(&func_type.results);
            info!(
                "    Function signature: (params: [{}], results: [{}])",
                params, results
            );
        }
    }

    // Display exports as well
    for export in &module.exports {
        info!("  - Export: {}", export.name);

        // Get function details
        if matches!(export.kind, ExportKind::Function) {
            display_component_function_details(
                module,
                &ExportEntry {
                    name: export.name.clone(),
                    kind: export.kind.clone(),
                    index: export.index,
                },
                &function_imports,
            );
        }
    }
}

/// Display details about a component function
fn display_component_function_details(
    module: &Module,
    export: &ExportEntry,
    function_imports: &[&wrt::Import],
) {
    // Find the function details
    let func_idx = export.index as usize;
    let func_count = module.functions.len();

    // Display function details if this is a non-imported function
    let import_func_count = function_imports.len();

    if func_idx >= import_func_count && (func_idx - import_func_count) < func_count {
        let adjusted_idx = func_idx - import_func_count;
        let func = &module.functions[adjusted_idx];
        let func_type = &module.types[func.type_idx as usize];

        let params = format_value_types(&func_type.params);
        let results = format_value_types(&func_type.results);

        info!(
            "    Function signature: (params: [{}], results: [{}])",
            params, results
        );
    }
}

/// Execute a function in a component
fn execute_component_function(
    engine: &mut Engine,
    instance_idx: u32,
    func_name: &str,
) -> Result<()> {
    info!("Executing component function: {}", func_name);

    // Find the function index
    let mut func_idx = 0;
    let mut found = false;

    if (instance_idx as usize) < engine.instances.len() {
        for export in &engine.instances[instance_idx as usize].module.exports {
            if export.name == func_name && matches!(export.kind, ExportKind::Function) {
                func_idx = export.index;
                found = true;
                break;
            }
        }
    }

    if found {
        // Get statistics before execution to measure differences
        let stats_before = engine.stats().clone();

        // Call the function
        let exec_start = Instant::now();
        match engine.execute(instance_idx, func_idx, vec![]) {
            Ok(results) => {
                let execution_time = exec_start.elapsed();

                // Get statistics after execution
                let stats_after = engine.stats();

                // Calculate actual instruction count
                let instructions_executed =
                    stats_after.instructions_executed - stats_before.instructions_executed;
                let fuel_consumed = stats_after.fuel_consumed - stats_before.fuel_consumed;
                let function_calls = stats_after.function_calls - stats_before.function_calls;
                let memory_operations =
                    stats_after.memory_operations - stats_before.memory_operations;

                info!(
                    "Function execution completed in {:?} with results: {:?}",
                    execution_time, results
                );
                // Only show component stats if they're non-zero (to avoid confusion)
                if instructions_executed > 0
                    || fuel_consumed > 0
                    || function_calls > 0
                    || memory_operations > 0
                {
                    info!(
                        "Component statistics: {} instructions, {} fuel units, {} function calls, {} memory operations",
                        instructions_executed, fuel_consumed, function_calls, memory_operations
                    );
                } else {
                    warn!(
                        "Component execution statistics not available - component model support is limited in the runtime"
                    );
                }

                // Print for tests to check
                println!("Function result: {:?}", results);
                Ok(())
            }
            Err(e) => {
                let execution_time = exec_start.elapsed();
                error!(
                    "Function execution failed after {:?}: {}",
                    execution_time, e
                );
                Err(anyhow::anyhow!("Function execution error: {}", e))
            }
        }
    } else {
        warn!("Function '{}' not found in component", func_name);
        Err(anyhow::anyhow!(
            "Function '{}' not found in component",
            func_name
        ))
    }
}

/// Handles a log call from a component by mapping it to the appropriate tracing level
fn handle_component_log(level: &str, message: &str) {
    // Map the level from the component to the appropriate tracing level
    let log_level = LogLevel::from_string_or_default(level);

    // Determine the message prefix based on content
    let prefix = if message.starts_with("[Host function]") || message.starts_with("[") {
        // This is from an imported host function or already has a prefix
        "" // No additional prefix since it already has one
    } else {
        // This is from a component
        "[Component] "
    };

    match log_level {
        LogLevel::Trace => tracing::trace!("{}{}", prefix, message),
        LogLevel::Debug => tracing::debug!("{}{}", prefix, message),
        LogLevel::Info => tracing::info!("{}{}", prefix, message),
        LogLevel::Warn => tracing::warn!("{}{}", prefix, message),
        LogLevel::Error => tracing::error!("{}{}", prefix, message),
        LogLevel::Critical => tracing::error!("{}CRITICAL: {}", prefix, message),
    }

    // Also print to standard output for easier debugging during development - show full message
    println!("[LOG] {}{}: {}", prefix, level, message);
}

/// Display timing information for module execution
fn display_timing_breakdown(timing: &ExecutionTiming) {
    info!("=== Performance Timing ===");
    info!("Total runtime: {:?}", timing.total_time);

    if timing.load_time.as_secs_f64() > 0.0 {
        info!(
            "File loading: {:?} ({:.2}%)",
            timing.load_time,
            (timing.load_time.as_secs_f64() / timing.total_time.as_secs_f64()) * 100.0
        );
    }

    info!(
        "Module parsing: {:?} ({:.2}%)",
        timing.parse_time,
        (timing.parse_time.as_secs_f64() / timing.total_time.as_secs_f64()) * 100.0
    );

    info!(
        "Module instantiation: {:?} ({:.2}%)",
        timing.instantiate_time,
        (timing.instantiate_time.as_secs_f64() / timing.total_time.as_secs_f64()) * 100.0
    );

    info!(
        "Function execution: {:?} ({:.2}%)",
        timing.execution_time,
        (timing.execution_time.as_secs_f64() / timing.total_time.as_secs_f64()) * 100.0
    );

    info!("===========================");
}

/// Displays execution statistics
fn display_execution_stats(engine: &Engine) {
    let stats = engine.stats();

    info!("=== Execution Statistics ===");
    info!("Instructions executed:  {}", stats.instructions_executed);

    // Note about component model statistics
    if stats.instructions_executed <= 1 {
        warn!(
            "Note: WebAssembly Component Model support requires valid core modules in components."
        );
        warn!("      The runtime extracts and executes the core module from component binaries.");
        warn!("      If execution fails, check if the component contains a valid core WebAssembly module.");
    }

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

    // Display time breakdowns if available
    #[cfg(feature = "std")]
    {
        // Calculate total measured time in microseconds
        let total_time = stats.local_global_time_us
            + stats.control_flow_time_us
            + stats.arithmetic_time_us
            + stats.memory_ops_time_us
            + stats.function_call_time_us;

        if total_time > 0 {
            info!("Time breakdown:");
            info!(
                "  Local/Global ops:    {} µs ({:.1}%)",
                stats.local_global_time_us,
                (stats.local_global_time_us as f64 / total_time as f64) * 100.0
            );
            info!(
                "  Control flow:        {} µs ({:.1}%)",
                stats.control_flow_time_us,
                (stats.control_flow_time_us as f64 / total_time as f64) * 100.0
            );
            info!(
                "  Arithmetic ops:      {} µs ({:.1}%)",
                stats.arithmetic_time_us,
                (stats.arithmetic_time_us as f64 / total_time as f64) * 100.0
            );
            info!(
                "  Memory operations:   {} µs ({:.1}%)",
                stats.memory_ops_time_us,
                (stats.memory_ops_time_us as f64 / total_time as f64) * 100.0
            );
            info!(
                "  Function calls:      {} µs ({:.1}%)",
                stats.function_call_time_us,
                (stats.function_call_time_us as f64 / total_time as f64) * 100.0
            );
        }
    }

    info!("===========================");
}
