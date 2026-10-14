use crate::ontology::monad::Monad;
use crate::weaver::ast_extractor::extract_raw_monads_lang;

pub type ExtractedMonad = crate::ontology::monad::Monad;

/// Extractor canónico de mónadas funcionales para pruebas y utilidades ontológicas
pub struct MonadExtractor;

impl MonadExtractor {
    /// Extrae mónadas funcionales desde código fuente en Rust (con identidad canónica Merkle-AST)
    pub fn extract_functions_from_source(source: &str) -> anyhow::Result<Vec<Monad>> {
        Ok(extract_raw_monads_lang(source, 0, "rust"))
    }
}
