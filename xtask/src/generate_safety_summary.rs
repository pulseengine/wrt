//! Generate safety verification summary for Sphinx documentation

use std::{fs, path::Path};
use anyhow::{Context, Result};
use crate::safety_verification::{SafetyVerificationConfig, generate_safety_report, load_requirements};

/// Generate safety summary RST file for inclusion in documentation
pub fn generate_safety_summary_rst(output_path: &Path) -> Result<()> {
    let requirements_path = Path::new("requirements.toml");
    
    if !requirements_path.exists() {
        generate_placeholder_safety_summary(output_path)?;
        return Ok(());
    }

    // Generate safety report
    let config = SafetyVerificationConfig {
        requirements_file: requirements_path.to_path_buf(),
        verify_files: false, // Skip file verification for docs generation
        ..Default::default()
    };

    let requirements = match load_requirements(&config.requirements_file) {
        Ok(req) => req,
        Err(_) => {
            generate_placeholder_safety_summary(output_path)?;
            return Ok(());
        }
    };

    let report = match generate_safety_report(&requirements, &[]) {
        Ok(report) => report,
        Err(_) => {
            generate_placeholder_safety_summary(output_path)?;
            return Ok(());
        }
    };

    // Generate RST content
    let rst_content = format!(
r#"Safety Verification Status
===========================

.. raw:: html

   <div class="safety-status-card">
     <div class="safety-header">
       <h3>🛡️ WRT Safety Verification Dashboard</h3>
       <span class="timestamp">Last Updated: {}</span>
     </div>
   </div>

Current Safety Status
---------------------

.. list-table:: ASIL Compliance Overview
   :widths: 20 20 20 20 20
   :header-rows: 1

   * - ASIL Level
     - Current Coverage
     - Required Coverage
     - Status
     - Gap
{}

.. note::
   🎯 **Overall Certification Readiness: {:.1}%**
   
   Status: {}

Requirements Traceability
-------------------------

.. list-table:: Requirements by Category
   :widths: 30 70
   :header-rows: 1

   * - Category
     - Count
{}

Test Coverage Status
--------------------

.. list-table:: Test Coverage Analysis
   :widths: 25 25 25 25
   :header-rows: 1

   * - Test Category
     - Coverage %
     - Test Count
     - Status
   * - Unit Tests
     - {:.1}%
     - {}
     - {}
   * - Integration Tests
     - {:.1}%
     - {}
     - {}
   * - ASIL-Tagged Tests
     - {:.1}%
     - {}
     - {}
   * - Safety Tests
     - {:.1}%
     - {}
     - {}
   * - Component Tests
     - {:.1}%
     - {}
     - {}

{}

Quick Actions
-------------

To update this status or get detailed reports:

.. code-block:: bash

   # Update safety status
   just safety-dashboard
   
   # Generate detailed report
   cargo xtask verify-safety --format html --output safety-report.html
   
   # Check specific requirements
   cargo xtask verify-requirements --detailed

For complete safety verification documentation, see :doc:`developer/tooling/safety_verification`.

.. raw:: html

   <style>
   .safety-status-card {{
     background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
     color: white;
     padding: 1rem;
     border-radius: 8px;
     margin: 1rem 0;
   }}
   .safety-header {{
     display: flex;
     justify-content: space-between;
     align-items: center;
   }}
   .safety-header h3 {{
     margin: 0;
     color: white;
   }}
   .timestamp {{
     font-size: 0.9em;
     opacity: 0.9;
   }}
   </style>
"#,
        report.timestamp,
        generate_asil_table(&report.asil_compliance),
        report.certification_readiness.overall_readiness,
        report.certification_readiness.readiness_status,
        generate_requirements_table(&report.requirements_by_asil, &report.requirements_by_type),
        report.test_coverage.unit_tests.coverage_percent,
        report.test_coverage.unit_tests.test_count,
        format_status(&report.test_coverage.unit_tests.status),
        report.test_coverage.integration_tests.coverage_percent,
        report.test_coverage.integration_tests.test_count,
        format_status(&report.test_coverage.integration_tests.status),
        report.test_coverage.asil_tagged_tests.coverage_percent,
        report.test_coverage.asil_tagged_tests.test_count,
        format_status(&report.test_coverage.asil_tagged_tests.status),
        report.test_coverage.safety_tests.coverage_percent,
        report.test_coverage.safety_tests.test_count,
        format_status(&report.test_coverage.safety_tests.status),
        report.test_coverage.component_tests.coverage_percent,
        report.test_coverage.component_tests.test_count,
        format_status(&report.test_coverage.component_tests.status),
        if report.missing_files.is_empty() {
            "✅ All referenced files exist".to_string()
        } else {
            format!("❌ Missing Files:\n\n{}", 
                report.missing_files.iter()
                    .map(|f| format!("   - {}", f))
                    .collect::<Vec<_>>()
                    .join("\n"))
        }
    );

    fs::write(output_path, rst_content)
        .with_context(|| format!("Failed to write safety summary to {:?}", output_path))?;

    println!("✅ Generated safety summary: {:?}", output_path);
    Ok(())
}

fn generate_asil_table(asil_compliance: &[crate::safety_verification::AsilCompliance]) -> String {
    asil_compliance.iter()
        .map(|compliance| {
            let status_icon = match compliance.status {
                crate::safety_verification::ComplianceStatus::Pass => "✅ PASS",
                crate::safety_verification::ComplianceStatus::Fail => "❌ FAIL",
            };
            let gap = compliance.required_coverage - compliance.current_coverage;
            format!(
                "   * - {}\n     - {:.1}%\n     - {:.1}%\n     - {}\n     - {:.1}%",
                compliance.level,
                compliance.current_coverage,
                compliance.required_coverage,
                status_icon,
                gap.max(0.0)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn generate_requirements_table(asil_reqs: &std::collections::HashMap<String, usize>, type_reqs: &std::collections::HashMap<String, usize>) -> String {
    let mut rows = Vec::new();
    
    // ASIL breakdown
    for (asil, count) in asil_reqs {
        rows.push(format!("   * - ASIL {}\n     - {} requirements", asil, count));
    }
    
    // Type breakdown  
    for (req_type, count) in type_reqs {
        rows.push(format!("   * - {} Requirements\n     - {} requirements", req_type, count));
    }
    
    rows.join("\n")
}

fn format_status(status: &crate::safety_verification::CoverageStatus) -> String {
    match status {
        crate::safety_verification::CoverageStatus::Good => "✅ Good",
        crate::safety_verification::CoverageStatus::Warning => "⚠️ Warning", 
        crate::safety_verification::CoverageStatus::Poor => "❌ Poor",
    }.to_string()
}

/// Generate placeholder safety summary when verification fails
pub fn generate_placeholder_safety_summary(output_path: &Path) -> Result<()> {
    let placeholder_content = r#"Safety Verification Status
===========================

.. warning::
   
   Safety verification report could not be generated.
   
   This typically means:
   
   - No ``requirements.toml`` file found
   - Safety verification system not yet configured
   - Build errors preventing verification
   
   To set up safety verification:
   
   .. code-block:: bash
   
      # Initialize requirements template
      cargo xtask init-requirements
      
      # Run safety verification
      cargo xtask verify-safety

For setup instructions, see :doc:`developer/tooling/safety_verification`.
"#;

    fs::write(output_path, placeholder_content)
        .with_context(|| format!("Failed to write placeholder safety summary to {:?}", output_path))?;

    Ok(())
}