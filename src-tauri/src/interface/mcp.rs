use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use crate::ontology::bindu::Bindu;
use crate::persistence::surreal_bridge::{
    connect_db, find_project_root, get_active_monads, get_all_edges, get_all_monads, Db, EdgeDto,
};
use crate::persistence::MandalaStore;
use crate::template::DistillationTemplate;
use crate::weaver::source_compiler::{distill_source, validate_source_coherence};
use crate::weaver::spatial_weaver::{AngularPolicy, DistillationRecipe, SeedSpec, Weaver};
use crate::weaver::threader::trace_full_chain;
use surrealdb::Surreal;

#[derive(Deserialize, Debug)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    #[serde(default)]
    pub jsonrpc: String,
    pub id: Option<JsonValue>,
    pub method: String,
    #[serde(default)]
    pub params: Option<JsonValue>,
}

#[derive(Serialize, Debug)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonValue>,
}

pub async fn run_mcp_server() -> anyhow::Result<()> {
    eprintln!("🌌 Mandala Weaver MCP Server v2.0 (SO(2) Spatial + Tripartite Graph) starting on stdio...");

    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut stdout = io::stdout();

    let mut line = String::new();

    while reader.read_line(&mut line)? > 0 {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            line.clear();
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                let err_resp = json!({
                    "jsonrpc": "2.0",
                    "id": JsonValue::Null,
                    "error": { "code": -32700, "message": format!("Parse error: {}", e) }
                });
                writeln!(stdout, "{}", serde_json::to_string(&err_resp)?)?;
                stdout.flush()?;
                line.clear();
                continue;
            }
        };

        if let Some(resp) = handle_mcp_request(req).await {
            writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
            stdout.flush()?;
        }

        line.clear();
    }

    Ok(())
}

pub async fn handle_mcp_request(req: JsonRpcRequest) -> Option<JsonRpcResponse> {
    match req.method.as_str() {
        "initialize" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": { "listChanged": false },
                    "resources": { "subscribe": false, "listChanged": false },
                    "prompts": { "listChanged": false }
                },
                "serverInfo": {
                    "name": "mandala-weaver-vcs",
                    "version": "0.2.0-so2",
                    "safetyMode": "non-destructive-sandbox"
                }
            })),
            error: None,
        }),
        "notifications/initialized" => None,
        "ping" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result: Some(json!({})),
            error: None,
        }),
        "tools/list" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result: Some(json!({
                "tools": get_mcp_tools_schema()
            })),
            error: None,
        }),
        "tools/call" => {
            let params = req.params.unwrap_or(json!({}));
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

            let result = execute_tool(tool_name, arguments).await;
            match result {
                Ok(content) => Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: Some(json!({
                        "content": [
                            {
                                "type": "text",
                                "text": content
                            }
                        ]
                    })),
                    error: None,
                }),
                Err(err_msg) => Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: Some(json!({
                        "isError": true,
                        "content": [
                            {
                                "type": "text",
                                "text": format!("Error executing tool '{}': {}", tool_name, err_msg)
                            }
                        ]
                    })),
                    error: None,
                }),
            }
        }
        "resources/list" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result: Some(json!({
                "resources": [
                    {
                        "uri": "mandala://topology",
                        "name": "Mandala Topology State",
                        "mimeType": "application/json",
                        "description": "Current active constellation, rings, and lineage edges"
                    },
                    {
                        "uri": "mandala://rings",
                        "name": "Mandala Rings Overview",
                        "mimeType": "application/json",
                        "description": "Per-ring monad counts, languages, and angular distribution"
                    },
                    {
                        "uri": "mandala://contracts",
                        "name": "Tripartite Graph Contracts & Ports",
                        "mimeType": "application/json",
                        "description": "Exposed contracts, ports, and outbound dependencies across monads"
                    },
                    {
                        "uri": "mandala://vectors",
                        "name": "SO(2) Angular Domains & Lineage Edges",
                        "mimeType": "application/json",
                        "description": "Angular sector breakdown (CORE, IO, DATA, UI, SERVICES) and evolution edges"
                    },
                    {
                        "uri": "mandala://recipe",
                        "name": "Active Distillation Recipe (.mandala/distill.yaml)",
                        "mimeType": "text/yaml",
                        "description": "Current YAML distillation manifest if present in .mandala/distill.yaml"
                    },
                    {
                        "uri": "mandala://languages",
                        "name": "Supported Polyglot AST Languages (39 Languages)",
                        "mimeType": "application/json",
                        "description": "Matrix of all 39 supported programming languages, file extensions, and AST extraction engines"
                    }
                ]
            })),
            error: None,
        }),
        "resources/read" => {
            let params = req.params.unwrap_or(json!({}));
            let uri = params.get("uri").and_then(|v| v.as_str()).unwrap_or("");
            let root = find_project_root();
            let db = match get_or_init_mcp_db(root.as_deref()).await {
                Ok(d) => d,
                Err(e) => {
                    return Some(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req.id,
                        result: None,
                        error: Some(json!({ "code": -32000, "message": format!("Database connection failed: {}", e) })),
                    })
                }
            };

            match read_mcp_resource(&db, uri, root.as_deref()).await {
                Ok((mime, payload)) => Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: Some(json!({
                        "contents": [
                            {
                                "uri": uri,
                                "mimeType": mime,
                                "text": payload
                            }
                        ]
                    })),
                    error: None,
                }),
                Err(msg) => Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: None,
                    error: Some(json!({ "code": -32002, "message": msg })),
                }),
            }
        }
        "prompts/list" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result: Some(json!({
                "prompts": [
                    {
                        "name": "architect_guidance",
                        "description": "Guidance on Circular Time, SO(2) Geodesic Distillation, Tripartite Graph (Monad-Contract-Port), and non-destructive operations.",
                        "arguments": []
                    },
                    {
                        "name": "spatial_distill_prompt",
                        "description": "Interactive template for designing an SO(2) Spatial Weaver DistillationRecipe.",
                        "arguments": [
                            {
                                "name": "seed_symbol",
                                "description": "Entry-point function or struct name to weave from",
                                "required": true
                            }
                        ]
                    }
                ]
            })),
            error: None,
        }),
        "prompts/get" => {
            let params = req.params.unwrap_or(json!({}));
            let prompt_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("architect_guidance");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));

            match prompt_name {
                "spatial_distill_prompt" => {
                    let seed = args.get("seed_symbol").and_then(|v| v.as_str()).unwrap_or("main");
                    Some(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req.id,
                        result: Some(json!({
                            "description": "SO(2) Spatial Weaver Recipe Designer",
                            "messages": [
                                {
                                    "role": "user",
                                    "content": {
                                        "type": "text",
                                        "text": format!(
                                            "Design an SO(2) Spatial Weaver DistillationRecipe starting from seed symbol `{}`.\nUse `mandala_search_monads` to locate its ring and angle, `mandala_resolve_contract` to inspect geodesic port candidates, and `mandala_distill_spatial` to assemble the target artifact non-destructively.",
                                            seed
                                        )
                                    }
                                }
                            ]
                        })),
                        error: None,
                    })
                }
                _ => Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: Some(json!({
                        "description": "Architect Guidance for Mandala Weaver VCS",
                        "messages": [
                            {
                                "role": "user",
                                "content": {
                                    "type": "text",
                                    "text": "Mandala Weaver operates on Circular Time, SO(2) Geodesic Space, and a Tripartite Ontological Graph:\n- Monads: Atomic logic units extracted across 22 languages with 1-based line numbers and BLAKE3 semantic/contract/implementation hashes.\n- Bindu (0,0): Immutable genesis of project purpose.\n- Rings (r): Concentric evolutionary layers expanding outward.\n- Vectors (theta in SO(2)): Logical domains (CORE 0°-72°, IO 72°-144°, DATA 144°-216°, UI 216°-288°, SERVICES 288°-360°).\n- Tripartite Graph: Monad -> implements -> Contract <- exposes_port <- Port <- requires_port <- Monad.\n- Non-Destructive Guarantee: All MCP tools operate strictly in-memory or inside `.mandala/` (e.g., `.mandala/distilled/`) and NEVER delete or overwrite project source files on disk."
                                }
                            }
                        ]
                    })),
                    error: None,
                }),
            }
        }
        _ => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result: None,
            error: Some(json!({
                "code": -32601,
                "message": format!("Method not found: {}", req.method)
            })),
        }),
    }
}

