use axum::{routing::get, routing::post, Router, ServiceExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use ml::Scene;
use crate::routes::api_routes;
use crate::state::AppState;

mod routes;
mod state;
mod handlers;

#[tokio::main]
async fn main() {
    let state = AppState::new();

    let app = api_routes(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}