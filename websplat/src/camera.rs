use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            eye: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            aspect,
            fovy: 45.0f32.to_radians(),
            znear: 0.1,
            zfar: 1000.0,
        }
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);
        proj * view
    }

    pub fn orbit(&mut self, delta: Vec2) {
        // vector from target to eye
        let offset = self.eye - self.target;
        let radius = offset.length();

        // spherical coords
        let mut theta = offset.z.atan2(offset.x); // azimuth
        let mut phi = (offset.y / radius).acos(); // polar angle

        // apply deltas (scale to taste)
        theta -= delta.x * 0.01;
        phi -= delta.y * 0.01;
        phi = phi.clamp(0.01, std::f32::consts::PI - 0.01);

        // convert back to Cartesian
        let new_offset = Vec3::new(
            radius * theta.cos() * phi.sin(),
            radius * phi.cos(),
            radius * theta.sin() * phi.sin(),
        );

        self.eye = self.target + new_offset;
    }

    pub fn zoom(&mut self, delta: f32) {
        let dir = (self.target - self.eye).normalize();
        let dist = (self.target - self.eye).length();

        // exponential zoom feels nicer
        let new_dist = (dist * (1.0 - delta * 0.1)).max(0.1);
        self.eye = self.target - dir * new_dist;
    }
}
