==========================
Interface Catalog
==========================

**Teaching Point**: Interfaces define contracts between components. This catalog shows the actual interfaces implemented in the codebase.

Core Runtime Interfaces
-----------------------

Engine Behavior Interface
~~~~~~~~~~~~~~~~~~~~~~~~~

.. arch_interface:: Engine Behavior
   :id: ARCH_IF_001
   :component: ARCH_COMP_001
   :file: wrt/src/behavior.rs
   :type: provided
   :stability: stable

**Purpose**: Defines how execution engines behave.

**Actual Implementation**:

.. code-block:: rust

   pub trait EngineBehavior: StackBehavior + FrameBehavior {
       type ModuleInstanceType: ModuleBehavior;
       
       fn new_module(&mut self, module: Module) -> WrtResult<ModuleInstanceIndex>;
       fn get_module_instance(&self, instance_idx: ModuleInstanceIndex) -> Option<&Self::ModuleInstanceType>;
       fn get_module_instance_mut(&mut self, instance_idx: ModuleInstanceIndex) -> Option<&mut Self::ModuleInstanceType>;
       fn instantiate(&mut self, module_idx: ModuleInstanceIndex) -> WrtResult<ModuleInstanceIndex>;
       fn execute(&mut self, instance_idx: ModuleInstanceIndex, func_idx: FuncIdx, args: Vec<Value>) -> WrtResult<Vec<Value>>;
   }

**Environment Variations**:

- **std**: Thread-safe with `Arc<Mutex<T>>`
- **no_std + alloc**: Single-threaded with `RefCell`
- **no_std + no_alloc**: Static dispatch with bounded instance pool

Memory Provider Interface
~~~~~~~~~~~~~~~~~~~~~~~~~

.. arch_interface:: Memory Provider
   :id: ARCH_IF_020
   :component: ARCH_COMP_002
   :file: wrt-foundation/src/traits.rs
   :type: provided
   :stability: stable

**Purpose**: Abstracts memory allocation across environments.

**Actual Trait Definition**:

.. code-block:: rust

   pub trait MemoryProvider: Clone + PartialEq + Eq {
       type Allocator: Allocator;
       
       fn len(&self) -> usize;
       fn read_bytes(&self, offset: usize, length: usize) -> Result<&[u8]>;
       fn write_bytes(&mut self, offset: usize, data: &[u8]) -> Result<()>;
       fn read_slice(&self, offset: usize, length: usize) -> Result<Slice<Self>>;
       fn write_slice(&mut self, offset: usize, length: usize) -> Result<SliceMut<Self>>;
       fn resize(&mut self, new_len: usize) -> Result<()>;
       fn copy_within(&mut self, src_offset: usize, dst_offset: usize, len: usize) -> Result<()>;
   }

**Implementations**:

.. list-table:: Memory Provider Implementations
   :header-rows: 1

   * - Environment
     - Implementation
     - Characteristics
   * - std
     - ``StdProvider``
     - Dynamic sizing with ``Vec<u8>``
   * - no_std + alloc
     - ``AllocProvider``
     - Uses global allocator
   * - no_std + no_alloc
     - ``NoStdProvider<const N: usize>``
     - Fixed size ``[u8; N]``

Component Model Interfaces
--------------------------

Component Instance Interface
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

.. arch_interface:: Component Instance
   :id: ARCH_IF_030
   :component: ARCH_COMP_003
   :file: wrt-component/src/component_traits.rs
   :type: provided
   :stability: stable

**Actual Trait**:

.. code-block:: rust

   pub trait ComponentInstance {
       fn new(runtime: Arc<dyn ComponentRuntime>) -> WrtResult<Self> where Self: Sized;
       fn add_import(&mut self, name: String, instance: Arc<dyn ComponentInstance>) -> WrtResult<()>;
       fn get_export(&self, name: &str) -> Option<Arc<dyn Any>>;
       fn instantiate(&mut self) -> WrtResult<()>;
   }

Host Function Interface
~~~~~~~~~~~~~~~~~~~~~~~

.. arch_interface:: Host Function
   :id: ARCH_IF_031
   :component: ARCH_COMP_003
   :file: wrt-component/src/component_traits.rs
   :type: required
   :stability: stable

