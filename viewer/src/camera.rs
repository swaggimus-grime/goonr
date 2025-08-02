use glam::{Mat4, Vec2, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraMode {
    Fly,
    Orbit,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

pub struct Camera {
    pub mode: CameraMode,

    // Shared
    pub aspect: f32,
    pub fov_y: f32,
    pub near: f32,
    pub far: f32,

    // Fly mode
    pub position: Vec3,
    pub yaw: f32,   // rotation around Y axis
    pub pitch: f32, // rotation around X axis

    // Orbit mode
    pub focus: Vec3,
    pub radius: f32,
    pub orbit_yaw: f32,
    pub orbit_pitch: f32,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            mode: CameraMode::Fly,
            aspect,
            fov_y: 45.0_f32.to_radians(),
            near: 0.1,
            far: 1000.0,

            // fly init
            position: Vec3::new(0.0, 0.0, 5.0),
            yaw: 0.0,
            pitch: 0.0,

            // orbit init
            focus: Vec3::ZERO,
            radius: 5.0,
            orbit_yaw: 0.0,
            orbit_pitch: 0.0,
        }
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        match self.mode {
            CameraMode::Fly => {
                let dir = self.get_fly_direction();
                return Mat4::look_to_rh(self.position, dir, Vec3::Y);
            }
            CameraMode::Orbit => {
                let (x, y, z) = (
                    self.orbit_pitch.cos() * self.orbit_yaw.sin(),
                    self.orbit_pitch.sin(),
                    self.orbit_pitch.cos() * self.orbit_yaw.cos(),
                );
                let eye = self.focus + Vec3::new(x, y, z) * self.radius;
                return Mat4::look_at_rh(eye, self.focus, Vec3::Y);
            }
        }
    }

    pub fn apply_input(&mut self, movement: Vec3, rotation: Vec2, dt: f32) {
        match self.mode {
            CameraMode::Fly => {
                let dir = self.get_fly_direction();
                let right = Vec3::Y.cross(dir).normalize();

                self.position += (dir * movement.z + right * movement.x + Vec3::Y * movement.y) * dt;

                let sensitivity = 1.0;
                self.yaw += rotation.x * sensitivity;
                self.pitch += rotation.y * sensitivity;
                self.pitch = self.pitch.clamp(-1.5, 1.5);
            }
            CameraMode::Orbit => {
                self.orbit_yaw -= rotation.x;
                self.orbit_pitch = (self.orbit_pitch - rotation.y).clamp(-1.5, 1.5);
                self.radius = (self.radius - movement.z).clamp(0.5, 50.0);
            }
        }
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_y, self.aspect, self.near, self.far)
    }

    pub fn get_view_proj(&self) -> Mat4 {
        self.get_projection_matrix() * self.get_view_matrix()
    }

    pub fn get_fly_direction(&self) -> Vec3 {
        Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        )
            .normalize()
    }
}

