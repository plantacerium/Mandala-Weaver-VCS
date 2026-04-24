use surrealdb::engine::local::Mem;
use surrealdb::Surreal;
use serde_json::Value as JsonValue;
use crate::ontology::monad::Monad;
use crate::persistence::schemas::get_initialization_queries;

pub type Db = surrealdb::engine::local::Db;

/// Conecta a la base de datos Mem (temporal)
pub async fn connect_embedded() -> anyhow::Result<Surreal<Db>> {
    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("mandala").use_db("weaver").await?;
    
    // Ejecutar inicialización
    for query in get_initialization_queries() {
        db.query(query).await?;
    }
    
    Ok(db)
}

/// Persiste una mónada en el espacio y crea la arista evolutiva desde su versión anterior.
pub async fn insert_and_link(
    db: &Surreal<Db>, 
    current: &Monad, 
    parent_id: Option<&str>
) -> anyhow::Result<()> {
    let json_value = serde_json::to_value(current)?;
    
    // Insertar la mónada
    let _: Option<JsonValue> = db.create(("monad", current.id.as_str()))
        .content(json_value)
        .await?;
        
    // Si hay un padre, crear la arista de evolución
    if let Some(pid) = parent_id {
        let parent_owned = pid.to_string();
        let current_owned = current.id.clone();
        db.query("RELATE monad:$parent -> evolves_to -> monad:$current")
            .bind(("parent", parent_owned))
            .bind(("current", current_owned))
            .await?;
    }
    
    Ok(())
}

/// Recupera todas las mónadas de un anillo específico.
pub async fn get_ring(db: &Surreal<Db>, ring: u32) -> anyhow::Result<Vec<Monad>> {
    let mut response = db.query("SELECT * FROM monad WHERE ring = $ring AND is_archived = false")
        .bind(("ring", ring))
        .await?;
    
    let values: Vec<JsonValue> = response.take(0)?;
    let monads: Vec<Monad> = values
        .into_iter()
        .map(|v| serde_json::from_value(v).unwrap_or_else(|_| Monad::spawn("error".to_string(), "error".to_string(), crate::geometry::polar_space::PolarCoord::new(0.0, 0.0), "error".to_string(), 0)))
        .collect();
    Ok(monads)
}

/// Recupera mónadas dentro de un sector angular específico.
#[allow(dead_code)]
pub async fn get_vector_sector(
    db: &Surreal<Db>, 
    min_theta: f64, 
    max_theta: f64
) -> anyhow::Result<Vec<Monad>> {
    let mut response = db.query("SELECT * FROM monad WHERE coord.theta >= $min AND coord.theta <= $max")
        .bind(("min", min_theta))
        .bind(("max", max_theta))
        .await?;
    
    let values: Vec<JsonValue> = response.take(0)?;
    let monads: Vec<Monad> = values
        .into_iter()
        .map(|v| serde_json::from_value(v).unwrap_or_else(|_| Monad::spawn("error".to_string(), "error".to_string(), crate::geometry::polar_space::PolarCoord::new(0.0, 0.0), "error".to_string(), 0)))
        .collect();
    Ok(monads)
}

/// Recupera absolutamente todas las mónadas del Mandala.
pub async fn get_all_monads(db: &Surreal<Db>) -> anyhow::Result<Vec<Monad>> {
    let values: Vec<JsonValue> = db.select("monad").await?;
    let monads: Vec<Monad> = values
        .into_iter()
        .map(|v| serde_json::from_value(v).unwrap_or_else(|_| Monad::spawn("error".to_string(), "error".to_string(), crate::geometry::polar_space::PolarCoord::new(0.0, 0.0), "error".to_string(), 0)))
        .collect();
    Ok(monads)
}

/// Get only active (non-archived) monads
#[allow(dead_code)]
pub async fn get_active_monads(db: &Surreal<Db>) -> anyhow::Result<Vec<Monad>> {
    let mut response = db.query(
        "SELECT * FROM monad WHERE is_archived = false"
    ).await?;
    
    let values: Vec<JsonValue> = response.take(0)?;
    let monads: Vec<Monad> = values
        .into_iter()
        .map(|v| serde_json::from_value(v).unwrap_or_else(|_| Monad::spawn("error".to_string(), "error".to_string(), crate::geometry::polar_space::PolarCoord::new(0.0, 0.0), "error".to_string(), 0)))
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
#[allow(dead_code)]
pub async fn restore_monad(db: &Surreal<Db>, monad_id: &str) -> anyhow::Result<()> {
    let id_owned = monad_id.to_string();
    db.query("UPDATE monad:$id SET is_archived = false")
        .bind(("id", id_owned))
        .await?;
    Ok(())
}

/// Busca una mónada por nombre (contiene el string)
#[allow(dead_code)]
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
