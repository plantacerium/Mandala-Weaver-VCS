use crate::ontology::monad::{Monad, MonadKind};
use crate::geometry::polar_space::PolarCoord;
use crate::ontology::semantic_hash::generate_pure_hash;
use ast_grep_core::AstGrep;
use ast_grep_core::tree_sitter::StrDoc;
use ast_grep_language::SupportLang;

/// Maps a tree-sitter node kind across supported languages to our MonadKind enum.
fn kind_from_node(node_kind: &str) -> Option<MonadKind> {
    match node_kind {
        // Functions & Methods (Rust, TS/JS, Python, Go, C/C++, Java, C#, Kotlin, Swift, Ruby, PHP, Lua, Bash, Scala, Haskell, Solidity, Elixir)
        "function_item"
        | "function_declaration"
        | "function_definition"
        | "async_function_definition"
        | "function_definition_statement"
        | "local_function_definition_statement"
        | "method_definition"
        | "method_declaration"
        | "constructor_declaration"
        | "init_declaration"
        | "generator_function_declaration"
        | "method"
        | "singleton_method"
        | "preproc_function_def"
        | "macro_definition"
        | "modifier_definition"
        | "function" => Some(MonadKind::Function),

        // Structs, Classes, Contracts, Records & Objects (Rust, TS/JS, Python, C/C++, Java, C#, Kotlin, Swift, Ruby, PHP, Scala, Solidity, CSS)
        "struct_item"
        | "struct_specifier"
        | "struct_declaration"
        | "class_declaration"
        | "abstract_class_declaration"
        | "class_definition"
        | "class_specifier"
        | "class"
        | "record_declaration"
        | "object_declaration"
        | "object_definition"
        | "contract_declaration"
        | "library_declaration"
        | "data_type"
        | "adt"
        | "newtype"
        | "rule_set" => Some(MonadKind::Struct),

        // Enums (Rust, TS, C/C++, Java, C#, Kotlin, Swift, Scala, Solidity)
        "enum_item" | "enum_declaration" | "enum_specifier" | "enum_definition" => {
            Some(MonadKind::Enum)
        }

        // Impls, Extensions & Instances (Rust, Swift, Haskell)
        "impl_item" | "extension_declaration" | "instance_decl" | "instance" => {
            Some(MonadKind::Impl)
        }

        // Traits, Interfaces, Protocols & Typeclasses (Rust, TS, Java, C#, Kotlin, Swift, PHP, Scala, Solidity, Haskell)
        "trait_item"
        | "trait_definition"
        | "interface_declaration"
        | "protocol_declaration"
        | "trait_declaration"
        | "class_decl" => Some(MonadKind::Trait),

        // Modules, Packages & Namespaces (Rust, TS, C++, C#, Ruby, Scala, CSS @media/@keyframes)
        "mod_item"
        | "internal_module"
        | "module"
        | "namespace_definition"
        | "namespace_declaration"
        | "package_clause"
        | "media_statement"
        | "keyframes_statement" => Some(MonadKind::Module),

        // Constants, State Variables & Top-level Lexical/Var/Val Declarations (Rust, TS/JS, Go, C/C++, Scala, Solidity, Nix)
        "const_item"
        | "static_item"
        | "lexical_declaration"
        | "variable_declaration"
        | "const_declaration"
        | "var_declaration"
        | "val_definition"
        | "var_definition"
        | "state_variable_declaration"
        | "event_definition"
        | "binding" => Some(MonadKind::Constant),

        // Type Aliases & Type Declarations (Rust, TS, Go, C/C++, Scala, Haskell)
        "type_item"
        | "type_alias_declaration"
        | "type_declaration"
        | "type_definition"
        | "type_synomym"
        | "type_alias" => Some(MonadKind::TypeAlias),

        _ => None,
    }
}

