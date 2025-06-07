===============================
SCORE Safety Verification Tools
===============================

WRT implements a comprehensive safety verification framework inspired by the SCORE (Safety Critical Object-Oriented Real-time Embedded) methodology. This system provides automated tools for tracking safety requirements, ASIL compliance, and certification readiness.

.. contents:: On this page
   :local:
   :depth: 2

Overview
--------

The safety verification system implements automotive and aerospace safety standards (ISO 26262, DO-178C) through:

- **Requirements Traceability**: Link requirements to implementation, tests, and documentation
- **ASIL Compliance Monitoring**: Track Automotive Safety Integrity Levels (QM through ASIL-D)
- **Test Coverage Analysis**: Categorize tests by safety level and track coverage
- **Documentation Verification**: Ensure proper documentation for safety requirements
- **Platform Verification**: Multi-platform safety verification (Linux, macOS, QNX, Zephyr)
- **Certification Readiness**: Track progress toward safety certification

Quick Start
-----------

Initialize Requirements
~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   # Create requirements template
   just init-requirements
   
   # Or with xtask directly
   cargo xtask init-requirements

Run Safety Verification
~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   # Quick verification dashboard
   just safety-dashboard
   
   # Check requirements traceability
   just check-requirements
   
   # Full safety verification
   just verify-safety
   
   # Detailed requirements verification
   just verify-requirements

Generate Reports
~~~~~~~~~~~~~~~~

.. code-block:: bash

   # Text report
   just safety-report
   
   # JSON report
   cargo xtask verify-safety --format json
   
   # Save to file
   cargo xtask safety-report --format json --output safety.json

Available Commands
------------------

Core Commands
~~~~~~~~~~~~~

All safety verification commands are implemented in ``xtask`` for proper integration with the WRT build system:

.. list-table:: Safety Verification Commands
   :widths: 30 50 20
   :header-rows: 1

   * - Command
     - Description
     - Output Formats
   * - ``cargo xtask check-requirements``
     - Quick requirements file validation
     - Text
   * - ``cargo xtask verify-requirements``
     - Detailed file existence checking
     - Text
   * - ``cargo xtask verify-safety``
     - SCORE-inspired safety framework verification
     - Text, JSON, HTML
   * - ``cargo xtask safety-report``
     - Generate comprehensive safety reports
     - Text, JSON, HTML
   * - ``cargo xtask safety-dashboard``
     - Complete safety status overview
     - Text
   * - ``cargo xtask init-requirements``
     - Create requirements template
     - N/A

Advanced Options
~~~~~~~~~~~~~~~~

.. code-block:: bash

   # JSON output for CI integration
   cargo xtask verify-safety --format json --output safety.json
   
   # Detailed requirements verification
   cargo xtask verify-requirements --detailed --requirements-file custom.toml
   
   # Skip file verification (faster checks)
   cargo xtask verify-requirements --skip-files
   
   # HTML report for stakeholders
   cargo xtask safety-report --format html --output report.html

Requirements Format
-------------------

Requirements are defined in ``requirements.toml`` at the workspace root:

.. code-block:: toml

   [meta]
   project = "WRT WebAssembly Runtime"
   version = "0.2.0"
   safety_standard = "ISO26262"
   
   [[requirement]]
   id = "REQ_MEM_001"
   title = "Memory Bounds Checking"
   description = "All memory operations must be bounds-checked"
   type = "Memory"
   asil_level = "AsilC"
   implementations = ["wrt-foundation/src/safe_memory.rs"]
   tests = ["wrt-foundation/tests/memory_tests_moved.rs"]
   documentation = ["docs/architecture/memory_model.rst"]

ASIL Levels Reference
~~~~~~~~~~~~~~~~~~~~~

.. list-table:: ASIL Compliance Levels
   :widths: 15 25 15 45
   :header-rows: 1

   * - Level
     - Description
     - Coverage Target
     - Use Cases
   * - QM
     - Quality Management
     - 70%
     - No safety requirements
   * - ASIL-A
     - Lowest safety integrity
     - 80%
     - Light injury potential
   * - ASIL-B
     - Light safety requirements
     - 90%
     - Moderate injury potential
   * - ASIL-C
     - Moderate safety requirements
     - 90%
     - Severe injury potential
   * - ASIL-D
     - Highest safety integrity
     - 95%
     - Life-threatening potential

Tool Output Examples & Interpretation
--------------------------------------

The safety verification tool provides comprehensive reports with actionable insights. Here are real examples from the WRT project:

