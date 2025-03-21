(module
  (func $simple_loop (export "simple_loop") (result i32)
    ;; Declare a local variable for the counter
    (local $counter i32)
    
    ;; Initialize counter to 0
    i32.const 0
    local.set $counter
    
    ;; Start a loop block
    (loop $my_loop
      ;; Increment the counter
      local.get $counter
      i32.const 1
      i32.add
      local.set $counter
      
      ;; If counter < 3, branch to the loop beginning
      local.get $counter
      i32.const 3
      i32.lt_s
      br_if $my_loop
    )
    
    ;; Return the final counter value
    local.get $counter
  )
) 