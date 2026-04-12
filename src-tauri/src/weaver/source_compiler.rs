use crate::ontology::monad::Monad;

/// Ensambla una colección de mónadas (una Constelación o selección) en un archivo de texto válido (La Fuente).
pub fn distill_source(monads: &[Monad]) -> String {
    let mut source = String::new();

    // Ordenamos por coordenada temática (theta) o simplemente por orden de aparición
    // En un sistema real, el orden de las mónadas en el archivo es crítico.
    // Por ahora las unimos separadas por saltos de línea.
    for monad in monads {
        source.push_str(&monad.content);
        source.push_str("\n\n");
    }

    source
}
