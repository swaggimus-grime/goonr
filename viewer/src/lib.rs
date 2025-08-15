pub mod error;
mod splats;
mod renderer;
mod camera;
mod quad;
mod surface;

use std::ops::Deref;
use glam::Vec3;
use web_sys::HtmlCanvasElement;
use wgpu::StoreOp;
use wgpu::util::DeviceExt;
use web_cmn::splats::RawSplats;
use crate::camera::Camera;
use crate::error::{Result, ViewerError};
pub use crate::renderer::Renderer;
use crate::splats::{GpuSplat, GpuSplats};
use crate::surface::Surface;

pub struct Context<'window> {
    instance: wgpu::Instance,
    surface: Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    gpu_splats: Option<GpuSplats>,
    pub camera: Camera
}

impl<'window> Context<'window> {
    pub async fn from_html_canvas(canvas: HtmlCanvasElement) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = Surface::from_canvas(&instance, canvas);

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(surface.deref()),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("Failed to get device");

        let surface_caps = surface.get_capabilities(&adapter);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps.formats[0],
            width: surface.width(),
            height: surface.height(),
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 1,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        let aspect = config.width as f32 / config.height as f32;
        let camera = Camera::new(aspect);

        Ok(Self {
            instance,
            surface,
            device,
            queue,
            gpu_splats: None,
            camera
        })
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width == 0 || new_height == 0 {
            return; // Avoid resizing to invalid size
        }

        self.surface.resize(&self.device, new_width, new_height);
    }

    pub fn upload_splats(&mut self, raw: &RawSplats) {
        self.gpu_splats = Some(GpuSplats::from_raw(&self.device, raw));
    }

    pub fn render_frame(&mut self, renderer: &Renderer) {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => {
                self.surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture")
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        renderer.render(&view, &mut encoder, self);

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
