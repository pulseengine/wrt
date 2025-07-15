;; Very simple test - just return constant
(module
  (func (export "get_five") (result i32)
    i32.const 5
  )
)

(assert_return (invoke "get_five") (i32.const 5))
(assert_return (invoke "get_five") (i32.const 999))