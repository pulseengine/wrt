;; Simple WASM module that demonstrates real execution
(module
  ;; Import memory from the host
  (memory (export "memory") 1)
  
  ;; Function that writes a specific pattern to memory
  ;; This will prove we're actually executing
  (func $write_pattern (export "write_pattern")
    ;; Write "WASM" (0x5741534D) at address 0
    i32.const 0
    i32.const 0x5741534D  ;; "WASM" in hex
    i32.store
    
    ;; Write execution counter at address 4
    i32.const 4
    i32.const 42
    i32.store
    
    ;; Write checksum at address 8
    i32.const 8
    i32.const 0xDEADBEEF
    i32.store
  )
  
  ;; Function that reads and returns the pattern
  (func $verify_pattern (export "verify_pattern") (result i32)
    ;; Read from address 0
    i32.const 0
    i32.load
    
    ;; Should return 0x5741534D if write_pattern was executed
  )
  
  ;; Function that adds two numbers (visible computation)
  (func $add (export "add") (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )
  
  ;; Start function that runs automatically
  (func $start
    call $write_pattern
  )
  (start $start)
)