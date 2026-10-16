import { invoke } from '@tauri-apps/api/core';
import type { MandalaState, Monad, EdgeDto, DistilledArtifact } from '../../types/ontology';

import { isTauri } from './isTauri';

// ─── Browser-only dev mock ────────────────────────────────────────────────────
// Used ONLY when running outside Tauri (e.g. `astro dev` in browser).
// All evolutionary edges here are positional approximations for visual testing.
// In the real Tauri app, edges come from the `evolves_to` graph in SurrealDB.
function getMockState(): MandalaState {
    const monads: any[] = [];
    const ringCounts = [3, 5, 8, 4, 6];
    const ringNames = ['Core', 'UI', 'Utils', 'Network', 'Data'];

    for (let ring = 1; ring <= 5; ring++) {
        const count = ringCounts[ring - 1] || 3;
        for (let i = 0; i < count; i++) {
            const angle = (360 / count) * i + (ring * 15);
            const kinds = ['Function', 'Struct', 'Module', 'Unknown'];
            const kind = kinds[i % kinds.length] as any;

            monads.push({
                id: `monad-${ring}-${i}`,
                semantic_hash: `hash_${ring}_${i}_${Math.random().toString(16).slice(2)}`,
                name: `${kind === 'Struct' ? 'Struct_' : kind === 'Module' ? 'Mod_' : 'fn_'}${ringNames[ring - 1].toLowerCase()}_${i}`,
                coord: { r: ring * 80, theta: angle },
                ring,
                kind,
                language: 'rust',
                line_start: i * 10,
                line_end: i * 10 + 8,
                content: `// Ring ${ring} - ${ringNames[ring - 1]}\nexport const example_${i} = () => {\n  return true;\n}`
            });
        }
    }

    const constellations: any[] = [];
    let prevRingMonads: any[] = [];
    // Mock edges: positional approximation only — NOT real evolves_to graph data
    const edges: EdgeDto[] = [];
    for (let ring = 1; ring <= 5; ring++) {
        const ringMonads = monads.filter(m => m.ring === ring);
        constellations.push({ ring_level: ring, monads: ringMonads });

        if (prevRingMonads.length > 0) {
            ringMonads.forEach((m, idx) => {
                const parent = prevRingMonads[idx % prevRingMonads.length];
                edges.push({ parent_id: parent.id, child_id: m.id });
            });
        }
        prevRingMonads = ringMonads;
    }

    return { bindu_name: 'Genesis_Project [DEV MOCK]', constellations, edges };
}

// ─── IPC Commands ─────────────────────────────────────────────────────────────

/**
 * Fetches the complete spatial state of the Mandala from the Rust backend.
 * In Tauri: calls `export_mandala_state` — edges are real `evolves_to` graph data from SurrealDB.
 * In browser: returns dev mock with positional edge approximations.
 */
export async function fetchMandalaState(): Promise<MandalaState> {
    if (isTauri()) {
        try {
            const stateStr = await invoke<string>('export_mandala_state');
            const parsed = JSON.parse(stateStr) as MandalaState;
            if (parsed && parsed.total_monads > 0) {
                return parsed;
            }
        } catch (error) {
            console.warn('[WARN] Failed to fetch from Tauri IPC:', error);
        }
    }

    // Attempt to load the real exported project state from /mandala_state.json
    // This allows both regular browser preview (Chrome/Edge) and Tauri fallback
    // to render the REAL repository monads (336 nodes) instead of the 26-node mock!
    try {
        const res = await fetch('/mandala_state.json');
        if (res.ok) {
            const staticState = await res.json() as MandalaState;
            if (staticState && staticState.constellations && staticState.constellations.length > 0) {
                console.log(`[INFO] Loaded real Mandala state (${staticState.total_monads} monads) from mandala_state.json`);
                return staticState;
            }
        }
    } catch (e) {
        console.warn('[WARN] Failed to load static mandala_state.json:', e);
    }

    console.log('[DEV] Running outside Tauri with no static state — using fallback mock');
    return getMockState();
}

/**
 * Fetches the full evolutionary lineage chain for a monad (real SurrealDB graph traversal).
 * Calls `trace_monad_lineage` → `threader::trace_full_chain()` → bidirectional TRAVERSE
 * via `<-evolves_to<-` and `->evolves_to->` in SurrealDB.
 *
 * Returns an ordered array of ancestor/descendant monads + the total chain depth.
 */
export async function fetchLineageEdges(monadId: string): Promise<{ monads: Monad[]; depth: number }> {
    if (!isTauri()) {
        console.log(`[DEV] Mock lineage for monad: ${monadId}`);
        return { monads: [], depth: 0 };
    }

    try {
        const resultStr = await invoke<string>('trace_monad_lineage', { monadId });
        return JSON.parse(resultStr) as { monads: Monad[]; depth: number };
    } catch (error) {
        console.warn('[WARN] Failed to fetch lineage from Tauri:', error);
        return { monads: [], depth: 0 };
    }
}

