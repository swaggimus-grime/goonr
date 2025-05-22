// src/components/viewer_canvas.rs
use yew::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlCanvasElement;
use wgpu::{InstanceDescriptor, SurfaceTarget};

#[function_component(ViewerCanvas)]
pub fn viewer_canvas() -> Html {
    use_effect(|| {
        spawn_local(async {
            #[cfg(target_arch = "wasm32")]
            {
                use gloo::utils::document;
                use wgpu::StoreOp;

                let canvas = document()
                    .get_element_by_id("viewer-canvas")
                    .unwrap()
                    .dyn_into::<HtmlCanvasElement>()
                    .unwrap();

                // Create instance
                let instance = wgpu::Instance::new(&InstanceDescriptor {
                    backends: wgpu::Backends::all(),
                    ..Default::default()
                });

                // Create surface
                let surface = instance
                    .create_surface(SurfaceTarget::Canvas(canvas.clone()))
                    .expect("Failed to create surface");

                // Request adapter
                let adapter = instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::HighPerformance,
                        compatible_surface: Some(&surface),
                        force_fallback_adapter: false,
                    })
                    .await
                    .expect("Failed to find an appropriate adapter");

                // Request device + queue
                let (device, queue) = adapter
                    .request_device(
                        &wgpu::DeviceDescriptor {
                            label: None,
                            required_features: Default::default(),
                            required_limits: Default::default(),
                            memory_hints: Default::default(),
                            trace: Default::default(),
                        },
                    )
                    .await
                    .expect("Failed to create device");

                // Configure surface
                let config = wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: surface.get_capabilities(&adapter).formats[0],
                    width: canvas.width(),
                    height: canvas.height(),
                    present_mode: wgpu::PresentMode::Fifo,
                    desired_maximum_frame_latency: 1,
                    alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    view_formats: vec![],
                };
                surface.configure(&device, &config);

                // Get current frame
                let frame = surface
                    .get_current_texture()
                    .expect("Timeout getting texture");

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // Render pass
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Clear Encoder"),
                });

                {
                    let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.1,
                                    b: 0.1,
                                    a: 1.0,
                                }),
                                store: StoreOp::Store, // fix: Store so we can present the image
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                }

                // Submit
                queue.submit(Some(encoder.finish()));
                frame.present();
            }
        });

        || ()
    });

    html! {
        <div class="flex h-screen w-screen bg-gray-950 text-white font-sans">
            // Sidebar
            <div class="w-64 bg-gray-900 p-4 border-r border-gray-800">
                <h1 class="text-xl font-bold mb-6">{"Goonr Viewer"}</h1>

                <div class="space-y-4">
                    <div>
                        <label class="block text-sm mb-1 text-gray-400">{"Load Scene"}</label>
                        <button class="w-full bg-blue-600 hover:bg-blue-700 text-white py-2 px-3 rounded">
                            {"Upload"}
                        </button>
                    </div>

                    <div>
                        <label class="block text-sm mb-1 text-gray-400">{"Camera"}</label>
                        <select class="w-full bg-gray-800 text-white p-2 rounded">
                            <option>{"Orbit"}</option>
                            <option>{"Free Look"}</option>
                        </select>
                    </div>

                    <div>
                        <label class="block text-sm mb-1 text-gray-400">{"Point Size"}</label>
                        <input type="range" min="0.1" max="5.0" step="0.1"
                               class="w-full accent-blue-500" />
                    </div>
                </div>
            </div>

            // Main viewer/canvas area
            <div class="flex-1 relative">
                <canvas
                    id="viewer-canvas"
                    class="absolute top-0 left-0 w-full h-full"
                    width="1280"
                    height="720"
                ></canvas>
            </div>
        </div>
    }
}
