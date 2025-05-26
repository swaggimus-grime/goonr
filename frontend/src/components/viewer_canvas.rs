use std::rc::Rc;
use glam::{Mat4, Vec3};
use yew::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlCanvasElement;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    mvp: [[f32; 4]; 4],
}

impl Uniforms {
    fn new() -> Self {
        Self {
            mvp: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    fn update(&mut self, mvp: Mat4) {
        self.mvp = mvp.to_cols_array_2d();
    }
}

impl Vertex {
    const fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct ViewerCanvasProps {
    pub scene_id: String,
}

#[function_component(ViewerCanvas)]
pub fn viewer_canvas(props: &ViewerCanvasProps) -> Html {
    let id = Rc::new(props.scene_id.clone());
    let canvas_ref = use_node_ref();

    let id_effect = id.clone();
    let canvas_ref_effect = canvas_ref.clone();

    use_effect_with(id_effect, move |id| {
        let id = id.clone();
        let canvas_ref = canvas_ref_effect.clone();

        spawn_local(async move {
            #[cfg(target_arch = "wasm32")]
            {
                use gloo::utils::window;
                use wgpu::StoreOp;

                let canvas = canvas_ref
                    .cast::<HtmlCanvasElement>()
                    .expect("Canvas element not found");

                let width = canvas.client_width() as u32;
                let height = canvas.client_height() as u32;
                canvas.set_width(width);
                canvas.set_height(height);

                let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                    backends: wgpu::Backends::all(),
                    ..Default::default()
                });

                let surface = instance
                    .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
                    .expect("Failed to create surface");

                let adapter = instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::HighPerformance,
                        compatible_surface: Some(&surface),
                        force_fallback_adapter: false,
                    })
                    .await
                    .expect("Failed to find adapter");

                let (device, queue) = adapter
                    .request_device(&wgpu::DeviceDescriptor::default())
                    .await
                    .expect("Failed to get device");

                let config = wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: surface.get_capabilities(&adapter).formats[0],
                    width,
                    height,
                    present_mode: wgpu::PresentMode::Fifo,
                    desired_maximum_frame_latency: 1,
                    alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    view_formats: vec![],
                };
                surface.configure(&device, &config);

                let response = gloo_net::http::Request::get(&format!("/api/pointcloud/{}", id))
                    .send()
                    .await
                    .expect("Failed to fetch point cloud");

                let buffer = response.binary().await.expect("Invalid binary data");
                let float_data = bytemuck::cast_slice::<u8, f32>(&buffer);
                let vertices: &[Vertex] = bytemuck::cast_slice(float_data);

                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

                let mut uniforms = Uniforms::new();
                let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 3.0), Vec3::ZERO, Vec3::Y);
                let proj = Mat4::perspective_rh_gl(45.0f32.to_radians(), width as f32 / height as f32, 0.1, 100.0);
                let mvp = proj * view * Mat4::IDENTITY;
                uniforms.update(mvp);

                let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Uniform Buffer"),
                    contents: bytemuck::cast_slice(&[uniforms]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

                let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Uniform Bind Group Layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

                let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Uniform Bind Group"),
                    layout: &uniform_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buffer.as_entire_binding(),
                    }],
                });

                let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Shader"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/triangle.wgsl").into()),
                });

                let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Pipeline Layout"),
                    bind_group_layouts: &[&uniform_bind_group_layout],
                    push_constant_ranges: &[],
                });

                let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_main"),
                        compilation_options: Default::default(),
                        buffers: &[Vertex::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_main"),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::PointList,
                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: Default::default(),
                    multiview: None,
                    cache: None,
                });

                surface.configure(&device, &config);
                let frame = surface
                    .get_current_texture()
                    .expect("Timeout getting texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
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

                    render_pass.set_pipeline(&pipeline);
                    render_pass.set_bind_group(0, &uniform_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                    render_pass.draw(0..vertices.len() as u32, 0..1);
                }

                queue.submit(Some(encoder.finish()));
                frame.present();
            }
        });

        || ()
    });

    html! {
        <canvas
            ref={canvas_ref}
            class="absolute top-0 left-0 w-full h-full"
        />
    }
}
