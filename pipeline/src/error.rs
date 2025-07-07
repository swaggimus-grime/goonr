use thiserror::Error;
use dataset::error::DatasetError;

pub type Result<T> = std::result::Result<T, PipelineError>;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("Dataset Error")]
    Dataset(#[from] DatasetError),
    
}