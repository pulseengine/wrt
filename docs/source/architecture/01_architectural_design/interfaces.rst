==========================
Architectural Interfaces
==========================

This document defines the key interfaces between architectural components using sphinx-needs for full traceability.

Core Runtime Interfaces
-----------------------

.. arch_interface:: Runtime Execution Interface
   :id: ARCH_IF_001
   :provided_by: ARCH_COMP_001
   :required_by: ARCH_COMP_003, ARCH_COMP_005
   :status: implemented
   
   Interface for executing WebAssembly instructions and managing execution state.
   
   .. code-block:: rust
   
      pub trait EngineBehavior {
          fn execute_instruction(&mut self, instr: Instruction) -> Result<()>;
          fn get_module_instance(&self, idx: usize) -> Option<&ModuleInstance>;
          fn push_frame(&mut self, frame: Frame) -> Result<()>;
          fn pop_frame(&mut self) -> Result<Frame>;
      }

.. arch_interface:: Module Management Interface
   :id: ARCH_IF_002
   :provided_by: ARCH_COMP_001
   :required_by: ARCH_COMP_003
   :status: implemented
   
   Interface for instantiating and managing WebAssembly modules.
   
   .. code-block:: rust
   
      pub trait ModuleInstanceManager {
          fn instantiate(module: Module, imports: Imports) -> Result<Self>;
          fn get_export(&self, name: &str) -> Option<Export>;
          fn invoke(&mut self, name: &str, args: &[Value]) -> Result<Vec<Value>>;
      }

Memory Interfaces
-----------------

.. arch_interface:: Memory Provider Interface
   :id: ARCH_IF_020
   :provided_by: ARCH_COMP_002
   :required_by: ARCH_COMP_001, ARCH_COMP_004
   :status: implemented
   :links: ARCH_REQ_004
   
   Safe memory access interface with bounds checking.
   
   .. code-block:: rust
   
      pub trait MemoryProvider: Clone + PartialEq + Eq {
          fn len(&self) -> usize;
          fn read_bytes(&self, offset: usize, length: usize) -> Result<&[u8]>;
          fn write_bytes(&mut self, offset: usize, data: &[u8]) -> Result<()>;
          fn validate_access(&self, offset: usize, length: usize) -> Result<()>;
      }

.. arch_interface:: Linear Memory Interface
   :id: ARCH_IF_021
   :provided_by: ARCH_COMP_002
   :required_by: ARCH_COMP_001
   :status: implemented
   
   WebAssembly linear memory operations.
   
   .. code-block:: rust
   
      pub trait LinearMemory {
          fn size(&self) -> u32;  // In pages
          fn grow(&mut self, delta: u32) -> Result<u32>;
          fn data(&self) -> &[u8];
          fn data_mut(&mut self) -> &mut [u8];
      }

Component Model Interfaces
--------------------------

.. arch_interface:: Component Instance Interface
   :id: ARCH_IF_030
   :provided_by: ARCH_COMP_003
   :required_by: Application Layer
   :status: implemented
   
   High-level interface for Component Model instances.
   
   .. code-block:: rust
   
      pub trait ComponentInstance {
          fn instantiate(component: &Component, imports: Imports) -> Result<Self>;
          fn export(&self, name: &str) -> Option<Export>;
          fn call(&mut self, name: &str, args: &[Value]) -> Result<Vec<Value>>;
          fn get_resource(&self, handle: ResourceHandle) -> Option<&Resource>;
      }

.. arch_interface:: Canonical ABI Interface
   :id: ARCH_IF_031
   :provided_by: ARCH_COMP_003
   :required_by: ARCH_COMP_001
   :status: implemented
   
   Interface for canonical ABI value conversions.
   
   .. code-block:: rust
   
      pub trait CanonicalABI {
          fn lower_value(value: &ComponentValue, ty: &Type) -> Result<Vec<u8>>;
          fn lift_value(bytes: &[u8], ty: &Type) -> Result<ComponentValue>;
          fn lower_func_args(args: &[ComponentValue], params: &[Type]) -> Result<Vec<u8>>;
          fn lift_func_results(bytes: &[u8], results: &[Type]) -> Result<Vec<ComponentValue>>;
      }

Platform Abstraction Interfaces
-------------------------------

