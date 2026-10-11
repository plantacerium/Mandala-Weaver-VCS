use std::path::PathBuf;
use surrealdb::Surreal;
use crate::persistence::surreal_bridge::{Db, connect_db, find_project_root};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DbEngine {
    Mem,
    File(PathBuf),
}

pub struct FileBridge {
    project_root: PathBuf,
    db: Surreal<Db>,
}

impl FileBridge {
    /// Connects to persistent SurrealKV storage in `.mandala/db/`.
    pub async fn connect(project_root: &PathBuf) -> anyhow::Result<Self> {
        let db_dir = project_root.join(".mandala").join("db");
        std::fs::create_dir_all(&db_dir)?;

        let templates_dir = project_root.join(".mandala").join("templates");
        std::fs::create_dir_all(&templates_dir)?;

        let db = connect_db(Some(project_root)).await?;

        Ok(Self {
            project_root: project_root.clone(),
            db,
        })
    }

    /// Auto-detects project root by walking up from cwd.
    pub fn auto_detect() -> anyhow::Result<PathBuf> {
        find_project_root().ok_or_else(|| {
            anyhow::anyhow!("No `.mandala/` directory found. Initialize with `weave bindu` first.")
        })
    }

    /// Connects automatically to auto-detected project root.
    pub async fn connect_auto() -> anyhow::Result<Self> {
        let root = Self::auto_detect()?;
        Self::connect(&root).await
    }

    pub fn db(&self) -> &Surreal<Db> {
        &self.db
    }

    pub fn project_root(&self) -> &PathBuf {
        &self.project_root
    }
}
