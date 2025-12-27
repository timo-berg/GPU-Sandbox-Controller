# WASM Test Modules

This directory contains test WASM modules for the GPU Sandbox Controller.
All modules are compiled from WebAssembly Text format (.wat files).

## Available Modules

| Module | Capabilities | Expected Result | Description |
|--------|-------------|-----------------|-------------|
| `ultra-simple` | None | 42 | Just returns 42 |
| `simple-compute` | None | 60 | Computes `(10 + 20) * 2` |
| `gpu-compute` | `gpu.compute` | 42 | Calls host function `gpu_compute(21)` |
| `logging-test` | `logging` | 100 | Logs "Hello from WASM!" |

## Compiling WAT to WASM

If you modify the `.wat` files, recompile using wasm-tools:

```bash
wasm-tools parse modules/simple-compute.wat -o modules/simple-compute.wasm
wasm-tools parse modules/gpu-compute.wat -o modules/gpu-compute.wasm
wasm-tools parse modules/logging-test.wat -o modules/logging-test.wasm
wasm-tools parse modules/ultra-simple.wat -o modules/ultra-simple.wasm
```

## Testing

Run the server first:
```bash
cargo run
```

Then run the test script:
```powershell
.\test_modules.ps1
```
