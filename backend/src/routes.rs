mod scene;

use std::num::NonZeroU64;
use std::sync::Arc;
use axum::{Extension, Router};
use axum::routing::{get, post, put};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower_http::limit::RequestBodyLimitLayer;
use crate::routes::scene::{list_scenes, parse_scene, upload_scene};
use crate::state::AppState;

pub fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/upload_scene", post(upload_scene))
        .route("/parse_scene/{scene_name}", put(parse_scene))
        .route("/list_scenes", get(list_scenes))
}