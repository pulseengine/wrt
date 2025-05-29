================
Product Overview
================

.. image:: ../_static/icons/logo.svg
   :width: 120px
   :align: right
   :alt: WRT Logo

Welcome to the WRT documentation. This section provides an overview of the product, its features, and architecture.

.. contents:: On this page
   :local:
   :depth: 2

Introduction
------------

**WRT (WebAssembly Runtime): Precision Runtime for Mission-Critical Systems**

WRT is a pure Rust implementation of a WebAssembly runtime supporting both the core WebAssembly specification and the WebAssembly Component Model. It is engineered for environments where every cycle matters, delivering deterministic behavior, continuous oversight, and relentless precision, all essential for systems in IoT, medicine, automotive, and avionics.

Key Capabilities
----------------

- **Interpretation at Its Core**: WRT interprets WebAssembly code with deterministic execution
- **Continuous Monitoring**: Built-in real-time checks to capture anomalies early
- **Steady Throughput**: Consistent performance guarantees with precise timing
- **Deterministic Execution**: Every cycle is predictable and verifiable
- **WebAssembly Core & Component Model**: Full implementation of specifications
- **Safety Features**: Stackless design, bounded execution, and state migration

Product Status
--------------

Requirement Status
------------------

.. commenting out needpie directives until they can be fixed
..
.. .. needpie::
..    :labels: Implemented, Partial, Not Started
..    :filter: id =~ "REQ_.*" and status != "removed"

See :doc:`../requirements/index` for detailed requirements and their implementation status. 