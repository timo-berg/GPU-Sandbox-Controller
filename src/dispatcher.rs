use time::OffsetDateTime;
use tokio::sync::mpsc::Receiver;

use crate::domain::{Job, JobStatus};
use crate::sandbox::{ExecutionResult, SandboxError, SandboxExecutor};
use crate::state::AppState;
use crate::tenant;
use crate::tenant::TenantStatus;

pub async fn run_dispatcher(mut rx: Receiver<Job>, state: AppState) {
    while let Some(job) = rx.recv().await {
        // Fetch the tenant
        let tenant_opt = {
            let tenants = state.tenants.read().await;
            tenants.get(&job.tenant_id).cloned()
        };

        let Some(tenant) = tenant_opt else {
            let mut inner = state.inner.write().await;
            if let Some(job_in_map) = inner.jobs.get_mut(&job.job_id) {
                job_in_map.status = JobStatus::Failed("Tenant ID not found".to_string());
                job_in_map.finished_at = Some(OffsetDateTime::now_utc());
            }
            continue;
        };

        // Validate tenant
        if !matches!(tenant.status, tenant::TenantStatus::Active) {
            let mut inner = state.inner.write().await;
            if let Some(job_in_map) = inner.jobs.get_mut(&job.job_id) {
                job_in_map.status = JobStatus::Failed("Tenant not authorized".to_string());
                job_in_map.finished_at = Some(OffsetDateTime::now_utc());
            }
            continue;
        }

        if job
            .capabilities
            .iter()
            .any(|c| !tenant.allowed_capabilities.contains(c))
        {
            let mut inner = state.inner.write().await;
            if let Some(job_in_map) = inner.jobs.get_mut(&job.job_id) {
                job_in_map.status =
                    JobStatus::Failed("Unauthorized capabilities requested".to_string());
                job_in_map.finished_at = Some(OffsetDateTime::now_utc());
            }
            continue;
        }

        // Try to allocate ressources
        {
            let mut gpu_manager = state.gpu_manager.write().await;
            if let Err(_) = gpu_manager.try_reserve_slot(&tenant) {
                let mut inner = state.inner.write().await;

                if let Some(job_in_map) = inner.jobs.get_mut(&job.job_id) {
                    job_in_map.status =
                        JobStatus::Failed("No GPU capacity, please try again later".to_string());
                    job_in_map.finished_at = Some(OffsetDateTime::now_utc());
                }
                continue;
            }
        }

        let state_clone = state.clone();

        // This can return a handle so we can manage sth like timeouts
        let _handle = tokio::spawn(run_task(job, state_clone));
    }
}

async fn run_task(job: Job, state: AppState) {
    // Mark as running
    {
        let mut inner = state.inner.write().await;

        if let Some(job_in_map) = inner.jobs.get_mut(&job.job_id) {
            job_in_map.status = JobStatus::Running;
            job_in_map.started_at = Some(OffsetDateTime::now_utc());
        }
    }

    match execute_job(&job).await {
        Ok(result) => {
            let mut inner = state.inner.write().await;
            if let Some(job_in_map) = inner.jobs.get_mut(&job.job_id) {
                job_in_map.status =
                    JobStatus::Finished("Successfully wasted 5 seconds".to_string());
                let finished = OffsetDateTime::now_utc();
                job_in_map.finished_at = Some(finished);
                if let Some(started) = job_in_map.started_at {
                    job_in_map.duration = Some(finished - started);
                }
                job_in_map.result = Some(result);
            }
        }
        Err(e) => {
            let mut inner = state.inner.write().await;
            if let Some(job_in_map) = inner.jobs.get_mut(&job.job_id) {
                job_in_map.status = JobStatus::Failed(format!("Job execution failed: {:?}", e));
                let finished = OffsetDateTime::now_utc();
                job_in_map.finished_at = Some(finished);
                if let Some(started) = job_in_map.started_at {
                    job_in_map.duration = Some(finished - started);
                }
            }
        }
    }

    {
        let mut gpu_manager = state.gpu_manager.write().await;
        let _ = gpu_manager.release_slot(&job.tenant_id);
    }
}

async fn execute_job(job: &Job) -> Result<ExecutionResult, SandboxError> {
    let executor = SandboxExecutor::default()?;
    executor.execute(job).await
}
