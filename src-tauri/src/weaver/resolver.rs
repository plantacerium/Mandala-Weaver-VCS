use crate::ontology::monad::{Monad, DeltaType};

/// Compares two monads to determine if one has evolved from the other.
/// Uses semantic hash comparison (blake3 of whitespace-stripped content)
/// instead of raw id comparison — ensuring formatting changes are ignored.
pub fn has_evolved(base: &Monad, target: &Monad) -> bool {
    base.is_semantically_different(target)
}

/// Identifies which monads in a new set are deltas (changes) relative to a base set.
/// Returns a Vec of (Monad, DeltaType) tuples with categorized changes.
pub fn identify_deltas_typed(base_set: &[Monad], new_set: &[Monad]) -> Vec<(Monad, DeltaType)> {
    let mut deltas = Vec::new();

    // Detect Added and Modified monads
    for new_monad in new_set {
        let existing = base_set.iter().find(|m| m.name == new_monad.name);

        match existing {
            Some(base_monad) => {
                if has_evolved(base_monad, new_monad) {
                    // Check if it was renamed (same hash, different name handled elsewhere)
                    deltas.push((new_monad.clone(), DeltaType::Modified));
                }
                // else: Unchanged — not a delta
            },
            None => {
                // Check if it might be a rename (same hash exists under different name)
                let renamed_from = base_set.iter().find(|m| {
                    m.semantic_hash == new_monad.semantic_hash && m.name != new_monad.name
                });

                if renamed_from.is_some() {
                    deltas.push((new_monad.clone(), DeltaType::Renamed));
                } else {
                    deltas.push((new_monad.clone(), DeltaType::Added));
                }
            }
        }
    }

    // Detect Deleted monads (present in base but absent in new)
    for base_monad in base_set {
        let still_exists = new_set.iter().any(|m| m.name == base_monad.name);
        let was_renamed = new_set.iter().any(|m| {
            m.semantic_hash == base_monad.semantic_hash && m.name != base_monad.name
        });

        if !still_exists && !was_renamed {
            deltas.push((base_monad.clone(), DeltaType::Deleted));
        }
    }

    deltas
}

/// Backward-compatible wrapper: returns only the monads that changed (excluding deletions).
/// This is used by the existing expand pipeline.
pub fn identify_deltas(base_set: &[Monad], new_set: &[Monad]) -> Vec<Monad> {
    identify_deltas_typed(base_set, new_set)
        .into_iter()
        .filter(|(_, dt)| matches!(dt, DeltaType::Added | DeltaType::Modified | DeltaType::Renamed))
        .map(|(m, _)| m)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::polar_space::PolarCoord;

    fn make_monad(name: &str, content: &str, ring: u32) -> Monad {
        Monad::spawn(
            format!("id_{}", name),
            name.to_string(),
            PolarCoord::new(ring as f64 * 100.0, 45.0),
            content.to_string(),
            ring,
        )
    }

    #[test]
    fn test_no_changes() {
        let base = vec![make_monad("foo", "fn foo() { 1 }", 1)];
        let new = vec![make_monad("foo", "fn foo() { 1 }", 2)];
        let deltas = identify_deltas(&base, &new);
        assert!(deltas.is_empty(), "Identical content should produce no deltas");
    }

    #[test]
    fn test_whitespace_ignored() {
        let base = vec![make_monad("foo", "fn foo() { 1 }", 1)];
        let new = vec![make_monad("foo", "fn foo()  {  1  }", 2)];
        let deltas = identify_deltas(&base, &new);
        assert!(deltas.is_empty(), "Whitespace-only changes should not be deltas");
    }

    #[test]
    fn test_content_change_detected() {
        let base = vec![make_monad("foo", "fn foo() { 1 }", 1)];
        let new = vec![make_monad("foo", "fn foo() { 2 }", 2)];
        let deltas = identify_deltas(&base, &new);
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].name, "foo");
    }

    #[test]
    fn test_new_monad_added() {
        let base = vec![make_monad("foo", "fn foo() {}", 1)];
        let new = vec![
            make_monad("foo", "fn foo() {}", 2),
            make_monad("bar", "fn bar() {}", 2),
        ];
        let deltas = identify_deltas(&base, &new);
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].name, "bar");
    }

    #[test]
    fn test_deletion_detected() {
        let base = vec![
            make_monad("foo", "fn foo() {}", 1),
            make_monad("bar", "fn bar() {}", 1),
        ];
        let new = vec![make_monad("foo", "fn foo() {}", 2)];
        let typed = identify_deltas_typed(&base, &new);
        let deleted: Vec<_> = typed.iter().filter(|(_, dt)| *dt == DeltaType::Deleted).collect();
        assert_eq!(deleted.len(), 1);
        assert_eq!(deleted[0].0.name, "bar");
    }

    #[test]
    fn test_rename_detected() {
        let base = vec![make_monad("foo", "fn foo() { magic() }", 1)];
        let new = vec![make_monad("bar", "fn foo() { magic() }", 2)]; // same content, different name
        let typed = identify_deltas_typed(&base, &new);
        let renamed: Vec<_> = typed.iter().filter(|(_, dt)| *dt == DeltaType::Renamed).collect();
        assert_eq!(renamed.len(), 1);
        assert_eq!(renamed[0].0.name, "bar");
    }
}
