use crate::ontology::monad::Monad;
use crate::persistence::surreal_bridge::Db;
use crate::template::{
    DistillationTemplate, ExcludeRule, RingSelector, VectorSelector,
};
use anyhow::Result;
use surrealdb::Surreal;
use glob::Pattern;

pub struct TemplateEngine {
    db: Surreal<Db>,
}

impl TemplateEngine {
    pub fn new(db: Surreal<Db>) -> Self {
        Self { db }
    }

    pub async fn resolve(&self, template: &DistillationTemplate) -> Result<Vec<Monad>> {
        let mut candidates = Vec::new();

        let ring_monads = self.resolve_rings(&template.rings).await?;
        candidates.extend(ring_monads);

        if !template.vectors.is_empty() {
            candidates = self.filter_by_vectors(candidates, &template.vectors).await;
        }

        if !template.exclude.is_empty() {
            candidates = self.apply_exclusions(candidates, &template.exclude);
        }

        if !template.adapters.is_empty() {
            crate::template::adapter::AdapterEngine::resolve_with_adapters(&candidates, &template.adapters);
        }

        Ok(candidates)
    }

    async fn resolve_rings(&self, selector: &RingSelector) -> Result<Vec<Monad>> {
        match selector {
            RingSelector::Level(n) => {
                crate::persistence::surreal_bridge::get_ring(&self.db, *n).await
            }
            RingSelector::Latest => {
                let all = crate::persistence::surreal_bridge::get_all_monads(&self.db).await?;
                let max_ring = all.iter().map(|m| m.ring).max().unwrap_or(0);
                if max_ring == 0 {
                    Ok(vec![])
                } else {
                    crate::persistence::surreal_bridge::get_ring(&self.db, max_ring).await
                }
            }
            RingSelector::Range { min, max } => {
                let all = crate::persistence::surreal_bridge::get_all_monads(&self.db).await?;
                Ok(all.into_iter()
                    .filter(|m| m.ring >= *min && m.ring <= *max)
                    .collect())
            }
            RingSelector::All => {
                crate::persistence::surreal_bridge::get_all_monads(&self.db).await
            }
        }
    }

    async fn filter_by_vectors(&self, monads: Vec<Monad>, vectors: &[VectorSelector]) -> Vec<Monad> {
        monads.into_iter()
            .filter(|m| {
                vectors.iter().any(|v| {
                    if let (Some(start), Some(end)) = (v.angle_start, v.angle_end) {
                        m.coord.theta >= start && m.coord.theta <= end
                    } else if v.ring.is_some() {
                        v.ring.map(|r| m.ring == r).unwrap_or(false)
                    } else {
                        true
                    }
                })
            })
            .collect()
    }

    pub fn apply_exclusions(&self, mut monads: Vec<Monad>, excludes: &[ExcludeRule]) -> Vec<Monad> {
        for exclude in excludes {
            monads.retain(|m| {
                let keep = if let Some(ref name) = exclude.name {
                    m.name != *name
                } else if let Some(ref pattern) = exclude.pattern {
                    !match_glob(&m.name, pattern)
                } else if let Some(ref kind) = exclude.kind {
                    m.kind.to_string() != *kind
                } else if let Some(ring) = exclude.ring {
                    m.ring != ring
                } else {
                    true
                };
                keep
            });
        }
        monads
    }
}

fn match_glob(name: &str, pattern: &str) -> bool {
    Pattern::new(pattern)
        .map(|p| p.matches(name))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::polar_space::PolarCoord;
    use crate::ontology::monad::Monad;
    use surrealdb::engine::local::Mem;

    fn make_test_monad(name: &str, ring: u32, theta: f64) -> Monad {
        Monad::spawn(
            format!("hash_{}", name),
            name.to_string(),
            PolarCoord::new(ring as f64 * 100.0, theta),
            format!("fn {}() {{ }}", name),
            ring,
        )
    }

    #[tokio::test]
    async fn test_exclusion_by_pattern() {
        let db = Surreal::new::<Mem>(()).await.unwrap();
        db.use_ns("mandala").use_db("weaver").await.unwrap();
        
        let monads = vec![
            make_test_monad("foo", 1, 45.0),
            make_test_monad("test_foo", 1, 90.0),
        ];
        let exclude = vec![ExcludeRule {
            pattern: Some("test_*".to_string()),
            ..Default::default()
        }];
        let engine = TemplateEngine::new(db);
        let result = engine.apply_exclusions(monads, &exclude);
        assert_eq!(result.len(), 1);
    }
}