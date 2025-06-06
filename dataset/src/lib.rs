use crate::scene::{Scene, SceneView};

mod config;
mod formats;
mod filesystem;
mod scene;
mod error;

pub use formats::load_dataset;
pub use config::LoadConfig;

#[derive(Clone)]
pub struct Dataset {
    pub train: Scene,
    pub eval: Option<Scene>,
}

impl Dataset {
    pub fn from_views(train_views: Vec<SceneView>, eval_views: Vec<SceneView>) -> Self {
        Self {
            train: Scene::new(train_views),
            eval: if eval_views.is_empty() {
                None
            } else {
                Some(Scene::new(eval_views))
            },
        }
    }
}
