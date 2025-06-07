//! ASIL-Tagged Testing Framework
//!
//! This module provides a comprehensive testing framework with ASIL level tagging,
//! inspired by SCORE's testing methodology. It enables categorization of tests by
//! safety level, platform, and verification requirements.

use wrt_foundation::{
    safety_system::AsilLevel,
    prelude::*,
};
use core::fmt;

/// Test category for organizing test suites
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TestCategory {
    /// Unit tests for individual components
    Unit,
    /// Integration tests for component interactions
    Integration,
    /// System tests for end-to-end functionality
    System,
    /// Performance tests for timing and throughput
    Performance,
    /// Safety tests for ASIL compliance
    Safety,
    /// Security tests for attack resistance
    Security,
    /// Platform-specific tests
    Platform(String),
    /// Memory safety tests
    Memory,
    /// Real-time tests
    RealTime,
    /// Regression tests
    Regression,
}

impl fmt::Display for TestCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestCategory::Unit => write!(f, "unit"),
            TestCategory::Integration => write!(f, "integration"),
            TestCategory::System => write!(f, "system"),
            TestCategory::Performance => write!(f, "performance"),
            TestCategory::Safety => write!(f, "safety"),
            TestCategory::Security => write!(f, "security"),
            TestCategory::Platform(p) => write!(f, "platform-{}", p),
            TestCategory::Memory => write!(f, "memory"),
            TestCategory::RealTime => write!(f, "realtime"),
            TestCategory::Regression => write!(f, "regression"),
        }
    }
}

/// Test priority level
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TestPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Test execution mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestMode {
    /// Standard test execution
    Standard,
    /// Stress test with heavy load
    Stress,
    /// Long-running endurance test
    Endurance,
    /// Fault injection test
    FaultInjection,
    /// Deterministic test for safety verification
    Deterministic,
}

/// Platform constraints for test execution
#[derive(Debug, Clone)]
pub struct PlatformConstraints {
    /// Required platforms for test execution
    pub required_platforms: Vec<String>,
    /// Excluded platforms
    pub excluded_platforms: Vec<String>,
    /// Minimum memory requirement (bytes)
    pub min_memory: Option<usize>,
    /// Maximum execution time (milliseconds)
    pub max_execution_time: Option<u64>,
    /// Required features
    pub required_features: Vec<String>,
}

impl PlatformConstraints {
    pub fn new() -> Self {
        Self {
            required_platforms: Vec::new(),
            excluded_platforms: Vec::new(),
            min_memory: None,
            max_execution_time: None,
            required_features: Vec::new(),
        }
    }
    
    pub fn require_platform(mut self, platform: impl Into<String>) -> Self {
        self.required_platforms.push(platform.into());
        self
    }
    
    pub fn exclude_platform(mut self, platform: impl Into<String>) -> Self {
        self.excluded_platforms.push(platform.into());
        self
    }
    
    pub fn min_memory(mut self, bytes: usize) -> Self {
        self.min_memory = Some(bytes);
        self
    }
    
    pub fn max_time(mut self, ms: u64) -> Self {
        self.max_execution_time = Some(ms);
        self
    }
    
    pub fn require_feature(mut self, feature: impl Into<String>) -> Self {
        self.required_features.push(feature.into());
        self
    }
}

impl Default for PlatformConstraints {
    fn default() -> Self {
        Self::new()
    }
}

/// ASIL-tagged test metadata
#[derive(Debug, Clone)]
pub struct AsilTestMetadata {
    /// Test name/identifier
    pub name: String,
    /// Test description
    pub description: String,
    /// ASIL level this test verifies
    pub asil_level: AsilLevel,
    /// Test category
    pub category: TestCategory,
    /// Test priority
    pub priority: TestPriority,
    /// Test execution mode
    pub mode: TestMode,
    /// Platform constraints
    pub constraints: PlatformConstraints,
    /// Requirements this test verifies
    pub verifies_requirements: Vec<String>,
    /// Tags for filtering
    pub tags: Vec<String>,
    /// Expected test duration (milliseconds)
    pub expected_duration: Option<u64>,
    /// Whether test is deterministic
    pub is_deterministic: bool,
    /// Module/file containing the test
    pub test_module: String,
}

