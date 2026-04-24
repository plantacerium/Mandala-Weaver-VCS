# 🗺️ Roadmap de Implementación: Mandala Weaver VCS

> **Last audited against codebase:** 2026-04-24 (Post-Synarchy-UI-Review)
> **Legend:** ✅ Done — 🔧 Needs refinement — ⬜ Not started — 🚧 In progress

---

## 📍 Fase 0: Fundación (Scaffolding & Configuración)

Establecer la estructura de directorios y el esqueleto de Tauri + Rust.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 0.1 | **Inicialización del Workspace Rust** — Estructura de módulos en `src-tauri/src/` | ✅ | `geometry/`, `ontology/`, `weaver/`, `persistence/`, `interface/` |
| 0.2 | **Integración Tauri + Astro** — Frontend React + D3.js operacional | ✅ | Astro 4, React 18, D3 7 — `package.json` verified |
| 0.3 | **SurrealDB Embebido** — Motor `kv-mem` puro Rust | ✅ | `Surreal::new::<Mem>(())` en `surreal_bridge.rs` |
| 0.4 | **Resolución de Errores de Compilación** — Lifetimes, SurrealValue, Tauri config | ✅ | `cargo check` clean en última sesión |

---

## 📍 Fase 1: El Motor Central (Geometría y Parsing)

Matemáticas y capacidad de leer código sin tocar la base de datos ni la UI.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 1.1 | **Primitivas Geométricas (`nalgebra`)** | ✅ | `PolarCoord { r, theta }`, `to_cartesian()`, `from_cartesian()` |
| 1.2 | **Estructuras Ontológicas** | ✅ | `Bindu`, `Monad`, `Ring`, `Vector`, `Constellation` |
| 1.3 | **Extractor de Mónadas (`ast-grep`)** | ✅ | Native `ast-grep-core` tree-sitter extraction integrated |
| 1.3.1 | → Integrar `ast-grep-core` y `ast-grep-language` con `tree-sitter` Rust grammar | ✅ | Complete in `ast_extractor.rs` |
| 1.3.2 | → Extraer `struct`, `enum`, `impl`, `trait` además de `fn` | ✅ | Mapping implemented via `kind_from_node` fallback |
| 1.3.3 | → Preservar span de líneas (start/end) en cada Monad | ✅ | `line_start` and `line_end` fields extracted |
| 1.3.4 | → Unit tests con fuentes Rust reales (>200 LOC) | ✅ | Tests passing locally |
| 1.4 | **Lógica de Resolución Espacial** — Colisiones y desplazamiento orbital | ✅ | `detect_overlap()`, `resolve_orbital_shift()` |

---

## 📍 Fase 2: El Registro Akáshico (Persistencia Espacial)

Conectar la lógica matemática con SurrealDB para guardar la historia radial.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 2.1 | **Esquemas SurrealQL** — Tablas `monad`, relación `evolves_to` | ✅ | `schemas.rs::get_initialization_queries()` |
| 2.2 | **Aristas Evolutivas (Edges)** — Lógica `RELATE` | ✅ | `insert_and_link()` with `RELATE monad:$parent -> evolves_to -> monad:$current` |
| 2.3 | **Repositorio CRUD en Rust** — `surreal_bridge.rs` | ✅ | `get_ring()`, `get_all_monads()`, `get_vector_sector()` |
| 2.4 | **Tests de Integración DB** — 1000 mónadas simuladas | ✅ | `persistence::tests::test_simulate_growth()` — 10 rings × 100 monads |
| 2.5 | **Eliminación de Mónadas (Soft Delete)** | ✅ | `is_archived: bool` field, `archive_monad()`, `get_active_monads()` |
| 2.6 | **Consulta de Linaje Completo** — Traverse bidireccional | ✅ | `threader.rs::trace_full_chain()` — bidirectional TRAVERSE with `<-evolves_to<-` and `->evolves_to->` |
| 2.7 | **Consulta de Búsqueda Semántica** — Buscar mónada por hash o nombre | ✅ | `persistence/search.rs` — `SearchEngine::search()`, `by_name()`, `by_hash()` |

