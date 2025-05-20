use axum::Router;
use axum::routing::post;
use crate::AppState;
use crate::handlers::{load_scene};

pub fn api_routes(state: AppState) -> Router {
    Router::new()
        .route("/load", post(load_scene)).with_state(state.clone())
}