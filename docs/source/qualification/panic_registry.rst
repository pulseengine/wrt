.. _panic-registry:

Panic Registry
==============

This document contains all documented panic conditions in the WRT codebase.
Each panic is tracked as a qualification requirement using sphinx-needs.

.. contents:: Table of Contents
   :local:
   :depth: 2

Summary
-------

* Total panic points: 10
* Status:
  * Todo: 10
  * In Progress: 0
  * Resolved: 0

The original CSV version of this registry is maintained at:
docs/source/development/panic_registry.csv

.. csv-table:: Panic Registry CSV
   :file: panic_registry.csv
   :header-rows: 1
   :widths: 20, 15, 5, 20, 5, 10, 10, 15

Panic Details
------------

.. qual:: f32_nearest
   :id: WRTQ-XXX (qualification requirement tracking ID).
   :status: Todo
   :implementation: 
   :tags: panic, unknown

   **File:** wrt/src/execution.rs
   **Line:** 389
   **Function:** f32_nearest
   **Safety Impact:** [LOW|MEDIUM|HIGH] - [Brief explanation of the safety implication]
   **Last Updated:** 2025-04-25

   This function will panic if the provided value is not an F32 value.

.. qual:: f64_nearest
   :id: WRTQ-XXX (qualification requirement tracking ID).
   :status: Todo
   :implementation: 
   :tags: panic, unknown

   **File:** wrt/src/execution.rs
   **Line:** 425
   **Function:** f64_nearest
   **Safety Impact:** [LOW|MEDIUM|HIGH] - [Brief explanation of the safety implication]
   **Last Updated:** 2025-04-25

   This function will panic if the provided value is not an F64 value.

.. qual:: new
   :id: WRTQ-XXX (qualification requirement tracking ID).
   :status: Todo
   :implementation: 
   :tags: panic, unknown

   **File:** wrt-sync/src/mutex.rs
   **Line:** 53
   **Function:** new
   **Safety Impact:** [LOW|MEDIUM|HIGH] - [Brief explanation of the safety implication]
   **Last Updated:** 2025-04-25

   This function does not panic.

.. qual:: new
   :id: WRTQ-XXX (qualification requirement tracking ID).
   :status: Todo
   :implementation: 
   :tags: panic, unknown

   **File:** wrt-types/src/safe_memory.rs
   **Line:** 50
   **Function:** new
   **Safety Impact:** [LOW|MEDIUM|HIGH] - [Brief explanation of the safety implication]
   **Last Updated:** 2025-04-25

   This function will panic if the initial integrity verification fails. This can happen if memory corruption is detected during initialization.

.. qual:: push
   :id: WRTQ-XXX (qualification requirement tracking ID).
   :status: Todo
   :implementation: Return Result instead of panic
   :tags: panic, unknown

   **File:** wrt-types/src/bounded.rs
   **Line:** 196
   **Function:** push
   **Safety Impact:** [LOW|MEDIUM|HIGH] - [Brief explanation of the safety implication]
   **Last Updated:** 2025-04-25

   This function does not panic.

.. qual:: encode
   :id: WRTQ-XXX (qualification requirement tracking ID).
   :status: WRTQ-XXX (qualification requirement tracking ID).
   :implementation: WRTQ-XXX (qualification requirement tracking ID).
   :tags: panic, unknown

   **File:** wrt-decoder/src/module.rs
   **Line:** 214
   **Function:** encode
   **Safety Impact:** [LOW|MEDIUM|HIGH] - [Brief explanation of the safety implication]
   **Last Updated:** 2025-04-25

   This function will panic if it attempts to access the last element of an empty custom_sections vector, which can happen if the implementation tries to process a custom section before any custom sections have been added to the module.

.. qual:: buffer
   :id: WRTQ-XXX (qualification requirement tracking ID).
   :status: WRTQ-XXX (qualification requirement tracking ID).
   :implementation: WRTQ-XXX (qualification requirement tracking ID).
   :tags: panic, unknown

   **File:** wrt-runtime/src/memory.rs
   **Line:** 229
   **Function:** buffer
   **Safety Impact:** [LOW|MEDIUM|HIGH] - [Brief explanation of the safety implication]
   **Last Updated:** 2025-04-25

   In `no_std` environments, this method will panic if the read lock for the metrics cannot be acquired. This would typically only happen in case of a deadlock or if the lock is poisoned due to a panic in another thread holding the lock.

.. qual:: peak_memory
   :id: WRTQ-XXX (qualification requirement tracking ID).
   :status: WRTQ-XXX (qualification requirement tracking ID).
   :implementation: WRTQ-XXX (qualification requirement tracking ID).
   :tags: panic, unknown

   **File:** wrt-runtime/src/memory.rs
   **Line:** 251
   **Function:** peak_memory
   **Safety Impact:** [LOW|MEDIUM|HIGH] - [Brief explanation of the safety implication]
   **Last Updated:** 2025-04-25

   In `no_std` environments, this method will panic if the read lock for the metrics cannot be acquired. This would typically only happen in case of a deadlock or if the lock is poisoned due to a panic in another thread holding the lock.

.. qual:: access_count
   :id: WRTQ-XXX (qualification requirement tracking ID).
   :status: WRTQ-XXX (qualification requirement tracking ID).
   :implementation: WRTQ-XXX (qualification requirement tracking ID).
   :tags: panic, unknown

   **File:** wrt-runtime/src/memory.rs
   **Line:** 273
   **Function:** access_count
   **Safety Impact:** [LOW|MEDIUM|HIGH] - [Brief explanation of the safety implication]
   **Last Updated:** 2025-04-25

   In `no_std` environments, this method will panic if the write lock for the metrics cannot be acquired. This would typically only happen in case of a deadlock or if the lock is poisoned due to a panic in another thread holding the lock.

.. qual:: increment_access_count
   :id: WRTQ-XXX (qualification requirement tracking ID).
   :status: WRTQ-XXX (qualification requirement tracking ID).
   :implementation: WRTQ-XXX (qualification requirement tracking ID).
   :tags: panic, unknown

   **File:** wrt-runtime/src/memory.rs
   **Line:** 294
   **Function:** increment_access_count
   **Safety Impact:** [LOW|MEDIUM|HIGH] - [Brief explanation of the safety implication]
   **Last Updated:** 2025-04-25

   In `no_std` environments, this method will panic if the write lock for the metrics cannot be acquired. This would typically only happen in case of a deadlock or if the lock is poisoned due to a panic in another thread holding the lock.