impl AsilTestMetadata {
    pub fn new(name: impl Into<String>, asil_level: AsilLevel) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            asil_level,
            category: TestCategory::Unit,
            priority: TestPriority::Medium,
            mode: TestMode::Standard,
            constraints: PlatformConstraints::default(),
            verifies_requirements: Vec::new(),
            tags: Vec::new(),
            expected_duration: None,
            is_deterministic: false,
            test_module: String::new(),
        }
    }
    
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
    
    pub fn category(mut self, category: TestCategory) -> Self {
        self.category = category;
        self
    }
    
    pub fn priority(mut self, priority: TestPriority) -> Self {
        self.priority = priority;
        self
    }
    
    pub fn mode(mut self, mode: TestMode) -> Self {
        self.mode = mode;
        self
    }
    
    pub fn constraints(mut self, constraints: PlatformConstraints) -> Self {
        self.constraints = constraints;
        self
    }
    
    pub fn verifies(mut self, requirement: impl Into<String>) -> Self {
        self.verifies_requirements.push(requirement.into());
        self
    }
    
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
    
    pub fn expected_duration(mut self, ms: u64) -> Self {
        self.expected_duration = Some(ms);
        self
    }
    
    pub fn deterministic(mut self) -> Self {
        self.is_deterministic = true;
        self
    }
    
    pub fn test_module(mut self, module: impl Into<String>) -> Self {
        self.test_module = module.into();
        self
    }
    
    /// Check if this test should run on the current platform
    pub fn should_run_on_platform(&self, platform: &str) -> bool {
        // Check if platform is excluded
        if self.constraints.excluded_platforms.iter().any(|p| p == platform) {
            return false;
        }
        
        // Check if platform is required (if any requirements specified)
        if !self.constraints.required_platforms.is_empty() {
            return self.constraints.required_platforms.iter().any(|p| p == platform);
        }
        
        true
    }
    
    /// Check if this test matches the given filters
    pub fn matches_filters(&self, filters: &TestFilters) -> bool {
        // ASIL level filter
        if let Some(asil) = filters.asil_level {
            if self.asil_level != asil {
                return false;
            }
        }
        
        // Category filter
        if let Some(ref category) = filters.category {
            if self.category != *category {
                return false;
            }
        }
        
        // Priority filter
        if let Some(ref priority) = filters.priority {
            if self.priority < *priority {
                return false;
            }
        }
        
        // Tag filter
        if !filters.tags.is_empty() {
            if !filters.tags.iter().any(|tag| self.tags.contains(tag)) {
                return false;
            }
        }
        
        // Platform filter
        if let Some(ref platform) = filters.platform {
            if !self.should_run_on_platform(platform) {
                return false;
            }
        }
        
        true
    }
}

/// Test execution filters
#[derive(Debug, Clone, Default)]
pub struct TestFilters {
    pub asil_level: Option<AsilLevel>,
    pub category: Option<TestCategory>,
    pub priority: Option<TestPriority>,
    pub tags: Vec<String>,
    pub platform: Option<String>,
    pub include_deterministic_only: bool,
}

impl TestFilters {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn asil_level(mut self, asil: AsilLevel) -> Self {
        self.asil_level = Some(asil);
        self
    }
    
    pub fn category(mut self, category: TestCategory) -> Self {
        self.category = Some(category);
        self
    }
    
    pub fn priority(mut self, priority: TestPriority) -> Self {
        self.priority = Some(priority);
        self
    }
    
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
    
    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }
    
    pub fn deterministic_only(mut self) -> Self {
        self.include_deterministic_only = true;
        self
    }
}

/// Test registry for ASIL-tagged tests
pub struct AsilTestRegistry {
    tests: Vec<AsilTestMetadata>,
}

