use axum::{
    Router,
    routing::{get, post},
};

mod api;
mod config;
mod dispatcher;
mod domain;
mod state;

use api::{get_job, submit_job};
use state::AppState;
use tokio::sync::mpsc;

use crate::config::Config;
use crate::domain::Job;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load("config.toml").await?;

    let (tx, rx) = mpsc::channel::<Job>(config.queue_length);

    let state = AppState::new(tx);

    let state_clone = state.inner.clone();
    tokio::spawn(dispatcher::run_dispatcher(rx, state_clone));

    let app = Router::new()
        .route("/healthz", get(|| async { "Hello Sandbox" }))
        .route("/jobs", post(submit_job))
        .route("/jobs/{job_id}", get(get_job))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
