use std::sync::Arc;
use tokio::sync::RwLock;
use ml::Scene;

#[derive(Clone)]
pub struct AppState {
    pub scene: Arc<RwLock<Option<Scene>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            scene: Arc::new(RwLock::new(None)),
        }
    }
}
