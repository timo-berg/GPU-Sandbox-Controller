use axum::{
    Router,
    routing::{get, post},
};

mod api;
mod domain;
mod state;

use api::{get_job, submit_job};
use state::AppState;

#[tokio::main]
async fn main() {
    let state = AppState::new();

    let app = Router::new()
        .route("/healthz", get(|| async { "Hello Sandbox" }))
        .route("/jobs", post(submit_job))
        .route("/jobs/{job_id}", get(get_job))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
