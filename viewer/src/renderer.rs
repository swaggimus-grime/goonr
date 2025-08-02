use std::ops::Deref;
use glam::{Mat4, UVec2, Vec2, Vec3, Vec4};
use wgpu::{Device, StoreOp, SurfaceConfiguration};
use wgpu::util::DeviceExt;
use web_cmn::splats::RawSplats;
use crate::{quad, Context};
use crate::camera::CameraUniform;
use crate::quad::Quad;
use crate::splats::{GpuSplat};

struct RenderUniforms {
    view_mat: Mat4,
    cam_pos: Vec4,
    focal: Vec2,
    pixel_center: Vec2,
    img_size: UVec2,
    tile_bounds: UVec2,
    sh_degree: u32,
}

pub struct Renderer {
    quad: Quad,
    shader: wgpu::ShaderModule,
    pipeline: wgpu::RenderPipeline,
    pipeline_layout: wgpu::PipelineLayout,
    uniform_buf: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
}

impl Renderer {
    pub fn new(ctx: &Context) -> Self {
        let quad = Quad::new(&ctx.device);

        let shader = ctx.device.create_shader_module(wgpu::include_wgsl!("shaders/splat.wgsl"));

        let cam_uniform = CameraUniform {
            view_proj: ctx.camera.get_view_proj().to_cols_array_2d(),
        };

        let uniform_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[cam_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout = ctx.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT, // use FRAGMENT too if needed
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let uniform_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
            label: Some("Camera Bind Group"),
        });

        let pipeline_layout = ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Splat Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Splat Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[
                    Quad::layout(),
                    GpuSplat::layout(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: ctx.config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            quad,
            shader,
            pipeline,
            pipeline_layout,
            uniform_bind_group,
            uniform_bind_group_layout,
            uniform_buf,
        }
    }

    pub fn render(&self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder, ctx: &mut Context) {
        let Some(gpu_splats) = &ctx.gpu_splats else { return };

        let cam_uniform = CameraUniform {
            view_proj: ctx.camera.get_view_proj().to_cols_array_2d(),
        };

        ctx.queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(&[cam_uniform]));

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Splat Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.quad.vb.slice(..));
        render_pass.set_vertex_buffer(1, gpu_splats.deref().slice(..));
        render_pass.set_index_buffer(self.quad.ib.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..gpu_splats.count());
    }
}