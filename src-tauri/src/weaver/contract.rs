// Pre-implementation: Contract operation
// ==============================

use crate::persistence::surreal_bridge::{Db, get_all_monads, get_active_monads, archive_monad};
use surrealdb::Surreal;

/// Contracts (archives) the outermost active ring
pub async fn contract_ring(db: &Surreal<Db>) -> anyhow::Result<u32> {
    let active = get_active_monads(db).await?;
    let max_ring = active.iter().map(|m| m.ring).max().unwrap_or(0);
    
    if max_ring == 0 {
        return Err(anyhow::anyhow!("No active rings to contract"));
    }
    
    let ring_monads: Vec<_> = active.into_iter().filter(|m| m.ring == max_ring).collect();
    
    for monad in ring_monads {
        archive_monad(db, &monad.id).await?;
    }
    
    Ok(max_ring.saturating_sub(1))
}

/// Reverts to a specific ring (restores from archive)
pub async fn revert_to_ring(db: &Surreal<Db>, ring: u32) -> anyhow::Result<()> {
    db.query("UPDATE monad SET is_archived = false WHERE ring <= $ring")
        .bind(("ring", ring))
        .await?;
    
    let all = get_all_monads(db).await?;
    let max = all.iter().map(|m| m.ring).max().unwrap_or(0);
    
    if max > ring {
        db.query("UPDATE monad SET is_archived = true WHERE ring > $ring")
            .bind(("ring", ring))
            .await?;
    }
    
    Ok(())
}

/// Gets the count of archived monads
pub async fn get_archived_count(db: &Surreal<Db>) -> anyhow::Result<usize> {
    let mut response = db.query("SELECT count() as count FROM monad WHERE is_archived = true GROUP ALL")
        .await?;
    
    let rows: Vec<serde_json::Value> = response.take(0)?;
    let count = rows
        .first()
        .and_then(|r| r.get("count"))
        .and_then(|c| c.as_u64())
        .unwrap_or(0) as usize;
    Ok(count)
}

/// Permanently deletes archived monads
pub async fn purge_archived(db: &Surreal<Db>) -> anyhow::Result<u32> {
    let mut response = db.query("DELETE FROM monad WHERE is_archived = true RETURN BEFORE")
        .await?;
    
    let deleted: Vec<serde_json::Value> = response.take(0)?;
    Ok(deleted.len() as u32)
}