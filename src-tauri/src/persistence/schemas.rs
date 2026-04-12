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
        
        // Definición de la relación evolutiva
        "DEFINE TABLE evolves_to TYPE RELATION FROM monad TO monad;",
        
        // Índices para búsqueda rápida radial
        "DEFINE INDEX idx_monad_ring ON TABLE monad COLUMNS ring;",
    ]
}
