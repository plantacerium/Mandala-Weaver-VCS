# 🗺️ Roadmap de Implementación: Mandala Weaver VCS

> **Last audited against codebase:** 2026-10-16 (Post-MCP-v2-and-Synarchy-Upgrade)
> **Legend:** ✅ Done — 🔧 Needs refinement — ⬜ Not started — 🚧 In progress

---

## 📍 Fase 0: Fundación (Scaffolding & Configuración)

Establecer la estructura de directorios y el esqueleto de Tauri + Rust.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 0.1 | **Inicialización del Workspace Rust** — Estructura de módulos en `src-tauri/src/` | ✅ | `geometry/`, `ontology/`, `weaver/`, `persistence/`, `interface/` |
| 0.2 | **Integración Tauri + Astro** — Frontend React + D3.js operacional | ✅ | Astro 4, React 18, D3 7 — `package.json` verified |
| 0.3 | **SurrealDB Embebido** — Motor `kv-mem` puro Rust | ✅ | `Surreal::new::<Mem>(())` en `surreal_bridge.rs` |
| 0.4 | **Resolución de Errores de Compilación** — Lifetimes, SurrealValue, Tauri config | ✅ | `cargo check` y `cargo test --lib` limpios con 0 warnings críticos |

---

## 📍 Fase 1: El Motor Central (Geometría y Parsing)

Matemáticas y capacidad de leer código sin tocar la base de datos ni la UI.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 1.1 | **Primitivas Geométricas (`nalgebra`)** | ✅ | `PolarCoord { r, theta }`, `to_cartesian()`, `from_cartesian()`, rotaciones continuas $SO(2)$ |
| 1.2 | **Estructuras Ontológicas** | ✅ | `Bindu`, `Monad`, `Ring`, `Vector`, `Constellation`, `Contract`, `Port` |
| 1.3 | **Extractor de Mónadas (`ast-grep`)** | ✅ | Native `ast-grep-core` tree-sitter extraction políglota para 22 lenguajes |
| 1.3.1 | → Integrar `ast-grep-core` y gramáticas Tree-sitter | ✅ | Rust, TypeScript, JavaScript, Python, Go, Java, C, C++, C#, Ruby, etc. |
| 1.3.2 | → Extraer `struct`, `enum`, `impl`, `trait`, `class`, `interface` además de `fn` | ✅ | Mapeo riguroso en `kind_from_node` para todos los lenguajes soportados |
| 1.3.3 | → Preservar span de líneas reales (1-based start/end) en cada Monad | ✅ | `node.start_pos().line() + 1` y `node.end_pos().line() + 1` implementados |
| 1.3.4 | → Unit tests con fuentes reales políglotas (>200 LOC) | ✅ | Tests pasando localmente (`test_polyglot_ast_extraction`, etc.) |
| 1.4 | **Lógica de Resolución Espacial** — Colisiones y desplazamiento orbital | ✅ | `detect_overlap()`, `resolve_orbital_shift()`, dispersión áurea ($137.5^\circ$) |

---

## 📍 Fase 2: El Registro Akáshico (Persistencia Espacial)

Conectar la lógica matemática con SurrealDB para guardar la historia radial y el Grafo Tripartito.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 2.1 | **Esquemas SurrealQL** — Tablas `monad`, relación `evolves_to`, contratos y puertos | ✅ | `schemas.rs::get_initialization_queries()` |
| 2.2 | **Aristas Evolutivas (Edges)** — Lógica `RELATE` compatible con SurrealDB v3 | ✅ | `LET $p = type::record(...); LET $c = ...; RELATE $p -> evolves_to -> $c;` |
| 2.3 | **Repositorio CRUD en Rust** — `surreal_bridge.rs` & `store.rs` | ✅ | `get_ring()`, `get_all_monads()`, `get_vector_sector()`, `MandalaStore::upsert_monad` |
| 2.4 | **Tests de Integración DB** — Mónadas y grafos simulados | ✅ | `persistence::tests::test_simulate_growth()`, `test_tripartite_graph_upsert_and_resolve` |
| 2.5 | **Eliminación de Mónadas (Soft Delete)** | ✅ | `is_archived: bool` field, `archive_monad()`, `get_active_monads()`, `purge_archived_monads()` |
| 2.6 | **Consulta de Linaje Completo** — Traverse bidireccional | ✅ | `threader.rs::trace_full_chain()` con traversals `<-evolves_to<-` y `->evolves_to->` |
| 2.7 | **Consulta de Búsqueda Semántica** — Buscar mónada por hash, nombre o código | ✅ | `projection_api::search_monads()` y MCP `mandala_search_monads` |

