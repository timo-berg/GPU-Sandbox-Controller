use serde::Serialize;
use time::Duration;

use wasmtime::{Config, Engine, Linker, Module, Store};

use crate::domain::Job;

pub struct SandboxExecutor {
    engine: Engine,
    config: SandboxConfig,
}

#[derive(Clone, Serialize)]
pub struct ExecutionResult {
    pub output: Vec<u8>,
    pub execution_time: Duration,
    pub memory_used: usize,
}

#[derive(Clone, Copy)]
pub struct SandboxConfig {
    pub max_memory_bytes: usize,
    pub max_execution_time: Duration,
    pub module_cache_size: usize,
    pub enable_fuel: bool,
}

struct SandboxContext {
    pub job_id: uuid::Uuid,
    pub tenant_id: String,
    pub max_memory: usize,
}

#[derive(Debug)]
pub enum SandboxError {
    ModuleNotFound(String),
    ModeleLoadFailed(String),
    ExecutionFailed(String),
    Timeout,
    OutOfMemory,
    CapabilityViolation(String),
    TrapOccured(String),
}

impl SandboxExecutor {
    pub fn new(sandbox_config: SandboxConfig) -> Result<Self, SandboxError> {
        let mut config = Config::new();

        if sandbox_config.enable_fuel {
            config.consume_fuel(true);
        }

        // Note: epoch_interruption requires calling store.set_epoch_deadline()
        // and periodically incrementing the engine epoch. We use fuel + tokio timeout instead.

        config.max_wasm_stack(2 * 1024 * 1024); // 2MB stack limit

        config.cranelift_opt_level(wasmtime::OptLevel::Speed);

        // Enable WASM features for modern modules
        config.wasm_bulk_memory(true);
        config.wasm_reference_types(true);
        config.wasm_multi_value(true);
        config.wasm_multi_memory(false); // Keep single memory for security
        config.wasm_simd(true);

        let engine = Engine::new(&config).map_err(|e| {
            SandboxError::ModeleLoadFailed(format!("Engine creation failed: {}", e))
        })?;

        Ok(SandboxExecutor {
            engine,
            config: sandbox_config,
        })
    }

    pub fn default() -> Result<Self, SandboxError> {
        let default_config = SandboxConfig {
            max_memory_bytes: 64 * 1024 * 1024, // 64MB
            max_execution_time: Duration::seconds(30),
            module_cache_size: 10,
            enable_fuel: true,
        };

        Self::new(default_config)
    }

