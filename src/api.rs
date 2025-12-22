use crate::domain::{Job, JobStatus, SubmitJobRequest, SubmitJobResponse};
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
        status: JobStatus::Queued,
    };
    {
        let mut inner = state.inner.write().await;
        inner.queue.push_back(job_id);
        inner.jobs.insert(job_id, job);
    }

    (StatusCode::ACCEPTED, Json(SubmitJobResponse { job_id }))
}

pub async fn get_job(State(state): State<AppState>, Path(job_id): Path<Uuid>) -> impl IntoResponse {
    let inner = state.inner.read().await;

    match inner.jobs.get(&job_id).cloned() {
        Some(job) => (StatusCode::OK, Json(job)).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
