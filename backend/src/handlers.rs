use std::path::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use serde::Deserialize;
use log::error;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct LoadRequest {
    pub path: String,
}


pub async fn load_scene(
    State(state): State<AppState>,
    Json(payload): Json<LoadRequest>,
) -> impl IntoResponse {
    match ml::Scene::new(&Path::new(&payload.path)).await {
        Ok(scene) => {
            *state.scene.write().await = Some(scene);
            (StatusCode::OK, "Scene loaded successfully")
        }
        Err(e) => {
            //error!("Failed to load scene from {}: {}", payload.path, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load scene")
        }
    }
}