pub fn get_mcp_tools_schema() -> Vec<JsonValue> {
    vec![
        json!({
            "name": "mandala_telemetry",
            "description": "Scan Mandala topology and return ecosystem pulse, ring counts, language distribution, and tripartite graph stats.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "verbose": { "type": "boolean", "description": "Include detailed ring and language breakdown" }
                }
            }
        }),
        json!({
            "name": "mandala_bindu",
            "description": "Initialize Point Zero (0,0) (genesis) for a project in SurrealDB.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_name": { "type": "string", "description": "Project name" }
                },
                "required": ["project_name"]
            }
        }),
        json!({
            "name": "mandala_crystallize",
            "description": "Radial commit: Parse a source file with the 22-language polyglot AST extractor and expand outward into a new ring (non-destructive to disk).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Source file path to crystallize" },
                    "message": { "type": "string", "description": "Optional intent message" }
                },
                "required": ["file_path"]
            }
        }),
        json!({
            "name": "mandala_unweave",
            "description": "InnerKnow Unweaver: Shatter a file or directory into atomic monads, populate the tripartite graph (ports & contracts), and generate .mandala/distill.yaml without modifying any source files.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Directory or file path to unweave (defaults to current project root)" }
                }
            }
        }),
        json!({
            "name": "mandala_distill_spatial",
            "description": "Execute the SO(2) Spatial Weaver from seed symbols using geodesic angular resolution on the tripartite graph. Non-destructive: returns assembled code in JSON (and optionally writes to .mandala/distilled/ sandbox).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "seeds": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Entry-point symbol names (e.g. ['main', 'procesar_pago'])"
                    },
                    "target_angle_deg": {
                        "type": "number",
                        "description": "Target angular sector in degrees (0-360) for geodesic contract resolution"
                    },
                    "max_ring": {
                        "type": "integer",
                        "description": "Maximum concentric ring horizon to search"
                    },
                    "policy": {
                        "type": "string",
                        "enum": ["StrictGeodesic", "SectorBounded", "NearestFallback"],
                        "description": "Angular resolution policy (default: NearestFallback)"
                    },
                    "tolerance_deg": {
                        "type": "number",
                        "description": "Tolerance in degrees when policy is SectorBounded (default: 45)"
                    },
                    "save_to_sandbox": {
                        "type": "boolean",
                        "description": "If true, writes output strictly to .mandala/distilled/ without touching project files"
                    }
                },
                "required": ["seeds"]
            }
        }),
        json!({
            "name": "mandala_resolve_contract",
            "description": "Resolve the optimal monad implementation for a port symbol using SO(2) geodesic distance at a target angle and ring horizon.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "symbol_name": { "type": "string", "description": "Port or function symbol name to resolve" },
                    "target_angle_deg": { "type": "number", "description": "Target angle in degrees (0-360)" },
                    "max_ring": { "type": "integer", "description": "Maximum ring horizon (default: 10)" }
                },
                "required": ["symbol_name"]
            }
        }),
        json!({
            "name": "mandala_search_monads",
            "description": "Search monads across the Mandala by name, kind, language, semantic hash, or code content.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query string" },
                    "ring": { "type": "integer", "description": "Optional ring filter" },
                    "kind": { "type": "string", "description": "Optional MonadKind filter (Function, Struct, Trait, Enum, Impl, Module, Constant)" },
                    "language": { "type": "string", "description": "Optional language filter (rust, typescript, python, go, etc.)" },
                    "limit": { "type": "integer", "description": "Max results to return (default: 30)" }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "mandala_audit_topology",
            "description": "Validate semantic coherence, duplicate symbols, and tripartite contract/port integrity across active monads.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "ring": { "type": "integer", "description": "Optional specific ring to audit (audits all active rings if omitted)" }
                }
            }
        }),
        json!({
            "name": "mandala_ring_diff",
            "description": "Compare two concentric rings and compute added (+), modified (~, same symbol with new hash), removed (-), and unchanged (=) monads.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "base_ring": { "type": "integer", "description": "Base ring level (e.g. 1)" },
                    "target_ring": { "type": "integer", "description": "Target ring level to compare against (e.g. 2)" }
                },
                "required": ["base_ring", "target_ring"]
            }
        }),
        json!({
            "name": "mandala_vectors",
            "description": "Inspect SO(2) angular domain sectors (CORE, IO, DATA, UI, SERVICES) and evolutionary/call lineage edges.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "sector": { "type": "string", "description": "Optional sector filter (CORE, IO, DATA, UI, SERVICES)" }
                }
            }
        }),
        json!({
            "name": "mandala_contract_ring",
            "description": "Non-destructively contract (archive in SurrealDB) the outermost active ring or revert to a target ring level without deleting any disk files.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "target_ring": { "type": "integer", "description": "Optional ring level to revert to (contracts only the outermost ring if omitted)" }
                }
            }
        }),
        json!({
            "name": "mandala_distill",
            "description": "Distill and compile source code from monad IDs or a YAML template/recipe manifest (non-destructive, returns code in response).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "monad_ids": { "type": "array", "items": { "type": "string" }, "description": "List of monad IDs to weave" },
                    "template_content": { "type": "string", "description": "Optional YAML template or SO(2) recipe content" },
                    "save_to_sandbox": { "type": "boolean", "description": "If true, saves compiled output strictly inside .mandala/distilled/" }
                }
            }
        }),
        json!({
            "name": "mandala_focus",
            "description": "Inspect full details of a monad (AST kind, 1-based line numbers, contract hash, outbound calls, polar coordinates, source code).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "monad_name_or_id": { "type": "string", "description": "Monad name, ID, or semantic hash prefix" }
                },
                "required": ["monad_name_or_id"]
            }
        }),
        json!({
            "name": "mandala_lineage",
            "description": "Query the evolutionary lineage/ancestor chain of a monad across inner rings.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "monad_id": { "type": "string", "description": "Monad ID or name" }
                },
                "required": ["monad_id"]
            }
        }),
        json!({
            "name": "mandala_spectrum",
            "description": "Analyze chromatic shift (BLAKE3 HSL variance) of a monad across rings.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "monad_name": { "type": "string", "description": "Monad name" }
                },
                "required": ["monad_name"]
            }
        }),
        json!({
            "name": "mandala_scan_project",
            "description": "Scan a directory with the 22-language polyglot AST extractor and index monads, contracts, ports, and lineage edges into SurrealDB (100% read-only on project files).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Folder path to scan" },
                    "force_rescan": { "type": "boolean", "description": "If true, refreshes existing SurrealDB monads with newly extracted AST nodes" }
                },
                "required": ["path"]
            }
        }),
        json!({
            "name": "mandala_git_import",
            "description": "Import git commit history as concentric evolutionary rings into the Mandala.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "repo_path": { "type": "string", "description": "Git repository root path" }
                },
                "required": ["repo_path"]
            }
        }),
        json!({
            "name": "mandala_validate_template",
            "description": "Validate a YAML distillation template or SO(2) DistillationRecipe manifest.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "template_yaml": { "type": "string", "description": "YAML content" }
                },
                "required": ["template_yaml"]
            }
        }),
        json!({
            "name": "mandala_supported_languages",
            "description": "List all 39 supported programming languages, file extensions, and Tree-sitter / polyglot AST extraction engines.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        json!({
            "name": "mandala_extract_ast",
            "description": "Extract atomic Monads, 1-based line numbers, Vogel Golden Angle coordinates, BLAKE3 hashes, and outbound calls from source code or a file path across 39 supported languages (100% read-only in-memory).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "source_code": { "type": "string", "description": "Source code string to parse (optional if file_path is provided)" },
                    "language": { "type": "string", "description": "Language or extension (e.g. 'rust', 'tsx', 'python', 'sol', 'scala', 'zig', 'tf', 'astro')" },
                    "file_path": { "type": "string", "description": "Optional file path to read and extract monads from" },
                    "ring": { "type": "integer", "description": "Target ring level for polar coordinates (default: 1)" }
                }
            }
        }),
    ]
}

