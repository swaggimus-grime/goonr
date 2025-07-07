use async_fn_stream::{try_fn_stream, StreamEmitter, TryFnStream, TryStreamEmitter};
use async_trait::async_trait;
use futures::Stream;
use futures::stream::BoxStream;
use crate::error::PipelineError;
use crate::message::PipelineMessage;

#[async_trait]
pub trait PipelineStream: Send {
    async fn run(&mut self, emitter: TryStreamEmitter<PipelineMessage, anyhow::Error>) -> anyhow::Result<()>;
}
