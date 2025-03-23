(module
  (memory (export "memory") 1)
  (func $store (export "store")
    i32.const 100      ;; address
    i32.const 42       ;; value 
    i32.store)         ;; store 42 at address 100
    
  (func $load (export "load") (result i32)
    i32.const 100      ;; address
    i32.load)          ;; load value from address 100
    
  (func $run (export "run") (result i32)
    call $store        ;; store 42 at address 100
    call $load         ;; load value from address 100
    i32.const 42       ;; expected value
    i32.ne             ;; compare result with expected (0 if equal, 1 if not equal)
    )
) 