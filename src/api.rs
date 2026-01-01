use std::collections::VecDeque;

use crate::domain::{
    Job, JobErrorResponse, JobListItem, JobListResponse, JobStatus, SubmitJobRequest,
    SubmitJobResponse,
};

use crate::state::AppState;
use crate::tenant::TenantStatus;
use axum::{Json, extract::Path, extract::State, http::StatusCode, response::IntoResponse};
use time::Duration;
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
        module_id: req.module_id,
        payload: req.payload,
        capabilities: req.capabilities,
        submitted_at,
        started_at: None,
        finished_at: None,
        duration: None,
        status: JobStatus::Queued,
        result: None,
    };

    let tenant = {
        let tenants = state.tenants.read().await;
        tenants.get(&job.tenant_id).cloned()
    };

    let Some(t) = tenant else {
        return (
            StatusCode::NOT_FOUND,
            Json(JobErrorResponse {
                error: "unknown_tenant".to_string(),
                message: format!("Tenant ID {} not known", job.tenant_id),
            }),
        )
            .into_response();
    };

    if !matches!(t.status, TenantStatus::Active) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(JobErrorResponse {
                error: "unauthorized_tenant".to_string(),
                message: format!("Tenant ID {} not authorized", job.tenant_id),
            }),
        )
            .into_response();
    }

    if job
        .capabilities
        .iter()
        .any(|c| !t.allowed_capabilities.contains(c))
    {
        let unpermitted_capabilities: Vec<&String> = job
            .capabilities
            .iter()
            .filter(|c| !t.allowed_capabilities.contains(*c))
            .collect();

        return (
            StatusCode::FORBIDDEN,
            Json(JobErrorResponse {
                error: "unpermitted_capabilities".to_string(),
                message: format!(
                    "Unpermitted capabilities requested: {:?} ",
                    unpermitted_capabilities
                ),
            }),
        )
            .into_response();
    }

    // Rate limit: tenant.rate_limit is "#jobs / minute"
    let now = OffsetDateTime::now_utc();
    let window = Duration::minutes(1);

    // Lock order: tenant_usage -> inner (avoid accidental deadlocks later)
    let mut tenant_usage_map = state.tenant_usage.write().await;
    if t.rate_limit > 0 {
        let usage: &mut VecDeque<OffsetDateTime> = tenant_usage_map
            .entry(job.tenant_id.clone())
            .or_insert_with(VecDeque::new);

        while usage.front().is_some_and(|ts| now - *ts > window) {
            usage.pop_front();
        }

        if usage.len() >= t.rate_limit {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(JobErrorResponse {
                    error: "rate_limit_exceeded".to_string(),
                    message: format!(
                        "Rate limit exceeded for tenant {}: max {} jobs per minute",
                        job.tenant_id, t.rate_limit
                    ),
                }),
            )
                .into_response();
        }

        // Reserve a slot in the window; we roll this back if queueing fails.
        usage.push_back(now);
    }

    let mut inner = state.inner.write().await;

    let job_for_queue = job.clone();

    match inner.queue.try_send(job_for_queue) {
        Ok(()) => {
            inner.jobs.insert(job_id, job);
        }
        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
            // Roll back rate limit reservation if we made one.
            if t.rate_limit > 0 {
                if let Some(usage) = tenant_usage_map.get_mut(&job.tenant_id) {
                    let _ = usage.pop_back();
                    if usage.is_empty() {
                        tenant_usage_map.remove(&job.tenant_id);
                    }
                }
            }
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
            // Roll back rate limit reservation if we made one.
            if t.rate_limit > 0 {
                if let Some(usage) = tenant_usage_map.get_mut(&job.tenant_id) {
                    let _ = usage.pop_back();
                    if usage.is_empty() {
                        tenant_usage_map.remove(&job.tenant_id);
                    }
                }
            }
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

pub async fn list_jobs(State(state): State<AppState>) -> impl IntoResponse {
    let inner = state.inner.read().await;

    let mut jobs: Vec<JobListItem> = inner
        .jobs
        .values()
        .map(|job| JobListItem {
            job_id: job.job_id,
            tenant_id: job.tenant_id.clone(),
            status: job.status.clone(),
            submitted_at: job.submitted_at,
        })
        .collect();

    jobs.sort_by_key(|job| job.submitted_at);

    Json(JobListResponse { jobs })
}
