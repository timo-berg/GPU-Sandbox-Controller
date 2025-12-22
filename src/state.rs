use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::domain::Job;

pub struct InnerState {
    pub jobs: HashMap<Uuid, Job>,
    pub queue: Sender<Job>,
}

impl InnerState {
    fn new(sender: Sender<Job>) -> Self {
        Self {
            jobs: HashMap::new(),
            queue: sender,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<RwLock<InnerState>>,
}

impl AppState {
    pub fn new(sender: Sender<Job>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(InnerState::new(sender))),
        }
    }
}
