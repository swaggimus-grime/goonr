use crate::filesystem::File;
use serde::{Deserialize, Serialize};
use tokio::stream;
use tokio_util::io::StreamReader;
use crate::error::Result;
use crate::filesystem::Filesystem;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Source {
    Zip { path: String },
    Url { url: String },
    Dir { path: String },
}

impl Source {
    pub async fn into_fs(self) -> Result<Filesystem> {
        match self {
            Source::Zip { path } => {
                let file = tokio::fs::File::open(&path).await?;
                Filesystem::from_reader(file).await
            },
            Source::Dir { path } => {
                todo!();
            },
            Source::Url { url } => {
                todo!();
            },
        }
    }
}

