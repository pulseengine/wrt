//! Budget visualization and debug dump functionality
//!
//! This module provides tools for visualizing memory budget usage
//! and generating debug dumps for troubleshooting memory issues.

#![cfg_attr(not(feature = "std"), no_std)]

use crate::{WrtResult, Error, ErrorCategory, codes};
use crate::budget_aware_provider::{
    BudgetAwareProviderFactory, CrateId, CrateAllocationStats, SharedPoolStats
};
use crate::memory_analysis::{MemorySnapshot, MemoryHealthReport};
use core::fmt::{self, Write as FmtWrite};

#[cfg(any(feature = "std", feature = "alloc"))]
use alloc::{string::String, vec::Vec, format};

/// Visualization output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualizationFormat {
    /// ASCII text visualization
    Ascii,
    /// JSON data format
    Json,
    /// CSV data format  
    Csv,
    /// HTML report with charts
    Html,
    /// Markdown report
    Markdown,
}

/// Budget visualization configuration
#[derive(Debug, Clone)]
pub struct VisualizationConfig {
    /// Output format
    pub format: VisualizationFormat,
    /// Include detailed crate breakdown
    pub include_crate_details: bool,
    /// Include shared pool information
    pub include_shared_pool: bool,
    /// Include historical data if available
    pub include_history: bool,
    /// Maximum width for ASCII charts
    pub chart_width: usize,
    /// Show utilization percentages
    pub show_percentages: bool,
    /// Color output (terminal colors)
    pub use_colors: bool,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            format: VisualizationFormat::Ascii,
            include_crate_details: true,
            include_shared_pool: true,
            include_history: false,
            chart_width: 60,
            show_percentages: true,
            use_colors: false,
        }
    }
}

/// Memory budget visualizer
pub struct BudgetVisualizer;

impl BudgetVisualizer {
    /// Generate a visualization of current memory budget usage
    pub fn generate_visualization(config: VisualizationConfig) -> WrtResult<String> {
        match config.format {
            VisualizationFormat::Ascii => Self::generate_ascii_visualization(config),
            VisualizationFormat::Json => Self::generate_json_visualization(config),
            VisualizationFormat::Csv => Self::generate_csv_visualization(config),
            VisualizationFormat::Html => Self::generate_html_visualization(config),
            VisualizationFormat::Markdown => Self::generate_markdown_visualization(config),
        }
    }