**Purpose**: Allows host environment to provide functions to WASM.

.. code-block:: rust

   pub trait HostFunction: Send + Sync {
       fn call(&self, args: &[Value]) -> WrtResult<Vec<Value>>;
       fn signature(&self) -> &FuncType;
   }

Platform Abstraction Interfaces
-------------------------------

Page Allocator Interface
~~~~~~~~~~~~~~~~~~~~~~~~

.. arch_interface:: Page Allocator
   :id: ARCH_IF_050
   :component: ARCH_COMP_005
   :file: wrt-platform/src/memory.rs
   :type: provided
   :stability: stable

**Teaching Point**: This abstracts memory page management across different OSes:

.. code-block:: rust

   pub trait PageAllocator: Send + Sync {
       fn allocate(&mut self, pages: usize) -> Result<*mut u8, Error>;
       fn deallocate(&mut self, ptr: *mut u8, pages: usize) -> Result<(), Error>;
       fn grow(&mut self, ptr: *mut u8, old_pages: usize, new_pages: usize) -> Result<*mut u8, Error>;
       fn protect(&mut self, ptr: *mut u8, pages: usize, prot: Protection) -> Result<(), Error>;
   }

**Platform Implementations**:

- Linux: ``mmap``/``munmap`` with ``PROT_MTE`` support
- macOS: ``mmap`` with guard pages
- QNX: Arena allocator with partitions
- Bare-metal: Static buffer allocation

Synchronization Interface
~~~~~~~~~~~~~~~~~~~~~~~~~

.. arch_interface:: Futex-like Operations
   :id: ARCH_IF_051
   :component: ARCH_COMP_005
   :file: wrt-platform/src/sync.rs
   :type: provided
   :stability: stable

.. code-block:: rust

   pub trait FutexLike: Send + Sync {
       fn wait(&self, addr: &AtomicU32, expected: u32, timeout: Option<Duration>) -> Result<(), Error>;
       fn wake(&self, addr: &AtomicU32, count: u32) -> Result<u32, Error>;
   }

Internal Interfaces
-------------------

Instruction Traits
~~~~~~~~~~~~~~~~~~

.. arch_interface:: Pure Instruction
   :id: ARCH_IF_060
   :component: ARCH_COMP_011
   :file: wrt-instructions/src/instruction_traits.rs
   :type: internal
   :stability: stable

**Purpose**: Common behavior for all instructions.

.. code-block:: rust

   pub trait PureInstruction {
       fn execute<C: InstructionContext>(&self, context: &mut C) -> Result<(), Error>;
       fn get_opcode(&self) -> u8;
   }

Verification Interface
~~~~~~~~~~~~~~~~~~~~~~

.. arch_interface:: Validatable
   :id: ARCH_IF_070
   :component: ARCH_COMP_002
   :file: wrt-foundation/src/traits.rs
   :type: internal
   :stability: stable

.. code-block:: rust

   pub trait Validatable {
       fn validate(&self) -> Result<(), ValidationError>;
       fn validate_with_level(&self, level: VerificationLevel) -> Result<(), ValidationError>;
   }

Interface Compatibility Matrix
------------------------------

.. list-table:: Feature-Based Interface Availability
   :header-rows: 1

   * - Interface
     - std
     - no_std + alloc
     - no_std + no_alloc
   * - EngineBehavior
     - ✓ Full
     - ✓ Full
     - ✓ Limited instances
   * - MemoryProvider
     - ✓ Dynamic
     - ✓ Dynamic
     - ✓ Static only
   * - ComponentInstance
     - ✓ Full
     - ✓ Full
     - ✓ Bounded
   * - PageAllocator
     - ✓ OS-based
     - ✓ OS-based
     - ✓ Static
   * - FutexLike
     - ✓ Native
     - ✓ Emulated
     - ✗ Spin-only

Cross-References
----------------

- **Component Definitions**: See :doc:`../01_architectural_design/components`
- **API Contracts**: See :doc:`api_contracts`
- **Usage Examples**: See component-specific examples in :doc:`/examples/index`