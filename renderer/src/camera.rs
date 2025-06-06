use glam::{Vec2, Vec3, Quat};

#[derive(Debug, Default, Clone)]
pub struct Camera {
    pub fov_x: f64,
    pub fov_y: f64,
    pub center_uv: Vec2,
    pub position: Vec3,
    pub rotation: Quat,
}

impl Camera {
    pub fn new(
        position: Vec3,
        rotation: Quat,
        fov_x: f64,
        fov_y: f64,
        center_uv: Vec2,
    ) -> Self {
        Self {
            fov_x,
            fov_y,
            center_uv,
            position,
            rotation,
        }
    }
}

// Converts field of view to focal length
pub fn fov_to_focal(fov_rad: f64, pixels: u32) -> f64 {
    0.5 * (pixels as f64) / (fov_rad * 0.5).tan()
}

// Converts focal length to field of view
pub fn focal_to_fov(focal: f64, pixels: u32) -> f64 {
    2.0 * f64::atan((pixels as f64) / (2.0 * focal))
}