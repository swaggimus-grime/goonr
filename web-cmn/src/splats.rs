use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RawSplats {
    pub means: Vec<f32>,
    pub rotation: Vec<f32>,
    pub log_scales: Vec<f32>,
    pub sh_coeffs: Vec<f32>,
    pub sh_coeffs_dims: [usize; 3],
    pub raw_opacity: Vec<f32>,
}
