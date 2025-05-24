use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CoverageData {
    data: Vec<CoverageItem>,
    #[serde(rename = "type")]
    coverage_type: String,
    version: String,
}

#[derive(Debug, Deserialize)]
struct CoverageItem {
    files: Vec<FileCoverage>,
    totals: CoverageTotals,
}

#[derive(Debug, Deserialize)]
struct FileCoverage {
    filename: String,
    summary: CoverageSummary,
}

#[derive(Debug, Deserialize)]
struct CoverageSummary {
    lines: CoverageMetric,
    functions: CoverageMetric,
    branches: Option<CoverageMetric>,
}

#[derive(Debug, Deserialize)]
struct CoverageMetric {
    count: u64,
    covered: u64,
    percent: f64,
}

#[derive(Debug, Deserialize)]
struct CoverageTotals {
    lines: CoverageMetric,
    functions: CoverageMetric,
    branches: Option<CoverageMetric>,
}

pub fn generate_coverage_summary_rst(coverage_json_path: &Path, output_path: &Path) -> Result<()> {
    // Read coverage JSON
    let coverage_json =
        fs::read_to_string(coverage_json_path).context("Failed to read coverage JSON file")?;

    let coverage_data: CoverageData =
        serde_json::from_str(&coverage_json).context("Failed to parse coverage JSON")?;

    // Generate RST content
    let mut rst_content = String::new();

    rst_content.push_str("Coverage Summary\n");
    rst_content.push_str("================\n\n");

    rst_content.push_str(".. note::\n");
    rst_content.push_str("   Coverage data generated from the latest test run.\n");
    rst_content.push_str("   MC/DC coverage is required for safety-critical components.\n\n");

    // Overall coverage summary
    rst_content.push_str("Overall Project Coverage\n");
    rst_content.push_str("------------------------\n\n");

    rst_content.push_str(".. list-table::\n");
    rst_content.push_str("   :header-rows: 1\n");
    rst_content.push_str("   :widths: 30 20 20 30\n\n");
    rst_content.push_str("   * - Metric\n");
    rst_content.push_str("     - Covered\n");
    rst_content.push_str("     - Total\n");
    rst_content.push_str("     - Percentage\n");

    // Get totals from the first data item (usually there's only one)
    let totals = coverage_data
        .data
        .get(0)
        .map(|item| &item.totals)
        .context("No coverage data items found")?;

    // Line coverage
    rst_content.push_str(&format!(
        "   * - **Line Coverage**\n     - {}\n     - {}\n     - {:.1}%\n",
        totals.lines.covered, totals.lines.count, totals.lines.percent
    ));

    // Function coverage
    rst_content.push_str(&format!(
        "   * - **Function Coverage**\n     - {}\n     - {}\n     - {:.1}%\n",
        totals.functions.covered, totals.functions.count, totals.functions.percent
    ));

    // Branch coverage (if available)
    if let Some(branches) = &totals.branches {
        rst_content.push_str(&format!(
            "   * - **Branch Coverage**\n     - {}\n     - {}\n     - {:.1}%\n",
            branches.covered, branches.count, branches.percent
        ));
    }

    // Safety requirements
    rst_content.push_str("\nSafety Requirements\n");
    rst_content.push_str("-------------------\n\n");

    rst_content.push_str(".. list-table::\n");
    rst_content.push_str("   :header-rows: 1\n");
    rst_content.push_str("   :widths: 40 20 20 20\n\n");
    rst_content.push_str("   * - Requirement\n");
    rst_content.push_str("     - Target\n");
    rst_content.push_str("     - Actual\n");
    rst_content.push_str("     - Status\n");

    // Check against safety requirements
    let line_status = if totals.lines.percent >= 90.0 { "✓ PASS" } else { "✗ FAIL" };
    let func_status = if totals.functions.percent >= 95.0 { "✓ PASS" } else { "✗ FAIL" };
    let branch_status = if let Some(branches) = &totals.branches {
        if branches.percent >= 85.0 {
            "✓ PASS"
        } else {
            "✗ FAIL"
        }
    } else {
        "N/A"
    };

    rst_content.push_str(&format!(
        "   * - Line Coverage (ASIL-D)\n     - ≥90%\n     - {:.1}%\n     - {}\n",
        totals.lines.percent, line_status
    ));

    rst_content.push_str(&format!(
        "   * - Function Coverage (ASIL-D)\n     - ≥95%\n     - {:.1}%\n     - {}\n",
        totals.functions.percent, func_status
    ));

    rst_content.push_str(&format!(
        "   * - Branch Coverage (ASIL-D)\n     - ≥85%\n     - {:.1}%\n     - {}\n",
        totals.branches.as_ref().map(|b| b.percent).unwrap_or(0.0),
        branch_status
    ));

    rst_content.push_str("   * - MC/DC Coverage (Safety-Critical)\n");
    rst_content.push_str("     - 100%\n");
    rst_content.push_str("     - TBD\n");
    rst_content.push_str("     - ⚠ Pending\n");

    // Per-crate coverage (safety-critical crates)
    rst_content.push_str("\nSafety-Critical Crates Coverage\n");
    rst_content.push_str("--------------------------------\n\n");

    let safety_critical_crates =
        ["wrt-runtime", "wrt-instructions", "wrt-sync", "wrt-foundation", "wrt-platform"];

    rst_content.push_str(".. list-table::\n");
    rst_content.push_str("   :header-rows: 1\n");
    rst_content.push_str("   :widths: 30 20 20 30\n\n");
    rst_content.push_str("   * - Crate\n");
    rst_content.push_str("     - Line Coverage\n");
    rst_content.push_str("     - Function Coverage\n");
    rst_content.push_str("     - MC/DC Required\n");

    for crate_name in &safety_critical_crates {
        // Find coverage data for this crate
        let crate_coverage = coverage_data
            .data
            .get(0)
            .and_then(|item| item.files.iter().find(|f| f.filename.contains(crate_name)));

        if let Some(coverage) = crate_coverage {
            rst_content.push_str(&format!(
                "   * - {}\n     - {:.1}%\n     - {:.1}%\n     - Yes\n",
                crate_name, coverage.summary.lines.percent, coverage.summary.functions.percent
            ));
        } else {
            rst_content
                .push_str(&format!("   * - {}\n     - N/A\n     - N/A\n     - Yes\n", crate_name));
        }
    }

    // Links to detailed reports
    rst_content.push_str("\nDetailed Reports\n");
    rst_content.push_str("----------------\n\n");
    rst_content.push_str("- `HTML Coverage Report <../_static/coverage/index.html>`_\n");
    rst_content.push_str("- `LCOV Report <../_static/coverage/lcov.info>`_\n");
    rst_content.push_str("- `MC/DC Analysis <../_static/coverage/mcdc_report.html>`_\n");

    // Write to file
    fs::write(output_path, rst_content).context("Failed to write coverage summary RST file")?;

    Ok(())
}

/// Generate a placeholder when coverage data is not available
pub fn generate_placeholder_coverage_summary(output_path: &Path) -> Result<()> {
    let content = include_str!("../../docs/source/_generated_coverage_summary.rst.template");
    fs::write(output_path, content).context("Failed to write placeholder coverage summary")?;
    Ok(())
}
