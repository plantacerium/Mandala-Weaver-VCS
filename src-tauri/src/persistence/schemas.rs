/// Retorna los scripts SurrealQL para inicializar las tablas radiales.
pub fn get_initialization_queries() -> Vec<&'static str> {
    vec![
        // Definición de la tabla Monada
        "DEFINE TABLE monad SCHEMAFULL;",
        "DEFINE FIELD id ON TABLE monad TYPE string;",
        "DEFINE FIELD name ON TABLE monad TYPE string;",
        "DEFINE FIELD content ON TABLE monad TYPE string;",
        "DEFINE FIELD ring ON TABLE monad TYPE int;",
        "DEFINE FIELD coord ON TABLE monad TYPE object;",
        "DEFINE FIELD coord.r ON TABLE monad TYPE float;",
        "DEFINE FIELD coord.theta ON TABLE monad TYPE float;",
        "DEFINE FIELD kind ON TABLE monad TYPE string;",
        "DEFINE FIELD semantic_hash ON TABLE monad TYPE string;",
        "DEFINE FIELD line_start ON TABLE monad TYPE int;",
        "DEFINE FIELD line_end ON TABLE monad TYPE int;",
        "DEFINE FIELD language ON TABLE monad TYPE string;",
        "DEFINE FIELD is_archived ON TABLE monad TYPE bool DEFAULT false;",
        
        // Definición de la relación evolutiva
        "DEFINE TABLE evolves_to TYPE RELATION FROM monad TO monad;",

        // Definición de la tabla Bindu (proyecto)
        "DEFINE TABLE bindu SCHEMAFULL;",
        "DEFINE FIELD project_name ON TABLE bindu TYPE string;",
        "DEFINE FIELD timestamp ON TABLE bindu TYPE int;",
        
        // Índices para búsqueda rápida radial
        "DEFINE INDEX idx_monad_ring ON TABLE monad COLUMNS ring;",
        "DEFINE INDEX idx_monad_name ON TABLE monad COLUMNS name;",
        "DEFINE INDEX idx_monad_hash ON TABLE monad COLUMNS semantic_hash;",
    ]
}
