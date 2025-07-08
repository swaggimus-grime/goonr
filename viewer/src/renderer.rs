use wgpu::util::DeviceExt;
use web_cmn::splats::RawSplats;
use crate::splats::Vertex;

pub struct Renderer {
    bind_group_layout: wgpu::BindGroupLayout,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub vertex_count: usize,
    pub pipeline: Option<wgpu::RenderPipeline>,
}

impl Renderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        // 1. Load shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Splat Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/splat.wgsl").into()),
        });

        // 2. Define bind group layout (will be updated later for splat buffer, camera, etc.)
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Splat Bind Group Layout"),
            entries: &[], // will be filled later
        });

        // 3. Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Splat Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = Some(Self::create_pipeline(device, format, &shader));

        Self {
            pipeline,
            bind_group_layout,
            vertex_buffer: None,
            vertex_count: 0,
        }
    }

    pub fn upload_splats(&mut self, device: &wgpu::Device, splats: &RawSplats) {
        let mut vertices = Vec::new();

        let means = splats.means.chunks(3);
        let colors = splats.sh_coeffs.chunks(3); // Only using SH[0] RGB for now

        for (mean, color) in means.zip(colors) {
            vertices.push(Vertex {
                position: [mean[0], mean[1], mean[2]],
                color: [color[0], color[1], color[2]],
            });
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Splat Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        self.vertex_buffer = Some(vertex_buffer);
        self.vertex_count = vertices.len();
    }

    pub fn create_pipeline(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        shader_module: &wgpu::ShaderModule,
    ) -> wgpu::RenderPipeline {
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Splat Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Splat Render Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: shader_module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Vertex::layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: shader_module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::PointList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        })
    }
}