impl AsilTestRegistry {
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
        }
    }
    
    /// Register a test with ASIL metadata
    pub fn register_test(&mut self, metadata: AsilTestMetadata) {
        self.tests.push(metadata);
    }
    
    /// Get all tests matching the given filters
    pub fn get_filtered_tests(&self, filters: &TestFilters) -> Vec<&AsilTestMetadata> {
        self.tests.iter()
            .filter(|test| test.matches_filters(filters))
            .filter(|test| !filters.include_deterministic_only || test.is_deterministic)
            .collect()
    }
    
    /// Get tests by ASIL level
    pub fn get_tests_by_asil(&self, asil_level: AsilLevel) -> Vec<&AsilTestMetadata> {
        self.tests.iter()
            .filter(|test| test.asil_level == asil_level)
            .collect()
    }
    
    /// Get tests by category
    pub fn get_tests_by_category(&self, category: TestCategory) -> Vec<&AsilTestMetadata> {
        self.tests.iter()
            .filter(|test| test.category == category)
            .collect()
    }
    
    /// Get tests verifying a specific requirement
    pub fn get_tests_for_requirement(&self, requirement_id: &str) -> Vec<&AsilTestMetadata> {
        self.tests.iter()
            .filter(|test| test.verifies_requirements.iter().any(|req| req == requirement_id))
            .collect()
    }
    
    /// Get all ASIL-D tests (highest priority)
    pub fn get_critical_tests(&self) -> Vec<&AsilTestMetadata> {
        self.get_tests_by_asil(AsilLevel::ASIL_D)
    }
    
    /// Generate test execution plan
    pub fn generate_execution_plan(&self, filters: &TestFilters) -> TestExecutionPlan {
        let filtered_tests = self.get_filtered_tests(filters);
        
        let mut plan = TestExecutionPlan {
            tests: Vec::new(),
            total_estimated_time: 0,
            asil_coverage: std::collections::HashMap::new(),
            requirement_coverage: std::collections::HashMap::new(),
        };
        
        // Sort tests by priority and ASIL level
        let mut sorted_tests = filtered_tests.clone();
        sorted_tests.sort_by(|a, b| {
            // First by ASIL level (higher levels first)
            match b.asil_level.cmp(&a.asil_level) {
                core::cmp::Ordering::Equal => {
                    // Then by priority (higher priorities first)
                    b.priority.cmp(&a.priority)
                }
                other => other,
            }
        });
        
        for test in sorted_tests {
            plan.tests.push(test.clone());
            
            // Add to estimated time
            if let Some(duration) = test.expected_duration {
                plan.total_estimated_time += duration;
            }
            
            // Update ASIL coverage
            *plan.asil_coverage.entry(test.asil_level).or_insert(0) += 1;
            
            // Update requirement coverage
            for req in &test.verifies_requirements {
                *plan.requirement_coverage.entry(req.clone()).or_insert(0) += 1;
            }
        }
        
        plan
    }
}

impl Default for AsilTestRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Test execution plan
#[derive(Debug)]
pub struct TestExecutionPlan {
    pub tests: Vec<AsilTestMetadata>,
    pub total_estimated_time: u64,
    pub asil_coverage: std::collections::HashMap<AsilLevel, usize>,
    pub requirement_coverage: std::collections::HashMap<String, usize>,
}

impl TestExecutionPlan {
    /// Get test count by ASIL level
    pub fn test_count_for_asil(&self, asil_level: AsilLevel) -> usize {
        self.asil_coverage.get(&asil_level).copied().unwrap_or(0)
    }
    
    /// Check if a requirement has test coverage
    pub fn has_requirement_coverage(&self, requirement_id: &str) -> bool {
        self.requirement_coverage.contains_key(requirement_id)
    }
    
    /// Get estimated execution time in a human-readable format
    pub fn estimated_time_formatted(&self) -> String {
        let ms = self.total_estimated_time;
        
        if ms < 1000 {
            format!("{}ms", ms)
        } else if ms < 60_000 {
            format!("{:.1}s", ms as f64 / 1000.0)
        } else if ms < 3_600_000 {
            format!("{:.1}min", ms as f64 / 60_000.0)
        } else {
            format!("{:.1}h", ms as f64 / 3_600_000.0)
        }
    }
}