/// The list of top-level AST node kinds we want to extract as Monads across supported languages.
const EXTRACTABLE_KINDS: &[&str] = &[
    // Rust
    "function_item",
    "struct_item",
    "enum_item",
    "impl_item",
    "trait_item",
    "mod_item",
    "const_item",
    "static_item",
    "type_item",
    "macro_definition",
    // TypeScript / JavaScript
    "function_declaration",
    "generator_function_declaration",
    "class_declaration",
    "abstract_class_declaration",
    "interface_declaration",
    "type_alias_declaration",
    "enum_declaration",
    "method_definition",
    "lexical_declaration",
    // Python
    "function_definition",
    "async_function_definition",
    "class_definition",
    // Go
    "method_declaration",
    "type_declaration",
    "const_declaration",
    "var_declaration",
    // C / C++
    "struct_specifier",
    "enum_specifier",
    "class_specifier",
    "namespace_definition",
    "type_definition",
    "preproc_function_def",
    // Java / C# / Kotlin / Swift / Scala
    "record_declaration",
    "struct_declaration",
    "object_declaration",
    "object_definition",
    "trait_definition",
    "enum_definition",
    "val_definition",
    "var_definition",
    "protocol_declaration",
    "constructor_declaration",
    "init_declaration",
    "extension_declaration",
    "namespace_declaration",
    // Solidity / Haskell / Nix
    "contract_declaration",
    "library_declaration",
    "modifier_definition",
    "event_definition",
    "state_variable_declaration",
    "function",
    "data_type",
    "adt",
    "newtype",
    "type_synomym",
    "type_alias",
    "class_decl",
    "instance_decl",
    "binding",
    // Ruby / PHP / Lua / Bash
    "method",
    "singleton_method",
    "class",
    "module",
    "trait_declaration",
    "function_definition_statement",
    "local_function_definition_statement",
    // CSS / SCSS
    "rule_set",
    "keyframes_statement",
    "media_statement",
];

/// Extracts the identifier name directly from a tree-sitter AST node or falls back to text heuristics.
fn extract_name_from_node(
    node: &ast_grep_core::Node<'_, StrDoc<SupportLang>>,
    text: &str,
    kind: &MonadKind,
) -> String {
    // 1. Try standard tree-sitter field `name`
    if let Some(name_node) = node.field("name") {
        let n = name_node.text().trim().to_string();
        if !n.is_empty() {
            return n;
        }
    }

    // 2. For C/C++ function_definition, try `declarator` -> `declarator` / `identifier`
    if let Some(decl_node) = node.field("declarator") {
        if let Some(inner) = decl_node.field("declarator") {
            let n = inner.text().trim().to_string();
            if !n.is_empty() {
                return n.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == ':').collect();
            }
        }
        let n = decl_node.text().trim().to_string();
        let ident: String = n.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
        if !ident.is_empty() {
            return ident;
        }
    }

    // 3. For Rust `impl` blocks, try `type` field
    if let Some(type_node) = node.field("type") {
        let n = type_node.text().trim().to_string();
        if !n.is_empty() {
            return n;
        }
    }

    // 4. Look for child `variable_declarator`, `type_spec`, `const_spec`, `selectors`, or `identifier`
    for child in node.children() {
        let ck = child.kind();
        if ck == "variable_declarator" || ck == "type_spec" || ck == "const_spec" || ck == "var_spec" {
            if let Some(sub_name) = child.field("name") {
                let n = sub_name.text().trim().to_string();
                if !n.is_empty() {
                    return n;
                }
            }
            for sub in child.children() {
                if sub.kind() == "identifier" || sub.kind() == "type_identifier" {
                    let n = sub.text().trim().to_string();
                    if !n.is_empty() {
                        return n;
                    }
                }
            }
        } else if ck == "selectors" {
            let sel = child.text().trim().to_string();
            if !sel.is_empty() {
                return sel;
            }
        }
    }

    extract_name_from_text(text, kind)
}

