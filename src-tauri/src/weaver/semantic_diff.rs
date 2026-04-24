// Pre-implementation: Semantic diff
// ==========================

use serde::Serialize;
use crate::ontology::monad::Monad;

/// Semantic diff result between two monads
#[derive(Debug, Serialize)]
pub struct DiffResult {
    pub monad_id: String,
    pub old_hash: String,
    pub new_hash: String,
    pub changes: Vec<Change>,
}

/// Single change in semantic diff
#[derive(Debug, Serialize)]
pub struct Change {
    pub kind: ChangeKind,
    pub description: String,
}

/// Type of change detected
#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum ChangeKind {
    ContentModified,
    NameChanged,
    TypeChanged,
    LocationChanged,
    Added,
    Removed,
}

/// Semantic diff generator
pub struct SemanticDiff;

impl SemanticDiff {
    /// Generates semantic diff between two monads (AST-level, not text-level)
    pub fn generate(old: &Monad, new: &Monad) -> DiffResult {
        let mut changes = Vec::new();
        
        if old.content != new.content {
            changes.push(Change {
                kind: ChangeKind::ContentModified,
                description: "Function body changed".to_string(),
            });
        }
        
        if old.name != new.name {
            changes.push(Change {
                kind: ChangeKind::NameChanged,
                description: format!("Renamed from {} to {}", old.name, new.name),
            });
        }
        
        if old.kind != new.kind {
            changes.push(Change {
                kind: ChangeKind::TypeChanged,
                description: format!("Kind changed from {:?} to {:?}", old.kind, new.kind),
            });
        }
        
        if old.coord != new.coord {
            changes.push(Change {
                kind: ChangeKind::LocationChanged,
                description: format!(
                    "Coordinates changed from ({:.1}, {:.1}) to ({:.1}, {:.1})",
                    old.coord.r, old.coord.theta, new.coord.r, new.coord.theta
                ),
            });
        }
        
        DiffResult {
            monad_id: new.id.clone(),
            old_hash: old.semantic_hash.clone(),
            new_hash: new.semantic_hash.clone(),
            changes,
        }
    }
    
    /// Format as colored diff output
    pub fn format_colored(diff: &DiffResult) -> String {
        let mut output = String::new();
        
        for change in &diff.changes {
            let symbol = match change.kind {
                ChangeKind::ContentModified => "~",
                ChangeKind::NameChanged => "R",
                ChangeKind::TypeChanged => "T",
                ChangeKind::LocationChanged => "L",
                ChangeKind::Added => "+",
                ChangeKind::Removed => "-",
            };
            output.push_str(&format!("{} {}\n", symbol, change.description));
        }
        
        output
    }
    
    /// Generate diff between a vector of old and new monads
    pub fn generate_batch(old_monads: &[Monad], new_monads: &[Monad]) -> Vec<DiffResult> {
        let mut results = Vec::new();
        
        for new_m in new_monads {
            if let Some(old_m) = old_monads.iter().find(|o| o.name == new_m.name) {
                let diff = Self::generate(old_m, new_m);
                if !diff.changes.is_empty() {
                    results.push(diff);
                }
            } else {
                results.push(DiffResult {
                    monad_id: new_m.id.clone(),
                    old_hash: String::new(),
                    new_hash: new_m.semantic_hash.clone(),
                    changes: vec![Change {
                        kind: ChangeKind::Added,
                        description: format!("{} added in ring {}", new_m.name, new_m.ring),
                    }],
                });
            }
        }
        
        for old_m in old_monads {
            if !new_monads.iter().any(|n| n.name == old_m.name) {
                results.push(DiffResult {
                    monad_id: old_m.id.clone(),
                    old_hash: old_m.semantic_hash.clone(),
                    new_hash: String::new(),
                    changes: vec![Change {
                        kind: ChangeKind::Removed,
                        description: format!("{} removed", old_m.name),
                    }],
                });
            }
        }
        
        results
    }
    
    /// Calculate similarity score between two monads (0.0 to 1.0)
    pub fn similarity(old: &Monad, new: &Monad) -> f64 {
        let mut score = 0.0;
        
        if old.name == new.name {
            score += 0.3;
        }
        if old.kind == new.kind {
            score += 0.2;
        }
        if old.ring == new.ring {
            score += 0.1;
        }
        
        let old_chars: Vec<char> = old.content.chars().collect();
        let new_chars: Vec<char> = new.content.chars().collect();
        
        if !old_chars.is_empty() && !new_chars.is_empty() {
            let common = old_chars.iter()
                .filter(|c| new_chars.contains(c))
                .count();
            let max = old_chars.len().max(new_chars.len());
            score += (common as f64 / max as f64) * 0.4;
        }
        
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::polar_space::PolarCoord;
    use crate::ontology::monad::Monad;

    #[test]
    fn test_generate() {
        let old = Monad::spawn(
            "id1".to_string(),
            "test_fn".to_string(),
            PolarCoord::new(1.0, 0.0),
            "fn test() {}".to_string(),
            1,
        );
        
        let new = Monad::spawn(
            "id2".to_string(),
            "test_fn".to_string(),
            PolarCoord::new(2.0, 0.0),
            "fn test() { println!(\"changed\") }".to_string(),
            2,
        );
        
        let diff = SemanticDiff::generate(&old, &new);
        assert_eq!(diff.changes.len(), 2);
    }
}