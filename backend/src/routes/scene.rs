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
use db::repo::SplatRepository;
use web_cmn::responses::scene::SceneMetadata;
use crate::error::BackendError;
use crate::state::{AppState};

#[axum::debug_handler]
pub async fn upload_scene(
    State(state): State<Arc<AppState>>,
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

    let metadata = SceneMetadata {
        id: Uuid::new_v4(),
        name: filename,
    };

    state.repo.add_scene(metadata.clone()).await.expect(format!("Failed to add scene: {}", metadata.id).as_str());
        
    Ok(Json(metadata))
}

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct GpuPoint {
    position: [f32; 3],
    color: [f32; 3],
}

//pub async fn get_pointcloud(
//    State(state): State<AppState>, // Note: Use Arc<AppState> if wrapped
//    Path(scene_id): Path<Uuid>,
//) -> axum::response::Response {
//    let scenes = state.scenes.read().await;
//    let scene = match scenes.get(&scene_id) {
//        Some(scene) => scene,
//        None => return Response::builder()
//            .status(404)
//            .body("Scene not found".into())
//            .unwrap(),
//    };
//
//    let points = scene.raw.points();
//
//    let gpu_points: Vec<GpuPoint> = points
//        .values()
//        .map(|p| GpuPoint {
//            position: p.xyz.to_array(),
//            color: [
//                p.rgb[0] as f32 / 255.0,
//                p.rgb[1] as f32 / 255.0,
//                p.rgb[2] as f32 / 255.0,
//            ],
//        })
//        .collect();
//
//    let bytes = bytemuck::cast_slice(&gpu_points).to_vec();
//
//    Response::builder()
//        .header("Content-Type", "application/octet-stream")
//        .body(bytes.into())
//        .unwrap()
//}

pub async fn list_scenes(State(state): State<Arc<AppState>>) -> Json<Vec<SceneMetadata>> {
    let scenes = state.repo.list_scenes().await.unwrap_or_default();
    Json(scenes)
}