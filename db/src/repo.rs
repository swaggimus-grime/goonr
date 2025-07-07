use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::{Surreal};
use surrealdb::opt::IntoQuery;
use surrealdb::sql::Thing;
use tracing::error;
use scene_source::Source;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneMetadata {
    pub name: String,
    pub source: Source,
}

#[async_trait]
pub trait SplatRepository: Send + Sync {
    async fn add_scene(&self, scene: SceneMetadata) -> anyhow::Result<()>;
    async fn get_scene(&self, name: &str) -> anyhow::Result<Option<SceneMetadata>>;
    async fn list_scenes(&self) -> anyhow::Result<Vec<SceneMetadata>>;
    
    async fn can_add(&self, name: &str) -> bool;
}

const ROOT_NS: &'static str = "goonr";
const DB_NAME: &'static str = "goonr_db";
const TABLE_SCENE: &str = "scene";

pub struct SplatRepo {
    db: Surreal<Db>,
}

impl SplatRepo {
    pub async fn new() -> anyhow::Result<Self> {
        let db = Surreal::new::<RocksDb>("goonr.db").await?;
        db.use_ns(ROOT_NS).use_db(DB_NAME).await?;
        Ok(Self { db })
    }
}

#[async_trait]
impl SplatRepository for SplatRepo {
    async fn add_scene(&self, scene: SceneMetadata) -> anyhow::Result<()> {
        let _: Option<SceneMetadata> = self.db
            .create((TABLE_SCENE, scene.name.as_str()))
            .content(scene)
            .await?;
        Ok(())
    }

    async fn get_scene(&self, name: &str) -> anyhow::Result<Option<SceneMetadata>> {
        Ok(self.db.select((TABLE_SCENE, name)).await?)
    }

    async fn list_scenes(&self) -> anyhow::Result<Vec<SceneMetadata>> {
        Ok(self.db.select(TABLE_SCENE).await?)
    }
    
    async fn can_add(&self, name: &str) -> bool {
        !self.get_scene(name).await.unwrap().is_some()
    }
}