---

## 📍 Fase 3: El Tejedor (Lógica de Negocio y Cooperación de Versiones)

Comandos lógicos que reemplazan git commit, git checkout y git merge con tejido continuo $SO(2)$.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 3.1 | **Operación Expand (El nuevo Commit)** | ✅ | `weaver/mod.rs::expand_from_source()` — lectura, extracción, delta y persistencia |
| 3.2 | **Operación Distill (El nuevo Checkout/Build)** | ✅ | `source_compiler.rs::distill_source()` — ensamblado determinista con ordenamiento de dependencias |
| 3.2.1 | → Ordenar mónadas por dependencia (topo-sort) antes de concatenar | ✅ | `order_by_dependency()` — modules → const/type → struct/enum → trait → impl → fn |
| 3.2.2 | → Generar `use`/`mod` statements automáticos entre mónadas | ✅ | `weaver/auto_imports.rs::ImportAnalyzer` — análisis de referencias cruzadas |
| 3.2.3 | → Escribir resultado a disco con estructura de carpetas del proyecto | ✅ | `weaver/file_writer.rs::FileWriter` — soporte para sandbox `.mandala/distilled/` |
| 3.3 | **Detección de Incoherencias** | ✅ | Validación de fuentes contra definiciones duplicadas y bloqueos sintácticos |
| 3.3.1 | → Detector de definiciones duplicadas en una selección de mónadas | ✅ | `validate_source_coherence` comparando `name` y `contract_hash` |
| 3.3.2 | → Validación sintáctica del Source compilado (`syn::parse_file`) | ✅ | Validación sintáctica AST nativa en Rust, TS y Python |
| 3.3.3 | → Informe de conflictos con coordenadas de las mónadas conflictivas | ✅ | Retorna `Vec<IncoherenceReport>` a frontend y respuestas MCP |
| 3.4 | **Operación Contract (El Anti-Expand)** | ✅ | `weaver/contract.rs::contract_ring()` corregido con `GROUP ALL` y `mandala_contract_ring` |
| 3.5 | **Delta Semántico Mejorado** | ✅ | `resolver.rs::identify_deltas_typed` con hashes BLAKE3 normalizados |
| 3.5.1 | → Comparar hashes semánticos (blake3) en lugar de IDs | ✅ | `is_semantically_different()` usando el campo `semantic_hash` |
| 3.5.2 | → Calcular `DeltaType` enum: `Added`, `Modified`, `Renamed`, `Deleted` | ✅ | Categorización limpia en `identify_deltas_typed()` |
| 3.5.3 | → Generar diff semántico entre dos versiones de una mónada | ✅ | `weaver/semantic_diff.rs::SemanticDiff` & MCP `mandala_ring_diff` |

---

## 📍 Fase 4: El Telar (Puente Tauri IPC)

