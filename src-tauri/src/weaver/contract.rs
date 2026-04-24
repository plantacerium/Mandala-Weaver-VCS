// Pre-implementation: Contract operation
// ==============================

#![allow(dead_code)]
use crate::persistence::surreal_bridge::{Db, get_all_monads, archive_monad};
use surrealdb::Surreal;

/// Contracts (removes) the outermost ring
pub async fn contract_ring(db: &Surreal<Db>) -> anyhow::Result<u32> {
    let all = get_all_monads(db).await?;
    let max_ring = all.iter().map(|m| m.ring).max().unwrap_or(0);
    
    if max_ring == 0 {
        return Err(anyhow::anyhow!("No rings to contract"));
    }
    
    let ring_monads: Vec<_> = all.into_iter().filter(|m| m.ring == max_ring).collect();
    
    for monad in ring_monads {
        archive_monad(db, &monad.id).await?;
    }
    
    Ok(max_ring - 1)
}

/// Reverts to a specific ring (restores from archive)
pub async fn revert_to_ring(db: &Surreal<Db>, ring: u32) -> anyhow::Result<()> {
    db.query("UPDATE monad WHERE ring <= $ring SET is_archived = false")
        .bind(("ring", ring))
        .await?;
    
    let all = get_all_monads(db).await?;
    let max = all.iter().map(|m| m.ring).max().unwrap_or(0);
    
    if max > ring {
        db.query("UPDATE monad WHERE ring > $ring SET is_archived = true")
            .bind(("ring", ring))
            .await?;
    }
    
    Ok(())
}

/// Gets the count of archived monads
pub async fn get_archived_count(db: &Surreal<Db>) -> anyhow::Result<usize> {
    let mut response = db.query("SELECT count() as count FROM monad WHERE is_archived = true")
        .await?;
    
    let count: Option<usize> = response.take("count")?;
    Ok(count.unwrap_or(0))
}

/// Permanently deletes archived monads
pub async fn purge_archived(db: &Surreal<Db>) -> anyhow::Result<u32> {
    let mut response = db.query("DELETE FROM monad WHERE is_archived = true")
        .await?;
    
    let deleted: Vec<serde_json::Value> = response.take(0)?;
    Ok(deleted.len() as u32)
}