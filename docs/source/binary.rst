==================================
WebAssembly Binary Format in WRT
==================================

.. image:: _static/icons/validation_process.svg
   :width: 64px
   :align: right
   :alt: Validation Process Icon

This document describes the implementation of the WebAssembly Component Model Binary Format in WRT, following the `Component Model Binary Format specification <https://github.com/WebAssembly/component-model/blob/main/design/mvp/Binary.md>`_.

.. contents:: Table of Contents
   :local:
   :depth: 2

Core WebAssembly Format
=======================

.. image:: _static/icons/fuel_execution.svg
   :width: 32px
   :align: left
   :alt: Fuel-Based Execution Icon

WRT implements the Core WebAssembly binary format as specified in the `WebAssembly specification <https://webassembly.github.io/spec/core/bikeshed/>`_. The binary format begins with the magic number ``\0asm`` followed by the version ``\1\0\0\0``.

.. code-block:: text

   magic     ::= 0x00 0x61 0x73 0x6D
   version   ::= 0x01 0x00 0x00 0x00

Component Model Format Overview
===============================

Component Definitions
---------------------

The Component Model binary format in WRT follows this structure:

.. code-block:: text

   component ::= <preamble> s*:<section>*            => (component flatten(s*))
   preamble  ::= <magic> <version> <layer>
   magic     ::= 0x00 0x61 0x73 0x6D
   version   ::= 0x0D 0x00
   layer     ::= 0x01 0x00

Section types are defined as:

.. code-block:: text

   section   ::=    section_0(<core:custom>)         => ϵ
               | m: section_1(<core:module>)         => [core-prefix(m)]
               | i*:section_2(vec(<core:instance>))  => core-prefix(i)*
               | t*:section_3(vec(<core:type>))      => core-prefix(t)*
               | c: section_4(<component>)           => [c]
               | i*:section_5(vec(<instance>))       => i*
               | a*:section_6(vec(<alias>))          => a*
               | t*:section_7(vec(<type>))           => t*
               | c*:section_8(vec(<canon>))          => c*
               | s: section_9(<start>)               => [s]
               | i*:section_10(vec(<import>))        => i*
               | e*:section_11(vec(<export>))        => e*
               | v*:section_12(vec(<value>))         => v*

Current Implementation Status
-----------------------------

The WRT implementation differs from the specification in several key aspects:

**Version Field Discrepancy**

The specification defines:

.. code-block:: text

   version   ::= 0x0D 0x00
   layer     ::= 0x01 0x00

But WRT implements:

.. code-block:: text

   // Component Model binary format version - version 0.1
   COMPONENT_VERSION: [0x01, 0x00, 0x00, 0x01]

This means WRT uses a 4-byte field structured as "version + layer", with the first 2 bytes representing the version (0x01, 0x00) and the last 2 bytes representing the layer (0x00, 0x01).

Instance Definitions
====================

Core Instance Definitions
-------------------------

The specification defines:

.. code-block:: text

   core:instance       ::= ie:<core:instanceexpr>                             => (instance ie)
   core:instanceexpr   ::= 0x00 m:<moduleidx> arg*:vec(<core:instantiatearg>) => (instantiate m arg*)
                         | 0x01 e*:vec(<core:inlineexport>)                   => e*
   core:instantiatearg ::= n:<core:name> 0x12 i:<instanceidx>                 => (with n (instance i))
   core:sortidx        ::= sort:<core:sort> idx:<u32>                         => (sort idx)
   core:sort           ::= 0x00                                               => func
                         | 0x01                                               => table
                         | 0x02                                               => memory
                         | 0x03                                               => global
                         | 0x10                                               => type
                         | 0x11                                               => module
                         | 0x12                                               => instance
   core:inlineexport   ::= n:<core:name> si:<core:sortidx>                    => (export n si)

WRT implements the core sort values as constants:

