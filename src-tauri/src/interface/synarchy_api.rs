use crate::synarchy::{ProjectEntry, ProjectRegistry, ProjectScanner};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

pub struct SynarchyState {
    pub registry: Arc<RwLock<ProjectRegistry>>,
    pub registry_path: PathBuf,
    pub plugin_config: Arc<RwLock<crate::plugins::PluginConfig>>,
}

impl SynarchyState {
    pub fn new(data_dir: PathBuf) -> Self {
        let registry_path = data_dir.join("synarchy_registry.json");
        
        let registry = if registry_path.exists() {
            ProjectRegistry::load(&registry_path).unwrap_or_else(|_| ProjectRegistry::new())
        } else {
            ProjectRegistry::new()
        };
        
        Self {
            registry: Arc::new(RwLock::new(registry)),
            registry_path,
            plugin_config: Arc::new(RwLock::new(crate::plugins::PluginConfig::default())),
        }
    }
    
    pub async fn save(&self) -> Result<(), String> {
        let registry = self.registry.read().await;
        registry.save(&self.registry_path)
            .map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn get_projects(state: State<'_, SynarchyState>) -> Result<Vec<ProjectEntry>, String> {
    let registry = state.registry.read().await;
    Ok(registry.projects.clone())
}

#[tauri::command]
pub async fn add_project(
    path: String,
    name: String,
    state: State<'_, SynarchyState>,
) -> Result<ProjectEntry, String> {
    let path_buf = PathBuf::from(&path);
    
    if !path_buf.exists() {
        return Err("Path does not exist".to_string());
    }
    
    let entry = ProjectScanner::scan(&path_buf)
        .await
        .map_err(|e| e.to_string())?;
    
    let mut final_entry = entry;
    if !name.is_empty() {
        final_entry.name = name;
    }
    
    let mut registry = state.registry.write().await;
    
    if registry.find(&final_entry.id).is_some() {
        return Err("Project already exists".to_string());
    }
    
    registry.add(final_entry.clone());
    drop(registry);
    
    state.save().await?;
    
    Ok(final_entry)
}

#[tauri::command]
pub async fn remove_project(id: String, state: State<'_, SynarchyState>) -> Result<(), String> {
    let mut registry = state.registry.write().await;
    
    if registry.remove(&id).is_none() {
        return Err("Project not found".to_string());
    }
    
    drop(registry);
    state.save().await
}

#[tauri::command]
pub async fn get_project_detail(
    id: String,
    state: State<'_, SynarchyState>,
) -> Result<ProjectEntry, String> {
    let registry = state.registry.read().await;
    
    registry.find(&id)
        .cloned()
        .ok_or_else(|| "Project not found".to_string())
}

#[tauri::command]
pub async fn rescan_project(
    id: String,
    state: State<'_, SynarchyState>,
) -> Result<ProjectEntry, String> {
    let (path, name) = {
        let registry = state.registry.read().await;
        let project = registry.find(&id)
            .ok_or_else(|| "Project not found".to_string())?;
        (project.path.clone(), project.name.clone())
    };
    
    let mut new_entry = ProjectScanner::scan(&path)
        .await
        .map_err(|e| e.to_string())?;
    new_entry.name = name;
    
    {
        let mut registry = state.registry.write().await;
        if let Some(project) = registry.find_mut(&id) {
            *project = new_entry.clone();
        }
    }
    
    state.save().await?;
    
    Ok(new_entry)
}

#[tauri::command]
pub async fn get_project_mandala(
    id: String,
    state: State<'_, SynarchyState>,
) -> Result<serde_json::Value, String> {
    let registry = state.registry.read().await;
    
    let project = registry.find(&id)
        .ok_or_else(|| "Project not found".to_string())?;
    
    let db = crate::persistence::surreal_bridge::connect_db(Some(&project.path))
        .await
        .map_err(|e| e.to_string())?;

    let all_monads = crate::persistence::surreal_bridge::get_all_monads(&db).await.unwrap_or_default();
    let edges = crate::persistence::surreal_bridge::get_all_edges(&db).await.unwrap_or_default();
    let max_ring = all_monads.iter().map(|m| m.ring).max().unwrap_or(0);

    let mut rings_map: std::collections::HashMap<u32, Vec<crate::ontology::monad::Monad>> = std::collections::HashMap::new();
    for m in all_monads {
        rings_map.entry(m.ring).or_default().push(m);
    }

    let mut constellations = rings_map.into_iter()
        .map(|(ring_level, monads)| serde_json::json!({ "ring_level": ring_level, "monads": monads }))
        .collect::<Vec<_>>();
    constellations.sort_by_key(|c| c["ring_level"].as_u64().unwrap_or(0));

    let total: usize = constellations.iter().map(|c| c["monads"].as_array().map(|a| a.len()).unwrap_or(0)).sum();

    Ok(serde_json::json!({
        "bindu_name": project.name,
        "constellations": constellations,
        "edges": edges,
        "total_monads": total,
        "max_ring": max_ring
    }))
}