import { invoke } from '@tauri-apps/api/core';
import type { MandalaState } from '../../types/ontology';

function isTauri(): boolean {
    return typeof window !== 'undefined' && '__TAURI__' in window;
}

function getMockState(): MandalaState {
    const monads = [];
    const ringCounts = [3, 5, 8, 4, 6];
    const ringNames = ['Core', 'UI', 'Utils', 'Network', 'Data'];
    
    for (let ring = 1; ring <= 5; ring++) {
        const count = ringCounts[ring - 1] || 3;
        for (let i = 0; i < count; i++) {
            const angle = (360 / count) * i + (ring * 15);
            // Alternate kinds for visual variety in dev mode
            const kinds = ['Function', 'Struct', 'Module', 'Unknown'];
            const kind = kinds[i % kinds.length] as any;
            
            monads.push({
                id: `monad-${ring}-${i}`,
                semantic_hash: `hash_${ring}_${i}_${Math.random().toString(16).slice(2)}`,
                name: `${kind === 'Struct' ? 'Struct_' : kind === 'Module' ? 'Mod_' : 'fn_'}${ringNames[ring - 1].toLowerCase()}_${i}`,
                coord: { r: ring * 80, theta: angle },
                ring: ring,
                kind: kind,
                language: 'rust',
                line_start: i * 10,
                line_end: i * 10 + 8,
                content: `// Ring ${ring} - ${ringNames[ring - 1]}\nexport const example_${i} = () => {\n  return true;\n}`
            });
        }
    }
    
    const constellations = [];
    let prevRingMonads = [];
    const edges = [];
    for (let ring = 1; ring <= 5; ring++) {
        const ringMonads = monads.filter(m => m.ring === ring);
        constellations.push({ ring_level: ring, monads: ringMonads });
        
        // Generate some evolutionary edges between ring N and ring N+1
        if (prevRingMonads.length > 0) {
            ringMonads.forEach((m, idx) => {
                // Link each monad to a somewhat corresponding parent in the previous ring
                const parent = prevRingMonads[idx % prevRingMonads.length];
                edges.push({
                    parent_id: parent.id,
                    child_id: m.id
                });
            });
        }
        prevRingMonads = ringMonads;
    }
    
    return { bindu_name: 'Genesis_Project', constellations, edges };
}

/// Pide a Rust el estado espacial completo del Mandala.
export async function fetchMandalaState(): Promise<MandalaState> {
    if (!isTauri()) {
        console.log('Running outside Tauri - using mock data');
        return getMockState();
    }

    try {
        const stateStr = await invoke<string>('export_mandala_state');
        return JSON.parse(stateStr);
    } catch (error) {
        console.warn('Failed to fetch from Tauri, using mock data:', error);
        return getMockState();
    }
}

/// Envía un comando para expandir (hacer commit) un nuevo anillo.
export async function invokeExpand(filePath: string): Promise<number> {
    if (!isTauri()) {
        console.log('Running outside Tauri - mock expand');
        return 6;
    }

    try {
        return await invoke<number>('expand_ring', { filePath });
    } catch (error) {
        console.warn('Failed to invoke expand:', error);
        return 6;
    }
}
