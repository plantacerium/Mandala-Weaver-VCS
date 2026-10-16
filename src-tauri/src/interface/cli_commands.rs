use clap::{Parser, Subcommand};
use thiserror::Error;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "weave")]
#[command(about = "Mandala Weaver: Circular Version Cooperation System", long_about = None)]
pub struct Cli {
    #[arg(short, long, global = true)]
    pub json: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Bindu,
    Seed {
        source: String,
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    Telemetry {
        #[arg(short, long)]
        verbose: bool,
    },
    Focus {
        monad: String,
    },
    Crystallize {
        #[arg(short, long, default_value = "")]
        message: String,
        #[arg(default_value = ".")]
        file: String,
    },
    Vector {
        angle: f64,
    },
    Dormant,
    Distill {
        coordinates: Option<String>,
        #[arg(short, long)]
        ring: Option<u32>,
        #[arg(short, long)]
        vector: Option<String>,
        #[arg(short, long)]
        template: Option<PathBuf>,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    Spectrum {
        monad: String,
    },
    Lineage {
        #[arg(short, long)]
        monad: Option<String>,
        #[arg(short, long)]
        limit: Option<usize>,
    },
    Echo {
        ring_id: u32,
        #[arg(short, long)]
        monad: Option<String>,
    },
    Absorb {
        #[arg(short, long)]
        remote: Option<String>,
    },
    Synthesize {
        vector: String,
        #[arg(short, long)]
        with_vector: Option<String>,
    },
    Emanate {
        #[arg(short, long)]
        remote: Option<String>,
    },
    Status {
        #[arg(short, long)]
        verbose: bool,
    },
    Inspect {
        monad_id: String,
        #[arg(short, long)]
        full: bool,
    },
    Plugins,
    Audit,
    /// Run Model Context Protocol (MCP) server over stdio for AI agents and developer tools
    Mcp,
    /// Recursively scan and index codebase files into radial rings without git
    Scan {
        path: Option<PathBuf>,
    },
    /// Ontological self-knowledge: shatter monolith into atomic monads, centrality rings, and distill recipe
    #[command(alias = "unweave", alias = "shatter", alias = "refract")]
    Innerknow {
        path: Option<PathBuf>,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Debug, Error)]
pub enum CliError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Project not found: {0}")]
    ProjectNotFound(String),
    #[error("Monad not found: {0}")]
    MonadNotFound(String),
}

impl From<CliError> for String {
    fn from(e: CliError) -> String {
        e.to_string()
    }
}

pub type CliResult<T> = Result<T, CliError>;

pub async fn run_cli() -> Result<(), CliError> {
    let cli = Cli::parse();
    let json = cli.json;
    
    match cli.command {
        Commands::Bindu => cmd_bindu(json).await?,
        Commands::Seed { source, path } => cmd_seed(&source, path, json).await?,
        Commands::Telemetry { verbose } => cmd_telemetry(verbose, json).await?,
        Commands::Focus { monad } => cmd_focus(&monad, json).await?,
        Commands::Crystallize { message, file } => cmd_crystallize(&message, &file, json).await?,
        Commands::Vector { angle } => cmd_vector(angle, json).await?,
        Commands::Dormant => cmd_dormant(json).await?,
        Commands::Distill { coordinates, ring, vector, template, output } => {
            cmd_distill(coordinates, ring, vector, template, output, json).await?
        }
        Commands::Spectrum { monad } => cmd_spectrum(&monad, json).await?,
        Commands::Lineage { monad, limit } => cmd_lineage(monad, limit, json).await?,
        Commands::Echo { ring_id, monad } => cmd_echo(ring_id, monad, json).await?,
        Commands::Absorb { remote } => cmd_absorb(remote, json).await?,
        Commands::Synthesize { vector, with_vector } => cmd_synthesize(&vector, with_vector, json).await?,
        Commands::Emanate { remote } => cmd_emanate(remote, json).await?,
        Commands::Status { verbose } => cmd_status(verbose, json).await?,
        Commands::Inspect { monad_id, full } => cmd_inspect(&monad_id, full, json).await?,
        Commands::Plugins => cmd_plugins(json).await?,
        Commands::Audit => cmd_audit(json).await?,
        Commands::Mcp => crate::interface::mcp::run_mcp_server().await.map_err(|e| CliError::Database(e.to_string()))?,
        Commands::Scan { path } => cmd_scan(path, json).await?,
        Commands::Innerknow { path, output } => cmd_innerknow(path, output, json).await?,
    }

    Ok(())
}

fn print_result<T: serde::Serialize>(result: T, json: bool) {
    if json {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    }
}

use crate::ontology::bindu::Bindu;
use crate::ontology::monad::Monad;
use crate::persistence::surreal_bridge::{
    connect_db, get_all_monads, get_ring, insert_and_link,
};
use crate::weaver::ast_extractor::extract_raw_monads;
use crate::weaver::source_compiler::distill_source;
use crate::weaver::threader::trace_full_chain;

#[derive(serde::Serialize)]
struct BinduOutput {
    project_name: String,
    coordinates: (f64, f64),
    timestamp: u64,
}

async fn cmd_bindu(json: bool) -> Result<(), CliError> {
    println!("🌑 Initializing Bindu (Point Zero)...");
    
    let current_dir = std::env::current_dir().map_err(|e| CliError::Database(e.to_string()))?;
    let db = connect_db(Some(&current_dir)).await.map_err(|e| CliError::Database(e.to_string()))?;
    
    let bindu = Bindu::genesis("mandala-project");
    let content = serde_json::to_value(&bindu).map_err(|e| CliError::Parse(e.to_string()))?;
    
    let _: Option<serde_json::Value> = db.create(("bindu", "genesis"))
        .content(content)
        .await
        .map_err(|e| CliError::Database(e.to_string()))?;
    
    println!("  Timestamp: {}", bindu.timestamp);
    
    print_result(BinduOutput {
        project_name: bindu.project_name,
        coordinates: (0.0, 0.0),
        timestamp: bindu.timestamp,
    }, json);
    
    Ok(())
}

#[derive(serde::Serialize)]
struct SeedOutput {
    monads_imported: usize,
    rings: u32,
    project_name: String,
}

async fn cmd_seed(source: &str, path: Option<PathBuf>, json: bool) -> Result<(), CliError> {
    let target_path = path.unwrap_or_else(|| PathBuf::from(source));
    
    if !target_path.exists() {
        return Err(CliError::ProjectNotFound(format!("Path not found: {}", source)));
    }
    
    println!("🌱 Planting Bindu from: {}", target_path.display());
    
    let db = connect_db(crate::persistence::surreal_bridge::find_project_root().as_deref()).await.map_err(|e| CliError::Database(e.to_string()))?;
    
    if target_path.join(".git").exists() {
        let commits = crate::collaboration::import_git_history(&db, &target_path)
            .await
            .map_err(|e| CliError::Database(e.to_string()))?;
        println!("  Imported {} historical rings from Git.", commits);
    } else {
        let entry = crate::synarchy::ProjectScanner::scan_and_index_with_db(&db, &target_path)
            .await
            .map_err(|e| CliError::Database(e.to_string()))?;
        println!("  Indexed {} monads across {} rings.", entry.monad_count, entry.ring_count);
    }
    
    let all_monads = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
    let max_ring = all_monads.iter().map(|m| m.ring).max().unwrap_or(0);
    
    println!("✓ Planted {} monads in {} rings", all_monads.len(), max_ring.saturating_add(1));
    
    print_result(SeedOutput {
        monads_imported: all_monads.len(),
        rings: max_ring.saturating_add(1),
        project_name: source.to_string(),
    }, json);
    
    Ok(())
}

#[derive(serde::Serialize)]
struct TelemetryOutput {
    active_monads: usize,
    crystallized_monads: usize,
    ring_stats: Vec<RingStatOutput>,
    last_activity: Option<u64>,
}

#[derive(serde::Serialize)]
struct RingStatOutput {
    ring: u32,
    monad_count: usize,
}

async fn cmd_telemetry(verbose: bool, json: bool) -> Result<(), CliError> {
    println!("📡 Scanning topology...");
    
    let db = connect_db(crate::persistence::surreal_bridge::find_project_root().as_deref()).await.map_err(|e| CliError::Database(e.to_string()))?;
    let all_monads = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
    
    let mut ring_map: std::collections::HashMap<u32, Vec<&Monad>> = std::collections::HashMap::new();
    for m in &all_monads {
        ring_map.entry(m.ring).or_default().push(m);
    }
    
    let crystallized = all_monads.len();
    let active = 0;
    
    println!("\n📊 Ecosystem Pulse:");
    println!("  Active (latent): {}", active);
    println!("  Crystallized: {}", crystallized);
    
    let mut ring_stats = Vec::new();
    
    if verbose {
        println!("\n📀 Ring Breakdown:");
        let mut rings: Vec<_> = ring_map.keys().collect();
        rings.sort();
        for ring in rings {
            if let Some(monads) = ring_map.get(ring) {
                println!("  Ring {}: {} monads", ring, monads.len());
                ring_stats.push(RingStatOutput {
                    ring: *ring,
                    monad_count: monads.len(),
                });
            }
        }
    }
    
    print_result(TelemetryOutput {
        active_monads: active,
        crystallized_monads: crystallized,
        ring_stats,
        last_activity: None,
    }, json);
    
    Ok(())
}

#[derive(serde::Serialize)]
struct FocusOutput {
    monads_focused: Vec<String>,
}

async fn cmd_focus(monad_pattern: &str, json: bool) -> Result<(), CliError> {
    println!("🎯 Focusing: {}", monad_pattern);
    
    let db = connect_db(crate::persistence::surreal_bridge::find_project_root().as_deref()).await.map_err(|e| CliError::Database(e.to_string()))?;
    let all_monads = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
    
    let pattern = monad_pattern.replace('*', "");
    let matched: Vec<_> = all_monads.iter()
        .filter(|m| m.name.contains(&pattern))
        .collect();
    
    if matched.is_empty() {
        return Err(CliError::MonadNotFound(format!("No monads match: {}", monad_pattern)));
    }
    
    println!("✓ Focused {} monads", matched.len());
    let mut monads_focused = Vec::new();
    for m in &matched {
        println!("  - {} ({})", m.name, m.kind);
        monads_focused.push(m.name.clone());
    }
    
    print_result(FocusOutput {
        monads_focused,
    }, json);
    
    Ok(())
}

#[derive(serde::Serialize)]
struct CrystallizeOutput {
    new_ring: u32,
    monads_added: usize,
    monads_modified: usize,
    intent: String,
    timestamp: u64,
}

async fn cmd_crystallize(message: &str, file: &str, json: bool) -> Result<(), CliError> {
    println!("💎 Crystallizing: {}", if message.is_empty() { "No message" } else { message });
    
    let source_path = PathBuf::from(file);
    if !source_path.exists() {
        return Err(CliError::ProjectNotFound(format!("File not found: {}", file)));
    }
    
    let source_code = std::fs::read_to_string(&source_path)
        .map_err(|e| CliError::Io(e))?;
    
    let db = connect_db(crate::persistence::surreal_bridge::find_project_root().as_deref()).await.map_err(|e| CliError::Database(e.to_string()))?;
    let all_monads = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
    let current_max_ring = all_monads.iter().map(|m| m.ring).max().unwrap_or(0);
    let next_ring = current_max_ring + 1;
    
    let new_monads = extract_raw_monads(&source_code, next_ring);
    
    if new_monads.is_empty() {
        println!("⚠ No monads extracted from file");
        return Ok(());
    }
    
    let base_monads = get_ring(&db, current_max_ring).await.ok();
    let deltas = if let Some(ref base) = base_monads {
        crate::weaver::resolver::identify_deltas(base, &new_monads)
    } else {
        new_monads.clone()
    };
    
    let mut added = 0;
    let mut modified = 0;
    
    for monad in &deltas {
        let parent = base_monads.as_ref().and_then(|base| {
            base.iter().find(|m| m.name == monad.name)
        });
        let parent_id = parent.map(|p| p.id.as_str());
        
        insert_and_link(&db, monad, parent_id)
            .await
            .map_err(|e| CliError::Database(e.to_string()))?;
        
        if parent.is_some() {
            modified += 1;
        } else {
            added += 1;
        }
    }
    
    println!("✓ Ring {} created", next_ring);
    println!("  Added: {}", added);
    println!("  Modified: {}", modified);
    
    print_result(CrystallizeOutput {
        new_ring: next_ring,
        monads_added: added,
        monads_modified: modified,
        intent: message.to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }, json);
    
    Ok(())
}

#[derive(serde::Serialize)]
struct VectorOutput {
    old_angle: Option<f64>,
    new_angle: f64,
    vector_name: String,
    domain: String,
}

async fn cmd_vector(angle: f64, json: bool) -> Result<(), CliError> {
    println!("📐 Opening vector at {:.1}°", angle);
    
    let normalized = angle % 360.0;
    let domain = if normalized < 45.0 || normalized >= 315.0 {
        "CORE"
    } else if normalized < 135.0 {
        "UI"
    } else if normalized < 225.0 {
        "PERSISTENCE"
    } else {
        "NETWORK"
    };
    
    println!("✓ Vector opened: {} ({})", domain, normalized);
    
    print_result(VectorOutput {
        old_angle: None,
        new_angle: normalized,
        vector_name: domain.to_string(),
        domain: domain.to_string(),
    }, json);
    
    Ok(())
}

#[derive(serde::Serialize)]
struct DormantOutput {
    monads_dormant: usize,
    cache_cleared: bool,
}

async fn cmd_dormant(json: bool) -> Result<(), CliError> {
    println!("🌙 Entering dormant state...");

    let db = connect_db(crate::persistence::surreal_bridge::find_project_root().as_deref())
        .await
        .map_err(|e| CliError::Database(e.to_string()))?;

    let all = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
    let mut archived = 0u32;

    for monad in &all {
        if !monad.is_archived {
            let _: Option<serde_json::Value> = db
                .update(("monad", monad.id.as_str()))
                .merge(serde_json::json!({ "is_archived": true }))
                .await
                .map_err(|e| CliError::Database(e.to_string()))?;
            archived += 1;
        }
    }

    println!("✓ {} monads moved to latent space", archived);
    println!("✓ Cache cleared");

    print_result(DormantOutput {
        monads_dormant: archived as usize,
        cache_cleared: true,
    }, json);

    Ok(())
}

#[derive(serde::Serialize)]
struct DistillOutput2 {
    source: String,
    monad_count: usize,
    ring_count: usize,
}

async fn cmd_distill(
    _coordinates: Option<String>,
    ring: Option<u32>,
    vector: Option<String>,
    template: Option<PathBuf>,
    output: Option<PathBuf>,
    json: bool,
) -> Result<(), CliError> {
    println!("🔮 Distilling...");
    
    let db = connect_db(crate::persistence::surreal_bridge::find_project_root().as_deref()).await.map_err(|e| CliError::Database(e.to_string()))?;

    // Verificar si se especificó una receta espacial YAML o si existe distill.yaml
    let template_candidate = template.or_else(|| {
        let p = PathBuf::from("distill.yaml");
        if p.exists() && ring.is_none() && vector.is_none() {
            Some(p)
        } else {
            None
        }
    });

    if let Some(ref t_path) = template_candidate {
        if t_path.exists() {
            if let Ok(yaml_content) = std::fs::read_to_string(t_path) {
                if let Ok(recipe) = serde_yml::from_str::<crate::weaver::DistillationRecipe>(&yaml_content) {
                    let store = crate::persistence::MandalaStore::from_existing_db(db.clone());
                    let weaver = crate::weaver::Weaver::new(&store);
                    match weaver.distill(&recipe).await {
                        Ok(artifact) => {
                            if let Some(ref path) = output {
                                std::fs::write(path, &artifact.assembled_source).map_err(CliError::Io)?;
                                println!("✓ Spatial distilled artifact written to: {}", path.display());
                            } else {
                                println!("\n{}", artifact.assembled_source);
                            }
                            println!("✓ Spatially woven {} nodes for recipe '{}'", artifact.traces.len(), artifact.recipe_name);
                            print_result(DistillOutput2 {
                                source: if output.is_some() { "File".to_string() } else { "Stdout".to_string() },
                                monad_count: artifact.traces.len(),
                                ring_count: 1,
                            }, json);
                            return Ok(());
                        }
                        Err(e) => {
                            eprintln!("⚠ Spatial distillation notice: {}", e);
                        }
                    }
                }
            }
        }
    }
    
    let all_monads = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
    
    let selected: Vec<Monad> = if let Some(r) = ring {
        all_monads.iter().filter(|m| m.ring == r).cloned().collect()
    } else if let Some(ref v) = vector {
        let angle_range = match v.as_str() {
            "CORE" => (0.0, 45.0),
            "UI" => (45.0, 135.0),
            "PERSISTENCE" => (135.0, 225.0),
            "NETWORK" => (225.0, 315.0),
            _ => (0.0, 360.0),
        };
        all_monads.iter()
            .filter(|m| m.coord.theta >= angle_range.0 && m.coord.theta < angle_range.1)
            .cloned()
            .collect()
    } else {
        all_monads.clone()
    };
    
    let source = if selected.iter().map(|m| &m.language).collect::<std::collections::HashSet<_>>().len() > 1 {
        crate::weaver::source_compiler::distill_multi_lang(&selected, crate::weaver::source_compiler::CrossLangMode::SectionPerLanguage)
    } else {
        distill_source(&selected)
    };
    
    // Integration: Generate auto-imports for the distilled source
    for monad in &selected {
        let _imports = crate::weaver::auto_imports::ImportAnalyzer::generate_imports(monad, &all_monads);
    }
    
    if let Some(ref path) = output {
        if path.is_dir() {
            // Integration: Write as modules if output is a directory
            let _ = crate::weaver::file_writer::FileWriter::write_modules(&selected, path);
            println!("✓ Written as module tree to: {}", path.display());
        } else {
            std::fs::write(path, &source).map_err(|e| CliError::Io(e))?;
            println!("✓ Written to: {}", path.display());
        }
    } else {
        println!("\n{}", source);
    }
    
    let mut rings: Vec<u32> = selected.iter().map(|m| m.ring).collect();
    rings.sort();
    rings.dedup();
    
    println!("✓ Distilled {} monads from {} rings", selected.len(), rings.len());
    
    print_result(DistillOutput2 {
        source: if output.is_some() { "File".to_string() } else { "Stdout".to_string() },
        monad_count: selected.len(),
        ring_count: rings.len(),
    }, json);
    
    Ok(())
}

#[derive(serde::Serialize)]
struct SpectrumOutput2 {
    monad: String,
    ring_range: (u32, u32),
    old_hue: f64,
    new_hue: f64,
    hue_shift: f64,
    semantic_change: bool,
}

async fn cmd_spectrum(monad: &str, json: bool) -> Result<(), CliError> {
    println!("🌈 Analyzing spectrum: {}", monad);
    
    let db = connect_db(crate::persistence::surreal_bridge::find_project_root().as_deref()).await.map_err(|e| CliError::Database(e.to_string()))?;
    let monads = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
    
    let found: Vec<_> = monads.iter().filter(|m| m.name.contains(monad)).collect();
    
    if found.is_empty() {
        return Err(CliError::MonadNotFound(format!("No monad found: {}", monad)));
    }
    
    let mut rings: Vec<_> = found.iter().map(|m| m.ring).collect();
    rings.sort();
    let min_ring = *rings.first().unwrap();
    let max_ring = *rings.last().unwrap();
    
    let hex = &found.last().unwrap().semantic_hash[..6];
    let hue = u16::from_str_radix(hex, 16).unwrap_or(0) as f64 / 65535.0 * 360.0;
    
    let old_hex = &found.first().unwrap().semantic_hash[..6];
    let old_hue = u16::from_str_radix(old_hex, 16).unwrap_or(0) as f64 / 65535.0 * 360.0;
    
    let shift = (hue - old_hue).abs();
    let changed = shift > 1.0;
    
    println!("\n📊 Spectrum Analysis:");
    println!("  Monad: {}", found.last().unwrap().name);
    println!("  Rings: {} → {}", min_ring, max_ring);
    if min_ring != max_ring {
        println!("  Old hue: {:.1}°", old_hue);
    }
    println!("  New hue: {:.1}°", hue);
    println!("  Shift: {:.1}° ({})", shift, if changed { "CHANGED" } else { "UNCHANGED" });
    
    print_result(SpectrumOutput2 {
        monad: found.last().unwrap().name.clone(),
        ring_range: (min_ring, max_ring),
        old_hue,
        new_hue: hue,
        hue_shift: shift,
        semantic_change: changed,
    }, json);
    
    Ok(())
}

#[derive(serde::Serialize)]
struct LineageOutput2 {
    entries: Vec<LineageEntry2>,
    total_depth: usize,
}

#[derive(serde::Serialize)]
struct LineageEntry2 {
    monad_id: String,
    monad_name: String,
    ring: u32,
    theta: f64,
    semantic_hash: String,
    parent_id: Option<String>,
}

async fn cmd_lineage(monad: Option<String>, _limit: Option<usize>, json: bool) -> Result<(), CliError> {
    let _limit = _limit.unwrap_or(50);
    
    println!("🧬 Querying lineage...");
    
    let db = connect_db(crate::persistence::surreal_bridge::find_project_root().as_deref()).await.map_err(|e| CliError::Database(e.to_string()))?;
    
    let mut entries = Vec::new();
    
    if let Some(name) = monad {
        let monads = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
        let found: Vec<_> = monads.iter().filter(|m| m.name.contains(&name)).collect();
        
        if found.is_empty() {
            return Err(CliError::MonadNotFound(format!("No monad found: {}", name)));
        }
        
        let chain = trace_full_chain(&db, &found.last().unwrap().id)
            .await
            .map_err(|e| CliError::Database(e.to_string()))?;
        
        println!("\n📜 lineage for {}:", name);
        for (i, m) in chain.iter().enumerate() {
            println!("  [{}] Ring {}: {} ({:.1}°)", i, m.ring, m.name, m.coord.theta);
            entries.push(LineageEntry2 {
                monad_id: m.id.clone(),
                monad_name: m.name.clone(),
                ring: m.ring,
                theta: m.coord.theta,
                semantic_hash: m.semantic_hash.clone(),
                parent_id: None, // Simplified
            });
        }
    } else {
        let all = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
        let max_ring = all.iter().map(|m| m.ring).max().unwrap_or(0);
        
        println!("\n📜 Full lineage: {} rings", max_ring);
        for ring in (0..=max_ring).rev().take(5) {
            let count = all.iter().filter(|m| m.ring == ring).count();
            println!("  Ring {}: {} monads", ring, count);
        }
    }
    
    print_result(LineageOutput2 {
        entries,
        total_depth: 0,
    }, json);
    
    Ok(())
}

#[derive(serde::Serialize)]
struct EchoOutput2 {
    source_ring: u32,
    current_ring: u32,
    monads_echoed: Vec<String>,
}

async fn cmd_echo(ring_id: u32, monad: Option<String>, json: bool) -> Result<(), CliError> {
    println!("🔄 Echoing from Ring {}...", ring_id);

    let db = connect_db(crate::persistence::surreal_bridge::find_project_root().as_deref()).await.map_err(|e| CliError::Database(e.to_string()))?;
    let all = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;

    let source_monads: Vec<_> = if let Some(name) = monad {
        all.iter().filter(|m| m.ring == ring_id && m.name.contains(&name)).collect()
    } else {
        all.iter().filter(|m| m.ring == ring_id).collect()
    };

    let max_ring = all.iter().map(|m| m.ring).max().unwrap_or(0);
    let target_ring = max_ring + 1;

    if source_monads.is_empty() {
        return Err(CliError::MonadNotFound(format!("No monads in ring {}", ring_id)));
    }

    let mut echoed = 0;
    for source in &source_monads {
        let echoed_monad = crate::ontology::monad::Monad {
            id: format!("{}_echo_{}", source.id, target_ring),
            coord: crate::geometry::polar_space::PolarCoord {
                r: target_ring as f64,
                theta: source.coord.theta,
            },
            ring: target_ring,
            is_archived: false,
            ..(*source).clone()
        };

        insert_and_link(&db, &echoed_monad, Some(&source.id))
            .await
            .map_err(|e| CliError::Database(e.to_string()))?;
        echoed += 1;
    }

    println!("✓ Echoed {} monads to current ring", echoed);
    println!("  From: Ring {} → Ring {}", ring_id, target_ring);

    print_result(EchoOutput2 {
        source_ring: ring_id,
        current_ring: target_ring,
        monads_echoed: source_monads.iter().map(|m| m.name.clone()).collect(),
    }, json);

    Ok(())
}

#[derive(serde::Serialize)]
struct AbsorbOutput2 {
    monads_absorbed: usize,
    rings_expanded: u32,
    source: String,
}

async fn cmd_absorb(remote: Option<String>, json: bool) -> Result<(), CliError> {
    let source = remote.unwrap_or_else(|| "mandala.json".to_string());
    let path = PathBuf::from(&source);
    if !path.exists() {
        return Err(CliError::ProjectNotFound(format!("Archive not found: {}", source)));
    }

    println!("🌐 Absorbing from: {}", source);

    let db = connect_db(crate::persistence::surreal_bridge::find_project_root().as_deref())
        .await
        .map_err(|e| CliError::Database(e.to_string()))?;

    let before = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
    let before_rings = before.iter().map(|m| m.ring).max().unwrap_or(0);

    crate::collaboration::import_mandala(&db, &path)
        .await
        .map_err(|e| CliError::Database(e.to_string()))?;

    let after = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
    let after_rings = after.iter().map(|m| m.ring).max().unwrap_or(0);
    let absorbed = after.len().saturating_sub(before.len());
    let rings_expanded = after_rings.saturating_sub(before_rings);

    println!("✓ Absorbed {} new monads (total: {})", absorbed, after.len());

    print_result(AbsorbOutput2 {
        monads_absorbed: after.len(),
        rings_expanded,
        source,
    }, json);

    Ok(())
}

#[derive(serde::Serialize)]
struct SynthesizeOutput2 {
    vector_a: String,
    vector_b: Option<String>,
    synthesized_monads: Vec<String>,
    conflicts_resolved: usize,
}

async fn cmd_synthesize(vector: &str, with_vector: Option<String>, json: bool) -> Result<(), CliError> {
    println!("⚛ Synthesizing {} with {:?}...", vector, with_vector);
    
    let db = connect_db(crate::persistence::surreal_bridge::find_project_root().as_deref()).await.map_err(|e| CliError::Database(e.to_string()))?;

    // If vector points to an archive path, run geometric merge_mandala
    let candidate_path = PathBuf::from(vector);
    if candidate_path.exists() {
        let merge_res = crate::collaboration::merge_mandala(&db, &candidate_path)
            .await
            .map_err(|e| CliError::Database(e.to_string()))?;
        let names: Vec<String> = merge_res.merged_monads.iter().map(|m| m.name.clone()).collect();
        println!("✓ Merged {} monads ({} conflicts resolved)", names.len(), merge_res.conflicts_resolved.len());
        print_result(SynthesizeOutput2 {
            vector_a: vector.to_string(),
            vector_b: with_vector,
            synthesized_monads: names,
            conflicts_resolved: merge_res.conflicts_resolved.len(),
        }, json);
        return Ok(());
    }

    let all = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
    
    let angle_a = match vector {
        "CORE" => (0.0, 45.0),
        "UI" => (45.0, 135.0),
        "PERSISTENCE" => (135.0, 225.0),
        "NETWORK" => (225.0, 315.0),
        _ => return Err(CliError::Parse(format!("Unknown vector or archive path: {}", vector))),
    };
    
    let vec_a: Vec<_> = all.iter()
        .filter(|m| m.coord.theta >= angle_a.0 && m.coord.theta < angle_a.1)
        .collect();
    
    println!("✓ Vector {}: {} monads", vector, vec_a.len());
    
    let mut synthesized_monads: Vec<String> = vec_a.iter().map(|m| m.name.clone()).collect();
    let mut conflicts_resolved = 0usize;

    if let Some(b) = with_vector.clone() {
        let angle_b = match b.as_str() {
            "CORE" => (0.0, 45.0),
            "UI" => (45.0, 135.0),
            "PERSISTENCE" => (135.0, 225.0),
            "NETWORK" => (225.0, 315.0),
            _ => return Err(CliError::Parse(format!("Unknown vector: {}", b))),
        };
        
        let vec_b: Vec<_> = all.iter()
            .filter(|m| m.coord.theta >= angle_b.0 && m.coord.theta < angle_b.1)
            .collect();
        
        println!("  With {}: {} monads", b, vec_b.len());
        for m in vec_b {
            if synthesized_monads.contains(&m.name) {
                conflicts_resolved += 1;
            } else {
                synthesized_monads.push(m.name.clone());
            }
        }
    }
    
    print_result(SynthesizeOutput2 {
        vector_a: vector.to_string(),
        vector_b: with_vector,
        synthesized_monads,
        conflicts_resolved,
    }, json);
    
    Ok(())
}

#[derive(serde::Serialize)]
struct EmanateOutput2 {
    monads_emanated: usize,
    ring_range: (u32, u32),
    target: Option<String>,
}

async fn cmd_emanate(remote: Option<String>, json: bool) -> Result<(), CliError> {
    let target = remote.clone().unwrap_or_else(|| ".".to_string());
    let target_path = PathBuf::from(&target);
    println!("📡 Emanating to: {}", target);
    
    let db = connect_db(crate::persistence::surreal_bridge::find_project_root().as_deref()).await.map_err(|e| CliError::Database(e.to_string()))?;
    let all = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
    
    if all.is_empty() {
        return Ok(());
    }
    
    let exported_file = crate::collaboration::export_mandala(&db, "mandala-project", &target_path)
        .await
        .map_err(|e| CliError::Database(e.to_string()))?;
    
    let max_ring = all.iter().map(|m| m.ring).max().unwrap_or(0);
    let min_ring = all.iter().map(|m| m.ring).min().unwrap_or(0);
    
    println!("✓ Emanated {} monads to {}", all.len(), exported_file.display());
    println!("  Rings: {} → {}", min_ring, max_ring);
    
    print_result(EmanateOutput2 {
        monads_emanated: all.len(),
        ring_range: (min_ring, max_ring),
        target: Some(exported_file.display().to_string()),
    }, json);
    
    Ok(())
}

#[derive(serde::Serialize)]
struct StatusOutput {
    project_name: String,
    max_ring: u32,
    total_monads: usize,
    last_activity: Option<u64>,
}

async fn cmd_status(verbose: bool, json: bool) -> Result<(), CliError> {
    println!("📊 Mandala Status");
    println!("══════════════");
    
    let db = connect_db(crate::persistence::surreal_bridge::find_project_root().as_deref()).await.map_err(|e| CliError::Database(e.to_string()))?;
    let all = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
    
    let max_ring = all.iter().map(|m| m.ring).max().unwrap_or(0);
    
    println!("  Rings: {}", max_ring);
    println!("  Monads: {}", all.len());
    
    if verbose {
        let mut ring_map: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
        for m in &all {
            *ring_map.entry(m.ring).or_insert(0) += 1;
        }
        
        println!("\n📀 By Ring:");
        for ring in 0..=max_ring {
            if let Some(count) = ring_map.get(&ring) {
                println!("  Ring {}: {} monads", ring, count);
            }
        }
    }
    
    print_result(StatusOutput {
        project_name: "mandala-project".to_string(),
        max_ring,
        total_monads: all.len(),
        last_activity: None,
    }, json);
    
    Ok(())
}

#[derive(serde::Serialize)]
struct InspectOutput {
    id: String,
    name: String,
    kind: String,
    coord: (f64, f64),
    ring: u32,
    semantic_hash: String,
    content: String,
    line_start: u32,
    line_end: u32,
    language: String,
}

async fn cmd_inspect(monad_id: &str, full: bool, json: bool) -> Result<(), CliError> {
    let db = connect_db(crate::persistence::surreal_bridge::find_project_root().as_deref()).await.map_err(|e| CliError::Database(e.to_string()))?;
    let all = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
    
    let found = all.iter().find(|m| m.id == monad_id || m.name.contains(monad_id));
    
    let m = match found {
        Some(m) => m,
        None => return Err(CliError::MonadNotFound(format!("Not found: {}", monad_id))),
    };
    
    println!("🔍 {}", m.name);
    println!("  ID: {}", m.id);
    println!("  Kind: {}", m.kind);
    println!("  Ring: {}", m.ring);
    println!("  Position: ({:.1}, {:.1}°)", m.coord.r, m.coord.theta);
    println!("  Hash: {}", &m.semantic_hash[..12]);
    println!("  Lines: {} - {}", m.line_start, m.line_end);
    println!("  Language: {} ({})", m.language, crate::language::Language::from_extension(&m.language).display_name());
    
    if full {
        println!("\n📄 Content:");
        for line in m.content.lines().take(20) {
            println!("  {}", line);
        }
        if m.content.lines().count() > 20 {
            println!("  ... ({} more lines)", m.content.lines().count() - 20);
        }
    }
    
    print_result(InspectOutput {
        id: m.id.clone(),
        name: m.name.clone(),
        kind: m.kind.to_string(),
        coord: (m.coord.r, m.coord.theta),
        ring: m.ring,
        semantic_hash: m.semantic_hash.clone(),
        content: m.content.clone(),
        line_start: m.line_start,
        line_end: m.line_end,
        language: m.language.clone(),
    }, json);
    
    Ok(())
}

#[derive(serde::Serialize)]
struct PluginOutput {
    name: String,
    version: String,
    description: String,
}

async fn cmd_plugins(json: bool) -> Result<(), CliError> {
    println!("🔌 Loaded Plugins:");
    
    let registry = crate::plugins::PluginRegistry::init();
    let plugins = registry.get();
    
    let mut output = Vec::new();
    for p in plugins {
        println!("  - {} v{} ({})", p.name(), p.version(), p.description());
        let render = p.render(&[]);
        println!("    Render Support: {} (SVG Size: {} bytes)", render.interactive, render.svg.len());
        
        output.push(PluginOutput {
            name: p.name().to_string(),
            version: p.version().to_string(),
            description: p.description().to_string(),
        });
    }
    
    print_result(output, json);
    
    Ok(())
}

async fn cmd_audit(json: bool) -> Result<(), CliError> {
    println!("🔍 Auditing Mandala ecosystem...");
    
    let db = connect_db(crate::persistence::surreal_bridge::find_project_root().as_deref()).await.map_err(|e| CliError::Database(e.to_string()))?;
    let all = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
    
    let result = crate::weaver::source_compiler::validate_source_coherence(&all);
    
    match result {
        Ok(_) => {
            println!("✅ No incoherencies found. System is stable.");
            print_result(Vec::<String>::new(), json);
        }
        Err(reports) => {
            println!("⚠️ Found {} incoherencies:", reports.len());
            for r in &reports {
                println!("  - [{:?}] {}", r.kind, r.message);
            }
            print_result(reports, json);
        }
    }
    
    Ok(())
}

async fn cmd_scan(path: Option<PathBuf>, json: bool) -> Result<(), CliError> {
    let target = path.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let canonical = std::fs::canonicalize(&target).unwrap_or(target);
    if !json {
        println!("🌀 Scanning project at {} into Mandala space...", canonical.display());
    }

    let entry = crate::synarchy::ProjectScanner::scan_and_index(&canonical)
        .await
        .map_err(|e| CliError::Database(e.to_string()))?;

    if json {
        println!("{}", serde_json::to_string_pretty(&entry).unwrap_or_default());
    } else {
        println!("✅ Mandala Scan Complete!");
        println!("   Project:   {}", entry.name);
        println!("   Path:      {}", entry.path.display());
        println!("   Rings:     {}", entry.ring_count);
        println!("   Monads:    {}", entry.monad_count);
        println!("   Storage:   .mandala/db (SurrealKV)");
        println!("\n✨ Ready to view in UI! Run `pnpm run tauri dev` to see all nodes on the canvas.");
    }
    Ok(())
}

async fn cmd_innerknow(
    path: Option<PathBuf>,
    output: Option<PathBuf>,
    json: bool,
) -> Result<(), CliError> {
    let target = path.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let canonical = std::fs::canonicalize(&target).unwrap_or(target);

    if !json {
        println!("🔮 Invocando `innerknow` (autoconocimiento ontológico del código)...");
        println!("   Descomponiendo el monolito en mónadas atómicas por gravedad y contratos.");
        println!("   Objetivo: {}", canonical.display());
    }

    let report = crate::weaver::unweave::UnweaveEngine::unweave(&canonical, output.as_deref())
        .await
        .map_err(|e| CliError::Database(e.to_string()))?;

    if json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
    } else {
        println!("\n✨ ¡Proceso `innerknow` completado con éxito!");
        println!("   Proyecto:     {}", report.project_name);
        println!("   Mónadas:      {} unidades atómicas extraídas", report.monads_extracted);
        println!("   Anillos:      {} niveles concéntricos gravitacionales", report.max_ring);
        println!("   Dominios:     {} ({})", report.domains.len(), report.domains.join(", "));
        println!("   Receta YAML:  {}", report.recipe_path.display());
        println!("\n🏛️  Contratos Fundacionales / Núcleo Central (cercanos al Bindu):");
        for (i, c) in report.top_contracts.iter().take(8).enumerate() {
            println!(
                "   {:2}. [{}] (Anillo {}, Dominio: {}, In-Degree: {}, Out-Degree: {})",
                i + 1,
                if c.canonical_signature.is_empty() { &c.symbol_name } else { &c.canonical_signature },
                c.ring,
                c.domain,
                c.in_degree,
                c.out_degree
            );
        }
        println!("\n🌀 Para reconstruir o destilar el proyecto en cualquier momento:");
        println!("   weave distill --template {}", report.recipe_path.display());
        println!("✨ Abre la UI de Mandala Weaver (`pnpm run tauri dev`) para explorar el mandala organizado.");
    }

    Ok(())
}

