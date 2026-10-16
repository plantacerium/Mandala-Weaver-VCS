use crate::template::DistillationTemplate;
use crate::weaver::source_compiler;
use serde::{Deserialize, Serialize};
use tauri::State;

#[tauri::command]
pub async fn distill_from_template(
    db: State<'_, crate::persistence::surreal_bridge::SharedDb>,
    template_path: String,
) -> Result<String, String> {
    let template_content = std::fs::read_to_string(&template_path)
        .map_err(|e| format!("Failed to read template: {}", e))?;

    let db_conn = db.read().await;

    // Si es una receta espacial con semillas (o manifiesto unificado), intentar primero el Spatial Weaver SO(2)
    if let Ok(recipe) = serde_yml::from_str::<crate::weaver::spatial_weaver::DistillationRecipe>(&template_content) {
        if !recipe.seeds.is_empty() {
            let store = crate::persistence::MandalaStore::from_existing_db(db_conn.clone());
            let weaver = crate::weaver::spatial_weaver::Weaver::new(&store);
            if let Ok(artifact) = weaver.distill(&recipe).await {
                return Ok(artifact.assembled_source);
            }
        }
    }

    let template: DistillationTemplate = serde_yml::from_str(&template_content)
        .map_err(|e| format!("Failed to parse template: {}", e))?;

    let engine = crate::template::engine::TemplateEngine::new(db_conn.clone());
    let monads = engine
        .resolve(&template)
        .await
        .map_err(|e| format!("Failed to resolve template: {}", e))?;

    if monads.is_empty() {
        return Err("No monads matched the template criteria.".to_string());
    }

    let source = source_compiler::distill_source_with_adapters(&monads, &template.adapters);

    Ok(source)
}

#[tauri::command]
pub async fn distill_from_template_content(
    db: State<'_, crate::persistence::surreal_bridge::SharedDb>,
    template_content: String,
) -> Result<TemplateResult, String> {
    let db_conn = db.read().await;

    // Intentar primero con el motor Spatial Weaver SO(2) si tiene semillas definidas
    if let Ok(recipe) = serde_yml::from_str::<crate::weaver::spatial_weaver::DistillationRecipe>(&template_content) {
        if !recipe.seeds.is_empty() {
            let store = crate::persistence::MandalaStore::from_existing_db(db_conn.clone());
            let weaver = crate::weaver::spatial_weaver::Weaver::new(&store);
            if let Ok(artifact) = weaver.distill(&recipe).await {
                let monad_names: Vec<String> = artifact.traces.iter().map(|t| t.symbol_name.clone()).collect();
                let max_r = artifact.traces.iter().map(|t| t.selected_ring).max().unwrap_or(1);
                return Ok(TemplateResult {
                    source: artifact.assembled_source,
                    monad_count: artifact.traces.len(),
                    monad_names,
                    rings_included: max_r.to_string(),
                    output_file: format!("{}.rs", recipe.name),
                });
            }
        }
    }

    let template: DistillationTemplate = serde_yml::from_str(&template_content)
        .map_err(|e| format!("Invalid template YAML: {}", e))?;

    let engine = crate::template::engine::TemplateEngine::new(db_conn.clone());
    let monads = engine
        .resolve(&template)
        .await
        .map_err(|e| format!("Failed to resolve template: {}", e))?;

    if monads.is_empty() {
        return Err("No monads matched the template criteria.".to_string());
    }

    let source = source_compiler::distill_source_with_adapters(&monads, &template.adapters);
    let monad_names: Vec<String> = monads.iter().map(|m| m.name.clone()).collect();
    let rings: Vec<u32> = monads.iter().map(|m| m.ring).collect();
    let primary_ring = rings.iter().min().copied();

    Ok(TemplateResult {
        source,
        monad_count: monads.len(),
        monad_names,
        rings_included: primary_ring.map(|r| r.to_string()).unwrap_or_default(),
        output_file: template.output.filename,
    })
}

#[tauri::command]
pub async fn validate_template(template_path: String) -> Result<ValidationResult, String> {
    let template_content = std::fs::read_to_string(&template_path)
        .map_err(|e| format!("Failed to read template: {}", e))?;

    validate_template_content(template_content).await
}

#[tauri::command]
pub async fn validate_template_content(
    template_content: String,
) -> Result<ValidationResult, String> {
    if let Ok(template) = serde_yml::from_str::<DistillationTemplate>(&template_content) {
        return Ok(ValidationResult {
            valid: true,
            name: template.name,
            version: template.version,
            vectors_count: template.vectors.len(),
        });
    }

    if let Ok(recipe) = serde_yml::from_str::<crate::weaver::spatial_weaver::DistillationRecipe>(&template_content) {
        return Ok(ValidationResult {
            valid: true,
            name: recipe.name,
            version: "1.0.0-so2".to_string(),
            vectors_count: recipe.seeds.len(),
        });
    }

    let err = serde_yml::from_str::<DistillationTemplate>(&template_content)
        .err()
        .map(|e| e.to_string())
        .unwrap_or_else(|| "Unknown YAML format error".to_string());
    Err(format!("Invalid template YAML: {}", err))
}

