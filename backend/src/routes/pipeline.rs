use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use axum::body::Body;
use axum::extract::{Multipart, Path, Request, State, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info};
use db::repo::SplatRepository;
use pipeline::{Pipeline, PipelineMessage};
use scene_source::Source;
use web_cmn::pipeline::WiredPipelineMessage;
use crate::error::{BackendError, Result};
use crate::pipeline::splats_from_module;
use crate::state::AppState;

pub async fn train_scene(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    info!("🔌 Incoming WebSocket upgrade request for scene: {}", name);
    ws.on_upgrade(|socket| start_pipeline(socket, name, state))
}

async fn start_pipeline(
    mut socket: WebSocket,
    scene_name: String,
    state: Arc<AppState>,
) {
    // Start training
    if let Some(scene) = state.repo.get_scene(&scene_name).await.unwrap() {
        let mut pipeline = Pipeline::new(scene.source).unwrap();
        let mut stream = std::pin::pin!(pipeline.launch());
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(msg) => {
                    info!("Received pipeline message: {:?}", msg);
                    if let Some(wired_msg) = create_wired_msg(msg) {
                        let json = serde_json::to_string(&wired_msg).unwrap();
                        if socket.send(Message::from(json)).await.is_err() {
                            break;
                        }
                        if matches!(wired_msg, WiredPipelineMessage::Done) {
                            break;
                        }
                    }
                }
                Err(err) => {
                    error!("Training websocket loop failed due to invalid message: {}", err);
                    break;
                }
            }
        }
    } else {
        // send a failure message over WebSocket
        let _ = socket.send(Message::from("{\"error\": \"Scene not found\"}")).await;
    };

    let _ = socket.close().await;
    info!("End of pipeline websocket");
}
fn create_wired_msg(msg: PipelineMessage) -> Option<WiredPipelineMessage> {
    match msg {
        PipelineMessage::TrainStep {
            splats,
            stats,
            iter,
            total_elapsed,
        } => {
            Some(WiredPipelineMessage::TrainStep(splats_from_module(&*splats)))
        },
        PipelineMessage::Finished => Some(WiredPipelineMessage::Done),
        _ => None,
    }
}