    /// Generate ASCII bar chart visualization
    fn generate_ascii_visualization(config: VisualizationConfig) -> WrtResult<String> {
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::RUNTIME_ERROR,
                "ASCII visualization requires alloc feature"
            ));
        }

        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            let mut output = String::new();
            
            // Header
            output.push_str("WRT Memory Budget Visualization\n");
            output.push_str("===============================\n\n");

            // Global statistics
            let global_stats = BudgetAwareProviderFactory::get_global_stats()?;
            output.push_str(&format!(
                "Total Active Providers: {}\n",
                global_stats.total_active_providers
            ));
            output.push_str(&format!(
                "Total Allocated Memory: {}KB\n\n",
                global_stats.total_allocated_memory / 1024
            ));

            // Crate-by-crate breakdown
            if config.include_crate_details {
                output.push_str("Crate Budget Utilization:\n");
                output.push_str("-------------------------\n");

                let crates = [
                    CrateId::Foundation, CrateId::Runtime, CrateId::Component,
                    CrateId::Decoder, CrateId::Format, CrateId::Host,
                    CrateId::Debug, CrateId::Platform, CrateId::Instructions,
                    CrateId::Intercept, CrateId::Sync, CrateId::Math,
                    CrateId::Logging, CrateId::Panic,
                ];

                for &crate_id in &crates {
                    if let Ok(stats) = BudgetAwareProviderFactory::get_crate_stats(crate_id) {
                        let bar = Self::create_ascii_bar(
                            stats.utilization_percent,
                            config.chart_width,
                            config.use_colors
                        );
                        
                        output.push_str(&format!(
                            "{:12} [{}] {}%",
                            format!("{:?}", crate_id),
                            bar,
                            stats.utilization_percent
                        ));
                        
                        if config.show_percentages {
                            output.push_str(&format!(
                                " ({}/{}KB)",
                                stats.allocated_bytes / 1024,
                                stats.budget_bytes / 1024
                            ));
                        }
                        output.push('\n');
                    }
                }
                output.push('\n');
            }

            // Shared pool information
            if config.include_shared_pool {
                if let Ok(shared_stats) = BudgetAwareProviderFactory::get_shared_pool_stats() {
                    output.push_str("Shared Pool Status:\n");
                    output.push_str("-------------------\n");
                    
                    let pool_utilization = if shared_stats.total_budget > 0 {
                        (shared_stats.allocated * 100) / shared_stats.total_budget
                    } else {
                        0
                    };
                    
                    let bar = Self::create_ascii_bar(
                        pool_utilization,
                        config.chart_width,
                        config.use_colors
                    );
                    
                    output.push_str(&format!(
                        "Pool Usage   [{}] {}%\n",
                        bar,
                        pool_utilization
                    ));
                    
                    output.push_str(&format!(
                        "Available Providers: 4K:{} 16K:{} 64K:{} 256K:{} 1M:{}\n",
                        shared_stats.available_4k,
                        shared_stats.available_16k,
                        shared_stats.available_64k,
                        shared_stats.available_256k,
                        shared_stats.available_1m
                    ));
                    output.push('\n');
                }
            }

            // Memory health assessment
            if let Ok(health) = crate::memory_analysis::MemoryAnalyzer::generate_health_report() {
                output.push_str("Memory Health:\n");
                output.push_str("--------------\n");
                output.push_str(&format!("Health Score: {}/100\n", health.health_score));
                output.push_str(&format!("Risk Level: {:?}\n", health.risk_level));
                if health.critical_issue_count > 0 {
                    output.push_str(&format!("Critical Issues: {}\n", health.critical_issue_count));
                }
                output.push('\n');
            }

            Ok(output)
        }
    }

    /// Create ASCII progress bar
    fn create_ascii_bar(percentage: usize, width: usize, use_colors: bool) -> String {
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            "█".repeat(percentage.min(100) * width / 100)
        }

        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            let filled = (percentage.min(100) * width) / 100;
            let empty = width - filled;
            
            let bar_char = if use_colors {
                match percentage {
                    0..=50 => "█",      // Green
                    51..=80 => "█",     // Yellow  
                    _ => "█",           // Red
                }
            } else {
                "█"
            };
            
            format!("{}{}", 
                bar_char.repeat(filled),
                "░".repeat(empty)
            )
        }
    }

    /// Generate JSON visualization data
    fn generate_json_visualization(config: VisualizationConfig) -> WrtResult<String> {
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::RUNTIME_ERROR,
                "JSON visualization requires alloc feature"
            ));
        }

        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            let mut json = String::from("{\n");
            
            // Timestamp
            json.push_str("  \"timestamp\": \"");
            json.push_str(&Self::get_timestamp());
            json.push_str("\",\n");
            
            // Global stats
            let global_stats = BudgetAwareProviderFactory::get_global_stats()?;
            json.push_str("  \"global_stats\": {\n");
            json.push_str(&format!("    \"total_active_providers\": {},\n", global_stats.total_active_providers));
            json.push_str(&format!("    \"total_allocated_memory\": {}\n", global_stats.total_allocated_memory));
            json.push_str("  },\n");
            
            // Crate details
            if config.include_crate_details {
                json.push_str("  \"crates\": [\n");
                
                let crates = [
                    CrateId::Foundation, CrateId::Runtime, CrateId::Component,
                    CrateId::Decoder, CrateId::Format, CrateId::Host,
                ];
                
                for (i, &crate_id) in crates.iter().enumerate() {
                    if let Ok(stats) = BudgetAwareProviderFactory::get_crate_stats(crate_id) {
                        json.push_str("    {\n");
                        json.push_str(&format!("      \"name\": \"{:?}\",\n", crate_id));
                        json.push_str(&format!("      \"allocated_bytes\": {},\n", stats.allocated_bytes));
                        json.push_str(&format!("      \"budget_bytes\": {},\n", stats.budget_bytes));
                        json.push_str(&format!("      \"utilization_percent\": {},\n", stats.utilization_percent));
                        json.push_str(&format!("      \"provider_count\": {}\n", stats.provider_count));
                        json.push_str("    }");
                        if i < crates.len() - 1 {
                            json.push(',');
                        }
                        json.push('\n');
                    }
                }
                json.push_str("  ],\n");
            }
            
            // Shared pool
            if config.include_shared_pool {
                if let Ok(shared_stats) = BudgetAwareProviderFactory::get_shared_pool_stats() {
                    json.push_str("  \"shared_pool\": {\n");
                    json.push_str(&format!("    \"total_budget\": {},\n", shared_stats.total_budget));
                    json.push_str(&format!("    \"allocated\": {},\n", shared_stats.allocated));
                    json.push_str(&format!("    \"available_4k\": {},\n", shared_stats.available_4k));
                    json.push_str(&format!("    \"available_16k\": {},\n", shared_stats.available_16k));
                    json.push_str(&format!("    \"available_64k\": {},\n", shared_stats.available_64k));
                    json.push_str(&format!("    \"available_256k\": {},\n", shared_stats.available_256k));
                    json.push_str(&format!("    \"available_1m\": {}\n", shared_stats.available_1m));
                    json.push_str("  }\n");
                } else {
                    json.push_str("  \"shared_pool\": null\n");
                }
            } else {
                // Remove trailing comma
                if json.ends_with(",\n") {
                    json.truncate(json.len() - 2);
                    json.push('\n');
                }
            }
            
            json.push('}');
            Ok(json)
        }
    }

    /// Generate CSV visualization data
    fn generate_csv_visualization(_config: VisualizationConfig) -> WrtResult<String> {
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::RUNTIME_ERROR,
                "CSV visualization requires alloc feature"
            ));
        }

        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            let mut csv = String::new();
            
            // Header
            csv.push_str("timestamp,crate,allocated_bytes,budget_bytes,utilization_percent,provider_count\n");
            
            let timestamp = Self::get_timestamp();
            let crates = [
                CrateId::Foundation, CrateId::Runtime, CrateId::Component,
                CrateId::Decoder, CrateId::Format, CrateId::Host,
            ];
            
            for &crate_id in &crates {
                if let Ok(stats) = BudgetAwareProviderFactory::get_crate_stats(crate_id) {
                    csv.push_str(&format!(
                        "{},{:?},{},{},{},{}\n",
                        timestamp,
                        crate_id,
                        stats.allocated_bytes,
                        stats.budget_bytes,
                        stats.utilization_percent,
                        stats.provider_count
                    ));
                }
            }
            
            Ok(csv)
        }
    }

    /// Generate HTML visualization with charts
    fn generate_html_visualization(config: VisualizationConfig) -> WrtResult<String> {
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::RUNTIME_ERROR,
                "HTML visualization requires alloc feature"
            ));
        }

        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            let mut html = String::new();
            
            // HTML header
            html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
            html.push_str("<title>WRT Memory Budget Report</title>\n");
            html.push_str("<style>\n");
            html.push_str("body { font-family: Arial, sans-serif; margin: 20px; }\n");
            html.push_str(".bar { height: 20px; background: #ddd; margin: 5px 0; }\n");
            html.push_str(".fill { height: 100%; background: linear-gradient(90deg, #4CAF50, #FFC107, #F44336); }\n");
            html.push_str(".stats { display: flex; gap: 20px; }\n");
            html.push_str(".card { border: 1px solid #ddd; padding: 15px; border-radius: 5px; }\n");
            html.push_str("</style>\n</head>\n<body>\n");
            
            // Title
            html.push_str("<h1>WRT Memory Budget Report</h1>\n");
            html.push_str(&format!("<p>Generated: {}</p>\n", Self::get_timestamp()));
            
            // Global stats
            let global_stats = BudgetAwareProviderFactory::get_global_stats()?;
            html.push_str("<div class=\"stats\">\n");
            html.push_str(&format!(
                "<div class=\"card\"><h3>Active Providers</h3><p>{}</p></div>\n",
                global_stats.total_active_providers
            ));
            html.push_str(&format!(
                "<div class=\"card\"><h3>Total Memory</h3><p>{}KB</p></div>\n",
                global_stats.total_allocated_memory / 1024
            ));
            html.push_str("</div>\n");
            
            // Crate utilization charts
            if config.include_crate_details {
                html.push_str("<h2>Crate Memory Utilization</h2>\n");
                
                let crates = [CrateId::Foundation, CrateId::Runtime, CrateId::Component];
                for &crate_id in &crates {
                    if let Ok(stats) = BudgetAwareProviderFactory::get_crate_stats(crate_id) {
                        html.push_str(&format!("<h3>{:?}</h3>\n", crate_id));
                        html.push_str("<div class=\"bar\">\n");
                        html.push_str(&format!(
                            "<div class=\"fill\" style=\"width: {}%;\"></div>\n",
                            stats.utilization_percent
                        ));
                        html.push_str("</div>\n");
                        html.push_str(&format!(
                            "<p>{}% ({}/{}KB)</p>\n",
                            stats.utilization_percent,
                            stats.allocated_bytes / 1024,
                            stats.budget_bytes / 1024
                        ));
                    }
                }
            }
            
            html.push_str("</body>\n</html>");
            Ok(html)
        }
    }

    /// Generate Markdown visualization
    fn generate_markdown_visualization(config: VisualizationConfig) -> WrtResult<String> {
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::RUNTIME_ERROR,
                "Markdown visualization requires alloc feature"
            ));
        }

        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            let mut md = String::new();
            
            // Title and metadata
            md.push_str("# WRT Memory Budget Report\n\n");
            md.push_str(&format!("**Generated:** {}\n\n", Self::get_timestamp()));
            
            // Global statistics
            let global_stats = BudgetAwareProviderFactory::get_global_stats()?;
            md.push_str("## Global Statistics\n\n");
            md.push_str(&format!("- **Active Providers:** {}\n", global_stats.total_active_providers));
            md.push_str(&format!("- **Total Allocated Memory:** {}KB\n\n", global_stats.total_allocated_memory / 1024));
            
            // Crate breakdown table
            if config.include_crate_details {
                md.push_str("## Crate Memory Usage\n\n");
                md.push_str("| Crate | Allocated | Budget | Utilization | Providers |\n");
                md.push_str("|-------|-----------|--------|-------------|----------|\n");
                
                let crates = [
                    CrateId::Foundation, CrateId::Runtime, CrateId::Component,
                    CrateId::Decoder, CrateId::Format, CrateId::Host,
                ];
                
                for &crate_id in &crates {
                    if let Ok(stats) = BudgetAwareProviderFactory::get_crate_stats(crate_id) {
                        md.push_str(&format!(
                            "| {:?} | {}KB | {}KB | {}% | {} |\n",
                            crate_id,
                            stats.allocated_bytes / 1024,
                            stats.budget_bytes / 1024,
                            stats.utilization_percent,
                            stats.provider_count
                        ));
                    }
                }
                md.push('\n');
            }
            
            // Shared pool information
            if config.include_shared_pool {
                if let Ok(shared_stats) = BudgetAwareProviderFactory::get_shared_pool_stats() {
                    md.push_str("## Shared Pool Status\n\n");
                    md.push_str(&format!("- **Total Budget:** {}KB\n", shared_stats.total_budget / 1024));
                    md.push_str(&format!("- **Allocated:** {}KB\n", shared_stats.allocated / 1024));
                    md.push_str(&format!("- **Available 4K Providers:** {}\n", shared_stats.available_4k));
                    md.push_str(&format!("- **Available 16K Providers:** {}\n", shared_stats.available_16k));
                    md.push_str(&format!("- **Available 64K Providers:** {}\n", shared_stats.available_64k));
                    md.push('\n');
                }
            }
            
            Ok(md)
        }
    }

    /// Get current timestamp (simplified for no_std compatibility)
    fn get_timestamp() -> String {
        #[cfg(feature = "std")]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(duration) => format!("{}", duration.as_secs()),
                Err(_) => "unknown".to_string(),
            }
        }

        #[cfg(not(feature = "std"))]
        {
            "no_std_timestamp".to_string()
        }
    }
}