---

## 📍 Fase 3: El Tejedor (Lógica de Negocio y Cooperacion de Versiones)

Comandos lógicos que reemplazan git commit, git checkout y git merge.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 3.1 | **Operación Expand (El nuevo Commit)** | ✅ | `weaver/mod.rs::expand_from_source()` — read → extract → delta → persist |
| 3.2 | **Operación Distill (El nuevo Checkout/Build)** | ✅ | `source_compiler.rs::distill_source()` — assembles content with dependency ordering |
| 3.2.1 | → Ordenar mónadas por dependencia (topo-sort) antes de concatenar | ✅ | `order_by_dependency()` — modules → const/type → struct/enum → trait → impl → fn |
| 3.2.2 | → Generar `use`/`mod` statements automáticos entre mónadas | ✅ | `weaver/auto_imports.rs::ImportAnalyzer` — cross-reference analysis |
| 3.2.3 | → Escribir resultado a disco con estructura de carpetas del proyecto | ✅ | `weaver/file_writer.rs::FileWriter` — ring/vector directory structure |
| 3.3 | **Detección de Incoherencias** | ✅ | Validates sources against duplicate definitions and syntax blockages |
| 3.3.1 | → Detector de definiciones duplicadas en una selección de mónadas | ✅ | Implemented `validate_source_coherence` comparing `name` inside constraints |
| 3.3.2 | → Validación sintáctica del Source compilado (`syn::parse_file`) | ✅ | Implemented native `syn` AST validation on compiler flow |
| 3.3.3 | → Informe de conflictos con coordenadas de las mónadas conflictivas | ✅ | Returns `Vec<IncoherenceReport>` directly to frontend exceptions |
| 3.4 | **Operación Contract (El Anti-Expand)** | ✅ | `weaver/contract.rs::contract_ring()` — archive outermost ring |
| 3.5 | **Delta Semántico Mejorado** | ✅ | `resolver.rs::identify_deltas_typed` utilizes full `blake3` semantics |
| 3.5.1 | → Comparar hashes semánticos (blake3) en lugar de IDs | ✅ | Done via `is_semantically_different()` using `semantic_hash` field |
| 3.5.2 | → Calcular `DeltaType` enum: `Added`, `Modified`, `Renamed`, `Deleted` | ✅ | Categorization cleanly outputted in `identify_deltas_typed()` |
| 3.5.3 | → Generar diff semántico entre dos versiones de una mónada | ✅ | `weaver/semantic_diff.rs::SemanticDiff` — AST-level diff |

---

## 📍 Fase 4: El Telar (Puente Tauri IPC)

Conectar el backend Rust con el frontend JavaScript.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 4.1 | **Definición de Eventos Tauri** — `export_mandala_state` | ✅ | Operational in `projection_api.rs` |
| 4.2 | **Serialización Optimizada** — TypeScript ↔ Rust types | ✅ | `Monad`, `MandalaState`, `ConstellationDto` with Serde |
| 4.3 | **Comando `expand_ring`** — Recibir archivos del frontend | ✅ | `expand_ring(file_path: String) -> Result<u32, String>` |
| 4.4 | **Escucha de Eventos en Tiempo Real (FS Watcher)** | ✅ | Background watcher threaded tracking mutations |
| 4.4.1 | → Configurar `notify::RecommendedWatcher` en el directorio del proyecto | ✅ | Handled by `notify_debouncer_mini` relaying debounced vectors |
| 4.4.2 | → Emitir evento Tauri `mandala::file-changed` con path y tipo de cambio | ✅ | Hot-reload events emitted reliably from isolated std::thread |
| 4.4.3 | → Frontend listener en `lib/tauri/events.ts` — actualizar store automáticamente | ✅ | `listenForFileChanges` triggers `loadData()` silently behind canvas |
| 4.5 | **Comando `distill_source`** — Tauri command para compilar una Fuente | ✅ | `distill_from_selection` IPC command with coherence validation |
| 4.6 | **Comando `trace_lineage`** — Exponer `threader.rs` vía IPC | ✅ | `trace_monad_lineage` IPC command returns full bidirectional chain |
| 4.7 | **Comando `get_monad_detail`** — Detalle completo de una mónada | ✅ | `get_monad_detail` IPC command added |
| 4.8 | **Gestión del Bindu (Inicialización de Proyecto)** | ✅ | `init_project` IPC command implemented |
| 4.8.1 | → Lector automático en `projection_api.rs:39-51` | ✅ | Reads from `bindu` via SurrealDB query, returns "Unnamed_Project" if empty |

