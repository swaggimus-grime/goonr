use std::path::Path;
use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use serde::Deserialize;
use log::error;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SceneInput {
    input_path: String,
}

pub async fn load_scene(Json(payload): Json<SceneInput>) -> impl IntoResponse {
    let input_path = &payload.input_path;
    let scene_id = Uuid::new_v4().to_string();
    let output_dir = format!("data/{}", scene_id);

    fs::create_dir_all(&output_dir).await.unwrap();

    // Call `ml-cmd` regardless of zip or directory – it handles both
    let ml_status = Command::new("ml-cmd")
        .args(["--input", input_path, "--output", &output_dir])
        .status();

    match ml_status.await {
        Ok(status) if status.success() => (StatusCode::OK, scene_id).into_response(),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "ml-cmd failed to process input",
        )
            .into_response(),
    }
}