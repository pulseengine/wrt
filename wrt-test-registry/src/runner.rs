use clap::{Parser, Subcommand};
use colored::Colorize;
use wrt_test_registry::TestRegistry;

/// Command-line interface for running WebAssembly Runtime (WRT) tests.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Filter tests by name (supports partial matching)
    #[arg(short, long)]
    name: Option<String>,

    /// Filter tests by category (supports partial matching)
    #[arg(short, long)]
    category: Option<String>,

    /// Skip tests that require the standard library
    #[arg(long)]
    no_std: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// List all registered tests
    List {
        /// Filter tests by name (supports partial matching)
        #[arg(short, long)]
        name: Option<String>,

        /// Filter tests by category (supports partial matching)
        #[arg(short, long)]
        category: Option<String>,

        /// Show only tests that require the standard library
        #[arg(long)]
        std_only: bool,

        /// Show only tests that don't require the standard library
        #[arg(long)]
        no_std_only: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let registry = TestRegistry::global();

    // Register all compatibility tests
    wrt_test_registry::compatibility::register_compatibility_tests();

    match cli.command {
        Some(Commands::List {
            name,
            category,
            std_only,
            no_std_only,
        }) => {
            list_tests(
                registry,
                name.as_deref(),
                category.as_deref(),
                std_only,
                no_std_only,
            );
        }
        None => {
            // Run tests
            run_tests(
                registry,
                cli.name.as_deref(),
                cli.category.as_deref(),
                !cli.no_std,
                cli.verbose,
            );
        }
    }
}

fn list_tests(
    registry: &TestRegistry,
    name_filter: Option<&str>,
    category_filter: Option<&str>,
    std_only: bool,
    no_std_only: bool,
) {
    let tests = registry.get_tests();
    let mut filtered_tests = Vec::new();

    for test in tests {
        // Apply filters
        if let Some(name) = name_filter {
            if !test.name().contains(name) {
                continue;
            }
        }

        if let Some(category) = category_filter {
            if !test.category().contains(category) {
                continue;
            }
        }

        if std_only && !test.requires_std() {
            continue;
        }

        if no_std_only && test.requires_std() {
            continue;
        }

        filtered_tests.push(test);
    }

    // Print test count
    println!(
        "{} (filtered from {} total)",
        format!("{} tests found", filtered_tests.len())
            .green()
            .bold(),
        registry.count()
    );

    // Group tests by category
    let mut categories = std::collections::HashMap::new();
    for test in filtered_tests {
        categories
            .entry(test.category())
            .or_insert_with(Vec::new)
            .push(test);
    }

    // Print tests by category
    for (category, tests) in categories {
        println!("\n{}", format!("Category: {}", category).blue().bold());
        for test in tests {
            let std_marker = if test.requires_std() {
                format!("[{}]", "std".yellow())
            } else {
                format!("[{}]", "no_std".green())
            };
            println!("  {} {}", test.name(), std_marker);
        }
    }
}

fn run_tests(
    registry: &TestRegistry,
    name_filter: Option<&str>,
    category_filter: Option<&str>,
    allow_std: bool,
    verbose: bool,
) {
    println!(
        "{}",
        "Running WebAssembly Runtime (WRT) tests...".green().bold()
    );

    if let Some(name) = name_filter {
        println!("Filtering by name: {}", name);
    }

    if let Some(category) = category_filter {
        println!("Filtering by category: {}", category);
    }

    if !allow_std {
        println!("Skipping tests that require the standard library");
    }

    println!("\n{}", "Test Results:".underline());
    let failed_count = registry.run_filtered_tests(name_filter, category_filter, allow_std);
    let total_count = registry.count();

    println!("\n{}", "Summary:".bold());
    if failed_count == 0 {
        println!("{}", "All tests passed!".green().bold());
    } else {
        println!("{}", format!("{} tests failed", failed_count).red().bold());
        std::process::exit(1);
    }

    println!(
        "Ran {} tests out of {} total registered tests",
        total_count - failed_count,
        total_count
    );
}

/// Module for testing compatibility between std and no_std environments
#[cfg(feature = "std")]
pub mod compatibility {
    use crate::{TestRegistry, TestResult};

    /// Create a test registry with compatibility tests
    pub fn create_compatibility_test_registry() -> TestRegistry {
        let registry = TestRegistry::new();

        // Register basic compatibility tests
        registry.register(Box::new(MemoryCompatibilityTest));
        registry.register(Box::new(ModuleCompatibilityTest));
        registry.register(Box::new(TypesCompatibilityTest));
        registry.register(Box::new(ErrorHandlingTest));

        registry
    }

