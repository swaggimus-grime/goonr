use thiserror::Error;

pub(crate) type WsResult<T> = std::result::Result<T, WebSplatError>;

#[derive(Debug, Error)]
pub enum WebSplatError {
    #[error("Context creation failed")]
    ContextCreation,
    
    #[error("Surface error: {0}")]
    SurfaceError(#[from] wgpu::SurfaceError),

}
