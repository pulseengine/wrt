PlantUML Test
=============

Testing if PlantUML rendering works in the documentation build.

Simple Test Diagram
-------------------

.. uml::

   @startuml
   actor User
   participant "WRT Runtime" as WRT
   database "WASM Module" as WASM
   
   User -> WRT: Execute module
   WRT -> WASM: Load binary
   WASM --> WRT: Module loaded
   WRT -> WRT: Validate
   WRT -> WRT: Instantiate
   WRT --> User: Execution result
   @enduml

Component Test
--------------

.. uml::

   @startuml
   component "Test Component" as TC {
       component "Module A" as MA
       component "Module B" as MB
   }
   
   MA --> MB : uses
   @enduml