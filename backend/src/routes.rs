mod scene;
mod pipeline;

use std::sync::Arc;
use axum::{Extension, Router};
use axum::extract::DefaultBodyLimit;
use axum::routing::{any, get, post, put};
use crate::routes::pipeline::train_scene;
use crate::routes::scene::{get_scene, get_scenes, upload_scene};
use crate::state::AppState;

pub fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/upload_scene", post(upload_scene))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024 * 2))
        .route("/scene/{name}", get(get_scene))
        .route("/scenes", get(get_scenes))
        .route("/train/{name}", any(train_scene))
}