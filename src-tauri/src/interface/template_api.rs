use crate::persistence::surreal_bridge::Db;
use crate::template::DistillationTemplate;
use crate::weaver::source_compiler;
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use tauri::State;

#[tauri::command]
pub async fn distill_from_template(
    db: State<'_, Surreal<Db>>,
    template_path: String,
) -> Result<String, String> {
    let template_content = std::fs::read_to_string(&template_path)
        .map_err(|e| format!("Failed to read template: {}", e))?;

    let template: DistillationTemplate = serde_yaml::from_str(&template_content)
        .map_err(|e| format!("Failed to parse template: {}", e))?;

    let engine = crate::template::engine::TemplateEngine::new(db.inner().clone());
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
    db: State<'_, Surreal<Db>>,
    template_content: String,
) -> Result<TemplateResult, String> {
    let template: DistillationTemplate = serde_yaml::from_str(&template_content)
        .map_err(|e| format!("Invalid template YAML: {}", e))?;

    let engine = crate::template::engine::TemplateEngine::new(db.inner().clone());
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

    let template: DistillationTemplate = serde_yaml::from_str(&template_content)
        .map_err(|e| format!("Invalid template YAML: {}", e))?;

    Ok(ValidationResult {
        valid: true,
        name: template.name,
        version: template.version,
        vectors_count: template.vectors.len(),
    })
}

#[tauri::command]
pub async fn validate_template_content(
    template_content: String,
) -> Result<ValidationResult, String> {
    let template: DistillationTemplate = serde_yaml::from_str(&template_content)
        .map_err(|e| format!("Invalid template YAML: {}", e))?;

    Ok(ValidationResult {
        valid: true,
        name: template.name,
        version: template.version,
        vectors_count: template.vectors.len(),
    })
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
    serde_yaml::to_string(&template).map_err(|e| format!("Failed to serialize: {}", e))
}

#[tauri::command]
pub async fn distill_and_write(
    db: State<'_, Surreal<Db>>,
    template_content: String,
    output_dir: String,
) -> Result<WriteResult, String> {
    let template: DistillationTemplate = serde_yaml::from_str(&template_content)
        .map_err(|e| format!("Invalid template YAML: {}", e))?;

    let engine = crate::template::engine::TemplateEngine::new(db.inner().clone());
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