/// Debug dump functionality
pub struct DebugDumper;

impl DebugDumper {
    /// Generate comprehensive debug dump
    pub fn generate_debug_dump() -> WrtResult<String> {
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            return Err(Error::new(
                ErrorCategory::Runtime,
                codes::RUNTIME_ERROR,
                "Debug dump requires alloc feature"
            ));
        }

        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            let mut dump = String::new();
            
            dump.push_str("WRT MEMORY DEBUG DUMP\n");
            dump.push_str("====================\n\n");
            
            // System information
            dump.push_str("SYSTEM INFORMATION:\n");
            dump.push_str("-------------------\n");
            dump.push_str(&format!("Timestamp: {}\n", BudgetVisualizer::get_timestamp()));
            
            if let Ok(config) = crate::memory_system_initializer::get_global_memory_config() {
                dump.push_str(&format!("Total Budget: {}KB\n", config.wrt_memory_budget / 1024));
                dump.push_str(&format!("Enforcement Level: {:?}\n", config.enforcement_level));
                dump.push_str(&format!("Platform Type: {:?}\n", config.capabilities.platform_type));
            }
            dump.push('\n');
            
            // Global statistics
            dump.push_str("GLOBAL MEMORY STATISTICS:\n");
            dump.push_str("------------------------\n");
            let global_stats = BudgetAwareProviderFactory::get_global_stats()?;
            dump.push_str(&format!("Active Providers: {}\n", global_stats.total_active_providers));
            dump.push_str(&format!("Total Allocated: {}KB\n", global_stats.total_allocated_memory / 1024));
            dump.push('\n');
            