/// Extracts the "name" from a tree-sitter node's text.
/// For `fn foo(...)`, returns "foo". For `struct Bar`, returns "Bar".
/// Falls back to a hash-prefix if no name identifier is found.
fn extract_name_from_text(text: &str, kind: &MonadKind) -> String {
    let prefixes: &[&str] = match kind {
        MonadKind::Function => &["fn ", "function ", "def ", "func ", "macro_rules! ", "sub "],
        MonadKind::Struct => &["struct ", "class ", "record ", "object "],
        MonadKind::Enum => &["enum "],
        MonadKind::Impl => &["impl "],
        MonadKind::Trait => &["trait ", "interface ", "protocol "],
        MonadKind::Module => &["mod ", "namespace ", "module ", "@keyframes ", "@media "],
        MonadKind::Constant => &["const ", "static ", "let ", "var "],
        MonadKind::TypeAlias => &["type ", "typedef "],
        MonadKind::Unknown => &[],
    };

    for prefix in prefixes {
        if let Some(after) = text.find(prefix) {
            let rest = &text[after + prefix.len()..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect();
            if !name.is_empty() {
                return name;
            }
        }
    }

    // CSS selector or first identifier fallback
    let first_line = text.lines().next().unwrap_or("").trim();
    if let Some(before_brace) = first_line.split('{').next() {
        let candidate = before_brace.trim();
        if !candidate.is_empty() && candidate.len() <= 40 {
            return candidate.to_string();
        }
    }

    let hash = generate_pure_hash(text);
    format!("anon_{}", &hash[..8])
}

/// Extracts all semantic code units (functions, structs, enums, impls, traits, etc.)
/// from a source string using ast-grep's tree-sitter integration.
///
/// Each extracted unit becomes a `Monad` positioned at an auto-assigned coordinate
/// within the specified ring.
pub fn extract_raw_monads(source_code: &str, ring: u32) -> Vec<Monad> {
    extract_raw_monads_lang(source_code, ring, "rust")
}

use crate::language::Language;

/// Language-aware extraction across 20+ languages and component formats (.astro, .vue, .svelte).
pub fn extract_raw_monads_lang(source_code: &str, ring: u32, language: &str) -> Vec<Monad> {
    let lower_ext = language.trim_start_matches('.').to_lowercase();

    // Specialized handling for SFC / Component files (.astro, .vue, .svelte, .html with scripts)
    if matches!(lower_ext.as_str(), "astro" | "vue" | "svelte")
        || (matches!(lower_ext.as_str(), "html" | "htm")
            && (source_code.trim_start().starts_with("---") || source_code.contains("<script")))
    {
        let sfc_monads = extract_component_blocks(source_code, ring, &lower_ext);
        if !sfc_monads.is_empty() {
            return sfc_monads;
        }
    }

    let lang_enum = Language::from_extension(&lower_ext);
    let lang = if lower_ext == "tsx" {
        std::str::FromStr::from_str("tsx")
            .ok()
            .or_else(|| lang_enum.ast_grep_lang())
    } else {
        lang_enum.ast_grep_lang()
    };

    match lang {
        Some(supported_lang) => {
            let extracted = extract_with_ast_grep(source_code, ring, supported_lang, &lower_ext);
            if !extracted.is_empty() {
                extracted
            } else {
                extract_with_heuristic(source_code, ring, &lower_ext)
            }
        }
        None => extract_with_heuristic(source_code, ring, &lower_ext),
    }
}

/// Extracts frontmatter (`---`), `<script>`, `<style>`, and template blocks from `.astro`, `.vue`, and `.svelte` files.
fn extract_component_blocks(source_code: &str, ring: u32, ext: &str) -> Vec<Monad> {
    let mut monads = Vec::new();
    let lines: Vec<&str> = source_code.lines().collect();

    // 1. Astro frontmatter `--- ... ---` at top of file
    if ext == "astro" && lines.first().map(|l| l.trim()) == Some("---") {
        if let Some(end_idx) = lines.iter().skip(1).position(|l| l.trim() == "---") {
            let fm_end = end_idx + 1;
            let fm_source = lines[1..fm_end].join("\n");
            if !fm_source.trim().is_empty() {
                let mut sub_monads = extract_with_ast_grep(&fm_source, ring, SupportLang::TypeScript, "astro");
                for m in &mut sub_monads {
                    m.line_start += 1;
                    m.line_end += 1;
                }
                monads.extend(sub_monads);

                let hash = generate_pure_hash(&fm_source);
                let angle = ((monads.len() as f64) * 137.50776405) % 360.0;
                monads.push(Monad::spawn_typed(
                    hash,
                    "astro_frontmatter".to_string(),
                    PolarCoord::new((ring as f64) * 100.0, angle),
                    fm_source,
                    ring,
                    MonadKind::Module,
                    1,
                    (fm_end as u32) + 1,
                    ext,
                ));
            }
        }
    }

    // 2. Extract `<script...>` and `<style...>` blocks
    let mut idx = 0;
    while idx < lines.len() {
        let trimmed = lines[idx].trim();
        if trimmed.starts_with("<script") {
            let start_line = idx;
            while idx < lines.len() && !lines[idx].contains("</script>") {
                idx += 1;
            }
            let end_line = idx.min(lines.len().saturating_sub(1));
            let block = lines[start_line..=end_line].join("\n");
            let hash = generate_pure_hash(&block);
            let angle = ((monads.len() as f64) * 137.50776405) % 360.0;
            monads.push(Monad::spawn_typed(
                hash,
                format!("{}_script", ext),
                PolarCoord::new((ring as f64) * 100.0, angle),
                block,
                ring,
                MonadKind::Function,
                (start_line as u32) + 1,
                (end_line as u32) + 1,
                ext,
            ));
        } else if trimmed.starts_with("<style") {
            let start_line = idx;
            while idx < lines.len() && !lines[idx].contains("</style>") {
                idx += 1;
            }
            let end_line = idx.min(lines.len().saturating_sub(1));
            let block = lines[start_line..=end_line].join("\n");
            let hash = generate_pure_hash(&block);
            let angle = ((monads.len() as f64) * 137.50776405) % 360.0;
            monads.push(Monad::spawn_typed(
                hash,
                format!("{}_styles", ext),
                PolarCoord::new((ring as f64) * 100.0, angle),
                block,
                ring,
                MonadKind::Struct,
                (start_line as u32) + 1,
                (end_line as u32) + 1,
                ext,
            ));
        }
        idx += 1;
    }

    monads
}

/// Core AST-based extraction using ast-grep tree-sitter parsing.
fn extract_with_ast_grep(
    source_code: &str,
    ring: u32,
    lang: SupportLang,
    language_name: &str,
) -> Vec<Monad> {
    let grep = AstGrep::new(source_code, lang);
    let root = grep.root();
    let mut monads = Vec::new();

    fn walk_and_extract(
        node: &ast_grep_core::Node<'_, StrDoc<SupportLang>>,
        monads: &mut Vec<Monad>,
        ring: u32,
        language_name: &str,
    ) {
        let kind_str = node.kind();
        let kind_ref: &str = &kind_str;

        // Unwrap export/decorator/template wrappers into their inner declaration
        if matches!(
            kind_ref,
            "export_statement" | "decorated_definition" | "template_declaration"
        ) {
            for child in node.children() {
                walk_and_extract(&child, monads, ring, language_name);
            }
            return;
        }

        if EXTRACTABLE_KINDS.contains(&kind_ref) {
            let text = node.text().to_string();
            let mut monad_kind = kind_from_node(kind_ref).unwrap_or(MonadKind::Unknown);
            if (kind_ref == "lexical_declaration" || kind_ref == "variable_declaration")
                && (text.contains("=>") || text.contains("function"))
            {
                monad_kind = MonadKind::Function;
            }
            let name = extract_name_from_node(node, &text, &monad_kind);

            let (identity, outbound_calls) = if kind_ref == "function_item" && (language_name == "rust" || language_name == "rs") {
                crate::ontology::canonical_hasher::extract_rust_function_identity(node)
                    .unwrap_or_else(|_| crate::ontology::canonical_hasher::extract_generic_identity(node, &name))
            } else {
                crate::ontology::canonical_hasher::extract_generic_identity(node, &name)
            };

            let hash = if !identity.implementation_hash.is_empty() {
                identity.implementation_hash.clone()
            } else {
                generate_pure_hash(&text)
            };

            let line_start = (node.start_pos().line() as u32) + 1;
            let line_end = (node.end_pos().line() as u32) + 1;

            let angle = ((monads.len() as f64) * 137.50776405) % 360.0;
            let coord = PolarCoord::new((ring as f64) * 100.0, angle);

            let mut monad = Monad::spawn_typed(
                hash,
                name,
                coord,
                text,
                ring,
                monad_kind,
                line_start,
                line_end,
                language_name,
            );
            monad = monad.with_canonical_identity(identity, outbound_calls.into_iter().collect());

            monads.push(monad);
            return;
        }

        for child in node.children() {
            walk_and_extract(&child, monads, ring, language_name);
        }
    }

    for child in root.children() {
        walk_and_extract(&child, &mut monads, ring, language_name);
    }
    monads
}

/// Fallback structural extractor for languages without ast-grep grammar or structured files
/// (Zig, Dart, Julia, OCaml, Clojure, GraphQL, Protobuf, Terraform/HCL, Shaders, SQL, TOML, Markdown, etc.).
fn extract_with_heuristic(source_code: &str, ring: u32, language: &str) -> Vec<Monad> {
    let mut monads = Vec::new();
    let lines: Vec<&str> = source_code.lines().collect();
    let mut current_block = String::new();
    let mut current_name = String::new();
    let mut current_kind = MonadKind::Unknown;
    let mut block_start_line: u32 = 1;

    let keywords: &[(&str, MonadKind)] = &[
        ("pub fn ", MonadKind::Function),
        ("fn ", MonadKind::Function),
        ("function ", MonadKind::Function),
        ("func ", MonadKind::Function),
        ("defp ", MonadKind::Function),
        ("defmacro ", MonadKind::Function),
        ("def ", MonadKind::Function),
        ("sub ", MonadKind::Function),
        ("(defn- ", MonadKind::Function),
        ("(defn ", MonadKind::Function),
        ("(defmacro ", MonadKind::Function),
        ("let rec ", MonadKind::Function),
        ("rpc ", MonadKind::Function),
        ("mutable struct ", MonadKind::Struct),
        ("struct ", MonadKind::Struct),
        ("class ", MonadKind::Struct),
        ("object ", MonadKind::Struct),
        ("contract ", MonadKind::Struct),
        ("library ", MonadKind::Module),
        ("record ", MonadKind::Struct),
        ("(defrecord ", MonadKind::Struct),
        ("message ", MonadKind::Struct),
        ("service ", MonadKind::Trait),
        ("resource ", MonadKind::Struct),
        ("variable ", MonadKind::Constant),
        ("output ", MonadKind::Constant),
        ("input ", MonadKind::Struct),
        ("type ", MonadKind::TypeAlias),
        ("enum ", MonadKind::Enum),
        ("union ", MonadKind::Enum),
        ("impl ", MonadKind::Impl),
        ("extension ", MonadKind::Impl),
        ("mixin ", MonadKind::Trait),
        ("trait ", MonadKind::Trait),
        ("interface ", MonadKind::Trait),
        ("protocol ", MonadKind::Trait),
        ("defprotocol ", MonadKind::Trait),
        ("(defprotocol ", MonadKind::Trait),
        ("defmodule ", MonadKind::Module),
        ("namespace ", MonadKind::Module),
        ("module ", MonadKind::Module),
        ("(ns ", MonadKind::Module),
        ("CREATE TABLE ", MonadKind::Struct),
        ("create table ", MonadKind::Struct),
        ("CREATE VIEW ", MonadKind::Trait),
        ("CREATE FUNCTION ", MonadKind::Function),
        ("CREATE PROCEDURE ", MonadKind::Function),
    ];

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let line_num = (i as u32) + 1;

        // Detect TOML `[section]` or Markdown `# Heading` or Terraform `resource "type" "name"` or code keyword
        let matched = if (language == "toml" || language == "ini") && trimmed.starts_with('[') && trimmed.ends_with(']') {
            Some((trimmed.trim_matches(|c| c == '[' || c == ']').to_string(), MonadKind::Module))
        } else if (language == "md" || language == "mdx" || language == "markdown") && trimmed.starts_with('#') {
            let heading = trimmed.trim_start_matches('#').trim().to_string();
            if !heading.is_empty() {
                Some((heading, MonadKind::Module))
            } else {
                None
            }
        } else if (language == "tf" || language == "hcl" || language == "terraform")
            && (trimmed.starts_with("resource ") || trimmed.starts_with("data ") || trimmed.starts_with("variable ") || trimmed.starts_with("output ") || trimmed.starts_with("module "))
        {
            let parts: Vec<&str> = trimmed
                .split_whitespace()
                .map(|p| p.trim_matches(|c| c == '"' || c == '{'))
                .filter(|p| !p.is_empty())
                .collect();
            let kw = parts.first().copied().unwrap_or("resource");
            let name = if parts.len() >= 3 {
                format!("{}.{}", parts[1], parts[2])
            } else if parts.len() >= 2 {
                parts[1].to_string()
            } else {
                kw.to_string()
            };
            let kind = match kw {
                "variable" | "output" => MonadKind::Constant,
                "module" => MonadKind::Module,
                _ => MonadKind::Struct,
            };
            Some((name, kind))
        } else {
            keywords.iter().find_map(|(kw, kind)| {
                if let Some(pos) = trimmed.find(kw) {
                    if pos == 0 || trimmed[..pos].trim().split_whitespace().all(|w| matches!(w, "pub" | "export" | "async" | "static" | "public" | "private" | "protected" | "abstract" | "final" | "open" | "inline")) {
                        let after = trimmed[pos + kw.len()..].trim_start_matches('"');
                        let name: String = after
                            .chars()
                            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '.' || *c == '-')
                            .collect();
                        if !name.is_empty() {
                            return Some((name, kind.clone()));
                        }
                    }
                }
                None
            })
        };

        if let Some((new_name, new_kind)) = matched {
            if !current_block.trim().is_empty() && !current_name.is_empty() {
                let hash = generate_pure_hash(&current_block);
                let angle = ((monads.len() as f64) * 137.50776405) % 360.0;
                let coord = PolarCoord::new((ring as f64) * 100.0, angle);
                monads.push(Monad::spawn_typed(
                    hash,
                    current_name.clone(),
                    coord,
                    current_block.clone(),
                    ring,
                    current_kind.clone(),
                    block_start_line,
                    line_num.saturating_sub(1).max(block_start_line),
                    language,
                ));
            }
            current_block = line.to_string();
            current_name = new_name;
            current_kind = new_kind;
            block_start_line = line_num;
        } else if !current_name.is_empty() {
            current_block.push('\n');
            current_block.push_str(line);
        }
    }

    if !current_block.trim().is_empty() && !current_name.is_empty() {
        let hash = generate_pure_hash(&current_block);
        let angle = ((monads.len() as f64) * 137.50776405) % 360.0;
        let coord = PolarCoord::new((ring as f64) * 100.0, angle);
        monads.push(Monad::spawn_typed(
            hash,
            current_name,
            coord,
            current_block,
            ring,
            current_kind,
            block_start_line,
            (lines.len() as u32).max(block_start_line),
            language,
        ));
    }

    monads
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rust_functions() {
        let source = r#"
fn hello() {
    println!("Hello");
}

fn world(x: i32) -> i32 {
    x + 1
}
"#;
        let monads = extract_raw_monads(source, 1);
        assert!(monads.len() >= 2, "Should extract at least 2 functions, got {}", monads.len());

        let names: Vec<&str> = monads.iter().map(|m| m.name.as_str()).collect();
        assert!(names.contains(&"hello"), "Should find 'hello', got {:?}", names);
        assert!(names.contains(&"world"), "Should find 'world', got {:?}", names);
    }

    #[test]
    fn test_extract_structs_and_enums() {
        let source = r#"
struct Point {
    x: f64,
    y: f64,
}

enum Color {
    Red,
    Green,
    Blue,
}

fn draw(p: Point, c: Color) {}
"#;
        let monads = extract_raw_monads(source, 1);
        assert!(monads.iter().any(|m| m.kind == MonadKind::Struct), "Should find a struct");
        assert!(monads.iter().any(|m| m.kind == MonadKind::Enum), "Should find an enum");
        assert!(monads.iter().any(|m| m.kind == MonadKind::Function), "Should find a function");
    }

    #[test]
    fn test_extract_impl_blocks() {
        let source = r#"
struct Foo;

impl Foo {
    fn bar(&self) -> i32 {
        42
    }

    fn baz(&self) {}
}
"#;
        let monads = extract_raw_monads(source, 1);
        assert!(monads.iter().any(|m| m.kind == MonadKind::Impl), "Should find an impl block");
    }

    #[test]
    fn test_semantic_hash_populated() {
        let source = "fn test_hash() { 42 }\n";
        let monads = extract_raw_monads(source, 1);
        assert!(!monads.is_empty());
        assert!(!monads[0].semantic_hash.is_empty(), "Semantic hash should be populated");
    }

    #[test]
    fn test_heuristic_fallback() {
        let source = "function hello() { return 1; }\nfunction world() { return 2; }\n";
        let monads = extract_raw_monads_lang(source, 1, "unknown_lang");
        assert!(monads.len() >= 2, "Heuristic should find at least 2 functions");
    }

    #[test]
    fn test_line_numbers_are_1_based() {
        let source = "// comment line 1\nfn alpha() {\n    let x = 1;\n}\n";
        let monads = extract_raw_monads(source, 1);
        assert_eq!(monads.len(), 1);
        assert_eq!(monads[0].line_start, 2);
        assert_eq!(monads[0].line_end, 4);
    }

    #[test]
    fn test_polyglot_ast_extraction() {
        let ts_source = "export interface User {\n  id: string;\n}\nexport function greetUser(u: User) {\n  return u.id;\n}\n";
        let ts_monads = extract_raw_monads_lang(ts_source, 1, "typescript");
        let ts_names: Vec<&str> = ts_monads.iter().map(|m| m.name.as_str()).collect();
        assert!(ts_names.contains(&"User"), "TS should extract interface User, got {:?}", ts_names);
        assert!(ts_names.contains(&"greetUser"), "TS should extract function greetUser, got {:?}", ts_names);

        let py_source = "class Pipeline:\n    pass\n\ndef run_pipeline():\n    return Pipeline()\n";
        let py_monads = extract_raw_monads_lang(py_source, 1, "python");
        let py_names: Vec<&str> = py_monads.iter().map(|m| m.name.as_str()).collect();
        assert!(py_names.contains(&"Pipeline"), "Python should extract class Pipeline, got {:?}", py_names);
        assert!(py_names.contains(&"run_pipeline"), "Python should extract def run_pipeline, got {:?}", py_names);

        let go_source = "package main\n\nfunc Compute(x int) int {\n    return x * 2\n}\n";
        let go_monads = extract_raw_monads_lang(go_source, 1, "go");
        let go_names: Vec<&str> = go_monads.iter().map(|m| m.name.as_str()).collect();
        assert!(go_names.contains(&"Compute"), "Go should extract func Compute, got {:?}", go_names);
    }

    #[test]
    fn test_component_and_config_extraction() {
        let astro_source = "---\nexport interface Props {\n  title: string;\n}\nfunction formatTitle(t: string) {\n  return t.toUpperCase();\n}\n---\n<main><h1>{title}</h1></main>\n";
        let astro_monads = extract_raw_monads_lang(astro_source, 1, "astro");
        let astro_names: Vec<&str> = astro_monads.iter().map(|m| m.name.as_str()).collect();
        assert!(astro_names.contains(&"Props"), "Astro should extract Props interface, got {:?}", astro_names);
        assert!(astro_names.contains(&"formatTitle"), "Astro should extract formatTitle function, got {:?}", astro_names);

        let toml_source = "[package]\nname = \"mandala\"\n\n[dependencies]\nserde = \"1.0\"\n";
        let toml_monads = extract_raw_monads_lang(toml_source, 1, "toml");
        let toml_names: Vec<&str> = toml_monads.iter().map(|m| m.name.as_str()).collect();
        assert!(toml_names.contains(&"package"), "TOML should extract [package], got {:?}", toml_names);
        assert!(toml_names.contains(&"dependencies"), "TOML should extract [dependencies], got {:?}", toml_names);
    }

    #[test]
    fn test_extended_polyglot_languages_extraction() {
        let sol_source = "pragma solidity ^0.8.20;\ncontract Vault {\n    function deposit(uint256 amount) public {}\n}\n";
        let sol_monads = extract_raw_monads_lang(sol_source, 1, "sol");
        assert!(!sol_monads.is_empty(), "Solidity should extract Vault contract/functions");

        let zig_source = "pub fn compute(a: u32) u32 {\n    return a + 1;\n}\n";
        let zig_monads = extract_raw_monads_lang(zig_source, 1, "zig");
        assert_eq!(zig_monads.len(), 1);
        assert_eq!(zig_monads[0].name, "compute");

        let tf_source = "resource \"aws_s3_bucket\" \"archive\" {\n  bucket = \"mandala\"\n}\nvariable \"region\" {\n  type = string\n}\n";
        let tf_monads = extract_raw_monads_lang(tf_source, 1, "tf");
        let tf_names: Vec<&str> = tf_monads.iter().map(|m| m.name.as_str()).collect();
        assert!(tf_names.contains(&"aws_s3_bucket.archive"));
        assert!(tf_names.contains(&"region"));
    }
}