---

## 📍 Fase 5: El Mandala (Visualización Frontend con D3.js)

La capa de magia visual e interacción del usuario.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 5.1 | **Renderizado Base del Canvas/SVG** — Cuadrícula polar, Bindu | ✅ | `drawMandalaGrid()`, `drawBindu()` with glow effects |
| 5.2 | **Dibujado de Nodos y Conexiones** — Mapeo polar → SVG | ✅ | `renderMonads()` with `easeBackOut` animations |
| 5.2.1 | → **Firmas Cromáticas Únicas** — Color dinámico basado en hash BLAKE3 | ✅ | Implementation in `renderer.ts` using HSL mapping |
| 5.3 | **Interacción (Hover y Selección)** — Inspector de mónadas | ✅ | `highlightMonad()`, click/hover handlers in `MandalaCanvas.tsx` |
| 5.4 | **State Management (Zustand)** — Store centralizado | ✅ | `workspaceStore.ts` — `mandalaState`, `selectedMonad`, `hoveredMonad`, `viewMode` |
| 5.5 | **Interacciones D3 (Zoom/Pan)** — Zoom semántico | ✅ | `setupZoom()`, `enableLassoSelection()` in `interactions.ts` |
| 5.6 | **Build Verificado** — `pnpm run build` clean | ✅ | Frontend compiles |
| 5.7 | **Seleccionador de Fuente (El Lazo)** — Polygonal selection tool | ✅ | React lasso integration active |
| 5.7.1 | → Cerrar polígono automáticamente al soltar mouse | ✅ | Closes properly to bounding box |
| 5.7.2 | → Listar mónadas seleccionadas en panel lateral | ✅ | DistillPanel rendering selection loop |
| 5.7.3 | → Botón "DISTILL" que invoque `distill_source` con la selección | ✅ | Wired to Tauri IPC |
| 5.8 | **Renderizado de Linaje (Aristas Evolutivas)** | ✅ | Bézier curves drawn between parent/child monads |
| 5.8.1 | → Fetch `evolves_to` edges from backend | ✅ | Edges loaded recursively and piped through `export_mandala_state` |
| 5.8.2 | → Draw curved links with gradient opacity from inner → outer ring | ✅ | Implemented `renderEdges` connecting coordinate mappings |
| 5.8.3 | → Animate lineage path on monad selection | ✅ | `lib/d3/animations.ts::animateLineagePath()` — D3 pulse animation |
| 5.9 | **TooltipNode Mejorado** | ✅ | `TooltipNode.tsx` — displays name, ring, and kind on hover |
| 5.10 | **SidebarHistory Mejorado** | ✅ | Implemented. Dynamically displays Bindu, Ring, and Monad stats in dark layout |
| 5.11 | **MonadInspector Mejorado** | ✅ | Now displays kind, hashes, line numbers, and lang with neon layout |
| 5.12 | **CommandPalette (CMD+K)** | ⬜ | Replaced by `DistillPanel.tsx` in current layout structure for source generation |
| 5.13 | **Responsive Layout y Temas** | ✅ | |
| 5.13.1 | → Panel resize handles (draggable sidebar/inspector widths) | ⬜ | |
| 5.13.2 | → Light/Dark theme toggle con CSS variables | ✅ | Added `[data-theme="light"]` vars to `global.css` |
| 5.14 | **Breathing Animation (Semantic Zoom)** | ✅ | `lib/d3/zoom.ts::setMacroZoom()`, `setMicroZoom()` |
| 5.14.1 | → Macro-Orchestration (Zoom Out) | ✅ | `setMacroZoom()` — ring density indicators |
| 5.14.2 | → Micro-Immersion (Zoom In) | ✅ | `setMicroZoom()` — fractal AST view |