.. arch_interface:: Platform Memory Interface
   :id: ARCH_IF_040
   :provided_by: ARCH_COMP_010
   :required_by: ARCH_COMP_002
   :status: implemented
   :links: ARCH_REQ_003
   
   Platform-specific memory allocation and protection.
   
   .. code-block:: rust
   
      pub trait PlatformMemory {
          fn allocate(size: usize, protection: Protection) -> Result<*mut u8>;
          fn deallocate(ptr: *mut u8, size: usize) -> Result<()>;
          fn protect(ptr: *mut u8, size: usize, protection: Protection) -> Result<()>;
          fn query_protection(ptr: *const u8) -> Result<Protection>;
      }

.. arch_interface:: Platform Synchronization Interface
   :id: ARCH_IF_041
   :provided_by: ARCH_COMP_011
   :required_by: ARCH_COMP_001, ARCH_COMP_002
   :status: implemented
   
   Platform-specific synchronization primitives.
   
   .. code-block:: rust
   
      pub trait PlatformSync {
          type Mutex<T>: MutexTrait<T>;
          type RwLock<T>: RwLockTrait<T>;
          type Once: OnceTrait;
          
          fn create_mutex<T>(value: T) -> Self::Mutex<T>;
          fn create_rwlock<T>(value: T) -> Self::RwLock<T>;
          fn create_once() -> Self::Once;
      }

Host Integration Interfaces
---------------------------

.. arch_interface:: Host Function Interface
   :id: ARCH_IF_050
   :provided_by: ARCH_COMP_020
   :required_by: ARCH_COMP_001, ARCH_COMP_003
   :status: implemented
   
   Interface for host function callbacks.
   
   .. code-block:: rust
   
      pub trait HostFunction {
          fn signature(&self) -> &FunctionType;
          fn call(&self, args: &[Value], results: &mut [Value]) -> Result<()>;
          fn name(&self) -> &str;
      }

.. arch_interface:: Interceptor Interface
   :id: ARCH_IF_051
   :provided_by: ARCH_COMP_021
   :required_by: ARCH_COMP_001
   :status: implemented
   
   Interface for intercepting and monitoring execution.
   
   .. code-block:: rust
   
      pub trait ExecutionInterceptor {
          fn pre_instruction(&mut self, instr: &Instruction) -> InterceptResult;
          fn post_instruction(&mut self, instr: &Instruction, result: &Result<()>);
          fn pre_function_call(&mut self, name: &str, args: &[Value]) -> InterceptResult;
          fn post_function_call(&mut self, name: &str, result: &Result<Vec<Value>>);
      }

Decoder Interfaces
------------------

.. arch_interface:: Module Decoder Interface
   :id: ARCH_IF_010
   :provided_by: ARCH_COMP_004
   :required_by: ARCH_COMP_001
   :status: implemented
   
   Interface for decoding WebAssembly modules.
   
   .. code-block:: rust
   
      pub trait ModuleDecoder {
          fn decode(bytes: &[u8]) -> Result<Module>;
          fn validate(module: &Module) -> Result<()>;
          fn decode_section(section_id: u8, bytes: &[u8]) -> Result<Section>;
      }

.. arch_interface:: Component Decoder Interface
   :id: ARCH_IF_011
   :provided_by: ARCH_COMP_004
   :required_by: ARCH_COMP_003
   :status: implemented
   
   Interface for decoding WebAssembly components.
   
   .. code-block:: rust
   
      pub trait ComponentDecoder {
          fn decode(bytes: &[u8]) -> Result<Component>;
          fn validate(component: &Component) -> Result<()>;
          fn decode_type_section(bytes: &[u8]) -> Result<Vec<ComponentType>>;
      }

Interface Dependencies
----------------------

The following diagram illustrates the dependencies between interfaces:

.. needflow::
   :filter: type == 'arch_interface'

Interface Verification
----------------------

All interfaces are verified through:

1. **Unit Tests**: Each interface has comprehensive test coverage
2. **Integration Tests**: Interface interactions are tested end-to-end
3. **Formal Verification**: Critical interfaces use formal methods
4. **Performance Tests**: Interfaces meet timing requirements

Cross-References
----------------

- **Component Definitions**: :doc:`components` - Components that provide/require these interfaces
- **Dynamic Behavior**: :doc:`../04_dynamic_behavior/interaction_flows` - How interfaces interact at runtime
- **API Contracts**: :doc:`../03_interfaces/api_contracts` - Detailed API specifications