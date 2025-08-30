use thiserror::Error;

pub type Result<T> = std::result::Result<T, FrontendError>;

#[derive(Error, Debug)]
pub enum FrontendError {
    #[error("Bad Request: {0}")]
    BadRequest(#[from] gloo_net::Error),
    
    #[error("Viewer error: {0}")]
    ViewerError(&'static str),
    
    #[error("WebSplat error: {0}")]
    WebSplatError(#[from] websplat::WebSplatError)
}