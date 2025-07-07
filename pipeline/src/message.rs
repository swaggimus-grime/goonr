use std::time::Duration;
use glam::Vec3;
use render::gaussian_splats::Splats;
use render::MainBackend;
use serde::{Deserialize, Serialize};
use train::msg::{RefineStats, TrainStepStats};

#[derive(Debug)]
pub enum PipelineMessage {
    NewSource,
    StartLoading {
        training: bool,
    },
    ViewSplats {
        up_axis: Option<Vec3>,
        splats: Box<Splats<MainBackend>>,
        frame: u32,
        total_frames: u32,
    },
    /// Some number of training steps are done.
    #[allow(unused)]
    TrainStep {
        splats: Box<Splats<MainBackend>>,
        stats: Box<TrainStepStats<MainBackend>>,
        iter: u32,
        total_elapsed: Duration,
    },
    /// Some number of training steps are done.
    #[allow(unused)]
    RefineStep {
        stats: Box<RefineStats>,
        cur_splat_count: u32,
        iter: u32,
    },
    /// Eval was run successfully with these results.
    #[allow(unused)]
    EvalResult {
        iter: u32,
        avg_psnr: f32,
        avg_ssim: f32,
    },
    Finished
}