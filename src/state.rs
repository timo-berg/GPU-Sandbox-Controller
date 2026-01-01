use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use time::OffsetDateTime;
use tokio::sync::RwLock;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::config::Config;
use crate::domain::Job;
use crate::gpu_manager::GpuManager;
use crate::tenant::Tenant;

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
    pub gpu_manager: Arc<RwLock<GpuManager>>,
    pub tenants: Arc<RwLock<HashMap<String, Tenant>>>,
    pub tenant_usage: Arc<RwLock<HashMap<String, VecDeque<OffsetDateTime>>>>,
}

impl AppState {
    pub fn new(sender: Sender<Job>, config: &Config, tenants: HashMap<String, Tenant>) -> Self {
        Self {
            inner: Arc::new(RwLock::new(InnerState::new(sender))),
            gpu_manager: Arc::new(RwLock::new(GpuManager::new(config))),
            tenants: Arc::new(RwLock::new(tenants)),
            tenant_usage: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