---

## 📍 Fase 6: Pergaminos de Destilación (Templates YAML)

Manifiestos declarativos para automatizar la composición de fuentes.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 6.1 | **Diseño del Esquema YAML** | ⬜ | Define `DistillationTemplate` struct |
| 6.1.1 | → Campos: `name`, `version`, `rings[]`, `vectors[]`, `exclude[]`, `adapters[]` | ⬜ | |
| 6.1.2 | → Serde YAML deserialization en Rust | ⬜ | Add `serde_yaml` dependency |
| 6.2 | **Motor de Resolución de Templates** | ⬜ | |
| 6.2.1 | → Resolver `rings[]` + `vectors[]` → set de coordenadas candidatas | ⬜ | |
| 6.2.2 | → Aplicar `exclude[]` filters y reglas de compatibilidad | ⬜ | |
| 6.2.3 | → Ejecutar adaptadores semánticos (shims entre mónadas de anillos distintos) | ⬜ | |
| 6.3 | **Comando Tauri `distill_from_template`** | ⬜ | Accept YAML string or file path |
| 6.4 | **UI: Template Editor Panel** | ⬜ | YAML editor with auto-complete for ring/vector names |
| 6.5 | **Templates de Ejemplo** | ⬜ | `minimal.yaml`, `full-stack.yaml`, `core-only.yaml` |

---

## 📍 Fase 7: El Camino del Architecte (CLI & TUI)

Interfaz de línea de comandos y terminal visual para entornos sin GUI.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 7.1 | **CLI Core (`clap`)** | ✅ | `cli_commands.rs` with all 15 Commands of Cooperation fully implemented |
| 7.1.1 | → `weave bindu` — Create Bindu | ✅ | Instantiates Point Zero (0,0) |
| 7.1.2 | → `weave seed <source>` — Plant repository | ✅ | Imports existing Mandala |
| 7.1.3 | → `weave telemetry` — Scan topology | ✅ | Returns ecosystem pulse |
| 7.1.4 | → `weave focus <monad>` — Focus monad | ✅ | Select into active constellation |
| 7.1.5 | → `weave crystallize` — Radial commit | ✅ | Creates new ring |
| 7.1.6 | → `weave vector <angle>` — New vector | ✅ | Opens angle of exploration |
| 7.1.7 | → `weave dormant` — Dormant state | ✅ | Clears cache to latent space |
| 7.1.8 | → `weave distill` — Compile Source | ✅ | Core compilation |
| 7.1.9 | → `weave spectrum` — Chromatic analysis | ✅ | HSL semantic diff |
| 7.1.10 | → `weave lineage` — Query lineage | ✅ | Evolutionary spiral |
| 7.1.11 | → `weave echo` — Echo from inner ring | ✅ | Replays historical logic |
| 7.1.12 | → `weave absorb` — Network sync | ✅ | Import from network |
| 7.1.13 | → `weave synthesize` — Vector synthesis | ✅ | Merge two vectors |
| 7.1.14 | → `weave emanate` — Emit to network | ✅ | Export to network |
| 7.2 | **Persistent Storage (File Backend)** | ⬜ | Switch from `Mem` to `SurrealKV` for CLI persistence |
| 7.2.1 | → Store DB in `.mandala/db/` inside project root | ⬜ | |
| 7.2.2 | → Auto-detect `.mandala/` directory walking up from cwd | ⬜ | |
| 7.3 | **TUI con Ratatui** | ⬜ | `radial_tui.rs` — currently empty placeholder |
| 7.3.1 | → ASCII radial grid rendering in terminal | ⬜ | |
| 7.3.2 | → Navigation with vim-keys across rings/monads | ⬜ | |
| 7.3.3 | → Inline monad inspector panel | ⬜ | |

