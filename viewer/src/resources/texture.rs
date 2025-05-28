pub struct Texture {
    raw: wgpu::Texture
}

impl Texture {
    pub fn new(
        renderer: Arc<EguiRwLock<Renderer>>,
        device: wgpu::Device,
        queue: wgpu::Queue,
    ) -> Self {
        Self {
            state: None,
            device,
            queue,
            renderer,
        }
    }

    pub fn update_texture(&mut self, img: Tensor<MainBackend, 3>) -> TextureId {

    }
}
