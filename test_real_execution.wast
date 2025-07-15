;; Test real execution - this should pass
(module
  (func (export "add") (param i32) (param i32) (result i32)
    local.get 0
    local.get 1
    i32.add
  )
)

(assert_return (invoke "add" (i32.const 2) (i32.const 3)) (i32.const 5))

;; This should FAIL if we're doing real execution
(assert_return (invoke "add" (i32.const 2) (i32.const 3)) (i32.const 999))