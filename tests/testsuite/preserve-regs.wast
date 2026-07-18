;; if

(module
  (global $g0 (mut f32) (f32.const 10))
  (global $g1 (mut f32) (f32.const 20))
  (func (export "condition-reg(f32)-overwrite.if") (param i32) (result f32)
    (global.get $g0)
    (if
        (local.get 0)
        (then
            (global.get $g1)
            (drop)
        )
    )
  )
)
(assert_return
    (invoke "condition-reg(f32)-overwrite.if" (i32.const 0))
    (f32.const 10)
)
(assert_return
    (invoke "condition-reg(f32)-overwrite.if" (i32.const 1))
    (f32.const 10)
)

(module
  (global $g0 (mut f64) (f64.const 10))
  (global $g1 (mut f64) (f64.const 20))
  (func (export "condition-reg(f64)-overwrite.if") (param i32) (result f64)
    (global.get $g0)
    (if
        (local.get 0)
        (then
            (global.get $g1)
            (drop)
        )
    )
  )
)
(assert_return
    (invoke "condition-reg(f64)-overwrite.if" (i32.const 0))
    (f64.const 10)
)
(assert_return
    (invoke "condition-reg(f64)-overwrite.if" (i32.const 1))
    (f64.const 10)
)

(module
  (global $g0 (mut i32) (i32.const 10))
  (global $g1 (mut i32) (i32.const 20))
  (func (export "condition-reg(i64)-overwrite.if") (param i32) (result i32)
    (global.get $g0)
    (if
        (local.get 0)
        (then
            (global.get $g1)
            (drop)
        )
    )
  )
)
(assert_return
    (invoke "condition-reg(i64)-overwrite.if" (i32.const 0))
    (i32.const 10)
)
(assert_return
    (invoke "condition-reg(i64)-overwrite.if" (i32.const 1))
    (i32.const 10)
)

;; block

(module
    (global $g0 (mut f32) (f32.const 10))
    (global $g1 (mut f32) (f32.const 20))
    (func (export "condition-reg(f32)-overwrite.block") (param i32) (result f32)
        (global.get $g0)
        (block $break
            (local.get 0)
            (br_if $break)
            (global.get $g1)
            (drop)
        )
    )
)
(assert_return
  (invoke "condition-reg(f32)-overwrite.block" (i32.const 0))
  (f32.const 10)
)
(assert_return
  (invoke "condition-reg(f32)-overwrite.block" (i32.const 1))
  (f32.const 10)
)

(module
    (global $g0 (mut f64) (f64.const 10))
    (global $g1 (mut f64) (f64.const 20))
    (func (export "condition-reg(f64)-overwrite.block") (param i32) (result f64)
        (global.get $g0)
        (block $break
            (local.get 0)
            (br_if $break)
            (global.get $g1)
            (drop)
        )
    )
)
(assert_return
  (invoke "condition-reg(f64)-overwrite.block" (i32.const 0))
  (f64.const 10)
)
(assert_return
  (invoke "condition-reg(f64)-overwrite.block" (i32.const 1))
  (f64.const 10)
)

(module
    (global $g0 (mut i32) (i32.const 10))
    (global $g1 (mut i32) (i32.const 20))
    (func (export "condition-reg(i32)-overwrite.block") (param i32) (result i32)
        (global.get $g0)
        (block $break
            (local.get 0)
            (br_if $break)
            (global.get $g1)
            (drop)
        )
    )
)
(assert_return
  (invoke "condition-reg(i32)-overwrite.block" (i32.const 0))
  (i32.const 10)
)
(assert_return
  (invoke "condition-reg(i32)-overwrite.block" (i32.const 1))
  (i32.const 10)
)

;; loop

(module
    (global $g0 (mut f32) (f32.const 10))
    (global $g1 (mut f32) (f32.const 20))
    (func (export "condition-reg(f32)-overwrite.loop") (param i32) (result f32)
        (global.get $g0)
        (loop $continue (result i32)
            (if (result i32)
                (local.get 0)
                (then (i32.const 0))
                (else
                    (global.get $g1)
                    (local.set 0 (i32.const 1))
                    (br $continue)
                )
            )
        )
        (drop)
    )
)
(assert_return
    (invoke "condition-reg(f32)-overwrite.loop" (i32.const 0))
    (f32.const 10)
)
(assert_return
    (invoke "condition-reg(f32)-overwrite.loop" (i32.const 1))
    (f32.const 10)
)

(module
    (global $g0 (mut f64) (f64.const 10))
    (global $g1 (mut f64) (f64.const 20))
    (func (export "condition-reg(f64)-overwrite.loop") (param i32) (result f64)
        (global.get $g0)
        (loop $continue (result i32)
            (if (result i32)
                (local.get 0)
                (then (i32.const 0))
                (else
                    (global.get $g1)
                    (local.set 0 (i32.const 1))
                    (br $continue)
                )
            )
        )
        (drop)
    )
)
(assert_return
    (invoke "condition-reg(f64)-overwrite.loop" (i32.const 0))
    (f64.const 10)
)
(assert_return
    (invoke "condition-reg(f64)-overwrite.loop" (i32.const 1))
    (f64.const 10)
)

(module
    (global $g0 (mut i32) (i32.const 10))
    (global $g1 (mut i32) (i32.const 20))
    (func (export "condition-reg(i32)-overwrite.loop") (param i32) (result i32)
        (global.get $g0)
        (loop $continue (result i32)
            (if (result i32)
                (local.get 0)
                (then (i32.const 0))
                (else
                    (global.get $g1)
                    (local.set 0 (i32.const 1))
                    (br $continue)
                )
            )
        )
        (drop)
    )
)
(assert_return
    (invoke "condition-reg(i32)-overwrite.loop" (i32.const 0))
    (i32.const 10)
)
(assert_return
    (invoke "condition-reg(i32)-overwrite.loop" (i32.const 1))
    (i32.const 10)
)

(module
    (func (export "if-else.reg-result.i32") (param i32) (result i32)
        (i32.const 1)
        (if (param i32) (result i32)
            (local.get 0)
            (then (i32.add (i32.const 10))) ;; 1, lives in accumulator
            (else (i32.add (i32.const 20))) ;; 1, lives in accumulator
        )
    )
)
(assert_return (invoke "if-else.reg-result.i32" (i32.const 1)) (i32.const 11))
(assert_return (invoke "if-else.reg-result.i32" (i32.const 0)) (i32.const 21))
