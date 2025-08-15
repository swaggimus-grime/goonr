use std::ops::Deref;
use wgpu::{Adapter, CompositeAlphaMode, Device, Instance, TextureFormat};
use wgpu::wgt::SurfaceConfiguration;

pub struct Surface<'window> {
    surface: wgpu::Surface<'window>,
    config: wgpu::SurfaceConfiguration,
}

impl<'window> Surface<'window> {
    pub fn from_canvas(instance: &Instance, canvas: web_sys::HtmlCanvasElement) -> Self {
        let (width, height) = (canvas.width(), canvas.height());

        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
            .expect("Failed to create surface");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bc3RgbaUnorm,
            width: width,
            height: height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 1,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        
        Self {
            surface,
            config
        }
    }
    
    pub fn width(&self) -> u32 {
        self.config.width
    }
    
    pub fn height(&self) -> u32 {
        self.config.height
    }
    
    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }
    
    pub fn resize(&mut self, device: &Device, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(device, &self.config);
    }
    
    pub fn reconfigure(&mut self, device: &wgpu::Device, config: wgpu::SurfaceConfiguration) {
        self.config = config;
        self.surface.configure(device, &self.config);
    }
}

impl<'window> Deref for Surface<'window> {
    type Target = wgpu::Surface<'window>;

    fn deref(&self) -> &Self::Target {
        &self.surface
    }
}