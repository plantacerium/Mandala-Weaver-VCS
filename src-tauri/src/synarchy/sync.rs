use super::registry::{ProjectEntry, ProjectScanner, ProjectStatus};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

pub struct Synchronizer {
    projects: Arc<RwLock<Vec<ProjectEntry>>>,
    running: Arc<RwLock<bool>>,
}

impl Synchronizer {
    pub fn new() -> Self {
        Self {
            projects: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }
    
    pub async fn start(&self) {
        *self.running.write().await = true;
        
        let mut interval = interval(Duration::from_secs(300));
        
        while *self.running.read().await {
            interval.tick().await;
            self.scan_all().await;
        }
    }
    
    pub async fn stop(&self) {
        *self.running.write().await = false;
    }
    
    pub async fn scan_all(&self) {
        let projects = self.projects.read().await;
        
        for project in projects.iter() {
            if let Err(e) = ProjectScanner::scan(&project.path).await {
                eprintln!("Scan error for {}: {}", project.name, e);
            }
        }
    }
    
    pub async fn register(&self, entry: ProjectEntry) {
        self.projects.write().await.push(entry);
    }
    
    pub async fn unregister(&self, id: &str) -> Option<ProjectEntry> {
        let mut projects = self.projects.write().await;
        if let Some(pos) = projects.iter().position(|p| p.id == id) {
            Some(projects.remove(pos))
        } else {
            None
        }
    }
    
    pub async fn get_all(&self) -> Vec<ProjectEntry> {
        self.projects.read().await.clone()
    }
    
    pub async fn update_status(&self, id: &str, status: ProjectStatus) {
        let mut projects = self.projects.write().await;
        if let Some(project) = projects.iter_mut().find(|p| p.id == id) {
            project.status = status;
        }
    }
}

impl Default for Synchronizer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProjectChangeEvent {
    pub project_id: String,
    pub project_name: String,
    pub change_type: ChangeType,
    pub monads_added: usize,
    pub rings_changed: Vec<u32>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum ChangeType {
    New,
    Modified,
    Removed,
    Rescan,
}