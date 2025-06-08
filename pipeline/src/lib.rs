use std::path::{Path, PathBuf};
use async_fn_stream::try_fn_stream;
use futures::Stream;
use crate::pipeline_stream::PipelineStream;
use crate::train_stream::TrainStream;
use crate::view_stream::ViewStream;
use crate::error::Result;

pub use crate::error::PipelineError;

mod train_stream;
mod message;
mod view_stream;
mod error;
mod pipeline_stream;

pub struct Pipeline {
    train: TrainStream,
    view: ViewStream,
}

impl Pipeline {
    pub fn new(scene_dir: PathBuf) -> Result<Self> {
        let train = TrainStream::new(scene_dir);
        let view =  ViewStream::new();
        
        Ok(Self {
            train,
            view
        })
    }
    
    pub fn launch(&mut self) {
        self.train.launch();
        self.view.launch();
    }
}