    /// Test memory operations in both std and no_std environments
    struct MemoryCompatibilityTest;

    impl crate::TestCase for MemoryCompatibilityTest {
        fn name(&self) -> &'static str {
            "memory_compatibility"
        }

        fn category(&self) -> &'static str {
            "compatibility"
        }

        fn requires_std(&self) -> bool {
            false // Works in both environments
        }

        fn run(&self) -> TestResult {
            use wrt_types::memory::MemoryType;

            // Create a memory instance
            let mem_type = MemoryType::new(1, Some(2), false);
            let memory = wrt::new_memory(mem_type);

            // Verify initial size
            assert_eq_test!(memory.size(), 1, "Initial memory size should be 1 page");
            assert_eq_test!(
                memory.data_size(),
                65536,
                "Initial data size should be 65536 bytes"
            );

            // Test basic memory operations
            let test_data = [1, 2, 3, 4];
            memory.write(100, &test_data).map_err(|e| e.to_string())?;

            let mut read_buffer = [0; 4];
            memory
                .read(100, &mut read_buffer)
                .map_err(|e| e.to_string())?;

            assert_eq_test!(
                read_buffer,
                test_data,
                "Memory read/write should work correctly"
            );

            // Test memory growth
            memory.grow(1).map_err(|e| e.to_string())?;
            assert_eq_test!(
                memory.size(),
                2,
                "Memory size should be 2 pages after growing"
            );

            Ok(())
        }
    }

    /// Test module operations in both std and no_std environments
    struct ModuleCompatibilityTest;

    impl crate::TestCase for ModuleCompatibilityTest {
        fn name(&self) -> &'static str {
            "module_compatibility"
        }

        fn category(&self) -> &'static str {
            "compatibility"
        }

        fn requires_std(&self) -> bool {
            false // Works in both environments
        }

        fn run(&self) -> TestResult {
            // Create a new module
            let module = wrt::new_module().map_err(|e| e.to_string())?;

            // Verify module properties
            assert_eq_test!(
                module.functions().len(),
                0,
                "New module should have 0 functions"
            );
            assert_eq_test!(
                module.imports().len(),
                0,
                "New module should have 0 imports"
            );
            assert_eq_test!(
                module.exports().len(),
                0,
                "New module should have 0 exports"
            );

            Ok(())
        }
    }

    /// Test type system compatibility between std and no_std
    struct TypesCompatibilityTest;

    impl crate::TestCase for TypesCompatibilityTest {
        fn name(&self) -> &'static str {
            "types_compatibility"
        }

        fn category(&self) -> &'static str {
            "compatibility"
        }

        fn requires_std(&self) -> bool {
            false // Works in both environments
        }

        fn run(&self) -> TestResult {
            use wrt_types::values::{ValType, Value};

            // Test value types
            let i32_val = Value::I32(42);
            let i64_val = Value::I64(84);
            let f32_val = Value::F32(42.0);
            let f64_val = Value::F64(84.0);

            assert_eq_test!(
                i32_val.value_type(),
                ValType::I32,
                "I32 value type should be I32"
            );
            assert_eq_test!(
                i64_val.value_type(),
                ValType::I64,
                "I64 value type should be I64"
            );
            assert_eq_test!(
                f32_val.value_type(),
                ValType::F32,
                "F32 value type should be F32"
            );
            assert_eq_test!(
                f64_val.value_type(),
                ValType::F64,
                "F64 value type should be F64"
            );

            Ok(())
        }
    }

    /// Test error handling compatibility between std and no_std
    struct ErrorHandlingTest;

    impl crate::TestCase for ErrorHandlingTest {
        fn name(&self) -> &'static str {
            "error_handling_compatibility"
        }

        fn category(&self) -> &'static str {
            "compatibility"
        }

        fn requires_std(&self) -> bool {
            false // Works in both environments
        }

        fn run(&self) -> TestResult {
            use wrt_error::{kinds, Error};

            // Create an error
            let validation_error = Error::new(kinds::ErrorKind::Validation, 1, Some("test error"));

            // Check error properties
            assert_eq_test!(
                validation_error.kind(),
                kinds::ErrorKind::Validation,
                "Error kind should be Validation"
            );
            assert_eq_test!(validation_error.code(), 1, "Error code should be 1");

            Ok(())
        }
    }
}
