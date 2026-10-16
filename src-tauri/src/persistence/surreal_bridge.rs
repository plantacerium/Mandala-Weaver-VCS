use surrealdb::engine::local::{Mem, SurrealKv};
use std::path::Path;
use surrealdb::Surreal;
use serde_json::Value as JsonValue;
use crate::ontology::monad::Monad;
use crate::persistence::schemas::get_initialization_queries;

use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type Db = surrealdb::engine::local::Db;
pub type SharedDb = Arc<RwLock<Surreal<Db>>>;
pub type ActiveProjectState = Arc<RwLock<Option<std::path::PathBuf>>>;

/// Busca el directorio raíz del proyecto (.mandala) subiendo desde el directorio actual
pub fn find_project_root() -> Option<std::path::PathBuf> {
    let mut current = env::current_dir().ok()?;
    loop {
        if current.join(".mandala").is_dir() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

/// Conecta a la base de datos persistente (SurrealKV) o Memoria
pub async fn connect_db(project_path: Option<&Path>) -> anyhow::Result<Surreal<Db>> {
    let db = match project_path {
        Some(path) => {
            let db_path = path.join(".mandala").join("db");
            std::fs::create_dir_all(&db_path).ok();
            Surreal::new::<SurrealKv>(db_path.to_string_lossy().to_string()).await?
        }
        None => Surreal::new::<Mem>(()).await?,
    };

    db.use_ns("mandala").use_db("weaver").await?;
    
    // Ejecutar inicialización
    for query in get_initialization_queries() {
        db.query(query).await?;
    }
    
    Ok(db)
}

/// Conecta a la base de datos Mem (temporal) - wrapper para tests y compatibilidad
pub async fn connect_embedded() -> anyhow::Result<Surreal<Db>> {
    connect_db(None).await
}

/// Deserializa de forma segura un valor JSON de SurrealDB a `Monad`,
/// evitando inyectar mónadas fantasma con nombre "error" si un registro tiene campos de `PersistedMonad`.
fn parse_monad_value(v: JsonValue) -> Option<Monad> {
    if let Ok(mut m) = serde_json::from_value::<Monad>(v.clone()) {
        if !m.id.is_empty() && m.id != "error" {
            if m.name.is_empty() {
                if let Some(sym) = v.get("symbol_name").and_then(|s| s.as_str()) {
                    m.name = sym.to_string();
                }
            }
            if m.content.is_empty() {
                if let Some(raw) = v.get("raw_source").and_then(|s| s.as_str()) {
                    m.content = raw.to_string();
                }
            }
            return Some(m);
        }
    }

    // Fallback para registros que provengan exclusivamente de PersistedMonad
    let obj = v.as_object()?;
    let id = obj
        .get("implementation_hash")
        .or_else(|| obj.get("id"))
        .and_then(|val| val.as_str())?
        .to_string();
    let name = obj
        .get("name")
        .or_else(|| obj.get("symbol_name"))
        .and_then(|val| val.as_str())
        .unwrap_or("unnamed_symbol")
        .to_string();
    let content = obj
        .get("content")
        .or_else(|| obj.get("raw_source"))
        .and_then(|val| val.as_str())
        .unwrap_or("")
        .to_string();
    let ring = obj.get("ring").and_then(|val| val.as_u64()).unwrap_or(1) as u32;
    let r = obj
        .get("coord")
        .and_then(|c| c.get("r"))
        .and_then(|val| val.as_f64())
        .unwrap_or((ring as f64) * 100.0);
    let theta = obj
        .get("coord")
        .and_then(|c| c.get("theta"))
        .and_then(|val| val.as_f64())
        .unwrap_or(0.0);

    let mut m = Monad::spawn(
        id,
        name,
        crate::geometry::polar_space::PolarCoord::new(r, theta),
        content,
        ring,
    );
    if let Some(c_hash) = obj.get("contract_hash").and_then(|val| val.as_str()) {
        m.contract_hash = c_hash.to_string();
    }
    if let Some(i_hash) = obj.get("implementation_hash").and_then(|val| val.as_str()) {
        m.implementation_hash = i_hash.to_string();
    }
    if let Some(sig) = obj.get("canonical_signature").and_then(|val| val.as_str()) {
        m.canonical_signature = sig.to_string();
    }
    if let Some(archived) = obj.get("is_archived").and_then(|val| val.as_bool()) {
        m.is_archived = archived;
    }
    Some(m)
}

/// Persiste una mónada en el espacio, registra su contrato y puerto en el grafo tripartito
/// y crea la arista evolutiva desde su versión anterior.
pub async fn insert_and_link(
    db: &Surreal<Db>, 
    current: &Monad, 
    parent_id: Option<&str>
) -> anyhow::Result<()> {
    let store = crate::persistence::MandalaStore::from_existing_db(db.clone());
    let mut polar = crate::geometry::PolarCoordinate::from(&current.coord);
    polar.ring = crate::geometry::polar_coordinate::RingLevel(current.ring);

    // 1. Insertar o actualizar en el Grafo Tripartito (monad + contract + port + outbound_calls)
    store.upsert_monad(current, &polar).await?;
        
    // 2. Si hay un padre, crear la arista de evolución tanto en evolves_to como en evolves_from (SO(2))
    if let Some(pid) = parent_id {
        let child_hash = if !current.implementation_hash.is_empty() {
            current.implementation_hash.as_str()
        } else {
            current.id.as_str()
        };
        let parent_owned = pid.to_string();
        let current_owned = child_hash.to_string();
        let rel_query = r#"
            LET $p = type::record("monad", $parent);
            LET $c = type::record("monad", $current);
            RELATE $p -> evolves_to -> $c;
        "#;
        let _ = db.query(rel_query)
            .bind(("parent", parent_owned))
            .bind(("current", current_owned))
            .await;

        let _ = store.record_evolution(pid, child_hash, "Radial expansion").await;
    }
    
    Ok(())
}

/// Persiste múltiples mónadas y sus relaciones en lote para alto rendimiento (Phase 9.2),
/// poblando simultáneamente el Grafo Tripartito Ontológico (`monad` - `contract` - `port`).
pub async fn insert_batch(
    db: &Surreal<Db>,
    items: &[(&Monad, Option<&str>)],
) -> anyhow::Result<()> {
    if items.is_empty() {
        return Ok(());
    }

    let store = crate::persistence::MandalaStore::from_existing_db(db.clone());

    for (monad, parent_id) in items {
        let mut polar = crate::geometry::PolarCoordinate::from(&monad.coord);
        polar.ring = crate::geometry::polar_coordinate::RingLevel(monad.ring);
        store.upsert_monad(monad, &polar).await?;

        if let Some(pid) = parent_id {
            let child_hash = if !monad.implementation_hash.is_empty() {
                monad.implementation_hash.as_str()
            } else {
                monad.id.as_str()
            };
            let rel_query = r#"
                LET $p = type::record("monad", $parent);
                LET $c = type::record("monad", $current);
                RELATE $p -> evolves_to -> $c;
            "#;
            let _ = db.query(rel_query)
                .bind(("parent", pid.to_string()))
                .bind(("current", child_hash.to_string()))
                .await;

            let _ = store.record_evolution(pid, child_hash, "Batch radial expansion").await;
        }
    }

    Ok(())
}

/// Recupera mónadas con paginación optimizada (Phase 9.3 Lazy Loading)
pub async fn get_monads_paginated(
    db: &Surreal<Db>,
    page: usize,
    page_size: usize,
    ring_filter: Option<u32>,
) -> anyhow::Result<Vec<Monad>> {
    let start = page.saturating_mul(page_size);
    let mut response = if let Some(ring) = ring_filter {
        db.query("SELECT * FROM monad WHERE ring = $ring AND is_archived = false ORDER BY coord.theta ASC LIMIT $limit START $start")
            .bind(("ring", ring))
            .bind(("limit", page_size))
            .bind(("start", start))
            .await?
    } else {
        db.query("SELECT * FROM monad WHERE is_archived = false ORDER BY ring ASC, coord.theta ASC LIMIT $limit START $start")
            .bind(("limit", page_size))
            .bind(("start", start))
            .await?
    };

    let values: Vec<JsonValue> = response.take(0)?;
    let monads: Vec<Monad> = values.into_iter().filter_map(parse_monad_value).collect();
    Ok(monads)
}

/// Recupera todas las mónadas de un anillo específico.
pub async fn get_ring(db: &Surreal<Db>, ring: u32) -> anyhow::Result<Vec<Monad>> {
    let mut response = db.query("SELECT * FROM monad WHERE ring = $ring AND is_archived = false")
        .bind(("ring", ring))
        .await?;
    
    let values: Vec<JsonValue> = response.take(0)?;
    let monads: Vec<Monad> = values
        .into_iter()
        .filter_map(parse_monad_value)
        .collect();
    Ok(monads)
}

/// Recupera mónadas dentro de un sector angular específico.

pub async fn get_vector_sector(
    db: &Surreal<Db>, 
    min_theta: f64, 
    max_theta: f64
) -> anyhow::Result<Vec<Monad>> {
    let mut response = db.query("SELECT * FROM monad WHERE coord.theta >= $min AND coord.theta <= $max AND is_archived = false")
        .bind(("min", min_theta))
        .bind(("max", max_theta))
        .await?;
    
    let values: Vec<JsonValue> = response.take(0)?;
    let monads: Vec<Monad> = values
        .into_iter()
        .filter_map(parse_monad_value)
        .collect();
    Ok(monads)
}

/// Recupera absolutamente todas las mónadas del Mandala.
pub async fn get_all_monads(db: &Surreal<Db>) -> anyhow::Result<Vec<Monad>> {
    let values: Vec<JsonValue> = db.select("monad").await?;
    let monads: Vec<Monad> = values
        .into_iter()
        .filter_map(parse_monad_value)
        .collect();
    Ok(monads)
}

/// Get only active (non-archived) monads

pub async fn get_active_monads(db: &Surreal<Db>) -> anyhow::Result<Vec<Monad>> {
    let mut response = db.query(
        "SELECT * FROM monad WHERE is_archived = false"
    ).await?;
    
    let values: Vec<JsonValue> = response.take(0)?;
    let monads: Vec<Monad> = values
        .into_iter()
        .filter_map(parse_monad_value)
        .collect();
    Ok(monads)
}

/// Archive (soft delete) a monad
pub async fn archive_monad(db: &Surreal<Db>, monad_id: &str) -> anyhow::Result<()> {
    let id_owned = monad_id.to_string();
    db.query("UPDATE monad:$id SET is_archived = true")
        .bind(("id", id_owned))
        .await?;
    Ok(())
}

/// Restore an archived monad

pub async fn restore_monad(db: &Surreal<Db>, monad_id: &str) -> anyhow::Result<()> {
    let id_owned = monad_id.to_string();
    db.query("UPDATE monad:$id SET is_archived = false")
        .bind(("id", id_owned))
        .await?;
    Ok(())
}

/// Busca una mónada por nombre (contiene el string)

pub async fn get_monad_by_name(db: &Surreal<Db>, name_contains: &str) -> anyhow::Result<Vec<Monad>> {
    let name_owned = name_contains.to_string();
    let mut response = db.query("SELECT * FROM monad WHERE name CONTAINS $name")
        .bind(("name", name_owned))
        .await?;
    let values: Vec<JsonValue> = response.take(0)?;
    let monads: Vec<Monad> = values
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect();
    Ok(monads)
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct EdgeDto {
    pub parent_id: String,
    pub child_id: String,
}

/// Recupera todas las aristas de evolución
pub async fn get_all_edges(db: &Surreal<Db>) -> anyhow::Result<Vec<EdgeDto>> {
    let mut response = db.query(r#"
        SELECT 
            string::slice(type::string(in), 6) as parent_id,
            string::slice(type::string(out), 6) as child_id 
        FROM evolves_to
    "#).await?;
    let values: Vec<JsonValue> = response.take(0)?;
    let edges: Vec<EdgeDto> = values.into_iter().filter_map(|v| serde_json::from_value(v).ok()).collect();
    Ok(edges)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::polar_space::PolarCoord;
    use crate::ontology::monad::Monad;

    #[tokio::test]
    async fn test_simulate_growth() -> anyhow::Result<()> {
        let db = connect_embedded().await?;
        
        // Simular 1000 mónadas en 10 anillos
        for r in 1..=10 {
            for i in 0..100 {
                let hash = format!("hash_{}_{}", r, i);
                let name = format!("monad_{}_{}", r, i);
                let coord = PolarCoord::new((r as f64) * 50.0, (i as f64) * 3.6);
                let monad = Monad::spawn(hash, name, coord, "content".to_string(), r as u32);
                
                insert_and_link(&db, &monad, None).await?;
            }
        }
        
        let all = get_all_monads(&db).await?;
        assert_eq!(all.len(), 1000);
        
        let ring_5 = get_ring(&db, 5).await?;
        assert_eq!(ring_5.len(), 100);
        
        println!("✅ Simulación de 1000 mónadas completada con éxito.");
        Ok(())
    }
}
