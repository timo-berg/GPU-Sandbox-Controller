use crate::domain::{Job, JobErrorResponse, JobStatus, SubmitJobRequest, SubmitJobResponse};
use crate::state::AppState;
use axum::{Json, extract::Path, extract::State, http::StatusCode, response::IntoResponse};
use time::OffsetDateTime;
use uuid::Uuid;

pub async fn submit_job(
    State(state): State<AppState>,
    Json(req): Json<SubmitJobRequest>,
) -> impl IntoResponse {
    let job_id = Uuid::new_v4();
    let submitted_at = OffsetDateTime::now_utc();

    let job = Job {
        job_id,
        tenant_id: req.tenant_id,
        model_id: req.model_id,
        payload: req.payload,
        capabilities: req.capabilities,
        submitted_at,
        started_at: None,
        finished_at: None,
        duration: None,
        status: JobStatus::Queued,
    };
    {
        let mut inner = state.inner.write().await;

        let job_for_queue = job.clone();

        match inner.queue.try_send(job_for_queue) {
            Ok(()) => {
                inner.jobs.insert(job_id, job);
            }
            Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(JobErrorResponse {
                        error: "queue_full".to_string(),
                        message: "Job queue full please kwewe later".to_string(),
                    }),
                )
                    .into_response();
            }
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(JobErrorResponse {
                        error: "service_closed".to_string(),
                        message: "Sorry, we are closed for business".to_string(),
                    }),
                )
                    .into_response();
            }
        }
    }

    (StatusCode::ACCEPTED, Json(SubmitJobResponse { job_id })).into_response()
}

pub async fn get_job(State(state): State<AppState>, Path(job_id): Path<Uuid>) -> impl IntoResponse {
    let inner = state.inner.read().await;

    match inner.jobs.get(&job_id).cloned() {
        Some(job) => (StatusCode::OK, Json(job)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(JobErrorResponse {
                error: "job_not_found".to_string(),
                message: format!("Job with id {} not found", job_id),
            }),
        )
            .into_response(),
    }
}
