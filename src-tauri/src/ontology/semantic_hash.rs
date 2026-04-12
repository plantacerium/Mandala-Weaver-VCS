/// Genera un hash ignorando espacios y comentarios (AST-based helper, actual AST-grep logic later).
pub fn generate_pure_hash(content: &str) -> String {
    // Por ahora, solo limpiamos espacios para el hash semántico básico
    let clean_content: String = content.chars().filter(|c| !c.is_whitespace()).collect();
    let hash = blake3::hash(clean_content.as_bytes());
    hash.to_hex().to_string()
}
