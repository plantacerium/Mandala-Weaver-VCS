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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectType {
    Local,
    Remote,
    Git,
    Weaver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "message")]
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
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let id = blake3::hash(format!("{}_{}", name, path.display()).as_bytes())
            .to_hex()
            .to_string()[..8]
            .to_string();
        
        let monad_count = Self::count_source_files(path).await?;
        
        let mut ring_count: u32 = 0;
        
        let db_path = path.join(".mandala");
        if db_path.exists() {
            ring_count = 3;
        }
        
        Ok(ProjectEntry {
            id,
            name,
            path: path.clone(),
            project_type: ProjectType::Local,
            last_scanned: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ring_count,
            monad_count,
            status: ProjectStatus::Active,
        })
    }
    
    pub async fn scan_with_rings(path: &PathBuf, ring_count: u32) -> anyhow::Result<ProjectEntry> {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let id = blake3::hash(format!("{}_{}", name, path.display()).as_bytes())
            .to_hex()
            .to_string()[..8]
            .to_string();
        
        let monad_count = Self::count_source_files(path).await?;
        
        Ok(ProjectEntry {
            id,
            name,
            path: path.clone(),
            project_type: ProjectType::Local,
            last_scanned: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ring_count,
            monad_count,
            status: ProjectStatus::Active,
        })
    }
    
    async fn count_source_files(path: &PathBuf) -> anyhow::Result<usize> {
        let mut count = 0;
        let extensions = ["rs", "ts", "tsx", "js", "jsx", "py", "go", "java", "c", "cpp", "h"];
        
        Self::count_recursive(path, &extensions, &mut count)?;
        Ok(count)
    }
    
    fn count_recursive(path: &PathBuf, extensions: &[&str], count: &mut usize) -> anyhow::Result<()> {
        if !path.exists() {
            return Ok(());
        }
        
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if extensions.contains(&ext.to_str().unwrap_or("")) {
                        *count += 1;
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
                    Self::count_recursive(&path, extensions, count)?;
                }
            }
        }
        
        Ok(())
    }
}