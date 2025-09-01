use glam::Mat4;
use crate::camera::CameraUniform;
use crate::quad::create_quad_buffers;
use crate::splats::GpuSplat;
use wgpu::util::DeviceExt;

pub struct Renderer {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: Option<wgpu::BindGroup>,
    pub instance_buffer: Option<wgpu::Buffer>,
    camera_buffer: wgpu::Buffer,
    quad_vb: wgpu::Buffer,
    quad_ib: wgpu::Buffer,
    pub num_splats: usize,
}

impl Renderer {
    pub fn new(device: &wgpu::Device) -> Self {
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Splat Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Splat Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/render.wgsl"));

        let (quad_vb, quad_ib) = create_quad_buffers(device);

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Splat Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[
                    // Quad vertex buffer
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32;2]>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0=>Float32x2],
                    },
                    // Instance buffer
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<GpuSplat>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            bind_group: None,
            instance_buffer: None,
            camera_buffer,
            quad_vb,
            quad_ib,
            num_splats: 0,
        }
    }

    pub fn set_instance_buffer(&mut self, device: &wgpu::Device, instance_buffer: &wgpu::Buffer) {
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Splat Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding:0, resource: instance_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding:1, resource: self.camera_buffer.as_entire_binding() },
            ],
        }));
        self.instance_buffer = Some(instance_buffer.clone());
    }

    pub fn update_camera(&self, queue: &wgpu::Queue, view_proj: &Mat4) {
        let uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
        };
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::bytes_of(&uniform));
    }

    pub fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, num_splats: u32) {
        // Early out if no instance buffer
        let instance_buffer = match &self.instance_buffer {
            Some(buf) => buf,
            None => return,
        };

        pass.set_pipeline(&self.pipeline);
        if let Some(bg) = &self.bind_group {
            pass.set_bind_group(0, bg, &[]);
        }
        pass.set_vertex_buffer(0, self.quad_vb.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.slice(..)); // guaranteed Some
        pass.set_index_buffer(self.quad_ib.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..6, 0, 0..num_splats);
    }

    /// New: issue an indirect indexed draw using the provided indirect buffer.
    pub fn draw_indirect<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, indirect: &wgpu::Buffer) {
        // Early out if no instance buffer
        let instance_buffer = match &self.instance_buffer {
            Some(buf) => buf,
            None => return,
        };

        pass.set_pipeline(&self.pipeline);
        if let Some(bg) = &self.bind_group {
            pass.set_bind_group(0, bg, &[]);
        }
        pass.set_vertex_buffer(0, self.quad_vb.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.slice(..));
        pass.set_index_buffer(self.quad_ib.slice(..), wgpu::IndexFormat::Uint16);

        // Issues draw_indexed_indirect(indirect_buffer, offset)
        pass.draw_indexed_indirect(indirect, 0);
    }

    pub fn camera_buffer(&self) -> &wgpu::Buffer { &self.camera_buffer }
}
