use crate::geometry::polar_coordinate::PolarCoordinate;
use crate::ontology::monad::Monad;
use crate::persistence::schemas::get_initialization_queries;
use serde::{Deserialize, Serialize};
use surrealdb::engine::local::{Db, Mem, SurrealKv};
use surrealdb::Surreal;
use thiserror::Error;

const DEFAULT_RING_SPACING: f64 = 100.0;

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("Error en el motor SurrealDB: {0}")]
    Database(#[from] surrealdb::Error),
    #[error("No se encontró una de las Mónadas para registrar el linaje evolutivo: {0}")]
    MonadNotFound(String),
    #[error("Error de serialización: {0}")]
    Serialization(String),
}

/// Estructura plana para serializar la geometría hacia SurrealDB sin pérdida de precisión SO(2)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct StoredCoordinate {
    pub ring: i64,
    pub radius_offset: f64,
    pub sector_re: f64,
    pub sector_im: f64,
    pub angle_rad: f64,
    pub cartesian_x: f64,
    pub cartesian_y: f64,
    #[serde(default)]
    pub r: f64,
    #[serde(default)]
    pub theta: f64,
}

impl From<&PolarCoordinate> for StoredCoordinate {
    fn from(pc: &PolarCoordinate) -> Self {
        let cart = pc.to_cartesian(DEFAULT_RING_SPACING);
        let effective_r = (pc.ring.0 as f64 * DEFAULT_RING_SPACING) + pc.radius_offset;
        let theta_deg = pc.angle_degrees();
        Self {
            ring: pc.ring.0 as i64,
            radius_offset: pc.radius_offset,
            sector_re: pc.sector.re,
            sector_im: pc.sector.im,
            angle_rad: pc.angle_radians(),
            cartesian_x: cart.x,
            cartesian_y: cart.y,
            r: effective_r,
            theta: theta_deg,
        }
    }
}

impl From<&StoredCoordinate> for PolarCoordinate {
    fn from(sc: &StoredCoordinate) -> Self {
        PolarCoordinate::new(sc.ring as u32, sc.radius_offset, sc.angle_rad)
    }
}

/// DTO de lectura para recuperar una Mónada completa desde SurrealDB con coordenadas continuas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedMonad {
    #[serde(default)]
    pub id: Option<serde_json::Value>,
    #[serde(default)]
    pub symbol_name: String,
    #[serde(default)]
    pub implementation_hash: String,
    #[serde(default)]
    pub contract_hash: String,
    #[serde(default)]
    pub canonical_signature: String,
    #[serde(default)]
    pub raw_source: String,
    #[serde(default)]
    pub ring: u32,
    #[serde(default)]
    pub coord: StoredCoordinate,
}

/// Resultado de la resolución de dependencias por navegación de grafo tripartito
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyResolution {
    pub required_symbol: String,
    pub available_contracts: Vec<String>,
    pub candidate_monads: Vec<PersistedMonad>,
}

/// Mónada Puente candidata devuelta por adapts_contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeCandidate {
    pub bridge_monad: PersistedMonad,
    #[serde(default)]
    pub is_lossy: bool,
    #[serde(default)]
    pub penalty_rad: f64,
}

/// Gestor del Grafo Tripartito Ontológico (`monad` - `contract` - `port`) en SurrealDB
pub struct MandalaStore {
    db: Surreal<Db>,
}

impl MandalaStore {
    /// Inicializa el almacén en disco en la ruta indicada (ej. `.mandala/db`)
    pub async fn open_disk<P: AsRef<std::path::Path>>(path: P) -> Result<Self, PersistenceError> {
        let db_path = path.as_ref();
        std::fs::create_dir_all(db_path).ok();
        let db = Surreal::new::<SurrealKv>(db_path.to_string_lossy().to_string()).await?;
        db.use_ns("mandala").use_db("weaver").await?;
        for query in get_initialization_queries() {
            db.query(query).await?;
        }
        Ok(Self { db })
    }

