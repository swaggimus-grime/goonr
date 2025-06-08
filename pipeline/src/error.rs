use thiserror::Error;
use dataset::DatasetError;

pub type Result<T> = std::result::Result<T, PipelineError>;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("Failed to load format.")]
    DatasetError(#[from] DatasetError),
    
}