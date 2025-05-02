Traceability Matrix
===================

This document provides traceability between requirements, specifications, implementations, and test cases.

Overview
--------

The traceability matrix maps relationships between different artifact types to ensure complete requirements coverage and verification.

Requirements to Specifications
------------------------------

This section shows how requirements are addressed by specifications.

.. needflow::
   :types: req, spec
   :show_link_names:

Specifications to Implementations
---------------------------------

This section shows how specifications are implemented.

.. needflow::
   :types: spec, impl
   :show_link_names:

Safety Requirements Tracing
---------------------------

This section specifically traces safety requirements to their implementations.

.. needflow::
   :types: req, spec, impl, safety
   :show_link_names:

Complete Requirement Coverage
-----------------------------

This table shows the complete mapping of requirements to their corresponding specifications and implementations.

.. needtable::
   :columns: id;title;status;links
   :filter: type == "req"

All Specifications
------------------

This table lists all specifications and their implementation status.

.. needtable::
   :columns: id;title;status;links
   :filter: type == "spec"

All Implementations
-------------------

This table lists all implementation details.

.. needtable::
   :columns: id;title;status;links
   :filter: type == "impl"

Qualification Requirements Coverage
-----------------------------------

This section shows the traceability for qualification-specific requirements.

.. needtable::
   :columns: id;title;status;links
   :filter: id in ['QUAL_001', 'QUAL_002', 'QUAL_003', 'QUAL_004', 'QUAL_005', 'QUAL_006', 'QUAL_007', 'QUAL_008'] 