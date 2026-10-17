use tauri::State;
use serde::Serialize;
use crate::persistence::surreal_bridge::{Db, get_all_monads, get_all_edges, EdgeDto};
use crate::ontology::monad::Monad;
use crate::ontology::bindu::Bindu;
use crate::weaver::source_compiler::{distill_source, validate_source_coherence};
use crate::weaver::threader;
use surrealdb::Surreal;
use serde_json::Value as JsonValue;

#[derive(Serialize)]
pub struct MandalaState {
    pub bindu_name: String,
    pub constellations: Vec<ConstellationDto>,
    pub edges: Vec<EdgeDto>,
    pub total_monads: usize,
    pub max_ring: u32,
}

#[derive(Serialize)]
pub struct ConstellationDto {
    pub ring_level: u32,
    pub monads: Vec<Monad>,
}

#[derive(Serialize)]
pub struct DistillResult {
    pub source: String,
    pub monad_count: usize,
    pub ring_count: usize,
}

#[derive(Serialize)]
pub struct LineageResult {
    pub monads: Vec<Monad>,
    pub depth: usize,
}

async fn get_bindu_name(db: &Surreal<Db>) -> String {
    let result: Result<Vec<JsonValue>, _> = db.select("bindu").await;
    match result {
        Ok(bindus) if !bindus.is_empty() => {
            if let Some(name) = bindus[0].get("project_name").and_then(|v| v.as_str()) {
                if !name.is_empty() && name != "unknown" && name != "Unnamed_Project" {
                    return name.to_string();
                }
            }
            "Mandala Weaver VCS".to_string()
        }
        _ => "Mandala Weaver VCS".to_string(),
    }
}

/// Exports the complete spatial state of the Mandala as JSON.
/// Returns: { bindu_name, constellations, edges, total_monads, max_ring }
#[tauri::command]
pub async fn export_mandala_state(db: State<'_, crate::persistence::surreal_bridge::SharedDb>) -> Result<String, String> {
    let db_conn = db.read().await;
    let mut all_monads: Vec<Monad> = get_all_monads(&*db_conn)
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(|m| !m.is_archived && m.name != "mandala_state.json")
        .collect();

    // Auto-heal legacy or skewed ring distributions (e.g. >5 watcher rings or single-ring Git dumps)
    if crate::synarchy::ProjectScanner::needs_rebalance(&all_monads) {
        for m in &mut all_monads {
            m.ring = crate::synarchy::ProjectScanner::classify_monad_ring(m);
        }
    }

    // Apply coupled Golden Ratio annular bounds + Vogel Sunflower Phyllotaxis (r_n, θ_n)
    crate::synarchy::ProjectScanner::apply_golden_vogel_layout(&mut all_monads, 320.0);

    let mut edges = get_all_edges(&*db_conn).await.unwrap_or_default();

    // Synthesize cross-ring evolution and outbound-call edges if DB edges are empty
    if edges.is_empty() && !all_monads.is_empty() {
        let mut seen = std::collections::HashSet::new();
        let mut sorted = all_monads.clone();
        sorted.sort_by_key(|m| m.ring);
        let mut by_name: std::collections::HashMap<String, String> = std::collections::HashMap::new();

        for m in &sorted {
            if let Some(prev_id) = by_name.get(&m.name) {
                if prev_id != &m.id && seen.insert((prev_id.clone(), m.id.clone())) {
                    edges.push(EdgeDto {
                        parent_id: prev_id.clone(),
                        child_id: m.id.clone(),
                    });
                }
            }
            for call in &m.outbound_calls {
                if let Some(target_id) = by_name.get(call) {
                    if target_id != &m.id && seen.insert((target_id.clone(), m.id.clone())) {
                        edges.push(EdgeDto {
                            parent_id: target_id.clone(),
                            child_id: m.id.clone(),
                        });
                    }
                }
            }
            by_name.insert(m.name.clone(), m.id.clone());
        }
    }

    let bindu_name = get_bindu_name(&*db_conn).await;
    let total_monads = all_monads.len();
    let max_ring = all_monads.iter().map(|m| m.ring).max().unwrap_or(0);

    // Group by rings (preserving deterministic Vogel order within each ring)
    let mut rings_map: std::collections::BTreeMap<u32, Vec<Monad>> = std::collections::BTreeMap::new();
    for m in all_monads {
        rings_map.entry(m.ring).or_default().push(m);
    }

    let constellations: Vec<ConstellationDto> = rings_map
        .into_iter()
        .map(|(ring_level, monads)| ConstellationDto { ring_level, monads })
        .collect();

    let state = MandalaState {
        bindu_name,
        constellations,
        edges,
        total_monads,
        max_ring,
    };

    serde_json::to_string(&state)
        .map_err(|e| e.to_string())
}

