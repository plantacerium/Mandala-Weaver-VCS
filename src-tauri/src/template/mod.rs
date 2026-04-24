use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationTemplate {
    pub name: String,
    pub version: String,
    pub rings: RingSelector,
    pub vectors: Vec<VectorSelector>,
    pub exclude: Vec<ExcludeRule>,
    pub adapters: Vec<AdapterConfig>,
    pub output: OutputConfig,
    #[serde(default)]
    pub variables: HashMap<String, VariableDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDef {
    pub description: String,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum RingSelector {
    Level(u32),
    Latest,
    Range { min: u32, max: u32 },
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSelector {
    pub name: String,
    #[serde(default)]
    pub angle_start: Option<f64>,
    #[serde(default)]
    pub angle_end: Option<f64>,
    #[serde(default)]
    pub ring: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExcludeRule {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub ring: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    pub name: String,
    pub from: String,
    pub to: String,
    pub adapter_type: AdapterType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum AdapterType {
    Import { module: String },
    Alias { new_name: String },
    Wrapper { before: String, after: String },
    TypeConvert { from: String, to: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub filename: String,
    pub structure: OutputStructure,
    #[serde(default = "default_true")]
    pub generate_modules: bool,
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum OutputStructure {
    Flat,
    VectorDirs,
    RingDirs,
    Nested { rings: bool, vectors: bool },
}

pub mod adapter;
pub mod engine;

pub use adapter::AdapterEngine;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_template() {
        let yaml = r#"
name: test
version: "0.1.0"
rings:
  type: Latest
vectors:
  - name: CORE
exclude: []
adapters: []
output:
  filename: "test.rs"
  structure: Flat
"#;
        let template: DistillationTemplate = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(template.name, "test");
        assert_eq!(template.version, "0.1.0");
    }

    #[test]
    fn test_parse_ring_selector() {
        let yaml = r#"
name: test
version: "0.1.0"
rings:
  type: Range
  min: 1
  max: 5
vectors: []
exclude: []
adapters: []
output:
  filename: "out.rs"
  structure: Flat
"#;
        let template: DistillationTemplate = serde_yaml::from_str(yaml).unwrap();
        if let RingSelector::Range { min, max } = template.rings {
            assert_eq!(min, 1);
            assert_eq!(max, 5);
        } else {
            panic!("Expected Range variant");
        }
    }
}