/// Macros for creating ASIL-tagged tests
#[macro_export]
macro_rules! asil_test {
    (
        name: $name:literal,
        asil: $asil:expr,
        category: $category:expr,
        verifies: [$($req:literal),*],
        $($attr:meta),*
    ) => {
        $(#[$attr])*
        #[test]
        fn $name() {
            // Register test metadata
            let metadata = AsilTestMetadata::new(stringify!($name), $asil)
                .category($category)
                .test_module(module_path!())
                $(
                    .verifies($req)
                )*;
                
            // TODO: Submit to global registry
            // For now, just run the test
            
            // Test implementation goes here
        }
    };
}

/// Macro for safety-critical test suites
#[macro_export]
macro_rules! safety_test_suite {
    (
        suite: $suite_name:ident,
        asil: $asil:expr,
        requirements: [$($req:literal),*],
        tests: {
            $(
                fn $test_name:ident() $test_body:block
            )*
        }
    ) => {
        mod $suite_name {
            use super::*;
            
            $(
                #[test]
                fn $test_name() {
                    let _metadata = AsilTestMetadata::new(stringify!($test_name), $asil)
                        .category(TestCategory::Safety)
                        .deterministic()
                        $(
                            .verifies($req)
                        )*;
                    
                    $test_body
                }
            )*
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_asil_test_metadata_creation() {
        let metadata = AsilTestMetadata::new("test_memory_bounds", AsilLevel::ASIL_C)
            .description("Test memory boundary validation")
            .category(TestCategory::Memory)
            .priority(TestPriority::High)
            .verifies("REQ_MEM_001")
            .tag("memory")
            .tag("bounds")
            .deterministic();
        
        assert_eq!(metadata.name, "test_memory_bounds");
        assert_eq!(metadata.asil_level, AsilLevel::ASIL_C);
        assert_eq!(metadata.category, TestCategory::Memory);
        assert_eq!(metadata.priority, TestPriority::High);
        assert!(metadata.is_deterministic);
        assert!(metadata.verifies_requirements.contains(&"REQ_MEM_001".to_string()));
        assert!(metadata.tags.contains(&"memory".to_string()));
    }
    
    #[test]
    fn test_platform_constraints() {
        let constraints = PlatformConstraints::new()
            .require_platform("linux")
            .exclude_platform("macos")
            .min_memory(1024 * 1024)
            .max_time(5000);
        
        assert!(constraints.required_platforms.contains(&"linux".to_string()));
        assert!(constraints.excluded_platforms.contains(&"macos".to_string()));
        assert_eq!(constraints.min_memory, Some(1024 * 1024));
        assert_eq!(constraints.max_execution_time, Some(5000));
    }
    
    #[test]
    fn test_test_filtering() {
        let mut registry = AsilTestRegistry::new();
        
        let test1 = AsilTestMetadata::new("test1", AsilLevel::ASIL_D)
            .category(TestCategory::Safety)
            .priority(TestPriority::Critical);
        
        let test2 = AsilTestMetadata::new("test2", AsilLevel::ASIL_A)
            .category(TestCategory::Unit)
            .priority(TestPriority::Low);
        
        registry.register_test(test1);
        registry.register_test(test2);
        
        let filters = TestFilters::new()
            .asil_level(AsilLevel::ASIL_D)
            .category(TestCategory::Safety);
        
        let filtered = registry.get_filtered_tests(&filters);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "test1");
    }
    
    #[test]
    fn test_execution_plan_generation() {
        let mut registry = AsilTestRegistry::new();
        
        let test = AsilTestMetadata::new("performance_test", AsilLevel::ASIL_B)
            .category(TestCategory::Performance)
            .expected_duration(1000)
            .verifies("REQ_PERF_001");
        
        registry.register_test(test);
        
        let filters = TestFilters::new();
        let plan = registry.generate_execution_plan(&filters);
        
        assert_eq!(plan.tests.len(), 1);
        assert_eq!(plan.total_estimated_time, 1000);
        assert_eq!(plan.test_count_for_asil(AsilLevel::ASIL_B), 1);
        assert!(plan.has_requirement_coverage("REQ_PERF_001"));
    }
}