use glam::Vec3;
use render::gaussian_splats::Splats;
use render::MainBackend;

pub struct ParseMetadata {
    pub up_axis: Option<Vec3>,
    pub total_splats: u32,
    pub frame_count: u32,
    pub current_frame: u32,
}

pub struct SplatMessage {
    pub meta: ParseMetadata,
    pub splats: Splats<MainBackend>,
}