use axum::{routing::get, routing::post, Router, ServiceExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::routes::api_routes;

mod routes;
mod state;
mod scene;

#[tokio::main]
async fn main() {
    let app = api_routes();

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}