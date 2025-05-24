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

.. grid:: 2
   :gutter: 3

   .. grid-item-card:: Getting Started
      :link: getting_started/index
      :link-type: doc

      Installation guides for supported platforms including Linux, macOS, QNX, Zephyr, and bare-metal environments

   .. grid-item-card:: Architecture Guide
      :link: architecture/index
      :link-type: doc

      System design, component model, and runtime architecture

   .. grid-item-card:: Safety Guidelines
      :link: safety/index
      :link-type: doc

      Safety constraints, mechanisms, and best practices

   .. grid-item-card:: Binary Format
      :link: binary
      :link-type: doc

      WebAssembly binary format specifications

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

Developer Resources
-------------------

.. grid:: 2
   :gutter: 3

   .. grid-item-card:: Development Guide
      :link: development/index
      :link-type: doc

      Contributing guidelines and development setup

   .. grid-item-card:: Examples
      :link: examples/index
      :link-type: doc

      Code examples and tutorials

   .. grid-item-card:: Changelog
      :link: changelog
      :link-type: doc

      Release notes and version history

.. toctree::
   :hidden:
   :caption: User Documentation

   getting_started/index
   overview/index
   architecture/index
   safety/index
   binary

.. toctree::
   :hidden:
   :caption: API Reference

   wrt-runtime/lib
   wrt-component/lib
   wrt-host/lib
   wrt-instructions/lib
   wrt-logging/lib
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
   :caption: Resources

   examples/index
   development/index
   changelog
   
.. include:: _generated_symbols.rst

.. include:: _generated_coverage_summary.rst

Indices and tables
==================

* :ref:`genindex`
* :ref:`search`