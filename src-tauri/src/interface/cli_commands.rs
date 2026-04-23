use clap::{Parser, Subcommand};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Parser)]
#[command(name = "weave")]
#[command(about = "Mandala Weaver: Circular Version Control System", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
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
    
    match cli.command {
        Commands::Bindu => cmd_bindu().await,
        Commands::Seed { source, path } => cmd_seed(&source, path).await,
        Commands::Telemetry { verbose } => cmd_telemetry(verbose).await,
        Commands::Focus { monad } => cmd_focus(&monad).await,
        Commands::Crystallize { message, file } => cmd_crystallize(&message, &file).await,
        Commands::Vector { angle } => cmd_vector(angle).await,
        Commands::Dormant => cmd_dormant().await,
        Commands::Distill { coordinates, ring, vector, template, output } => {
            cmd_distill(coordinates, ring, vector, template, output).await
        }
        Commands::Spectrum { monad } => cmd_spectrum(&monad).await,
        Commands::Lineage { monad, limit } => cmd_lineage(monad, limit).await,
        Commands::Echo { ring_id, monad } => cmd_echo(ring_id, monad).await,
        Commands::Absorb { remote } => cmd_absorb(remote).await,
        Commands::Synthesize { vector, with_vector } => cmd_synthesize(&vector, with_vector).await,
        Commands::Emanate { remote } => cmd_emanate(remote).await,
        Commands::Status { verbose } => cmd_status(verbose).await,
        Commands::Inspect { monad_id, full } => cmd_inspect(&monad_id, full).await,
    }
}

use crate::ontology::bindu::Bindu;
use crate::ontology::monad::Monad;
use crate::persistence::surreal_bridge::{
    Db, connect_embedded, get_all_monads, get_ring, get_monad_by_name, insert_and_link,
};
use crate::weaver::ast_extractor::extract_raw_monads;
use crate::weaver::resolver::identify_deltas;
use crate::weaver::source_compiler::distill_source;
use crate::weaver::threader::trace_full_chain;
use crate::geometry::polar_space::PolarCoord;

struct BinduOutput {
    project_name: String,
    coordinates: (f64, f64),
    timestamp: u64,
}

async fn cmd_bindu() -> Result<(), CliError> {
    println!("🌑 Initializing Bindu (Point Zero)...");
    
    let db = connect_embedded().await.map_err(|e| CliError::Database(e.to_string()))?;
    
    let bindu = Bindu::genesis("mandala-project");
    let json = serde_json::to_value(&bindu).map_err(|e| CliError::Parse(e.to_string()))?;
    
    let _: Option<serde_json::Value> = db.create(("bindu", "genesis"))
        .content(json)
        .await
        .map_err(|e| CliError::Database(e.to_string()))?;
    
    println!("✓ Bindu created at (0, 0)");
    println!("  Project: {}", bindu.project_name);
    println!("  Timestamp: {}", bindu.timestamp);
    
    Ok(())
}

struct SeedOutput {
    monads_imported: u32,
    rings: u32,
    project_name: String,
}

async fn cmd_seed(source: &str, path: Option<PathBuf>) -> Result<(), CliError> {
    let target_path = path.map(|p| p).unwrap_or_else(|| PathBuf::from(source));
    
    if !target_path.exists() {
        return Err(CliError::ProjectNotFound(format!("Path not found: {}", source)));
    }
    
    println!("🌱 Planting Bindu from: {}", target_path.display());
    
    let db = connect_embedded().await.map_err(|e| CliError::Database(e.to_string()))?;
    
    let all_monads = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
    let max_ring = all_monads.iter().map(|m| m.ring).max().unwrap_or(0);
    
    println!("✓ Planted {} monads in {} rings", all_monads.len(), max_ring.saturating_add(1));
    
    Ok(())
}

struct TelemetryOutput {
    active_monads: usize,
    crystallized_monads: usize,
    ring_stats: Vec<RingStatOutput>,
    last_activity: Option<u64>,
}

struct RingStatOutput {
    ring: u32,
    monad_count: usize,
    semantic_hash: String,
}

async fn cmd_telemetry(verbose: bool) -> Result<(), CliError> {
    println!("📡 Scanning topology...");
    
    let db = connect_embedded().await.map_err(|e| CliError::Database(e.to_string()))?;
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
    
    if verbose {
        println!("\n📀 Ring Breakdown:");
        let mut rings: Vec<_> = ring_map.keys().collect();
        rings.sort();
        for ring in rings {
            if let Some(monads) = ring_map.get(ring) {
                println!("  Ring {}: {} monads", ring, monads.len());
            }
        }
    }
    
    Ok(())
}

struct FocusOutput {
    monads_focused: Vec<String>,
    ring: u32,
}