/**
 * Sends an expand (commit) command to create a new ring from a source file.
 */
export async function invokeExpand(filePath: string): Promise<number> {
    if (!isTauri()) {
        console.log('[DEV] Mock expand — returning ring 6');
        return 6;
    }

    try {
        return await invoke<number>('expand_ring', { filePath });
    } catch (error) {
        console.warn('[WARN] Failed to invoke expand:', error);
        return 6;
    }
}

/**
 * Opens a project by pointing the database to its persistent `.mandala/db`.
 */
export async function openProject(projectPath: string): Promise<string> {
    if (!isTauri()) {
        console.log(`[DEV] Mock openProject: ${projectPath}`);
        return 'Project opened (mock)';
    }
    return await invoke<string>('open_project', { projectPath });
}

/**
 * Imports Git history as evolutionary concentric rings.
 */
export async function importGitRepository(repoPath: string): Promise<number> {
    if (!isTauri()) {
        console.log(`[DEV] Mock importGitRepository: ${repoPath}`);
        return 3;
    }
    return await invoke<number>('import_git_repository', { repoPath });
}

/**
 * Fetches monads using optimized database pagination (Phase 9.3).
 */
export async function getMonadsPaginated(
    page: number,
    pageSize: number = 50,
    ringFilter?: number
): Promise<Monad[]> {
    if (!isTauri()) {
        return [];
    }
    return await invoke<Monad[]>('get_monads_paginated_api', {
        page,
        pageSize,
        ringFilter: ringFilter ?? null,
    });
}

/**
 * Executes a spatial distillation using a YAML recipe (Weaver Espacial).
 * Traverses the tripartite graph, resolves contracts, and injects bridge monads when needed.
 */
export async function distillSpatialRecipe(recipeYaml: string): Promise<DistilledArtifact> {
    if (isTauri()) {
        return await invoke<DistilledArtifact>('distill_spatial_recipe', { recipeYaml });
    }
    return {
        recipe_name: 'browser_mock',
        assembled_source: '// Spatial Distillation Mock (Browser Mode)\n// Demonstrating Weaver Espacial with SO(2) continuous polar guidance\nfn mock_spatial_distillation() {\n    println!("Spatial Weaver Active");\n}',
        traces: [],
        bridge_traces: [],
    };
}

/**
 * Quick spatial distillation using angular direction and max ring from UI controls.
 */
export async function distillSpatialQuick(
    name: string,
    targetAngleDeg: number,
    maxRing: number,
    seeds: string[]
): Promise<DistilledArtifact> {
    if (isTauri()) {
        return await invoke<DistilledArtifact>('distill_spatial_quick', {
            name,
            targetAngleDeg,
            maxRing,
            seeds,
        });
    }
    return {
        recipe_name: name,
        assembled_source: `// Quick Spatial Distillation for angle ${targetAngleDeg.toFixed(1)}° (Max Ring ${maxRing})\n// Seeds: ${seeds.join(', ')}\nfn main() {}`,
        traces: [],
        bridge_traces: [],
    };
}

/**
 * Searches monads by name, kind, language, or content via Tauri `search_monads` or local state fallback.
 */
export async function searchMonads(query: string, fallbackMonads: Monad[] = []): Promise<Monad[]> {
    const trimmed = query.trim();
    if (!trimmed) {
        return fallbackMonads.slice(0, 30);
    }

    if (isTauri()) {
        try {
            const results = await invoke<Monad[]>('search_monads', { query: trimmed });
            if (results && results.length > 0) {
                return results;
            }
        } catch (err) {
            console.warn('[WARN] search_monads IPC failed, using local fallback:', err);
        }
    }

    const q = trimmed.toLowerCase();
    return fallbackMonads.filter(
        (m) =>
            m.name.toLowerCase().includes(q) ||
            (m.kind && m.kind.toLowerCase().includes(q)) ||
            (m.language && m.language.toLowerCase().includes(q)) ||
            (m.content && m.content.toLowerCase().includes(q)) ||
            `r${m.ring}` === q
    );
}

/**
 * Contracts (archives) the outermost active ring.
 */
export async function contractOuterRing(): Promise<number> {
    if (!isTauri()) {
        return 0;
    }
    return await invoke<number>('contract_outer_ring');
}

/**
 * Reverts (archives) all rings above `ring`.
 */
export async function revertToLevel(ring: number): Promise<number> {
    if (!isTauri()) {
        return 0;
    }
    return await invoke<number>('revert_to_level', { ring });
}

/**
 * Returns the count of archived (latent) monads in SurrealDB.
 */
export async function getArchivedMonadsCount(): Promise<number> {
    if (!isTauri()) {
        return 0;
    }
    try {
        return await invoke<number>('get_archived_monads_count');
    } catch {
        return 0;
    }
}

/**
 * Permanently purges archived (latent) monads from SurrealDB.
 */
export async function purgeArchivedMonads(): Promise<number> {
    if (!isTauri()) {
        return 0;
    }
    return await invoke<number>('purge_archived_monads');
}

