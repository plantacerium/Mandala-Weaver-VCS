// Pre-implementation: File system writer
// ===============================

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::ontology::monad::Monad;
use crate::weaver::source_compiler::distill_source;

/// Output directory structure
#[derive(Debug, Clone)]
pub enum OutputStructure {
    Flat,
    VectorDirs,
    RingDirs,
    Nested,
}

/// File system writer for distilled monads
pub struct FileWriter;

impl FileWriter {
    /// Writes distilled monads to disk with directory structure
    pub async fn write_to_disk(
        monads: &[Monad],
        output_dir: &Path,
        structure: OutputStructure,
    ) -> anyhow::Result<Vec<PathBuf>> {
        let mut written = Vec::new();
        
        let content = distill_source(monads);
        
        match structure {
            OutputStructure::Flat => {
                let path = output_dir.join("main.rs");
                std::fs::write(&path, &content)?;
                written.push(path);
            }
            OutputStructure::VectorDirs => {
                let groups = Self::group_by_theta(monads);
                for (theta, group) in groups {
                    let dir_name = format!("vector_{:.0}", theta);
                    let path = output_dir.join(dir_name).join("mod.rs");
                    std::fs::create_dir_all(path.parent().unwrap())?;
                    let content = distill_source(group);
                    std::fs::write(&path, &content)?;
                    written.push(path);
                }
            }
            OutputStructure::RingDirs => {
                let groups = Self::group_by_ring(monads);
                for (ring, group) in groups {
                    let dir_name = format!("ring_{}", ring);
                    let path = output_dir.join(dir_name).join("mod.rs");
                    std::fs::create_dir_all(path.parent().unwrap())?;
                    let content = distill_source(group);
                    std::fs::write(&path, &content)?;
                    written.push(path);
                }
            }
            OutputStructure::Nested => {
                let ring_groups = Self::group_by_ring(monads);
                for (ring, ring_monads) in ring_groups {
                    let vector_groups = Self::group_by_theta(&ring_monads);
                    for (theta, group) in vector_groups {
                        let dir_name = format!("ring_{}/vector_{:.0}", ring, theta);
                        let path = output_dir.join(dir_name).join("mod.rs");
                        std::fs::create_dir_all(path.parent().unwrap())?;
                        let content = distill_source(group);
                        std::fs::write(&path, &content)?;
                        written.push(path);
                    }
                }
            }
        }
        
        Ok(written)
    }
    
    fn group_by_theta(monads: &[Monad]) -> HashMap<f64, Vec<&Monad> {
        let mut groups: HashMap<f64, Vec<&Monad>> = HashMap::new();
        for m in monads {
            let sector = (m.coord.theta / 90.0).floor() * 90.0;
            groups.entry(sector).or_default().push(m);
        }
        groups
    }
    
    fn group_by_ring(monads: &[Monad]) -> HashMap<u32, Vec<&Monad>> {
        let mut groups: HashMap<u32, Vec<&Monad>> = HashMap::new();
        for m in monads {
            groups.entry(m.ring).or_default().push(m);
        }
        groups
    }
    
    /// Write main.rs with module declarations
    pub fn write modules(monads: &[Monad], output_dir: &Path) -> anyhow::Result<PathBuf> {
        let mut modules = String::new();
        
        let rings = Self::group_by_ring(monads);
        let mut ring_keys: Vec<_> = rings.keys().collect();
        ring_keys.sort();
        
        for ring in ring_keys {
            modules.push_str(&format!("pub mod ring_{};\n", ring));
        }
        
        modules.push_str("\n// Module tree\n");
        for ring in ring_keys {
            modules.push_str(&format!("pub mod ring_{};\n", ring));
        }
        
        let path = output_dir.join("modules.rs");
        std::fs::write(&path, &modules)?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::polar_space::PolarCoord;
    use crate::ontology::monad::Monad;

    #[test]
    fn test_group_by_ring() {
        let monads = vec![
            Monad::spawn("1".to_string(), "a".to_string(), PolarCoord::new(1.0, 0.0), "fn a() {}".to_string(), 1),
            Monad::spawn("2".to_string(), "b".to_string(), PolarCoord::new(1.0, 90.0), "fn b() {}".to_string(), 2),
            Monad::spawn("3".to_string(), "c".to_string(), PolarCoord::new(1.0, 180.0), "fn c() {}".to_string(), 1),
        ];
        
        let groups = FileWriter::group_by_ring(&monads);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get(&1).unwrap().len(), 2);
        assert_eq!(groups.get(&2).unwrap().len(), 1);
    }
}