pub async fn read_mcp_resource(
    db: &Surreal<Db>,
    uri: &str,
    root: Option<&std::path::Path>,
) -> Result<(String, String), String> {
    match uri {
        "mandala://topology" => {
            let all_monads = get_active_monads(db).await.unwrap_or_default();
            let edges = get_all_edges(db).await.unwrap_or_default();
            let max_ring = all_monads.iter().map(|m| m.ring).max().unwrap_or(0);
            let payload = serde_json::to_string_pretty(&json!({
                "total_monads": all_monads.len(),
                "max_ring": max_ring,
                "edges_count": edges.len(),
                "monads": all_monads,
                "edges": edges
            }))
            .unwrap_or_default();
            Ok(("application/json".to_string(), payload))
        }
        "mandala://rings" => {
            let all_monads = get_active_monads(db).await.unwrap_or_default();
            let mut rings: std::collections::BTreeMap<u32, Vec<JsonValue>> =
                std::collections::BTreeMap::new();
            for m in &all_monads {
                rings.entry(m.ring).or_default().push(json!({
                    "id": m.id,
                    "name": m.name,
                    "kind": m.kind,
                    "language": m.language,
                    "theta": m.coord.theta,
                    "lines": [m.line_start, m.line_end]
                }));
            }
            let payload = serde_json::to_string_pretty(&json!({
                "ring_count": rings.len(),
                "rings": rings
            }))
            .unwrap_or_default();
            Ok(("application/json".to_string(), payload))
        }
        "mandala://contracts" => {
            let all_monads = get_active_monads(db).await.unwrap_or_default();
            let contracts: Vec<JsonValue> = all_monads
                .iter()
                .map(|m| {
                    json!({
                        "monad_id": m.id,
                        "symbol": m.name,
                        "ring": m.ring,
                        "theta": m.coord.theta,
                        "contract_hash": m.contract_hash,
                        "implementation_hash": m.implementation_hash,
                        "outbound_calls": m.outbound_calls,
                    })
                })
                .collect();
            let payload = serde_json::to_string_pretty(&json!({
                "total_contracts": contracts.len(),
                "contracts": contracts
            }))
            .unwrap_or_default();
            Ok(("application/json".to_string(), payload))
        }
        "mandala://vectors" => {
            let summary = build_vectors_summary(db, None).await?;
            Ok(("application/json".to_string(), summary))
        }
        "mandala://recipe" => {
            let base = root
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
            let recipe_path = base.join(".mandala").join("distill.yaml");
            if recipe_path.exists() {
                let text = std::fs::read_to_string(&recipe_path).map_err(|e| e.to_string())?;
                Ok(("text/yaml".to_string(), text))
            } else {
                Ok((
                    "text/yaml".to_string(),
                    "# No .mandala/distill.yaml recipe generated yet.\n".to_string(),
                ))
            }
        }
        "mandala://languages" => {
            let langs = crate::language::Language::all_supported_info();
            let payload = serde_json::to_string_pretty(&json!({
                "total_supported_languages": langs.len(),
                "languages": langs
            }))
            .unwrap_or_default();
            Ok(("application/json".to_string(), payload))
        }
        _ => Err(format!("Resource not found: {}", uri)),
    }
}

