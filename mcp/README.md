# 🌌 Mandala Weaver VCS — Model Context Protocol (MCP) Server v2.0

> **El Protocolo de Contexto para Modelos (MCP) de Mandala Weaver VCS** conecta directamente a agentes autónomos de Inteligencia Artificial (Claude Desktop, Cursor, Antigravity, Cline, Windsurf, Roo Code, Gemini) con la topología radial continua $SO(2)$, el Grafo Tripartito ontológico y el motor de destilación espacial de software.

El servidor MCP opera sobre **entrada/salida estándar (`stdio`)** utilizando el estándar JSON-RPC 2.0 (versión de protocolo `2024-11-05`), garantizando **cero perturbaciones en `stdout`** (toda salida diagnóstica y telemetría se emite exclusivamente a `stderr`) y una estricta **garantía de no destrucción** de archivos de código en disco.

---

## 🚀 Inicio Rápido

### Mediante la CLI `weave`
```bash
# Iniciar en modo servidor MCP
weave mcp

# Con bandera global de proyecto (opcional)
weave mcp --path /ruta/a/tu/proyecto
```

### Mediante Cargo / Binario Compilado
```bash
# En desarrollo
cargo run --manifest-path src-tauri/Cargo.toml -- mcp

# Con binario de release optimizado
./src-tauri/target/release/mandala-weaver mcp
```

---

## 🛡️ Garantía de Seguridad No-Destructiva (Non-Destructive Sandbox)

El servidor MCP de Mandala Weaver implementa una política de **seguridad no destructiva por diseño**:
1. **Ensamblado y Destilación en Memoria:** Las herramientas `mandala_distill`, `mandala_distill_spatial` y `mandala_unweave` generan el código fuente ensamblado y lo devuelven íntegro dentro del payload JSON en el campo `result.code` o `result.content`, permitiendo que el agente de IA lo analice, lo compare o lo proponga como diff al usuario sin modificar los archivos originales en disco.
2. **Sandbox Aislado:** Si el agente o el usuario activan la bandera `"save_to_sandbox": true`, la salida se escribe **estrictamente** dentro del directorio aislado `.mandala/distilled/` del proyecto, protegiendo al 100% el árbol de código fuente existente contra sobrescrituras o pérdidas accidentales.
3. **Contracción sin Borrado Físico:** La herramienta `mandala_contract_ring` realiza un *soft-archive* en la base de datos SurrealDB (`is_archived = true`), dejando los archivos del repositorio completamente intactos.

---

## 🧰 Catálogo de Herramientas Expuestas (19 Tools)

El servidor expone **19 herramientas especializadas** agrupadas según su dominio en el ecosistema radial:

### 1. Topología, Telemetría y Génesis

| Herramienta | Parámetros | Descripción |
|---|---|---|
| `mandala_telemetry` | `verbose?: bool` | Escanea la topología completa: pulso del ecosistema, recuento de anillos, desglose por lenguajes (39 soportados), mónadas activas/archivadas y contratos indexados. |
| `mandala_bindu` | `project_name: string` | Instancia el Punto Cero $(0,0)$ o génesis ontológico del proyecto en SurrealDB. |
| `mandala_audit_topology` | `ring?: number` | Audita la coherencia semántica del proyecto: detecta colisiones de símbolos duplicados, contratos huérfanos y violaciones de interfaz en anillos activos. |

### 2. Extracción Políglota (39 Lenguajes) y Autoconocimiento

| Herramienta | Parámetros | Descripción |
|---|---|---|
| `mandala_scan_project` | `path: string`<br>`force_rescan?: bool` | Escanea un directorio con el extractor AST nativo para 39 lenguajes, registrando mónadas (con líneas reales 1-based), puertos, contratos y aristas evolutivas en SurrealDB. |
| `mandala_extract_ast` | `source_code?: string`<br>`language?: string`<br>`file_path?: string`<br>`ring?: number` | Extrae en memoria mónadas atómicas, rangos de línea 1-based, coordenadas de Vogel, hashes BLAKE3 y llamadas salientes desde un fragmento de código o archivo en cualquiera de los 39 lenguajes. |
| `mandala_supported_languages` | Ninguno | Devuelve la matriz completa de los 39 lenguajes de programación y formatos soportados, sus extensiones y su motor AST/estructural. |
| `mandala_crystallize` | `file_path: string`<br>`message?: string` | Commit radial: extrae mónadas AST de un archivo fuente políglota y expande la constelación hacia un nuevo anillo exterior. |
| `mandala_unweave` | `path?: string` | **InnerKnow Unweaver:** Descompone un monolito en mónadas atómicas, puebla el Grafo Tripartito (puertos/contratos) y genera automáticamente `.mandala/distill.yaml`. |
| `mandala_git_import` | `repo_path: string` | Importa el historial cronológico de Git traduciendo cada commit a anillos concéntricos evolutivos. |

