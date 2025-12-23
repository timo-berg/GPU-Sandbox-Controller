use time::OffsetDateTime;
use tokio::sync::mpsc::Receiver;
use tokio::time::{Duration, sleep};

use crate::state::AppState;

use crate::domain::{Job, JobStatus};

pub async fn run_dispatcher(mut rx: Receiver<Job>, state: AppState) {
    while let Some(job) = rx.recv().await {
        // Try to allocate ressources
        {
            let mut gpu_manager = state.gpu_manager.write().await;
            if let Err(_) = gpu_manager.try_reserve_slot(&job.tenant_id) {
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
        tokio::spawn(run_task(job, state_clone));
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

    // Mock task execution
    sleep(Duration::from_secs(5)).await;

    // Mark as finished
    {
        let mut inner = state.inner.write().await;

        if let Some(job_in_map) = inner.jobs.get_mut(&job.job_id) {
            job_in_map.status = JobStatus::Finished("Successfully wasted 5 seconds".to_string());
            let finished = OffsetDateTime::now_utc();
            job_in_map.finished_at = Some(finished);
            if let Some(started) = job_in_map.started_at {
                job_in_map.duration = Some(finished - started);
            }
        }
    }

    {
        let mut gpu_manager = state.gpu_manager.write().await;
        let _ = gpu_manager.release_slot(&job.tenant_id);
    }
}
