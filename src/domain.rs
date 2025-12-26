use std::str;

use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::sandbox::ExecutionResult;

#[derive(Deserialize)]
pub struct SubmitJobRequest {
    pub tenant_id: String,
    pub module_id: String,
    pub payload: serde_json::Value,
    pub capabilities: Vec<String>,
}

#[derive(Serialize)]
pub struct SubmitJobResponse {
    pub job_id: Uuid,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Finished(String),
    Failed(String),
}

#[derive(Serialize)]
pub struct JobErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Clone, Serialize)]
pub struct Job {
    pub job_id: Uuid,
    pub tenant_id: String,
    pub module_id: String,
    pub payload: serde_json::Value,
    pub capabilities: Vec<String>,
    pub submitted_at: OffsetDateTime,
    pub started_at: Option<OffsetDateTime>,
    pub finished_at: Option<OffsetDateTime>,
    pub duration: Option<Duration>,
    pub status: JobStatus,
    pub result: Option<ExecutionResult>,
}

#[derive(Serialize)]
pub struct JobListItem {
    pub job_id: Uuid,
    pub tenant_id: String,
    pub status: JobStatus,
    pub submitted_at: OffsetDateTime,
}

#[derive(Serialize)]
pub struct JobListResponse {
    pub jobs: Vec<JobListItem>,
}
