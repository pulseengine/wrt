(module
  ;; Import a function from the environment
  (import "env" "print_i32" (func $print_i32 (param i32)))
  
  ;; Import a memory from the environment
  (import "env" "memory" (memory $mem 1))
  
  ;; Import a global from the environment
  (import "env" "global_i32" (global $global_i32 (mut i32)))
  
  ;; Import a table from the environment
  (import "env" "table" (table $table 10 funcref))
  
  ;; Define a function that uses the imported function
  (func $test_imports (export "test_imports") (param i32)
    ;; Get the parameter and call the imported function
    local.get 0
    call $print_i32
  )
)