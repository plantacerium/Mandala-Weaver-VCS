use crate::language::Language;

/// Traza la línea de dominio lógico desde el Bindu hacia el exterior.
/// Asigna un nombre de dominio (UI, Logic, DB, etc.) según el ángulo.
pub fn snap_to_nearest_domain(angle: f64) -> String {
    let normalized = ((angle % 360.0) + 360.0) % 360.0;
    
    match normalized {
        a if (0.0..45.0).contains(&a) || (315.0..360.0).contains(&a) => "Core".to_string(),
        a if (45.0..135.0).contains(&a) => "UI".to_string(),
        a if (135.0..225.0).contains(&a) => "Persistence".to_string(),
        a if (225.0..315.0).contains(&a) => "Network".to_string(),
        _ => "Unknown".to_string(),
    }
}

/// Maps a language to its default vector angle.
pub fn language_to_vector(language: Language) -> f64 {
    match language {
        Language::Rust => 45.0,
        Language::TypeScript | Language::JavaScript => 225.0,
        Language::Python => 135.0,
        Language::Go => 90.0,
        Language::Unknown => 0.0,
    }
}

/// Gets vector name for language.
pub fn language_vector_name(language: Language) -> &'static str {
    match language {
        Language::Rust => "CORE",
        Language::TypeScript | Language::JavaScript => "UI",
        Language::Python => "DATA",
        Language::Go => "IO",
        Language::Unknown => "MISC",
    }
}
