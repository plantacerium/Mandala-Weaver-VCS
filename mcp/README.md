# Mandala Weaver VCS - Model Context Protocol (MCP) Server

El servidor MCP permite que agentes de Inteligencia Artificial (Claude Desktop, Cursor, Antigravity, Cline, Windsurf, etc.) y desarrolladores interactúen con la topología radial, mónadas, anillos, linajes y destilaciones de Mandala Weaver VCS.

## Herramientas Expuestas (Tools)

| Herramienta | Descripción |
|---|---|
| `mandala_telemetry` | Escanea la topología del proyecto, pulso de anillos, mónadas activas e invariantes. |
| `mandala_bindu` | Instancia el Punto Cero (0, 0) o génesis de un proyecto. |
| `mandala_crystallize` | Commit radial: extrae mónadas AST de un archivo fuente y expande un nuevo anillo. |
| `mandala_distill` | Compila código fuente ejecutable a partir de IDs de mónadas o una plantilla YAML. |
| `mandala_focus` | Inspecciona el AST, contenido, hash semántico y líneas de una mónada específica. |
| `mandala_lineage` | Traza el linaje evolutivo bidireccional de una mónada a través de anillos interiores. |
| `mandala_spectrum` | Análisis cromático de mutaciones semánticas (HSL / BLAKE3). |
| `mandala_scan_project` | Escanea un directorio o repositorio, indexa archivos políglotas y los agrupa en anillos. |
| `mandala_git_import` | Importa el historial de commits de Git convirtiéndolos en anillos concéntricos. |
| `mandala_validate_template` | Valida manifiestos declarativos de destilación en formato YAML. |

## Recursos (Resources)
- `mandala://topology` - Estado actual de constelaciones, anillos y aristas.
- `mandala://rings` - Desglose de anillos radiales y recuentos.

## Configuración en Claude Desktop / Cursor / Antigravity

Añade lo siguiente a tu archivo de configuración de MCP (por ejemplo, `claude_desktop_config.json` o settings de Cursor/Antigravity):

```json
{
  "mcpServers": {
    "mandala-weaver": {
      "command": "cargo",
      "args": [
        "run",
        "--manifest-path",
        "<PATH_TO_MANDALA_WEAVER>/src-tauri/Cargo.toml",
        "--",
        "mcp"
      ]
    }
  }
}
```

O apuntando directamente al binario compilado:

```json
{
  "mcpServers": {
    "mandala-weaver": {
      "command": "<PATH_TO_MANDALA_WEAVER>/src-tauri/target/release/mandala-weaver",
      "args": ["mcp"]
    }
  }
}
```
