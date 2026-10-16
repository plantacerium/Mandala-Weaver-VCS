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

    pub fn by_name(&self, name: &str) -> Option<&dyn MandalaPlugin> {
        self.plugins.iter().find(|p| p.name() == name).map(|p| p.as_ref())
    }

    /// Initialize with built-in production plugins
    pub fn init() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(PolyglotAstPlugin));
        registry.register(Box::new(ContractTopologyPlugin));
        registry
    }
}

/// Built-in Polyglot AST Extractor & Polar Constellation Renderer Plugin
pub struct PolyglotAstPlugin;

impl MandalaPlugin for PolyglotAstPlugin {
    fn name(&self) -> &str {
        "polyglot-ast-extractor"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "Tree-sitter / ast-grep polyglot monad extractor and SVG polar constellation renderer"
    }

    fn extract(&self, source: &str, language: &str) -> Vec<Monad> {
        crate::weaver::ast_extractor::extract_raw_monads_lang(source, 1, language)
    }

    fn render(&self, monads: &[Monad]) -> RenderOutput {
        let mut svg = String::from(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 240 240" width="240" height="240">"#,
        );
        svg.push_str(r##"<circle cx="120" cy="120" r="6" fill="#00e5ff" />"##);

        for m in monads.iter().take(64) {
            let r = ((m.ring as f64) * 22.0).min(105.0);
            let rad = m.coord.theta.to_radians();
            let cx = 120.0 + r * rad.cos();
            let cy = 120.0 + r * rad.sin();
            svg.push_str(&format!(
                r##"<circle cx="{:.1}" cy="{:.1}" r="3.5" fill="#9d4edd" data-monad="{}" />"##,
                cx, cy, m.name
            ));
        }

        svg.push_str("</svg>");
        RenderOutput {
            svg,
            interactive: true,
        }
    }
}

/// Built-in Contract & Port Topology Analyzer Plugin
pub struct ContractTopologyPlugin;

impl MandalaPlugin for ContractTopologyPlugin {
    fn name(&self) -> &str {
        "contract-topology-weaver"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "Analyzes Merkle-AST contracts, outbound port dependencies, and SO(2) sector density"
    }

    fn extract(&self, source: &str, language: &str) -> Vec<Monad> {
        crate::weaver::ast_extractor::extract_raw_monads_lang(source, 1, language)
            .into_iter()
            .filter(|m| !m.contract_hash.is_empty() || !m.outbound_calls.is_empty())
            .collect()
    }

    fn render(&self, monads: &[Monad]) -> RenderOutput {
        let max_ring = monads.iter().map(|m| m.ring).max().unwrap_or(1).max(1);
        let mut svg = String::from(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 240 240" width="240" height="240">"#,
        );
        for ring in 1..=max_ring.min(5) {
            let radius = (ring as f64) * 20.0;
            svg.push_str(&format!(
                r#"<circle cx="120" cy="120" r="{:.1}" fill="none" stroke="rgba(0,229,255,0.25)" stroke-width="1" />"#,
                radius
            ));
        }
        svg.push_str("</svg>");
        RenderOutput {
            svg,
            interactive: true,
        }
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in language plugins
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LanguagePlugin {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    C,
    Cpp,
    Java,
    CSharp,
    Kotlin,
    Swift,
    Ruby,
    Php,
    Lua,
    Shell,
    Html,
    Css,
    Json,
    Yaml,
    Toml,
    Sql,
    Markdown,
}

impl LanguagePlugin {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.trim_start_matches('.').to_lowercase().as_str() {
            "rs" => Some(Self::Rust),
            "ts" | "tsx" | "mts" | "cts" => Some(Self::TypeScript),
            "js" | "jsx" | "mjs" | "cjs" => Some(Self::JavaScript),
            "py" | "pyi" | "pyw" => Some(Self::Python),
            "go" => Some(Self::Go),
            "c" | "h" => Some(Self::C),
            "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => Some(Self::Cpp),
            "java" => Some(Self::Java),
            "cs" => Some(Self::CSharp),
            "kt" | "kts" => Some(Self::Kotlin),
            "swift" => Some(Self::Swift),
            "rb" => Some(Self::Ruby),
            "php" => Some(Self::Php),
            "lua" => Some(Self::Lua),
            "sh" | "bash" | "zsh" | "ps1" => Some(Self::Shell),
            "html" | "htm" | "astro" | "vue" | "svelte" => Some(Self::Html),
            "css" | "scss" | "sass" | "less" => Some(Self::Css),
            "json" | "jsonc" => Some(Self::Json),
            "yaml" | "yml" => Some(Self::Yaml),
            "toml" => Some(Self::Toml),
            "sql" => Some(Self::Sql),
            "md" | "mdx" => Some(Self::Markdown),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        self.name()
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
            Self::JavaScript => "javascript",
            Self::Python => "python",
            Self::Go => "go",
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::Java => "java",
            Self::CSharp => "csharp",
            Self::Kotlin => "kotlin",
            Self::Swift => "swift",
            Self::Ruby => "ruby",
            Self::Php => "php",
            Self::Lua => "lua",
            Self::Shell => "bash",
            Self::Html => "html",
            Self::Css => "css",
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Toml => "toml",
            Self::Sql => "sql",
            Self::Markdown => "markdown",
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
        assert_eq!(LanguagePlugin::from_extension("astro"), Some(LanguagePlugin::Html));
        assert_eq!(LanguagePlugin::from_extension("xyz"), None);

        let registry = PluginRegistry::init();
        assert_eq!(registry.get().len(), 2);
        let extracted = registry.get()[0].extract("fn demo() { let x = 1; }", "rust");
        assert_eq!(extracted.len(), 1);
    }
}