(module
  (func $tiny_loop (export "tiny_loop") (result i32)
    ;; Initialize counter to 0
    (local $counter i32)
    
    ;; Initialize counter to 0
    i32.const 0
    local.set $counter
    
    ;; First increment: counter = 1
    local.get $counter
    i32.const 1
    i32.add
    local.set $counter
    
    ;; Second increment: counter = 2
    local.get $counter
    i32.const 1
    i32.add
    local.set $counter
    
    ;; Third increment: counter = 3
    local.get $counter
    i32.const 1
    i32.add
    local.set $counter
    
    ;; Return the counter (should be 3)
    local.get $counter
  )
) 