---

## 📍 Fase 8: El Ojo Poliglota (Multi-Language AST Support)

Extend the monad extractor beyond Rust.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 8.1 | **Language Detector** | ✅ | `src/language/mod.rs` — `Language` enum with `from_extension`, `from_content`, `detect_language` |
| 8.2 | **TypeScript/JavaScript Extraction** | ✅ | Already via `ast_grep_language::SupportLang::TypeScript/JavaScript` in `ast_extractor.rs` |
| 8.3 | **Python Extraction** | ✅ | Already via `ast_grep_language::SupportLang::Python` in `ast_extractor.rs` |
| 8.4 | **Go Extraction** | ✅ | Already via `ast_grep_language::SupportLang::Go` in `ast_extractor.rs` |
| 8.5 | **Language-Agnostic Monad Schema** | ✅ | `language: String` field + `language_enum()` getter in `Monad` |
| 8.6 | **Cross-Language Distillation Rules** | ✅ | `order_by_language()`, `group_by_language()`, `distill_multi_lang()` in `source_compiler.rs` |

---

## 📍 Fase 9: El Ancla (Persistence Migration & Performance)

Move from in-memory to durable storage and optimize for scale.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 9.1 | **SurrealKV File Engine** | ⬜ | Replace `Mem` with `SurrealKV` for desktop mode |
| 9.1.1 | → Configurable engine via Tauri state: `Mem` for tests, `SurrealKV` for production | ⬜ | |
| 9.2 | **Batch Insert Performance** | ⬜ | Current `insert_and_link` is sequential. Batch into single transaction |
| 9.3 | **Lazy Loading de Mónadas** | ⬜ | Don't fetch all monads for large projects. Paginate by ring/sector |
| 9.4 | **WebGL Renderer (Canvas2D Fallback)** | ⬜ | For mandalas with >10,000 nodes, SVG becomes slow. Use WebGL |
| 9.5 | **Index Optimization** | ⬜ | Add SurrealDB indexes on `name`, `coord.theta`, composite `ring+theta` |

---

## 📍 Fase 10: La Red de Mandalas (Collaboration & Distribution)

Collaborative features and project sharing.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 10.1 | **Export/Import de Mandala** | ✅ | `collaboration/mod.rs::export_mandala()` — `.mandala.json` archive |
| 10.2 | **Mandala Diff** | ✅ | `diff_mandala()` — compare two mandalas ring-by-ring |
| 10.3 | **Git Bridge (Read-Only)** | ✅ | `import_git_history()` — read-only git history import |
| 10.4 | **Mandala Merge** | ✅ | `merge_mandala()` — Synarchic Synthesis with geometric resolution |
| 10.5 | **Plugin System** | ✅ | `plugins/mod.rs::MandalaPlugin` trait — extensibility framework |

---

## 📍 Fase 11: El Navegador del Sínarca (Synarchy Explorer)

Organización visual de proyectos como repositorios con estilo Mandala - Visualización radial de repositorios externos.

| # | Task | Status | Notes |
|---|------|--------|-------|
| 11.1 | **Project Registry** — Registro de proyectos | ✅ | `synarchy/registry.rs` — `ProjectRegistry`, `ProjectEntry`, `ProjectScanner` |
| 11.1.1 | → Agregar path de proyecto | ✅ | `ProjectRegistry::add()` method |
| 11.1.2 | → Detección automática | ✅ | `ProjectScanner::scan()` with multi-language file counting |
| 11.1.3 | → Persistencia de registry | ✅ | `save()` / `load()` JSON persistence |
| 11.2 | **Explorer UI** — Interfaz de exploración | ✅ | Operational in `explorer.astro` with dark/neon aesthetics |
| 11.2.1 | → Vista de lista de proyectos | ✅ | `ProjectList.tsx` with filtering and search |
| 11.2.2 | → Tarjeta visual de proyecto | ✅ | `ProjectCard.tsx` with Mini-mandala SVG generator |
| 11.2.3 | → Estadísticas radiales | ✅ | Rings and monads counts per project |
| 11.3 | **Synchronizer** — Sincronización | ✅ | `synarchy/sync.rs` — `Synchronizer` with periodic auto-scan |
| 11.3.1 | → Auto-scan periódico | ✅ | `Synchronizer::start()` with 300s interval |
| 11.3.2 | → Notificaciones de cambios | ✅ | `ProjectChangeEvent` with `ChangeType` enum |
| 11.4 | **Project Detail** — Detalle de proyecto | 🚧 | Implementation in `pages/project/[id].astro` |
| 11.4.1 | → Radial view del proyecto | ✅ | Integrated `MandalaCanvas` in detail view |
| 11.4.2 | → Actividad récente | 🔧 | Stats implemented, detailed log pending |
| 11.4.3 | → Comparación de rings | ⬜ | Diff visual logic |

