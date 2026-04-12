use crate::ontology::monad::Monad;

/// Compara dos mónadas para determinar si han evolucionado.
pub fn has_evolved(base: &Monad, target: &Monad) -> bool {
    base.id != target.id
}

/// Identifica qué mónadas de un set nuevo son deltas (cambios) respecto a un set base.
pub fn identify_deltas(base_set: &[Monad], new_set: &[Monad]) -> Vec<Monad> {
    let mut deltas = Vec::new();

    for new_monad in new_set {
        let existing = base_set.iter().find(|m| m.name == new_monad.name);
        
        match existing {
            Some(base_monad) => {
                if has_evolved(base_monad, new_monad) {
                    deltas.push(new_monad.clone());
                }
            },
            None => {
                // Es una nueva funcionalidad/mónada
                deltas.push(new_monad.clone());
            }
        }
    }

    deltas
}