/// Expands a new ring from a source file.
#[tauri::command]
pub async fn expand_ring(
    db: State<'_, crate::persistence::surreal_bridge::SharedDb>, 
    file_path: String
) -> Result<u32, String> {
    let db_conn = db.read().await;
    crate::weaver::expand_from_source(&*db_conn, &file_path).await
        .map_err(|e| e.to_string())
}

/// Initializes a new project by creating the Bindu (center point).
#[tauri::command]
pub async fn init_project(
    db: State<'_, crate::persistence::surreal_bridge::SharedDb>,
    active_root: State<'_, crate::persistence::surreal_bridge::ActiveProjectState>,
    watcher: State<'_, crate::weaver::watcher::SharedWatcher>,
    project_name: String,
    project_path: Option<String>,
) -> Result<String, String> {
    if let Some(path_str) = &project_path {
        let path = std::path::PathBuf::from(path_str);
        let new_db = crate::persistence::surreal_bridge::connect_db(Some(&path))
            .await
            .map_err(|e| e.to_string())?;
        
        let mut db_write = db.write().await;
        *db_write = new_db;

        let mut current_write = active_root.write().await;
        *current_write = Some(path.clone());

        watcher.watch_directory(path);
    }

    let db_conn = db.read().await;
    let bindu = Bindu::genesis(&project_name);
    let json_value = serde_json::to_value(&bindu)
        .map_err(|e| e.to_string())?;

    let _: Option<serde_json::Value> = db_conn.upsert(("bindu", "genesis"))
        .content(json_value)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("Project '{}' initialized at the Bindu.", project_name))
}

#[tauri::command]
pub async fn open_project(
    db: State<'_, crate::persistence::surreal_bridge::SharedDb>,
    active_root: State<'_, crate::persistence::surreal_bridge::ActiveProjectState>,
    watcher: State<'_, crate::weaver::watcher::SharedWatcher>,
    project_path: String,
) -> Result<String, String> {
    let target_path = std::path::PathBuf::from(&project_path);
    let target_canonical = target_path.canonicalize().unwrap_or(target_path.clone());

    {
        let current = active_root.read().await;
        if let Some(ref curr) = *current {
            let curr_canonical = curr.canonicalize().unwrap_or(curr.clone());
            if curr_canonical == target_canonical {
                return Ok("Project is already active.".to_string());
            }
        }
    }

    let mut db_write = db.write().await;
    // Release previous file lock
    let temp_mem = crate::persistence::surreal_bridge::connect_embedded()
        .await
        .map_err(|e| e.to_string())?;
    *db_write = temp_mem;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let new_db = crate::persistence::surreal_bridge::connect_db(Some(&target_path))
        .await
        .map_err(|e| e.to_string())?;
        
    *db_write = new_db;

    let mut current_write = active_root.write().await;
    *current_write = Some(target_path.clone());

    watcher.watch_directory(target_path);
    
    Ok("Project successfully opened.".to_string())
}

