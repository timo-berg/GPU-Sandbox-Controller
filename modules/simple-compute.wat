(module
  ;; Define memory (1 page = 64KB)
  (memory (export "memory") 1)
  
  ;; Export a 'run' function that takes no parameters and returns an i32
  (func $run (export "run") (result i32)
    ;; Simple computation: (10 + 20) * 2
    i32.const 10
    i32.const 20
    i32.add
    i32.const 2
    i32.mul
  )
)

