use crate::geometry::PolarCoordinate;
use crate::persistence::{MandalaStore, PersistedMonad, PersistenceError};
use petgraph::algo::tarjan_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WeaveError {
    #[error("Error al parsear la plantilla YAML: {0}")]
    YamlError(#[from] serde_yml::Error),
    #[error("Error de persistencia en SurrealDB: {0}")]
    StorageError(#[from] PersistenceError),
    #[error("Símbolo requerido '{symbol}' (invocado por '{caller}') no tiene ninguna Mónada en anillo <= {max_ring}")]
    UnresolvedPort {
        symbol: String,
        caller: String,
        max_ring: u32,
    },
    #[error("La Mónada fijada (pin) para '{symbol}' con hash '{hash}' no existe en SurrealDB")]
    PinnedMonadNotFound { symbol: String, hash: String },
    #[error("Colisión de contratos para el símbolo '{symbol}': se requieren contratos incompatibles {contract_a} y {contract_b}")]
    ContractCollision {
        symbol: String,
        contract_a: String,
        contract_b: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AngularPolicy {
    /// Elige la Mónada más cercana al ángulo global de la receta (`target_angle_rad`)
    GlobalTarget,
    /// Elige la Mónada más cercana al ángulo de la Mónada padre que realiza la llamada
    CallerLocal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedSpec {
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationRecipe {
    pub name: String,
    pub target_angle_rad: f64,
    pub max_ring: u32,
    pub angular_policy: AngularPolicy,
    #[serde(default)]
    pub ring_penalty_rad: f64,
    #[serde(default)]
    pub prelude: String,
    pub seeds: Vec<SeedSpec>,
    #[serde(default)]
    pub pins: BTreeMap<String, String>,
    #[serde(default)]
    pub external_symbols: BTreeSet<String>,
}

pub fn is_default_prelude_symbol(s: &str) -> bool {
    matches!(
        s,
        "Ok" | "Err" | "Some" | "None" | "println" | "eprintln" | "format" | "panic"
        | "assert" | "assert_eq" | "assert_ne" | "todo" | "unimplemented" | "unreachable" | "vec"
    )
}

impl DistillationRecipe {
    pub fn is_external(&self, symbol: &str) -> bool {
        self.external_symbols.contains(symbol) || is_default_prelude_symbol(symbol)
    }
}

/// Diagnóstico de selección de cada nodo para auditoría de la destilación
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WovenNodeTrace {
    pub symbol_name: String,
    pub implementation_hash: String,
    pub contract_hash: String,
    pub selected_ring: i64,
    pub selected_angle_rad: f64,
    pub angular_distance_rad: f64,
    pub alternatives_considered: usize,
}

/// Registro de auditoría cuando el Weaver sintetiza un puente automático
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WovenBridgeTrace {
    pub caller_symbol: String,
    pub target_symbol: String,
    pub bridge_alias: String,
    pub bridge_impl_hash: String,
    pub from_expected_contract: String,
    pub to_actual_contract: String,
    pub is_lossy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistilledArtifact {
    pub recipe_name: String,
    pub assembled_source: String,
    pub traces: Vec<WovenNodeTrace>,
    #[serde(default)]
    pub bridge_traces: Vec<WovenBridgeTrace>,
}

pub struct Weaver<'a> {
    store: &'a MandalaStore,
}

impl<'a> Weaver<'a> {
    pub fn new(store: &'a MandalaStore) -> Self {
        Self { store }
    }

    /// Lee una receta YAML en texto y ejecuta el proceso completo de destilación espacial
    pub async fn distill_from_yaml(
        &self,
        yaml_content: &str,
    ) -> Result<DistilledArtifact, WeaveError> {
        let recipe: DistillationRecipe = serde_yml::from_str(yaml_content)?;
        self.distill(&recipe).await
    }

    pub async fn distill(
        &self,
        recipe: &DistillationRecipe,
    ) -> Result<DistilledArtifact, WeaveError> {
        // Mapa symbol_name -> Mónada seleccionada (garantiza 1 sola implementación por puerto nominal)
        let mut resolved_symbols: HashMap<String, PersistedMonad> = HashMap::new();
        let mut traces_by_symbol: HashMap<String, WovenNodeTrace> = HashMap::new();

        // Grafo dirigido de llamadas: Caller Symbol -> Callee Symbol
        let mut dep_graph: DiGraph<String, ()> = DiGraph::new();
        let mut graph_nodes: HashMap<String, NodeIndex> = HashMap::new();

        // Cola BFS: (Mónada actual a expandir)
        let mut queue: VecDeque<PersistedMonad> = VecDeque::new();

        let global_ref_polar = PolarCoordinate::new(0, 0.0, recipe.target_angle_rad);

        let mut bridge_traces: Vec<WovenBridgeTrace> = Vec::new();

        // 1. Resolver las Mónadas Semilla (Seeds)
        for seed in &recipe.seeds {
            let (seed_monad, trace) = self
                .resolve_single_symbol(
                    &seed.symbol,
                    "<seed>",
                    &global_ref_polar,
                    recipe,
                    None,
                )
                .await?;

            let sym = seed_monad.symbol_name.clone();
            let _ = *graph_nodes
                .entry(sym.clone())
                .or_insert_with(|| dep_graph.add_node(sym.clone()));

            resolved_symbols.insert(sym.clone(), seed_monad.clone());
            traces_by_symbol.insert(sym, trace);
            queue.push_back(seed_monad);
        }

        // 2. Recorrido BFS consultando SurrealDB mediante `resolve_dependencies_for_monad`
        while let Some(caller_monad) = queue.pop_front() {
            let caller_sym = caller_monad.symbol_name.clone();
            let caller_idx = graph_nodes[&caller_sym];
            let caller_polar = PolarCoordinate::from(&caller_monad.coord);

            // Determinamos el vector angular de referencia según la política del YAML
            let reference_polar = match recipe.angular_policy {
                AngularPolicy::GlobalTarget => global_ref_polar,
                AngularPolicy::CallerLocal => caller_polar,
            };

            let dep_resolutions = self
                .store
                .resolve_dependencies_for_monad(&caller_monad.implementation_hash, recipe.max_ring)
                .await?;

            for dep in dep_resolutions {
                let req_sym = dep.required_symbol;

                // Ignorar llamadas a la librería estándar o crates externos declarados
                if recipe.is_external(&req_sym) {
                    continue;
                }

                // Si el símbolo aún no ha sido tejido en esta sesión, seleccionamos el óptimo angular
                let selected = if !resolved_symbols.contains_key(&req_sym) {
                    let (s, t) = self
                        .resolve_single_symbol(
                            &req_sym,
                            &caller_sym,
                            &reference_polar,
                            recipe,
                            Some(dep.candidate_monads),
                        )
                        .await?;

                    graph_nodes
                        .entry(req_sym.clone())
                        .or_insert_with(|| dep_graph.add_node(req_sym.clone()));

                    resolved_symbols.insert(req_sym.clone(), s.clone());
                    traces_by_symbol.insert(req_sym.clone(), t);
                    queue.push_back(s.clone());
                    s
                } else {
                    resolved_symbols[&req_sym].clone()
                };

                let callee_idx = graph_nodes[&req_sym];

                // Comprobamos si el invocador exigió un contrato específico para este puerto
                let expected_contract_opt = self
                    .store
                    .get_expected_contract(&caller_monad.implementation_hash, &req_sym)
                    .await?;

                if let Some(expected_contract) = expected_contract_opt {
                    if expected_contract != selected.contract_hash {
                        // ¡Colisión de contrato detectada! Buscamos e inyectamos la Mónada Puente
                        let caller_mut = resolved_symbols.get_mut(&caller_sym).unwrap();
                        let (bridge_monad, bridge_trace) = self
                            .resolve_and_inject_bridge(
                                caller_mut,
                                &req_sym,
                                &expected_contract,
                                &selected,
                                &reference_polar,
                                recipe,
                            )
                            .await?;

                        let bridge_sym = bridge_monad.symbol_name.clone();
                        let bridge_idx = *graph_nodes
                            .entry(bridge_sym.clone())
                            .or_insert_with(|| dep_graph.add_node(bridge_sym.clone()));

                        // Inyectamos aristas: Caller -> Bridge -> Provider
                        dep_graph.update_edge(caller_idx, bridge_idx, ());
                        dep_graph.update_edge(bridge_idx, callee_idx, ());

                        // Añadimos trace para el puente
                        let bridge_polar = PolarCoordinate::from(&bridge_monad.coord);
                        let bridge_ang_dist = reference_polar.angular_distance(&bridge_polar);
                        traces_by_symbol.insert(bridge_sym.clone(), WovenNodeTrace {
                            symbol_name: bridge_sym.clone(),
                            implementation_hash: bridge_monad.implementation_hash.clone(),
                            contract_hash: bridge_monad.contract_hash.clone(),
                            selected_ring: bridge_monad.coord.ring,
                            selected_angle_rad: bridge_monad.coord.angle_rad,
                            angular_distance_rad: bridge_ang_dist,
                            alternatives_considered: 1,
                        });

                        resolved_symbols.insert(bridge_sym, bridge_monad);
                        bridge_traces.push(bridge_trace);
                        continue;
                    }
                }

                dep_graph.update_edge(caller_idx, callee_idx, ());
            }
        }

        // 3. Ordenación Topológica Determinista (Tarjan SCC soporta recursión mutua)
        // tarjan_scc devuelve las componentes en orden topológico inverso (las hojas/dependencias primero)
        let sccs = tarjan_scc(&dep_graph);
        let mut ordered_symbols = Vec::new();
        for mut component in sccs {
            // Orden alfabético interno si hay un ciclo de recursión mutua para total determinismo
            component.sort_by(|a, b| dep_graph[*a].cmp(&dep_graph[*b]));
            for node_idx in component {
                ordered_symbols.push(dep_graph[node_idx].clone());
            }
        }

        // 4. Ensamblaje del artefacto final con metadatos de trazabilidad espacial
        let mut assembled = String::new();
        if !recipe.prelude.trim().is_empty() {
            assembled.push_str(recipe.prelude.trim());
            assembled.push_str("\n\n");
        }

        let mut ordered_traces = Vec::new();
        for sym in ordered_symbols {
            let monad = &resolved_symbols[&sym];
            let trace = traces_by_symbol.remove(&sym).unwrap();

            if let Some(b_trace) = bridge_traces.iter().find(|b| b.bridge_alias == sym) {
                assembled.push_str(&format!(
                    "// ─── @mandala-bridge: {} (adapts {}.. -> {}.. | lossy: {}) ───\n",
                    b_trace.bridge_alias,
                    &b_trace.from_expected_contract[..4.min(b_trace.from_expected_contract.len())],
                    &b_trace.to_actual_contract[..4.min(b_trace.to_actual_contract.len())],
                    b_trace.is_lossy
                ));
            } else {
                assembled.push_str(&format!(
                    "// ─── @mandala-monad: {} | ring: {} | angle: {:.4} rad (Δθ: {:.4} rad) | impl: {}.. ───\n",
                    monad.symbol_name,
                    monad.coord.ring,
                    monad.coord.angle_rad,
                    trace.angular_distance_rad,
                    &monad.implementation_hash[..12.min(monad.implementation_hash.len())]
                ));
            }
            assembled.push_str(monad.raw_source.trim());
            assembled.push_str("\n\n");

            ordered_traces.push(trace);
        }

        Ok(DistilledArtifact {
            recipe_name: recipe.name.clone(),
            assembled_source: assembled,
            traces: ordered_traces,
            bridge_traces,
        })
    }

    /// Busca e inyecta la Mónada Puente óptima cuando dos contratos colisionan
    pub async fn resolve_and_inject_bridge(
        &self,
        caller_monad: &mut PersistedMonad,
        target_symbol: &str,
        expected_contract_hash: &str,
        actual_provider: &PersistedMonad,
        reference_polar: &PolarCoordinate,
        recipe: &DistillationRecipe,
    ) -> Result<(PersistedMonad, WovenBridgeTrace), WeaveError> {
        let candidates = self
            .store
            .find_bridge_candidates(
                expected_contract_hash,
                &actual_provider.contract_hash,
                recipe.max_ring,
            )
            .await?;

        if candidates.is_empty() {
            return Err(WeaveError::ContractCollision {
                symbol: target_symbol.to_string(),
                contract_a: expected_contract_hash.to_string(),
                contract_b: actual_provider.contract_hash.clone(),
            });
        }

        // Seleccionamos la Mónada Puente más alineada angularmente con el vector de destilación
        let best_candidate = candidates
            .into_iter()
            .min_by(|a, b| {
                let polar_a = PolarCoordinate::from(&a.bridge_monad.coord);
                let polar_b = PolarCoordinate::from(&b.bridge_monad.coord);

                let score_a = reference_polar.angular_distance(&polar_a)
                    + (a.bridge_monad.coord.ring as f64 * recipe.ring_penalty_rad)
                    + a.penalty_rad;
                let score_b = reference_polar.angular_distance(&polar_b)
                    + (b.bridge_monad.coord.ring as f64 * recipe.ring_penalty_rad)
                    + b.penalty_rad;

                score_a
                    .partial_cmp(&score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();

        let mut bridge_monad = best_candidate.bridge_monad;
        let short_hash = &bridge_monad.implementation_hash[..8.min(bridge_monad.implementation_hash.len())];
        let bridge_alias = format!("{}__bridge_{}", target_symbol, short_hash);

        // 1. Renombramos la declaración de la Mónada Puente para evitar colisión de símbolos
        bridge_monad.raw_source =
            crate::weaver::rewriter::AstSynthesizer::rename_function_declaration(&bridge_monad.raw_source, &bridge_alias);
        bridge_monad.symbol_name = bridge_alias.clone();

        // 2. Redirigimos las llamadas dentro del Caller hacia el alias de la Mónada Puente
        caller_monad.raw_source = crate::weaver::rewriter::AstSynthesizer::redirect_call_sites(
            &caller_monad.raw_source,
            target_symbol,
            &bridge_alias,
        );

        let bridge_trace = WovenBridgeTrace {
            caller_symbol: caller_monad.symbol_name.clone(),
            target_symbol: target_symbol.to_string(),
            bridge_alias,
            bridge_impl_hash: bridge_monad.implementation_hash.clone(),
            from_expected_contract: expected_contract_hash.to_string(),
            to_actual_contract: actual_provider.contract_hash.clone(),
            is_lossy: best_candidate.is_lossy,
        };

        Ok((bridge_monad, bridge_trace))
    }

    /// Selecciona la Mónada óptima para un símbolo aplicando `pins` o cercanía angular con `UnitComplex`
    pub async fn resolve_single_symbol(
        &self,
        symbol: &str,
        caller: &str,
        reference_polar: &PolarCoordinate,
        recipe: &DistillationRecipe,
        preloaded_candidates: Option<Vec<PersistedMonad>>,
    ) -> Result<(PersistedMonad, WovenNodeTrace), WeaveError> {
        // Caso 1: Existe un pin explícito en el YAML
        if let Some(pinned_hash) = recipe.pins.get(symbol) {
            let monad = self
                .store
                .get_monad_by_hash(pinned_hash)
                .await?
                .ok_or_else(|| WeaveError::PinnedMonadNotFound {
                    symbol: symbol.to_string(),
                    hash: pinned_hash.clone(),
                })?;

            let candidate_polar = PolarCoordinate::from(&monad.coord);
            let angular_dist = reference_polar.angular_distance(&candidate_polar);

            let trace = WovenNodeTrace {
                symbol_name: monad.symbol_name.clone(),
                implementation_hash: monad.implementation_hash.clone(),
                contract_hash: monad.contract_hash.clone(),
                selected_ring: monad.coord.ring,
                selected_angle_rad: monad.coord.angle_rad,
                angular_distance_rad: angular_dist,
                alternatives_considered: 1,
            };
            return Ok((monad, trace));
        }

        // Caso 2: Selección espacial entre las candidatas devueltas por SurrealDB
        let candidates = match preloaded_candidates {
            Some(list) => list,
            None => {
                self.store
                    .get_candidates_for_symbol(symbol, recipe.max_ring)
                    .await?
            }
        };

        if candidates.is_empty() {
            return Err(WeaveError::UnresolvedPort {
                symbol: symbol.to_string(),
                caller: caller.to_string(),
                max_ring: recipe.max_ring,
            });
        }

        let total_alternatives = candidates.len();

        // Evaluamos cada candidata mediante la métrica geodésica angular + penalización de anillo
        let (best_monad, best_ang_dist, _best_score) = candidates
            .into_iter()
            .map(|cand| {
                let cand_polar = PolarCoordinate::from(&cand.coord);
                let ang_dist = reference_polar.angular_distance(&cand_polar);
                let ring_penalty = (cand.coord.ring as f64) * recipe.ring_penalty_rad;
                let total_score = ang_dist + ring_penalty;
                (cand, ang_dist, total_score)
            })
            .min_by(|(a_monad, _, a_score), (b_monad, _, b_score)| {
                a_score
                    .partial_cmp(b_score)
                    // Desempate 1: Anillo más cercano al núcleo (Ring 0 prefiere sobre Ring 1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a_monad.coord.ring.cmp(&b_monad.coord.ring))
                    // Desempate 2: Hash lexicográfico para determinismo absoluto
                    .then_with(|| {
                        a_monad
                            .implementation_hash
                            .cmp(&b_monad.implementation_hash)
                    })
            })
            .unwrap();

        let trace = WovenNodeTrace {
            symbol_name: best_monad.symbol_name.clone(),
            implementation_hash: best_monad.implementation_hash.clone(),
            contract_hash: best_monad.contract_hash.clone(),
            selected_ring: best_monad.coord.ring,
            selected_angle_rad: best_monad.coord.angle_rad,
            angular_distance_rad: best_ang_dist,
            alternatives_considered: total_alternatives,
        };

        Ok((best_monad, trace))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::PolarCoordinate;
    use crate::ontology::extractor::MonadExtractor;
    use crate::persistence::MandalaStore;

    #[tokio::test]
    async fn test_angular_distillation_switches_implementation() {
        let store = MandalaStore::open_in_memory().await.unwrap();

        // 1. Ingestamos la Mónada principal en el Anillo 0 (ángulo 0.0 rad)
        let root_src = r#"
            pub fn calcular_tension(masa: f64, radio: f64) -> f64 {
                let factor = normalizar(radio);
                masa * factor
            }
        "#;
        let m_root = &MonadExtractor::extract_functions_from_source(root_src).unwrap()[0];
        store
            .upsert_monad(m_root, &PolarCoordinate::new(0, 0.0, 0.0))
            .await
            .unwrap();

        // 2. Ingestamos Variante A de `normalizar` en Anillo 1, Sector 0.2 rad (Alta precisión)
        let norm_precision_src = r#"
            pub fn normalizar(r: f64) -> f64 {
                (r * 1000.0).round() / 1000.0
            }
        "#;
        let m_norm_a = &MonadExtractor::extract_functions_from_source(norm_precision_src).unwrap()[0];
        store
            .upsert_monad(m_norm_a, &PolarCoordinate::new(1, 0.0, 0.2))
            .await
            .unwrap();

        // 3. Ingestamos Variante B de `normalizar` en Anillo 1, Sector 3.0 rad (Rápida / Lineal)
        let norm_fast_src = r#"
            pub fn normalizar(r: f64) -> f64 {
                r * 0.5
            }
        "#;
        let m_norm_b = &MonadExtractor::extract_functions_from_source(norm_fast_src).unwrap()[0];
        store
            .upsert_monad(m_norm_b, &PolarCoordinate::new(1, 0.0, 3.0))
            .await
            .unwrap();

        let weaver = Weaver::new(&store);

        // 4. Receta YAML apuntando a 0.15 rad -> Debe elegir la Variante A (0.2 rad)
        let yaml_precision = r#"
name: "build_precision"
target_angle_rad: 0.15
max_ring: 1
angular_policy: "GlobalTarget"
seeds:
  - symbol: "calcular_tension"
"#;
        let artifact_a = weaver.distill_from_yaml(yaml_precision).await.unwrap();
        assert!(artifact_a.assembled_source.contains("(r * 1000.0).round()"));

        // 5. Receta YAML apuntando a 2.95 rad -> Debe elegir la Variante B (3.0 rad)
        let yaml_fast = r#"
name: "build_fast"
target_angle_rad: 2.95
max_ring: 1
angular_policy: "GlobalTarget"
seeds:
  - symbol: "calcular_tension"
"#;
        let artifact_b = weaver.distill_from_yaml(yaml_fast).await.unwrap();
        assert!(artifact_b.assembled_source.contains("r * 0.5"));

        // 6. Probar que un 'pin' explícito anula la cercanía angular
        let yaml_pinned = format!(
            r#"
name: "build_pinned"
target_angle_rad: 0.15 # Apunta a variante A
max_ring: 1
angular_policy: "GlobalTarget"
seeds:
  - symbol: "calcular_tension"
pins:
  normalizar: "{}" # Pero fijamos forzosamente variante B
"#,
            m_norm_b.implementation_hash
        );
        let artifact_pinned = weaver.distill_from_yaml(&yaml_pinned).await.unwrap();
        assert!(artifact_pinned.assembled_source.contains("r * 0.5"));
    }

    #[tokio::test]
    async fn test_unresolved_port_fails_cleanly() {
        let store = MandalaStore::open_in_memory().await.unwrap();

        let root_src = r#"
            pub fn punto_entrada() -> f64 {
                funcion_inexistente()
            }
        "#;
        let m_root = &MonadExtractor::extract_functions_from_source(root_src).unwrap()[0];
        store
            .upsert_monad(m_root, &PolarCoordinate::new(0, 0.0, 0.0))
            .await
            .unwrap();

        let weaver = Weaver::new(&store);
        let yaml = r#"
name: "build_broken"
target_angle_rad: 0.0
max_ring: 1
angular_policy: "GlobalTarget"
seeds:
  - symbol: "punto_entrada"
"#;
        let res = weaver.distill_from_yaml(yaml).await;
        assert!(matches!(res, Err(WeaveError::UnresolvedPort { .. })));
    }

    #[tokio::test]
    async fn test_automatic_bridge_injection_on_contract_collision() {
        let store = MandalaStore::open_in_memory().await.unwrap();

        // 1. Contrato antiguo (V1) en Anillo 0, Sector π (lejos de nuestro ángulo objetivo)
        let norm_v1_src = r#"
            pub fn normalizar(r: f64) -> f64 { r * 0.5 }
        "#;
        let m_norm_v1 = &MonadExtractor::extract_functions_from_source(norm_v1_src).unwrap()[0];
        store
            .upsert_monad(m_norm_v1, &PolarCoordinate::new(0, 0.0, std::f64::consts::PI))
            .await
            .unwrap();

        // 2. Contrato nuevo (V2) en Anillo 1, Sector 0.1 rad (cambiando el retorno a Result)
        let norm_v2_src = r#"
            pub fn normalizar(r: f64) -> Result<f64, &'static str> {
                if r >= 0.0 { Ok(r * 0.95) } else { Err("Radio negativo") }
            }
        "#;
        let m_norm_v2 = &MonadExtractor::extract_functions_from_source(norm_v2_src).unwrap()[0];
        store
            .upsert_monad(m_norm_v2, &PolarCoordinate::new(1, 0.0, 0.1))
            .await
            .unwrap();

        // 3. Mónada raíz `calcular_tension` en Anillo 0, atada al contrato V1 (espera f64, no Result)
        let root_src = r#"
            pub fn calcular_tension(masa: f64, radio: f64) -> f64 {
                let factor = normalizar(radio);
                masa * factor
            }
        "#;
        let m_root = &MonadExtractor::extract_functions_from_source(root_src).unwrap()[0];
        store
            .upsert_monad(m_root, &PolarCoordinate::new(0, 0.0, 0.0))
            .await
            .unwrap();
        store
            .bind_expected_contract(
                &m_root.identity.implementation_hash,
                "normalizar",
                &m_norm_v1.identity.contract_hash,
            )
            .await
            .unwrap();

        // 4. Registramos la Mónada Puente que expone el contrato V1 e invoca a V2
        let bridge_src = r#"
            pub fn normalizar_adapter(r: f64) -> f64 {
                normalizar(r).unwrap_or(1.0)
            }
        "#;
        let m_bridge = &MonadExtractor::extract_functions_from_source(bridge_src).unwrap()[0];
        store
            .register_bridge_monad(
                m_bridge,
                &PolarCoordinate::new(1, 0.0, 0.12),
                &m_norm_v1.identity.contract_hash, // IN: Contrato esperado (f64 -> f64)
                &m_norm_v2.identity.contract_hash, // OUT: Contrato real (f64 -> Result)
                "normalizar",
                true, // is_lossy: descarta el Err con unwrap_or
                0.05,
            )
            .await
            .unwrap();

        // 5. Ejecutamos la destilación apuntando al sector 0.1 rad (donde vive normalizar V2)
        let mut caller_persisted = store
            .get_monad_by_hash(&m_root.identity.implementation_hash)
            .await
            .unwrap()
            .unwrap();
        let provider_persisted = store
            .get_monad_by_hash(&m_norm_v2.identity.implementation_hash)
            .await
            .unwrap()
            .unwrap();

        let recipe = DistillationRecipe {
            name: "build_auto_bridged".into(),
            target_angle_rad: 0.1,
            max_ring: 1,
            angular_policy: AngularPolicy::GlobalTarget,
            ring_penalty_rad: 0.01,
            prelude: String::new(),
            seeds: vec![SeedSpec { symbol: "calcular_tension".into() }],
            pins: Default::default(),
            external_symbols: Default::default(),
        };

        let weaver = Weaver::new(&store);
        let (injected_bridge, trace) = weaver
            .resolve_and_inject_bridge(
                &mut caller_persisted,
                "normalizar",
                &m_norm_v1.identity.contract_hash,
                &provider_persisted,
                &PolarCoordinate::new(0, 0.0, 0.1),
                &recipe,
            )
            .await
            .unwrap();

        // Verificamos que el Caller ahora invoca al alias del puente y el puente invoca a `normalizar` V2
        assert!(caller_persisted.raw_source.contains(&trace.bridge_alias));
        assert!(injected_bridge.raw_source.contains("normalizar(r).unwrap_or(1.0)"));

        // 6. Prueba End-to-End con distill(&recipe) completa
        let artifact = weaver.distill(&recipe).await.unwrap();
        assert_eq!(artifact.bridge_traces.len(), 1);
        assert!(artifact.assembled_source.contains("@mandala-bridge"));
        assert!(artifact.assembled_source.contains("Result<f64, &'static str>"));
        assert!(artifact.assembled_source.contains("normalizar(r).unwrap_or(1.0)"));
    }
}

