(module
  ;; Import a function from the environment
  (import "env" "print_i32" (func $print_i32 (param i32)))
  
  ;; Define a function that adds two numbers
  (func $add (param i32) (param i32) (result i32)
    ;; Add a comment to make the file changed
    local.get 0
    local.get 1
    i32.add
  )
  
  ;; Define a function with locals that calculates factorial
  (func $factorial (param i32) (result i32)
    (local i32)  ;; Local for the result
    
    ;; Initialize result to 1
    i32.const 1
    local.set 1
    
    ;; Check if input is less than 2
    local.get 0
    i32.const 2
    i32.lt_s
    if
      ;; If n < 2, return 1
      i32.const 1
      return
    end
    
    ;; Calculate factorial via loop
    (loop $loop
      ;; Multiply result by counter
      local.get 1    ;; Load result
      local.get 0    ;; Load counter
      i32.mul        ;; Multiply
      local.set 1    ;; Store in result
      
      ;; Decrement counter
      local.get 0
      i32.const 1
      i32.sub
      local.set 0
      
      ;; Continue loop if counter >= 2
      local.get 0
      i32.const 1
      i32.gt_s
      br_if $loop
    )
    
    ;; Print the result
    local.get 1
    call $print_i32
    
    ;; Return the result
    local.get 1
  )
  
  ;; Define a function that uses and if/else statement
  (func $compare (param i32) (param i32) (result i32)
    local.get 0
    local.get 1
    i32.gt_s
    if (result i32)
      ;; If a > b, return a
      local.get 0
    else
      ;; Otherwise, return b
      local.get 1
    end
  )
  
  ;; Export our functions
  (export "add" (func $add))
  (export "factorial" (func $factorial))
  (export "compare" (func $compare))
)