use serde::{Serialize, Deserialize};
use crate::ontology::monad::Monad;
use crate::template::{AdapterConfig, AdapterEngine};
use std::collections::HashMap;

/// Assembles a collection of monads (a Constellation or selection) into a valid
/// source file (The Source), ordered by logical domain and dependency.
pub fn distill_source(monads: &[Monad]) -> String {
    distill_source_with_adapters(monads, &[])
}

pub fn distill_source_with_adapters(monads: &[Monad], adapters: &[AdapterConfig]) -> String {
    if monads.is_empty() {
        return String::new();
    }

    if !adapters.is_empty() {
        return AdapterEngine::apply_adapters(monads, adapters);
    }

    // Group by kind for proper ordering: modules/types first, then impls, then functions
    let ordered = order_by_dependency(monads);

    let mut source = String::new();

    // Distillation header
    let mut rings: Vec<u32> = monads.iter().map(|m| m.ring).collect();
    rings.sort();
    rings.dedup();
    let primary_lang = monads.first().map(|m| m.language.as_str()).unwrap_or("unknown");

    source.push_str(&format!("// Distilled Source — {} monads from {} ring(s)\n", 
        monads.len(), rings.len()
    ));
    source.push_str(&format!("// Language: {}\n\n", primary_lang));

    // Auto-generate imports and mod statements
    if primary_lang == "rust" {
        let mod_statements: Vec<String> = crate::weaver::auto_imports::ImportAnalyzer::generate_mod_statements(monads);
        for stmt in mod_statements.iter() {
            source.push_str(stmt);
            source.push('\n');
        }
        source.push('\n');
    }

    for monad in &ordered {
        source.push_str(&monad.content);
        source.push_str("\n\n");
    }

    source
}

/// Orders monads by dependency priority:
/// 1. Modules (mod)
/// 2. Constants and type aliases (const, type)
/// 3. Structs and enums (data definitions)
/// 4. Traits (interfaces)
/// 5. Impl blocks (implementations)
/// 6. Functions (loose functions)
/// 7. Unknown
fn order_by_dependency(monads: &[Monad]) -> Vec<Monad> {
    let mut sorted = monads.to_vec();
    sorted.sort_by_key(|m| {
        use crate::ontology::monad::MonadKind;
        match m.kind {
            MonadKind::Module => 0,
            MonadKind::Constant => 1,
            MonadKind::TypeAlias => 2,
            MonadKind::Struct => 3,
            MonadKind::Enum => 4,
            MonadKind::Trait => 5,
            MonadKind::Impl => 6,
            MonadKind::Function => 7,
            MonadKind::Unknown => 8,
        }
    });
    sorted
}