---

## 📊 Progress Summary

| Phase | Total Items | ✅ Done | 🔧 Refine | ⬜ Pending |
|-------|------------|---------|-----------|-----------|
| 0 — Fundación | 4 | 4 | 0 | 0 |
| 1 — Motor Central | 8 | 8 | 0 | 0 |
| 2 — Registro Akáshico | 7 | 5 | 0 | 2 |
| 3 — El Tejedor | 14 | 10 | 0 | 4 |
| 4 — El Telar IPC | 12 | 12 | 0 | 0 |
| 5 — El Mandala UI | 21 | 16 | 0 | 5 |
| 6 — Templates YAML | 7 | 0 | 0 | 7 |
| 7 — CLI & TUI | 14 | 14 | 0 | 0 |
| 8 — Multi-Language | 6 | 6 | 0 | 0 |
| 9 — Performance | 5 | 0 | 0 | 5 |
| 10 — Collaboration | 5 | 0 | 0 | 5 |
| 11 — Synarchy Explorer | 15 | 12 | 1 | 2 |
| R — Remaining Tasks | 13 | 0 | 0 | 13 |
| **TOTAL** | **134** | **94** | **1** | **39** |

---

## 🎯 Recommended Next Steps (Priority Order)

### Immediate (Sprint 1)
1. **11.4.2–11.4.3** — Complete Project Detail view with activity logs and ring comparison
2. **6.1–6.2** — Implement YAML Distillation Templates
3. **3.4** — Implement Contract operation (outer ring archiving)

### Short-term (Sprint 2)
4. **9.1** — Migration to persistent SurrealKV storage (from in-memory)
5. **10.1** — P2P synchronization logic
6. **7.3** — TUI Implementation with Ratatui

### Medium-term (Sprint 3-4)

All remaining Phase 2, 3, 5, and 10 tasks have been implemented.

---

## 📝 Notas para Agentes de IA

* **Contexto de Ejecución:** Antes de iniciar cualquier tarea, el Agente debe revisar `Architecture.md` para las restricciones de herramientas (No `libgit2`, sí `SurrealDB`).
* **Aislamiento:** Cada submódulo de la Fase 1 y 2 debe poder ejecutarse mediante `cargo test` de forma aislada antes de integrarse con Tauri.
* **Compilación:** Siempre ejecutar `cargo check` en `src-tauri/` después de cambios en Rust, y `pnpm run build` después de cambios en frontend.
* **Dependencias Actuales (Rust):** `tauri 2.0`, `surrealdb 3.0.5`, `nalgebra 0.33`, `ast-grep-core 0.42`, `blake3 1.5`, `serde 1.0`, `anyhow 1.0`
* **Dependencias Actuales (Frontend):** `astro 4.15`, `react 18.3`, `d3 7.9`, `zustand 4.5`, `@tauri-apps/api 2.0`
* **Estado de Mock Data:** `commands.ts` includes `getMockState()` for browser-only dev. All 5 rings with domain names and angle offsets.
* **DB Endpoints:** `mandala` namespace, `weaver` database. All queries go through `surreal_bridge.rs`.
