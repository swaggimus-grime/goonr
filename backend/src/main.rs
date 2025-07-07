#![recursion_limit = "256"]

use axum::{routing::get, routing::post, Router, ServiceExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::fmt::layer;
use crate::routes::api_routes;
use crate::state::AppState;

mod routes;
mod state;
mod error;
mod pipeline;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Initialize shared app state
    let state = Arc::new(AppState::new().await);

    let app = Router::new()
        .merge(api_routes())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("🚀 Listening on http://{}", addr);

    // Start the server
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}