/// Validates that a collection of monads intended for distillation
/// does not contain incoherences.
pub fn validate_source_coherence(monads: &[Monad]) -> Result<(), Vec<IncoherenceReport>> {
    let mut errors = Vec::new();

    // Check for duplicate definitions (same name + kind)
    let mut seen: HashMap<String, Vec<&Monad>> = HashMap::new();
    for monad in monads {
        let key = format!("{}::{}", monad.kind, monad.name);
        seen.entry(key).or_default().push(monad);
    }

    for (key, group) in &seen {
        if group.len() > 1 {
            errors.push(IncoherenceReport {
                kind: IncoherenceKind::DuplicateDefinition,
                message: format!(
                    "Duplicate definition '{}' found in {} monads (rings: {:?})",
                    key,
                    group.len(),
                    group.iter().map(|m| m.ring).collect::<Vec<_>>()
                ),
                monad_ids: group.iter().map(|m| m.id.clone()).collect(),
            });
        }
    }

    // Syntax Validation for Rust code
    let primary_lang = monads.first().map(|m| m.language.as_str()).unwrap_or("unknown");
    if primary_lang == "rust" {
        let compiled_source = distill_source(monads);
        if let Err(syn_err) = syn::parse_file(&compiled_source) {
            errors.push(IncoherenceReport {
                kind: IncoherenceKind::SyntaxError,
                message: format!("Syntax error in distilled source: {}", syn_err),
                monad_ids: monads.iter().map(|m| m.id.clone()).collect(),
            });
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Orders monads by language for mixed-language source output.

pub fn order_by_language(monads: &[Monad]) -> Vec<Monad> {
    let mut sorted = monads.to_vec();
    sorted.sort_by_key(|m| match m.language.as_str() {
        "rust" => 0,
        "go" => 1,
        "typescript" | "javascript" => 2,
        "python" => 3,
        _ => 99,
    });
    sorted
}

/// Groups monads by language.

pub fn group_by_language(monads: &[Monad]) -> HashMap<String, Vec<Monad>> {
    let mut groups: HashMap<String, Vec<Monad>> = HashMap::new();
    for monad in monads {
        groups.entry(monad.language.clone()).or_default().push(monad.clone());
    }
    groups
}

/// Distills source for multiple languages, generating separate sections per language.

pub fn distill_multi_lang(monads: &[Monad], mode: CrossLangMode) -> String {
    if monads.is_empty() {
        return String::new();
    }

    let grouped = group_by_language(monads);
    let mut source = String::new();

    source.push_str(&format!(
        "// Multi-Language Distilled Source — {} monads across {} language(s)\n\n",
        monads.len(),
        grouped.len()
    ));

    match mode {
        CrossLangMode::SingleFile => {
            let ordered = order_by_language(monads);
            for monad in &ordered {
                source.push_str(&format!("// === {} ===\n", monad.language));
                source.push_str(&monad.content);
                source.push_str("\n\n");
            }
        }
        CrossLangMode::SectionPerLanguage => {
            for (lang, lang_monads) in &grouped {
                source.push_str(&format!("// ===== {} =====\n\n", lang));
                let ordered = order_by_dependency(lang_monads);
                for monad in &ordered {
                    source.push_str(&monad.content);
                    source.push_str("\n\n");
                }
            }
        }
    }

    source
}

/// Mode for handling multiple languages during distillation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum CrossLangMode {
    SingleFile,
    SectionPerLanguage,
}

/// Describes a detected incoherence in a distillation selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncoherenceReport {
    pub kind: IncoherenceKind,
    pub message: String,
    pub monad_ids: Vec<String>,
}

/// Types of incoherences that can occur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncoherenceKind {
    /// Two monads define the same entity (same name + kind).
    DuplicateDefinition,
    /// A monad references an entity that doesn't exist in the selection.
    MissingDependency,
    /// Generated source has syntax errors.
    SyntaxError,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::polar_space::PolarCoord;
    use crate::ontology::monad::MonadKind;

    fn make_typed_monad(name: &str, content: &str, ring: u32, kind: MonadKind) -> Monad {
        Monad::spawn_typed(
            format!("hash_{}", name),
            name.to_string(),
            PolarCoord::new(ring as f64 * 100.0, 45.0),
            content.to_string(),
            ring,
            kind,
            0,
            5,
            "rust",
        )
    }

    #[test]
    fn test_ordering_structs_before_functions() {
        let monads = vec![
            make_typed_monad("do_thing", "fn do_thing() {}", 1, MonadKind::Function),
            make_typed_monad("MyStruct", "struct MyStruct {}", 1, MonadKind::Struct),
        ];
        let source = distill_source(&monads);
        let struct_pos = source.find("struct MyStruct").unwrap();
        let fn_pos = source.find("fn do_thing").unwrap();
        assert!(struct_pos < fn_pos, "Structs should appear before functions");
    }

    #[test]
    fn test_duplicate_detection() {
        let monads = vec![
            make_typed_monad("foo", "fn foo() { 1 }", 1, MonadKind::Function),
            make_typed_monad("foo", "fn foo() { 2 }", 2, MonadKind::Function),
        ];
        let result = validate_source_coherence(&monads);
        assert!(result.is_err(), "Should detect duplicate definitions");
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].kind, IncoherenceKind::DuplicateDefinition);
    }

    #[test]
    fn test_no_duplicates_different_kinds() {
        let monads = vec![
            make_typed_monad("Foo", "struct Foo {}", 1, MonadKind::Struct),
            make_typed_monad("Foo", "impl Foo {}", 1, MonadKind::Impl),
        ];
        let result = validate_source_coherence(&monads);
        assert!(result.is_ok(), "struct Foo and impl Foo are not duplicates");
    }

    #[test]
    fn test_order_by_language() {
        let monads = vec![
            make_typed_monad("fn1", "fn fn1() {}", 1, MonadKind::Function),
            make_typed_monad("fn2", "def fn2(): pass", 1, MonadKind::Function),
            make_typed_monad("fn3", "func fn3()", 1, MonadKind::Function),
        ];
        let mut m = monads.clone();
        m[0].language = "rust".to_string();
        m[1].language = "python".to_string();
        m[2].language = "go".to_string();

        let ordered = order_by_language(&m);
        assert_eq!(ordered[0].language, "rust");
        assert_eq!(ordered[1].language, "go");
        assert_eq!(ordered[2].language, "python");
    }

    #[test]
    fn test_group_by_language() {
        let mut m1 = make_typed_monad("fn1", "fn fn1() {}", 1, MonadKind::Function);
        let mut m2 = make_typed_monad("fn2", "def fn2(): pass", 1, MonadKind::Function);
        let mut m3 = make_typed_monad("fn3", "fn fn3() {}", 1, MonadKind::Function);
        m1.language = "rust".to_string();
        m2.language = "python".to_string();
        m3.language = "rust".to_string();

        let groups = group_by_language(&[m1, m2, m3]);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("rust").unwrap().len(), 2);
        assert_eq!(groups.get("python").unwrap().len(), 1);
    }

    #[test]
    fn test_distill_multi_lang_single_file() {
        let mut m1 = make_typed_monad("fn1", "fn fn1() {}", 1, MonadKind::Function);
        let mut m2 = make_typed_monad("fn2", "def fn2(): pass", 1, MonadKind::Function);
        m1.language = "rust".to_string();
        m2.language = "python".to_string();

        let source = distill_multi_lang(&[m1, m2], CrossLangMode::SingleFile);
        assert!(source.contains("rust"));
        assert!(source.contains("python"));
        assert!(source.contains("fn fn1()"));
        assert!(source.contains("def fn2"));
    }

    #[test]
    fn test_distill_multi_lang_section_per_language() {
        let mut m1 = make_typed_monad("fn1", "fn fn1() {}", 1, MonadKind::Function);
        let mut m2 = make_typed_monad("fn2", "def fn2(): pass", 1, MonadKind::Function);
        m1.language = "rust".to_string();
        m2.language = "python".to_string();

        let source = distill_multi_lang(&[m1, m2], CrossLangMode::SectionPerLanguage);
        assert!(source.contains("rust"));
        assert!(source.contains("python"));
    }
}
