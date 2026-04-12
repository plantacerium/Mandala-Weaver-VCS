use crate::ontology::monad::Monad;
use crate::persistence::surreal_bridge::Db;
use surrealdb::Surreal;
use serde_json::Value as JsonValue;

/// Traza el linaje de una mónada hacia el Bindu a través de relaciones SurrealDB.
pub async fn trace_lineage(db: &Surreal<Db>, monad_id: &str) -> anyhow::Result<Vec<Monad>> {
    let start_id = monad_id.to_string();
    let mut response = db.query("
        SELECT * FROM (
            TRAVERSE ->evolves_to FROM monad:$start
        ) WHERE id IS NOT NULL
    ")
    .bind(("start", start_id))
    .await?;

    let values: Vec<JsonValue> = response.take(0)?;
    let monads: Vec<Monad> = values
        .into_iter()
        .map(|v| serde_json::from_value(v).unwrap_or_else(|_| Monad::spawn("error".to_string(), "error".to_string(), crate::geometry::polar_space::PolarCoord::new(0.0, 0.0), "error".to_string(), 0)))
        .collect();
    Ok(monads)
}
