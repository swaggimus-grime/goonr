use wgpu::Buffer;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pub corner: [f32; 2],
}

pub struct Quad {
    pub vb: Buffer,
    pub ib: Buffer,
}

impl Quad {
    pub fn new(device: &wgpu::Device) -> Self {
        let vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(&Self::VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Index Buffer"),
            contents: bytemuck::cast_slice(&Self::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self { vb, ib }
    }

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2],
        }
    }

    const VERTICES: [Vertex; 4] = [
        Vertex { corner: [-1.0, -1.0] },
        Vertex { corner: [1.0, -1.0] },
        Vertex { corner: [1.0, 1.0] },
        Vertex { corner: [-1.0, 1.0] },
    ];

    const INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];
}