            // Per-crate detailed breakdown
            dump.push_str("PER-CRATE BREAKDOWN:\n");
            dump.push_str("-------------------\n");
            
            let all_crates = [
                CrateId::Foundation, CrateId::Runtime, CrateId::Component,
                CrateId::Decoder, CrateId::Format, CrateId::Host,
                CrateId::Debug, CrateId::Platform, CrateId::Instructions,
                CrateId::Intercept, CrateId::Sync, CrateId::Math,
                CrateId::Logging, CrateId::Panic,
            ];
            
            for &crate_id in &all_crates {
                if let Ok(stats) = BudgetAwareProviderFactory::get_crate_stats(crate_id) {
                    dump.push_str(&format!("{:?}:\n", crate_id));
                    dump.push_str(&format!("  Allocated: {}KB\n", stats.allocated_bytes / 1024));
                    dump.push_str(&format!("  Budget: {}KB\n", stats.budget_bytes / 1024));
                    dump.push_str(&format!("  Peak: {}KB\n", stats.peak_bytes / 1024));
                    dump.push_str(&format!("  Utilization: {}%\n", stats.utilization_percent));
                    dump.push_str(&format!("  Providers: {}\n", stats.provider_count));
                    dump.push('\n');
                }
            }
            
            // Shared pool details
            dump.push_str("SHARED POOL DETAILS:\n");
            dump.push_str("-------------------\n");
            if let Ok(shared_stats) = BudgetAwareProviderFactory::get_shared_pool_stats() {
                dump.push_str(&format!("Total Budget: {}KB\n", shared_stats.total_budget / 1024));
                dump.push_str(&format!("Allocated: {}KB\n", shared_stats.allocated / 1024));
                dump.push_str(&format!("Free: {}KB\n", (shared_stats.total_budget - shared_stats.allocated) / 1024));
                dump.push_str("Available Providers by Size:\n");
                dump.push_str(&format!("  4KB: {}\n", shared_stats.available_4k));
                dump.push_str(&format!("  16KB: {}\n", shared_stats.available_16k));
                dump.push_str(&format!("  64KB: {}\n", shared_stats.available_64k));
                dump.push_str(&format!("  256KB: {}\n", shared_stats.available_256k));
                dump.push_str(&format!("  1MB: {}\n", shared_stats.available_1m));
            } else {
                dump.push_str("Shared pool not initialized\n");
            }
            dump.push('\n');
            
