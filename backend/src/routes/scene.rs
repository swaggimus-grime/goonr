use axum::{extract::State, Json};
use std::{
    sync::Arc,
    path::PathBuf,
};
use anyhow::Error;
use axum::body::Body;
use axum::extract::{FromRequest, Multipart, Path, Request};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::{Serialize, Deserialize};
use tempfile::tempdir;
use tokio::fs;
use tracing::{error, info};
use zip_extract::extract;
use db::repo::{SceneMetadata, SplatRepository};
use pipeline::Pipeline;
use web_cmn::scene::{SceneResponse};
use crate::error::{Result, BackendError};
use crate::state::AppState;

use scene_source::Source;

pub async fn upload_scene(
    State(state): State<Arc<AppState>>,
    req: Request<Body>
) -> Result<Json<SceneResponse>> {
    let content_type = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if content_type.contains("multipart/form-data") {
        let multipart = Multipart::from_request(req, &state).await
            .map_err(|_| BackendError::BadRequest("Failed to parse multipart data".into()))?;

        handle_multipart_upload(state, multipart).await
    } else if content_type.contains("application/json") {
        let Json(source) = axum::Json::<Source>::from_request(req, &state).await
            .map_err(|_| BackendError::BadRequest("Failed to parse JSON data".into()))?;

        handle_json_upload(state, source).await
    } else {
        Err(BackendError::BadRequest("Unsupported content type".into()))
    }
}

async fn handle_multipart_upload(
    state: Arc<AppState>,
    mut multipart: Multipart
) -> Result<Json<SceneResponse>> {
    let mut base_path = None;
    let mut scene_name = None;
    let mut source = None;
    while let Some(mut field) = multipart.next_field().await.unwrap_or(None) {
        let field_name = field.name().unwrap_or_default().to_string();
        let filename = field.file_name().unwrap_or("unknown").to_string();
        let data = field.bytes().await
            .map_err(|_| BackendError::BadRequest("Failed to read upload data".into()))?;

        if field_name == "name" {
            let name = String::from_utf8_lossy(&data).to_string();
            scene_name = Some(name.clone());
            if state.repo.can_add(name.as_str()).await {
                base_path = Some(format!("data/scenes/{}", name));
                fs::create_dir_all(base_path.clone().unwrap()).await?;
            } else {
                return Err(BackendError::BadRequest(format!("Upload already exists: {}", name).into()));
            }
        } else if filename.ends_with(".zip") {
            if let Some(dir) = base_path.clone() {
                let zip_path = extract_zip_data(&data, dir.as_str()).await
                    .map_err(|e| BackendError::Internal(Error::from(e)))?;
                source = Some(Source::Zip {path: zip_path.to_string_lossy().to_string()});
            }
        } else {
            if let Some(dir) = base_path.clone() {
                copy_to_dir(&data, filename.as_str(), dir.as_str()).await?;

                if source.is_none() {
                    source = Some(Source::Dir { path: dir });
                }
            }
        }
    }

    if let Some(name) = scene_name {
        let metadata = SceneMetadata {
            name: name.clone(),
            source: source.unwrap(),
        };

        state.repo.add_scene(metadata).await.expect(format!("Failed to add scene: {}", &name).as_str());

        return Ok(Json(SceneResponse {
            name
        }));
    }

    Err(BackendError::BadRequest("Failed to parse multipart data".parse().unwrap()))
}

async fn handle_json_upload(
    state: Arc<AppState>,
    source: Source
) -> Result<Json<SceneResponse>> {
    match source {
        Source::Url { url } => {
            info!("Received URL upload: {}", url);

            let final_source = download_and_process_url(&url).await
                .map_err(|e| BackendError::Internal(e.into()))?;

            let metadata = SceneMetadata {
                name: url.clone(),
                source: final_source,
            };
            
            state.repo.add_scene(metadata.clone()).await?;
            
            Ok(Json(SceneResponse {
                name: url,
            }))
        }
        other => Err(BackendError::BadRequest(format!(
            "Only Source::Url is supported via JSON. Got: {:?}", other
        )))
    }
}

async fn extract_zip_data(
    data: &[u8],
    extract_path: &str
) -> Result<PathBuf> {
    let zip_filename = "scene.zip";
    let zip_path = PathBuf::from(format!("{}/{}", extract_path, zip_filename));

    if let Some(parent) = zip_path.parent() {
        // Spawn dir creation in the background
        let parent = parent.to_path_buf();
        let data = data.to_vec();
        let zip_path_clone = zip_path.clone();
        
        tokio::spawn(async move {
            let _ = tokio::fs::create_dir_all(parent).await;
            let _ = tokio::fs::write(zip_path_clone, data).await;
        });
    }

    Ok(zip_path)
}

fn sanitize_file_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");

    let components: Vec<&str> = normalized
        .split('/')
        .filter(|component| {
            !component.is_empty()
                && *component != "."
                && *component != ".."
                && !component.starts_with('.')
        })
        .collect();

    components.join("/")
}

async fn copy_to_dir(data: &[u8], file_path: &str, dir: &str) -> Result<()> {
    let sanitized_path = sanitize_file_path(file_path);
    let file_path = format!("{}/{}", dir, sanitized_path);

    if let Some(parent) = PathBuf::from(&file_path).parent() {
        fs::create_dir_all(parent).await
            .map_err(|e| BackendError::Internal(e.into()))?;
    }

    tokio::fs::write(&file_path, &data).await
        .map_err(|e| BackendError::Internal(e.into()))
}

async fn download_and_process_url(url: &str) -> Result<( Source)> {
    let base_path = format!("data/scenes/{}", url);

    fs::create_dir_all(&base_path).await.map_err(|e| BackendError::Internal(e.into()))?;

    let response = reqwest::get(url).await.map_err(|e| BackendError::Internal(e.into()))?;
    if !response.status().is_success() {
        return Err(BackendError::BadRequest("Failed to download url".into()));
    }

    // Extract headers before consuming body
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let data = response.bytes().await.map_err(|e| BackendError::Internal(e.into()))?;

    let source = if content_type.contains("application/zip") || url.to_lowercase().ends_with(".zip") {
        extract_zip_data(&data, &base_path).await.map_err(|e| BackendError::Internal(e.into()))?;
        Source::Zip { path: base_path }
    } else {
        let filename = url.split('/').last().unwrap_or("downloaded_file");
        let sanitized = sanitize_file_path(filename);
        let file_path = format!("{}/{}", base_path, sanitized);
        tokio::fs::write(&file_path, &data).await.map_err(|e| BackendError::Internal(e.into()))?;
        Source::Dir { path: base_path }
    };

    Ok(source)
}

pub fn scene_metadata_to_response(metadata: SceneMetadata) -> SceneResponse {
    SceneResponse {
        name: metadata.name,
    }
}

pub async fn get_scene(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>
) -> Result<Json<SceneResponse>> {
    if let Some(scene) = state.repo.get_scene(&name).await? { 
        return Ok(Json(scene_metadata_to_response(scene)));
    }
    Err(BackendError::NotFound)
}

pub async fn get_scenes(
    State(state): State<Arc<AppState>>
) -> Result<Json<Vec<SceneResponse>>> {
    let scenes = state.repo.list_scenes().await?;
    let responses = scenes
        .into_iter()
        .map(scene_metadata_to_response)
        .collect();
    Ok(Json(responses))
}