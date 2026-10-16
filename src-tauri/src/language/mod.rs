use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
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
    Unknown,
}

impl Language {
    pub fn from_extension(ext: &str) -> Self {
        match ext.trim_start_matches('.').to_lowercase().as_str() {
            "rs" | "rust" => Language::Rust,
            "ts" | "tsx" | "mts" | "cts" | "typescript" => Language::TypeScript,
            "js" | "jsx" | "mjs" | "cjs" | "javascript" => Language::JavaScript,
            "py" | "pyi" | "pyw" | "python" => Language::Python,
            "go" | "golang" => Language::Go,
            "c" | "h" => Language::C,
            "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" | "c++" => Language::Cpp,
            "java" => Language::Java,
            "cs" | "csharp" | "c#" => Language::CSharp,
            "kt" | "kts" | "kotlin" => Language::Kotlin,
            "swift" => Language::Swift,
            "rb" | "ruby" => Language::Ruby,
            "php" => Language::Php,
            "lua" => Language::Lua,
            "sh" | "bash" | "zsh" | "ps1" | "shell" => Language::Shell,
            "html" | "htm" | "astro" | "vue" | "svelte" => Language::Html,
            "css" | "scss" | "sass" | "less" => Language::Css,
            "json" | "jsonc" => Language::Json,
            "yaml" | "yml" => Language::Yaml,
            "toml" => Language::Toml,
            "sql" => Language::Sql,
            "md" | "mdx" | "markdown" => Language::Markdown,
            _ => Language::Unknown,
        }
    }

    pub fn from_content(content: &str) -> Self {
        if content.starts_with("#!") {
            if content.contains("python") {
                return Language::Python;
            }
            if content.contains("node") || content.contains("deno") || content.contains("bun") {
                return Language::JavaScript;
            }
            if content.contains("sh") || content.contains("bash") || content.contains("zsh") {
                return Language::Shell;
            }
            if content.contains("ruby") {
                return Language::Ruby;
            }
        }
        if content.contains(": React.FC") || content.contains("React.ReactElement") || content.contains("interface ") {
            return Language::TypeScript;
        }
        if content.contains("fn ") && (content.contains("pub fn") || content.contains("impl ")) {
            return Language::Rust;
        }
        if content.contains("package ") && content.contains("func ") {
            return Language::Go;
        }
        Language::Unknown
    }

    /// Extensiones de código fuente principal para escaneo automático de proyectos
    pub fn code_extensions() -> &'static [&'static str] {
        &[
            "rs", "ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs",
            "py", "pyi", "go", "c", "h", "cpp", "cc", "cxx", "hpp", "hh",
            "java", "cs", "kt", "kts", "swift", "rb", "php", "lua",
            "sh", "bash", "astro", "vue", "svelte", "css", "scss", "sql",
        ]
    }

    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Language::Rust => &["rs"],
            Language::TypeScript => &["ts", "tsx", "mts", "cts"],
            Language::JavaScript => &["js", "jsx", "mjs", "cjs"],
            Language::Python => &["py", "pyi", "pyw"],
            Language::Go => &["go"],
            Language::C => &["c", "h"],
            Language::Cpp => &["cpp", "cc", "cxx", "hpp", "hh", "hxx"],
            Language::Java => &["java"],
            Language::CSharp => &["cs"],
            Language::Kotlin => &["kt", "kts"],
            Language::Swift => &["swift"],
            Language::Ruby => &["rb"],
            Language::Php => &["php"],
            Language::Lua => &["lua"],
            Language::Shell => &["sh", "bash", "zsh", "ps1"],
            Language::Html => &["html", "htm", "astro", "vue", "svelte"],
            Language::Css => &["css", "scss", "sass", "less"],
            Language::Json => &["json", "jsonc"],
            Language::Yaml => &["yaml", "yml"],
            Language::Toml => &["toml"],
            Language::Sql => &["sql"],
            Language::Markdown => &["md", "mdx"],
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
            Language::C => "c",
            Language::Cpp => "cpp",
            Language::Java => "java",
            Language::CSharp => "csharp",
            Language::Kotlin => "kotlin",
            Language::Swift => "swift",
            Language::Ruby => "ruby",
            Language::Php => "php",
            Language::Lua => "lua",
            Language::Shell => "bash",
            Language::Html => "html",
            Language::Css => "css",
            Language::Json => "json",
            Language::Yaml => "yaml",
            Language::Toml => "toml",
            Language::Sql => "sql",
            Language::Markdown => "markdown",
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
            Language::C => "C",
            Language::Cpp => "C++",
            Language::Java => "Java",
            Language::CSharp => "C#",
            Language::Kotlin => "Kotlin",
            Language::Swift => "Swift",
            Language::Ruby => "Ruby",
            Language::Php => "PHP",
            Language::Lua => "Lua",
            Language::Shell => "Shell",
            Language::Html => "HTML/Component",
            Language::Css => "CSS/Styles",
            Language::Json => "JSON",
            Language::Yaml => "YAML",
            Language::Toml => "TOML",
            Language::Sql => "SQL",
            Language::Markdown => "Markdown",
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
            Language::C => ast_grep_language::SupportLang::from_str("c").ok(),
            Language::Cpp => ast_grep_language::SupportLang::from_str("cpp").ok(),
            Language::Java => ast_grep_language::SupportLang::from_str("java").ok(),
            Language::CSharp => ast_grep_language::SupportLang::from_str("csharp").ok()
                .or_else(|| ast_grep_language::SupportLang::from_str("cs").ok()),
            Language::Kotlin => ast_grep_language::SupportLang::from_str("kotlin").ok(),
            Language::Swift => ast_grep_language::SupportLang::from_str("swift").ok(),
            Language::Ruby => ast_grep_language::SupportLang::from_str("ruby").ok(),
            Language::Php => ast_grep_language::SupportLang::from_str("php").ok(),
            Language::Lua => ast_grep_language::SupportLang::from_str("lua").ok(),
            Language::Shell => ast_grep_language::SupportLang::from_str("bash").ok(),
            Language::Html => ast_grep_language::SupportLang::from_str("html").ok(),
            Language::Css => ast_grep_language::SupportLang::from_str("css").ok(),
            Language::Json => ast_grep_language::SupportLang::from_str("json").ok(),
            Language::Yaml => ast_grep_language::SupportLang::from_str("yaml").ok(),
            Language::Sql => ast_grep_language::SupportLang::from_str("sql").ok(),
            Language::Toml | Language::Markdown | Language::Unknown => None,
        }
    }
}

pub fn detect_language(path: &Path) -> Language {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let lang = Language::from_extension(ext);

    if lang != Language::Unknown {
        return lang;
    }

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
        assert_eq!(detect_language(Path::new("module.mts")), Language::TypeScript);
    }

    #[test]
    fn test_detect_javascript() {
        assert_eq!(detect_language(Path::new("index.js")), Language::JavaScript);
        assert_eq!(detect_language(Path::new("config.mjs")), Language::JavaScript);
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
    fn test_detect_extended_languages() {
        assert_eq!(detect_language(Path::new("engine.cpp")), Language::Cpp);
        assert_eq!(detect_language(Path::new("App.java")), Language::Java);
        assert_eq!(detect_language(Path::new("Program.cs")), Language::CSharp);
        assert_eq!(detect_language(Path::new("Layout.astro")), Language::Html);
        assert_eq!(detect_language(Path::new("styles.scss")), Language::Css);
        assert_eq!(detect_language(Path::new("schema.sql")), Language::Sql);
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
        assert_eq!(Language::Cpp.display_name(), "C++");
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