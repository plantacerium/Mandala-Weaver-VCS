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
        "DEFINE FIELD contract_hash ON TABLE monad TYPE string DEFAULT '';",
        "DEFINE FIELD implementation_hash ON TABLE monad TYPE string DEFAULT '';",
        "DEFINE FIELD canonical_signature ON TABLE monad TYPE string DEFAULT '';",
        "DEFINE FIELD outbound_calls ON TABLE monad TYPE array DEFAULT [];",
        "DEFINE FIELD symbol_name ON TABLE monad TYPE string DEFAULT '';",
        "DEFINE FIELD raw_source ON TABLE monad TYPE string DEFAULT '';",
        "DEFINE FIELD coord.ring ON TABLE monad TYPE option<int>;",
        "DEFINE FIELD coord.radius_offset ON TABLE monad TYPE option<float>;",
        "DEFINE FIELD coord.sector_re ON TABLE monad TYPE option<float>;",
        "DEFINE FIELD coord.sector_im ON TABLE monad TYPE option<float>;",
        "DEFINE FIELD coord.angle_rad ON TABLE monad TYPE option<float>;",
        "DEFINE FIELD coord.cartesian_x ON TABLE monad TYPE option<float>;",
        "DEFINE FIELD coord.cartesian_y ON TABLE monad TYPE option<float>;",
        "DEFINE FIELD identity ON TABLE monad TYPE option<object>;",
        "DEFINE FIELD identity.contract_hash ON TABLE monad TYPE option<string>;",
        "DEFINE FIELD identity.implementation_hash ON TABLE monad TYPE option<string>;",
        "DEFINE FIELD identity.canonical_signature ON TABLE monad TYPE option<string>;",

        // ==========================================
        // GRAFO TRIPARTITO ONTOLÓGICO (Fase 2)
        // ==========================================
        // 1. Puerto nominal (símbolos expuestos o requeridos por AST)
        "DEFINE TABLE port SCHEMAFULL;",
        "DEFINE FIELD symbol_name ON TABLE port TYPE string;",
        "DEFINE INDEX idx_port_symbol ON TABLE port COLUMNS symbol_name UNIQUE;",

        // 2. Contrato inmutable (definido por contract_hash y firma canónica)
        "DEFINE TABLE contract SCHEMAFULL;",
        "DEFINE FIELD contract_hash ON TABLE contract TYPE string;",
        "DEFINE FIELD symbol_name ON TABLE contract TYPE string;",
        "DEFINE FIELD canonical_signature ON TABLE contract TYPE string;",
        "DEFINE FIELD created_at ON TABLE contract TYPE option<datetime> DEFAULT time::now();",
        "DEFINE INDEX idx_contract_hash ON TABLE contract COLUMNS contract_hash UNIQUE;",

        // 3. Aristas del Grafo Tripartito
        "DEFINE TABLE implements TYPE RELATION FROM monad TO contract;",
        "DEFINE TABLE exposes_port TYPE RELATION FROM contract TO port;",
        "DEFINE TABLE requires_port TYPE RELATION FROM monad TO port;",

        // 4. Arista evolutiva con métricas de deriva geométrica y preservación de contrato
        "DEFINE TABLE evolves_from TYPE RELATION FROM monad TO monad;",
        "DEFINE FIELD ring_delta ON TABLE evolves_from TYPE int;",
        "DEFINE FIELD angular_drift_rad ON TABLE evolves_from TYPE float;",
        "DEFINE FIELD contract_preserved ON TABLE evolves_from TYPE bool;",
        "DEFINE FIELD reason ON TABLE evolves_from TYPE option<string>;",
        "DEFINE FIELD timestamp ON TABLE evolves_from TYPE option<datetime> DEFAULT time::now();",
        
        // Definición de la relación evolutiva legacy (compatibilidad)
        "DEFINE TABLE evolves_to TYPE RELATION FROM monad TO monad;",

        // ==========================================
        // EXTENSIÓN DE ARISTAS PARA MÓNADAS PUENTE
        // ==========================================
        "DEFINE TABLE requires_contract TYPE RELATION FROM monad TO contract;",
        "DEFINE FIELD symbol_name ON TABLE requires_contract TYPE string;",

        "DEFINE TABLE adapts_contract TYPE RELATION FROM contract TO contract;",
        "DEFINE FIELD bridge_monad ON TABLE adapts_contract TYPE record<monad>;",
        "DEFINE FIELD target_symbol ON TABLE adapts_contract TYPE string;",
        "DEFINE FIELD is_lossy ON TABLE adapts_contract TYPE bool DEFAULT false;",
        "DEFINE FIELD penalty_rad ON TABLE adapts_contract TYPE float DEFAULT 0.1;",

        // Definición de la tabla Bindu (proyecto)
        "DEFINE TABLE bindu SCHEMAFULL;",
        "DEFINE FIELD project_name ON TABLE bindu TYPE string;",
        "DEFINE FIELD timestamp ON TABLE bindu TYPE int;",

        // Operation log for sync tracking
        "DEFINE TABLE operation_log SCHEMAFULL;",
        "DEFINE FIELD op_type ON TABLE operation_log TYPE string;",
        "DEFINE FIELD monad_id ON TABLE operation_log TYPE string;",
        "DEFINE FIELD timestamp ON TABLE operation_log TYPE int;",
        "DEFINE FIELD payload ON TABLE operation_log TYPE object;",
        
        // Índices para búsqueda rápida radial
        "DEFINE INDEX idx_monad_ring ON TABLE monad COLUMNS ring;",
        "DEFINE INDEX idx_monad_name ON TABLE monad COLUMNS name;",
        "DEFINE INDEX idx_monad_hash ON TABLE monad COLUMNS semantic_hash;",
        "DEFINE INDEX idx_monad_ring_theta ON TABLE monad COLUMNS ring, coord.theta;",
        "DEFINE INDEX idx_monad_name_ring ON TABLE monad COLUMNS name, ring;",
        "DEFINE INDEX idx_monad_contract ON TABLE monad COLUMNS contract_hash;",
        "DEFINE INDEX idx_monad_impl ON TABLE monad COLUMNS implementation_hash;",
        "DEFINE INDEX idx_oplog_ts ON TABLE operation_log COLUMNS timestamp;",
    ]
}
