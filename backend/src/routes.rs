mod scene;

use axum::Router;
use axum::routing::{get, post};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use crate::routes::scene::{get_scene, upload_scene};
use crate::state::AppState;

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/upload_scene", post(upload_scene))
        .route("/scene/{scene_id}", get(get_scene))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}