.. code-block:: text

   COMPONENT_CORE_SORT_FUNC: 0x00
   COMPONENT_CORE_SORT_TABLE: 0x01
   COMPONENT_CORE_SORT_MEMORY: 0x02
   COMPONENT_CORE_SORT_GLOBAL: 0x03
   COMPONENT_CORE_SORT_TYPE: 0x10
   COMPONENT_CORE_SORT_MODULE: 0x11
   COMPONENT_CORE_SORT_INSTANCE: 0x12

The data structure in WRT:

.. code-block:: text

   pub enum CoreInstanceExpr {
       /// Instantiate a core module
       Instantiate {
           /// Module index
           module_idx: u32,
           /// Instantiation arguments
           args: Vec<CoreInstantiateArg>,
       },
       /// Collection of inlined exports
       InlineExports(Vec<CoreInlineExport>),
   }

Component Instance Definitions
------------------------------

The specification defines:

.. code-block:: text

   instance            ::= ie:<instanceexpr>                                  => (instance ie)
   instanceexpr        ::= 0x00 c:<componentidx> arg*:vec(<instantiatearg>)   => (instantiate c arg*)
                         | 0x01 e*:vec(<inlineexport>)                        => e*
   instantiatearg      ::= n:<name>  si:<sortidx>                             => (with n si)
   name                ::= n:<core:name>                                      => n
   sortidx             ::= sort:<sort> idx:<u32>                              => (sort idx)
   sort                ::= 0x00 cs:<core:sort>                                => core cs
                         | 0x01                                               => func
                         | 0x02                                               => value
                         | 0x03                                               => type
                         | 0x04                                               => component
                         | 0x05                                               => instance
   inlineexport        ::= n:<exportname> si:<sortidx>                        => (export n si)

WRT implements these sort values as constants:

.. code-block:: text

   COMPONENT_SORT_CORE: 0x00
   COMPONENT_SORT_FUNC: 0x01
   COMPONENT_SORT_VALUE: 0x02
   COMPONENT_SORT_TYPE: 0x03
   COMPONENT_SORT_COMPONENT: 0x04
   COMPONENT_SORT_INSTANCE: 0x05

Component Type Definitions
==========================

The WRT implementation provides support for the following component type definitions with data structures in ``wrt-format/src/component.rs``:

.. code-block:: text

   pub enum ComponentTypeDefinition {
       /// Component type
       Component {
           /// Component imports
           imports: Vec<(String, String, ExternType)>,
           /// Component exports
           exports: Vec<(String, ExternType)>,
       },
       /// Instance type
       Instance {
           /// Instance exports
           exports: Vec<(String, ExternType)>,
       },
       /// Function type
       Function {
           /// Parameter types
           params: Vec<(String, ValType)>,
           /// Result types
           results: Vec<ValType>,
       },
       /// Value type
       Value(ValType),
       /// Resource type
       Resource {
           /// Resource representation type
           representation: ResourceRepresentation,
           /// Whether the resource is nullable
           nullable: bool,
       },
   }

This implements the specification's component type definitions, though the binary parsing is not yet complete for all types.

Value Types
-----------

The WRT implementation supports the following value types:

.. code-block:: text

   pub enum ValType {
       /// Boolean type
       Bool,
       /// 8-bit signed integer
       S8,
       /// 8-bit unsigned integer
       U8,
       /// 16-bit signed integer
       S16,
       /// 16-bit unsigned integer
       U16,
       /// 32-bit signed integer
       S32,
       /// 32-bit unsigned integer
       U32,
       /// 64-bit signed integer
       S64,
       /// 64-bit unsigned integer
       U64,
       /// 32-bit float
       F32,
       /// 64-bit float
       F64,
       /// Unicode character
       Char,
       /// String type
       String,
       /// Reference type
       Ref(u32),
       /// Record type with named fields
       Record(Vec<(String, ValType)>),
       /// Variant type
       Variant(Vec<(String, Option<ValType>)>),
       /// List type
       List(Box<ValType>),
       /// Tuple type
       Tuple(Vec<ValType>),
       /// Flags type
       Flags(Vec<String>),
       /// Enum type
       Enum(Vec<String>),
       /// Option type
       Option(Box<ValType>),
       /// Result type (ok only)
       Result(Box<ValType>),
       /// Result type (error only)
       ResultErr(Box<ValType>),
       /// Result type (ok and error)
       ResultBoth(Box<ValType>, Box<ValType>),
       /// Own a resource
       Own(u32),
       /// Borrow a resource
       Borrow(u32),
   }