Safety Dashboard Output
~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: text

   🔍 SCORE-Inspired Safety Verification Framework
   ════════════════════════════════════════════════════════════
   Generated: 2025-06-07T03:47:46.379649+00:00

   📋 Requirements Traceability Framework
   ────────────────────────────────────────
     Total Requirements: 6
     Requirements by ASIL Level:
       AsilD: 1 requirements
       AsilB: 2 requirements
       AsilC: 3 requirements

   🛡️  ASIL Compliance Analysis:
   ┌─────────┬────────────┬──────────┬────────────┐
   │ ASIL    │ Current    │ Required │ Status     │
   ├─────────┼────────────┼──────────┼────────────┤
   │ QM      │    100.0% │   70.0% │ ✅ PASS     │
   │ AsilA   │     95.0% │   80.0% │ ✅ PASS     │
   │ AsilB   │     85.0% │   90.0% │ ❌ FAIL     │
   │ AsilC   │     75.0% │   90.0% │ ❌ FAIL     │
   │ AsilD   │     60.0% │   95.0% │ ❌ FAIL     │
   └─────────┴────────────┴──────────┴────────────┘

   🧪 Test Coverage Analysis
   ─────────────────────────
     ✅ Unit Tests: 87.5% (156 tests)
     ⚠️ Integration Tests: 72.3% (89 tests)
     ❌ ASIL-Tagged Tests: 68.1% (34 tests)
     ✅ Safety Tests: 91.2% (23 tests)
     ✅ Component Tests: 83.7% (67 tests)

   ❌ Missing Files:
     • [REQ_COMP_001] Documentation: docs/architecture/component_model.rst
     • [REQ_ASYNC_001] Documentation: docs/architecture/async_threading.rst
     • [REQ_PARSE_001] Documentation: docs/architecture/intercept_system.rst
     • [REQ_ERROR_001] Documentation: docs/architecture/logging.rst

   🎯 Certification Readiness Assessment
   ─────────────────────────────────────
     Requirements Traceability: 90%
     Test Coverage (ASIL-D): 60%
     Documentation Completeness: 75%
     Code Review Coverage: 88%
     Static Analysis Clean: 95%
     MISRA C Compliance: 82%
     Formal Verification: 45%

   🎯 Overall Certification Readiness: 76.4%
      Status: Approaching readiness - address key gaps

Interpreting the Results
~~~~~~~~~~~~~~~~~~~~~~~~

**🟢 Strengths (Immediate Certification Ready)**
   - **QM & ASIL-A**: Full compliance achieved
   - **Unit Tests**: 87.5% coverage exceeds industry standards
   - **Static Analysis**: 95% clean - excellent code quality
   - **Requirements Traceability**: 90% - strong linkage

**🟡 Warning Areas (Need Attention)**
   - **Integration Tests**: 72.3% - boost to 80%+ for robustness
   - **Documentation**: 75% - address missing architecture files

**🔴 Critical Gaps (Block Certification)**
   - **ASIL-D Coverage**: 60% → 95% required (35% gap)
   - **ASIL-B/C**: Under 90% threshold - add safety tests
   - **ASIL-Tagged Tests**: 68.1% - implement test categorization

**📋 Action Items from Report**
   1. Create missing documentation files (4 files identified)
   2. Add 25+ ASIL-D tagged safety tests  
   3. Expand integration test coverage to 80%+
   4. Implement formal verification methods (45% → 60%+)

Report Formats
--------------

Text Format
~~~~~~~~~~~

Default human-readable format with colored output and tables:

.. code-block:: text

   🔍 SCORE-Inspired Safety Verification Framework
   ════════════════════════════════════════════════════════════
   Generated: 2025-06-07T03:40:04.727731+00:00
   
   📋 Requirements Traceability Framework
   ────────────────────────────────────────
     Total Requirements: 6
     Requirements by ASIL Level:
       AsilD: 1 requirements
       AsilB: 2 requirements
       AsilC: 3 requirements

JSON Format
~~~~~~~~~~~

Machine-readable format for CI integration and automated processing:

.. code-block:: bash

   # Generate JSON report
   cargo xtask verify-safety --format json | jq '.certification_readiness.overall_readiness'
   # Output: 76.42857142857143

**Example JSON Output Structure:**

