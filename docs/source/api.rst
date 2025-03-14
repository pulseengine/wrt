API Reference
=============

This section provides documentation for the WRT API.

Core Module
-----------

The core module provides the basic WebAssembly runtime functionality.

.. code-block:: rust

   use wrt::Module;
   
   let module = Module::new();
   let instance = module.instantiate(imports)?;

Values
------

WebAssembly values are represented by the ``Value`` enum:

.. code-block:: rust

   use wrt::Value;
   
   let i32_val = Value::I32(42);
   let i64_val = Value::I64(9000);
   let f32_val = Value::F32(3.14);
   let f64_val = Value::F64(2.71);

Memory
------

WebAssembly memory is represented by the ``Memory`` struct:

.. code-block:: rust

   use wrt::{Memory, MemoryType};
   
   let memory_type = MemoryType::new(1, Some(2));
   let memory = Memory::new(memory_type);
   
   // Read and write to memory
   memory.write(0, &[1, 2, 3, 4])?;
   let data = memory.read(0, 4)?;

Tables
------

WebAssembly tables are represented by the ``Table`` struct:

.. code-block:: rust

   use wrt::{Table, TableType, Value};
   
   let table_type = TableType::new(Value::type_funcref(), 1, Some(10));
   let table = Table::new(table_type);
   
   // Set and get elements
   table.set(0, Value::FuncRef(Some(1)))?;
   let elem = table.get(0)?;

Component Model
---------------

WRT supports the WebAssembly Component Model:

.. code-block:: rust

   use wrt::{Component, ComponentType};
   
   let component_type = ComponentType { /* ... */ };
   let component = Component::new(component_type);
   let instance = component.instantiate(imports)?;