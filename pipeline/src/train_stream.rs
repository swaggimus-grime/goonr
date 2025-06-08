use std::path::PathBuf;
use async_fn_stream::{StreamEmitter, TryStreamEmitter};
use async_trait::async_trait;
use tokio_stream::StreamExt;
use dataset::LoadConfig;
use train::TrainContext;
use crate::message::Message;
use crate::pipeline_stream::*;
use crate::PipelineError;

pub struct TrainStream {
    ctx: TrainContext,
    scene_dir: PathBuf,
    load_config: LoadConfig,
}

impl TrainStream {
    pub fn new(scene_dir: PathBuf) -> Self {
        Self {
            ctx: TrainContext::new(),
            scene_dir,
            load_config: LoadConfig::new()
        }
    }
}

#[async_trait]
impl PipelineStream for TrainStream {
    async fn run(&mut self, emitter: TryStreamEmitter<Message, anyhow::Error>) -> anyhow::Result<()> {
        let (mut splat_stream, dataset) = dataset::load_dataset(self.scene_dir.clone(), self.load_config.clone(), self.ctx.device()).await?;
        
        let mut initial_splats = None;

        let estimated_up = dataset.estimate_up();
        
        while let Some(message) = splat_stream.next().await {
            let message = message?;
            let msg = Message::ViewSplats {
                // If the metadata has an up axis prefer that, otherwise estimate
                // the up direction.
                up_axis: message.meta.up_axis.or(Some(estimated_up)),
                splats: Box::new(message.splats.clone()),
                frame: 0,
                total_frames: 0,
            };
            emitter.emit(msg).await;
            initial_splats = Some(message.splats);
        }
        
        Ok(())
    }
}