Conectar el backend Rust con el frontend JavaScript de forma segura y en tiempo real.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 4.1 | **Definición de Eventos Tauri** — `export_mandala_state` | ✅ | Operacional en `projection_api.rs` |
| 4.2 | **Serialización Optimizada** — TypeScript ↔ Rust types | ✅ | `Monad`, `MandalaState`, `ConstellationDto` con Serde |
| 4.3 | **Comando `expand_ring`** — Recibir archivos del frontend | ✅ | `expand_ring(file_path: String) -> Result<u32, String>` |
| 4.4 | **Escucha de Eventos en Tiempo Real (FS Watcher)** | ✅ | Watcher en segundo plano con debounce y filtro de extensiones |
| 4.4.1 | → Configurar `notify::RecommendedWatcher` en el directorio activo | ✅ | Manejado por `notify_debouncer_mini` con auto-actualización en `open_project` |
| 4.4.2 | → Emitir evento Tauri seguro `mandala:file-changed` | ✅ | Notificación conforme al estándar alfanumérico de Tauri v2 |
| 4.4.3 | → Frontend listener en `lib/tauri/events.ts` | ✅ | `listenForFileChanges` recarga silenciosamente el estado del lienzo |
| 4.5 | **Comando `distill_source`** — Tauri command para compilar una Fuente | ✅ | `distill_from_selection` IPC con validación de coherencia |
| 4.6 | **Comando `trace_lineage`** — Exponer `threader.rs` vía IPC | ✅ | `trace_monad_lineage` devuelve la cadena evolutiva completa |
| 4.7 | **Comando `get_monad_detail`** — Detalle completo de una mónada | ✅ | `get_monad_detail` devuelve líneas 1-based, llamadas y polar coordinates |
| 4.8 | **Gestión del Bindu (Inicialización de Proyecto)** | ✅ | `init_project` IPC implementado y verificado en SurrealDB |

---

## 📍 Fase 5: El Mandala (Visualización Frontend con D3.js)

La capa de magia visual e interacción del usuario en el espacio polar.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 5.1 | **Renderizado Base del Canvas/SVG** — Cuadrícula polar, Bindu | ✅ | `drawMandalaGrid()`, `drawBindu()` con resplandor neon |
| 5.2 | **Dibujado de Nodos y Conexiones** — Mapeo polar → SVG | ✅ | `renderMonads()` con interpolación suave y física orbital |
| 5.2.1 | → **Firmas Cromáticas Únicas** — Color dinámico BLAKE3 $\to$ HSL | ✅ | `renderer.ts` mapeando hashes a frecuencias cromáticas estables |
| 5.3 | **Interacción (Hover y Selección)** — Inspector de mónadas | ✅ | `highlightMonad()`, click/hover handlers en `MandalaCanvas.tsx` |
| 5.4 | **State Management (Zustand)** — Store centralizado | ✅ | `workspaceStore.ts` — `mandalaState`, `selectedMonad`, `viewMode` |
| 5.5 | **Interacciones D3 (Zoom/Pan)** — Zoom semántico y respiración | ✅ | `setupZoom()`, `enableLassoSelection()` en `interactions.ts` |
| 5.6 | **Build Verificado** — `pnpm run build` limpio (código 0) | ✅ | Astro + React compilan sin errores de tipado |
| 5.7 | **Seleccionador de Fuente (El Lazo)** — Selección poligonal | ✅ | Integración de lazo poligonal cerrada automáticamente |
| 5.7.1 | → Cerrar polígono automáticamente al soltar mouse | ✅ | Bounding box y selección contenida activa |
| 5.7.2 | → Listar mónadas seleccionadas en panel lateral | ✅ | `DistillPanel.tsx` reflejando la selección activa |
| 5.7.3 | → Botón "DISTILL" que invoque `distill_source` | ✅ | Conectado a Tauri IPC con previsualización |
| 5.8 | **Renderizado de Linaje (Aristas Evolutivas)** | ✅ | Curvas Bézier trazadas entre mónadas progenitoras y derivadas |
| 5.8.1 | → Fetch `evolves_to` edges desde el backend | ✅ | Aristas cargadas y enviadas en `export_mandala_state` |
| 5.8.2 | → Enlaces curvos con gradiente de opacidad radial | ✅ | `renderEdges` conectando coordenadas polares |
| 5.8.3 | → Animación de ruta de linaje al seleccionar mónada | ✅ | Animación D3 pulse en `lib/d3/animations.ts` |
| 5.9 | **TooltipNode Mejorado** | ✅ | Muestra nombre, anillo, lenguaje y coordenadas en hover |
| 5.10 | **SidebarHistory Mejorado** | ✅ | Estadísticas dinámicas de Bindu, anillos y mónadas |
| 5.11 | **MonadInspector Zen a Altura Completa** | ✅ | Visor con gutter de números de línea, split view y botón Zen |
| 5.12 | **Búsqueda Rápida y Command Palette** | ✅ | Conectada a `search_monads` y expuesta en MCP y frontend |
| 5.13 | **Layout Responsivo y Divisores Arrastrables** | ✅ | |
| 5.13.1 | → Manijas de redimensionamiento (resize handles) | ✅ | Divisores arrastrables en `SplitViewZen` y `VectorsPanel` |
| 5.13.2 | → Selector de temas Claro / Oscuro con variables CSS | ✅ | Variables `[data-theme="light"]` en `global.css` |
| 5.14 | **Breathing Animation (Respiración Semántica)** | ✅ | `lib/d3/zoom.ts::setMacroZoom()`, `setMicroZoom()` |

