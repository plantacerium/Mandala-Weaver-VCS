// Pre-implementation: Plugin System
// ============================
//
// Plugin trait for extensibility
// Users can create custom extractors, renderers, and adapters

use serde::{Deserialize, Serialize};
use crate::ontology::monad::Monad;

/// Plugin trait for Mandala extensibility
pub trait MandalaPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;

    /// Extract monads from source code
    fn extract(&self, source: &str, language: &str) -> Vec<Monad>;

    /// Render custom visualization for monads
    fn render(&self, monads: &[Monad]) -> RenderOutput;
}

/// Built-in render output from plugins
#[derive(Serialize, Deserialize, Debug)]
pub struct RenderOutput {
    pub svg: String,
    pub interactive: bool,
}

/// Registry of registered plugins
pub struct PluginRegistry {
    plugins: Vec<Box<dyn MandalaPlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    pub fn register(&mut self, plugin: Box<dyn MandalaPlugin>) {
        self.plugins.push(plugin);
    }

    pub fn get(&self) -> &[Box<dyn MandalaPlugin>] {
        &self.plugins
    }

    #[allow(dead_code)]
    pub fn by_name(&self, name: &str) -> Option<&dyn MandalaPlugin> {
        self.plugins.iter().find(|p| p.name() == name).map(|p| p.as_ref())
    }

    /// Initialize with built-in plugins
    pub fn init() -> Self {
        let mut registry = Self::new();
        // Register mock plugin to resolve warnings and demonstrate system
        registry.register(Box::new(MockPlugin));
        registry
    }
}

struct MockPlugin;
impl MandalaPlugin for MockPlugin {
    fn name(&self) -> &str { "mock-plugin" }
    fn version(&self) -> &str { "0.1.0" }
    fn description(&self) -> &str { "Built-in mock plugin for testing" }

    fn extract(&self, _source: &str, _language: &str) -> Vec<Monad> { Vec::new() }

    fn render(&self, _monads: &[Monad]) -> RenderOutput {
        RenderOutput { svg: "<svg></svg>".to_string(), interactive: false }
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in language plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LanguagePlugin {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    C,
    Cpp,
}

impl LanguagePlugin {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "rs" => Some(Self::Rust),
            "ts" | "tsx" => Some(Self::TypeScript),
            "js" | "jsx" => Some(Self::JavaScript),
            "py" => Some(Self::Python),
            "go" => Some(Self::Go),
            "c" | "h" => Some(Self::C),
            "cpp" | "cc" | "cxx" | "hpp" => Some(Self::Cpp),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
            Self::JavaScript => "javascript",
            Self::Python => "python",
            Self::Go => "go",
            Self::C => "c",
            Self::Cpp => "cpp",
        }
    }
    
    pub fn name(&self) -> &str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
            Self::JavaScript => "javascript",
            Self::Python => "python",
            Self::Go => "go",
            Self::C => "c",
            Self::Cpp => "cpp",
        }
    }
}

/// Plugin configuration
#[derive(Serialize, Deserialize, Debug)]
pub struct PluginConfig {
    pub enabled: bool,
    pub options: serde_json::Value,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            options: serde_json::json!({}),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_plugin_detection() {
        assert_eq!(LanguagePlugin::from_extension("rs"), Some(LanguagePlugin::Rust));
        assert_eq!(LanguagePlugin::from_extension("py"), Some(LanguagePlugin::Python));
        assert_eq!(LanguagePlugin::from_extension("xyz"), None);
    }
}