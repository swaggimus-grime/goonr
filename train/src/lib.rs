use burn_wgpu::WgpuDevice;

pub struct TrainContext {
    device: WgpuDevice
}

impl TrainContext {
    pub fn new() -> Self {
        let device = WgpuDevice::default();
        
        Self {
            device
        }
    }
    
    pub fn device(&self) -> &WgpuDevice {
        &self.device
    }
}