---

## 📍 Fase 6: Pergaminos de Destilación (Templates YAML & Recetas $SO(2)$)

Manifiestos declarativos para automatizar la composición de fuentes y resolución espacial de contratos.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 6.1 | **Diseño del Esquema YAML Unificado** | ✅ | Estandarizado en `serde_yml`, eliminando librerías deprecadas |
| 6.1.1 | → Esquema dual: `DistillationTemplate` y `DistillationRecipe` | ✅ | Soporte simultáneo para selectores angulares y semillas $SO(2)$ |
| 6.1.2 | → Deserialización serde robusta | ✅ | Validadores integrados en `template/engine.rs` y `mcp.rs` |
| 6.2 | **Motor de Resolución de Templates** | ✅ | `TemplateEngine` resuelve a mónadas activas |
| 6.2.1 | → Resolver `rings[]` + `vectors[]` a coordenadas candidatas | ✅ | Filtros por nivel, rango o más recientes |
| 6.2.2 | → Aplicar filtros `exclude[]` y reglas de compatibilidad | ✅ | Exclusiones semánticas por patrones glob |
| 6.2.3 | → Ejecutar adaptadores semánticos (`@mandala-bridge`) | ✅ | Inyección automática de puentes en colisión de contratos |
| 6.3 | **Comandos Tauri de Plantillas** | ✅ | `distill_from_template`, `distill_spatial_recipe`, etc. |
| 6.4 | **UI: Editor de Plantillas y Recetas Espaciales** | ✅ | Pestaña Espacial $SO(2)$ en `DistillPanel.tsx` con selectores de ángulo |
| 6.5 | **Templates de Ejemplo Incluidos** | ✅ | `minimal.yaml`, `core-only.yaml`, `full-stack.yaml`, `backend-api.yaml` |

---

## 📍 Fase 7: El Camino del Arquitecto (CLI, TUI & Servidor MCP v2.0)

Interfaz de línea de comandos y servidor de protocolo para agentes de Inteligencia Artificial.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 7.1 | **CLI Core (`clap`)** — 18 Comandos Operacionales | ✅ | Implementados en `cli_commands.rs` con soporte `--json` y `--path` |
| 7.1.1 | → `weave bindu` — Crear Bindu (Punto Cero) | ✅ | Instancia la raíz $(0,0)$ en SurrealDB |
| 7.1.2 | → `weave seed <source>` — Plantar repositorio | ✅ | Importa historial Git o archivo Mandala |
| 7.1.3 | → `weave telemetry` — Escanear topología | ✅ | Retorna pulso y métricas de anillos |
| 7.1.4 | → `weave focus <monad>` — Enfocar mónada | ✅ | Inspecciona AST y linaje |
| 7.1.5 | → `weave crystallize` — Commit radial | ✅ | Genera un nuevo anillo con mónadas AST |
| 7.1.6 | → `weave vector <angle>` — Nuevo vector | ✅ | Abre sector de exploración angular |
| 7.1.7 | → `weave dormant` — Estado latente | ✅ | Limpia caché a espacio latente |
| 7.1.8 | → `weave distill` — Compilar Fuente | ✅ | Ensamblado modular en disco o sandbox |
| 7.1.9 | → `weave spectrum` — Análisis cromático | ✅ | Desplazamiento HSL por BLAKE3 |
| 7.1.10 | → `weave lineage` — Consultar linaje | ✅ | Espiral evolutiva de transformaciones |
| 7.1.11 | → `weave echo` — Eco de anillo interior | ✅ | Reproduce lógica histórica |
| 7.1.12 | → `weave absorb` — Sincronización de red | ✅ | Fusión geométrica desde archivo remoto |
| 7.1.13 | → `weave synthesize` — Síntesis vectorial | ✅ | Fusiona dos vectores funcionales |
| 7.1.14 | → `weave emanate` — Emisión a la red | ✅ | Exporta el estado a archivo `.mandala.json` |
| 7.1.15 | → `weave scan [path]` — Escaneo nativo políglota | ✅ | Indexa proyectos a anillos sin requerir Git |
| 7.1.16 | → `weave innerknow [path]` — Autoconocimiento | ✅ | Descompone monolitos y puebla puertos/contratos |
| 7.1.17 | → `weave audit` — Auditoría de coherencia | ✅ | Valida contratos, puertos y duplicados |
| 7.1.18 | → `weave mcp` — Servidor MCP sobre `stdio` | ✅ | Protocolo JSON-RPC para agentes de IA |
| 7.2 | **Persistencia Duradera (SurrealKV)** | ✅ | Base de datos embebida en `.mandala/db/` |
| 7.3 | **Model Context Protocol (MCP v2.0)** | ✅ | Servidor con 17 herramientas, 5 recursos, 2 prompts y sandbox no destructivo |

