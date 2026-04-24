use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Unknown,
}

impl Language {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Language::Rust,
            "ts" | "tsx" => Language::TypeScript,
            "js" | "jsx" => Language::JavaScript,
            "py" => Language::Python,
            "go" => Language::Go,
            _ => Language::Unknown,
        }
    }

    pub fn from_content(content: &str) -> Self {
        if content.starts_with("#!") {
            if content.contains("python") {
                return Language::Python;
            }
            if content.contains("node") {
                return Language::JavaScript;
            }
        }
        if content.contains(": React.FC") || content.contains("React.ReactElement") {
            return Language::TypeScript;
        }
        Language::Unknown
    }

    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Language::Rust => &["rs"],
            Language::TypeScript => &["ts", "tsx"],
            Language::JavaScript => &["js", "jsx"],
            Language::Python => &["py"],
            Language::Go => &["go"],
            Language::Unknown => &[],
        }
    }

    pub fn tree_sitter_name(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::TypeScript => "typescript",
            Language::JavaScript => "javascript",
            Language::Python => "python",
            Language::Go => "go",
            Language::Unknown => "",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::TypeScript => "TypeScript",
            Language::JavaScript => "JavaScript",
            Language::Python => "Python",
            Language::Go => "Go",
            Language::Unknown => "Unknown",
        }
    }

    pub fn ast_grep_lang(&self) -> Option<ast_grep_language::SupportLang> {
        match self {
            Language::Rust => Some(ast_grep_language::SupportLang::Rust),
            Language::TypeScript => Some(ast_grep_language::SupportLang::TypeScript),
            Language::JavaScript => Some(ast_grep_language::SupportLang::JavaScript),
            Language::Python => Some(ast_grep_language::SupportLang::Python),
            Language::Go => Some(ast_grep_language::SupportLang::Go),
            Language::Unknown => None,
        }
    }
}

pub fn detect_language(path: &Path) -> Language {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    
    // Active use of LanguagePlugin metadata
    if let Some(plugin) = crate::plugins::LanguagePlugin::from_extension(ext) {
        println!("  System: Handled by {} plugin infrastructure", plugin.name());
    }
    
    let lang = Language::from_extension(ext);
    
    if lang != Language::Unknown {
        return lang;
    }
    
    // Fallback to content-based detection
    if let Ok(content) = std::fs::read_to_string(path) {
        let content_lang = Language::from_content(&content);
        if content_lang != Language::Unknown {
            return content_lang;
        }
    }
    
    Language::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_rust() {
        assert_eq!(detect_language(Path::new("src/main.rs")), Language::Rust);
    }

    #[test]
    fn test_detect_typescript() {
        assert_eq!(detect_language(Path::new("component.tsx")), Language::TypeScript);
    }

    #[test]
    fn test_detect_javascript() {
        assert_eq!(detect_language(Path::new("index.js")), Language::JavaScript);
    }

    #[test]
    fn test_detect_python() {
        assert_eq!(detect_language(Path::new("script.py")), Language::Python);
    }

    #[test]
    fn test_detect_go() {
        assert_eq!(detect_language(Path::new("main.go")), Language::Go);
    }

    #[test]
    fn test_unknown_extension() {
        assert_eq!(detect_language(Path::new("file.txt")), Language::Unknown);
    }

    #[test]
    fn test_language_display_names() {
        assert_eq!(Language::Rust.display_name(), "Rust");
        assert_eq!(Language::TypeScript.display_name(), "TypeScript");
        assert_eq!(Language::Python.display_name(), "Python");
    }

    #[test]
    fn test_language_tree_sitter_names() {
        assert_eq!(Language::Rust.tree_sitter_name(), "rust");
        assert_eq!(Language::TypeScript.tree_sitter_name(), "typescript");
        assert_eq!(Language::JavaScript.tree_sitter_name(), "javascript");
        assert_eq!(Language::Python.tree_sitter_name(), "python");
        assert_eq!(Language::Go.tree_sitter_name(), "go");
    }
}