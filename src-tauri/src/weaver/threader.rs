use crate::ontology::monad::Monad;
use crate::persistence::surreal_bridge::Db;
use surrealdb::Surreal;
use serde_json::Value as JsonValue;

/// Traza el linaje de una mónada hacia el Bindu a través de relaciones SurrealDB.
/// Follows the `evolves_to` edges backward (from child to parent) toward the center.
pub async fn trace_lineage(db: &Surreal<Db>, monad_id: &str) -> anyhow::Result<Vec<Monad>> {
    let start_id = monad_id.to_string();
    let mut response = db.query("
        SELECT * FROM (
            TRAVERSE <-evolves_to<-monad FROM monad:$start
        ) WHERE id IS NOT NULL
    ")
    .bind(("start", start_id))
    .await?;

    let values: Vec<JsonValue> = response.take(0)?;
    let monads: Vec<Monad> = values
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect();
    Ok(monads)
}

/// Traces the evolution of a monad forward (from parent to children).
/// Returns all descendant monads through the `evolves_to` edges.
pub async fn trace_descendants(db: &Surreal<Db>, monad_id: &str) -> anyhow::Result<Vec<Monad>> {
    let start_id = monad_id.to_string();
    let mut response = db.query("
        SELECT * FROM (
            TRAVERSE ->evolves_to->monad FROM monad:$start
        ) WHERE id IS NOT NULL
    ")
    .bind(("start", start_id))
    .await?;

    let values: Vec<JsonValue> = response.take(0)?;
    let monads: Vec<Monad> = values
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect();
    Ok(monads)
}

/// Returns the full evolutionary chain for a monad — ancestors and descendants combined.
/// Ordered from oldest ancestor to newest descendant, with the queried monad in context.
pub async fn trace_full_chain(db: &Surreal<Db>, monad_id: &str) -> anyhow::Result<Vec<Monad>> {
    let mut ancestors = trace_lineage(db, monad_id).await?;
    let descendants = trace_descendants(db, monad_id).await?;

    // Ancestors come in reverse order (child -> parent), so reverse them
    ancestors.reverse();

    // Avoid duplicates: descendants already exclude the start node
    ancestors.extend(descendants);
    Ok(ancestors)
}
