mod colmap;

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use burn::backend::wgpu::WgpuDevice;
use futures::Stream;
use crate::config::LoadConfig;
use crate::Dataset;
use crate::error::FormatError;
use crate::filesystem::Filesystem;

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

pub trait DynStream<Item>: Stream<Item = Item> + SendNotWasm {}
pub type DataStream<T> = Pin<Box<dyn DynStream<Result<T, FormatError>>>>;

pub fn load_dataset(path: PathBuf, config: LoadConfig, device: &WgpuDevice) -> Result<Dataset, std::io::Error> {
    let fs = Arc::new(Filesystem::new(path));

    Ok(colmap::load(fs, config, device))
}