static MCP_DB_CACHE: std::sync::OnceLock<tokio::sync::Mutex<Option<Surreal<Db>>>> =
    std::sync::OnceLock::new();

/// Obtains a reusable SurrealDB handle for the MCP server.
/// If `.mandala/db` is locked by a running Tauri desktop app, automatically falls back to an
/// in-memory `Mem` database and hydrates it by scanning the project directory on-the-fly.
async fn get_or_init_mcp_db(root: Option<&std::path::Path>) -> Result<Surreal<Db>, String> {
    let mutex = MCP_DB_CACHE.get_or_init(|| tokio::sync::Mutex::new(None));
    let mut guard = mutex.lock().await;
    if let Some(ref existing) = *guard {
        return Ok(existing.clone());
    }

    let db = match connect_db(root).await {
        Ok(d) => d,
        Err(_lock_err) => {
            let mem_db = crate::persistence::surreal_bridge::connect_embedded()
                .await
                .map_err(|e| e.to_string())?;
            if let Some(proj_root) = root {
                let path_buf = proj_root.to_path_buf();
                let _ = crate::synarchy::ProjectScanner::scan_and_index_with_db_mode(
                    &mem_db,
                    &path_buf,
                    Some("mandala"),
                )
                .await;
            }
            mem_db
        }
    };

    *guard = Some(db.clone());
    Ok(db)
}

pub async fn execute_tool(name: &str, args: JsonValue) -> Result<String, String> {
    let root = find_project_root();
    let db = get_or_init_mcp_db(root.as_deref()).await?;
    execute_tool_with_db(&db, name, args, root).await
}

