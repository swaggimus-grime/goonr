use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneMetadata {
    pub id: Uuid,
    pub name: String,
    pub file_path: PathBuf
}