use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use crate::persistence::surreal_bridge::{
    connect_db, find_project_root, get_all_monads,
};
use crate::ontology::bindu::Bindu;
use crate::weaver::source_compiler::distill_source;
use crate::weaver::threader::trace_full_chain;
use crate::template::DistillationTemplate;

#[derive(Deserialize, Debug)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    #[serde(default)]
    jsonrpc: String,
    id: Option<JsonValue>,
    method: String,
    #[serde(default)]
    params: Option<JsonValue>,
}

#[derive(Serialize, Debug)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonValue>,
}

pub async fn run_mcp_server() -> anyhow::Result<()> {
    eprintln!("🌌 Mandala Weaver MCP Server starting on stdio...");

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

async fn handle_mcp_request(req: JsonRpcRequest) -> Option<JsonRpcResponse> {
    match req.method.as_str() {
        "initialize" => {
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id,
                result: Some(json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {},
                        "resources": {},
                        "prompts": {}
                    },
                    "serverInfo": {
                        "name": "mandala-weaver-vcs",
                        "version": "0.1.0"
                    }
                })),
                error: None,
            })
        }
        "notifications/initialized" => None,
        "ping" => {
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id,
                result: Some(json!({})),
                error: None,
            })
        }
        "tools/list" => {
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id,
                result: Some(json!({
                    "tools": [
                        {
                            "name": "mandala_telemetry",
                            "description": "Scan Mandala topology and return ecosystem pulse, ring counts, and monad stats.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "verbose": { "type": "boolean", "description": "Detailed ring breakdown" }
                                }
                            }
                        },
                        {
                            "name": "mandala_bindu",
                            "description": "Initialize Point Zero (0,0) (genesis) for a project.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "project_name": { "type": "string", "description": "Project name" }
                                },
                                "required": ["project_name"]
                            }
                        },
                        {
                            "name": "mandala_crystallize",
                            "description": "Radial commit: Parse source code into AST Monads and expand outward into a new ring.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "file_path": { "type": "string", "description": "Source file path to crystallize" },
                                    "message": { "type": "string", "description": "Intent message" }
                                },
                                "required": ["file_path"]
                            }
                        },
                        {
                            "name": "mandala_distill",
                            "description": "Distill and compile executable source code from monad IDs or a YAML template manifest.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "monad_ids": { "type": "array", "items": { "type": "string" }, "description": "List of monad IDs to weave" },
                                    "template_content": { "type": "string", "description": "Optional YAML template content" }
                                }
                            }
                        },
                        {
                            "name": "mandala_focus",
                            "description": "Inspect details of a monad (AST, content, lines, semantic hash, chromatic color).",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "monad_name_or_id": { "type": "string", "description": "Monad name or ID" }
                                },
                                "required": ["monad_name_or_id"]
                            }
                        },
                        {
                            "name": "mandala_lineage",
                            "description": "Query the evolutionary lineage/ancestor chain of a monad from inner rings.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "monad_id": { "type": "string", "description": "Monad ID" }
                                },
                                "required": ["monad_id"]
                            }
                        },
                        {
                            "name": "mandala_spectrum",
                            "description": "Analyze chromatic shift (BLAKE3 HSL variance) of a monad across rings.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "monad_name": { "type": "string", "description": "Monad name" }
                                },
                                "required": ["monad_name"]
                            }
                        },
                        {
                            "name": "mandala_scan_project",
                            "description": "Scan a directory or project repository, index source files, and extract into radial rings.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "path": { "type": "string", "description": "Folder path to scan" }
                                },
                                "required": ["path"]
                            }
                        },
                        {
                            "name": "mandala_git_import",
                            "description": "Import git commit history as concentric evolutionary rings into the Mandala.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "repo_path": { "type": "string", "description": "Git repository root path" }
                                },
                                "required": ["repo_path"]
                            }
                        },
                        {
                            "name": "mandala_validate_template",
                            "description": "Validate a YAML distillation template manifest.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "template_yaml": { "type": "string", "description": "YAML content" }
                                },
                                "required": ["template_yaml"]
                            }
                        }
                    ]
                })),
                error: None,
            })
        }
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
        "resources/list" => {
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id,
                result: Some(json!({
                    "resources": [
                        {
                            "uri": "mandala://topology",
                            "name": "Mandala Topology State",
                            "mimeType": "application/json",
                            "description": "Current active constellation and rings"
                        },
                        {
                            "uri": "mandala://rings",
                            "name": "Mandala Rings Overview",
                            "mimeType": "application/json",
                            "description": "Overview of rings and monad counts"
                        }
                    ]
                })),
                error: None,
            })
        }
        "resources/read" => {
            let params = req.params.unwrap_or(json!({}));
            let uri = params.get("uri").and_then(|v| v.as_str()).unwrap_or("");
            let root = find_project_root();
            let db = match connect_db(root.as_deref()).await {
                Ok(d) => d,
                Err(e) => return Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: None,
                    error: Some(json!({ "code": -32000, "message": format!("Database connection failed: {}", e) })),
                }),
            };

            match uri {
                "mandala://topology" | "mandala://rings" => {
                    let all_monads = get_all_monads(&db).await.unwrap_or_default();
                    let payload = serde_json::to_string_pretty(&all_monads).unwrap_or_default();
                    Some(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req.id,
                        result: Some(json!({
                            "contents": [
                                {
                                    "uri": uri,
                                    "mimeType": "application/json",
                                    "text": payload
                                }
                            ]
                        })),
                        error: None,
                    })
                }
                _ => Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: None,
                    error: Some(json!({ "code": -32002, "message": format!("Resource not found: {}", uri) })),
                }),
            }
        }
        "prompts/list" => {
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id,
                result: Some(json!({
                    "prompts": [
                        {
                            "name": "architect_guidance",
                            "description": "Guidance on how to reason about Circular Time, Monads, Rings, Vectors, and Distillation.",
                            "arguments": []
                        }
                    ]
                })),
                error: None,
            })
        }
        "prompts/get" => {
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id,
                result: Some(json!({
                    "description": "Architect Guidance for Mandala Weaver VCS",
                    "messages": [
                        {
                            "role": "user",
                            "content": {
                                "type": "text",
                                "text": "Mandala Weaver operates on Circular Time and Sacred Geometry:\n- Monads: Pure logic units isolated by ast-grep AST analysis.\n- Bindu (0,0): Immutable genesis of purpose.\n- Rings (r): Temporal iterations and evolutionary abstraction expanding outward.\n- Vectors (theta): Logical domains (CORE 0°-90°, IO/NETWORK 90°-180°, UI 180°-270°, DATA 270°-360°).\n- Distillation: Composing sources by picking coordinates or YAML manifests across rings.\n- Chromatic Synesthesia: BLAKE3 semantic hash mapped to HSL color."
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
            result: None,
            error: Some(json!({
                "code": -32601,
                "message": format!("Method not found: {}", req.method)
            })),
        }),
    }
}

async fn execute_tool(name: &str, args: JsonValue) -> Result<String, String> {
    let root = find_project_root();
    let db = connect_db(root.as_deref()).await.map_err(|e| e.to_string())?;

    match name {
        "mandala_telemetry" => {
            let all = get_all_monads(&db).await.map_err(|e| e.to_string())?;
            let mut ring_map: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
            for m in &all {
                *ring_map.entry(m.ring).or_default() += 1;
            }
            let max_ring = all.iter().map(|m| m.ring).max().unwrap_or(0);
            let info = json!({
                "total_monads": all.len(),
                "max_ring": max_ring,
                "ring_breakdown": ring_map,
                "status": "Synchronized"
            });
            Ok(serde_json::to_string_pretty(&info).unwrap())
        }
        "mandala_bindu" => {
            let project_name = args.get("project_name").and_then(|v| v.as_str()).unwrap_or("mandala-project");
            let bindu = Bindu::genesis(project_name);
            let val = serde_json::to_value(&bindu).map_err(|e| e.to_string())?;
            let _: Option<JsonValue> = db.create(("bindu", "genesis"))
                .content(val)
                .await
                .map_err(|e| e.to_string())?;
            Ok(format!("Point Zero instantiated: Project '{}' initialized at timestamp {}.", project_name, bindu.timestamp))
        }
        "mandala_crystallize" => {
            let file_path = args.get("file_path").and_then(|v| v.as_str()).ok_or("Missing file_path")?;
            let new_ring = crate::weaver::expand_from_source(&db, file_path).await.map_err(|e| e.to_string())?;
            Ok(format!("File '{}' crystallized into Ring {}.", file_path, new_ring))
        }
        "mandala_distill" => {
            if let Some(template_yaml) = args.get("template_content").and_then(|v| v.as_str()) {
                let template: DistillationTemplate = serde_yaml::from_str(template_yaml).map_err(|e| format!("YAML error: {}", e))?;
                let engine = crate::template::engine::TemplateEngine::new(db.clone());
                let monads = engine.resolve(&template).await.map_err(|e| e.to_string())?;
                if monads.is_empty() {
                    return Err("No monads matched the template criteria.".to_string());
                }
                let source = crate::weaver::source_compiler::distill_source_with_adapters(&monads, &template.adapters);
                Ok(source)
            } else if let Some(ids_val) = args.get("monad_ids").and_then(|v| v.as_array()) {
                let ids: Vec<String> = ids_val.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                let all = get_all_monads(&db).await.map_err(|e| e.to_string())?;
                let selected: Vec<_> = all.into_iter().filter(|m| ids.contains(&m.id)).collect();
                if selected.is_empty() {
                    return Err("No monads matched the IDs provided.".to_string());
                }
                let source = distill_source(&selected);
                Ok(source)
            } else {
                Err("Please provide either monad_ids or template_content".to_string())
            }
        }
        "mandala_focus" => {
            let target = args.get("monad_name_or_id").and_then(|v| v.as_str()).ok_or("Missing monad_name_or_id")?;
            let all = get_all_monads(&db).await.map_err(|e| e.to_string())?;
            let found = all.into_iter().find(|m| m.id == target || m.name == target);
            match found {
                Some(m) => Ok(serde_json::to_string_pretty(&m).unwrap()),
                None => Err(format!("Monad '{}' not found in Akashic Record", target)),
            }
        }
        "mandala_lineage" => {
            let id = args.get("monad_id").and_then(|v| v.as_str()).ok_or("Missing monad_id")?;
            let chain = trace_full_chain(&db, id).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_string_pretty(&chain).unwrap())
        }
        "mandala_spectrum" => {
            let name = args.get("monad_name").and_then(|v| v.as_str()).ok_or("Missing monad_name")?;
            let all = get_all_monads(&db).await.map_err(|e| e.to_string())?;
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
                    "kind": m.kind,
                    "theta": m.coord.theta,
                })).collect::<Vec<_>>()
            });
            Ok(serde_json::to_string_pretty(&info).unwrap())
        }
        "mandala_scan_project" => {
            let path_str = args.get("path").and_then(|v| v.as_str()).ok_or("Missing path")?;
            let path = PathBuf::from(path_str);
            if !path.exists() {
                return Err(format!("Path does not exist: {}", path_str));
            }
            let entry = crate::synarchy::ProjectScanner::scan(&path).await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_string_pretty(&entry).unwrap())
        }
        "mandala_git_import" => {
            let path_str = args.get("repo_path").and_then(|v| v.as_str()).ok_or("Missing repo_path")?;
            let path = PathBuf::from(path_str);
            let rings_imported = crate::collaboration::import_git_history(&db, &path).await.map_err(|e| e.to_string())?;
            Ok(format!("Imported git history from '{}': created {} evolutionary rings.", path_str, rings_imported))
        }
        "mandala_validate_template" => {
            let yaml = args.get("template_yaml").and_then(|v| v.as_str()).ok_or("Missing template_yaml")?;
            let template: DistillationTemplate = serde_yaml::from_str(yaml).map_err(|e| format!("Invalid YAML: {}", e))?;
            let info = json!({
                "valid": true,
                "name": template.name,
                "version": template.version,
                "vectors_count": template.vectors.len(),
            });
            Ok(serde_json::to_string_pretty(&info).unwrap())
        }
        _ => Err(format!("Unknown tool: {}", name)),
    }
}
