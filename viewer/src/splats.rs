use wgpu::util::DeviceExt;
use web_cmn::splats::RawSplats;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3], // mean
    pub color: [f32; 3],    // RGB (or simplified SH coeffs)
}

impl Vertex {
    pub const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x3, // position
        1 => Float32x3, // color
    ];

    pub fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct GpuSplats {
    pub means: wgpu::Buffer,
    pub rotation: wgpu::Buffer,
    pub log_scales: wgpu::Buffer,
    pub sh_coeffs: wgpu::Buffer,
    pub opacity: wgpu::Buffer,
    pub num_splats: u32,
}

impl GpuSplats {
    pub fn from_raw(device: &wgpu::Device, raw: &RawSplats) -> Self {
        let means = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Means Buffer"),
            contents: bytemuck::cast_slice(&raw.means),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let rotation = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rotation Buffer"),
            contents: bytemuck::cast_slice(&raw.rotation),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let log_scales = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Log Scales Buffer"),
            contents: bytemuck::cast_slice(&raw.log_scales),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let sh_coeffs = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SH Coeffs Buffer"),
            contents: bytemuck::cast_slice(&raw.sh_coeffs),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let opacity = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Opacity Buffer"),
            contents: bytemuck::cast_slice(&raw.raw_opacity),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let num_splats = (raw.means.len() / 3) as u32;

        Self {
            means,
            rotation,
            log_scales,
            sh_coeffs,
            opacity,
            num_splats,
        }
    }
}


pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Splat BindGroupLayout"),
        entries: &[
            // Means
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
            // Rotation
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Log scales
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // SH Coeffs
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Opacity
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

pub fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    splats: &GpuSplats,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Splat BindGroup"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: splats.means.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: splats.rotation.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: splats.log_scales.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: splats.sh_coeffs.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: splats.opacity.as_entire_binding(),
            },
        ],
    })
}