/// Core tool dispatcher accepting an explicit SurrealDB handle so it can be unit-tested
/// in-memory without touching any disk state, and guarantees non-destructive sandbox execution.
pub async fn execute_tool_with_db(
    db: &Surreal<Db>,
    name: &str,
    args: JsonValue,
    root: Option<PathBuf>,
) -> Result<String, String> {
    match name {
        "mandala_telemetry" => {
            let all = get_all_monads(db).await.map_err(|e| e.to_string())?;
            let active_count = all.iter().filter(|m| !m.is_archived).count();
            let archived_count = all.len().saturating_sub(active_count);
            let mut ring_map: std::collections::BTreeMap<u32, usize> =
                std::collections::BTreeMap::new();
            let mut lang_map: std::collections::BTreeMap<String, usize> =
                std::collections::BTreeMap::new();
            let mut contracts_count = 0usize;

            for m in all.iter().filter(|m| !m.is_archived) {
                *ring_map.entry(m.ring).or_default() += 1;
                let lang = if m.language.is_empty() {
                    "unknown".to_string()
                } else {
                    m.language.clone()
                };
                *lang_map.entry(lang).or_default() += 1;
                if !m.contract_hash.is_empty() {
                    contracts_count += 1;
                }
            }

            let max_ring = all
                .iter()
                .filter(|m| !m.is_archived)
                .map(|m| m.ring)
                .max()
                .unwrap_or(0);
            let edges = get_all_edges(db).await.unwrap_or_default();

            let info = json!({
                "total_monads": active_count,
                "archived_monads": archived_count,
                "max_ring": max_ring,
                "contracts_indexed": contracts_count,
                "lineage_edges": edges.len(),
                "ring_breakdown": ring_map,
                "language_breakdown": lang_map,
                "safety_mode": "non-destructive",
                "status": "Synchronized"
            });
            Ok(serde_json::to_string_pretty(&info).unwrap())
        }
        "mandala_bindu" => {
            let project_name = args
                .get("project_name")
                .and_then(|v| v.as_str())
                .unwrap_or("mandala-project");
            let bindu = Bindu::genesis(project_name);
            let val = serde_json::to_value(&bindu).map_err(|e| e.to_string())?;
            let _: Option<JsonValue> = db
                .upsert(("bindu", "genesis"))
                .content(val)
                .await
                .map_err(|e| e.to_string())?;
            Ok(format!(
                "Point Zero instantiated: Project '{}' initialized at timestamp {}.",
                project_name, bindu.timestamp
            ))
        }
        "mandala_crystallize" => {
            let file_path = args
                .get("file_path")
                .and_then(|v| v.as_str())
                .ok_or("Missing file_path")?;
            let new_ring = crate::weaver::expand_from_source(db, file_path)
                .await
                .map_err(|e| e.to_string())?;
            Ok(format!(
                "File '{}' crystallized into Ring {} (tripartite ports & contracts updated).",
                file_path, new_ring
            ))
        }
        "mandala_unweave" => {
            let target_path = args
                .get("path")
                .and_then(|v| v.as_str())
                .map(PathBuf::from)
                .or(root)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

            if !target_path.exists() {
                return Err(format!("Path does not exist: {}", target_path.display()));
            }

            let out_recipe = target_path.join(".mandala").join("distill.yaml");
            let summary = crate::weaver::unweave::UnweaveEngine::unweave_with_db(
                db,
                &target_path,
                Some(&out_recipe),
            )
            .await
            .map_err(|e| e.to_string())?;

            let info = json!({
                "status": "Unweaved (Non-Destructive)",
                "project_name": summary.project_name,
                "monads_extracted": summary.monads_extracted,
                "max_ring": summary.max_ring,
                "domains": summary.domains,
                "recipe_path": summary.recipe_path,
                "top_contracts": summary.top_contracts,
            });
            Ok(serde_json::to_string_pretty(&info).unwrap())
        }
        "mandala_distill_spatial" => {
            let seeds_arr = args
                .get("seeds")
                .and_then(|v| v.as_array())
                .ok_or("Missing 'seeds' array")?;
            let seed_names: Vec<String> = seeds_arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            if seed_names.is_empty() {
                return Err("At least one seed symbol name is required".to_string());
            }

            let target_angle_deg = args
                .get("target_angle_deg")
                .and_then(|v| v.as_f64())
                .unwrap_or(45.0);
            let max_ring = args
                .get("max_ring")
                .and_then(|v| v.as_u64())
                .map(|r| r as u32)
                .unwrap_or(10);
            let policy_str = args
                .get("policy")
                .and_then(|v| v.as_str())
                .unwrap_or("GlobalTarget");

            let policy = match policy_str {
                "CallerLocal" => AngularPolicy::CallerLocal,
                _ => AngularPolicy::GlobalTarget,
            };

            let seeds: Vec<SeedSpec> = seed_names
                .into_iter()
                .map(|sym| SeedSpec { symbol: sym })
                .collect();

            let recipe = DistillationRecipe {
                name: format!(
                    "mcp_spatial_{}",
                    seeds.iter().map(|s| s.symbol.as_str()).collect::<Vec<_>>().join("_")
                ),
                target_angle_rad: target_angle_deg.to_radians(),
                max_ring,
                angular_policy: policy,
                ring_penalty_rad: 0.1,
                prelude: String::new(),
                seeds,
                pins: std::collections::BTreeMap::new(),
                external_symbols: std::collections::BTreeSet::new(),
            };

            let store = MandalaStore::from_existing_db(db.clone());
            let weaver = Weaver::new(&store);
            let artifact = weaver.distill(&recipe).await.map_err(|e| e.to_string())?;

            let mut sandbox_file: Option<String> = None;
            if args
                .get("save_to_sandbox")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                let base = root.unwrap_or_else(|| {
                    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                });
                let sandbox_dir = base.join(".mandala").join("distilled");
                let _ = std::fs::create_dir_all(&sandbox_dir);
                let safe_path = sandbox_dir.join(format!("{}.rs", recipe.name));
                std::fs::write(&safe_path, &artifact.assembled_source)
                    .map_err(|e| e.to_string())?;
                sandbox_file = Some(safe_path.to_string_lossy().to_string());
            }

            let resp = json!({
                "recipe": artifact.recipe_name,
                "selected_monads_count": artifact.traces.len(),
                "traces": artifact.traces,
                "bridge_traces_count": artifact.bridge_traces.len(),
                "bridge_traces": artifact.bridge_traces,
                "sandbox_path": sandbox_file,
                "assembled_source": artifact.assembled_source,
            });
            Ok(serde_json::to_string_pretty(&resp).unwrap())
        }
        "mandala_resolve_contract" => {
            let symbol_name = args
                .get("symbol_name")
                .and_then(|v| v.as_str())
                .ok_or("Missing symbol_name")?;
            let target_angle_deg = args
                .get("target_angle_deg")
                .and_then(|v| v.as_f64())
                .unwrap_or(45.0);
            let max_ring = args
                .get("max_ring")
                .and_then(|v| v.as_u64())
                .map(|r| r as u32)
                .unwrap_or(10);

            let store = MandalaStore::from_existing_db(db.clone());
            let weaver = Weaver::new(&store);
            let recipe = DistillationRecipe {
                name: "mcp_resolve".to_string(),
                target_angle_rad: target_angle_deg.to_radians(),
                max_ring,
                angular_policy: AngularPolicy::GlobalTarget,
                ring_penalty_rad: 0.1,
                prelude: String::new(),
                seeds: vec![SeedSpec { symbol: symbol_name.to_string() }],
                pins: std::collections::BTreeMap::new(),
                external_symbols: std::collections::BTreeSet::new(),
            };
            let reference_polar = crate::geometry::PolarCoordinate::new(0, 0.0, target_angle_deg.to_radians());
            match weaver.resolve_single_symbol(symbol_name, "<mcp_resolve>", &reference_polar, &recipe, None).await {
                Ok((persisted, trace)) => {
                    let info = json!({
                        "resolved": true,
                        "symbol_name": symbol_name,
                        "monad_id": persisted.id,
                        "ring": persisted.ring,
                        "angle_deg": trace.selected_angle_rad.to_degrees(),
                        "geodesic_distance_rad": trace.angular_distance_rad,
                        "geodesic_distance_deg": trace.angular_distance_rad.to_degrees(),
                        "contract_hash": trace.contract_hash,
                        "implementation_hash": trace.implementation_hash,
                        "alternatives_considered": trace.alternatives_considered,
                    });
                    Ok(serde_json::to_string_pretty(&info).unwrap())
                }
                Err(e) => Ok(serde_json::to_string_pretty(&json!({
                    "resolved": false,
                    "symbol_name": symbol_name,
                    "message": format!("No candidate resolved: {}", e)
                })).unwrap()),
            }
        }
        "mandala_search_monads" => {
            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_lowercase();
            let ring_filter = args
                .get("ring")
                .and_then(|v| v.as_u64())
                .map(|r| r as u32);
            let kind_filter = args
                .get("kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_lowercase());
            let lang_filter = args
                .get("language")
                .and_then(|v| v.as_str())
                .map(|s| s.to_lowercase());
            let limit = args
                .get("limit")
                .and_then(|v| v.as_u64())
                .map(|l| l as usize)
                .unwrap_or(30);

            let all = get_active_monads(db).await.map_err(|e| e.to_string())?;
            let results: Vec<JsonValue> = all
                .into_iter()
                .filter(|m| {
                    if let Some(r) = ring_filter {
                        if m.ring != r {
                            return false;
                        }
                    }
                    if let Some(ref k) = kind_filter {
                        if format!("{:?}", m.kind).to_lowercase() != *k {
                            return false;
                        }
                    }
                    if let Some(ref l) = lang_filter {
                        if !m.language.to_lowercase().contains(l) {
                            return false;
                        }
                    }
                    if query.is_empty() {
                        return true;
                    }
                    m.name.to_lowercase().contains(&query)
                        || m.id.to_lowercase().contains(&query)
                        || m.semantic_hash.to_lowercase().contains(&query)
                        || m.language.to_lowercase().contains(&query)
                        || m.content.to_lowercase().contains(&query)
                })
                .take(limit)
                .map(|m| {
                    json!({
                        "id": m.id,
                        "name": m.name,
                        "kind": m.kind,
                        "ring": m.ring,
                        "theta": m.coord.theta,
                        "language": m.language,
                        "lines": [m.line_start, m.line_end],
                        "contract_hash": m.contract_hash,
                        "semantic_hash": m.semantic_hash,
                    })
                })
                .collect();

            Ok(serde_json::to_string_pretty(&json!({
                "count": results.len(),
                "results": results
            }))
            .unwrap())
        }
        "mandala_audit_topology" => {
            let ring_filter = args
                .get("ring")
                .and_then(|v| v.as_u64())
                .map(|r| r as u32);
            let all = get_active_monads(db).await.map_err(|e| e.to_string())?;
            let target_monads: Vec<_> = all
                .iter()
                .filter(|m| ring_filter.map(|r| m.ring == r).unwrap_or(true))
                .cloned()
                .collect();

            let coherence_warnings: Vec<crate::weaver::source_compiler::IncoherenceReport> =
                match validate_source_coherence(&target_monads) {
                    Ok(()) => Vec::new(),
                    Err(e) => e,
                };
            let known_symbols: std::collections::HashSet<String> =
                all.iter().map(|m| m.name.clone()).collect();

            let mut unresolved_calls = Vec::new();
            for m in &target_monads {
                for call in &m.outbound_calls {
                    if !known_symbols.contains(call) {
                        unresolved_calls.push(json!({
                            "caller": m.name,
                            "ring": m.ring,
                            "external_or_unresolved_symbol": call
                        }));
                    }
                }
            }

            let report = json!({
                "monads_audited": target_monads.len(),
                "ring_filter": ring_filter,
                "coherence_warnings_count": coherence_warnings.len(),
                "coherence_warnings": coherence_warnings,
                "external_or_unresolved_calls_count": unresolved_calls.len(),
                "external_or_unresolved_calls_sample": unresolved_calls.into_iter().take(25).collect::<Vec<_>>(),
                "healthy": coherence_warnings.is_empty()
            });
            Ok(serde_json::to_string_pretty(&report).unwrap())
        }
        "mandala_ring_diff" => {
            let base_ring = args
                .get("base_ring")
                .and_then(|v| v.as_u64())
                .ok_or("Missing base_ring")? as u32;
            let target_ring = args
                .get("target_ring")
                .and_then(|v| v.as_u64())
                .ok_or("Missing target_ring")? as u32;

            let all = get_active_monads(db).await.map_err(|e| e.to_string())?;
            let base_monads: Vec<_> = all.iter().filter(|m| m.ring == base_ring).collect();
            let target_monads: Vec<_> = all.iter().filter(|m| m.ring == target_ring).collect();

            let mut base_by_name = std::collections::HashMap::new();
            for m in &base_monads {
                base_by_name.insert(m.name.clone(), *m);
            }
            let mut target_names = std::collections::HashSet::new();

            let mut added = Vec::new();
            let mut modified = Vec::new();
            let mut unchanged = Vec::new();

            for m in &target_monads {
                target_names.insert(m.name.clone());
                if let Some(prev) = base_by_name.get(&m.name) {
                    if prev.semantic_hash == m.semantic_hash {
                        unchanged.push(m.name.clone());
                    } else {
                        modified.push(json!({
                            "name": m.name,
                            "kind": m.kind,
                            "base_hash": prev.semantic_hash,
                            "target_hash": m.semantic_hash
                        }));
                    }
                } else {
                    added.push(json!({
                        "name": m.name,
                        "kind": m.kind,
                        "language": m.language
                    }));
                }
            }

            let removed: Vec<_> = base_monads
                .iter()
                .filter(|m| !target_names.contains(&m.name))
                .map(|m| json!({ "name": m.name, "kind": m.kind }))
                .collect();

            let diff = json!({
                "base_ring": base_ring,
                "target_ring": target_ring,
                "base_count": base_monads.len(),
                "target_count": target_monads.len(),
                "added_count": added.len(),
                "modified_count": modified.len(),
                "removed_count": removed.len(),
                "unchanged_count": unchanged.len(),
                "added": added,
                "modified": modified,
                "removed": removed,
            });
            Ok(serde_json::to_string_pretty(&diff).unwrap())
        }
        "mandala_vectors" => {
            let sector_filter = args.get("sector").and_then(|v| v.as_str());
            build_vectors_summary(db, sector_filter).await
        }
        "mandala_contract_ring" => {
            if let Some(target_ring) = args.get("target_ring").and_then(|v| v.as_u64()) {
                let archived = crate::weaver::contract::revert_to_ring(db, target_ring as u32)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(serde_json::to_string_pretty(&json!({
                    "action": "revert_to_ring",
                    "target_ring": target_ring,
                    "monads_archived_in_db": archived,
                    "disk_files_modified": 0
                }))
                .unwrap())
            } else {
                let new_max = crate::weaver::contract::contract_ring(db)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(serde_json::to_string_pretty(&json!({
                    "action": "contract_outer_ring",
                    "new_max_ring": new_max,
                    "disk_files_modified": 0
                }))
                .unwrap())
            }
        }
        "mandala_distill" => {
            let source = if let Some(template_yaml) =
                args.get("template_content").and_then(|v| v.as_str())
            {
                if let Ok(recipe) = serde_yml::from_str::<DistillationRecipe>(template_yaml) {
                    if !recipe.seeds.is_empty() {
                        let store = MandalaStore::from_existing_db(db.clone());
                        let weaver = Weaver::new(&store);
                        if let Ok(artifact) = weaver.distill(&recipe).await {
                            artifact.assembled_source
                        } else {
                            return Err("Spatial Weaver failed to resolve recipe seeds.".to_string());
                        }
                    } else {
                        return Err("DistillationRecipe has no seeds.".to_string());
                    }
                } else {
                    let template: DistillationTemplate = serde_yml::from_str(template_yaml)
                        .map_err(|e| format!("YAML error: {}", e))?;
                    let engine = crate::template::engine::TemplateEngine::new(db.clone());
                    let monads = engine.resolve(&template).await.map_err(|e| e.to_string())?;
                    if monads.is_empty() {
                        return Err("No monads matched the template criteria.".to_string());
                    }
                    crate::weaver::source_compiler::distill_source_with_adapters(
                        &monads,
                        &template.adapters,
                    )
                }
            } else if let Some(ids_val) = args.get("monad_ids").and_then(|v| v.as_array()) {
                let ids: Vec<String> = ids_val
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                let all = get_all_monads(db).await.map_err(|e| e.to_string())?;
                let selected: Vec<_> = all.into_iter().filter(|m| ids.contains(&m.id)).collect();
                if selected.is_empty() {
                    return Err("No monads matched the IDs provided.".to_string());
                }
                distill_source(&selected)
            } else {
                return Err("Please provide either monad_ids or template_content".to_string());
            };

            if args
                .get("save_to_sandbox")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                let base = root.unwrap_or_else(|| {
                    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                });
                let sandbox_dir = base.join(".mandala").join("distilled");
                let _ = std::fs::create_dir_all(&sandbox_dir);
                let safe_path = sandbox_dir.join("distilled_output.txt");
                std::fs::write(&safe_path, &source).map_err(|e| e.to_string())?;
            }

            Ok(source)
        }
        "mandala_focus" => {
            let target = args
                .get("monad_name_or_id")
                .and_then(|v| v.as_str())
                .ok_or("Missing monad_name_or_id")?;
            let all = get_all_monads(db).await.map_err(|e| e.to_string())?;
            let found = all.into_iter().find(|m| {
                m.id == target
                    || m.name == target
                    || m.semantic_hash.starts_with(target)
                    || m.contract_hash.starts_with(target)
            });
            match found {
                Some(m) => Ok(serde_json::to_string_pretty(&m).unwrap()),
                None => Err(format!("Monad '{}' not found in Akashic Record", target)),
            }
        }
        "mandala_lineage" => {
            let id_or_name = args
                .get("monad_id")
                .and_then(|v| v.as_str())
                .ok_or("Missing monad_id")?;
            let chain = trace_full_chain(db, id_or_name)
                .await
                .map_err(|e| e.to_string())?;
            if !chain.is_empty() {
                return Ok(serde_json::to_string_pretty(&chain).unwrap());
            }
            // Fallback to cross-ring symbol evolution if no explicit `evolves_to` edge exists
            let all = get_all_monads(db).await.map_err(|e| e.to_string())?;
            let target_name = all
                .iter()
                .find(|m| m.id == id_or_name || m.name == id_or_name)
                .map(|m| m.name.clone())
                .unwrap_or_else(|| id_or_name.to_string());
            let mut history: Vec<_> = all
                .into_iter()
                .filter(|m| m.name == target_name)
                .collect();
            history.sort_by_key(|m| m.ring);
            Ok(serde_json::to_string_pretty(&history).unwrap())
        }
        "mandala_spectrum" => {
            let name = args
                .get("monad_name")
                .and_then(|v| v.as_str())
                .ok_or("Missing monad_name")?;
            let all = get_all_monads(db).await.map_err(|e| e.to_string())?;
            let matching: Vec<_> = all.into_iter().filter(|m| m.name == name).collect();
            if matching.is_empty() {
                return Err(format!("No monads with name '{}' found", name));
            }
            let info = json!({
                "monad_name": name,
                "versions_count": matching.len(),
                "evolution": matching.iter().map(|m| json!({
                    "ring": m.ring,
                    "hash": m.semantic_hash,
                    "contract_hash": m.contract_hash,
                    "kind": m.kind,
                    "theta": m.coord.theta,
                    "lines": [m.line_start, m.line_end],
                })).collect::<Vec<_>>()
            });
            Ok(serde_json::to_string_pretty(&info).unwrap())
        }
        "mandala_scan_project" => {
            let path_str = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or("Missing path")?;
            let force_rescan = args
                .get("force_rescan")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            let path = PathBuf::from(path_str);
            if !path.exists() {
                return Err(format!("Path does not exist: {}", path_str));
            }
            let mode = if force_rescan { Some("rescan") } else { None };
            let entry = crate::synarchy::ProjectScanner::scan_and_index_with_db_mode(db, &path, mode)
                .await
                .map_err(|e| e.to_string())?;
            Ok(serde_json::to_string_pretty(&entry).unwrap())
        }
        "mandala_git_import" => {
            let path_str = args
                .get("repo_path")
                .and_then(|v| v.as_str())
                .ok_or("Missing repo_path")?;
            let path = PathBuf::from(path_str);
            let rings_imported = crate::collaboration::import_git_history(db, &path)
                .await
                .map_err(|e| e.to_string())?;
            Ok(format!(
                "Imported git history from '{}': created {} evolutionary rings.",
                path_str, rings_imported
            ))
        }
        "mandala_validate_template" => {
            let yaml = args
                .get("template_yaml")
                .and_then(|v| v.as_str())
                .ok_or("Missing template_yaml")?;
            if let Ok(template) = serde_yml::from_str::<DistillationTemplate>(yaml) {
                if !template.vectors.is_empty() {
                    let info = json!({
                        "valid": true,
                        "schema": "DistillationTemplate",
                        "name": template.name,
                        "version": template.version,
                        "vectors_count": template.vectors.len(),
                    });
                    return Ok(serde_json::to_string_pretty(&info).unwrap());
                }
            }
            let recipe: DistillationRecipe =
                serde_yml::from_str(yaml).map_err(|e| format!("Invalid YAML: {}", e))?;
            let info = json!({
                "valid": true,
                "schema": "DistillationRecipe (SO(2))",
                "name": recipe.name,
                "version": "1.0.0-so2",
                "seeds_count": recipe.seeds.len(),
            });
            Ok(serde_json::to_string_pretty(&info).unwrap())
        }
        "mandala_supported_languages" => {
            let langs = crate::language::Language::all_supported_info();
            let info = json!({
                "total_supported_languages": langs.len(),
                "extensions_count": crate::language::Language::code_extensions().len(),
                "languages": langs
            });
            Ok(serde_json::to_string_pretty(&info).unwrap())
        }
        "mandala_extract_ast" => {
            let ring = args
                .get("ring")
                .and_then(|v| v.as_u64())
                .map(|r| r as u32)
                .unwrap_or(1);

            let (source, lang_str) = if let Some(fp) = args.get("file_path").and_then(|v| v.as_str()) {
                let path = PathBuf::from(fp);
                let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| {
                        crate::language::detect_language(&path)
                            .tree_sitter_name()
                            .to_string()
                    });
                (content, ext)
            } else {
                let src = args
                    .get("source_code")
                    .and_then(|v| v.as_str())
                    .ok_or("Provide either 'source_code' or 'file_path'")?
                    .to_string();
                let lang = args
                    .get("language")
                    .and_then(|v| v.as_str())
                    .unwrap_or("rust")
                    .to_string();
                (src, lang)
            };

            let monads = crate::weaver::ast_extractor::extract_raw_monads_lang(&source, ring, &lang_str);
            let resp = json!({
                "language": lang_str,
                "ring": ring,
                "monads_extracted": monads.len(),
                "monads": monads
            });
            Ok(serde_json::to_string_pretty(&resp).unwrap())
        }
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

