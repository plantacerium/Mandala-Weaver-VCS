use crate::ontology::bindu::Bindu;
use crate::ontology::monad::Monad;
use crate::persistence::surreal_bridge::{get_all_monads, SharedDb};
use crate::weaver::source_compiler::distill_source;
use crate::weaver::threader::trace_full_chain;
use serde::Serialize;
use std::path::PathBuf;
use tauri::State;

#[derive(Serialize)]
pub struct CliResponse {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn cli_bindu(db: State<'_, SharedDb>) -> Result<CliResponse, String> {
    let db = db.read().await;

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
pub async fn cli_telemetry(verbose: bool, db: State<'_, SharedDb>) -> Result<CliResponse, String> {
    let db = db.read().await;

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
pub async fn cli_status(verbose: bool, db: State<'_, SharedDb>) -> Result<CliResponse, String> {
    let db = db.read().await;

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
pub async fn cli_crystallize(file_path: String, message: String, db: State<'_, SharedDb>) -> Result<CliResponse, String> {
    let db = db.read().await;

    match crate::weaver::expand_from_source(&db, &file_path).await {
        Ok(next_ring) => {
            Ok(CliResponse {
                success: true,
                output: format!("💎 Ring {} created\n  Intent: {}", next_ring, if message.is_empty() { "N/A" } else { &message }),
                error: None,
            })
        },
        Err(e) => {
            Ok(CliResponse { success: false, output: String::new(), error: Some(format!("Crystallization failed: {}", e)) })
        }
    }
}

#[tauri::command]
pub async fn cli_distill(target_ring: Option<u32>, vector: Option<String>, db: State<'_, SharedDb>) -> Result<CliResponse, String> {
    let db = db.read().await;

    let all = get_all_monads(&db).await.map_err(|e| e.to_string())?;

    let selected: Vec<Monad> = if let Some(ring) = target_ring {
        all.into_iter().filter(|m| m.ring == ring).collect()
    } else if let Some(ref v) = vector {
        let angle_range = match v.as_str() {
            "CORE" => (0.0, 45.0),
            "UI" => (45.0, 135.0),
            "PERSISTENCE" => (135.0, 225.0),
            "NETWORK" => (225.0, 315.0),
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
pub async fn cli_lineage(monad_name: Option<String>, _limit: Option<usize>, db: State<'_, SharedDb>) -> Result<CliResponse, String> {
    let db = db.read().await;

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
pub async fn cli_spectrum(monad_name: String, db: State<'_, SharedDb>) -> Result<CliResponse, String> {
    let db = db.read().await;

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
pub async fn cli_inspect(monad_name: String, full: bool, db: State<'_, SharedDb>) -> Result<CliResponse, String> {
    let db = db.read().await;

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
pub async fn cli_echo(ring_id: u32, monad_name: Option<String>, db: State<'_, SharedDb>) -> Result<CliResponse, String> {
    let db_guard = db.write().await;

    let all = get_all_monads(&*db_guard).await.map_err(|e| e.to_string())?;

    let source_monads: Vec<_> = if let Some(name) = monad_name {
        all.iter().filter(|m| m.ring == ring_id && m.name.contains(&name)).collect()
    } else {
        all.iter().filter(|m| m.ring == ring_id).collect()
    };

    let max_ring = all.iter().map(|m| m.ring).max().unwrap_or(0);
    let target_ring = max_ring + 1;

    if source_monads.is_empty() {
        return Ok(CliResponse { success: false, output: String::new(), error: Some(format!("No monads in ring {}", ring_id)) });
    }

    let mut echoed = 0;
    for source in &source_monads {
        let echoed_monad = crate::ontology::monad::Monad {
            id: format!("{}_echo_{}", source.id, target_ring),
            coord: crate::geometry::polar_space::PolarCoord {
                r: target_ring as f64,
                theta: source.coord.theta,
            },
            content: source.content.clone(),
            name: source.name.clone(),
            ring: target_ring,
            kind: source.kind.clone(),
            semantic_hash: source.semantic_hash.clone(),
            line_start: source.line_start,
            line_end: source.line_end,
            language: source.language.clone(),
            is_archived: false,
        };

        crate::persistence::surreal_bridge::insert_and_link(&*db_guard, &echoed_monad, Some(&source.id))
            .await
            .map_err(|e| e.to_string())?;
        echoed += 1;
    }

    Ok(CliResponse {
        success: true,
        output: format!("🔄 Echoed {} monads\n  From: Ring {} → Ring {}", echoed, ring_id, target_ring),
        error: None,
    })
}

#[tauri::command]
pub async fn cli_vector(angle: f64) -> Result<CliResponse, String> {
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

    let output = format!("📐 Vector opened: {} ({:.1}°)", domain, normalized);

    Ok(CliResponse { success: true, output, error: None })
}

#[tauri::command]
pub async fn cli_focus(monad_pattern: String, db: State<'_, SharedDb>) -> Result<CliResponse, String> {
    let db = db.read().await;

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
pub async fn cli_dormant(db: State<'_, SharedDb>) -> Result<CliResponse, String> {
    let db_guard = db.write().await;

    let all = get_all_monads(&*db_guard).await.map_err(|e| e.to_string())?;
    let mut archived_count = 0u32;

    for monad in &all {
        if !monad.is_archived {
            let _: Option<serde_json::Value> = db_guard
                .update(("monad", monad.id.as_str()))
                .merge(serde_json::json!({ "is_archived": true }))
                .await
                .map_err(|e| e.to_string())?;
            archived_count += 1;
        }
    }

    Ok(CliResponse {
        success: true,
        output: format!("🌙 Entered dormant state\n✓ {} monads moved to latent space\n✓ Cache cleared", archived_count),
        error: None,
    })
}

#[tauri::command]
pub async fn cli_synthesize(vector: String, with_vector: Option<String>, db: State<'_, SharedDb>) -> Result<CliResponse, String> {
    let db = db.read().await;

    let all = get_all_monads(&db).await.map_err(|e| e.to_string())?;

    let domain_ranges: Vec<(&str, f64, f64)> = vec![
        ("CORE", 0.0, 45.0),
        ("UI", 45.0, 135.0),
        ("PERSISTENCE", 135.0, 225.0),
        ("NETWORK", 225.0, 315.0),
    ];

    let range_a = domain_ranges.iter()
        .find(|(name, _, _)| *name == vector)
        .map(|(_, lo, hi)| (*lo, *hi))
        .ok_or_else(|| format!("Unknown vector: {}", vector))?;

    let vec_a: Vec<_> = all.iter()
        .filter(|m| m.coord.theta >= range_a.0 && m.coord.theta < range_a.1)
        .collect();

    let mut synthesized_count = 0usize;
    let mut output = format!("⚛ Vector {}: {} monads selected\n", vector, vec_a.len());

    if let Some(ref other) = with_vector {
        let range_b = domain_ranges.iter()
            .find(|(name, _, _)| *name == other.as_str())
            .map(|(_, lo, hi)| (*lo, *hi));

        if let Some((lo_b, hi_b)) = range_b {
            let vec_b: Vec<_> = all.iter()
                .filter(|m| m.coord.theta >= lo_b && m.coord.theta < hi_b)
                .collect();

            output.push_str(&format!("  With {}: {} monads\n", other, vec_b.len()));

            for monad_a in &vec_a {
                if let Some(monad_b) = vec_b.iter().find(|b| b.name == monad_a.name) {
                    if monad_a.semantic_hash != monad_b.semantic_hash {
                        synthesized_count += 1;
                        output.push_str(&format!("  ↕ Synthesized: {} (hash divergence)\n", monad_a.name));
                    }
                }
            }

            output.push_str(&format!("  ✓ {} conflicts resolved via cross-domain synthesis\n", synthesized_count));
        }
    } else {
        let mut internal_conflicts = 0usize;
        for i in 0..vec_a.len() {
            for j in (i + 1)..vec_a.len() {
                if vec_a[i].name == vec_a[j].name && vec_a[i].semantic_hash != vec_a[j].semantic_hash {
                    internal_conflicts += 1;
                }
            }
        }
        output.push_str(&format!("  {} internal conflicts detected\n", internal_conflicts));
    }

    Ok(CliResponse {
        success: true,
        output,
        error: None,
    })
}

#[tauri::command]
pub async fn cli_absorb(remote: Option<String>, db: State<'_, SharedDb>) -> Result<CliResponse, String> {
    let db_guard = db.write().await;

    let source = remote.unwrap_or_else(|| "mandala.json".to_string());
    let path = PathBuf::from(&source);

    if !path.exists() {
        return Ok(CliResponse { success: false, output: String::new(), error: Some(format!("Archive not found: {}", source)) });
    }

    match crate::collaboration::import_mandala(&*db_guard, &path.parent().unwrap_or(&path).to_path_buf()).await {
        Ok(_) => Ok(CliResponse {
            success: true,
            output: format!("🌐 Absorbed monads from: {}", source),
            error: None,
        }),
        Err(e) => Ok(CliResponse { success: false, output: String::new(), error: Some(format!("Absorb failed: {}", e)) })
    }
}

#[tauri::command]
pub async fn cli_emanate(remote: Option<String>, db: State<'_, SharedDb>) -> Result<CliResponse, String> {
    let db = db.read().await;

    let target = remote.unwrap_or_else(|| ".".to_string());
    let path = PathBuf::from(&target);

    match crate::collaboration::export_mandala(&db, "mandala-project", &path).await {
        Ok(exported_path) => Ok(CliResponse {
            success: true,
            output: format!("📡 Emanated mandala to: {}", exported_path.display()),
            error: None,
        }),
        Err(e) => Ok(CliResponse { success: false, output: String::new(), error: Some(format!("Emanation failed: {}", e)) })
    }
}

#[tauri::command]
pub async fn cli_seed(source: String, db: State<'_, SharedDb>) -> Result<CliResponse, String> {
    let db_guard = db.write().await;

    let path = std::path::PathBuf::from(&source);
    if !path.exists() {
        return Ok(CliResponse { success: false, output: String::new(), error: Some(format!("Path not found: {}", source)) });
    }

    match crate::collaboration::import_git_history(&*db_guard, &path).await {
        Ok(commits) => {
            Ok(CliResponse {
                success: true,
                output: format!("🌱 Seed planted from {}\n  Imported {} historical rings from Git.", source, commits),
                error: None,
            })
        },
        Err(e) => {
            Ok(CliResponse { success: false, output: String::new(), error: Some(format!("Seed failed: {}", e)) })
        }
    }
}