---

## 📍 Fase 8: El Ojo Políglota (Extracción AST Multi-Lenguaje)

Soporte nativo de extracción sintáctica para 22 lenguajes de programación.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 8.1 | **Detector de Lenguajes** | ✅ | `src/language/mod.rs` con detección por extensión y contenido |
| 8.2 | **Extracción Rust, TS/JS, Python y Go con ast-grep** | ✅ | Gramáticas Tree-sitter nativas y mapeo de tipos de nodo |
| 8.3 | **Extracción de 17 Lenguajes Adicionales** | ✅ | Java, C, C++, C#, Kotlin, Swift, Ruby, PHP, Scala, Elixir, Haskell, Lua, Dart, SQL, HTML, CSS, YAML |
| 8.4 | **Líneas Reales 1-Based** | ✅ | `line_start` y `line_end` basados en posiciones de línea reales (no byte offsets) |
| 8.5 | **Normalización de Nombres de Módulos y Tests** | ✅ | Nombres calificados `<file_stem>::tests` y módulos sintéticos |
| 8.6 | **Reglas de Destilación Multi-Lenguaje** | ✅ | `distill_multi_lang()`, `order_by_language()`, `group_by_language()` |

---

## 📍 Fase 9: El Ancla (Persistencia SurrealKV y Rendimiento)

Almacenamiento persistente, aislamiento de concurrencia e indexación masiva.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 9.1 | **SurrealKV File Engine** | ✅ | Persistencia local en `.mandala/db/` con motor `SurrealKV` de SurrealDB v3 |
| 9.1.1 | → Conexión unificada en `State<'_, SharedDb>` | ✅ | Eliminados los bloqueos de archivo (`os error 33`) en llamadas concurrentes |
| 9.2 | **Inserciones Masivas Atómicas (`insert_batch`)** | ✅ | Carga de alto rendimiento para miles de nodos |
| 9.3 | **Paginación y Carga Perezosa de Mónadas** | ✅ | `get_monads_paginated` para escalabilidad en proyectos grandes |
| 9.4 | **Optimización de Renderizado** | ✅ | Agrupación en capas D3 y descarte de nodos fuera de viewport |
| 9.5 | **Indexación Compuesta** | ✅ | Índices en `(ring, coord.theta)` y `(name, ring)` en `schemas.rs` |

---

## 📍 Fase 10: La Red de Mandalas (Colaboración y Distribución)

Herramientas para intercambio y síntesis entre proyectos y colaboradores.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 10.1 | **Export/Import de Mandala** | ✅ | `collaboration/mod.rs::export_mandala()` — archivo `.mandala.json` |
| 10.2 | **Mandala Diff** | ✅ | `diff_mandala()` — comparación anillo por anillo |
| 10.3 | **Git Bridge (Lectura Completa)** | ✅ | `import_git_history()` convierte commits en anillos concéntricos |
| 10.4 | **Mandala Merge Geométrico** | ✅ | `merge_mandala()` con resolución espacial y base compartida |
| 10.5 | **Sistema de Plugins de Lenguaje** | ✅ | `plugins/mod.rs::MandalaPlugin` framework de extensibilidad |

