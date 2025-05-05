// Test registry for WebAssembly Runtime Testing
//
// This module provides a unified testing framework for the WebAssembly Runtime.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

// Conditional imports for std/no_std support
#[cfg(feature = "std")]
use std::{
    boxed::Box,
    string::String,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
    vec::Vec,
};

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String, vec::Vec};

// Import OnceCell for the global registry
#[cfg(feature = "std")]
use once_cell::sync::OnceCell;

#[cfg(not(feature = "std"))]
use core::cell::OnceCell;

#[cfg(feature = "std")]
use std::println;

/// Result type for test functions.
pub type TestResult = Result<(), String>;

/// Trait that all test cases must implement.
pub trait TestCase: Send + Sync {
    /// The name of the test case.
    fn name(&self) -> &'static str;

    /// The category of the test (e.g., "decoder", "runtime", etc.).
    fn category(&self) -> &'static str;

    /// Whether this test requires the standard library.
    fn requires_std(&self) -> bool;

    /// Run the test case.
    fn run(&self) -> TestResult;
}

/// The test registry that stores all registered tests.
pub struct TestRegistry {
    /// The list of all registered tests.
    #[cfg(feature = "std")]
    tests: Mutex<Vec<Box<dyn TestCase>>>,

    /// The list of all registered tests for no_std environments.
    #[cfg(not(feature = "std"))]
    tests: OnceCell<Vec<Box<dyn TestCase>>>,

    /// The number of registered tests.
    count: AtomicUsize,
}

impl Default for TestRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TestRegistry {
    /// Create a new, empty test registry.
    pub const fn new() -> Self {
        #[cfg(feature = "std")]
        {
            Self {
                tests: Mutex::new(Vec::new()),
                count: AtomicUsize::new(0),
            }
        }

        #[cfg(not(feature = "std"))]
        {
            Self {
                tests: OnceCell::new(),
                count: AtomicUsize::new(0),
            }
        }
    }

    /// Get the global test registry instance.
    pub fn global() -> &'static Self {
        static REGISTRY: OnceCell<TestRegistry> = OnceCell::new();
        REGISTRY.get_or_init(TestRegistry::new)
    }

    /// Register a new test case.
    pub fn register(&self, test: Box<dyn TestCase>) {
        #[cfg(feature = "std")]
        {
            self.tests.lock().unwrap().push(test);
        }

        #[cfg(not(feature = "std"))]
        {
            // For no_std, we can only register tests during initialization
            if self.tests.get().is_none() {
                let mut tests = Vec::new();
                tests.push(test);
                let _ = self.tests.set(tests);
            } else {
                // Ideally, this would panic, but we can't do that in no_std
                // Just silently ignore if we try to register after initialization
            }
        }

        self.count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get the number of registered tests.
    pub fn count(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    /// Execute a function with all the registered tests.
    /// This avoids the need to clone the tests.
    #[cfg(feature = "std")]
    pub fn with_tests<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Box<dyn TestCase>]) -> R,
    {
        let guard = self.tests.lock().unwrap();
        f(&guard)
    }

    /// Run all registered tests that match the given filters.
    ///
    /// # Arguments
    ///
    /// * `name_filter` - Optional filter for test names.
    /// * `category_filter` - Optional filter for test categories.
    /// * `require_std` - Whether to run tests that require the standard library.
    ///
    /// # Returns
    ///
    /// The number of failed tests.
    #[cfg(feature = "std")]
    pub fn run_filtered_tests(
        &self,
        name_filter: Option<&str>,
        category_filter: Option<&str>,
        require_std: bool,
    ) -> usize {
        self.with_tests(|tests| {
            let mut failed_count = 0;

            for test in tests.iter() {
                // Skip tests that don't match the filters
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

                // Skip tests that require std if we're not in a std environment
                if test.requires_std() && !require_std {
                    println!("Skipping '{}': requires std library", test.name());
                    continue;
                }

                // Run the test
                print!("Running test '{}' ({}): ", test.name(), test.category());
                match test.run() {
                    Ok(()) => println!("PASSED"),
                    Err(err) => {
                        println!("FAILED");
                        println!("  Error: {}", err);
                        failed_count += 1;
                    }
                }
            }

            failed_count
        })
    }

    /// Run all registered tests.
    ///
    /// # Returns
    ///
    /// The number of failed tests.
    #[cfg(feature = "std")]
    pub fn run_all_tests(&self) -> usize {
        self.run_filtered_tests(None, None, true)
    }
}

