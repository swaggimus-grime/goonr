#![recursion_limit = "256"]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use async_fn_stream::try_fn_stream;
use burn_cubecl::cubecl::Runtime;
use burn_wgpu::{WgpuDevice, WgpuRuntime};
use futures::{Stream, StreamExt};
use futures::stream::BoxStream;
use tokio::sync::mpsc::UnboundedSender;
use dataset::LoadConfig;
use scene_source::Source;
use train::config::TrainConfig;
use crate::config::PipelineConfig;
use crate::pipeline_stream::PipelineStream;
use crate::view_stream::ViewStream;

pub use crate::error::PipelineError;
pub use crate::message::PipelineMessage;

mod train_stream;
mod message;
mod view_stream;
mod error;
mod pipeline_stream;
mod config;
mod eval_export;

pub struct Pipeline {
    device: WgpuDevice,
    source: Source
}

impl Pipeline {
    pub fn new(source: Source) -> Result<Self, anyhow::Error> {
        let device = WgpuDevice::default();
        
        Ok(Self {
            device,
            source
        })
    }

    pub fn launch(&mut self) -> impl Stream<Item = Result<PipelineMessage, anyhow::Error>> + 'static
    {
        let device = self.device.clone();
        let source = self.source.clone();

        process_stream(source, device)
    }
}

fn process_stream(source: Source, device: WgpuDevice) -> impl Stream<Item = Result<PipelineMessage, anyhow::Error>> + 'static {
    try_fn_stream(|emitter| async move {
        log::info!("Starting process with source {source:?}");
        emitter.emit(PipelineMessage::NewSource).await;

        let client = WgpuRuntime::client(&device);
        // Start with memory cleared out.
        client.memory_cleanup();

        let mut load_config = LoadConfig::new();
        load_config.eval_split_every = Some(8);
        let mut pipeline_config = PipelineConfig::new();
        pipeline_config.export_path = String::from("eval");
        let train_config = TrainConfig::new();
        train_stream::run(source, load_config, pipeline_config, train_config, device, emitter).await?;

        log::info!("Completed train stream");
        Ok(())
    })
}