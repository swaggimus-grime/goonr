use wgpu::util::DeviceExt;
use crate::splats::GpuSplat;
use bytemuck;

pub struct Sorter {
    bitonic_pipeline: wgpu::ComputePipeline,
    map_pipeline: wgpu::ComputePipeline,
    params_buffer: wgpu::Buffer,

    bitonic_bgl: wgpu::BindGroupLayout,
    map_bgl: wgpu::BindGroupLayout,

    bind_group_bitonic: Option<wgpu::BindGroup>,
    bind_group_map: Option<wgpu::BindGroup>,

    input_buffer: Option<wgpu::Buffer>,
    full_splats_buffer: Option<wgpu::Buffer>,
    output_buffer: Option<wgpu::Buffer>,

    num_splats: u32,
    padded_n: u32,
}

impl Sorter {
    pub fn new(device: &wgpu::Device) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/sort.wgsl"));

        let bitonic_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bitonic BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding:0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage{read_only:false}, has_dynamic_offset:false, min_binding_size:None },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding:1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset:false, min_binding_size:None },
                    count: None,
                },
            ],
        });

        let map_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Map BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding:0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage{read_only:true}, has_dynamic_offset:false, min_binding_size:None },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding:1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage{read_only:true}, has_dynamic_offset:false, min_binding_size:None },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding:2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage{read_only:false}, has_dynamic_offset:false, min_binding_size:None },
                    count: None,
                },
            ],
        });

        let bitonic_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Bitonic Pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Bitonic Layout"),
                bind_group_layouts: &[&bitonic_bgl],
                push_constant_ranges: &[],
            })),
            module: &shader,
            entry_point: Some("bitonic_step"),
            compilation_options: Default::default(),
            cache: None,
        });

        let map_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Map Pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Map Layout"),
                bind_group_layouts: &[&map_bgl],
                push_constant_ranges: &[],
            })),
            module: &shader,
            entry_point: Some("map_to_full"),
            compilation_options: Default::default(),
            cache: None,
        });

        let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Params Buffer"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            bitonic_pipeline,
            map_pipeline,
            params_buffer,
            bitonic_bgl,
            map_bgl,
            bind_group_bitonic: None,
            bind_group_map: None,
            input_buffer: None,
            full_splats_buffer: None,
            output_buffer: None,
            num_splats: 0,
            padded_n: 0,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, num_splats: usize) {
        let mut padded = 1usize;
        while padded < num_splats { padded <<= 1; }
        let size = (padded * std::mem::size_of::<GpuSplat>()) as u64;

        self.output_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        }));

        self.num_splats = num_splats as u32;
        self.padded_n = padded as u32;
        self.bind_group_bitonic = None;
        self.bind_group_map = None;
    }

    pub fn set_input_from_preprocessor(&mut self, device: &wgpu::Device, input: &wgpu::Buffer, full_splats: &wgpu::Buffer) {
        if self.num_splats == 0 { return; }
        let output = match &self.output_buffer { Some(b)=>b, None=>return };

        self.bind_group_bitonic = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bitonic BG"),
            layout: &self.bitonic_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding:0, resource: input.as_entire_binding() },
                wgpu::BindGroupEntry { binding:1, resource: self.params_buffer.as_entire_binding() },
            ],
        }));

        self.bind_group_map = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Map BG"),
            layout: &self.map_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding:0, resource: input.as_entire_binding() },
                wgpu::BindGroupEntry { binding:1, resource: full_splats.as_entire_binding() },
                wgpu::BindGroupEntry { binding:2, resource: output.as_entire_binding() },
            ],
        }));

        self.input_buffer = Some(input.clone());
        self.full_splats_buffer = Some(full_splats.clone());
    }

    pub fn run(&self, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder) {
        if self.num_splats == 0 { return; }
        let bg_bitonic = match &self.bind_group_bitonic { Some(b)=>b, None=>return };
        let bg_map = match &self.bind_group_map { Some(b)=>b, None=>return };

        let N = self.padded_n;
        let group_count = ((N+63)/64).max(1);

        let mut k = 2;
        while k <= N {
            let mut j = k/2;
            while j >= 1 {
                queue.write_buffer(&self.params_buffer, 0, bytemuck::cast_slice(&[k,j]));
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label:Some("Bitonic Pass"), timestamp_writes: None });
                pass.set_pipeline(&self.bitonic_pipeline);
                pass.set_bind_group(0, bg_bitonic, &[]);
                pass.dispatch_workgroups(group_count,1,1);
                j /= 2;
            }
            k <<= 1;
        }

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label:Some("Map Pass"), timestamp_writes: None });
        pass.set_pipeline(&self.map_pipeline);
        pass.set_bind_group(0, bg_map, &[]);
        pass.dispatch_workgroups(group_count,1,1);
    }

    pub fn output_buffer(&self) -> Option<&wgpu::Buffer> { self.output_buffer.as_ref() }
}
