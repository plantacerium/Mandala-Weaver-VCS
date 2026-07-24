use crate::ontology::monad::{Monad, MonadKind};
use crate::geometry::polar_space::PolarCoord;
use crate::ontology::semantic_hash::generate_pure_hash;
use ast_grep_core::AstGrep;
use ast_grep_core::tree_sitter::StrDoc;
use ast_grep_language::SupportLang;

/// Maps a tree-sitter node kind to our MonadKind enum.
fn kind_from_node(node_kind: &str) -> Option<MonadKind> {
    match node_kind {
        "function_item" => Some(MonadKind::Function),
        "struct_item" => Some(MonadKind::Struct),
        "enum_item" => Some(MonadKind::Enum),
        "impl_item" => Some(MonadKind::Impl),
        "trait_item" => Some(MonadKind::Trait),
        "mod_item" => Some(MonadKind::Module),
        "const_item" | "static_item" => Some(MonadKind::Constant),
        "type_item" => Some(MonadKind::TypeAlias),
        _ => None,
    }
}

/// The list of top-level AST node kinds we want to extract as Monads.
const EXTRACTABLE_KINDS: &[&str] = &[
    "function_item",
    "struct_item",
    "enum_item",
    "impl_item",
    "trait_item",
    "mod_item",
    "const_item",
    "static_item",
    "type_item",
];

/// Extracts the "name" from a tree-sitter node's text.
/// For `fn foo(...)`, returns "foo". For `struct Bar`, returns "Bar".
/// Falls back to a hash-prefix if no name identifier is found.
fn extract_name_from_text(text: &str, kind: &MonadKind) -> String {
    let prefix = match kind {
        MonadKind::Function => "fn ",
        MonadKind::Struct => "struct ",
        MonadKind::Enum => "enum ",
        MonadKind::Impl => "impl ",
        MonadKind::Trait => "trait ",
        MonadKind::Module => "mod ",
        MonadKind::Constant => "",
        MonadKind::TypeAlias => "type ",
        MonadKind::Unknown => "",
    };

    if !prefix.is_empty() {
        if let Some(after) = text.find(prefix) {
            let rest = &text[after + prefix.len()..];
            let name: String = rest.chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                return name;
            }
        }
    }

    // Fallback for const/static: `const NAME: Type = ...`
    if matches!(kind, MonadKind::Constant) {
        for keyword in &["const ", "static "] {
            if let Some(after) = text.find(keyword) {
                let rest = &text[after + keyword.len()..];
                let name: String = rest.chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !name.is_empty() {
                    return name;
                }
            }
        }
    }

    // Last fallback: use a truncated hash
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

/// Language-aware extraction. Currently supports Rust natively through ast-grep.
/// For unsupported languages, falls back to the heuristic keyword scanner.
pub fn extract_raw_monads_lang(source_code: &str, ring: u32, language: &str) -> Vec<Monad> {
    let lang_enum = Language::from_extension(language);
    let lang = lang_enum.ast_grep_lang();

    match lang {
        Some(supported_lang) => extract_with_ast_grep(source_code, ring, supported_lang, language),
        None => extract_with_heuristic(source_code, ring, language),
    }
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

    // Walk through the AST recursively looking for extractable kinds.
    // Node type is `ast_grep_core::Node<'_, StrDoc<SupportLang>>`.
    fn walk_and_extract(
        node: &ast_grep_core::Node<'_, StrDoc<SupportLang>>,
        monads: &mut Vec<Monad>,
        ring: u32,
        language_name: &str,
    ) {
        let kind_str = node.kind();
        let kind_ref: &str = &kind_str;

        if EXTRACTABLE_KINDS.contains(&kind_ref) {
            let text = node.text().to_string();
            let monad_kind = kind_from_node(kind_ref).unwrap_or(MonadKind::Unknown);
            let name = extract_name_from_text(&text, &monad_kind);
            let hash = generate_pure_hash(&text);

            // Calculate line positions from byte range
            let range = node.range();
            let line_start = range.start as u32;
            let line_end = range.end as u32;

            // Auto-assign angular position based on monad index
            let angle = (monads.len() as f64) * (360.0 / 16.0);
            let coord = PolarCoord::new((ring as f64) * 100.0, angle);

            monads.push(Monad::spawn_typed(
                hash,
                name,
                coord,
                text,
                ring,
                monad_kind,
                line_start,
                line_end,
                language_name,
            ));
            // Don't recurse into children of extractable nodes to avoid
            // duplicating nested items (e.g. fn inside impl).
            return;
        }

        // Recurse into children for non-extractable nodes
        for child in node.children() {
            walk_and_extract(&child, monads, ring, language_name);
        }
    }

    walk_and_extract(&root, &mut monads, ring, language_name);
    monads
}

/// Fallback heuristic extractor for languages without ast-grep support.
/// Scans for keyword patterns like `fn`, `function`, `struct`, etc.
fn extract_with_heuristic(source_code: &str, ring: u32, language: &str) -> Vec<Monad> {
    let mut monads = Vec::new();
    let lines: Vec<&str> = source_code.lines().collect();
    let mut current_block = String::new();
    let mut current_name = String::new();
    let mut block_start_line: u32 = 0;

    let keywords = &["fn ", "function ", "struct ", "enum ", "impl ", "trait ", "class ", "def "];

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let is_keyword_line = keywords.iter().any(|kw| trimmed.contains(kw));

        if is_keyword_line {
            if !current_block.is_empty() {
                let hash = generate_pure_hash(&current_block);
                let angle = (monads.len() as f64) * (360.0 / 16.0);
                let coord = PolarCoord::new((ring as f64) * 100.0, angle);
                monads.push(Monad::spawn_typed(
                    hash,
                    current_name.clone(),
                    coord,
                    current_block.clone(),
                    ring,
                    MonadKind::Unknown,
                    block_start_line,
                    i as u32,
                    language,
                ));
            }
            current_block = line.to_string();
            current_name = trimmed.split_whitespace().nth(1).unwrap_or("unknown").to_string();
            current_name = current_name.chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            block_start_line = i as u32;
        } else {
            current_block.push('\n');
            current_block.push_str(line);
        }
    }

    // Flush the last block
    if !current_block.is_empty() {
        let hash = generate_pure_hash(&current_block);
        let angle = (monads.len() as f64) * (360.0 / 16.0);
        let coord = PolarCoord::new((ring as f64) * 100.0, angle);
        monads.push(Monad::spawn_typed(
            hash,
            current_name,
            coord,
            current_block,
            ring,
            MonadKind::Unknown,
            block_start_line,
            lines.len() as u32,
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
}
