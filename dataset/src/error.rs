use thiserror::Error;

pub type Result<T> = std::result::Result<T, DatasetError>;

#[derive(Error, Debug)]
pub enum FormatError {
    #[error("IO error while loading dataset: {0}")]
    Io(String),

    #[error("Error decoding camera parameters: {0}")]
    InvalidCamera(&'static str),

    #[error("Image error: {0}")]
    InvalidImage(#[from] image::ImageError),
}

#[derive(Debug, Error)]
pub enum DatasetError {
    #[error("Failed to load format.")]
    FormatError(#[from] FormatError),

    #[error("Format not recognized: Only colmap and nerfstudio json are supported.")]
    FormatNotSupported,
}