#[tauri::command]
pub fn create_template(name: String, version: String) -> Result<String, String> {
    let template = DistillationTemplate {
        name,
        version,
        rings: crate::template::RingSelector::Latest,
        vectors: vec![],
        exclude: vec![],
        adapters: vec![],
        output: crate::template::OutputConfig {
            filename: "output.rs".to_string(),
            structure: crate::template::OutputStructure::Flat,
            generate_modules: true,
        },
        variables: Default::default(),
    };
    serde_yml::to_string(&template).map_err(|e| format!("Failed to serialize: {}", e))
}

#[tauri::command]
pub async fn distill_and_write(
    db: State<'_, crate::persistence::surreal_bridge::SharedDb>,
    template_content: String,
    output_dir: String,
) -> Result<WriteResult, String> {
    let template: DistillationTemplate = serde_yml::from_str(&template_content)
        .map_err(|e| format!("Invalid template YAML: {}", e))?;

    let db_conn = db.read().await;
    let engine = crate::template::engine::TemplateEngine::new(db_conn.clone());
    let monads = engine
        .resolve(&template)
        .await
        .map_err(|e| format!("Failed to resolve template: {}", e))?;

    if monads.is_empty() {
        return Err("No monads matched the template criteria.".to_string());
    }

    let output_path = std::path::PathBuf::from(&output_dir);
    let written_files = crate::weaver::file_writer::FileWriter::write_with_structure(
        &monads,
        &output_path,
        &template.output.structure,
        &template.adapters,
    )
    .await
    .map_err(|e| format!("Failed to write files: {}", e))?;

    Ok(WriteResult {
        files_written: written_files.len(),
        monad_count: monads.len(),
        output_dir,
    })
}

#[derive(Serialize, Deserialize)]
pub struct TemplateResult {
    pub source: String,
    pub monad_count: usize,
    pub monad_names: Vec<String>,
    pub rings_included: String,
    pub output_file: String,
}

#[derive(Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub name: String,
    pub version: String,
    pub vectors_count: usize,
}

#[derive(Serialize, Deserialize)]
pub struct WriteResult {
    pub files_written: usize,
    pub monad_count: usize,
    pub output_dir: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TemplateInfo {
    pub name: String,
    pub filename: String,
    pub content: String,
}

#[tauri::command]
pub async fn list_templates() -> Result<Vec<TemplateInfo>, String> {
    let mut templates = Vec::new();
    let template_dir = std::path::PathBuf::from("templates");
    if template_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(template_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let filename = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                        let name = filename.trim_end_matches(".yaml").to_string();
                        templates.push(TemplateInfo {
                            name,
                            filename,
                            content,
                        });
                    }
                }
            }
        }
    }
    Ok(templates)
}

#[tauri::command]
pub async fn distill_spatial_recipe(
    db: State<'_, crate::persistence::surreal_bridge::SharedDb>,
    recipe_yaml: String,
) -> Result<crate::weaver::spatial_weaver::DistilledArtifact, String> {
    let db_conn = db.read().await.clone();
    let store = crate::persistence::MandalaStore::from_existing_db(db_conn);
    let weaver = crate::weaver::spatial_weaver::Weaver::new(&store);
    weaver.distill_from_yaml(&recipe_yaml).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn distill_spatial_quick(
    db: State<'_, crate::persistence::surreal_bridge::SharedDb>,
    name: String,
    target_angle_deg: f64,
    max_ring: u32,
    seeds: Vec<String>,
) -> Result<crate::weaver::spatial_weaver::DistilledArtifact, String> {
    let target_angle_rad = target_angle_deg.to_radians();
    let recipe = crate::weaver::spatial_weaver::DistillationRecipe {
        name,
        target_angle_rad,
        max_ring,
        angular_policy: crate::weaver::spatial_weaver::AngularPolicy::GlobalTarget,
        ring_penalty_rad: 0.05,
        prelude: String::new(),
        seeds: seeds.into_iter().map(|s| crate::weaver::spatial_weaver::SeedSpec { symbol: s }).collect(),
        pins: Default::default(),
        external_symbols: Default::default(),
    };
    let db_conn = db.read().await.clone();
    let store = crate::persistence::MandalaStore::from_existing_db(db_conn);
    let weaver = crate::weaver::spatial_weaver::Weaver::new(&store);
    weaver.distill(&recipe).await.map_err(|e| e.to_string())
}