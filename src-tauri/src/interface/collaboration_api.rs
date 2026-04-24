use tauri::State;
use crate::persistence::surreal_bridge::Db;
use crate::collaboration::{self, MandalaDiff, MergeResult};
use surrealdb::Surreal;
use std::path::PathBuf;

#[tauri::command]
pub async fn export_mandala_archive(
    db: State<'_, Surreal<Db>>,
    project_name: String,
    output_path: String,
) -> Result<String, String> {
    let path = PathBuf::from(output_path);
    collaboration::export_mandala(&db, &project_name, &path).await
        .map(|p| p.display().to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_mandala_archive(
    db: State<'_, Surreal<Db>>,
    archive_path: String,
) -> Result<(), String> {
    let path = PathBuf::from(archive_path);
    collaboration::import_mandala(&db, &path).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn diff_mandala_states(
    db: State<'_, Surreal<Db>>,
    remote_path: String,
) -> Result<MandalaDiff, String> {
    let path = PathBuf::from(remote_path);
    collaboration::diff_mandala(&db, &path).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn merge_mandala_states(
    remote_path: String,
) -> Result<MergeResult, String> {
    let path = PathBuf::from(remote_path);
    let _db = crate::persistence::surreal_bridge::connect_embedded()
        .await
        .map_err(|e| e.to_string())?;
    collaboration::merge_mandala(&_db, &path).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_project(
    id: String,
    path: String,
    synchronizer: State<'_, crate::synarchy::sync::Synchronizer>,
) -> Result<(), String> {
    let path_buf = PathBuf::from(path);
    let entry = crate::synarchy::registry::ProjectEntry {
        id: id.clone(),
        name: id.clone(),
        path: path_buf,
        project_type: crate::synarchy::registry::ProjectType::Local,
        last_scanned: 0,
        ring_count: 0,
        monad_count: 0,
        status: crate::synarchy::registry::ProjectStatus::Active,
    };
    synchronizer.register(entry).await;
    Ok(())
}

#[tauri::command]
pub async fn get_sync_status(
    synchronizer: State<'_, crate::synarchy::sync::Synchronizer>,
) -> Result<Vec<crate::synarchy::registry::ProjectEntry>, String> {
    Ok(synchronizer.get_all().await)
}
