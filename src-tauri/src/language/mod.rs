use serde::Serialize;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
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
    Scala,
    Elixir,
    Haskell,
    Solidity,
    Nix,
    Dart,
    Zig,
    Julia,
    R,
    Perl,
    OCaml,
    Erlang,
    Clojure,
    GraphQL,
    Protobuf,
    Terraform,
    Shader,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
pub struct LanguageSupportInfo {
    pub id: &'static str,
    pub display_name: &'static str,
    pub extensions: &'static [&'static str],
    pub engine: &'static str,
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
            "scala" | "sc" => Language::Scala,
            "ex" | "exs" | "elixir" => Language::Elixir,
            "hs" | "lhs" | "haskell" => Language::Haskell,
            "sol" | "solidity" => Language::Solidity,
            "nix" => Language::Nix,
            "dart" => Language::Dart,
            "zig" => Language::Zig,
            "jl" | "julia" => Language::Julia,
            "r" | "rmd" => Language::R,
            "pl" | "pm" | "perl" => Language::Perl,
            "ml" | "mli" | "fs" | "fsx" | "ocaml" => Language::OCaml,
            "erl" | "hrl" | "erlang" => Language::Erlang,
            "clj" | "cljs" | "cljc" | "edn" | "clojure" => Language::Clojure,
            "graphql" | "gql" => Language::GraphQL,
            "proto" | "protobuf" => Language::Protobuf,
            "tf" | "hcl" | "terraform" => Language::Terraform,
            "wgsl" | "glsl" | "hlsl" | "vert" | "frag" | "comp" => Language::Shader,
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
            if content.contains("elixir") {
                return Language::Elixir;
            }
            if content.contains("perl") {
                return Language::Perl;
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
        if content.contains("defmodule ") && content.contains(" do") {
            return Language::Elixir;
        }
        if content.contains("pragma solidity") || content.contains("contract ") {
            return Language::Solidity;
        }
        Language::Unknown
    }

    /// Extensiones de código fuente principal para escaneo automático de proyectos (39 lenguajes)
    pub fn code_extensions() -> &'static [&'static str] {
        &[
            "rs", "ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs",
            "py", "pyi", "go", "c", "h", "cpp", "cc", "cxx", "hpp", "hh",
            "java", "cs", "kt", "kts", "swift", "rb", "php", "lua",
            "sh", "bash", "astro", "vue", "svelte", "css", "scss", "sql",
            "scala", "sc", "ex", "exs", "hs", "lhs", "sol", "nix",
            "dart", "zig", "jl", "r", "pl", "pm", "ml", "mli", "fs",
            "erl", "hrl", "clj", "cljs", "cljc", "graphql", "gql",
            "proto", "tf", "hcl", "wgsl", "glsl", "hlsl",
        ]
    }

    pub fn all_languages() -> &'static [Language] {
        &[
            Language::Rust,
            Language::TypeScript,
            Language::JavaScript,
            Language::Python,
            Language::Go,
            Language::C,
            Language::Cpp,
            Language::Java,
            Language::CSharp,
            Language::Kotlin,
            Language::Swift,
            Language::Ruby,
            Language::Php,
            Language::Lua,
            Language::Shell,
            Language::Html,
            Language::Css,
            Language::Json,
            Language::Yaml,
            Language::Toml,
            Language::Sql,
            Language::Markdown,
            Language::Scala,
            Language::Elixir,
            Language::Haskell,
            Language::Solidity,
            Language::Nix,
            Language::Dart,
            Language::Zig,
            Language::Julia,
            Language::R,
            Language::Perl,
            Language::OCaml,
            Language::Erlang,
            Language::Clojure,
            Language::GraphQL,
            Language::Protobuf,
            Language::Terraform,
            Language::Shader,
        ]
    }

    pub fn all_supported_info() -> Vec<LanguageSupportInfo> {
        Self::all_languages()
            .iter()
            .map(|lang| LanguageSupportInfo {
                id: lang.tree_sitter_name(),
                display_name: lang.display_name(),
                extensions: lang.extensions(),
                engine: if lang.ast_grep_lang().is_some() {
                    "tree-sitter (ast-grep)"
                } else {
                    "polyglot-structural-parser"
                },
            })
            .collect()
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
            Language::Scala => &["scala", "sc"],
            Language::Elixir => &["ex", "exs"],
            Language::Haskell => &["hs", "lhs"],
            Language::Solidity => &["sol"],
            Language::Nix => &["nix"],
            Language::Dart => &["dart"],
            Language::Zig => &["zig"],
            Language::Julia => &["jl"],
            Language::R => &["r", "rmd"],
            Language::Perl => &["pl", "pm"],
            Language::OCaml => &["ml", "mli", "fs", "fsx"],
            Language::Erlang => &["erl", "hrl"],
            Language::Clojure => &["clj", "cljs", "cljc", "edn"],
            Language::GraphQL => &["graphql", "gql"],
            Language::Protobuf => &["proto"],
            Language::Terraform => &["tf", "hcl"],
            Language::Shader => &["wgsl", "glsl", "hlsl", "vert", "frag", "comp"],
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
            Language::Scala => "scala",
            Language::Elixir => "elixir",
            Language::Haskell => "haskell",
            Language::Solidity => "solidity",
            Language::Nix => "nix",
            Language::Dart => "dart",
            Language::Zig => "zig",
            Language::Julia => "julia",
            Language::R => "r",
            Language::Perl => "perl",
            Language::OCaml => "ocaml",
            Language::Erlang => "erlang",
            Language::Clojure => "clojure",
            Language::GraphQL => "graphql",
            Language::Protobuf => "protobuf",
            Language::Terraform => "terraform",
            Language::Shader => "wgsl",
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
            Language::Scala => "Scala",
            Language::Elixir => "Elixir",
            Language::Haskell => "Haskell",
            Language::Solidity => "Solidity",
            Language::Nix => "Nix",
            Language::Dart => "Dart",
            Language::Zig => "Zig",
            Language::Julia => "Julia",
            Language::R => "R",
            Language::Perl => "Perl",
            Language::OCaml => "OCaml/F#",
            Language::Erlang => "Erlang",
            Language::Clojure => "Clojure",
            Language::GraphQL => "GraphQL",
            Language::Protobuf => "Protocol Buffers",
            Language::Terraform => "Terraform/HCL",
            Language::Shader => "WGSL/GLSL Shader",
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
            Language::CSharp => ast_grep_language::SupportLang::from_str("csharp")
                .ok()
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
            Language::Scala => ast_grep_language::SupportLang::from_str("scala").ok(),
            Language::Elixir => ast_grep_language::SupportLang::from_str("elixir").ok(),
            Language::Haskell => ast_grep_language::SupportLang::from_str("haskell").ok(),
            Language::Solidity => ast_grep_language::SupportLang::from_str("solidity").ok(),
            Language::Nix => ast_grep_language::SupportLang::from_str("nix").ok(),
            _ => None,
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
        assert_eq!(detect_language(Path::new("Actor.scala")), Language::Scala);
        assert_eq!(detect_language(Path::new("worker.ex")), Language::Elixir);
        assert_eq!(detect_language(Path::new("Parser.hs")), Language::Haskell);
        assert_eq!(detect_language(Path::new("Vault.sol")), Language::Solidity);
        assert_eq!(detect_language(Path::new("flake.nix")), Language::Nix);
        assert_eq!(detect_language(Path::new("main.zig")), Language::Zig);
        assert_eq!(detect_language(Path::new("app.dart")), Language::Dart);
        assert_eq!(detect_language(Path::new("schema.graphql")), Language::GraphQL);
        assert_eq!(detect_language(Path::new("service.proto")), Language::Protobuf);
        assert_eq!(detect_language(Path::new("main.tf")), Language::Terraform);
        assert_eq!(detect_language(Path::new("shader.wgsl")), Language::Shader);
        assert!(Language::all_supported_info().len() >= 38);
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
        assert_eq!(Language::Solidity.display_name(), "Solidity");
        assert_eq!(Language::Elixir.display_name(), "Elixir");
    }

    #[test]
    fn test_language_tree_sitter_names() {
        assert_eq!(Language::Rust.tree_sitter_name(), "rust");
        assert_eq!(Language::TypeScript.tree_sitter_name(), "typescript");
        assert_eq!(Language::JavaScript.tree_sitter_name(), "javascript");
        assert_eq!(Language::Python.tree_sitter_name(), "python");
        assert_eq!(Language::Go.tree_sitter_name(), "go");
        assert_eq!(Language::Scala.tree_sitter_name(), "scala");
        assert_eq!(Language::Haskell.tree_sitter_name(), "haskell");
    }
}