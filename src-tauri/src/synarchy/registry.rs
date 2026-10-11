use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEntry {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub project_type: ProjectType,
    pub last_scanned: u64,
    pub ring_count: u32,
    pub monad_count: usize,
    pub status: ProjectStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    Local,
    Remote,
    Git,
    Weaver,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectStatus {
    Active,
    Dormant,
    Scanning,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectRegistry {
    pub projects: Vec<ProjectEntry>,
}

impl ProjectRegistry {
    pub fn new() -> Self {
        Self { projects: Vec::new() }
    }
    
    pub fn add(&mut self, entry: ProjectEntry) {
        self.projects.push(entry);
    }
    
    pub fn remove(&mut self, id: &str) -> Option<ProjectEntry> {
        if let Some(pos) = self.projects.iter().position(|p| p.id == id) {
            Some(self.projects.remove(pos))
        } else {
            None
        }
    }
    
    pub fn find(&self, id: &str) -> Option<&ProjectEntry> {
        self.projects.iter().find(|p| p.id == id)
    }

    pub fn find_mut(&mut self, id: &str) -> Option<&mut ProjectEntry> {
        self.projects.iter_mut().find(|p| p.id == id)
    }
    
    
    pub fn active_projects(&self) -> Vec<&ProjectEntry> {
        self.projects.iter()
            .filter(|p| matches!(p.status, ProjectStatus::Active))
            .collect()
    }
    
    pub fn save(&self, path: &PathBuf) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    pub fn load(path: &PathBuf) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let registry = serde_json::from_str(&content)?;
        Ok(registry)
    }
}

pub struct ProjectScanner;

impl ProjectScanner {
    pub async fn scan(path: &PathBuf) -> anyhow::Result<ProjectEntry> {
        Self::scan_with_mode(path, None).await
    }

    pub async fn scan_with_mode(path: &PathBuf, mode: Option<&str>) -> anyhow::Result<ProjectEntry> {
        let db = crate::persistence::surreal_bridge::connect_db(Some(path)).await?;
        Self::scan_and_index_with_db_mode(&db, path, mode).await
    }

    pub async fn scan_and_index(path: &PathBuf) -> anyhow::Result<ProjectEntry> {
        Self::scan_with_mode(path, None).await
    }

    pub async fn scan_and_index_with_db(
        db: &surrealdb::Surreal<crate::persistence::surreal_bridge::Db>,
        path: &PathBuf,
    ) -> anyhow::Result<ProjectEntry> {
        Self::scan_and_index_with_db_mode(db, path, None).await
    }

    pub async fn scan_and_index_with_db_mode(
        db: &surrealdb::Surreal<crate::persistence::surreal_bridge::Db>,
        path: &PathBuf,
        scan_mode: Option<&str>,
    ) -> anyhow::Result<ProjectEntry> {
        let canonical = path.canonicalize().unwrap_or(path.clone());
        let name = canonical.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Mandala-Weaver-VCS")
            .to_string();
        
        let id = blake3::hash(format!("{}_{}", name, path.display()).as_bytes())
            .to_hex()
            .to_string()[..8]
            .to_string();

        let is_git = path.join(".git").exists();
        let mandala_dir = path.join(".mandala");
        let has_mandala = mandala_dir.exists();

        let project_type = match scan_mode {
            Some("git") => {
                if !is_git {
                    anyhow::bail!("No se encontró un repositorio Git (.git) en: {}. Selecciona 'Mandala Scan' para escanearlo de forma nativa sin Git.", path.display());
                }
                ProjectType::Git
            }
            Some("mandala") => ProjectType::Weaver,
            _ => {
                if is_git {
                    ProjectType::Git
                } else if has_mandala {
                    ProjectType::Weaver
                } else {
                    ProjectType::Weaver
                }
            }
        };

        let mut all_monads = crate::persistence::surreal_bridge::get_all_monads(db).await.unwrap_or_default();

        if all_monads.is_empty() {
            let bindu = crate::ontology::bindu::Bindu::genesis(&name);
            let val = serde_json::to_value(&bindu)?;
            let _: Option<serde_json::Value> = db.create(("bindu", "genesis")).content(val).await?;

            let mut files = Vec::new();
            let extensions = [
                "rs", "ts", "tsx", "js", "jsx", "py", "go",
                "java", "kt", "c", "cpp", "cc", "cxx", "h", "hpp", "cs",
                "html", "css", "scss", "vue", "svelte", "astro",
                "json", "yaml", "yml", "toml", "md", "txt", "sh", "sql"
            ];
            Self::collect_files_recursive(path, &extensions, &mut files, 0)?;

            files.sort_by_key(|(depth, p)| (*depth, p.clone()));

            let mut ring_counters: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();

            for (depth, file_path) in files.iter().take(150) {
                if let Ok(content) = std::fs::read_to_string(file_path) {
                    let ring = ((*depth as u32) + 1).min(10);
                    let lang = crate::language::detect_language(file_path);
                    let monads = crate::weaver::ast_extractor::extract_raw_monads_lang(
                        &content,
                        ring,
                        lang.tree_sitter_name(),
                    );
                    if monads.is_empty() && !content.trim().is_empty() {
                        let filename = file_path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("module");
                        let hash = crate::ontology::semantic_hash::generate_pure_hash(&content);
                        let count = ring_counters.entry(ring).or_insert(0);
                        let angle = ((*count as f64) * 137.50776405) % 360.0;
                        *count += 1;
                        let m = crate::ontology::monad::Monad::spawn_typed(
                            hash,
                            filename.to_string(),
                            crate::geometry::polar_space::PolarCoord::new((ring as f64) * 100.0, angle),
                            content.clone(),
                            ring,
                            crate::ontology::monad::MonadKind::Module,
                            1,
                            content.lines().count() as u32,
                            lang.tree_sitter_name(),
                        );
                        let _ = crate::persistence::surreal_bridge::insert_and_link(db, &m, None).await;
                    } else {
                        for mut m in monads {
                            let count = ring_counters.entry(ring).or_insert(0);
                            m.coord.theta = ((*count as f64) * 137.50776405) % 360.0;
                            *count += 1;
                            let _ = crate::persistence::surreal_bridge::insert_and_link(db, &m, None).await;
                        }
                    }
                }
            }
            all_monads = crate::persistence::surreal_bridge::get_all_monads(db).await.unwrap_or_default();
        }

        let monad_count = all_monads.len();
        let ring_count = all_monads.iter().map(|m| m.ring).max().unwrap_or(0);

        Ok(ProjectEntry {
            id,
            name,
            path: path.clone(),
            project_type,
            last_scanned: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            ring_count,
            monad_count,
            status: ProjectStatus::Active,
        })
    }

    pub async fn scan_with_rings(path: &PathBuf, ring_count: u32) -> anyhow::Result<ProjectEntry> {
        let mut entry = Self::scan(path).await?;
        entry.ring_count = ring_count;
        Ok(entry)
    }

    fn collect_files_recursive(
        current: &PathBuf,
        extensions: &[&str],
        collected: &mut Vec<(usize, PathBuf)>,
        depth: usize,
    ) -> anyhow::Result<()> {
        if !current.exists() || depth > 8 {
            return Ok(());
        }

        for entry in std::fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if extensions.contains(&ext.to_str().unwrap_or("")) {
                        collected.push((depth, path));
                    }
                }
            } else if path.is_dir() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                if !name.starts_with('.')
                    && name != "target"
                    && name != "node_modules"
                    && name != "__pycache__"
                    && name != "dist"
                    && name != "build"
                    && name != ".git"
                {
                    Self::collect_files_recursive(&path, extensions, collected, depth + 1)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scan_mandala_project() {
        let root = PathBuf::from("..");
        if root.join(".git").exists() {
            let db = crate::persistence::surreal_bridge::connect_embedded().await.unwrap();
            let entry = ProjectScanner::scan_and_index_with_db(&db, &root).await.unwrap();
            assert!(entry.monad_count > 0);
            assert_eq!(entry.project_type, ProjectType::Git);
            println!("✅ Scanned project: {} with {} monads and {} rings", entry.name, entry.monad_count, entry.ring_count);
        }
    }
}