pub mod error;
mod splats;
mod renderer;

use web_sys::HtmlCanvasElement;
use wgpu::StoreOp;
use web_cmn::splats::RawSplats;
use crate::error::{ViewerError, Result};
use crate::renderer::Renderer;

pub struct Context<'window> {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: Renderer,
}

impl<'window> Context<'window> {
    pub async fn from_html_canvas(canvas: HtmlCanvasElement) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let (width, height) = (canvas.width(), canvas.height());

        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
            .expect("Failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
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
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 1,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let renderer = Renderer::new(&device, config.format);

        Ok(Self {
            instance,
            surface,
            device,
            queue,
            renderer,
        })
    }

    pub fn draw_splats(&mut self, splats: RawSplats) {
        self.renderer.upload_splats(&self.device, &splats);

        // Step 2: Get current frame
        let output = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(err) => {
                eprintln!("Dropped frame: {err}");
                return;
            }
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Step 3: Begin encoder and render pass
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Splat Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if let (Some(pipeline), Some(vertex_buffer)) = (
                &self.renderer.pipeline,
                &self.renderer.vertex_buffer,
            ) {
                render_pass.set_pipeline(pipeline);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw(0..self.renderer.vertex_count as u32, 0..1);
            }
        }

        // Step 4: Submit to GPU and present
        self.queue.submit(Some(encoder.finish()));
        output.present();
    }
}