            // Memory health analysis
            dump.push_str("MEMORY HEALTH ANALYSIS:\n");
            dump.push_str("----------------------\n");
            if let Ok(health) = crate::memory_analysis::MemoryAnalyzer::generate_health_report() {
                dump.push_str(&format!("Health Score: {}/100\n", health.health_score));
                dump.push_str(&format!("Risk Level: {:?}\n", health.risk_level));
                dump.push_str(&format!("Critical Issues: {}\n", health.critical_issue_count));
                dump.push_str(&format!("Total Issues: {}\n", health.critical_issue_count + health.warning_count));
                
                if health.critical_issue_count > 0 || health.warning_count > 0 {
                    dump.push_str("\nIdentified Issues:\n");
                    
                    // Add critical issues
                    for issue in &health.critical_issues {
                        if let Some(issue) = issue {
                            dump.push_str(&format!("- CRITICAL: {:?}\n", issue));
                        }
                    }
                    
                    // Add warnings
                    for warning in &health.warnings {
                        if let Some(warning) = warning {
                            dump.push_str(&format!("- WARNING: {:?}\n", warning));
                        }
                    }
                }
            } else {
                dump.push_str("Memory health analysis not available\n");
            }
            dump.push('\n');
            
            // Recommendations
            #[cfg(any(feature = "std", feature = "alloc"))]
            if let Ok(recommendations) = BudgetAwareProviderFactory::get_recommendations() {
                if !recommendations.is_empty() {
                    dump.push_str("OPTIMIZATION RECOMMENDATIONS:\n");
                    dump.push_str("----------------------------\n");
                    for rec in &recommendations {
                        dump.push_str(&format!(
                            "- {}: {} (Impact: {}%, Difficulty: {})\n",
                            rec.issue_type,
                            rec.description,
                            rec.estimated_impact,
                            rec.difficulty
                        ));
                    }
                    dump.push('\n');
                }
            }
            
