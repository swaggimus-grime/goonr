use thiserror::Error;

pub type Result<T> = std::result::Result<T, ViewerError>;

#[derive(Debug, Error)]
pub enum ViewerError {
    #[error("Context creation failed")]
    ContextCreation,
    
}
