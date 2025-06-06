use std::sync::Arc;
pub(crate) use crate::scene::image::ImageFile;

mod image;
pub mod splat;

#[derive(Clone)]
pub struct SceneView {
    pub image: ImageFile,
    pub camera: renderer::Camera,
}

#[derive(Clone)]
pub struct Scene {
    pub views: Arc<Vec<SceneView>>,
}

impl Scene {
    pub fn new(views: Vec<SceneView>) -> Self {
        Self {
            views: Arc::new(views),
        }
    }
}