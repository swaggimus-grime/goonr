use axum::Router;
use axum::routing::post;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use crate::scene::{load_scene};

pub fn api_routes() -> Router {
    Router::new()
        .route("/load", post(load_scene))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}