use thiserror::Error;

pub type Result<T> = std::result::Result<T, SceneSourceError>;

#[derive(Debug, Error)]
pub enum SceneSourceError {
    #[error("File system error")]
    Filesystem,

    #[error("File IO error")]
    File(#[from] std::io::Error),

    #[error("Zip error")]
    Zip(#[from] zip::result::ZipError),

    #[error("Unknown source")]
    UnknownSource,
}