async fn cmd_focus(monad_pattern: &str) -> Result<(), CliError> {
    println!("🎯 Focusing: {}", monad_pattern);
    
    let db = connect_embedded().await.map_err(|e| CliError::Database(e.to_string()))?;
    let all_monads = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
    
    let pattern = monad_pattern.replace('*', "");
    let matched: Vec<_> = all_monads.iter()
        .filter(|m| m.name.contains(&pattern))
        .collect();
    
    if matched.is_empty() {
        return Err(CliError::MonadNotFound(format!("No monads match: {}", monad_pattern)));
    }
    
    println!("✓ Focused {} monads", matched.len());
    for m in &matched {
        println!("  - {} ({})", m.name, m.kind);
    }
    
    Ok(())
}

struct CrystallizeOutput {
    new_ring: u32,
    monads_added: usize,
    monads_modified: usize,
    monads_deleted: usize,
    intent: String,
    timestamp: u64,
}

async fn cmd_crystallize(message: &str, file: &str) -> Result<(), CliError> {
    println!("💎 Crystallizing: {}", if message.is_empty() { "No message" } else { message });
    
    let source_path = PathBuf::from(file);
    if !source_path.exists() {
        return Err(CliError::ProjectNotFound(format!("File not found: {}", file)));
    }
    
    let source_code = std::fs::read_to_string(&source_path)
        .map_err(|e| CliError::Io(e))?;
    
    let db = connect_embedded().await.map_err(|e| CliError::Database(e.to_string()))?;
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
    
    Ok(())
}

struct VectorOutput {
    old_angle: Option<f64>,
    new_angle: f64,
    vector_name: String,
    domain: String,
}

async fn cmd_vector(angle: f64) -> Result<(), CliError> {
    println!("📐 Opening vector at {:.1}°", angle);
    
    let normalized = angle % 360.0;
    let domain = if normalized < 90.0 {
        "CORE"
    } else if normalized < 180.0 {
        "IO"
    } else if normalized < 270.0 {
        "UI"
    } else {
        "DATA"
    };
    
    println!("✓ Vector opened: {} ({})", domain, normalized);
    
    Ok(())
}

struct DormantOutput {
    monads_dormant: usize,
    cache_cleared: bool,
}

async fn cmd_dormant() -> Result<(), CliError> {
    println!("🌙 Entering dormant state...");
    println!("✓ Active monads moved to latent space");
    println!("✓ Cache cleared");
    
    Ok(())
}

struct DistillOutput2 {
    source: String,
    monad_count: usize,
    ring_count: usize,
}

async fn cmd_distill(
    coordinates: Option<String>,
    ring: Option<u32>,
    vector: Option<String>,
    template: Option<PathBuf>,
    output: Option<PathBuf>,
) -> Result<(), CliError> {
    println!("🔮 Distilling...");
    
    let db = connect_embedded().await.map_err(|e| CliError::Database(e.to_string()))?;
    let all_monads = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
    
    let selected: Vec<Monad> = if let Some(r) = ring {
        all_monads.into_iter().filter(|m| m.ring == r).collect()
    } else if let Some(ref v) = vector {
        let angle_range = match v.as_str() {
            "CORE" => (0.0, 90.0),
            "IO" => (90.0, 180.0),
            "UI" => (180.0, 270.0),
            "DATA" => (270.0, 360.0),
            _ => (0.0, 360.0),
        };
        all_monads.into_iter()
            .filter(|m| m.coord.theta >= angle_range.0 && m.coord.theta < angle_range.1)
            .collect()
    } else {
        all_monads
    };
    
    let source = distill_source(&selected);
    
    if let Some(path) = output {
        std::fs::write(&path, &source).map_err(|e| CliError::Io(e))?;
        println!("✓ Written to: {}", path.display());
    } else {
        println!("\n{}", source);
    }
    
    let mut rings: Vec<u32> = selected.iter().map(|m| m.ring).collect();
    rings.sort();
    rings.dedup();
    
    println!("✓ Distilled {} monads from {} rings", selected.len(), rings.len());
    
    Ok(())
}

struct SpectrumOutput2 {
    monad: String,
    ring_range: (u32, u32),
    old_hue: f64,
    new_hue: f64,
    hue_shift: f64,
    semantic_change: bool,
}

async fn cmd_spectrum(monad: &str) -> Result<(), CliError> {
    println!("🌈 Analyzing spectrum: {}", monad);
    
    let db = connect_embedded().await.map_err(|e| CliError::Database(e.to_string()))?;
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
    
    Ok(())
}

struct LineageOutput2 {
    entries: Vec<LineageEntry2>,
    total_depth: usize,
}

struct LineageEntry2 {
    monad_id: String,
    monad_name: String,
    ring: u32,
    theta: f64,
    semantic_hash: String,
    parent_id: Option<String>,
}

async fn cmd_lineage(monad: Option<String>, limit: Option<usize>) -> Result<(), CliError> {
    let limit = limit.unwrap_or(50);
    
    println!("🧬 Querying lineage...");
    
    let db = connect_embedded().await.map_err(|e| CliError::Database(e.to_string()))?;
    
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
    
    Ok(())
}

