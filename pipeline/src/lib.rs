use std::path::{Path, PathBuf};
use async_fn_stream::try_fn_stream;
use futures::Stream;
use crate::message::Message;

mod training;
mod message;



pub async fn splat_stream(scene_path: PathBuf) -> impl Stream<Item = Result<Message, anyhow::Error>> + 'static {
    try_fn_stream(|emitter| async move {
        emitter.emit(Message::NewSource).await;
        
        let (mut stream, dataset) = dataset::load_dataset(scene_path, dataset::LoadConfig::new())?;
        
    });
}

