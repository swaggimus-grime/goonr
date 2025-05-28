// crates/viewer/src/viewer.rs

use glam::Mat4;
use wgpu::{Device, Queue, SurfaceConfiguration, Surface, RenderPipeline, Buffer};
use crate::camera::Camera;

pub struct Renderer {
    pub pipeline: RenderPipeline,
    pub camera: Camera,
    pub vertex_buffer: Option<Buffer>,
    pub vertex_count: usize,
}

impl Renderer {
    pub fn new(device: &Device, config: &SurfaceConfiguration) -> Self {
        let camera = Camera::new(config.width as f32 / config.height as f32);

        // Dummy placeholder pipeline (setup in pipeline.rs in the future)
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Temp Shader"),
            source: wgpu::ShaderSource::Wgsl("".into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            pipeline,
            camera,
            vertex_buffer: None,
            vertex_count: 0,
        }
    }

    pub fn render(
        &self,
        device: &Device,
        queue: &Queue,
        surface: &Surface,
        config: &SurfaceConfiguration,
    ) {
        let frame = surface
            .get_current_texture()
            .expect("Failed to acquire next swap chain texture");
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            if let Some(buffer) = &self.vertex_buffer {
                render_pass.set_vertex_buffer(0, buffer.slice(..));
                render_pass.draw(0..self.vertex_count as u32, 0..1);
            }
        }

        queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
