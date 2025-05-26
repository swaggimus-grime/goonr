mod scene;

use std::num::NonZeroU64;
use std::sync::Arc;
use axum::{Extension, Router};
use axum::routing::{get, post};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower_http::limit::RequestBodyLimitLayer;
use crate::routes::scene::{get_pointcloud, upload_scene};
use crate::state::AppState;

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/upload_scene", post(upload_scene))
        .route("/pointcloud/{scene_id}", get(get_pointcloud))
}