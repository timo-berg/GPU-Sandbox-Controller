(module
  ;; Import the gpu_compute host function (requires "gpu.compute" capability)
  (import "env" "gpu_compute" (func $gpu_compute (param i32) (result i32)))
  
  ;; Export a 'run' function that takes no parameters and returns an i32
  (func $run (export "run") (result i32)
    ;; Call the GPU compute function with input value 21
    i32.const 21
    call $gpu_compute
    ;; The mock GPU function should return 21 * 2 = 42
  )
)

