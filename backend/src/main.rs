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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Initialize shared app state
    let state = AppState::new();

    let app = Router::new()
        .merge(api_routes())
        .with_state(state);
        //.layer(CorsLayer::permissive())
        //.layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("🚀 Listening on http://{}", addr);

    // Start the server
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}