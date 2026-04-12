use tauri::State;
use serde::Serialize;
use crate::persistence::surreal_bridge::{Db, get_all_monads};
use crate::ontology::monad::Monad;
use surrealdb::Surreal;

#[derive(Serialize)]
pub struct MandalaState {
    pub bindu_name: String,
    pub constellations: Vec<ConstellationDto>,
}

#[derive(Serialize)]
pub struct ConstellationDto {
    pub ring_level: u32,
    pub monads: Vec<Monad>,
}

#[tauri::command]
pub async fn export_mandala_state(db: State<'_, Surreal<Db>>) -> Result<String, String> {
    let all_monads = get_all_monads(&db).await
        .map_err(|e| e.to_string())?;

    // Agrupar por anillos
    let mut rings_map: std::collections::HashMap<u32, Vec<Monad>> = std::collections::HashMap::new();
    for m in all_monads {
        rings_map.entry(m.ring).or_default().push(m);
    }

    let constellations = rings_map.into_iter()
        .map(|(ring_level, monads)| ConstellationDto { ring_level, monads })
        .collect();

    let state = MandalaState {
        bindu_name: "Untitled_Project".to_string(),
        constellations,
    };

    serde_json::to_string(&state)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn expand_ring(
    db: State<'_, Surreal<Db>>, 
    file_path: String
) -> Result<u32, String> {
    crate::weaver::expand_from_source(&db, &file_path).await
        .map_err(|e| e.to_string())
}
