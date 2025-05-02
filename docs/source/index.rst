.. wrt documentation master file, created by
   sphinx-quickstart on Sun Mar 17 00:48:53 2024.
   You can adapt this file completely to your liking, but it should at least
   contain the root `toctree` directive.

Welcome to SPE_wrt's documentation!
===================================

.. image:: _static/icons/logo.svg
   :width: 120px
   :align: center
   :alt: SentryPulse Engine (WRT Edition) Logo

.. toctree::
   :maxdepth: 1
   :caption: Main Navigation

   overview/index
   requirements/index
   binary
   requirements
   safety/index

.. toctree::
   :maxdepth: 1
   :caption: Additional Documentation
   :hidden:

   safety_requirements
   safety_mechanisms
   safety_implementations
   safety_test_cases
   qualification/index
   development/index
   api/index
   changelog

Quick Links
-----------

- :doc:`overview/features` - Product features and capabilities
- :doc:`requirements/safety` - Safety requirements
- :doc:`safety/constraints` - Safety constraints and guidelines
- :doc:`architecture` - System architecture
- :doc:`binary` - Binary format details
- :doc:`api/index` - API documentation

Project Status
--------------

Requirements Status
^^^^^^^^^^^^^^^^^^^^

.. commenting out needpie directives until they can be fixed
..
.. .. needpie::
..    :labels: Implemented, Partial, Not Started
..    :filter: id =~ "REQ_.*" and status != "removed"

.. include:: _generated_symbols.rst

.. include:: _generated_coverage_summary.rst

Indices and tables
==================

* :ref:`genindex`
* :ref:`modindex`
* :ref:`search`
