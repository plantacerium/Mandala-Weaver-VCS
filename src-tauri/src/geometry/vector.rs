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
        Language::Rust | Language::C | Language::Cpp | Language::Zig | Language::Haskell | Language::OCaml => 45.0,
        Language::Go | Language::Shell | Language::Erlang | Language::Elixir | Language::Nix | Language::Terraform => 90.0,
        Language::Python | Language::Ruby | Language::Lua | Language::Julia | Language::R | Language::Perl | Language::Clojure => 135.0,
        Language::Sql | Language::Json | Language::Yaml | Language::Toml | Language::GraphQL | Language::Protobuf => 180.0,
        Language::TypeScript | Language::JavaScript | Language::Html | Language::Css | Language::Dart | Language::Shader => 225.0,
        Language::Java | Language::CSharp | Language::Kotlin | Language::Swift | Language::Php | Language::Scala | Language::Solidity => 315.0,
        Language::Markdown | Language::Unknown => 0.0,
    }
}

/// Gets vector name for language.
pub fn language_vector_name(language: Language) -> &'static str {
    match language {
        Language::Rust | Language::C | Language::Cpp | Language::Zig | Language::Haskell | Language::OCaml => "CORE",
        Language::TypeScript | Language::JavaScript | Language::Html | Language::Css | Language::Dart | Language::Shader => "UI",
        Language::Python | Language::Sql | Language::Json | Language::Yaml | Language::Toml | Language::Julia | Language::R | Language::GraphQL | Language::Protobuf => "DATA",
        Language::Go | Language::Shell | Language::Lua | Language::Ruby | Language::Perl | Language::Erlang | Language::Elixir | Language::Nix | Language::Terraform => "IO",
        Language::Java | Language::CSharp | Language::Kotlin | Language::Swift | Language::Php | Language::Scala | Language::Solidity | Language::Clojure => "SERVICES",
        Language::Markdown | Language::Unknown => "MISC",
    }
}
