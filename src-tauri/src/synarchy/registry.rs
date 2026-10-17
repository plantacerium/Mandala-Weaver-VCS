use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEntry {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub project_type: ProjectType,
    pub last_scanned: u64,
    pub ring_count: u32,
    pub monad_count: usize,
    pub status: ProjectStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    Local,
    Remote,
    Git,
    Weaver,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectStatus {
    Active,
    Dormant,
    Scanning,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectRegistry {
    pub projects: Vec<ProjectEntry>,
}

impl ProjectRegistry {
    pub fn new() -> Self {
        Self { projects: Vec::new() }
    }
    
    pub fn add(&mut self, entry: ProjectEntry) {
        self.projects.push(entry);
    }
    
    pub fn remove(&mut self, id: &str) -> Option<ProjectEntry> {
        if let Some(pos) = self.projects.iter().position(|p| p.id == id) {
            Some(self.projects.remove(pos))
        } else {
            None
        }
    }
    
    pub fn find(&self, id: &str) -> Option<&ProjectEntry> {
        self.projects.iter().find(|p| p.id == id)
    }

    pub fn find_mut(&mut self, id: &str) -> Option<&mut ProjectEntry> {
        self.projects.iter_mut().find(|p| p.id == id)
    }
    
    
    pub fn active_projects(&self) -> Vec<&ProjectEntry> {
        self.projects.iter()
            .filter(|p| matches!(p.status, ProjectStatus::Active))
            .collect()
    }
    
    pub fn save(&self, path: &PathBuf) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    pub fn load(path: &PathBuf) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let registry = serde_json::from_str(&content)?;
        Ok(registry)
    }
}

pub struct ProjectScanner;

impl ProjectScanner {
    pub async fn scan(path: &PathBuf) -> anyhow::Result<ProjectEntry> {
        Self::scan_with_mode(path, None).await
    }

    pub async fn scan_with_mode(path: &PathBuf, mode: Option<&str>) -> anyhow::Result<ProjectEntry> {
        let db = crate::persistence::surreal_bridge::connect_db(Some(path)).await?;
        Self::scan_and_index_with_db_mode(&db, path, mode).await
    }

    pub async fn scan_and_index(path: &PathBuf) -> anyhow::Result<ProjectEntry> {
        Self::scan_with_mode(path, None).await
    }

    pub async fn scan_and_index_with_db(
        db: &surrealdb::Surreal<crate::persistence::surreal_bridge::Db>,
        path: &PathBuf,
    ) -> anyhow::Result<ProjectEntry> {
        Self::scan_and_index_with_db_mode(db, path, None).await
    }

    pub async fn scan_and_index_with_db_mode(
        db: &surrealdb::Surreal<crate::persistence::surreal_bridge::Db>,
        path: &PathBuf,
        scan_mode: Option<&str>,
    ) -> anyhow::Result<ProjectEntry> {
        let canonical = path.canonicalize().unwrap_or(path.clone());
        let name = canonical.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Mandala-Weaver-VCS")
            .to_string();
        
        let id = blake3::hash(format!("{}_{}", name, path.display()).as_bytes())
            .to_hex()
            .to_string()[..8]
            .to_string();

        let is_git = path.join(".git").exists();
        let mandala_dir = path.join(".mandala");
        let has_mandala = mandala_dir.exists();

        let project_type = match scan_mode {
            Some("git") => {
                if !is_git {
                    anyhow::bail!("No se encontró un repositorio Git (.git) en: {}. Selecciona 'Mandala Scan' para escanearlo de forma nativa sin Git.", path.display());
                }
                ProjectType::Git
            }
            Some("mandala") => ProjectType::Weaver,
            _ => {
                if is_git {
                    ProjectType::Git
                } else if has_mandala {
                    ProjectType::Weaver
                } else {
                    ProjectType::Weaver
                }
            }
        };

        let mut all_monads = crate::persistence::surreal_bridge::get_all_monads(db).await.unwrap_or_default();
        let force_rescan = matches!(scan_mode, Some("rescan") | Some("force") | Some("git") | Some("mandala"))
            || Self::needs_rebalance(&all_monads)
            || all_monads.iter().any(|m| m.name == "mandala_state.json" || m.name.trim().is_empty());

        if all_monads.is_empty() || force_rescan {
            if !all_monads.is_empty() {
                let _ = db.query("DELETE monad; DELETE evolves_to;").await;
            }

            let bindu = crate::ontology::bindu::Bindu::genesis(&name);
            let val = serde_json::to_value(&bindu)?;
            let _: Option<serde_json::Value> = db.upsert(("bindu", "genesis")).content(val).await?;

            let mut files = Vec::new();
            let extensions = crate::language::Language::code_extensions();
            Self::collect_files_recursive(path, extensions, &mut files, 0)?;

            files.sort_by_key(|(depth, p)| (*depth, p.clone()));

            let mut staged_monads: Vec<crate::ontology::monad::Monad> = Vec::new();

            for (depth, file_path) in files.iter().take(250) {
                if let Ok(content) = std::fs::read_to_string(file_path) {
                    let lang = crate::language::detect_language(file_path);
                    let ext_str = file_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or(lang.tree_sitter_name());
                    let file_stem = file_path
                        .file_stem()
                        .and_then(|n| n.to_str())
                        .unwrap_or("module");

                    let monads = crate::weaver::ast_extractor::extract_raw_monads_lang(
                        &content,
                        1,
                        ext_str,
                    );
                    if monads.is_empty() && !content.trim().is_empty() {
                        let filename = file_path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("module");
                        let ring = Self::classify_architectural_ring(file_path, &crate::ontology::monad::MonadKind::Module, *depth);
                        let hash = crate::ontology::semantic_hash::generate_pure_hash(&content);
                        let m = crate::ontology::monad::Monad::spawn_typed(
                            hash,
                            filename.to_string(),
                            crate::geometry::polar_space::PolarCoord::new((ring as f64) * 60.0, 0.0),
                            content.clone(),
                            ring,
                            crate::ontology::monad::MonadKind::Module,
                            1,
                            content.lines().count() as u32,
                            lang.tree_sitter_name(),
                        );
                        staged_monads.push(m);
                    } else {
                        for mut m in monads {
                            let ring = Self::classify_architectural_ring(file_path, &m.kind, *depth);
                            m.ring = ring;

                            if m.name == "tests" || m.name == "test" {
                                m.name = format!("{}::{}", file_stem, m.name);
                            } else if m.name.trim().is_empty() {
                                m.name = format!("{}_{}", file_stem, m.line_start);
                            }
                            staged_monads.push(m);
                        }
                    }
                }
            }

            // Apply deterministic Vogel Sunflower Phyllotaxis + Golden Ratio per ring
            Self::apply_golden_vogel_layout(&mut staged_monads, 320.0);

            // Persist monads and link semantic call & evolution edges across rings
            let mut last_id_by_symbol: std::collections::HashMap<String, String> = std::collections::HashMap::new();
            for m in &staged_monads {
                let parent_id = last_id_by_symbol
                    .get(&m.name)
                    .cloned()
                    .or_else(|| {
                        m.outbound_calls
                            .iter()
                            .find_map(|callee| last_id_by_symbol.get(callee).cloned())
                    });

                let _ = crate::persistence::surreal_bridge::insert_and_link(
                    db,
                    m,
                    parent_id.as_deref(),
                ).await;
                last_id_by_symbol.insert(m.name.clone(), m.id.clone());
            }

            all_monads = crate::persistence::surreal_bridge::get_all_monads(db).await.unwrap_or_default();
        }

        let monad_count = all_monads.len();
        let ring_count = all_monads.iter().map(|m| m.ring).max().unwrap_or(0);

        Ok(ProjectEntry {
            id,
            name,
            path: path.clone(),
            project_type,
            last_scanned: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            ring_count,
            monad_count,
            status: ProjectStatus::Active,
        })
    }

    pub async fn scan_with_rings(path: &PathBuf, ring_count: u32) -> anyhow::Result<ProjectEntry> {
        let mut entry = Self::scan(path).await?;
        entry.ring_count = ring_count;
        Ok(entry)
    }

    /// Detects whether an existing monad collection has a legacy or skewed ring distribution
    /// (e.g., > 5 rings from watcher auto-expansion, < 5 rings, or > 38% of monads piled into a single ring).
    pub fn needs_rebalance(monads: &[crate::ontology::monad::Monad]) -> bool {
        if monads.is_empty() {
            return false;
        }
        let max_ring = monads.iter().map(|m| m.ring).max().unwrap_or(0);
        if max_ring > 5 || max_ring == 0 {
            return true;
        }
        if monads.len() >= 25 {
            let mut counts = std::collections::HashMap::<u32, usize>::new();
            for m in monads {
                *counts.entry(m.ring).or_insert(0) += 1;
            }
            if counts.len() < 5 {
                return true;
            }
            let max_in_ring = counts.values().copied().max().unwrap_or(0);
            let min_in_ring = counts.values().copied().min().unwrap_or(0);
            if max_in_ring * 100 / monads.len() > 38 || min_in_ring * 100 / monads.len() < 4 {
                return true;
            }
        }
        false
    }

    /// Classifies a monad when only in-memory `Monad` metadata is available (self-healing fallback).
    pub fn classify_monad_ring(m: &crate::ontology::monad::Monad) -> u32 {
        use crate::ontology::monad::MonadKind;
        let lang = m.language.to_lowercase();
        let name_lower = m.name.to_lowercase();
        let trimmed = m.content.trim_start();

        // Ring 5: Sensory Periphery, UI Components, D3 Renderer, Styles & Views
        if matches!(lang.as_str(), "css" | "scss" | "sass" | "less" | "astro" | "html" | "tsx" | "jsx" | "vue" | "svelte")
            || m.content.contains("React.FC")
            || m.content.contains("useState")
            || m.content.contains("useEffect")
            || m.content.contains("className=")
            || m.content.contains("d3.select")
            || m.content.contains(".attr(")
            || (trimmed.starts_with('.') && m.content.contains('{') && (m.content.contains("color:") || m.content.contains("display:") || m.content.contains("background") || m.content.contains("padding") || m.content.contains("border")))
            || name_lower.contains("canvas")
            || name_lower.contains("panel")
            || name_lower.contains("sidebar")
            || name_lower.contains("header")
            || name_lower.contains("dialog")
            || name_lower.contains("inspector")
            || name_lower.contains("palette")
            || name_lower.contains("render")
            || name_lower.contains("draw")
            || name_lower.contains("highlight")
            || name_lower.contains("lasso")
            || name_lower.contains("zoom")
        {
            return 5;
        }

        // Ring 1: Core Ontology, Sacred Geometry, Contracts, Traits, Enums & Manifest Configs
        if matches!(lang.as_str(), "toml" | "json" | "yaml" | "yml" | "graphql" | "gql" | "proto" | "protobuf")
            || matches!(m.kind, MonadKind::Trait | MonadKind::Enum | MonadKind::TypeAlias)
            || trimmed.starts_with("pub enum ")
            || trimmed.starts_with("enum ")
            || trimmed.starts_with("pub trait ")
            || trimmed.starts_with("trait ")
            || trimmed.starts_with("export interface ")
            || trimmed.starts_with("interface ")
            || trimmed.starts_with("export type ")
            || name_lower.contains("monad")
            || name_lower.contains("bindu")
            || name_lower.contains("polar")
            || name_lower.contains("coord")
            || name_lower.contains("golden")
            || name_lower.contains("geodesic")
            || name_lower.contains("identity")
            || name_lower.contains("canonical")
            || name_lower.contains("ontology")
            || name_lower.contains("delta")
        {
            return 1;
        }

        // Ring 4: Interfaces, CLI, MCP Server, Tauri IPC, Watcher & Git Collaboration
        if name_lower.starts_with("cmd_")
            || name_lower.starts_with("cli_")
            || name_lower.contains("mcp")
            || name_lower.contains("api")
            || name_lower.contains("invoke")
            || name_lower.contains("fetch")
            || name_lower.contains("watcher")
            || name_lower.contains("merge")
            || name_lower.contains("git")
            || name_lower.contains("export_")
            || name_lower.contains("open_project")
            || name_lower.contains("workspace")
            || m.content.contains("#[tauri::command]")
            || m.content.contains("jsonrpc")
        {
            return 4;
        }

        // Ring 2: Substrate Data Models, Structs, Constants, Persistence & Templates
        if matches!(m.kind, MonadKind::Struct | MonadKind::Constant)
            || trimmed.starts_with("pub struct ")
            || trimmed.starts_with("struct ")
            || name_lower.contains("store")
            || name_lower.contains("surreal")
            || name_lower.contains("db")
            || name_lower.contains("upsert")
            || name_lower.contains("insert")
            || name_lower.contains("query")
            || name_lower.contains("template")
            || name_lower.contains("recipe")
            || name_lower.contains("contract")
            || name_lower.contains("port")
            || name_lower.contains("bridge")
            || name_lower.contains("adapter")
            || name_lower.contains("plugin")
            || name_lower.contains("registry")
            || name_lower.contains("project")
            || name_lower.contains("detect_")
        {
            return 2;
        }

        // Ring 3: Domain Weavers, AST Extractors, Compilers & Sync Engines
        3
    }

    /// Computes exact Golden Ratio annular bounds and Vogel Sunflower Phyllotaxis coordinates
    /// for all monads across rings 1..=5.
    pub fn apply_golden_vogel_layout(monads: &mut [crate::ontology::monad::Monad], max_radius: f64) {
        if monads.is_empty() {
            return;
        }

        const PHI: f64 = 1.618_033_988_749_895;
        const GOLDEN_ANGLE_DEG: f64 = 137.507_764_050_037_85; // 360 / φ²

        // Sort deterministically by (ring, kind, language, name, line_start, id)
        monads.sort_by(|a, b| {
            a.ring
                .cmp(&b.ring)
                .then_with(|| format!("{}", a.kind).cmp(&format!("{}", b.kind)))
                .then_with(|| a.language.cmp(&b.language))
                .then_with(|| a.name.cmp(&b.name))
                .then_with(|| a.line_start.cmp(&b.line_start))
                .then_with(|| a.id.cmp(&b.id))
        });

        // If >= 25 monads and any of the 5 Golden Rings is sparse (<5%) or overloaded (>38%),
        // apply Golden Ratio annular area weights φ^((k-1)*0.32) across Rings 1..=5
        if monads.len() >= 25 {
            let mut counts = [0usize; 6];
            for m in monads.iter() {
                let r = m.ring.clamp(1, 5) as usize;
                counts[r] += 1;
            }
            let min_required = (monads.len() / 20).max(2);
            let max_allowed = monads.len() * 38 / 100;
            let balanced = (1..=5).all(|r| counts[r] >= min_required && counts[r] <= max_allowed);

            if !balanced {
                let weights: Vec<f64> = (1..=5)
                    .map(|k| PHI.powf((k as f64 - 1.0) * 0.32))
                    .collect();
                let weight_sum: f64 = weights.iter().sum();
                let total = monads.len();

                let mut cumulative = 0.0;
                let mut thresholds = [0usize; 5];
                for (i, w) in weights.iter().enumerate() {
                    cumulative += w / weight_sum;
                    thresholds[i] = if i == 4 {
                        total
                    } else {
                        (cumulative * (total as f64)).round() as usize
                    };
                }

                for (idx, m) in monads.iter_mut().enumerate() {
                    let mut assigned = 5u32;
                    for (r_idx, &thresh) in thresholds.iter().enumerate() {
                        if idx < thresh {
                            assigned = (r_idx as u32) + 1;
                            break;
                        }
                    }
                    m.ring = assigned;
                }
            }
        }

        let total_rings = monads.iter().map(|m| m.ring).max().unwrap_or(5).max(4);

        // Re-sort deterministically by (ring, language, kind, name, line_start, id)
        // so the same index `n` drives BOTH angle θ_n AND radius r_n (Vogel's invariant)
        monads.sort_by(|a, b| {
            a.ring
                .cmp(&b.ring)
                .then_with(|| a.language.cmp(&b.language))
                .then_with(|| format!("{}", a.kind).cmp(&format!("{}", b.kind)))
                .then_with(|| a.name.cmp(&b.name))
                .then_with(|| a.line_start.cmp(&b.line_start))
                .then_with(|| a.id.cmp(&b.id))
        });

        // Precompute Golden Harmonic annular bands [r_start, r_end] outside the Bindu sanctuary
        let r_start = (max_radius * 0.14).max(44.0);
        let r_end = max_radius * 0.92;
        let weights: Vec<f64> = (1..=total_rings)
            .map(|k| PHI.powf((k as f64 - 1.0) * 0.32))
            .collect();
        let weight_sum: f64 = weights.iter().sum::<f64>().max(1.0);
        let usable_span = (r_end - r_start).max(100.0);

        let mut band_bounds: Vec<(f64, f64, f64)> = Vec::with_capacity(total_rings as usize);
        let mut cursor = r_start;
        for w in &weights {
            let span = usable_span * (w / weight_sum);
            let center = cursor + span * 0.5;
            // Golden division: active seed zone = span / φ (61.8%), breathing corridor = 38.2%
            let half_active = (span / PHI) * 0.5;
            band_bounds.push((center - half_active, center, center + half_active));
            cursor += span;
        }

        // Group slices by ring and assign coupled (r_n, θ_n)
        let mut start = 0;
        while start < monads.len() {
            let ring = monads[start].ring.clamp(1, total_rings);
            let mut end = start + 1;
            while end < monads.len() && monads[end].ring == monads[start].ring {
                end += 1;
            }

            let count = end - start;
            let (r_min, r_center, r_max) = band_bounds[(ring - 1) as usize];
            let ring_phase_deg = ((ring - 1) as f64) * 72.0; // Pentagonal golden phase shift

            for (idx, m) in monads[start..end].iter_mut().enumerate() {
                let theta = ((idx as f64) * GOLDEN_ANGLE_DEG + ring_phase_deg) % 360.0;
                let r = if count <= 6 {
                    r_center
                } else {
                    // Exact Vogel area-preserving sunflower radius on annulus [r_min, r_max]
                    let u = ((idx as f64) + 0.5) / (count as f64);
                    (r_min * r_min + u * (r_max * r_max - r_min * r_min)).sqrt()
                };
                m.coord.r = r;
                m.coord.theta = theta;
            }

            start = end;
        }
    }

    pub fn classify_architectural_ring(
        file_path: &std::path::Path,
        kind: &crate::ontology::monad::MonadKind,
        depth: usize,
    ) -> u32 {
        use crate::ontology::monad::MonadKind;

        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let path_str = file_path
            .to_string_lossy()
            .replace('\\', "/")
            .to_lowercase();

        // 1. Ring 5: Sensory Periphery, UI Components, Panels, Views & Styles
        if matches!(ext.as_str(), "css" | "scss" | "sass" | "less" | "astro" | "html" | "tsx" | "jsx" | "vue" | "svelte")
            || path_str.contains("/components/")
            || path_str.contains("/panels/")
            || path_str.contains("/pages/")
            || path_str.contains("/styles/")
            || path_str.contains("/views/")
            || path_str.contains("/ui/")
        {
            return 5;
        }

        // 2. Ring 1: Core Ontology, Contracts, Schemas, Traits, Enums & Manifests
        if path_str.contains("/ontology/")
            || path_str.contains("/types/")
            || path_str.contains("/schema")
            || path_str.contains("/models/")
            || path_str.contains("/primitives/")
            || matches!(ext.as_str(), "toml" | "json" | "yaml" | "yml")
            || matches!(kind, MonadKind::Trait | MonadKind::Enum | MonadKind::TypeAlias)
        {
            return 1;
        }

        // 3. Ring 2: Mathematical/Spatial Substrate, Persistence, State & Data Structs
        if path_str.contains("/geometry/")
            || path_str.contains("/polar")
            || path_str.contains("/math")
            || path_str.contains("/persistence/")
            || path_str.contains("/store")
            || path_str.contains("/surreal")
            || path_str.contains("/language/")
            || path_str.contains("/plugins/")
            || path_str.contains("/lib/d3/")
            || path_str.contains("/lib/state/")
            || path_str.contains("/utils/")
            || matches!(kind, MonadKind::Struct | MonadKind::Constant)
        {
            return 2;
        }

        // 4. Ring 4: Interfaces, Tauri IPC, CLI, MCP Server, Watcher & Boundary APIs
        if path_str.contains("/interface/")
            || path_str.contains("/api/")
            || path_str.contains("/cli")
            || path_str.contains("/mcp")
            || path_str.contains("/lib/tauri/")
            || path_str.contains("/commands")
            || path_str.contains("watcher")
            || path_str.ends_with("/main.rs")
            || path_str.ends_with("/lib.rs")
        {
            return 4;
        }

        // 5. Ring 3: Domain Weavers, Distillation Engines, Compilers, Synarchy & Collaboration
        if path_str.contains("/weaver/")
            || path_str.contains("/compiler")
            || path_str.contains("/diff")
            || path_str.contains("/template/")
            || path_str.contains("/synarchy/")
            || path_str.contains("/collaboration/")
            || path_str.contains("/network/")
            || path_str.contains("/threader")
            || path_str.contains("/services/")
        {
            return 3;
        }

        // Fallback: distribute across [2..=4] by depth
        (((depth as u32) % 3) + 2).clamp(1, 5)
    }

    fn collect_files_recursive(
        current: &PathBuf,
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
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if matches!(
                    file_name,
                    "mandala_state.json"
                        | "package-lock.json"
                        | "pnpm-lock.yaml"
                        | "Cargo.lock"
                        | "yarn.lock"
                ) {
                    continue;
                }
                if let Some(ext) = path.extension() {
                    if extensions.contains(&ext.to_str().unwrap_or("")) {
                        collected.push((depth, path));
                    }
                }
            } else if path.is_dir() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                if !name.starts_with('.')
                    && name != "target"
                    && name != "node_modules"
                    && name != "__pycache__"
                    && name != "dist"
                    && name != "build"
                    && name != ".git"
                {
                    Self::collect_files_recursive(&path, extensions, collected, depth + 1)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scan_mandala_project() {
        let root = PathBuf::from("..");
        if root.join(".git").exists() {
            let db = crate::persistence::surreal_bridge::connect_embedded().await.unwrap();
            let entry = ProjectScanner::scan_and_index_with_db(&db, &root).await.unwrap();
            assert!(entry.monad_count > 0);
            assert_eq!(entry.project_type, ProjectType::Git);
            println!("✅ Scanned project: {} with {} monads and {} rings", entry.name, entry.monad_count, entry.ring_count);
        }
    }
}