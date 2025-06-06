use burn::prelude::Config;

#[derive(Config, Debug)]
pub struct LoadConfig {
    /// Max nr. of frames of dataset to load
    pub max_frames: Option<usize>,
    /// Max resolution of images to load.
    #[config(default = 1920)]
    pub max_resolution: u32,
    /// Create an eval dataset by selecting every nth image
    pub eval_split_every: Option<usize>,
    /// Load only every nth frame
    pub subsample_frames: Option<u32>,
    /// Load only every nth point from the initial sfm data
    pub subsample_points: Option<u32>,
}