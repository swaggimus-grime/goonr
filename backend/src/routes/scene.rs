use axum::{extract::{State}, Extension, Json};
use std::{
    sync::Arc,
    path::PathBuf,
    io::Cursor,
};
use axum::extract::{Multipart, Path};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use tempfile::tempdir;
use tracing::{error, info};
use uuid::Uuid;
use zip::ZipArchive;

use crate::error::BackendError;
use crate::state::{AppState, Scene, SceneMetadata};

#[axum::debug_handler]
pub async fn upload_scene(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<SceneMetadata>, BackendError> {
    info!("Received upload request");

    let field = multipart
        .next_field()
        .await?
        .ok_or_else(|| BackendError::BadRequest("Missing file".into()))?;

    if field.name() != Some("scene_zip") {
        return Err(BackendError::BadRequest("Expected field 'scene_zip'".into()));
    }

    let filename = field
        .file_name()
        .map(str::to_string)
        .unwrap_or_else(|| "scene.zip".to_string());

    // Read and log bytes progressively
    let mut bytes = Vec::new();
    let mut total_size = 0;
    let mut stream = field;

    while let Some(chunk) = stream.chunk().await? {
        total_size += chunk.len();
        info!("Received chunk of size {}, total: {}", chunk.len(), total_size);
        bytes.extend_from_slice(&chunk);
    }

    info!("Finished receiving file: total size = {} bytes", total_size);

    let scene_id = Uuid::new_v4();
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let temp_path = temp_dir.path().to_str().unwrap();
    let scene_path = PathBuf::from(format!("{temp_path}\\goonr_uploads\\{scene_id}"));
    std::fs::create_dir_all(&scene_path)?;

    let reader = Cursor::new(&bytes);
    let mut archive = ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = scene_path.join(file.sanitized_name());

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    let raw_ml_scene = ml::Scene::new(&scene_path).await?;
    let raw_scene = Arc::new(raw_ml_scene);

    let scene = Scene {
        id: scene_id,
        name: filename,
        raw: raw_scene,
        scene_path: scene_path.clone(),
    };

    {
        let mut scenes = state.scenes.write().await;
        scenes.insert(scene_id, scene.clone());
    }

    let metadata = SceneMetadata {
        id: scene_id,
        name: scene.name,
        path: scene_path.to_string_lossy().into(),
    };

    Ok(Json(metadata))
}

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct GpuPoint {
    position: [f32; 3],
    color: [f32; 3],
}

pub async fn get_pointcloud(
    State(state): State<AppState>, // Note: Use Arc<AppState> if wrapped
    Path(scene_id): Path<Uuid>,
) -> axum::response::Response {
    let scenes = state.scenes.read().await;
    let scene = match scenes.get(&scene_id) {
        Some(scene) => scene,
        None => return Response::builder()
            .status(404)
            .body("Scene not found".into())
            .unwrap(),
    };

    let points = scene.raw.points();

    let gpu_points: Vec<GpuPoint> = points
        .values()
        .map(|p| GpuPoint {
            position: p.xyz.to_array(),
            color: [
                p.rgb[0] as f32 / 255.0,
                p.rgb[1] as f32 / 255.0,
                p.rgb[2] as f32 / 255.0,
            ],
        })
        .collect();

    let bytes = bytemuck::cast_slice(&gpu_points).to_vec();

    Response::builder()
        .header("Content-Type", "application/octet-stream")
        .body(bytes.into())
        .unwrap()
}