### 3. Tejido Espacial $SO(2)$ y Geodesia de Contratos

| Herramienta | Parámetros | Descripción |
|---|---|---|
| `mandala_distill_spatial` | `seeds: string[]`<br>`target_angle_deg?: number`<br>`policy?: "StrictGeodesic" \| "SectorBounded" \| "NearestFallback"`<br>`tolerance_deg?: number`<br>`max_ring?: number`<br>`save_to_sandbox?: bool` | **Weaver Espacial $SO(2)$:** Ensambla código fuente a partir de símbolos semilla mediante resolución geodésica continua sobre el Grafo Tripartito. Retorna código en JSON y/o sandbox. |
| `mandala_resolve_contract` | `symbol_name: string`<br>`target_angle_deg?: number`<br>`max_ring?: number` | Resuelve la mónada óptima que implementa un puerto o función evaluando la distancia angular geodésica más cercana en el círculo continuo $SO(2)$. |
| `mandala_vectors` | `sector?: "CORE" \| "IO" \| "DATA" \| "UI" \| "SERVICES"` | Inspecciona la densidad y mónadas en los 5 sectores angulares del espacio $SO(2)$ y lista las aristas de linaje y llamadas salientes (`outbound_calls`). |

### 4. Búsqueda, Inspección y Linaje

| Herramienta | Parámetros | Descripción |
|---|---|---|
| `mandala_search_monads` | `query: string`<br>`kind?: string`<br>`language?: string`<br>`ring?: number`<br>`limit?: number` | Busca mónadas en tiempo real por nombre, tipo (Function, Struct, Trait, Impl, Module), lenguaje, hash semántico o fragmento de código fuente. |
| `mandala_focus` | `monad_name_or_id: string` | Inspecciona en profundidad una mónada: coordenadas polares $(r, \theta)$, tipo AST, hash semántico BLAKE3, contrato canonical, llamadas salientes y código fuente completo. |
| `mandala_lineage` | `monad_id: string` | Traza el linaje evolutivo bidireccional (`evolves_to`) de una mónada a través de los anillos concéntricos interiores y exteriores. |
| `mandala_spectrum` | `monad_name: string` | Análisis cromático de mutaciones semánticas calculando el desplazamiento espectral HSL derivado de su hash BLAKE3 a través de los anillos. |

### 5. Compilación, Comparación y Gestión de Anillos

| Herramienta | Parámetros | Descripción |
|---|---|---|
| `mandala_distill` | `monad_ids?: string[]`<br>`template_content?: string`<br>`save_to_sandbox?: bool` | Compila y ordena topológicamente el código fuente a partir de una lista de IDs de mónadas o un pergamino declarativo YAML. |
| `mandala_ring_diff` | `base_ring: number`<br>`target_ring: number` | Comparador formal de anillos: calcula mónadas añadidas (`+`), modificadas (`~`), eliminadas (`-`) e idénticas (`=`) entre dos niveles del Mandala. |
| `mandala_contract_ring` | `target_ring?: number` | Contrae (archiva de forma no destructiva en SurrealDB) el anillo exterior o revierte al nivel indicado sin eliminar ningún archivo en disco. |
| `mandala_validate_template` | `template_yaml: string` | Valida la sintaxis y semántica de una plantilla `DistillationTemplate` o receta espacial `DistillationRecipe`. |

---

## 📦 Recursos Expuestos (6 Resources)

Los agentes pueden suscribirse o leer directamente los siguientes esquemas URI:

