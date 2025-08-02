use std::ops::Deref;
use wgpu::{Device, Queue};
use wgpu::util::DeviceExt;
use web_cmn::splats::RawSplats;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuSplat {
    pub mean: [f32; 3],
    _pad0: f32,
    pub rotation: [f32; 3],
    _pad1: f32,
    pub log_scale: [f32; 3],
    _pad2: f32,
    pub opacity: f32,
    _pad3: [f32; 3],
}

impl GpuSplat {
    pub const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        1 => Float32x3, // mean
        2 => Float32x3, // rotation
        3 => Float32x3, // log_scale
        4 => Float32,   // opacity
    ];

    pub fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: Self::ATTRIBUTES,
        }
    }

    fn vec_from_raw(raw: &RawSplats) -> Vec<Self> {
        let count = raw.means.len() / 3;
        let mut out = Vec::with_capacity(count);

        for i in 0..count {
            out.push(GpuSplat {
                mean: [
                    raw.means[i * 3],
                    raw.means[i * 3 + 1],
                    raw.means[i * 3 + 2],
                ],
                _pad0: 0.0,
                rotation: [
                    raw.rotation[i * 3],
                    raw.rotation[i * 3 + 1],
                    raw.rotation[i * 3 + 2],
                ],
                _pad1: 0.0,
                log_scale: [
                    raw.log_scales[i * 3],
                    raw.log_scales[i * 3 + 1],
                    raw.log_scales[i * 3 + 2],
                ],
                _pad2: 0.0,
                opacity: raw.raw_opacity[i],
                _pad3: [0.0; 3],
            });
        }

        out
    }
}

pub struct GpuSplats {
    buffer: wgpu::Buffer,
    count: u32,
}

impl GpuSplats {
    pub fn from_raw(device: &Device, raw: &RawSplats) -> Self {
        let splats = GpuSplat::vec_from_raw(raw);
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Splats Instance Buffer"),
            contents: bytemuck::cast_slice(&splats),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        
        Self {
            buffer,
            count: splats.len() as u32,
        }
    }
    
    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn rewrite(&mut self, queue: &Queue, raw: &RawSplats) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&GpuSplat::vec_from_raw(raw)));
    }
}

impl Deref for GpuSplats {
    type Target = wgpu::Buffer;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }   
}