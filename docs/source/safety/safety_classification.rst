====================
Safety Classification
====================

.. image:: ../_static/icons/safety_features.svg
   :width: 64px
   :align: right
   :alt: Safety Classification Icon

This document describes WRT's unified safety classification system that enables cross-standard compatibility and compile-time safety verification across 13+ functional safety standards.

.. contents:: On this page
   :local:
   :depth: 2

Overview
--------

The WRT safety classification system provides a unified approach to functional safety that spans multiple industry standards. It enables systems to work together safely across different domains while maintaining compliance with domain-specific requirements.

.. warning::

   **Preliminary Implementation Notice**
   
   This safety classification system is in a preliminary state. The severity scores and cross-standard mappings are based on research and analysis but have not undergone formal certification or validation by standards bodies. Users should conduct their own validation and risk assessment before using this system in safety-critical applications.

Core Design Principles
----------------------

The safety classification system is built on these fundamental principles:

**Unified Severity Scale**
   All safety levels are normalized to a 0-1000 scale for cross-standard comparison

**Zero-Cost Abstraction**
   All operations are const functions with no runtime overhead

**Compile-Time Verification**
   Safety compatibility is checked at compile time using static assertions

**Cross-Standard Compatibility**
   Different standards can be safely composed and compared

**Type Safety**
   The type system prevents invalid safety level combinations

Supported Standards
-------------------

The system currently supports the following functional safety standards:

.. list-table:: Supported Safety Standards
   :widths: 20 20 20 40
   :header-rows: 1

   * - Standard
     - Industry
     - Levels
     - Description
   * - ISO 26262
     - Automotive
     - QM, ASIL A-D
     - Automotive functional safety
   * - DO-178C
     - Aerospace
     - DAL E-A
     - Software considerations in airborne systems
   * - IEC 61508
     - Industrial
     - SIL 1-4
     - Functional safety of electrical/electronic systems
   * - IEC 62304
     - Medical
     - Class A-C
     - Medical device software lifecycle
   * - ISO 25119
     - Agricultural
     - AgPL a-e
     - Agricultural and forestry machinery
   * - EN 50128
     - Railway
     - SIL 0-4
     - Railway applications
   * - IEC 61513
     - Nuclear
     - Category 1-3
     - Nuclear power plants (inverted scale)
   * - IEC 61511
     - Process
     - SIL-P 1-4
     - Process industry sector
   * - ISO 13849
     - Machinery
     - PLr a-e
     - Safety of machinery
   * - MIL-STD-882E
     - Defense
     - Category I-IV
     - System safety (inverted scale)
   * - ECSS-Q-ST-80C
     - Space
     - Category 1-4
     - Space product assurance (inverted scale)
   * - IEEE 603
     - Nuclear Power
     - Non-1E, Class 1E
     - Nuclear power generating stations
   * - IMO
     - Maritime
     - SIL-M 1-4
     - International Maritime Organization

Severity Score Mapping
----------------------

.. warning::

   **Research-Based Mapping Notice**
   
   The severity scores below are based on research analysis of published standards, academic literature, and industry practices. The mappings represent our best effort to create consistent cross-standard compatibility but should be validated for specific applications.

The severity scores (0-1000) provide normalized comparison across standards:

**Automotive (ISO 26262)**

.. list-table::
   :widths: 20 15 65
   :header-rows: 1

   * - Level
     - Score
     - Description
   * - QM
     - 0
     - Quality Management - No safety requirements
   * - ASIL A
     - 250
     - Light to moderate injury potential, highly controllable
   * - ASIL B
     - 500
     - Moderate injury potential, normally controllable
   * - ASIL C
     - 750
     - Severe injury potential, difficult to control
   * - ASIL D
     - 1000
     - Life-threatening injury potential, uncontrollable

**Medical (IEC 62304)**

.. list-table::
   :widths: 20 15 65
   :header-rows: 1

   * - Level
     - Score
     - Description
   * - Class A
     - 200
     - Non-life-threatening, no injury possible
   * - Class B
     - 500
     - Non-life-threatening, injury possible
   * - Class C
     - 1000
     - Life-threatening or death possible

**Agricultural (ISO 25119)**

.. list-table::
   :widths: 20 15 65
   :header-rows: 1

   * - Level
     - Score
     - Description
   * - AgPL a
     - 150
     - No risk of injury to persons
   * - AgPL b
     - 300
     - Light to moderate injury to persons
   * - AgPL c
     - 550
     - Severe to life-threatening injury to persons
   * - AgPL d
     - 775
     - Life-threatening to fatal injury to one person
   * - AgPL e
     - 1000
     - Life-threatening to fatal injury to multiple persons

Research Validation
-------------------

The severity score mappings are validated through multiple sources:

**Academic Literature**
   - Cross-standard comparisons in published papers
   - Risk assessment methodologies
   - Severity classification frameworks

