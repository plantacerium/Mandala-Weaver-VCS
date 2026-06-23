import { invoke } from '@tauri-apps/api/core';
import type { MandalaState, Monad, EdgeDto } from '../../types/ontology';

function isTauri(): boolean {
    return typeof window !== 'undefined' && '__TAURI__' in window;
}

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
    if (!isTauri()) {
        console.log('[DEV] Running outside Tauri — using mock Mandala state with approximate edges');
        return getMockState();
    }

    try {
        const stateStr = await invoke<string>('export_mandala_state');
        return JSON.parse(stateStr) as MandalaState;
    } catch (error) {
        console.warn('[WARN] Failed to fetch from Tauri, falling back to mock:', error);
        return getMockState();
    }
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