            dump.push_str("END OF DEBUG DUMP\n");
            Ok(dump)
        }
    }

    /// Generate memory snapshot for point-in-time analysis
    pub fn capture_memory_snapshot() -> WrtResult<MemorySnapshot> {
        crate::memory_analysis::MemoryAnalyzer::capture_snapshot()
    }

    /// Compare two memory snapshots to show changes
    pub fn compare_snapshots(before: &MemorySnapshot, after: &MemorySnapshot) -> String {
        #[cfg(not(any(feature = "std", feature = "alloc")))]
        {
            "Snapshot comparison requires alloc feature".to_string()
        }

        #[cfg(any(feature = "std", feature = "alloc"))]
        {
            let mut comparison = String::new();
            
            comparison.push_str("MEMORY SNAPSHOT COMPARISON\n");
            comparison.push_str("=========================\n\n");
            
            // Overall changes
            let total_change = after.total_allocated as i64 - before.total_allocated as i64;
            comparison.push_str(&format!(
                "Total Memory Change: {:+}KB\n",
                total_change / 1024
            ));
            
            comparison.push_str(&format!(
                "Before: {}KB, After: {}KB\n\n",
                before.total_allocated / 1024,
                after.total_allocated / 1024
            ));
            
            // Per-crate changes
            comparison.push_str("Per-Crate Changes:\n");
            comparison.push_str("------------------\n");
            
            // Compare crate usage (simplified)
            for (crate_id, before_detail) in &before.crate_usage.by_crate {
                if let Some((_, after_detail)) = after.crate_usage.by_crate.iter()
                    .find(|(id, _)| id == crate_id) {
                    
                    let change = after_detail.allocated as i64 - before_detail.allocated as i64;
                    if change != 0 {
                        comparison.push_str(&format!(
                            "{:?}: {:+}KB\n",
                            crate_id,
                            change / 1024
                        ));
                    }
                }
            }
            
            comparison
        }
    }
}

/// Convenience functions for quick visualization
pub fn quick_ascii_dump() -> WrtResult<String> {
    BudgetVisualizer::generate_visualization(VisualizationConfig::default())
}

pub fn quick_json_dump() -> WrtResult<String> {
    BudgetVisualizer::generate_visualization(VisualizationConfig {
        format: VisualizationFormat::Json,
        ..Default::default()
    })
}

pub fn quick_debug_dump() -> WrtResult<String> {
    DebugDumper::generate_debug_dump()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_system_initializer;

    #[test]
    fn test_visualization_config_default() {
        let config = VisualizationConfig::default();
        assert_eq!(config.format, VisualizationFormat::Ascii);
        assert!(config.include_crate_details);
        assert!(config.include_shared_pool);
        assert_eq!(config.chart_width, 60);
    }

    #[test]
    fn test_ascii_bar_creation() {
        let bar = BudgetVisualizer::create_ascii_bar(50, 10, false);
        // Should be roughly half filled
        assert!(bar.contains("█"));
        assert!(bar.contains("░"));
    }

    #[test]
    fn test_visualization_formats() {
        let _ = memory_system_initializer::presets::test();
        
        // Test that all formats can be generated without panicking
        for format in [
            VisualizationFormat::Ascii,
            VisualizationFormat::Json,
            VisualizationFormat::Csv,
            VisualizationFormat::Html,
            VisualizationFormat::Markdown,
        ] {
            let config = VisualizationConfig {
                format,
                ..Default::default()
            };
            
            let result = BudgetVisualizer::generate_visualization(config);
            // Should either succeed or fail gracefully
            match result {
                Ok(output) => assert!(!output.is_empty()),
                Err(_) => {} // Expected in no_alloc environments
            }
        }
    }

    #[test]
    fn test_debug_dump_generation() {
        let _ = memory_system_initializer::presets::test();
        
        let result = DebugDumper::generate_debug_dump();
        match result {
            Ok(dump) => {
                assert!(dump.contains("WRT MEMORY DEBUG DUMP"));
                assert!(dump.contains("SYSTEM INFORMATION"));
            }
            Err(_) => {} // Expected in no_alloc environments
        }
    }

    #[test]
    fn test_quick_functions() {
        let _ = memory_system_initializer::presets::test();
        
        // These should work or fail gracefully
        let _ = quick_ascii_dump();
        let _ = quick_json_dump();
        let _ = quick_debug_dump();
    }
}