    /// Inicializa el almacén en memoria para tests y simulaciones efímeras
    pub async fn open_in_memory() -> Result<Self, PersistenceError> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns("mandala").use_db("weaver").await?;
        for query in get_initialization_queries() {
            db.query(query).await?;
        }
        Ok(Self { db })
    }

    /// Envuelve una conexión existente de SurrealDB
    pub fn from_existing_db(db: Surreal<Db>) -> Self {
        Self { db }
    }

    /// Obtiene una referencia a la conexión subyacente de SurrealDB
    pub fn db(&self) -> &Surreal<Db> {
        &self.db
    }

    /// Persiste una Mónada, su Contrato inmutable, su Puerto nominal y todas sus llamadas salientes
    /// en UNA SOLA transacción atómica de SurrealQL para máxima velocidad y coherencia referencial.
    pub async fn upsert_monad(
        &self,
        monad: &Monad,
        coord: &PolarCoordinate,
    ) -> Result<(), PersistenceError> {
        let symbol_name = if !monad.name.is_empty() {
            &monad.name
        } else {
            "unnamed_symbol"
        };

        let contract_hash = if !monad.contract_hash.is_empty() {
            monad.contract_hash.clone()
        } else {
            crate::ontology::semantic_hash::generate_pure_hash(symbol_name)
        };

        let impl_hash = if !monad.implementation_hash.is_empty() {
            monad.implementation_hash.clone()
        } else if !monad.id.is_empty() {
            monad.id.clone()
        } else {
            crate::ontology::semantic_hash::generate_pure_hash(&monad.content)
        };

        let canonical_sig = if !monad.canonical_signature.is_empty() {
            monad.canonical_signature.clone()
        } else {
            format!("fn {}()", symbol_name)
        };

        let stored_coord = StoredCoordinate::from(coord);
        let outbound_list = monad.outbound_calls.clone();

        // 1. Asegurar el puerto nominal expuesto
        let port_content = serde_json::json!({
            "symbol_name": symbol_name,
        });
        let _: Option<serde_json::Value> = self.db.upsert(("port", symbol_name)).content(port_content).await?;

        // 2. Asegurar el contrato inmutable
        let contract_exists: Option<serde_json::Value> = self.db.select(("contract", contract_hash.as_str())).await.ok().flatten();
        if contract_exists.is_none() {
            let contract_content = serde_json::json!({
                "contract_hash": contract_hash,
                "symbol_name": symbol_name,
                "canonical_signature": canonical_sig,
            });
            let _: Option<serde_json::Value> = self.db.upsert(("contract", contract_hash.as_str())).content(contract_content).await?;
        }

        // 3. Asegurar la Mónada direccionada por implementation_hash
        let monad_content = serde_json::json!({
            "id": impl_hash,
            "name": symbol_name,
            "symbol_name": symbol_name,
            "implementation_hash": impl_hash,
            "semantic_hash": impl_hash,
            "contract_hash": contract_hash,
            "canonical_signature": canonical_sig,
            "content": monad.content,
            "raw_source": monad.content,
            "kind": monad.kind.to_string(),
            "ring": coord.ring.0,
            "line_start": monad.line_start,
            "line_end": monad.line_end,
            "language": monad.language,
            "is_archived": false,
            "outbound_calls": outbound_list,
            "coord": stored_coord,
        });
        let _: Option<serde_json::Value> = self.db.upsert(("monad", impl_hash.as_str())).content(monad_content).await?;

        // 4. Conectar Mónada -> implements -> Contrato -> exposes_port -> Puerto
        let _ = self.db
            .query(r#"
                LET $m = type::record("monad", $impl_hash);
                LET $c = type::record("contract", $c_hash);
                LET $p = type::record("port", $sym);
                RELATE $m -> implements -> $c;
                RELATE $c -> exposes_port -> $p;
            "#)
            .bind(("impl_hash", impl_hash.clone()))
            .bind(("c_hash", contract_hash.clone()))
            .bind(("sym", symbol_name.to_string()))
            .await?
            .check()?;

        // 5. Conectar dependencias salientes: Mónada -> requires_port -> Puerto destino
        for target_sym in &outbound_list {
            if target_sym.is_empty() || target_sym == symbol_name {
                continue;
            }
            let target_port_content = serde_json::json!({
                "symbol_name": target_sym,
            });
            let _: Result<Option<serde_json::Value>, _> = self.db.upsert(("port", target_sym.as_str())).content(target_port_content).await;

            let _ = self.db
                .query(r#"
                    LET $m = type::record("monad", $impl_hash);
                    LET $p = type::record("port", $target_sym);
                    RELATE $m -> requires_port -> $p;
                "#)
                .bind(("impl_hash", impl_hash.clone()))
                .bind(("target_sym", target_sym.clone()))
                .await?
                .check()?;
        }

        Ok(())
    }

    /// Registra una arista evolutiva con deriva geométrica SO(2) continua y flag de preservación de contrato
    pub async fn record_evolution(
        &self,
        parent_impl_hash: &str,
        child_impl_hash: &str,
        reason: &str,
    ) -> Result<(), PersistenceError> {
        let parent_val: Option<serde_json::Value> = self.db.select(("monad", parent_impl_hash)).await?;
        let parent: Option<PersistedMonad> = parent_val.and_then(|v| serde_json::from_value(v).ok());

        let child_val: Option<serde_json::Value> = self.db.select(("monad", child_impl_hash)).await?;
        let child: Option<PersistedMonad> = child_val.and_then(|v| serde_json::from_value(v).ok());

        let (parent, child) = match (parent, child) {
            (Some(p), Some(c)) => (p, c),
            (None, _) => return Err(PersistenceError::MonadNotFound(parent_impl_hash.to_string())),
            (_, None) => return Err(PersistenceError::MonadNotFound(child_impl_hash.to_string())),
        };

        // Reconstruir coordenadas polares en SO(2)
        let parent_polar = PolarCoordinate::from(&parent.coord);
        let child_polar = PolarCoordinate::from(&child.coord);

        let ring_delta = (child.coord.ring - parent.coord.ring) as i32;
        let angular_drift = parent_polar.angular_distance(&child_polar);
        let contract_preserved = parent.contract_hash == child.contract_hash;

        let surql = r#"
            LET $c = type::record('monad', $child_hash);
            LET $p = type::record('monad', $parent_hash);
            RELATE $c -> evolves_from -> $p SET
                ring_delta = $ring_delta,
                angular_drift_rad = $angular_drift,
                contract_preserved = $contract_preserved,
                reason = $reason,
                timestamp = time::now();
        "#;

        self.db
            .query(surql)
            .bind(("child_hash", child_impl_hash.to_string()))
            .bind(("parent_hash", parent_impl_hash.to_string()))
            .bind(("ring_delta", ring_delta))
            .bind(("angular_drift", angular_drift))
            .bind(("contract_preserved", contract_preserved))
            .bind(("reason", reason.to_string()))
            .await?
            .check()?;

        Ok(())
    }

    /// Resuelve en una sola consulta de grafo qué Mónadas candidatas satisfacen las llamadas salientes
    /// de una Mónada dentro de un nivel de anillo máximo especificado
    pub async fn resolve_dependencies_for_monad(
        &self,
        impl_hash: &str,
        max_ring: u32,
    ) -> Result<Vec<DependencyResolution>, PersistenceError> {
        // Consultar puertos requeridos
        let mut ports_resp = self.db
            .query("SELECT VALUE ->requires_port->port.symbol_name FROM type::record('monad', $hash);")
            .bind(("hash", impl_hash.to_string()))
            .await?
            .check()?;

        let ports_raw: Vec<serde_json::Value> = ports_resp.take(0).unwrap_or_default();
        let mut required_symbols = Vec::new();
        for item in ports_raw {
            if let Some(s) = item.as_str() {
                required_symbols.push(s.to_string());
            } else if let Some(sub_arr) = item.as_array() {
                for sub in sub_arr {
                    if let Some(s) = sub.as_str() {
                        required_symbols.push(s.to_string());
                    }
                }
            }
        }

        let mut resolutions = Vec::new();

        for sym in required_symbols {
            // Contratos que exponen este puerto
            let mut contracts_resp = self.db
                .query("SELECT VALUE <-exposes_port<-contract.contract_hash FROM type::record('port', $sym);")
                .bind(("sym", sym.clone()))
                .await?
                .check()?;

            let contracts_raw: Vec<serde_json::Value> = contracts_resp.take(0).unwrap_or_default();
            let mut available_contracts = Vec::new();
            for item in contracts_raw {
                if let Some(s) = item.as_str() {
                    available_contracts.push(s.to_string());
                } else if let Some(sub_arr) = item.as_array() {
                    for sub in sub_arr {
                        if let Some(s) = sub.as_str() {
                            available_contracts.push(s.to_string());
                        }
                    }
                }
            }


            // Mónadas candidatas que implementan esos contratos dentro del anillo máximo
            let mut monads_resp = self.db
                .query("SELECT * FROM monad WHERE symbol_name = $sym;")
                .bind(("sym", sym.clone()))
                .await?
                .check()?;
            let monads_raw: Vec<serde_json::Value> = monads_resp.take(0).unwrap_or_default();
            let candidate_monads: Vec<PersistedMonad> = monads_raw
                .into_iter()
                .filter_map(|v| serde_json::from_value(v).ok())
                .filter(|m: &PersistedMonad| m.ring <= max_ring)
                .collect();

            resolutions.push(DependencyResolution {
                required_symbol: sym,
                available_contracts,
                candidate_monads,
            });
        }

        Ok(resolutions)
    }

    /// Obtiene una Mónada específica por su implementation_hash exacto (usado en `pins`)
    pub async fn get_monad_by_hash(
        &self,
        impl_hash: &str,
    ) -> Result<Option<PersistedMonad>, PersistenceError> {
        let val: Option<serde_json::Value> = self.db.select(("monad", impl_hash)).await?;
        let monad = val.and_then(|v| serde_json::from_value(v).ok());
        Ok(monad)
    }

    /// Obtiene todas las Mónadas candidatas para un símbolo inicial dentro del anillo permitido
    pub async fn get_candidates_for_symbol(
        &self,
        symbol_name: &str,
        max_ring: u32,
    ) -> Result<Vec<PersistedMonad>, PersistenceError> {
        let surql = r#"
            SELECT * FROM monad
            WHERE symbol_name = $symbol_name;
        "#;

        let mut response = self
            .db
            .query(surql)
            .bind(("symbol_name", symbol_name.to_string()))
            .await?
            .check()?;

        let monads_raw: Vec<serde_json::Value> = response.take(0).unwrap_or_default();
        let candidates: Vec<PersistedMonad> = monads_raw
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .filter(|m: &PersistedMonad| m.coord.ring <= max_ring as i64 || m.ring <= max_ring)
            .collect();
        Ok(candidates)
    }

    /// Registra una Mónada como puente entre `expected_contract_hash` y `actual_contract_hash`
    pub async fn register_bridge_monad(
        &self,
        bridge: &Monad,
        coord: &PolarCoordinate,
        expected_contract_hash: &str,
        actual_contract_hash: &str,
        target_symbol: &str,
        is_lossy: bool,
        penalty_rad: f64,
    ) -> Result<(), PersistenceError> {
        // 1. Persistimos la Mónada Puente en el espacio polar
        self.upsert_monad(bridge, coord).await?;

        let bridge_impl_hash = if !bridge.implementation_hash.is_empty() {
            &bridge.implementation_hash
        } else {
            &bridge.id
        };

        // 2. Creamos la arista dirigida entre ambos contratos apuntando a esta Mónada
        let surql = r#"
            LET $expected = type::record('contract', $expected_hash);
            LET $actual = type::record('contract', $actual_hash);
            LET $bridge_rec = type::record('monad', $bridge_impl_hash);

            RELATE $expected -> adapts_contract -> $actual SET
                bridge_monad = $bridge_rec,
                target_symbol = $target_symbol,
                is_lossy = $is_lossy,
                penalty_rad = $penalty_rad;
        "#;

        self.db
            .query(surql)
            .bind(("expected_hash", expected_contract_hash.to_string()))
            .bind(("actual_hash", actual_contract_hash.to_string()))
            .bind(("bridge_impl_hash", bridge_impl_hash.to_string()))
            .bind(("target_symbol", target_symbol.to_string()))
            .bind(("is_lossy", is_lossy))
            .bind(("penalty_rad", penalty_rad))
            .await?
            .check()?;

        Ok(())
    }

    /// Vincula explícitamente qué `contract_hash` espera una Mónada al llamar a un símbolo
    pub async fn bind_expected_contract(
        &self,
        caller_impl_hash: &str,
        symbol_name: &str,
        expected_contract_hash: &str,
    ) -> Result<(), PersistenceError> {
        let contract_exists: Option<serde_json::Value> = self.db.select(("contract", expected_contract_hash)).await.ok().flatten();
        if contract_exists.is_none() {
            let contract_content = serde_json::json!({
                "contract_hash": expected_contract_hash,
                "symbol_name": symbol_name,
                "canonical_signature": format!("fn {}", symbol_name),
            });
            let _: Option<serde_json::Value> = self.db.upsert(("contract", expected_contract_hash)).content(contract_content).await?;
        }

        let surql = r#"
            LET $caller = type::record('monad', $caller_hash);
            LET $contract = type::record('contract', $contract_hash);
            RELATE $caller -> requires_contract -> $contract SET symbol_name = $symbol_name;
        "#;

        self.db
            .query(surql)
            .bind(("caller_hash", caller_impl_hash.to_string()))
            .bind(("contract_hash", expected_contract_hash.to_string()))
            .bind(("symbol_name", symbol_name.to_string()))
            .await?
            .check()?;

        Ok(())
    }

    /// Consulta qué contract_hash específico exige una Mónada para un símbolo dado (si fue fijado)
    pub async fn get_expected_contract(
        &self,
        caller_impl_hash: &str,
        symbol_name: &str,
    ) -> Result<Option<String>, PersistenceError> {
        let surql = r#"
            SELECT VALUE IF out.contract_hash != NONE AND out.contract_hash != NULL THEN out.contract_hash ELSE record::id(out) END
            FROM requires_contract
            WHERE in = type::record('monad', $caller_hash)
              AND symbol_name = $symbol_name
            LIMIT 1;
        "#;

        let mut res = self
            .db
            .query(surql)
            .bind(("caller_hash", caller_impl_hash.to_string()))
            .bind(("symbol_name", symbol_name.to_string()))
            .await?
            .check()?;

        let raw: Vec<serde_json::Value> = res.take(0).unwrap_or_default();
        for item in raw {
            if let Some(s) = item.as_str() {
                if !s.is_empty() {
                    return Ok(Some(s.to_string()));
                }
            } else if let Some(sub_arr) = item.as_array() {
                if let Some(first) = sub_arr.first().and_then(|v| v.as_str()) {
                    if !first.is_empty() {
                        return Ok(Some(first.to_string()));
                    }
                }
            } else if let Some(obj) = item.as_object() {
                if let Some(id_val) = obj.get("id").and_then(|v| v.as_str()) {
                    return Ok(Some(id_val.to_string()));
                }
            }
        }
        Ok(None)
    }

    /// Busca todas las Mónadas Puente capaces de adaptar `expected_contract` -> `actual_contract`
    /// dentro del anillo máximo permitido por la receta YAML.
    pub async fn find_bridge_candidates(
        &self,
        expected_contract_hash: &str,
        actual_contract_hash: &str,
        max_ring: u32,
    ) -> Result<Vec<BridgeCandidate>, PersistenceError> {
        let surql = r#"
            SELECT
                bridge_monad.* AS bridge_monad,
                is_lossy,
                penalty_rad
            FROM adapts_contract
            WHERE in = type::record('contract', $expected_hash)
              AND out = type::record('contract', $actual_hash);
        "#;

        let mut res = self
            .db
            .query(surql)
            .bind(("expected_hash", expected_contract_hash.to_string()))
            .bind(("actual_hash", actual_contract_hash.to_string()))
            .await?
            .check()?;

        let raw: Vec<serde_json::Value> = res.take(0).unwrap_or_default();
        let candidates: Vec<BridgeCandidate> = raw
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .filter(|c: &BridgeCandidate| c.bridge_monad.coord.ring <= max_ring as i64 || c.bridge_monad.ring <= max_ring)
            .collect();
        Ok(candidates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[tokio::test]
    async fn test_tripartite_graph_upsert_and_resolve() {
        let store = MandalaStore::open_in_memory().await.unwrap();

        // 1. Crear mónada fundacional 'normalizar' en Ring 1
        let mut m1 = Monad::spawn(
            "hash_norm_v1".to_string(),
            "normalizar".to_string(),
            crate::geometry::polar_space::PolarCoord::new(100.0, 0.0),
            "pub fn normalizar(r: f64) -> f64 { r / 10.0 }".to_string(),
            1,
        );
        m1.contract_hash = "contract_norm_f64".to_string();
        m1.implementation_hash = "hash_norm_v1".to_string();
        m1.canonical_signature = "fn normalizar(f64) -> f64".to_string();

        let coord1 = PolarCoordinate::new(1, 0.0, 0.0);
        store.upsert_monad(&m1, &coord1).await.unwrap();

        // 2. Crear mónada consumidora 'calcular' en Ring 2 que invoca 'normalizar'
        let mut m2 = Monad::spawn(
            "hash_calc_v1".to_string(),
            "calcular".to_string(),
            crate::geometry::polar_space::PolarCoord::new(200.0, 45.0),
            "pub fn calcular(x: f64) -> f64 { normalizar(x) * 2.0 }".to_string(),
            2,
        );
        m2.contract_hash = "contract_calc_f64".to_string();
        m2.implementation_hash = "hash_calc_v1".to_string();
        m2.canonical_signature = "fn calcular(f64) -> f64".to_string();
        m2.outbound_calls = vec!["normalizar".to_string()];

        let coord2 = PolarCoordinate::new(2, 0.0, PI / 4.0);
        store.upsert_monad(&m2, &coord2).await.unwrap();

        // 3. Resolver dependencias para 'calcular' en Ring 1
        let resolutions = store
            .resolve_dependencies_for_monad("hash_calc_v1", 1)
            .await
            .unwrap();
        println!(">>> resolutions = {:?}", resolutions);

        assert_eq!(resolutions.len(), 1);
        assert_eq!(resolutions[0].required_symbol, "normalizar");
        assert!(resolutions[0].available_contracts.contains(&"contract_norm_f64".to_string()));
        assert_eq!(resolutions[0].candidate_monads.len(), 1);
        assert_eq!(resolutions[0].candidate_monads[0].symbol_name, "normalizar");

        // 4. Probar registro evolutivo entre dos versiones con deriva geométrica SO(2)
        let mut m1_v2 = Monad::spawn(
            "hash_norm_v2".to_string(),
            "normalizar".to_string(),
            crate::geometry::polar_space::PolarCoord::new(200.0, 15.0),
            "pub fn normalizar(r: f64) -> f64 { r * 0.1 }".to_string(),
            2,
        );
        m1_v2.contract_hash = "contract_norm_f64".to_string(); // Contrato preservado
        m1_v2.implementation_hash = "hash_norm_v2".to_string();
        m1_v2.canonical_signature = "fn normalizar(f64) -> f64".to_string();

        let coord1_v2 = PolarCoordinate::new(2, 0.0, 15.0_f64.to_radians());
        store.upsert_monad(&m1_v2, &coord1_v2).await.unwrap();

        store
            .record_evolution("hash_norm_v1", "hash_norm_v2", "Optimizacion matematica")
            .await
            .unwrap();

        let mut evo_resp = store.db.query("SELECT * FROM evolves_from;").await.unwrap();
        let evo_rows: Vec<serde_json::Value> = evo_resp.take(0).unwrap_or_default();
        assert_eq!(evo_rows.len(), 1);
        assert_eq!(evo_rows[0].get("ring_delta").and_then(|v| v.as_i64()), Some(1));
        assert_eq!(evo_rows[0].get("contract_preserved").and_then(|v| v.as_bool()), Some(true));
        assert!(evo_rows[0].get("angular_drift_rad").and_then(|v| v.as_f64()).unwrap() > 0.0);
    }
}
