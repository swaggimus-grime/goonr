use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Point3D {
    pub xyz: glam::Vec3,
    pub rgb: [u8; 3],
    pub error: f64,
    pub image_ids: Vec<i32>,
    pub point2d_idxs: Vec<i32>,
}