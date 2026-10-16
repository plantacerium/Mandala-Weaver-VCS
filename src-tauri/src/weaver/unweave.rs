use crate::geometry::polar_space::PolarCoord;
use crate::language::detect_language;
use crate::ontology::bindu::Bindu;
use crate::ontology::monad::{Monad, MonadKind};
use crate::persistence::surreal_bridge::{connect_db, insert_and_link, Db};
use crate::template::{OutputConfig, OutputStructure, RingSelector, VectorSelector};
use crate::weaver::ast_extractor::extract_raw_monads_lang;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use surrealdb::Surreal;

/// Configuración para el desensamblado / deconstrucción ontológica inversa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnweaveConfig {
    pub project_path: PathBuf,
    pub output_recipe: Option<PathBuf>,
}

/// Métrica de centralidad calculada a partir del grafo de llamadas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralityMetric {
    pub symbol_name: String,
    pub in_degree: usize,
    pub out_degree: usize,
    pub ring: u32,
    pub domain: String,
    pub canonical_signature: String,
}

/// Reporte completo del proceso de desensamblado ontológico (unweave / shatter)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnweaveReport {
    pub project_name: String,
    pub monads_extracted: usize,
    pub max_ring: u32,
    pub domains: Vec<String>,
    pub recipe_path: PathBuf,
    pub top_contracts: Vec<CentralityMetric>,
}

pub struct UnweaveEngine;

impl UnweaveEngine {
    /// Ejecuta el proceso de desensamblado (unweave / shatter) abriendo una conexión de base de datos:
    pub async fn unweave(
        project_path: &Path,
        output_recipe: Option<&Path>,
    ) -> anyhow::Result<UnweaveReport> {
        let canonical_path = project_path.canonicalize().unwrap_or_else(|_| project_path.to_path_buf());
        let db = connect_db(Some(&canonical_path)).await?;
        Self::unweave_with_db(&db, &canonical_path, output_recipe).await
    }

