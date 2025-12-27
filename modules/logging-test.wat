(module
  ;; Import the log_message host function (requires "logging" capability)
  (import "env" "log_message" (func $log_message (param i32 i32)))
  
  ;; Import memory (we need this to store strings)
  (memory (export "memory") 1)
  
  ;; Data section: store a message string in memory at offset 0
  (data (i32.const 0) "Hello from WASM!")
  
  ;; Export a 'run' function
  (func $run (export "run") (result i32)
    ;; Call log_message with pointer=0, length=16
    i32.const 0
    i32.const 16
    call $log_message
    
    ;; Return a success code
    i32.const 100
  )
)

