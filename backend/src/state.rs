use std::collections::HashMap;
use std::sync::Arc;
use serde::Serialize;
use tokio::sync::RwLock;
use db::repo::SplatRepo;
use pipeline::Pipeline;

pub struct AppState {
    pub repo: Arc<SplatRepo>,
    pub pipeline: Arc<RwLock<Option<Pipeline>>>,
}

impl AppState {
    pub async fn new() -> Self {
        Self {
            repo: Arc::new(SplatRepo::new().await.unwrap()),
            pipeline: Arc::new(RwLock::new(None)),
        }
    }
}
