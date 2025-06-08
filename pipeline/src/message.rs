use glam::Vec3;
use render::MainBackend;
use render::splat::Splats;

pub enum Message {
    NewSource,
    StartLoading {
        training: bool,
    },
    ViewSplats {
        up_axis: Option<Vec3>,
        splats: Box<Splats<MainBackend>>,
        frame: u32,
        total_frames: u32,
    }
}