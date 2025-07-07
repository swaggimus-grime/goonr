use thiserror::Error;

pub type Result<T> = std::result::Result<T, FrontendError>;

#[derive(Error, Debug)]
pub enum FrontendError {
    #[error("Bad Request: {0}")]
    BadRequest(#[from] gloo_net::Error)
}