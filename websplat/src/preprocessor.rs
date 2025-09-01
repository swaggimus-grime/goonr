//preprocessor.rs
use std::sync::Arc;
use wgpu::util::DeviceExt;
use crate::splats::GpuSplat;
use bytemuck;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct VisibleSplat {
    pos: [f32; 4],
    index: u32,
    depth: f32,
    _pad: [u32;2],
}

impl VisibleSplat {
    fn sentinel() -> Self {
        Self { pos:[0.0;4], index: u32::MAX, depth: 1.0, _pad:[0;2] }
    }
}

pub struct Preprocessor {
    pipeline: wgpu::ComputePipeline,        // cull pipeline (entry "main")
    reset_pipeline: wgpu::ComputePipeline,  // reset pipeline (entry "reset_main")

    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: Option<wgpu::BindGroup>,

    src_buffer: Option<wgpu::Buffer>,        // input splats
    compact_buffer: Option<wgpu::Buffer>,    // visible splats
    camera_buffer: Option<wgpu::Buffer>,     // camera uniform
    indirect_buffer: Option<wgpu::Buffer>,   // DrawIndexedIndirect args

    pub(crate) num_splats: u32,
    pub last_visible_count: Arc<std::sync::atomic::AtomicU32>, // optional readback
}

impl Preprocessor {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/preprocess.wgsl"));

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Preprocessor BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry { binding:0, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage {read_only:true}, has_dynamic_offset:false, min_binding_size:None }, count: None },
                wgpu::BindGroupLayoutEntry { binding:1, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset:false, min_binding_size:None }, count: None },
                wgpu::BindGroupLayoutEntry { binding:2, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage {read_only:false}, has_dynamic_offset:false, min_binding_size:None }, count: None },
                wgpu::BindGroupLayoutEntry { binding:3, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage {read_only:false}, has_dynamic_offset:false, min_binding_size:None }, count: None },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Preprocessor Pipeline Layout"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let reset_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Preprocessor Reset Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("reset_main"),
            compilation_options: Default::default(),
            cache: None,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Preprocessor Cull Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            pipeline,
            reset_pipeline,
            bind_group_layout: bgl,
            bind_group: None,
            src_buffer: None,
            compact_buffer: None,
            camera_buffer: None,
            indirect_buffer: None,
            num_splats: 0,
            last_visible_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, padded_n: usize, active_n: usize) {
        let src_size = (padded_n * std::mem::size_of::<GpuSplat>()) as u64;
        let compact_size = (padded_n * std::mem::size_of::<VisibleSplat>()) as u64;

        self.src_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Preprocessor Src"),
            size: src_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));

        let sentinel_vec = vec![VisibleSplat::sentinel(); padded_n];
        self.compact_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Compacted Init"),
            contents: bytemuck::cast_slice(&sentinel_vec),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_SRC,
        }));

        // indirect buffer for draw_indexed_indirect
        let zero_args: [u32;5] = [0u32;5];
        self.indirect_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Draw Indirect Args"),
            contents: bytemuck::cast_slice(&zero_args),
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        }));

        self.num_splats = active_n as u32;
        self.bind_group = None;
    }

    pub fn upload_splats(&self, queue: &wgpu::Queue, splats: &[GpuSplat]) {
        if let Some(src) = &self.src_buffer {
            queue.write_buffer(src, 0, bytemuck::cast_slice(splats));
        }
        if let Some(indirect) = &self.indirect_buffer {
            let zero_args: [u32;5] = [0u32;5];
            queue.write_buffer(indirect, 0, bytemuck::cast_slice(&zero_args));
        }
    }

    pub fn set_camera_buffer(&mut self, device: &wgpu::Device, camera: &wgpu::Buffer) {
        self.camera_buffer = Some(camera.clone());
        if let (Some(src), Some(compact), Some(cam), Some(indirect)) =
            (&self.src_buffer, &self.compact_buffer, &self.camera_buffer, &self.indirect_buffer)
        {
            self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Preprocessor BG"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: src.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 1, resource: cam.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 2, resource: compact.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 3, resource: indirect.as_entire_binding() },
                ],
            }));
        }
    }

    pub fn run(&self, encoder: &mut wgpu::CommandEncoder) {
        if self.num_splats == 0 { return; }
        let bg = match &self.bind_group { Some(bg) => bg, None => return };
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("Preprocessor"), timestamp_writes: None });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, bg, &[]);
        let workgroups = ((self.num_splats + 255) / 256).max(1);
        pass.dispatch_workgroups(workgroups, 1, 1);
    }

    pub fn compacted_buffer(&self) -> Option<&wgpu::Buffer> { self.compact_buffer.as_ref() }
    pub fn indirect_buffer(&self) -> Option<&wgpu::Buffer> { self.indirect_buffer.as_ref() }

    pub fn reset_counter(&self, queue: &wgpu::Queue) {
        if let Some(indirect) = &self.indirect_buffer {
            let zero_args: [u32;5] = [0u32;5];
            queue.write_buffer(indirect, 0, bytemuck::cast_slice(&zero_args));
        }
    }

    pub fn run_reset_on_gpu(&self, encoder: &mut wgpu::CommandEncoder) {
        let bg = match &self.bind_group { Some(bg) => bg, None => return };
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("Preprocessor Reset"), timestamp_writes: None });
        pass.set_pipeline(&self.reset_pipeline);
        pass.set_bind_group(0, bg, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }
}
