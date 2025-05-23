use std::collections::HashMap;
use std::sync::Arc;
use serde::Serialize;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub scenes: Arc<RwLock<HashMap<Uuid, Scene>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            scenes: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Scene {
    pub id: Uuid,
    pub name: String,
    pub raw: Arc<ml::Scene>,
    pub scene_path: std::path::PathBuf, // path to the full scene workspace
}

#[derive(Debug, Clone, Serialize)]
pub struct SceneMetadata {
    pub id: Uuid,
    pub name: String,
    pub path: String,
}