use serde::{Deserialize, Serialize};
use crate::ontology::monad::Monad;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constellation {
    pub ring_level: u32,
    pub monads: Vec<Monad>,
}

impl Constellation {
    pub fn new(ring_level: u32) -> Self {
        Self {
            ring_level,
            monads: Vec::new(),
        }
    }

    pub fn validate_harmony(&self) -> Result<(), String> {
        use std::collections::HashMap;

        let mut hash_counts: HashMap<&str, usize> = HashMap::new();
        let mut coord_occurrences: HashMap<(u32, u32), Vec<&str>> = HashMap::new();

        for monad in &self.monads {
            *hash_counts.entry(&monad.semantic_hash).or_insert(0) += 1;

            let coord_key = (
                (monad.coord.r * 10.0).round() as u32,
                (monad.coord.theta * 10.0).round() as u32,
            );
            coord_occurrences
                .entry(coord_key)
                .or_default()
                .push(&monad.name);
        }

        let mut issues: Vec<String> = Vec::new();

        for (hash, count) in &hash_counts {
            if *count > 1 {
                issues.push(format!(
                    "Duplicate semantic hash '{}' found {} times in ring {}",
                    &hash[..12.min(hash.len())], count, self.ring_level
                ));
            }
        }

        for (coord, names) in &coord_occurrences {
            if names.len() > 1 {
                issues.push(format!(
                    "Coordinate collision at ({}, {}°): {} monads share position",
                    coord.0, coord.1, names.len()
                ));
            }
        }

        for monad in &self.monads {
            if monad.content.trim().is_empty() {
                issues.push(format!(
                    "Empty content in monad '{}' (id: {})",
                    monad.name, monad.id
                ));
            }
        }

        if !issues.is_empty() {
            return Err(issues.join("\n"));
        }

        Ok(())
    }
}
