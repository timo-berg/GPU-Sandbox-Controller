use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::Job;

pub struct InnerState {
    pub jobs: HashMap<Uuid, Job>,
    pub queue: VecDeque<Uuid>,
}

impl InnerState {
    fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            queue: VecDeque::new(),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<RwLock<InnerState>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(InnerState::new())),
        }
    }
}
