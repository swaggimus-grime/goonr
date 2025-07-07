#![recursion_limit = "256"]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use async_fn_stream::try_fn_stream;
use burn_cubecl::cubecl::Runtime;
use burn_wgpu::{WgpuDevice, WgpuRuntime};
use futures::{Stream, StreamExt};
use futures::stream::BoxStream;
use tokio::sync::mpsc::UnboundedSender;
use scene_source::Source;
use crate::pipeline_stream::PipelineStream;
use crate::train_stream::TrainStream;
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

    pub async fn launch(&mut self, send: UnboundedSender<anyhow::Result<PipelineMessage>>)
        -> impl Stream<Item = Result<PipelineMessage, anyhow::Error>>
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

        //let vfs = Arc::new(source.clone().into_fs().await?);

        let client = WgpuRuntime::client(&device);
        // Start with memory cleared out.
        client.memory_cleanup();

        /**
        if vfs_counts == ply_count {
            drop(process_args);
            view_stream(vfs, device, emitter).await?;
        } else {
            // Receive the processing args.
            train_stream(vfs, process_args, device, emitter).await?;
        };
        */

        let mut stream = TrainStream::new(source, device);
        stream.run(emitter).await?;

        log::info!("Completed train stream");
        Ok(())
    })
}