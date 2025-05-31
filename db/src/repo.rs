use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use surrealdb::engine::local::{Db, Mem};
use surrealdb::Surreal;
use web_cmn::responses::scene::SceneMetadata;

#[async_trait::async_trait]
pub trait SplatRepository: Send + Sync {
    async fn add_scene(&self, scene: SceneMetadata) -> anyhow::Result<()>;
    async fn get_scene(&self, id: &str) -> anyhow::Result<Option<SceneMetadata>>;
    async fn list_scenes(&self) -> anyhow::Result<Vec<SceneMetadata>>;
}

const ROOT_NS: &'static str = "goonr";
const SCENE_COL: &'static str = "scene";

pub struct SplatRepo {
    db: Surreal<Db>,
}

impl SplatRepo {
    pub async fn new() -> anyhow::Result<Self> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns(ROOT_NS).use_db(SCENE_COL).await?;
        Ok(Self { db })
    }
}

#[async_trait]
impl SplatRepository for SplatRepo {
    async fn add_scene(&self, scene: SceneMetadata) -> anyhow::Result<()> {
        let _: Option<SceneMetadata> = self.db.create((SCENE_COL, scene.id.to_string()))
            .content(scene)
            .await?;
        Ok(())
    }

    async fn get_scene(&self, id: &str) -> anyhow::Result<Option<SceneMetadata>> {
        Ok(self.db.select((SCENE_COL, id)).await?)
    }

    async fn list_scenes(&self) -> anyhow::Result<Vec<SceneMetadata>> {
        Ok(self.db.select(SCENE_COL).await?)
    }
}