| URI | Descripción |
|---|---|
| `mandala://topology` | Grafo JSON completo del estado del Mandala (anillos, mónadas, constelaciones y aristas evolutivas). |
| `mandala://rings` | Desglose concéntrico de los anillos radiales, métricas de radio y distribución de nodos. |
| `mandala://contracts` | Catálogo de contratos canónicos, puertos nominales e implementaciones en el Grafo Tripartito. |
| `mandala://vectors` | Proyecciones angulares divididas por sectores funcionales $SO(2)$ (`CORE`, `IO`, `DATA`, `UI`, `SERVICES`). |
| `mandala://recipe` | Manifiesto actual `.mandala/distill.yaml` o receta espacial del proyecto activo. |
| `mandala://languages` | Matriz completa de los 39 lenguajes soportados, extensiones y motores de extracción AST. |

---

## 💡 Prompts Nativos (2 Prompts)

| Prompt | Argumentos | Descripción |
|---|---|---|
| `architect_guidance` | Ninguno | Guía integral del Arquitecto: directrices ontológicas sobre el tiempo circular del software, la inmutabilidad del centro (Bindu), la resolución geodésica y el principio de no interferencia. |
| `spatial_distill_prompt` | `seeds: string`<br>`target_angle_deg?: string` | Guía interactiva paso a paso para orquestar una destilación espacial $SO(2)$ a partir de semillas funcionales. |

---

## ⚙️ Ejemplos de Configuración en Entornos de IA

### 1. Claude Desktop (`claude_desktop_config.json`)

En Windows (`%APPDATA%\Claude\claude_desktop_config.json`) o macOS (`~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "mandala-weaver": {
      "command": "G:\\GitHub\\Mandala-Weaver-VCS-main\\Mandala-Weaver-VCS-main\\src-tauri\\target\\release\\mandala-weaver.exe",
      "args": ["mcp"]
    }
  }
}
```

O utilizando `cargo` directamente:
```json
{
  "mcpServers": {
    "mandala-weaver": {
      "command": "cargo",
      "args": [
        "run",
        "--manifest-path",
        "G:\\GitHub\\Mandala-Weaver-VCS-main\\Mandala-Weaver-VCS-main\\src-tauri\\Cargo.toml",
        "--",
        "mcp"
      ]
    }
  }
}
```

### 2. Cursor (`.cursor/mcp.json`)

Crea o edita `.cursor/mcp.json` en tu espacio de trabajo o configuración global:

```json
{
  "mcpServers": {
    "mandala-weaver": {
      "command": "mandala-weaver",
      "args": ["mcp"]
    }
  }
}
```

### 3. Antigravity / Gemini CLI / Windsurf / Cline

Agrega la entrada correspondiente en los ajustes de herramientas MCP:

```json
{
  "name": "mandala-weaver",
  "transport": "stdio",
  "command": "mandala-weaver",
  "args": ["mcp"]
}
```

---

## 🧪 Pruebas Interactivas del Protocolo sobre `stdio`

Puedes verificar el funcionamiento inmediato del servidor enviando tramas JSON-RPC directamente por la terminal:

### Handshake de Inicialización (`initialize`)
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}' | mandala-weaver mcp
```

**Respuesta esperada:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
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
  }
}
```

### Consulta de Telemetría (`mandala_telemetry`)
```bash
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"mandala_telemetry","arguments":{"verbose":true}}}' | mandala-weaver mcp
```

### Búsqueda de Mónadas (`mandala_search_monads`)
```bash
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"mandala_search_monads","arguments":{"query":"connect_db"}}}' | mandala-weaver mcp
```

---

## 🔬 Suite de Pruebas Automatizadas

El servidor MCP incluye pruebas unitarias integradas en Rust que verifican el protocolo JSON-RPC, el aislamiento de canales y la no-destructividad:

```bash
cargo test --lib test_mcp_protocol_and_non_destructive_tools
```
Resultados:
- ✅ Handshake `initialize` y `tools/list`
- ✅ Consulta de recursos `mandala://topology` y `mandala://rings`
- ✅ Búsqueda semántica `mandala_search_monads`
- ✅ Ensamblado no destructivo en memoria `mandala_distill`
- ✅ Confirmación de preservación de archivos en disco