/// Distills a Source from a selection of monad IDs.
/// Validates coherence before assembling.
#[tauri::command]
pub async fn distill_from_selection(
    db: State<'_, crate::persistence::surreal_bridge::SharedDb>,
    monad_ids: Vec<String>,
) -> Result<String, String> {
    let db_conn = db.read().await;
    let all_monads = get_all_monads(&*db_conn).await
        .map_err(|e| e.to_string())?;

    let selected: Vec<Monad> = all_monads.into_iter()
        .filter(|m| monad_ids.contains(&m.id))
        .collect();

    if selected.is_empty() {
        return Err("No monads found matching the provided IDs.".to_string());
    }

    // Validate coherence
    if let Err(errors) = validate_source_coherence(&selected) {
        let messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
        return Err(format!("Incoherences detected:\n{}", messages.join("\n")));
    }

    let result = DistillResult {
        source: distill_source(&selected),
        monad_count: selected.len(),
        ring_count: {
            let mut rings: Vec<u32> = selected.iter().map(|m| m.ring).collect();
            rings.sort();
            rings.dedup();
            rings.len()
        },
    };

    serde_json::to_string(&result)
        .map_err(|e| e.to_string())
}

/// Traces the evolutionary lineage of a monad toward the Bindu.
#[tauri::command]
pub async fn trace_monad_lineage(
    db: State<'_, crate::persistence::surreal_bridge::SharedDb>,
    monad_id: String,
) -> Result<String, String> {
    let db_conn = db.read().await;
    let chain = threader::trace_full_chain(&*db_conn, &monad_id).await
        .map_err(|e| e.to_string())?;

    let result = LineageResult {
        depth: chain.len(),
        monads: chain,
    };

    serde_json::to_string(&result)
        .map_err(|e| e.to_string())
}

/// Retrieves detailed information about a specific monad.
#[tauri::command]
pub async fn get_monad_detail(
    db: State<'_, crate::persistence::surreal_bridge::SharedDb>,
    monad_id: String,
) -> Result<String, String> {
    let db_conn = db.read().await;
    let all_monads = get_all_monads(&*db_conn).await
        .map_err(|e| e.to_string())?;

    let monad = all_monads.into_iter()
        .find(|m| m.id == monad_id)
        .ok_or_else(|| format!("Monad '{}' not found.", monad_id))?;

    serde_json::to_string(&monad)
        .map_err(|e| e.to_string())
}

/// Writes distilled monads to disk using a specific structure.
#[tauri::command]
pub async fn write_distilled_to_disk(
    db: State<'_, crate::persistence::surreal_bridge::SharedDb>,
    monad_ids: Vec<String>,
    output_path: String,
) -> Result<Vec<String>, String> {
    let db_conn = db.read().await;
    let all_monads = get_all_monads(&*db_conn).await
        .map_err(|e| e.to_string())?;

    let selected: Vec<Monad> = all_monads.into_iter()
        .filter(|m| monad_ids.contains(&m.id))
        .collect();

    if selected.is_empty() {
        return Err("No monads found matching the provided IDs.".to_string());
    }

    let path = std::path::PathBuf::from(output_path);
    let structure = crate::template::OutputStructure::Flat; // Default to flat for now
    
    crate::weaver::file_writer::FileWriter::write_with_structure(&selected, &path, &structure, &[])
        .await
        .map(|paths| paths.into_iter().map(|p| p.display().to_string()).collect())
        .map_err(|e| e.to_string())
}

/// Contracts the outermost ring.
#[tauri::command]
pub async fn contract_outer_ring(
    db: State<'_, crate::persistence::surreal_bridge::SharedDb>
) -> Result<u32, String> {
    let db_conn = db.read().await;
    crate::weaver::contract::contract_ring(&*db_conn).await
        .map_err(|e| e.to_string())
}

/// Reverts the system state to a specific ring level.
#[tauri::command]
pub async fn revert_to_level(
    db: State<'_, crate::persistence::surreal_bridge::SharedDb>,
    ring: u32,
) -> Result<(), String> {
    let db_conn = db.read().await;
    crate::weaver::contract::revert_to_ring(&*db_conn, ring).await
        .map_err(|e| e.to_string())
}

