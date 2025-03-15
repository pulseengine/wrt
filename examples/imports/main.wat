(module
  (import "env" "print" (func $print (param i32)))
  (func $main
    i32.const 42
    call $print
  )
  (export "main" (func $main))
)
