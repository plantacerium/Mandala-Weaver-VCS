// Pre-implementation: Semantic Search
// ==============================

use serde::Serialize;
use crate::ontology::monad::Monad;
use crate::persistence::surreal_bridge::Db;
use surrealdb::Surreal;

/// Search result including match type
#[derive(Serialize)]
pub struct SearchResult {
    pub id: String,
    pub name: String,
    pub ring: u32,
    pub coord: crate::geometry::polar_space::PolarCoord,
    pub kind: String,
    pub semantic_hash: String,
    pub match_type: MatchType,
}

/// Type of match in search results
#[derive(Serialize, Debug, PartialEq, Eq)]
pub enum MatchType {
    Name,
    Content,
    Hash,
    Fuzzy,
}

/// Search engine for monads
pub struct SearchEngine;

impl SearchEngine {
    /// Search monads by name pattern
    pub async fn by_name(db: &Surreal<Db>, query: &str) -> anyhow::Result<Vec<Monad>> {
        let query_owned = query.to_string();
        let mut response = db.query(
            "SELECT * FROM monad WHERE name CONTAINS $query AND is_archived = false"
        )
        .bind(("query", query_owned))
        .await?;
        
        let values: Vec<serde_json::Value> = response.take(0)?;
        let monads: Vec<Monad> = values
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
        Ok(monads)
    }
    
    /// Search by exact semantic hash
    pub async fn by_hash(db: &Surreal<Db>, hash: &str) -> anyhow::Result<Vec<Monad>> {
        let hash_owned = hash.to_string();
        let mut response = db.query(
            "SELECT * FROM monad WHERE semantic_hash = $hash"
        )
        .bind(("hash", hash_owned))
        .await?;
        
        let values: Vec<serde_json::Value> = response.take(0)?;
        let monads: Vec<Monad> = values
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
        Ok(monads)
    }
    
    /// Combined full-text + semantic search
    pub async fn search(db: &Surreal<Db>, query: &str) -> anyhow::Result<Vec<SearchResult>> {
        let query_owned = query.to_string();
        let mut response = db.query(r#"
            SELECT id, name, ring, coord, kind, semantic_hash,
            CASE 
                WHEN name CONTAINS $query THEN 'name'
                WHEN content CONTAINS $query THEN 'content'
                ELSE 'fuzzy'
            END as match_type
            FROM monad
            WHERE is_archived = false
            ORDER BY ring DESC
        "#)
        .bind(("query", query_owned))
        .await?;
        
        let values: Vec<serde_json::Value> = response.take(0)?;
        let results: Vec<SearchResult> = values
            .into_iter()
            .filter_map(|v| {
                let match_type_str = v.get("match_type")?.as_str()?;
                let match_type = match match_type_str {
                    "name" => MatchType::Name,
                    "content" => MatchType::Content,
                    "fuzzy" => MatchType::Fuzzy,
                    _ => MatchType::Fuzzy,
                };
                
                Some(SearchResult {
                    id: v.get("id")?.as_str()?.to_string(),
                    name: v.get("name")?.as_str()?.to_string(),
                    ring: v.get("ring")?.as_u64()? as u32,
                    coord: crate::geometry::polar_space::PolarCoord::new(
                        v.get("coord").and_then(|c| c.get("r"))?.as_f64().unwrap_or(0.0),
                        v.get("coord").and_then(|c| c.get("theta"))?.as_f64().unwrap_or(0.0),
                    ),
                    kind: v.get("kind")?.as_str()?.to_string(),
                    semantic_hash: v.get("semantic_hash")?.as_str()?.to_string(),
                    match_type,
                })
            })
            .collect();
        Ok(results)
    }
    
    /// Search by ring range
    pub async fn by_ring_range(
        db: &Surreal<Db>, 
        min_ring: u32, 
        max_ring: u32
    ) -> anyhow::Result<Vec<Monad>> {
        let mut response = db.query(
            "SELECT * FROM monad WHERE ring >= $min AND ring <= $max AND is_archived = false"
        )
        .bind(("min", min_ring))
        .bind(("max", max_ring))
        .await?;
        
        let values: Vec<serde_json::Value> = response.take(0)?;
        let monads: Vec<Monad> = values
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
        Ok(monads)
    }
    
    /// Search by theta (vector) range
    pub async fn by_theta_range(
        db: &Surreal<Db>, 
        min_theta: f64, 
        max_theta: f64
    ) -> anyhow::Result<Vec<Monad>> {
        let mut response = db.query(
            "SELECT * FROM monad WHERE coord.theta >= $min AND coord.theta <= $max AND is_archived = false"
        )
        .bind(("min", min_theta))
        .bind(("max", max_theta))
        .await?;
        
        let values: Vec<serde_json::Value> = response.take(0)?;
        let monads: Vec<Monad> = values
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
        Ok(monads)
    }
}