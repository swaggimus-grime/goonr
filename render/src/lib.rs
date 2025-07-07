#![recursion_limit = "256"]

use burn::prelude::Backend;
use burn::tensor::ops::FloatTensor;
use burn_cubecl::CubeBackend;
use burn_fusion::Fusion;
use burn_wgpu::graphics::{AutoGraphicsApi, GraphicsApi};
use burn_wgpu::{RuntimeOptions, WgpuDevice, WgpuRuntime};
use camera::Camera;
use glam::Vec3;
use render_aux::RenderAux;
use wgpu::{Adapter, Device, Queue};

mod burn_glue;
mod dim_check;
mod kernels;
pub mod render_aux;
pub mod sh;
pub mod bounding_box;
pub mod camera;
pub mod gaussian_splats;
pub mod render;
pub mod shaders;

pub type MainBackendBase = CubeBackend<WgpuRuntime, f32, i32, u32>;
pub type MainBackend = Fusion<MainBackendBase>;

#[derive(Debug, Clone)]
pub struct RenderStats {
    pub num_visible: u32,
    pub num_intersections: u32,
}

// This is limited by the sorting kernels which have 256 WG with 4 elems each -> 65k dispatches of 1024
const INTERSECTS_UPPER_BOUND: u32 = 1024 * 65535;
const GAUSSIANS_UPPER_BOUND: u32 = 256 * 65535;

pub trait SplatForward<B: Backend> {
    /// Render splats to a buffer.
    ///
    /// This projects the gaussians, sorts them, and rasterizes them to a buffer, in a
    /// differentiable way.
    /// The arguments are all passed as raw tensors. See [`Splats`] for a convenient Module that wraps this fun
    /// The [`xy_grad_dummy`] variable is only used to carry screenspace xy gradients.
    /// This function can optionally render a "u32" buffer, which is a packed RGBA (8 bits per channel)
    /// buffer. This is useful when the results need to be displayed immediately.
    fn render_splats(
        camera: &Camera,
        img_size: glam::UVec2,
        means: FloatTensor<B>,
        log_scales: FloatTensor<B>,
        quats: FloatTensor<B>,
        sh_coeffs: FloatTensor<B>,
        raw_opacities: FloatTensor<B>,
        background: Vec3,
        bwd_info: bool,
    ) -> (FloatTensor<B>, RenderAux<B>);
}

fn burn_options() -> RuntimeOptions {
    RuntimeOptions {
        tasks_max: 64,
        memory_config: burn_wgpu::MemoryConfiguration::ExclusivePages,
    }
}

pub fn burn_init_device(adapter: Adapter, device: Device, queue: Queue) -> WgpuDevice {
    let setup = burn_wgpu::WgpuSetup {
        instance: wgpu::Instance::new(&wgpu::InstanceDescriptor::default()), // unused... need to fix this in Burn.
        adapter,
        device,
        queue,
        backend: AutoGraphicsApi::backend(),
    };
    burn_wgpu::init_device(setup, burn_options())
}

pub async fn burn_init_setup() -> WgpuDevice {
    burn_wgpu::init_setup_async::<AutoGraphicsApi>(&WgpuDevice::DefaultDevice, burn_options())
        .await;
    WgpuDevice::DefaultDevice
}