.. code-block:: json

   {
     "timestamp": "2025-06-07T03:47:53.300873+00:00",
     "project_meta": {
       "project": "WRT WebAssembly Runtime",
       "version": "0.2.0",
       "safety_standard": "ISO26262"
     },
     "total_requirements": 6,
     "requirements_by_asil": {
       "AsilD": 1,
       "AsilC": 3,
       "AsilB": 2
     },
     "asil_compliance": [
       {
         "level": "AsilD",
         "current_coverage": 60.0,
         "required_coverage": 95.0,
         "status": "Fail"
       }
     ],
     "test_coverage": {
       "unit_tests": {
         "coverage_percent": 87.5,
         "test_count": 156,
         "status": "Good"
       },
       "asil_tagged_tests": {
         "coverage_percent": 68.1,
         "test_count": 34,
         "status": "Poor"
       }
     },
     "missing_files": [
       "[REQ_COMP_001] Documentation: docs/architecture/component_model.rst"
     ],
     "certification_readiness": {
       "overall_readiness": 76.42857142857143,
       "readiness_status": "Approaching readiness - address key gaps"
     }
   }

**CI Integration Examples:**

.. code-block:: bash

   # Fail CI if overall readiness < 75%
   READINESS=$(cargo xtask verify-safety --format json | jq '.certification_readiness.overall_readiness')
   if (( $(echo "$READINESS < 75.0" | bc -l) )); then
     echo "❌ Safety readiness below threshold: $READINESS%"
     exit 1
   fi
   
   # Check for critical ASIL-D failures
   ASIL_D_FAIL=$(cargo xtask verify-safety --format json | jq '.asil_compliance[] | select(.level=="AsilD" and .status=="Fail")')
   if [ ! -z "$ASIL_D_FAIL" ]; then
     echo "❌ ASIL-D compliance failure - blocking release"
     exit 1
   fi

HTML Format
~~~~~~~~~~~

Formatted reports for stakeholder presentations and documentation:

.. code-block:: bash

   cargo xtask safety-report --format html --output safety-report.html

CI Integration
--------------

Automated Safety Checks
~~~~~~~~~~~~~~~~~~~~~~~~

Add to your CI pipeline:

.. code-block:: yaml

   # .github/workflows/safety.yml
   - name: Safety Verification
     run: |
       cargo xtask verify-safety --format json --output safety-report.json
       cargo xtask check-requirements

   - name: Upload Safety Report
     uses: actions/upload-artifact@v3
     with:
       name: safety-report
       path: safety-report.json

Integration with Existing Tools
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

The safety verification system integrates with:

- **CI Pipeline**: Automated safety checks on every build
- **Documentation**: Requirements linked to Sphinx documentation  
- **Testing**: ASIL-tagged test categorization
- **Build System**: Integrated through xtask automation
- **Justfile**: Convenient command aliases

Implementation Details
----------------------

Core Components
~~~~~~~~~~~~~~~

- ``xtask/src/safety_verification.rs`` - Core verification framework
- ``requirements.toml`` - Requirements definition file
- ``justfile`` - Convenient command aliases
- ``docs/architecture/safety.rst`` - Safety documentation

File Structure
~~~~~~~~~~~~~~

.. code-block:: text

   wrt2/
   ├── requirements.toml           # Requirements definitions
   ├── xtask/src/
   │   └── safety_verification.rs  # Core implementation
   ├── justfile                    # Command aliases
   └── docs/
       ├── architecture/safety.rst # Architecture docs
       └── qualification/          # Certification materials

Certification Path
------------------

Development Phases
~~~~~~~~~~~~~~~~~~

1. **Phase 1** ✅: Basic requirements tracking established
2. **Phase 2** 🔄: ASIL test macros and categorization  
3. **Phase 3** 📋: CI integration and automated reporting
4. **Phase 4** 🎯: Certification artifacts generation
5. **Phase 5** 📊: External audit preparation

Next Steps
~~~~~~~~~~

1. Address ASIL-D coverage gaps (60% → 95%)
2. Complete missing architecture documentation
3. Expand formal verification coverage
4. Implement ASIL test macros
5. Integrate with CI pipeline

Using Results for Decision Making
----------------------------------

Release Gate Decisions
~~~~~~~~~~~~~~~~~~~~~~

Use safety verification results to make data-driven release decisions:

.. list-table:: Release Decision Matrix
   :widths: 25 25 25 25
   :header-rows: 1

   * - Overall Readiness
     - ASIL-D Status
     - Missing Files
     - Release Decision
   * - ≥ 85%
     - PASS
     - None
     - ✅ **Release Approved**
   * - 70-84%
     - PASS
     - < 5
     - ⚠️ **Conditional Release**
   * - < 70%
     - Any
     - Any
     - ❌ **Block Release**
   * - Any
     - FAIL
     - Any
     - ❌ **Block Release**

**Example CI Gate Logic:**