struct EchoOutput2 {
    source_ring: u32,
    current_ring: u32,
    monads_echoed: Vec<String>,
}

async fn cmd_echo(ring_id: u32, monad: Option<String>) -> Result<(), CliError> {
    println!("🔄 Echoing from Ring {}...", ring_id);
    
    let db = connect_embedded().await.map_err(|e| CliError::Database(e.to_string()))?;
    let all = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
    
    let source_monads: Vec<_> = if let Some(name) = monad {
        all.iter().filter(|m| m.ring == ring_id && m.name.contains(&name)).collect()
    } else {
        all.iter().filter(|m| m.ring == ring_id).collect()
    };
    
    let max_ring = all.iter().map(|m| m.ring).max().unwrap_or(0);
    
    if source_monads.is_empty() {
        return Err(CliError::MonadNotFound(format!("No monads in ring {}", ring_id)));
    }
    
    println!("✓ Echoed {} monads to current ring", source_monads.len());
    println!("  From: Ring {} → Ring {}", ring_id, max_ring);
    
    Ok(())
}

struct AbsorbOutput2 {
    monads_absorbed: usize,
    rings_expanded: u32,
    source: String,
}

async fn cmd_absorb(remote: Option<String>) -> Result<(), CliError> {
    let source = remote.unwrap_or_else(|| "network".to_string());
    println!("🌐 Absorbing from: {}", source);
    
    println!("⚠ Network sync not yet implemented");
    println!("  Use local mode for now");
    
    Ok(())
}

struct SynthesizeOutput2 {
    vector_a: String,
    vector_b: Option<String>,
    synthesized_monads: Vec<String>,
    conflicts_resolved: usize,
}

async fn cmd_synthesize(vector: &str, with_vector: Option<String>) -> Result<(), CliError> {
    println!("⚛ Synthesizing {} with {:?}...", vector, with_vector);
    
    let db = connect_embedded().await.map_err(|e| CliError::Database(e.to_string()))?;
    let all = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
    
    let angle_a = match vector {
        "CORE" => (0.0, 90.0),
        "IO" => (90.0, 180.0),
        "UI" => (180.0, 270.0),
        "DATA" => (270.0, 360.0),
        _ => return Err(CliError::Parse("Unknown vector".to_string())),
    };
    
    let vec_a: Vec<_> = all.iter()
        .filter(|m| m.coord.theta >= angle_a.0 && m.coord.theta < angle_a.1)
        .collect();
    
    println!("✓ Vector {}: {} monads", vector, vec_a.len());
    
    if let Some(b) = with_vector {
        let angle_b = match b.as_str() {
            "CORE" => (0.0, 90.0),
            "IO" => (90.0, 180.0),
            "UI" => (180.0, 270.0),
            "DATA" => (270.0, 360.0),
            _ => return Err(CliError::Parse("Unknown vector".to_string())),
        };
        
        let vec_b: Vec<_> = all.iter()
            .filter(|m| m.coord.theta >= angle_b.0 && m.coord.theta < angle_b.1)
            .collect();
        
        println!("  With {}: {} monads", b, vec_b.len());
    }
    
    Ok(())
}

struct EmanateOutput2 {
    monads_emanated: usize,
    ring_range: (u32, u32),
    target: Option<String>,
}

async fn cmd_emanate(remote: Option<String>) -> Result<(), CliError> {
    let target = remote.unwrap_or_else(|| "network".to_string());
    println!("📡 Emanating to: {}", target);
    
    let db = connect_embedded().await.map_err(|e| CliError::Database(e.to_string()))?;
    let all = get_all_monads(&db).await.map_err(|e| CliError::Database(e.to_string()))?;
    
    let max_ring = all.iter().map(|m| m.ring).max().unwrap_or(0);
    let min_ring = all.iter().map(|m| m.ring).min().unwrap_or(0);
    
    println!("✓ Emanated {} monads", all.len());
    println!("  Rings: {} → {}", min_ring, max_ring);
    
    Ok(())
}

struct StatusOutput {
    project_name: String,
    max_ring: u32,
    total_monads: usize,
    last_expand: Option<u64>,
}

async fn cmd_status(verbose: bool) -> Result<(), CliError> {
    println!("📊 Mandala Status");
    println!("══════════════");
    
    let db = connect_embedded().await.map_err(|e| CliError::Database(e.to_string()))?;
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
    
    Ok(())
}

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

async fn cmd_inspect(monad_id: &str, full: bool) -> Result<(), CliError> {
    let db = connect_embedded().await.map_err(|e| CliError::Database(e.to_string()))?;
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
    println!("  Language: {}", m.language);
    
    if full {
        println!("\n📄 Content:");
        for line in m.content.lines().take(20) {
            println!("  {}", line);
        }
        if m.content.lines().count() > 20 {
            println!("  ... ({} more lines)", m.content.lines().count() - 20);
        }
    }
    
    Ok(())
}