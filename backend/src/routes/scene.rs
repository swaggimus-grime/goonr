use std::io::Cursor;
use std::path::{PathBuf};
use std::sync::Arc;
use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::{debug_handler, extract, Json};
use axum::response::{IntoResponse};
use log::info;
use uuid::Uuid;
use zip::ZipArchive;
use crate::error::BackendError;
use crate::state::{AppState, Scene, SceneMetadata};

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

    let bytes = field
        .bytes()
        .await
        .map_err(|e| BackendError::BadRequest(format!("Failed to read file: {e}")))?;

    let scene_id = Uuid::new_v4();
    let scene_path = PathBuf::from(format!("/tmp/goonr_uploads/{scene_id}"));
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

    let raw_scene = Arc::new(ml::Scene::new(&scene_path).await?);

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

pub async fn get_scene(
    State(state): State<AppState>,
    extract::Path(scene_id): extract::Path<Uuid>,
) -> Result<Json<Scene>, BackendError> {
    let scenes = state.scenes.read().await;

    if let Some(scene) = scenes.get(&scene_id) {
        Ok(Json(scene.clone()))
    } else {
        Err(BackendError::NotFound)
    }
}