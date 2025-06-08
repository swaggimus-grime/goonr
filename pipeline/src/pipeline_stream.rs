use async_fn_stream::{try_fn_stream, StreamEmitter, TryFnStream, TryStreamEmitter};
use async_trait::async_trait;
use futures::Stream;
use crate::error::PipelineError;
use crate::message::Message;

#[async_trait]
pub trait PipelineStream {
    async fn run(&mut self, emitter: TryStreamEmitter<Message, anyhow::Error>) -> anyhow::Result<()>;
    
    fn launch(&mut self) -> impl Stream<Item = Result<Message, anyhow::Error>>  {
        try_fn_stream(|emitter| async move {
            self.run(emitter).await?;
            Ok(())
        })
    }
}