Alias Definitions
=================

The specification defines various forms of aliases, and WRT implements them as:

.. code-block:: text

   pub enum AliasTarget {
       /// Core WebAssembly export from an instance
       CoreInstanceExport {
           /// Instance index
           instance_idx: u32,
           /// Export name
           name: String,
           /// Kind of the target
           kind: CoreSort,
       },
       /// Export from a component instance
       InstanceExport {
           /// Instance index
           instance_idx: u32,
           /// Export name
           name: String,
           /// Kind of the target
           kind: Sort,
       },
       /// Outer definition from an enclosing component (forwarding from parent)
       Outer {
           /// Count of components to traverse outward
           count: u32,
           /// Kind of the target
           kind: Sort,
           /// Index within the kind
           idx: u32,
       },
   }

This differs slightly from the specification, which has more detailed alias forms.

Canonical Function Definitions
==============================

WRT implements canonical function operations:

.. code-block:: text

   pub enum CanonOperation {
       /// Lift a core function to the component ABI
       Lift {
           /// Core function index
           func_idx: u32,
           /// Type index for the lifted function
           type_idx: u32,
           /// Options for lifting
           options: LiftOptions,
       },
       /// Lower a component function to the core ABI
       Lower {
           /// Component function index
           func_idx: u32,
           /// Options for lowering
           options: LowerOptions,
       },
       /// Resource operations
       Resource(ResourceOperation),
   }

Start Definitions
=================

The specification defines:

.. code-block:: text

   start ::= f:<funcidx> arg*:vec(<valueidx>) r:<u32> => (start f (value arg)* (result (value))ʳ)

WRT implements this as:

.. code-block:: text

   pub struct Start {
       /// Function index
       pub func_idx: u32,
       /// Value arguments
       pub args: Vec<u32>,
       /// Number of results
       pub results: u32,
   }

However, the parsing is currently incomplete in WRT, as indicated by the implementation in ``parse_start_section`` which returns a not implemented error.

Import and Export Definitions
=============================

WRT implements imports and exports with these structures:

.. code-block:: text

   pub struct Import {
       /// Import name in namespace.name format
       pub name: ImportName,
       /// Type of the import
       pub ty: ExternType,
   }

   pub struct Export {
       /// Export name in "name" format
       pub name: ExportName,
       /// Sort of the exported item
       pub sort: Sort,
       /// Index within the sort
       pub idx: u32,
       /// Declared type (optional)
       pub ty: Option<ExternType>,
   }

These implement the specification imports and exports, though with some differences in the naming metadata structure.

Value Definitions
=================

WRT implements a Value structure, though the binary parsing is still incomplete:

.. code-block:: text

   pub struct Value {
       /// Type of the value
       pub ty: ValType,
       /// Encoded value data
       pub data: Vec<u8>,
   }

The specification defines more detailed value encoding rules which are not yet fully implemented.

Section Parsing Process
=======================

The decoding process in ``wrt-decoder/src/component/decode.rs`` follows these steps:

1. Verify the magic number (``\0asm``)
2. Read the version field
3. Iterate through sections:
   a. Read section ID and size
   b. Extract section bytes
   c. Parse section based on ID

Each section type has a corresponding parser in ``wrt-decoder/src/component/parse.rs``, but many of these are currently placeholders that don't fully implement the specification.

Binary Format Constants
=======================

The binary format constants are defined in ``wrt-format/src/binary.rs``:

.. code-block:: text

   // Component Model magic bytes (same as core: \0asm)
   COMPONENT_MAGIC: [0x00, 0x61, 0x73, 0x6D]

   // Component Model binary format version - version 0.1
   COMPONENT_VERSION: [0x01, 0x00, 0x00, 0x01]

   // Component Model version only (first two bytes of version)
   COMPONENT_VERSION_ONLY: [0x01, 0x00]

   // Component Model layer identifier - distinguishes components from modules
   COMPONENT_LAYER: [0x00, 0x01]

   // Component Model section IDs
   COMPONENT_CUSTOM_SECTION_ID: 0x00
   COMPONENT_CORE_MODULE_SECTION_ID: 0x01
   COMPONENT_CORE_INSTANCE_SECTION_ID: 0x02
   COMPONENT_CORE_TYPE_SECTION_ID: 0x03
   COMPONENT_COMPONENT_SECTION_ID: 0x04
   COMPONENT_INSTANCE_SECTION_ID: 0x05
   COMPONENT_ALIAS_SECTION_ID: 0x06
   COMPONENT_TYPE_SECTION_ID: 0x07
   COMPONENT_CANON_SECTION_ID: 0x08
   COMPONENT_START_SECTION_ID: 0x09
   COMPONENT_IMPORT_SECTION_ID: 0x0A
   COMPONENT_EXPORT_SECTION_ID: 0x0B
   COMPONENT_VALUE_SECTION_ID: 0x0C

Name Section Implementation
============================

The specification defines a name section for components, similar to the core WebAssembly name section. The WRT implementation has a partial implementation in ``wrt-decoder/src/component_name_section.rs`` but with some discrepancies:

The specification defines:

.. code-block:: text

   namesec    ::= section_0(namedata)
   namedata   ::= n:<name>                (if n = 'component-name')
                  name:<componentnamesubsec>?
                  sortnames*:<sortnamesubsec>*
   namesubsection_N(B) ::= N:<byte> size:<u32> B     (if size == |B|)

   componentnamesubsec ::= namesubsection_0(<name>)
   sortnamesubsec ::= namesubsection_1(<sortnames>)
   sortnames ::= sort:<sort> names:<namemap>

   namemap ::= names:vec(<nameassoc>)
   nameassoc ::= idx:<u32> name:<name>

Current Implementation Differences Summary
==========================================

1. **Version Implementation**: WRT uses a 4-byte version field ``[0x01, 0x00, 0x00, 0x01]`` while the specification separates this into a 2-byte version field ``[0x0D, 0x00]`` followed by a 2-byte layer field ``[0x01, 0x00]``.

2. **Placeholder Implementations**: Many section parsers are currently placeholder implementations that will be fully implemented in future versions:
   - ``parse_core_module_section``
   - ``parse_core_instance_section``
   - ``parse_core_type_section``
   - ``parse_component_section``
   - ``parse_instance_section``
   - ``parse_canon_section``
   - ``parse_component_type_section``
   - ``parse_start_section``
   - ``parse_import_section``
   - ``parse_export_section``
   - ``parse_value_section``
   - ``parse_alias_section``

3. **Resource Types Implementation**: The resource type representation in WRT has a different structure than specified, with specific types for handle32, handle64, record, and aggregate.

4. **Start Function Implementation**: The start function section is defined in the data structure but parsing is explicitly not implemented yet.

5. **Value Encoding/Decoding**: The specification defines detailed value encoding rules which are not yet fully implemented in WRT.

6. **Name Section Implementation**: The name section implementation in WRT differs from the specification in structure and completeness.

7. **Validation**: The specification requires detailed validation of each section's contents which is not yet fully implemented in WRT.

Future Work
===========

The WRT implementation of the Component Model binary format is under active development. Future work includes:

1. Complete implementation of all section parsers
2. Updating the version field structure to match the specification
3. Full validation according to the specification
4. Complete implementation of value encoding/decoding
5. Resource type handling improvements
6. Name section implementation according to specification
7. Support for experimental features marked with 🪙 in the specification
8. Optimization of parsing and validation 