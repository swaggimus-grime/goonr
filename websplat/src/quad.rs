use wgpu::util::DeviceExt;

// Quad vertices (unit quad)
const QUAD_VERTICES: [[f32; 2]; 4] = [
    [-0.5, -0.5],
    [ 0.5, -0.5],
    [ 0.5,  0.5],
    [-0.5,  0.5],
];

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

pub fn create_quad_buffers(device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer) {
    let quad_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Quad VB"),
        contents: bytemuck::cast_slice(&QUAD_VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });
    
    let quad_ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Quad IB"),
        contents: bytemuck::cast_slice(&QUAD_INDICES),
        usage: wgpu::BufferUsages::INDEX,
    });

    (quad_vb, quad_ib)
}
