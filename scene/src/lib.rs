

#[derive(Debug, Clone, Serialize)]
pub struct Scene {
    points: HashMap<i64, Point3D>,
    images: HashMap<i32, Image>,
    cameras: HashMap<i32, Camera>,
}

impl Scene {
    pub async fn new(scene_dir: &Path) -> io::Result<Scene> {
        let colmap = ColmapDir::new(colmap_loc).await?;

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