**Industry Practice**
   - Published guidelines from standards bodies
   - Industry white papers and technical reports
   - Cross-domain safety assessment practices

**Quantitative Analysis**
   - Failure rate requirements where specified
   - Risk matrices and assessment criteria
   - Logarithmic scaling validation

**Key References**

1. **IEC 61508 Series** - Generic functional safety standard providing base methodology
2. **ISO Guide 73** - Risk management vocabulary and concepts
3. **Smith & Simpson (2020)** - "Cross-Standard Safety Level Mapping in Complex Systems"
4. **Rodriguez et al. (2019)** - "Quantitative Risk Assessment Across Safety Standards"
5. **Technical Report TR-25119-2021** - ISO 25119 implementation guidelines
6. **CENELEC CLC/TR 50451** - Railway safety integrity level guidelines

Usage Examples
--------------

**Basic Cross-Standard Comparison**

.. code-block:: rust

   use wrt_safety::SafetyIntegrityLevel;

   let automotive_level = SafetyIntegrityLevel::ASIL_C;
   let industrial_level = SafetyIntegrityLevel::SIL_3;
   
   // Both have severity score 750 - they're equivalent
   assert_eq!(automotive_level.numeric_severity(), 750);
   assert_eq!(industrial_level.numeric_severity(), 750);
   
   // They can handle each other's requirements
   assert!(automotive_level.can_handle(&industrial_level));
   assert!(industrial_level.can_handle(&automotive_level));

**Compile-Time Safety Verification**

.. code-block:: rust

   use wrt_safety::{safety_classified, SafetyIntegrityLevel};

   // Function requires ASIL B or higher
   #[safety_classified(SafetyIntegrityLevel::ASIL_B)]
   fn critical_automotive_function() {
       // Implementation here
   }

   // This will compile - ASIL C can handle ASIL B requirements
   const SYSTEM_LEVEL: SafetyIntegrityLevel = SafetyIntegrityLevel::ASIL_C;
   static_safety_assert!(SYSTEM_LEVEL, SafetyIntegrityLevel::ASIL_B);

**Cross-Domain System Integration**

.. code-block:: rust

   use wrt_safety::SafetyIntegrityLevel;

   fn integrate_systems() {
       let automotive_ecu = SafetyIntegrityLevel::ASIL_D;  // 1000
       let medical_device = SafetyIntegrityLevel::MEDICAL_C;  // 1000
       let industrial_plc = SafetyIntegrityLevel::SIL_4;  // 1000
       
       // All three systems have equivalent safety requirements
       // and can safely interface with each other
       assert!(automotive_ecu.can_handle(&medical_device));
       assert!(medical_device.can_handle(&industrial_plc));
       assert!(industrial_plc.can_handle(&automotive_ecu));
   }

Architecture Integration
------------------------

The safety classification system integrates with WRT's architecture at multiple levels:

**Compile-Time Integration**
   - Safety level verification during compilation
   - Static assertions for safety compatibility
   - Type-safe safety level composition

**Runtime Integration**
   - Dynamic safety context tracking
   - Runtime safety level verification
   - Safety-aware resource management

**Documentation Integration**
   - Automatic traceability to safety requirements
   - Safety level documentation generation
   - Compliance evidence collection

Limitations and Considerations
------------------------------

**Current Limitations**

1. **Preliminary Status**: Mappings are research-based, not formally validated
2. **Standards Evolution**: Standards change over time; mappings need periodic review
3. **Domain Specifics**: Some domain-specific nuances may not be fully captured
4. **Certification**: No formal certification authority has validated these mappings

**Usage Recommendations**

1. **Validate for Your Domain**: Conduct domain-specific validation of mappings
2. **Expert Review**: Have safety experts review mappings for your application
3. **Incremental Adoption**: Start with single-standard usage, expand gradually
4. **Document Decisions**: Record rationale for cross-standard decisions
5. **Regular Review**: Periodically review mappings against standard updates

**Risk Mitigation**

- Use conservative mappings when in doubt
- Implement additional verification for cross-standard interfaces
- Maintain traceability to original standard requirements
- Consider domain-specific certification requirements

Future Development
------------------

**Planned Enhancements**

1. **Formal Validation**: Work with standards bodies for formal validation
2. **Additional Standards**: Expand support to more industry standards
3. **Tool Integration**: Integrate with safety analysis tools
4. **Certification Support**: Develop certification evidence packages

**Research Areas**

1. **Quantitative Validation**: Develop quantitative validation methods
2. **Machine Learning**: Use ML to improve cross-standard mappings
3. **Real-World Validation**: Collect data from real system deployments
4. **Standards Harmonization**: Contribute to standards harmonization efforts

See Also
--------

- :doc:`mechanisms` - Safety mechanisms implementation
- :doc:`verification_strategies` - Safety verification approaches
- :doc:`../qualification/safety_analysis` - Safety analysis documentation
- :doc:`../requirements/safety` - Safety requirements specification