    /// Ejecuta el proceso de desensamblado (unweave / shatter) reutilizando una conexión activa de SurrealDB:
    /// 1. Extrae todas las mónadas usando el Hasher Merkle-AST Canónico.
    /// 2. Construye el grafo de llamadas y calcula la centralidad de contratos.
    /// 3. Asigna anillos gravitacionales (Núcleo/Contrato -> Anillo 1, Hojas/UI -> Anillos exteriores).
    /// 4. Agrupa angularmente por dominio/módulo (θ).
    /// 5. Persiste en SurrealDB y genera el archivo `distill.yaml` unificado (Spatial Weaver + Template).
    pub async fn unweave_with_db(
        db: &Surreal<Db>,
        project_path: &Path,
        output_recipe: Option<&Path>,
    ) -> anyhow::Result<UnweaveReport> {
        let canonical_path = project_path.canonicalize().unwrap_or_else(|_| project_path.to_path_buf());
        let project_name = canonical_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("mandala-project")
            .to_string();

        // 1. Recolectar archivos fuente políglotas
        let mut source_files = Vec::new();
        let extensions = crate::language::Language::code_extensions();
        Self::collect_files_recursive(&canonical_path, extensions, &mut source_files, 0)?;
        source_files.sort_by_key(|(depth, p)| (*depth, p.clone()));

        // 2. Extraer mónadas con identidad canónica y puertos de salida
        let mut extracted_monads: Vec<(Monad, String)> = Vec::new(); // (Monad, domain)

        for (_depth, file_path) in &source_files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                if content.trim().is_empty() {
                    continue;
                }

                let lang = detect_language(file_path);
                let ext_str = file_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or(lang.tree_sitter_name());
                let relative = file_path
                    .strip_prefix(&canonical_path)
                    .unwrap_or(file_path);
                let domain = Self::extract_domain_from_path(relative);

                let monads = extract_raw_monads_lang(
                    &content,
                    1, // Se recalculará tras el análisis de centralidad
                    ext_str,
                );

                if monads.is_empty() {
                    // Fallback para archivos sin AST profundo (ej. configuraciones o módulos planos)
                    let filename = file_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("module");
                    let hash = crate::ontology::semantic_hash::generate_pure_hash(&content);
                    let m = Monad::spawn_typed(
                        hash,
                        filename.to_string(),
                        PolarCoord::new(100.0, 0.0),
                        content.clone(),
                        1,
                        MonadKind::Module,
                        1,
                        content.lines().count() as u32,
                        lang.tree_sitter_name(),
                    );
                    extracted_monads.push((m, domain));
                } else {
                    for m in monads {
                        extracted_monads.push((m, domain.clone()));
                    }
                }
            }
        }

        if extracted_monads.is_empty() {
            anyhow::bail!("No se encontraron mónadas extraíbles en: {}", canonical_path.display());
        }

        // 3. Construir grafo de dependencias y calcular centralidad
        let monad_count = extracted_monads.len();
        let mut name_to_indices: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, (m, _)) in extracted_monads.iter().enumerate() {
            name_to_indices.entry(m.name.clone()).or_default().push(idx);
        }

        let mut in_degrees = vec![0usize; monad_count];
        let mut out_dependencies: Vec<HashSet<usize>> = vec![HashSet::new(); monad_count];

        for (i, (m, _)) in extracted_monads.iter().enumerate() {
            for call in &m.outbound_calls {
                if let Some(target_indices) = name_to_indices.get(call) {
                    for &target_idx in target_indices {
                        if target_idx != i {
                            if out_dependencies[i].insert(target_idx) {
                                in_degrees[target_idx] += 1;
                            }
                        }
                    }
                }
            }
        }

        // 4. Asignación de Anillos Gravitacionales basada en Centralidad Topológica
        // Regla ontológica de Mandala Weaver:
        // - Anillo 1 (Núcleo / Bindu): Contratos invariantes, tipos/estructuras/traits y funciones sin dependencias internas salientes pero con dependientes entrantes (in_degree > 0).
        // - Anillos intermedios (2-3): Capas de lógica y servicios intermedios.
        // - Anillos exteriores (4+): Puntos de entrada, invocadores hoja, controladores y UI (out_degree alto, in_degree = 0).
        let mut assigned_rings = vec![1u32; monad_count];
        let mut memo_depth: HashMap<usize, u32> = HashMap::new();
        let mut visiting: HashSet<usize> = HashSet::new();

        fn compute_gravitational_ring(
            idx: usize,
            out_deps: &[HashSet<usize>],
            in_deg: &[usize],
            kinds: &[MonadKind],
            memo: &mut HashMap<usize, u32>,
            visiting: &mut HashSet<usize>,
        ) -> u32 {
            if let Some(&ring) = memo.get(&idx) {
                return ring;
            }

            if visiting.contains(&idx) {
                return 2; // Romper ciclo asignando anillo base de lógica
            }

            visiting.insert(idx);

            let deps = &out_deps[idx];
            let kind = &kinds[idx];

            let ring = if deps.is_empty() {
                // Nodo sin dependencias internas
                match kind {
                    MonadKind::Struct | MonadKind::Enum | MonadKind::Trait | MonadKind::Constant | MonadKind::TypeAlias => 1,
                    _ => {
                        if in_deg[idx] > 0 {
                            1 // Función fundacional invocada por el sistema
                        } else {
                            2 // Módulo o función aislada
                        }
                    }
                }
            } else {
                let mut max_dep_ring = 1;
                for &dep in deps {
                    let dep_ring = compute_gravitational_ring(dep, out_deps, in_deg, kinds, memo, visiting);
                    max_dep_ring = max_dep_ring.max(dep_ring);
                }
                (max_dep_ring + 1).min(6)
            };

            visiting.remove(&idx);
            memo.insert(idx, ring);
            ring
        }

        let monad_kinds: Vec<MonadKind> = extracted_monads.iter().map(|(m, _)| m.kind.clone()).collect();
        for i in 0..monad_count {
            assigned_rings[i] = compute_gravitational_ring(
                i,
                &out_dependencies,
                &in_degrees,
                &monad_kinds,
                &mut memo_depth,
                &mut visiting,
            );
        }

        // 5. Agrupación por Dominios y Coordenadas Polares (θ)
        let mut domain_set: BTreeSet<String> = BTreeSet::new();
        for (_, (_, domain)) in extracted_monads.iter().enumerate() {
            domain_set.insert(domain.clone());
        }

        let domains: Vec<String> = domain_set.into_iter().collect();
        let domain_count = domains.len().max(1);
        let sector_size = 360.0 / (domain_count as f64);

        let mut domain_to_index: HashMap<String, usize> = HashMap::new();
        for (i, d) in domains.iter().enumerate() {
            domain_to_index.insert(d.clone(), i);
        }

        // Contadores por (dominio, anillo) para distribución angular limpia sin solapamientos
        let mut domain_ring_counters: HashMap<(String, u32), usize> = HashMap::new();

        let mut final_monads: Vec<Monad> = Vec::with_capacity(monad_count);
        let mut centrality_metrics: Vec<CentralityMetric> = Vec::with_capacity(monad_count);

        for (i, (mut m, domain)) in extracted_monads.into_iter().enumerate() {
            let ring = assigned_rings[i];
            m.ring = ring;

            let d_idx = domain_to_index.get(&domain).copied().unwrap_or(0);
            let sector_start = (d_idx as f64) * sector_size;
            let counter = domain_ring_counters.entry((domain.clone(), ring)).or_insert(0);

            // Phyllotaxis / Golden Angle dentro del sector del dominio
            let local_angle = ((*counter as f64) * 137.50776405) % (sector_size * 0.9);
            let theta = (sector_start + (sector_size * 0.05) + local_angle) % 360.0;
            *counter += 1;

            let radius = (ring as f64) * 100.0;
            m.coord = PolarCoord::new(radius, theta);

            centrality_metrics.push(CentralityMetric {
                symbol_name: m.name.clone(),
                in_degree: in_degrees[i],
                out_degree: out_dependencies[i].len(),
                ring,
                domain: domain.clone(),
                canonical_signature: m.canonical_signature.clone(),
            });

            final_monads.push(m);
        }

        // 6. Persistir en SurrealDB
        Self::persist_unweave_state(db, &project_name, &final_monads, &out_dependencies).await?;

        let max_ring = final_monads.iter().map(|m| m.ring).max().unwrap_or(1);

        // 7. Generar receta unificada `distill.yaml` (compatible con Spatial Weaver SO(2) y TemplateEngine)
        let recipe_path = match output_recipe {
            Some(p) => p.to_path_buf(),
            None => canonical_path.join("distill.yaml"),
        };

        Self::generate_distill_recipe(
            &recipe_path,
            &project_name,
            &domains,
            sector_size,
            max_ring,
            &final_monads,
            &centrality_metrics,
        )?;

        // 8. Exportar estado a JSON para la UI de Mandala Weaver
        Self::export_mandala_json(&canonical_path, &project_name, &final_monads).ok();

        // Ordenar métricas de contratos más centrales (mayor in-degree)
        centrality_metrics.sort_by(|a, b| b.in_degree.cmp(&a.in_degree));
        let top_contracts = centrality_metrics.into_iter().take(15).collect();

        Ok(UnweaveReport {
            project_name,
            monads_extracted: final_monads.len(),
            max_ring,
            domains,
            recipe_path,
            top_contracts,
        })
    }

    /// Extrae el dominio/módulo semántico a partir de la ruta relativa del archivo
    fn extract_domain_from_path(relative_path: &Path) -> String {
        let mut components = relative_path.components();
        if let Some(first) = components.next() {
            let first_str = first.as_os_str().to_string_lossy();
            if first_str == "src" || first_str == "lib" || first_str == "app" {
                if let Some(second) = components.next() {
                    return second.as_os_str().to_string_lossy().to_string();
                }
            }
            if !first_str.is_empty() && first_str != "." {
                return first_str.to_string();
            }
        }
        "core".to_string()
    }

    /// Recolecta archivos recursivamente ignorando directorios de artefactos y dependencias
    fn collect_files_recursive(
        current: &Path,
        extensions: &[&str],
        collected: &mut Vec<(usize, PathBuf)>,
        depth: usize,
    ) -> anyhow::Result<()> {
        if !current.exists() || depth > 8 {
            return Ok(());
        }

        for entry in std::fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if extensions.contains(&ext.to_str().unwrap_or("")) {
                        collected.push((depth, path));
                    }
                }
            } else if path.is_dir() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                if !name.starts_with('.')
                    && name != "target"
                    && name != "node_modules"
                    && name != "__pycache__"
                    && name != "dist"
                    && name != "build"
                    && name != ".git"
                    && name != ".mandala"
                {
                    Self::collect_files_recursive(&path, extensions, collected, depth + 1)?;
                }
            }
        }

        Ok(())
    }

    /// Persiste el estado desensamblado en SurrealDB
    async fn persist_unweave_state(
        db: &Surreal<Db>,
        project_name: &str,
        monads: &[Monad],
        out_deps: &[HashSet<usize>],
    ) -> anyhow::Result<()> {
        // Inicializar o actualizar Bindu
        let bindu = Bindu::genesis(project_name);
        let val = serde_json::to_value(&bindu)?;
        let _: Option<serde_json::Value> = db.upsert(("bindu", "genesis")).content(val).await.ok().flatten();

        // Insertar cada mónada (insert_and_link ya puebla tanto `monad` como el Grafo Tripartito en MandalaStore)
        for m in monads {
            let _ = insert_and_link(db, m, None).await;
        }

        // Crear relaciones de grafo de llamadas/evolución en SurrealDB
        for (src_idx, deps) in out_deps.iter().enumerate() {
            let src_id = &monads[src_idx].id;
            for &target_idx in deps {
                let target_id = &monads[target_idx].id;
                let _ = db
                    .query("RELATE monad:$source -> evolves_to -> monad:$target")
                    .bind(("source", src_id.clone()))
                    .bind(("target", target_id.clone()))
                    .await;
            }
        }

        Ok(())
    }

    /// Genera la receta canónica unificada `distill.yaml` que funciona tanto con el
    /// Weaver Espacial SO(2) (`DistillationRecipe` por semillas y contratos) como con
    /// el motor de sectores (`DistillationTemplate`).
    fn generate_distill_recipe(
        recipe_path: &Path,
        project_name: &str,
        domains: &[String],
        sector_size: f64,
        max_ring: u32,
        monads: &[Monad],
        metrics: &[CentralityMetric],
    ) -> anyhow::Result<()> {
        let mut vectors = Vec::new();
        for (i, domain) in domains.iter().enumerate() {
            let start = (i as f64) * sector_size;
            let end = ((i + 1) as f64) * sector_size;
            vectors.push(VectorSelector {
                name: domain.clone(),
                angle_start: Some(start),
                angle_end: Some(end),
                ring: None,
            });
        }

        // Inferir semillas raíz óptimas para el Spatial Weaver:
        // 1. Si existe `main`, incluirla primero.
        // 2. Añadir puntos de entrada con out_degree > 0 e in_degree == 0 (orquestadores).
        // 3. Si no hay ninguno, usar las primeras funciones extraídas.
        let mut seed_symbols: Vec<String> = Vec::new();
        if monads.iter().any(|m| m.name == "main") {
            seed_symbols.push("main".to_string());
        }
        for metric in metrics {
            if metric.out_degree > 0 && metric.in_degree == 0 && !seed_symbols.contains(&metric.symbol_name) {
                seed_symbols.push(metric.symbol_name.clone());
                if seed_symbols.len() >= 5 {
                    break;
                }
            }
        }
        if seed_symbols.is_empty() {
            for m in monads.iter().filter(|m| m.kind == MonadKind::Function).take(3) {
                if !seed_symbols.contains(&m.name) {
                    seed_symbols.push(m.name.clone());
                }
            }
        }
        if seed_symbols.is_empty() {
            if let Some(first) = monads.first() {
                seed_symbols.push(first.name.clone());
            }
        }

        #[derive(Serialize)]
        struct UnifiedManifest {
            name: String,
            version: String,
            target_angle_rad: f64,
            max_ring: u32,
            angular_policy: crate::weaver::spatial_weaver::AngularPolicy,
            ring_penalty_rad: f64,
            prelude: String,
            seeds: Vec<crate::weaver::spatial_weaver::SeedSpec>,
            rings: RingSelector,
            vectors: Vec<VectorSelector>,
            exclude: Vec<crate::template::ExcludeRule>,
            adapters: Vec<crate::template::AdapterConfig>,
            output: OutputConfig,
            variables: HashMap<String, crate::template::VariableDef>,
        }

        let unified = UnifiedManifest {
            name: format!("{}-distill", project_name),
            version: "1.0.0".to_string(),
            target_angle_rad: 0.0,
            max_ring,
            angular_policy: crate::weaver::spatial_weaver::AngularPolicy::GlobalTarget,
            ring_penalty_rad: 0.05,
            prelude: format!("//! Artefacto destilado espacialmente por Mandala Weaver VCS para {}", project_name),
            seeds: seed_symbols
                .into_iter()
                .map(|symbol| crate::weaver::spatial_weaver::SeedSpec { symbol })
                .collect(),
            rings: RingSelector::All,
            vectors,
            exclude: Vec::new(),
            adapters: Vec::new(),
            output: OutputConfig {
                filename: "src".to_string(),
                structure: OutputStructure::VectorDirs,
                generate_modules: true,
            },
            variables: HashMap::new(),
        };

        let yaml_content = serde_yml::to_string(&unified)?;
        let header = format!(
            "# ====================================================================\n\
             # Mandala Weaver VCS — Receta de Destilación Unificada (SO(2) + Sectores)\n\
             # Generada automáticamente por el comando `weave innerknow` (unweave/shatter)\n\
             # Proyecto: {}\n\
             # Compatible con Spatial Weaver (seeds + contratos) y TemplateEngine (vectors)\n\
             # Reconstruir proyecto: `weave distill --template distill.yaml`\n\
             # ====================================================================\n\n",
            project_name
        );

        std::fs::write(recipe_path, format!("{}{}", header, yaml_content))?;
        Ok(())
    }

    /// Exporta el estado a `mandala_state.json` tanto en `.mandala` como en `public`
    fn export_mandala_json(
        project_path: &Path,
        project_name: &str,
        monads: &[Monad],
    ) -> anyhow::Result<()> {
        let max_ring = monads.iter().map(|m| m.ring).max().unwrap_or(1);
        let mut rings_map: HashMap<u32, Vec<Monad>> = HashMap::new();
        for m in monads {
            rings_map.entry(m.ring).or_default().push(m.clone());
        }

        let mut constellations = Vec::new();
        for (ring_level, ring_monads) in rings_map {
            constellations.push(crate::interface::projection_api::ConstellationDto {
                ring_level,
                monads: ring_monads,
            });
        }
        constellations.sort_by_key(|c| c.ring_level);

        let state = crate::interface::projection_api::MandalaState {
            bindu_name: project_name.to_string(),
            constellations,
            edges: Vec::new(),
            total_monads: monads.len(),
            max_ring,
        };

        let json = serde_json::to_string_pretty(&state)?;

        // Guardar en .mandala/mandala_state.json
        let mandala_dir = project_path.join(".mandala");
        std::fs::create_dir_all(&mandala_dir).ok();
        let _ = std::fs::write(mandala_dir.join("mandala_state.json"), &json);

        // Guardar en public/mandala_state.json si existe para visualización directa en Vite/Tauri
        let public_path = project_path.join("public").join("mandala_state.json");
        if public_path.parent().map(|p| p.exists()).unwrap_or(false) {
            let _ = std::fs::write(public_path, &json);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_domain_extraction_from_path() {
        let p1 = Path::new("src/ontology/monad.rs");
        assert_eq!(UnweaveEngine::extract_domain_from_path(p1), "ontology");

        let p2 = Path::new("src/weaver/unweave.rs");
        assert_eq!(UnweaveEngine::extract_domain_from_path(p2), "weaver");

        let p3 = Path::new("package.json");
        assert_eq!(UnweaveEngine::extract_domain_from_path(p3), "package.json");
    }

    #[tokio::test]
    async fn test_unweave_recipe_generation() {
        let temp_dir = std::env::temp_dir().join(format!("mandala_test_unweave_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let core_dir = temp_dir.join("src").join("core");
        let service_dir = temp_dir.join("src").join("service");
        std::fs::create_dir_all(&core_dir).unwrap();
        std::fs::create_dir_all(&service_dir).unwrap();

        // Core leaf contract (Ring 1)
        let core_code = r#"
            pub fn calcular_base(x: f64) -> f64 {
                x * 2.0
            }
        "#;
        std::fs::write(core_dir.join("math.rs"), core_code).unwrap();

        // Service orchestrator depending on core contract (Ring 2)
        let service_code = r#"
            pub fn ejecutar_flujo(valor: f64) -> f64 {
                let res = calcular_base(valor);
                res + 10.0
            }
        "#;
        std::fs::write(service_dir.join("flow.rs"), service_code).unwrap();

        let recipe_file = temp_dir.join("distill.yaml");
        let report = UnweaveEngine::unweave(&temp_dir, Some(&recipe_file)).await.unwrap();

        assert!(report.monads_extracted >= 2);
        assert!(report.domains.contains(&"core".to_string()));
        assert!(report.domains.contains(&"service".to_string()));
        assert!(recipe_file.exists());

        let recipe_content = std::fs::read_to_string(&recipe_file).unwrap();
        assert!(recipe_content.contains("distill"));
        assert!(recipe_content.contains("core"));
        assert!(recipe_content.contains("service"));

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
