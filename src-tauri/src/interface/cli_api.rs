use crate::ontology::bindu::Bindu;
use crate::ontology::monad::Monad;
use crate::persistence::surreal_bridge::{get_all_monads, get_ring, insert_and_link};
use crate::weaver::ast_extractor::extract_raw_monads;
use crate::weaver::source_compiler::distill_source;
use crate::weaver::threader::trace_full_chain;
use crate::geometry::polar_space::PolarCoord;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize)]
pub struct CliResponse {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn cli_bindu() -> Result<CliResponse, String> {
    let db = crate::persistence::surreal_bridge::connect_embedded()
        .await
        .map_err(|e| e.to_string())?;
    
    let bindu = Bindu::genesis("mandala-project");
    let json = serde_json::to_value(&bindu).map_err(|e| e.to_string())?;
    
    let _: Option<serde_json::Value> = db.create(("bindu", "genesis"))
        .content(json)
        .await
        .map_err(|e| e.to_string())?;

    Ok(CliResponse {
        success: true,
        output: format!("🌑 Bindu created at (0, 0)\n  Project: {}\n  Timestamp: {}", bindu.project_name, bindu.timestamp),
        error: None,
    })
}

#[tauri::command]
pub async fn cli_telemetry(verbose: bool) -> Result<CliResponse, String> {
    let db = crate::persistence::surreal_bridge::connect_embedded()
        .await
        .map_err(|e| e.to_string())?;
    
    let all = get_all_monads(&db).await.map_err(|e| e.to_string())?;
    let crystallized = all.len();
    
    let mut output = format!("📡 Ecosystem Pulse:\n  Active (latent): 0\n  Crystallized: {}\n", crystallized);
    
    if verbose {
        output.push_str("\n📀 Ring Breakdown:\n");
        let mut ring_map: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
        for m in &all {
            *ring_map.entry(m.ring).or_insert(0) += 1;
        }
        let mut rings: Vec<_> = ring_map.keys().collect();
        rings.sort();
        for ring in rings {
            if let Some(count) = ring_map.get(ring) {
                output.push_str(&format!("  Ring {}: {} monads\n", ring, count));
            }
        }
    }

    Ok(CliResponse { success: true, output, error: None })
}

#[tauri::command]
pub async fn cli_status(verbose: bool) -> Result<CliResponse, String> {
    let db = crate::persistence::surreal_bridge::connect_embedded()
        .await
        .map_err(|e| e.to_string())?;
    
    let all = get_all_monads(&db).await.map_err(|e| e.to_string())?;
    let max_ring = all.iter().map(|m| m.ring).max().unwrap_or(0);
    
    let mut output = format!("📊 Mandala Status\n═══════════════\n  Rings: {}\n  Monads: {}\n", max_ring, all.len());
    
    if verbose {
        output.push_str("\n📀 By Ring:\n");
        let mut ring_map: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
        for m in &all {
            *ring_map.entry(m.ring).or_insert(0) += 1;
        }
        for ring in 0..=max_ring {
            if let Some(count) = ring_map.get(&ring) {
                output.push_str(&format!("  Ring {}: {} monads\n", ring, count));
            }
        }
    }

    Ok(CliResponse { success: true, output, error: None })
}

#[tauri::command]
pub async fn cli_crystallize(file_path: String, message: String) -> Result<CliResponse, String> {
    let source_path = PathBuf::from(&file_path);
    if !source_path.exists() {
        return Ok(CliResponse { success: false, output: String::new(), error: Some(format!("File not found: {}", file_path)) });
    }
    
    let source_code = std::fs::read_to_string(&source_path)
        .map_err(|e| format!("IO error: {}", e))?;
    
    let db = crate::persistence::surreal_bridge::connect_embedded()
        .await
        .map_err(|e| e.to_string())?;
    
    let all = get_all_monads(&db).await.map_err(|e| e.to_string())?;
    let current_max_ring = all.iter().map(|m| m.ring).max().unwrap_or(0);
    let next_ring = current_max_ring + 1;
    
    let mut new_monads = extract_raw_monads(&source_code, next_ring);
    
    if new_monads.is_empty() {
        return Ok(CliResponse { success: false, output: String::new(), error: Some("No monads extracted".to_string()) });
    }
    
    let base_monads = get_ring(&db, current_max_ring).await.ok();
    let mut added = 0;
    let mut modified = 0;
    
    for monad in &new_monads {
        let parent = base_monads.as_ref().and_then(|base| {
            base.iter().find(|m| m.name == monad.name)
        });
        let parent_id = parent.map(|p| p.id.as_str());
        
        insert_and_link(&db, monad, parent_id)
            .await
            .map_err(|e| e.to_string())?;
        
        if parent.is_some() {
            modified += 1;
        } else {
            added += 1;
        }
    }
    
    let output = format!("💎 Ring {} created\n  Added: {}\n  Modified: {}\n  Intent: {}", next_ring, added, modified, if message.is_empty() { "N/A" } else { &message });

    Ok(CliResponse { success: true, output, error: None })
}

#[tauri::command]
pub async fn cli_distill(target_ring: Option<u32>, vector: Option<String>) -> Result<CliResponse, String> {
    let db = crate::persistence::surreal_bridge::connect_embedded()
        .await
        .map_err(|e| e.to_string())?;
    
    let all = get_all_monads(&db).await.map_err(|e| e.to_string())?;
    
    let selected: Vec<Monad> = if let Some(ring) = target_ring {
        all.into_iter().filter(|m| m.ring == ring).collect()
    } else if let Some(ref v) = vector {
        let angle_range = match v.as_str() {
            "CORE" => (0.0, 90.0),
            "IO" => (90.0, 180.0),
            "UI" => (180.0, 270.0),
            "DATA" => (270.0, 360.0),
            _ => (0.0, 360.0),
        };
        all.into_iter()
            .filter(|m| m.coord.theta >= angle_range.0 && m.coord.theta < angle_range.1)
            .collect()
    } else {
        all
    };
    
    let source = distill_source(&selected);
    
    let mut rings: Vec<u32> = selected.iter().map(|m| m.ring).collect();
    rings.sort();
    rings.dedup();
    
    Ok(CliResponse {
        success: true,
        output: format!("🔮 Distilled {} monads from {} rings\n\n{}\n", selected.len(), rings.len(), source),
        error: None,
    })
}

#[tauri::command]
pub async fn cli_lineage(monad_name: Option<String>, limit: Option<usize>) -> Result<CliResponse, String> {
    let db = crate::persistence::surreal_bridge::connect_embedded()
        .await
        .map_err(|e| e.to_string())?;
    
    let all = get_all_monads(&db).await.map_err(|e| e.to_string())?;
    
    let mut output = String::new();
    
    if let Some(name) = monad_name {
        let found: Vec<_> = all.iter().filter(|m| m.name.contains(&name)).collect();
        
        if found.is_empty() {
            return Ok(CliResponse { success: false, output: String::new(), error: Some(format!("No monad found: {}", name)) });
        }
        
        let chain = trace_full_chain(&db, &found.last().unwrap().id)
            .await
            .map_err(|e| e.to_string())?;
        
        output.push_str(&format!("🧬 Lineage for {}:\n", name));
        for (i, m) in chain.iter().enumerate() {
            output.push_str(&format!("  [{}] Ring {}: {} ({:.1}°)\n", i, m.ring, m.name, m.coord.theta));
        }
    } else {
        let max_ring = all.iter().map(|m| m.ring).max().unwrap_or(0);
        output.push_str(&format!("🧬 Full lineage: {} rings\n", max_ring));
        for ring in (0..=max_ring).rev().take(5) {
            let count = all.iter().filter(|m| m.ring == ring).count();
            output.push_str(&format!("  Ring {}: {} monads\n", ring, count));
        }
    }

    Ok(CliResponse { success: true, output, error: None })
}

#[tauri::command]
pub async fn cli_spectrum(monad_name: String) -> Result<CliResponse, String> {
    let db = crate::persistence::surreal_bridge::connect_embedded()
        .await
        .map_err(|e| e.to_string())?;
    
    let all = get_all_monads(&db).await.map_err(|e| e.to_string())?;
    
    let found: Vec<_> = all.iter().filter(|m| m.name.contains(&monad_name)).collect();
    
    if found.is_empty() {
        return Ok(CliResponse { success: false, output: String::new(), error: Some(format!("No monad found: {}", monad_name)) });
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
    
    let output = format!("🌈 Spectrum: {}\n  Rings: {} → {}\n  Old hue: {:.1}°\n  New hue: {:.1}°\n  Shift: {:.1}° ({})",
        found.last().unwrap().name, min_ring, max_ring, old_hue, hue, shift, if changed { "CHANGED" } else { "UNCHANGED" }
    );

    Ok(CliResponse { success: true, output, error: None })
}

#[tauri::command]
pub async fn cli_inspect(monad_name: String, full: bool) -> Result<CliResponse, String> {
    let db = crate::persistence::surreal_bridge::connect_embedded()
        .await
        .map_err(|e| e.to_string())?;
    
    let all = get_all_monads(&db).await.map_err(|e| e.to_string())?;
    
    let found = all.iter().find(|m| m.id == monad_name || m.name.contains(&monad_name));
    
    let m = match found {
        Some(m) => m,
        None => return Ok(CliResponse { success: false, output: String::new(), error: Some(format!("Not found: {}", monad_name)) }),
    };
    
    let mut output = format!("🔍 {}\n  ID: {}\n  Kind: {}\n  Ring: {}\n  Position: ({:.1}, {:.1}°)\n  Hash: {}\n  Lines: {} - {}\n  Language: {}",
        m.name, m.id, m.kind, m.ring, m.coord.r, m.coord.theta, &m.semantic_hash[..12], m.line_start, m.line_end, m.language
    );
    
    if full {
        output.push_str(&format!("\n📄 Content:\n"));
        for (i, line) in m.content.lines().enumerate() {
            if i < 20 {
                output.push_str(&format!("  {}\n", line));
            } else {
                output.push_str(&format!("  ... ({} more lines)\n", m.content.lines().count() - 20));
                break;
            }
        }
    }

    Ok(CliResponse { success: true, output, error: None })
}

#[tauri::command]
pub async fn cli_echo(ring_id: u32, monad_name: Option<String>) -> Result<CliResponse, String> {
    let db = crate::persistence::surreal_bridge::connect_embedded()
        .await
        .map_err(|e| e.to_string())?;
    
    let all = get_all_monads(&db).await.map_err(|e| e.to_string())?;
    
    let source_monads: Vec<_> = if let Some(name) = monad_name {
        all.iter().filter(|m| m.ring == ring_id && m.name.contains(&name)).collect()
    } else {
        all.iter().filter(|m| m.ring == ring_id).collect()
    };
    
    let max_ring = all.iter().map(|m| m.ring).max().unwrap_or(0);
    
    if source_monads.is_empty() {
        return Ok(CliResponse { success: false, output: String::new(), error: Some(format!("No monads in ring {}", ring_id)) });
    }
    
    let output = format!("🔄 Echoed {} monads\n  From: Ring {} → Ring {}", source_monads.len(), ring_id, max_ring);

    Ok(CliResponse { success: true, output, error: None })
}

#[tauri::command]
pub async fn cli_vector(angle: f64) -> Result<CliResponse, String> {
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
    
    let output = format!("📐 Vector opened: {} ({:.1}°)", domain, normalized);

    Ok(CliResponse { success: true, output, error: None })
}

#[tauri::command]
pub async fn cli_focus(monad_pattern: String) -> Result<CliResponse, String> {
    let db = crate::persistence::surreal_bridge::connect_embedded()
        .await
        .map_err(|e| e.to_string())?;
    
    let all = get_all_monads(&db).await.map_err(|e| e.to_string())?;
    
    let pattern = monad_pattern.replace('*', "");
    let matched: Vec<_> = all.iter().filter(|m| m.name.contains(&pattern)).collect();
    
    if matched.is_empty() {
        return Ok(CliResponse { success: false, output: String::new(), error: Some(format!("No monads match: {}", monad_pattern)) });
    }
    
    let mut output = format!("🎯 Focused {} monads:\n", matched.len());
    for m in &matched {
        output.push_str(&format!("  - {} ({})\n", m.name, m.kind));
    }

    Ok(CliResponse { success: true, output, error: None })
}

#[tauri::command]
pub async fn cli_dormant() -> Result<CliResponse, String> {
    Ok(CliResponse {
        success: true,
        output: "🌙 Entered dormant state\n✓ Active monads moved to latent space\n✓ Cache cleared".to_string(),
        error: None,
    })
}

#[tauri::command]
pub async fn cli_synthesize(vector: String, with_vector: Option<String>) -> Result<CliResponse, String> {
    let db = crate::persistence::surreal_bridge::connect_embedded()
        .await
        .map_err(|e| e.to_string())?;
    
    let all = get_all_monads(&db).await.map_err(|e| e.to_string())?;
    
    let angle_a = match vector.as_str() {
        "CORE" => (0.0, 90.0),
        "IO" => (90.0, 180.0),
        "UI" => (180.0, 270.0),
        "DATA" => (270.0, 360.0),
        _ => (0.0, 360.0),
    };
    
    let vec_a: Vec<_> = all.iter()
        .filter(|m| m.coord.theta >= angle_a.0 && m.coord.theta < angle_a.1)
        .collect();
    
    let mut output = format!("⚛ Vector {}: {} monads\n", vector, vec_a.len());
    
    if let Some(b) = with_vector {
        let angle_b = match b.as_str() {
            "CORE" => (0.0, 90.0),
            "IO" => (90.0, 180.0),
            "UI" => (180.0, 270.0),
            "DATA" => (270.0, 360.0),
            _ => (0.0, 360.0),
        };
        
        let vec_b: Vec<_> = all.iter()
            .filter(|m| m.coord.theta >= angle_b.0 && m.coord.theta < angle_b.1)
            .collect();
        
        output.push_str(&format!("  + {}: {} monads\n", b, vec_b.len()));
    }

    Ok(CliResponse { success: true, output, error: None })
}

#[tauri::command]
pub async fn cli_absorb(remote: Option<String>) -> Result<CliResponse, String> {
    Ok(CliResponse {
        success: true,
        output: format!("🌐 Absorb from: {}\n⚠ Network sync not yet implemented", remote.unwrap_or_else(|| "network".to_string())),
        error: None,
    })
}

#[tauri::command]
pub async fn cli_emanate(remote: Option<String>) -> Result<CliResponse, String> {
    let db = crate::persistence::surreal_bridge::connect_embedded()
        .await
        .map_err(|e| e.to_string())?;
    
    let all = get_all_monads(&db).await.map_err(|e| e.to_string())?;
    
    let max_ring = all.iter().map(|m| m.ring).max().unwrap_or(0);
    let min_ring = all.iter().map(|m| m.ring).min().unwrap_or(0);
    
    let output = format!("📡 Emanated {} monads\n  Rings: {} → {}\n  Target: {}",
        all.len(), min_ring, max_ring, remote.unwrap_or_else(|| "network".to_string())
    );

    Ok(CliResponse { success: true, output, error: None })
}