/// Gets the count of archived monads.
#[tauri::command]
pub async fn get_archived_monads_count(
    db: State<'_, crate::persistence::surreal_bridge::SharedDb>
) -> Result<usize, String> {
    let db_conn = db.read().await;
    crate::weaver::contract::get_archived_count(&*db_conn).await
        .map_err(|e| e.to_string())
}

/// Permanently deletes archived monads.
#[tauri::command]
pub async fn purge_archived_monads(
    db: State<'_, crate::persistence::surreal_bridge::SharedDb>
) -> Result<u32, String> {
    let db_conn = db.read().await;
    crate::weaver::contract::purge_archived(&*db_conn).await
        .map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn search_monads(
    query: String,
    db: State<'_, crate::persistence::surreal_bridge::SharedDb>,
) -> Result<Vec<crate::persistence::search::SearchResult>, String> {
    let db_conn = db.read().await;
    crate::persistence::search::SearchEngine::search(&*db_conn, &query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_monads_paginated_api(
    page: usize,
    page_size: usize,
    ring_filter: Option<u32>,
    db: State<'_, crate::persistence::surreal_bridge::SharedDb>,
) -> Result<Vec<Monad>, String> {
    let db_conn = db.read().await;
    crate::persistence::surreal_bridge::get_monads_paginated(&*db_conn, page, page_size, ring_filter)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_export_mandala_state_direct() {
        let db = crate::persistence::surreal_bridge::connect_embedded().await.unwrap();
        let m1 = Monad::spawn(
            "m_export_1".to_string(),
            "alpha".to_string(),
            crate::geometry::polar_space::PolarCoord::new(100.0, 0.0),
            "fn alpha() {}".to_string(),
            1,
        );
        crate::persistence::surreal_bridge::insert_and_link(&db, &m1, None).await.unwrap();
        let all_monads = get_all_monads(&db).await.unwrap();
        assert_eq!(all_monads.len(), 1);

        let edges = get_all_edges(&db).await.unwrap_or_default();
        let bindu_name = get_bindu_name(&db).await;
        let total_monads = all_monads.len();
        let max_ring = all_monads.iter().map(|m| m.ring).max().unwrap_or(0);

        let mut rings_map: std::collections::HashMap<u32, Vec<Monad>> = std::collections::HashMap::new();
        for m in all_monads {
            rings_map.entry(m.ring).or_default().push(m);
        }

        let mut constellations: Vec<ConstellationDto> = rings_map.into_iter()
            .map(|(ring_level, mut monads)| {
                let total = monads.len();
                if total > 0 {
                    let ring_gap = 70.0;
                    let base_ring_radius = 50.0 + ((ring_level as f64) - 1.0) * ring_gap;
                    let band_width = crate::geometry::polar_coordinate::compute_adaptive_band_width(
                        base_ring_radius,
                        ring_gap,
                        total,
                    );
                    let unique_angles: std::collections::HashSet<i64> = monads.iter()
                        .map(|m| m.coord.theta.round() as i64)
                        .collect();
                    let should_redistribute_angles = unique_angles.len() < total.min(32);
                    let offset = ((ring_level % 2) as f64) * (180.0 / total as f64);

                    for (idx, monad) in monads.iter_mut().enumerate() {
                        if should_redistribute_angles {
                            monad.coord.theta = ((360.0 / total as f64) * (idx as f64) + offset) % 360.0;
                        }
                        if band_width > 0.0 {
                            let r_offset = crate::geometry::polar_coordinate::compute_jittered_radius_offset(idx, band_width);
                            monad.coord.r = base_ring_radius + r_offset;
                        } else {
                            monad.coord.r = base_ring_radius;
                        }
                    }
                }
                ConstellationDto { ring_level, monads }
            })
            .collect();
        constellations.sort_by_key(|c| c.ring_level);

        let state = MandalaState {
            bindu_name,
            constellations,
            edges,
            total_monads,
            max_ring,
        };

        let json = serde_json::to_string_pretty(&state).unwrap();
        assert!(json.len() > 100);
    }
}
