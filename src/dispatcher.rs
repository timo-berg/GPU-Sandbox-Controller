use std::sync::Arc;

use time::OffsetDateTime;
use tokio::sync::{RwLock, mpsc::Receiver};
use tokio::time::{Duration, sleep};

use crate::state::InnerState;

use crate::domain::{Job, JobStatus};

pub async fn run_dispatcher(mut rx: Receiver<Job>, inner_state: Arc<RwLock<InnerState>>) {
    while let Some(job) = rx.recv().await {
        let mut inner = inner_state.write().await;

        if let Some(job_in_map) = inner.jobs.get_mut(&job.job_id) {
            job_in_map.status = JobStatus::Running;
            job_in_map.started_at = Some(OffsetDateTime::now_utc());
        }

        drop(inner);

        // Mock task execution
        sleep(Duration::from_secs(5)).await;

        let mut inner = inner_state.write().await;

        if let Some(job_in_map) = inner.jobs.get_mut(&job.job_id) {
            job_in_map.status = JobStatus::Finished("Successfully wasted 5 seconds".to_string());
            let finished = OffsetDateTime::now_utc();
            job_in_map.finished_at = Some(finished);
            if let Some(started) = job_in_map.started_at {
                job_in_map.duration = Some(finished - started);
            }
        }
    }
}
