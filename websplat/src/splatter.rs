//splatter.rs
use wgpu::util::DeviceExt;
use web_cmn::splats::RawSplats;
use crate::camera::Camera;
use crate::context::Context;
use crate::preprocessor::Preprocessor;
use crate::renderer::Renderer;
use crate::sorter::Sorter;
use crate::splats::GpuSplat;
use bytemuck;

/// Manages splat rendering, including preprocessing, optional sorting, and drawing.
pub struct Splatter {
    renderer: Renderer,
    preprocessor: Preprocessor,
    sorter: Sorter,
    full_splats: Option<wgpu::Buffer>, // GPU buffer holding all GpuSplats
}

impl Splatter {
    pub fn new(ctx: &Context) -> Self {
        let renderer = Renderer::new(&ctx.device);
        let preprocessor = Preprocessor::new(&ctx.device);
        let sorter = Sorter::new(&ctx.device);

        Self {
            renderer,
            preprocessor,
            sorter,
            full_splats: None,
        }
    }

    pub fn set_splats(&mut self, ctx: &Context, splats: &RawSplats) {
        let gpu_splats = GpuSplat::vec_from_raw(splats);
        let num_splats = gpu_splats.len();

        if num_splats == 0 {
            self.preprocessor.resize(&ctx.device, 0, 0);
            self.sorter.resize(&ctx.device, 0);
            self.full_splats = None;
            self.renderer.num_splats = 0;
            self.renderer.instance_buffer = None;
            return;
        }

        // Pad to next power of two for bitonic sorter
        let mut padded = 1;
        while padded < num_splats { padded <<= 1; }

        self.preprocessor.resize(&ctx.device, padded, num_splats);
        self.sorter.resize(&ctx.device, padded);

        // Upload splats to preprocessor
        self.preprocessor.upload_splats(&ctx.queue, &gpu_splats);

        // Create full GPU buffer for sorter
        let full_buf = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Full GpuSplats Buffer"),
            contents: bytemuck::cast_slice(&gpu_splats),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::VERTEX,
        });
        self.full_splats = Some(full_buf);

        self.preprocessor.set_camera_buffer(&ctx.device, self.renderer.camera_buffer());

        self.renderer.instance_buffer = None;
        self.renderer.num_splats = num_splats;
    }

    pub fn render(&mut self, ctx: &Context, camera: &Camera) {
        let view_proj = camera.build_view_projection_matrix();
        self.renderer.update_camera(&ctx.queue, &view_proj);

        let output = match ctx.surface.get_current_texture() {
            Ok(tex) => tex,
            Err(_) => return,
        };
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Reset on-GPU (zero visible counters & indirect args)
        self.preprocessor.run_reset_on_gpu(&mut encoder);

        // Run cull
        self.preprocessor.run(&mut encoder);

        if let Some(compacted) = self.preprocessor.compacted_buffer() {
            if let Some(full_buf) = &self.full_splats {
                self.sorter.set_input_from_preprocessor(&ctx.device, compacted, full_buf);
                self.sorter.run(&ctx.queue, &mut encoder);

                if let Some(sorted_buf) = self.sorter.output_buffer() {
                    self.renderer.set_instance_buffer(&ctx.device, sorted_buf);
                    self.renderer.num_splats = self.preprocessor.num_splats as usize;
                }
            }
        }

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Splatter Render Pass"),
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

            // If preprocessor produced an indirect buffer, use it. Otherwise fall back to previous draw call.
            if let Some(indirect) = self.preprocessor.indirect_buffer() {
                self.renderer.draw_indirect(&mut rpass, indirect);
            } else {
                self.renderer.draw(&mut rpass, self.renderer.num_splats as u32);
            }
        }

        ctx.queue.submit(Some(encoder.finish()));
        output.present();
    }
}
