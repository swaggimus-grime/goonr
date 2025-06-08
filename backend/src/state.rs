use std::collections::HashMap;
use std::sync::Arc;
use serde::Serialize;
use tokio::sync::RwLock;
use uuid::Uuid;
use db::repo::SplatRepo;
use pipeline::Pipeline;

#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<SplatRepo>,
    pub pipeline: Option<Arc<Pipeline>>,
}

impl AppState {
    pub async fn new() -> Self {
        Self {
            repo: Arc::new(SplatRepo::new().await.unwrap()),
            pipeline: None,
        }
    }
}
