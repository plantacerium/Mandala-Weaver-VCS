use tauri::State;
use serde::Serialize;
use crate::persistence::surreal_bridge::{Db, get_all_monads};
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

/// Retrieves the Bindu (project name) from SurrealDB, or returns "Unnamed_Project" as default.
async fn get_bindu_name(db: &Surreal<Db>) -> String {
    let result: Result<Vec<JsonValue>, _> = db.select("bindu").await;
    match result {
        Ok(bindus) if !bindus.is_empty() => {
            if let Some(name) = bindus[0].get("project_name").and_then(|v| v.as_str()) {
                return name.to_string();
            }
            "Unnamed_Project".to_string()
        }
        _ => "Unnamed_Project".to_string(),
    }
}

/// Exports the complete spatial state of the Mandala as JSON.
/// Returns: { bindu_name, constellations, total_monads, max_ring }
#[tauri::command]
pub async fn export_mandala_state(db: State<'_, Surreal<Db>>) -> Result<String, String> {
    let all_monads = get_all_monads(&db).await
        .map_err(|e| e.to_string())?;

    let bindu_name = get_bindu_name(&db).await;
    let total_monads = all_monads.len();
    let max_ring = all_monads.iter().map(|m| m.ring).max().unwrap_or(0);

    // Agrupar por anillos
    let mut rings_map: std::collections::HashMap<u32, Vec<Monad>> = std::collections::HashMap::new();
    for m in all_monads {
        rings_map.entry(m.ring).or_default().push(m);
    }

    let mut constellations: Vec<ConstellationDto> = rings_map.into_iter()
        .map(|(ring_level, monads)| ConstellationDto { ring_level, monads })
        .collect();
    
    // Sort constellations by ring level for consistent output
    constellations.sort_by_key(|c| c.ring_level);

    let state = MandalaState {
        bindu_name,
        constellations,
        total_monads,
        max_ring,
    };

    serde_json::to_string(&state)
        .map_err(|e| e.to_string())
}

/// Expands a new ring from a source file.
#[tauri::command]
pub async fn expand_ring(
    db: State<'_, Surreal<Db>>, 
    file_path: String
) -> Result<u32, String> {
    crate::weaver::expand_from_source(&db, &file_path).await
        .map_err(|e| e.to_string())
}

/// Initializes a new project by creating the Bindu (center point).
#[tauri::command]
pub async fn init_project(
    db: State<'_, Surreal<Db>>,
    project_name: String,
) -> Result<String, String> {
    let bindu = Bindu::genesis(&project_name);
    let json_value = serde_json::to_value(&bindu)
        .map_err(|e| e.to_string())?;

    let _: Option<JsonValue> = db.create(("bindu", "genesis"))
        .content(json_value)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("Project '{}' initialized at the Bindu.", project_name))
}

/// Distills a Source from a selection of monad IDs.
/// Validates coherence before assembling.
#[tauri::command]
pub async fn distill_from_selection(
    db: State<'_, Surreal<Db>>,
    monad_ids: Vec<String>,
) -> Result<String, String> {
    let all_monads = get_all_monads(&db).await
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
    db: State<'_, Surreal<Db>>,
    monad_id: String,
) -> Result<String, String> {
    let chain = threader::trace_full_chain(&db, &monad_id).await
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
    db: State<'_, Surreal<Db>>,
    monad_id: String,
) -> Result<String, String> {
    let all_monads = get_all_monads(&db).await
        .map_err(|e| e.to_string())?;

    let monad = all_monads.into_iter()
        .find(|m| m.id == monad_id)
        .ok_or_else(|| format!("Monad '{}' not found.", monad_id))?;

    serde_json::to_string(&monad)
        .map_err(|e| e.to_string())
}
