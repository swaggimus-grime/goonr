mod image;
mod camera;
mod point3D;
mod colmap;

use std::collections::HashMap;
use std::io;
use std::path::Path;
use serde::Serialize;
use crate::camera::Camera;
use crate::colmap::ColmapDir;
use crate::colmap::input::InputType;
use crate::image::Image;
use crate::point3D::Point3D;

#[derive(Debug, Clone, Serialize)]
pub struct Scene {
    points: HashMap<i64, Point3D>,
    images: HashMap<i32, Image>,
    cameras: HashMap<i32, Camera>,
}

impl Scene {
    pub async fn new(scene_dir: &Path) -> io::Result<Scene> {
        let colmap = ColmapDir::new(scene_dir).await?;

        Ok(Scene {
            points: colmap.query(InputType::Points3D).await?.as_points().unwrap(),
            images: colmap.query(InputType::Images).await?.as_images().unwrap(),
            cameras: colmap.query(InputType::Cameras).await?.as_cameras().unwrap(),
        })
    }

    pub fn points(&self) -> &HashMap<i64, Point3D> {
        &self.points
    }
}