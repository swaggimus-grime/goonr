use burn::serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Image {
    pub tvec: glam::Vec3,
    pub quat: glam::Quat,
    pub camera_id: i32,
    pub name: String,
    pub xys: Vec<glam::Vec2>,
    pub point3d_ids: Vec<i64>,
}