    pub async fn execute(&self, job: &Job) -> Result<ExecutionResult, SandboxError> {
        let start_time = time::OffsetDateTime::now_utc();

        let module = self.load_module(&job.module_id)?;

        let context = SandboxContext {
            job_id: job.job_id,
            tenant_id: job.tenant_id.clone(),
            max_memory: self.config.max_memory_bytes,
        };

        let mut store = Store::new(&self.engine, context);

        if self.config.enable_fuel {
            store
                .set_fuel(1_000_000_000)
                .map_err(|e| SandboxError::ExecutionFailed(format!("Fuel setup failed: {}", e)))?;
        }

        let linker = self.build_linker(&job.capabilities)?;

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))?;

        // Simple function that takes no input, returns a result
        let run_func = instance
            .get_typed_func::<(), i32>(&mut store, "run")
            .map_err(|e| {
                SandboxError::ExecutionFailed(format!("Function 'run' not found {}", e))
            })?;

        // Execute in blocking thread with timeout
        let timeout_duration = self.config.max_execution_time;

        let execution_handle = tokio::task::spawn_blocking(move || {
            let result = run_func
                .call(&mut store, ())
                .map_err(|e| SandboxError::ExecutionFailed(e.to_string()))?;

            Ok::<_, SandboxError>(result)
        });

        let result = tokio::time::timeout(
            std::time::Duration::from_micros(timeout_duration.whole_microseconds() as u64),
            execution_handle,
        )
        .await
        .map_err(|_| SandboxError::Timeout)?
        .map_err(|e| SandboxError::ExecutionFailed(format!("Task join failed: {}", e)))??;

        let end_time = time::OffsetDateTime::now_utc();
        let execution_time = end_time - start_time;

        Ok(ExecutionResult {
            output: result.to_string().into_bytes(),
            execution_time,
            memory_used: 0,
        })
    }

    fn load_module(&self, module_id: &str) -> Result<Module, SandboxError> {
        // Construct path to WASM module
        let module_path = format!("modules/{}.wasm", module_id);

        // Check if file exists
        if !std::path::Path::new(&module_path).exists() {
            return Err(SandboxError::ModuleNotFound(format!(
                "Module file not found: {}",
                module_path
            )));
        }

        // Read the WASM file
        let wasm_bytes = std::fs::read(&module_path).map_err(|e| {
            SandboxError::ModeleLoadFailed(format!(
                "Failed to read module file {}: {}",
                module_path, e
            ))
        })?;

        // Compile the module
        Module::from_binary(&self.engine, &wasm_bytes).map_err(|e| {
            SandboxError::ModeleLoadFailed(format!("Failed to compile module {}: {}", module_id, e))
        })
    }

    fn build_linker(
        &self,
        capabilities: &[String],
    ) -> Result<Linker<SandboxContext>, SandboxError> {
        let mut linker = Linker::new(&self.engine);

        // Capability: "gpu.compute" - allows GPU computation
        if capabilities.contains(&"gpu.compute".to_string()) {
            linker
                .func_wrap(
                    "env",
                    "gpu_compute",
                    |caller: wasmtime::Caller<'_, SandboxContext>, operation: i32| -> i32 {
                        let ctx = caller.data();
                        println!(
                            "GPU compute called by tenant: {} for job: {}",
                            ctx.tenant_id, ctx.job_id
                        );

                        // Mock GPU computation
                        // Real implementation would submit to GPU queue
                        operation * 2
                    },
                )
                .map_err(|e| {
                    SandboxError::ExecutionFailed(format!("Failed to link gpu_compute: {}", e))
                })?;
        }

        // Capability: "logging" - allows the WASM module to log messages
        if capabilities.contains(&"logging".to_string()) {
            linker
                .func_wrap(
                    "env",
                    "log_message",
                    |mut caller: wasmtime::Caller<'_, SandboxContext>,
                     msg_ptr: i32,
                     msg_len: i32| {
                        // Read message from WASM memory
                        if let Some(memory) =
                            caller.get_export("memory").and_then(|e| e.into_memory())
                        {
                            let mut buffer = vec![0u8; msg_len as usize];
                            if memory
                                .read(&mut caller, msg_ptr as usize, &mut buffer)
                                .is_ok()
                            {
                                let msg = String::from_utf8_lossy(&buffer);
                                let ctx = caller.data();
                                println!("[WASM Log | Job {}] {}", ctx.job_id, msg);
                            }
                        }
                    },
                )
                .map_err(|e| {
                    SandboxError::ExecutionFailed(format!("Failed to link log_message: {}", e))
                })?;
        }

        // Capability: "network.egress" - allows outbound network calls
        if capabilities.contains(&"network.egress".to_string()) {
            linker
                .func_wrap(
                    "env",
                    "http_post",
                    |caller: wasmtime::Caller<'_, SandboxContext>,
                     _url_ptr: i32,
                     _data_ptr: i32|
                     -> i32 {
                        let ctx = caller.data();
                        println!(
                            "Network egress requested by tenant: {} for job: {}",
                            ctx.tenant_id, ctx.job_id
                        );

                        // Mock network call - real implementation would validate and make HTTP request
                        200 // Mock HTTP status
                    },
                )
                .map_err(|e| {
                    SandboxError::ExecutionFailed(format!("Failed to link http_post: {}", e))
                })?;
        }

        Ok(linker)
    }
}
