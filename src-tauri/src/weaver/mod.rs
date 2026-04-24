pub mod ast_extractor;
pub mod threader;
pub mod resolver;
pub mod source_compiler;
pub mod watcher;
pub mod auto_imports;
pub mod file_writer;
pub mod contract;
pub mod semantic_diff;

use crate::persistence::surreal_bridge::{Db, insert_and_link, get_ring, get_all_monads};
use crate::weaver::ast_extractor::extract_raw_monads_lang;
use crate::weaver::resolver::identify_deltas;
use crate::weaver::semantic_diff::SemanticDiff;
use crate::language::detect_language;
use surrealdb::Surreal;
use std::fs;
use std::path::Path;

/// Realiza la operación 'Expand': Lee un archivo, detecta cambios y crea un nuevo anillo radial.
pub async fn expand_from_source(
    db: &Surreal<Db>, 
    file_path: &str
) -> anyhow::Result<u32> {
    // 1. Leer el código actual
    let source_code = fs::read_to_string(file_path)?;
    
    // 2. Determinar el nivel del anillo actual (el más exterior)
    let all_monads = get_all_monads(db).await?;
    let current_max_ring = all_monads.iter().map(|m| m.ring).max().unwrap_or(0);
    let next_ring = current_max_ring + 1;

    // 3. Detect language
    let path = Path::new(file_path);
    let lang = detect_language(path);
    let lang_name = lang.tree_sitter_name();

    // 4. Extraer mónadas del código fuente
    let new_monads = extract_raw_monads_lang(&source_code, next_ring, lang_name);
    
    // 5. Obtener mónadas del anillo base para comparar
    let base_monads = get_ring(db, current_max_ring).await?;
    
    // 6. Identificar deltas
    let deltas = identify_deltas(&base_monads, &new_monads);
    
    if deltas.is_empty() {
        return Ok(current_max_ring); // No hay cambios
    }

    // 7. Perform Semantic Diff for logging/telemetry
    let diffs = SemanticDiff::generate_batch(&base_monads, &new_monads);
    for diff in diffs {
        println!("  Δ Semantic Change in {}: {}", diff.monad_id, SemanticDiff::format_colored(&diff));
    }

    // 8. Persistir deltas y vincular con padres si existen
    for monad in deltas {
        let parent = base_monads.iter().find(|m| m.name == monad.name);
        let parent_id = parent.map(|p| p.id.as_str());
        
        insert_and_link(db, &monad, parent_id).await?;
    }

    Ok(next_ring)
}
