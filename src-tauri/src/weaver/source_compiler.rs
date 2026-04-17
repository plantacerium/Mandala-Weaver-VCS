use crate::ontology::monad::Monad;
use std::collections::HashMap;

/// Assembles a collection of monads (a Constellation or selection) into a valid
/// source file (The Source), ordered by logical domain and dependency.
pub fn distill_source(monads: &[Monad]) -> String {
    if monads.is_empty() {
        return String::new();
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

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Describes a detected incoherence in a distillation selection.
#[derive(Debug, Clone)]
pub struct IncoherenceReport {
    pub kind: IncoherenceKind,
    pub message: String,
    pub monad_ids: Vec<String>,
}

/// Types of incoherences that can occur.
#[derive(Debug, Clone, PartialEq, Eq)]
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
}
