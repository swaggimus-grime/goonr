pub mod camera;
pub mod sh;
pub mod splat;

use burn::backend::wgpu::{CubeBackend, WgpuRuntime};
use burn_fusion::Fusion;
pub use camera::Camera;

pub type MainBackendBase = CubeBackend<WgpuRuntime, f32, i32, u32>;
pub type MainBackend = Fusion<MainBackendBase>;

/*
pub async fn create_render_device(canvas: HtmlCanvasElement) -> (Device, Queue, Surface, SurfaceConfiguration) {
    let instance = wgpu::Instance::default();
    let surface = unsafe { instance.create_surface(&canvas) }.unwrap();

    let adapter = instance.request_adapter(&RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }).await.unwrap();

    let (device, queue) = adapter.request_device(&DeviceDescriptor::default(), None).await.unwrap();

    let config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_capabilities(&adapter).formats[0],
        width: canvas.width(),
        height: canvas.height(),
        present_mode: PresentMode::Fifo,
        alpha_mode: CompositeAlphaMode::Auto,
        view_formats: vec![],
    };

    surface.configure(&device, &config);

    (device, queue, surface, config)
}

 */