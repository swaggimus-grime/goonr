use wgpu::StoreOp;
use web_cmn::splats::RawSplats;
use crate::camera::Camera;
use crate::context::Context;
use crate::renderer::Renderer;
use crate::splats::GpuSplat;

pub struct Splatter {
    renderer: Renderer,
}

impl Splatter {
    pub fn new(ctx: &Context) -> Splatter {
        let renderer = Renderer::new(&ctx.device);

        Self {
            renderer
        }
    }

    pub fn set_splats(&mut self, ctx: &Context, splats: &RawSplats) {
        let gpu_splats = GpuSplat::vec_from_raw(splats);
        self.renderer.replace_splats(&ctx.device, &ctx.queue, &gpu_splats);
    }

    pub fn render(&mut self, ctx: &Context, camera: &Camera) {
        let view_proj = camera.build_view_projection_matrix();
        self.renderer.update_camera(&ctx.queue, &view_proj);

        let output = ctx.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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

            self.renderer.draw(&mut rpass);
        }
        ctx.queue.submit(Some(encoder.finish()));
        output.present();
    }
}