/// Register a test case with the global registry.
#[macro_export]
macro_rules! register_test {
    ($name:expr, $category:expr, $requires_std:expr, $body:expr) => {
        struct RegisteredTest {
            name: &'static str,
            category: &'static str,
            requires_std: bool,
            body: fn() -> $crate::TestResult,
        }

        impl $crate::TestCase for RegisteredTest {
            fn name(&self) -> &'static str {
                self.name
            }

            fn category(&self) -> &'static str {
                self.category
            }

            fn requires_std(&self) -> bool {
                self.requires_std
            }

            fn run(&self) -> $crate::TestResult {
                (self.body)()
            }
        }

        #[ctor::ctor]
        fn register_test() {
            let test = RegisteredTest {
                name: $name,
                category: $category,
                requires_std: $requires_std,
                body: $body,
            };

            $crate::TestRegistry::global().register(Box::new(test));
        }
    };
}

/// Assert that a condition is true, with a custom error message.
#[macro_export]
macro_rules! assert_test {
    ($condition:expr, $message:expr) => {
        if !$condition {
            return Err(format!("Assertion failed: {}", $message));
        }
    };
    ($condition:expr) => {
        if !$condition {
            return Err(format!("Assertion failed: {}", stringify!($condition)));
        }
    };
}

/// Assert that two values are equal, with a custom error message.
#[macro_export]
macro_rules! assert_eq_test {
    ($left:expr, $right:expr, $message:expr) => {
        if $left != $right {
            return Err(format!(
                "Assertion failed: {} != {} - {}",
                stringify!($left),
                stringify!($right),
                $message
            ));
        }
    };
    ($left:expr, $right:expr) => {
        if $left != $right {
            return Err(format!(
                "Assertion failed: {} != {}, got {:?} and {:?}",
                stringify!($left),
                stringify!($right),
                $left,
                $right
            ));
        }
    };
}

// Define modules
pub mod compatibility;

// Create a macro for creating test cases
#[macro_export]
macro_rules! test_case {
    (name: $name:expr, features: [$($feature:expr),*], test_fn: $test_fn:expr, description: $desc:expr) => {
        Box::new(crate::TestCaseImpl {
            name: $name,
            category: "compatibility",
            requires_std: false, // Default to working in both environments
            features: vec![$($feature.to_string()),*],
            test_fn: Box::new($test_fn),
            description: $desc,
        })
    };
    (name: $name:expr, category: $category:expr, test_fn: $test_fn:expr, description: $desc:expr) => {
        Box::new(crate::TestCaseImpl {
            name: $name,
            category: $category,
            requires_std: false, // Default to working in both environments
            features: vec!["std".to_string(), "no_std".to_string()],
            test_fn: Box::new($test_fn),
            description: $desc,
        })
    };
    (name: $name:expr, category: $category:expr, requires_std: $requires_std:expr, test_fn: $test_fn:expr, description: $desc:expr) => {
        Box::new(crate::TestCaseImpl {
            name: $name,
            category: $category,
            requires_std: $requires_std,
            features: if $requires_std { vec!["std".to_string()] } else { vec!["std".to_string(), "no_std".to_string()] },
            test_fn: Box::new($test_fn),
            description: $desc,
        })
    };
}

// Define a test configuration struct
pub struct TestConfig {
    /// Whether the test is running in std mode
    pub is_std: bool,
    /// Current feature set enabled
    pub features: Vec<String>,
    /// Optional test parameters
    #[cfg(feature = "std")]
    pub params: std::collections::HashMap<String, String>,
    /// Optional test parameters for no_std
    #[cfg(not(feature = "std"))]
    pub params: alloc::collections::BTreeMap<String, String>,
}

impl TestConfig {
    /// Create a new TestConfig
    pub fn new(is_std: bool, features: Vec<String>) -> Self {
        #[cfg(feature = "std")]
        let params = std::collections::HashMap::new();

        #[cfg(not(feature = "std"))]
        let params = alloc::collections::BTreeMap::new();

        Self {
            is_std,
            features,
            params,
        }
    }

    /// Check if a feature is enabled
    pub fn has_feature(&self, feature: &str) -> bool {
        self.features.iter().any(|f| f == feature)
    }

    /// Get a parameter value
    pub fn get_param(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }
}

/// The concrete implementation of a test case
pub struct TestCaseImpl {
    /// The name of the test case
    pub name: &'static str,
    /// The category of the test
    pub category: &'static str,
    /// Whether this test requires the standard library
    pub requires_std: bool,
    /// Which features this test supports
    pub features: Vec<String>,
    /// The test function to run
    pub test_fn: Box<dyn Fn(&TestConfig) -> TestResult + Send + Sync>,
    /// Description of the test
    pub description: &'static str,
}

impl TestCase for TestCaseImpl {
    fn name(&self) -> &'static str {
        self.name
    }

    fn category(&self) -> &'static str {
        self.category
    }

    fn requires_std(&self) -> bool {
        self.requires_std
    }

    fn run(&self) -> TestResult {
        // Create a test configuration
        #[cfg(feature = "std")]
        let config = TestConfig::new(true, self.features.clone());

        #[cfg(not(feature = "std"))]
        let config = TestConfig::new(false, self.features.clone());

        // Run the test function
        (self.test_fn)(&config)
    }
}