.. code-block:: bash

   #!/bin/bash
   # Safety gate for release pipeline
   
   RESULTS=$(cargo xtask verify-safety --format json)
   READINESS=$(echo "$RESULTS" | jq '.certification_readiness.overall_readiness')
   ASIL_D_STATUS=$(echo "$RESULTS" | jq -r '.asil_compliance[] | select(.level=="AsilD") | .status')
   MISSING_COUNT=$(echo "$RESULTS" | jq '.missing_files | length')
   
   echo "🔍 Safety Gate Assessment:"
   echo "   Overall Readiness: $READINESS%"
   echo "   ASIL-D Status: $ASIL_D_STATUS"
   echo "   Missing Files: $MISSING_COUNT"
   
   # Critical failure: ASIL-D must pass
   if [ "$ASIL_D_STATUS" != "Pass" ]; then
     echo "❌ CRITICAL: ASIL-D compliance failure"
     exit 1
   fi
   
   # Release readiness threshold
   if (( $(echo "$READINESS >= 85.0" | bc -l) )); then
     echo "✅ APPROVED: Ready for production release"
     exit 0
   elif (( $(echo "$READINESS >= 70.0" | bc -l) )) && [ "$MISSING_COUNT" -lt 5 ]; then
     echo "⚠️ CONDITIONAL: Release with risk acceptance"
     exit 0
   else
     echo "❌ BLOCKED: Insufficient safety readiness"
     exit 1
   fi

Sprint Planning Priorities
~~~~~~~~~~~~~~~~~~~~~~~~~~~

Use verification results to prioritize development work:

**High Priority (Sprint Blockers):**
  - ASIL-D failures (business critical)
  - Missing documentation files (quick wins)
  - Test coverage gaps > 20%

**Medium Priority (Next Sprint):**
  - ASIL-B/C improvements
  - Integration test coverage
  - Formal verification expansion

**Low Priority (Backlog):**
  - Documentation improvements
  - Code review coverage optimization
  - MISRA compliance refinements

Team Communication
~~~~~~~~~~~~~~~~~~

**Daily Standup Metrics:**

.. code-block:: bash

   # Quick standup status
   cargo xtask verify-safety | grep "Overall Certification Readiness"
   # Output: 🎯 Overall Certification Readiness: 76.4%

**Weekly Stakeholder Reports:**

.. code-block:: bash

   # Generate stakeholder-friendly HTML report
   cargo xtask safety-report --format html --output "weekly-safety-$(date +%Y%m%d).html"
   
   # Email-friendly summary
   echo "WRT Safety Status - Week $(date +%U)"
   cargo xtask verify-safety | grep -E "(Overall|ASIL.*FAIL|Missing Files)"

Best Practices
--------------

Requirements Management
~~~~~~~~~~~~~~~~~~~~~~~

- Link every requirement to implementation, tests, and documentation
- Use descriptive requirement IDs (e.g., ``REQ_MEM_001``)
- Assign appropriate ASIL levels based on safety analysis
- Keep requirements.toml in version control

Daily Development Workflow
~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   # Before committing changes
   just safety-dashboard
   
   # Check specific requirements
   cargo xtask verify-requirements --detailed
   
   # Generate report for stakeholders
   cargo xtask safety-report --format html --output weekly-report.html

Monitoring & Alerts
~~~~~~~~~~~~~~~~~~~

**Setup automated monitoring:**

.. code-block:: bash

   # Add to CI pipeline for trend monitoring
   cargo xtask verify-safety --format json > "safety-report-$(date +%Y%m%d).json"
   
   # Alert on readiness degradation
   PREV_READINESS=$(cat previous-safety.json | jq '.certification_readiness.overall_readiness')
   CURR_READINESS=$(cargo xtask verify-safety --format json | jq '.certification_readiness.overall_readiness')
   
   if (( $(echo "$CURR_READINESS < $PREV_READINESS - 5.0" | bc -l) )); then
     echo "🚨 ALERT: Safety readiness dropped by >5%"
     # Send notification to team
   fi

Troubleshooting
---------------

Common Issues
~~~~~~~~~~~~~

**Missing Files**
   If verification reports missing files, either:
   - Create the missing files
   - Update requirements.toml to reference existing files
   - Use ``--skip-files`` for quick checks during development

**Low ASIL Coverage**
   Improve test coverage by:
   - Adding ASIL-tagged tests
   - Expanding safety-critical test scenarios
   - Implementing formal verification methods

**Requirements File Errors**
   Validate TOML syntax:
   
   .. code-block:: bash
   
      # Check syntax
      cargo xtask check-requirements

See Also
--------

- :doc:`../testing/index` - Testing strategies and coverage
- :doc:`../../architecture/safety` - Safety architecture overview
- :doc:`../../qualification/index` - Qualification materials
- :doc:`../../safety/index` - Safety guidelines and constraints

---

**Status**: ✅ Operational - Ready for daily use in WRT development