---

## 📍 Fase 11: El Navegador del Sínarca (Synarchy Multi-Project Explorer)

Organización visual de proyectos en constelaciones radiales.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 11.1 | **Project Registry** — Registro central de proyectos | ✅ | `synarchy/registry.rs` con soporte de rescan y exclusión de locks |
| 11.2 | **Explorer UI** — Interfaz de exploración responsiva | ✅ | Tarjetas en grid `minmax(310px, 1fr)`, mini-mandalas y barra de progreso |
| 11.2.1 | → Botón global "⚡ Escanear Todos (Scan All)" | ✅ | Escaneo en lote de todos los proyectos registrados |
| 11.2.2 | → Botones directos "🧭 Abrir Lienzo" y "↻ Scan AST" | ✅ | Navegación inmediata por proyecto |
| 11.3 | **Inspector de Vectores y Linaje $SO(2)$ (`VectorsPanel`)** | ✅ | Desglose por sectores (`CORE`, `IO`, `DATA`, `UI`, `SERVICES`) y aristas inter-anulares |
| 11.4 | **Project Detail & Ring Diff** | ✅ | Métricas de actividad en tiempo real y comparador formal de anillos (`mandala_ring_diff`) |

---

## 📍 Próximos Horizontes (Fases 12 - 14)

| # | Milestone | Status | Description |
|---|-----------|--------|-------------|
| 12.1 | **Malla de Sincronización P2P en Tiempo Real** | ⬜ | Intercambio peer-to-peer de mónadas mediante libp2p o WebRTC sin servidor central. |
| 13.1 | **Compilador WASM y Plugins en Caliente** | ⬜ | Ejecución de transformadores de destilación y plugins de lenguaje directamente en WebAssembly. |
| 14.1 | **Visualizador Celestial 3D (WebGPU / Three.js)** | ⬜ | Proyección de constelaciones de software en esferas armónicas 3D y toroides topológicos. |

---

## 📊 Resumen de Progreso Actualizado

| Fase | Total Items | ✅ Done | 🔧 Refine | ⬜ Pending |
|-------|------------|---------|-----------|-----------|
| 0 — Fundación | 4 | 4 | 0 | 0 |
| 1 — Motor Central | 8 | 8 | 0 | 0 |
| 2 — Registro Akáshico | 7 | 7 | 0 | 0 |
| 3 — El Tejedor | 14 | 14 | 0 | 0 |
| 4 — El Telar IPC | 12 | 12 | 0 | 0 |
| 5 — El Mandala UI | 21 | 21 | 0 | 0 |
| 6 — Templates YAML & Recetas $SO(2)$ | 7 | 7 | 0 | 0 |
| 7 — CLI, TUI & MCP v2.0 | 18 | 18 | 0 | 0 |
| 8 — Multi-Language AST (22 Lenguajes) | 6 | 6 | 0 | 0 |
| 9 — Persistencia SurrealKV & Rendimiento | 5 | 5 | 0 | 0 |
| 10 — Colaboración y Red | 5 | 5 | 0 | 0 |
| 11 — Synarchy Explorer | 15 | 15 | 0 | 0 |
| 12-14 — Horizontes Futuros | 3 | 0 | 0 | 3 |
| **TOTAL** | **125** | **122** | **0** | **3** |

---

## 📝 Notas de Producción y Calidad

* **Compilación de Producción:** `"bundle": { "active": true }` y política CSP estricta habilitadas en `src-tauri/tauri.conf.json`.
* **Higiene de Logs:** Toda la salida del motor se envía exclusivamente a `stderr` (`eprintln!`), manteniendo `stdout` 100% puro para tramas JSON-RPC del protocolo MCP y la bandera `--json` del CLI.
* **Garantía No-Destructiva:** Ninguna herramienta de destilación o escaneo sobrescribe código fuente en disco; la salida generada se retorna en memoria o se escribe en el sandbox seguro `.mandala/distilled/`.
* **Pruebas Unitarias:** 62 pruebas unitarias pasando al 100% (`cargo test --lib`).
* **Frontend Build:** `pnpm run build` genera las rutas estáticas con 0 errores de compilación.