use async_fn_stream::TryStreamEmitter;
use async_trait::async_trait;
use crate::error::PipelineError;
use crate::message::Message;
use crate::pipeline_stream::PipelineStream;

pub struct ViewStream {

}

impl ViewStream {
    pub fn new() -> Self {
        Self {
            
        }
    }
}

#[async_trait]
impl PipelineStream for ViewStream {
    async fn run(&mut self, emitter: TryStreamEmitter<Message, anyhow::Error>) -> anyhow::Result<()> {
        todo!()
    }
}