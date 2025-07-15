(module
  (func (export "test") (result i32) (i32.const 42))
)

(assert_return (invoke "test") (i32.const 42))