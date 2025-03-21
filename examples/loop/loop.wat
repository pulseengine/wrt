(module
  (func $loop_counter (export "loop_counter") (result i32)
    (local $i i32)
    ;; Initialize counter to 0
    i32.const 0
    local.set $i
    
    ;; Start loop
    (block $done
      (loop $continue
        ;; Increment counter
        local.get $i
        i32.const 1
        i32.add
        local.set $i
        
        ;; Check if counter is less than 10, continue if so
        local.get $i
        i32.const 10
        i32.lt_s
        br_if $continue  ;; Branch to loop if i < 10
        
        ;; Otherwise, exit the loop (fall through)
      )
    )
    
    ;; Return final counter value
    local.get $i
  )
) 