async fn build_vectors_summary(
    db: &Surreal<Db>,
    sector_filter: Option<&str>,
) -> Result<String, String> {
    let all = get_active_monads(db).await.map_err(|e| e.to_string())?;
    let mut edges = get_all_edges(db).await.unwrap_or_default();

    if edges.is_empty() && !all.is_empty() {
        let mut seen = std::collections::HashSet::new();
        let mut sorted = all.clone();
        sorted.sort_by_key(|m| m.ring);
        let mut by_name: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        for m in &sorted {
            if let Some(prev_id) = by_name.get(&m.name) {
                if prev_id != &m.id && seen.insert((prev_id.clone(), m.id.clone())) {
                    edges.push(EdgeDto {
                        parent_id: prev_id.clone(),
                        child_id: m.id.clone(),
                    });
                }
            }
            for call in &m.outbound_calls {
                if let Some(target_id) = by_name.get(call) {
                    if target_id != &m.id && seen.insert((target_id.clone(), m.id.clone())) {
                        edges.push(EdgeDto {
                            parent_id: target_id.clone(),
                            child_id: m.id.clone(),
                        });
                    }
                }
            }
            by_name.insert(m.name.clone(), m.id.clone());
        }
    }

    let classify_sector = |theta: f64| -> &'static str {
        let norm = ((theta % 360.0) + 360.0) % 360.0;
        if norm < 72.0 {
            "CORE"
        } else if norm < 144.0 {
            "IO"
        } else if norm < 216.0 {
            "DATA"
        } else if norm < 288.0 {
            "UI"
        } else {
            "SERVICES"
        }
    };

    let mut sectors: std::collections::BTreeMap<&str, Vec<JsonValue>> =
        std::collections::BTreeMap::new();
    for m in &all {
        let sec = classify_sector(m.coord.theta);
        if let Some(filter) = sector_filter {
            if !sec.eq_ignore_ascii_case(filter) {
                continue;
            }
        }
        sectors.entry(sec).or_default().push(json!({
            "id": m.id,
            "name": m.name,
            "ring": m.ring,
            "theta": m.coord.theta,
            "kind": m.kind,
            "language": m.language
        }));
    }

    let sector_counts: std::collections::BTreeMap<&str, usize> =
        sectors.iter().map(|(k, v)| (*k, v.len())).collect();

    Ok(serde_json::to_string_pretty(&json!({
        "total_active_monads": all.len(),
        "sector_counts": sector_counts,
        "lineage_and_call_edges_count": edges.len(),
        "lineage_and_call_edges": edges.into_iter().take(50).collect::<Vec<_>>(),
        "sectors": sectors
    }))
    .unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::polar_space::PolarCoord;
    use crate::ontology::monad::{Monad, MonadKind};
    use crate::persistence::surreal_bridge::{connect_embedded, insert_and_link};

    #[tokio::test]
    async fn test_mcp_protocol_and_non_destructive_tools() -> anyhow::Result<()> {
        // 1. Verify initialize & tools/list
        let init_resp = handle_mcp_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "initialize".to_string(),
            params: None,
        })
        .await
        .unwrap();
        assert_eq!(
            init_resp.result.unwrap()["serverInfo"]["safetyMode"],
            "non-destructive-sandbox"
        );

        let tools_resp = handle_mcp_request(JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(2)),
            method: "tools/list".to_string(),
            params: None,
        })
        .await
        .unwrap();
        let tools = tools_resp.result.unwrap()["tools"].as_array().unwrap().clone();
        assert!(tools.len() >= 19, "Expected at least 19 MCP tools, got {}", tools.len());

        // 2. Test tools on an isolated in-memory SurrealDB instance
        let db = connect_embedded().await?;
        let m1_src = "pub fn calcular_impuesto(monto: f64) -> f64 { monto * 0.16 }";
        let m1 = crate::weaver::ast_extractor::extract_raw_monads_lang(m1_src, 1, "rust")
            .into_iter()
            .next()
            .unwrap();
        insert_and_link(&db, &m1, None).await?;

        let m2 = Monad::spawn_typed(
            "hash_r2".to_string(),
            "calcular_impuesto".to_string(),
            PolarCoord::new(200.0, 45.0),
            "pub fn calcular_impuesto(monto: f64) -> f64 { monto * 0.18 }".to_string(),
            2,
            MonadKind::Function,
            1,
            1,
            "rust",
        );
        insert_and_link(&db, &m2, Some(&m1.id)).await?;

        // Telemetry
        let telemetry_out = execute_tool_with_db(&db, "mandala_telemetry", json!({}), None)
            .await
            .unwrap();
        let telemetry_val: JsonValue = serde_json::from_str(&telemetry_out)?;
        assert_eq!(telemetry_val["total_monads"], 2);
        assert_eq!(telemetry_val["max_ring"], 2);

        // Search monads
        let search_out = execute_tool_with_db(
            &db,
            "mandala_search_monads",
            json!({ "query": "calcular_impuesto" }),
            None,
        )
        .await
        .unwrap();
        let search_val: JsonValue = serde_json::from_str(&search_out)?;
        assert_eq!(search_val["count"], 2);

        // Ring diff
        let diff_out = execute_tool_with_db(
            &db,
            "mandala_ring_diff",
            json!({ "base_ring": 1, "target_ring": 2 }),
            None,
        )
        .await
        .unwrap();
        let diff_val: JsonValue = serde_json::from_str(&diff_out)?;
        assert_eq!(diff_val["modified_count"], 1);

        // Audit topology
        let audit_out = execute_tool_with_db(&db, "mandala_audit_topology", json!({ "ring": 1 }), None)
            .await
            .unwrap();
        let audit_val: JsonValue = serde_json::from_str(&audit_out)?;
        assert_eq!(audit_val["healthy"], true);

        // Resolve contract geodesically
        let resolve_out = execute_tool_with_db(
            &db,
            "mandala_resolve_contract",
            json!({ "symbol_name": "calcular_impuesto", "target_angle_deg": 45.0, "max_ring": 2 }),
            None,
        )
        .await
        .unwrap();
        let resolve_val: JsonValue = serde_json::from_str(&resolve_out)?;
        assert_eq!(resolve_val["resolved"], true);

        // Distill spatial (in-memory, save_to_sandbox: false)
        let spatial_out = execute_tool_with_db(
            &db,
            "mandala_distill_spatial",
            json!({ "seeds": ["calcular_impuesto"], "target_angle_deg": 45.0, "max_ring": 2 }),
            None,
        )
        .await
        .unwrap();
        let spatial_val: JsonValue = serde_json::from_str(&spatial_out)?;
        assert!(spatial_val["assembled_source"]
            .as_str()
            .unwrap()
            .contains("calcular_impuesto"));

        // Supported languages (39+)
        let langs_out = execute_tool_with_db(&db, "mandala_supported_languages", json!({}), None)
            .await
            .unwrap();
        let langs_val: JsonValue = serde_json::from_str(&langs_out)?;
        assert!(langs_val["total_supported_languages"].as_u64().unwrap() >= 38);

        // Extract AST directly in-memory
        let ast_out = execute_tool_with_db(
            &db,
            "mandala_extract_ast",
            json!({
                "source_code": "contract Treasury {\n  function withdraw() public {}\n}",
                "language": "sol",
                "ring": 2
            }),
            None,
        )
        .await
        .unwrap();
        let ast_val: JsonValue = serde_json::from_str(&ast_out)?;
        assert!(ast_val["monads_extracted"].as_u64().unwrap() >= 1);

        Ok(())
    }
}
