=======================================
WebAssembly Runtime (WRT) Documentation
=======================================

.. raw:: html

   <div class="landing-header">
      <img src="_static/icons/logo.svg" alt="WRT Logo" class="landing-logo">
      <p class="landing-subtitle">A safety-critical WebAssembly runtime for embedded systems</p>
   </div>

.. warning::
   This documentation is for WRT version |version|. For other versions, use the version switcher in the navigation bar.

User Documentation
------------------

.. grid:: 3
   :gutter: 3

   .. grid-item-card:: Overview
      :link: overview/index
      :link-type: doc

      Product introduction, features, and status

   .. grid-item-card:: Getting Started
      :link: getting_started/index
      :link-type: doc

      Quick start guide and installation

   .. grid-item-card:: User Guide
      :link: user_guide/index
      :link-type: doc

      How to use WRT in your applications

   .. grid-item-card:: Examples
      :link: examples/index
      :link-type: doc

      Code examples and practical tutorials

   .. grid-item-card:: Platform Guides
      :link: platform_guides/index
      :link-type: doc

      Platform-specific usage guides

   .. grid-item-card:: Architecture
      :link: architecture/index
      :link-type: doc

      System design and component model

API Reference
-------------

.. grid:: 2
   :gutter: 3

   .. grid-item-card:: Core Runtime API
      :link: wrt-runtime/lib
      :link-type: doc

      Core runtime components and interfaces

   .. grid-item-card:: Component Model API
      :link: wrt-component/lib
      :link-type: doc

      Component model implementation

   .. grid-item-card:: Host Integration API
      :link: wrt-host/lib
      :link-type: doc

      Host function bindings and callbacks

   .. grid-item-card:: All Crates
      :link: api/index
      :link-type: doc

      Complete API documentation for all WRT crates

Qualification Material
----------------------

.. grid:: 2
   :gutter: 3

   .. grid-item-card:: Requirements
      :link: requirements/index
      :link-type: doc

      Functional, safety, and qualification requirements

   .. grid-item-card:: Qualification Plan
      :link: qualification/plan
      :link-type: doc

      ISO 26262 ASIL-D qualification approach

   .. grid-item-card:: Test Documentation
      :link: qualification/index
      :link-type: doc

      Test cases, coverage, and verification

   .. grid-item-card:: Safety Analysis
      :link: qualification/safety_analysis
      :link-type: doc

      FMEA, FTA, and safety assessments

Developer Documentation
------------------------

.. grid:: 3
   :gutter: 3

   .. grid-item-card:: Development Setup
      :link: developer/setup/index
      :link-type: doc

      Environment setup and toolchain installation

   .. grid-item-card:: Contributing
      :link: developer/contributing/index
      :link-type: doc

      Guidelines for contributing to WRT

   .. grid-item-card:: Build System
      :link: developer/build_system/index
      :link-type: doc

      Cargo workspace and build configuration

   .. grid-item-card:: Testing
      :link: developer/testing/index
      :link-type: doc

      Test strategies and coverage requirements

   .. grid-item-card:: Internals
      :link: developer/internals/index
      :link-type: doc

      Technical deep-dives and implementation details

   .. grid-item-card:: Tooling
      :link: developer/tooling/index
      :link-type: doc

      xtask commands and development tools

Reference Documentation
-----------------------

.. grid:: 2
   :gutter: 3

   .. grid-item-card:: Safety Guidelines
      :link: safety/index
      :link-type: doc

      Safety constraints, mechanisms, and best practices

   .. grid-item-card:: Binary Format
      :link: binary
      :link-type: doc

      WebAssembly binary format specifications

   .. grid-item-card:: Changelog
      :link: changelog.md
      :link-type: doc

      Release notes and version history

.. toctree::
   :hidden:
   :caption: User Documentation

   overview/index
   getting_started/index
   user_guide/index
   examples/index
   platform_guides/index
   architecture/index

.. toctree::
   :hidden:
   :caption: API Reference

   wrt-runtime/lib
   wrt-component/lib
   wrt-host/lib
   wrt-instructions/lib
   wrt-logging/lib
   wrt-safety/lib
   api/index

.. toctree::
   :hidden:
   :caption: Qualification

   requirements/index
   qualification/index
   safety_requirements
   safety_mechanisms
   safety_implementations
   safety_test_cases

.. toctree::
   :hidden:
   :caption: Developer Documentation

   developer/setup/index
   developer/contributing/index
   developer/build_system/index
   developer/testing/index
   developer/internals/index
   developer/tooling/index

.. toctree::
   :hidden:
   :caption: Reference

   safety/index
   binary
   changelog.md
   
.. include:: _generated_symbols.rst

.. include:: _generated_coverage_summary.rst

Indices and tables
==================

* :ref:`genindex`
* :ref:`search`