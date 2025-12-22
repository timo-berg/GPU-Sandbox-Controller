use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SubmitJobRequest {
    pub tenant_id: String,
    pub model_id: String,
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
}

#[derive(Clone, Serialize)]
pub struct Job {
    pub job_id: Uuid,
    pub tenant_id: String,
    pub model_id: String,
    pub payload: serde_json::Value,
    pub capabilities: Vec<String>,
    pub submitted_at: OffsetDateTime,
    pub status: JobStatus,
}
