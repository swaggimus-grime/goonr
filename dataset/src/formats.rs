mod colmap;

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use async_fn_stream::try_fn_stream;
use burn::backend::wgpu::WgpuDevice;
use futures::Stream;
use scene_source::Source;
use crate::config::LoadConfig;
use crate::Dataset;
use crate::error::{FormatError};

// On wasm, lots of things aren't Send that are send on non-wasm.
// Non-wasm tokio requires :Send for futures, tokio_with_wasm doesn't.
// So, it can help to annotate futures/objects as send only on not-wasm.
#[cfg(target_family = "wasm")]
mod wasm_send {
    pub trait SendNotWasm {}
    impl<T> SendNotWasm for T {}
}

#[cfg(not(target_family = "wasm"))]
mod wasm_send {
    pub trait SendNotWasm: Send {}
    impl<T: Send> SendNotWasm for T {}
}

pub use wasm_send::*;
use crate::scene::splat::SplatMessage;

pub trait DynStream<Item>: Stream<Item = Item> + SendNotWasm {}
impl<Item, T: Stream<Item = Item> + SendNotWasm> DynStream<Item> for T {}

pub type DataStream<T> = Pin<Box<dyn DynStream<Result<T, FormatError>>>>;

pub async fn load_dataset(source: Source, config: LoadConfig, device: &WgpuDevice) -> crate::error::Result<(DataStream<SplatMessage>, Dataset)> {
    let fs = source.into_fs().await?;
    Ok(colmap::load(Arc::new(fs), config, device).await?)
}