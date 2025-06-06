pub mod camera;
pub mod sh;
pub mod splat;

use burn::backend::wgpu::{CubeBackend, WgpuRuntime};
use burn_fusion::Fusion;
pub use camera::Camera;

pub type MainBackendBase = CubeBackend<WgpuRuntime, f32, i32, u32>;
pub type MainBackend = Fusion<MainBackendBase>;