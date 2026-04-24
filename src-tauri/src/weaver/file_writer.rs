// File system writer for distilled monads
// ====================================
// Integrated with template OutputStructure from template module

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::ontology::monad::Monad;
use crate::template::{OutputStructure, AdapterConfig};
use crate::weaver::source_compiler::distill_source_with_adapters;

pub struct FileWriter;

impl FileWriter {
    pub async fn write_with_structure(
        monads: &[Monad],
        output_dir: &Path,
        structure: &OutputStructure,
        adapters: &[AdapterConfig],
    ) -> anyhow::Result<Vec<PathBuf>> {
        let mut written = Vec::new();
        let content = distill_source_with_adapters(monads, adapters);
        
        match structure {
            OutputStructure::Flat => {
                let path = output_dir.join("main.rs");
                std::fs::write(&path, &content)?;
                written.push(path);
            }
            OutputStructure::VectorDirs => {
                let groups = Self::group_by_theta(monads);
                for (theta, group_monads) in groups {
                    let dir_name = format!("vector_{:.0}", theta);
                    let path = output_dir.join(dir_name).join("mod.rs");
                    std::fs::create_dir_all(path.parent().unwrap())?;
                    let content = distill_source_with_adapters(&group_monads, adapters);
                    std::fs::write(&path, &content)?;
                    written.push(path);
                }
            }
            OutputStructure::RingDirs => {
                let groups = Self::group_by_ring(monads);
                for (ring, group_monads) in groups {
                    let dir_name = format!("ring_{}", ring);
                    let path = output_dir.join(dir_name).join("mod.rs");
                    std::fs::create_dir_all(path.parent().unwrap())?;
                    let content = distill_source_with_adapters(&group_monads, adapters);
                    std::fs::write(&path, &content)?;
                    written.push(path);
                }
            }
            OutputStructure::Nested { rings: _, vectors: _ } => {
                let ring_groups = Self::group_by_ring(monads);
                for (ring, ring_monads) in ring_groups {
                    let vector_groups = Self::group_by_theta(&ring_monads);
                    for (theta, group_monads) in vector_groups {
                        let dir_name = format!("ring_{}/vector_{:.0}", ring, theta);
                        let path = output_dir.join(dir_name).join("mod.rs");
                        std::fs::create_dir_all(path.parent().unwrap())?;
                        let content = distill_source_with_adapters(&group_monads, adapters);
                        std::fs::write(&path, &content)?;
                        written.push(path);
                    }
                }
            }
        }
        
        Ok(written)
    }
    
    fn group_by_theta(monads: &[Monad]) -> HashMap<i32, Vec<Monad>> {
        let mut groups: HashMap<i32, Vec<Monad>> = HashMap::new();
        for m in monads {
            let sector = ((m.coord.theta / 90.0).floor() as i32) * 90;
            groups.entry(sector).or_default().push(m.clone());
        }
        groups
    }
    
    fn group_by_ring(monads: &[Monad]) -> HashMap<u32, Vec<Monad>> {
        let mut groups: HashMap<u32, Vec<Monad>> = HashMap::new();
        for m in monads {
            groups.entry(m.ring).or_default().push(m.clone());
        }
        groups
    }
    
    pub fn write_modules(monads: &[Monad], output_dir: &Path) -> anyhow::Result<PathBuf> {
        let rings = Self::group_by_ring(monads);
        let mut ring_keys: Vec<_> = rings.keys().collect();
        ring_keys.sort();
        
        let mut modules = String::new();
        for ring in &ring_keys {
            modules.push_str(&format!("pub mod ring_{};\n", ring));
        }
        
        modules.push_str("\n// Module tree\n");
        for ring in &ring_keys {
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
            Monad::spawn("2".to_string(), "b".to_string(), PolarCoord::new(1.0, 90.0), "fn b() {}", 2),
            Monad::spawn("3".to_string(), "c".to_string(), PolarCoord::new(1.0, 180.0), "fn c() {}", 1),
        ];
        
        let groups = FileWriter::group_by_ring(&monads);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get(&1).unwrap().len(), 2);
        assert_eq!(groups.get(&2).unwrap().len(), 1);
    }
}