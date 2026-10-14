import type { PolarCoord } from './geometry';

export type MonadKind = 'Function' | 'Struct' | 'Enum' | 'Impl' | 'Trait' | 'Module' | 'Constant' | 'TypeAlias' | 'Unknown';

export type DeltaType = 'Added' | 'Modified' | 'Renamed' | 'Deleted' | 'Unchanged';

export interface SemanticIdentity {
    contract_hash: string;
    implementation_hash: string;
    canonical_signature: string;
}

export interface Monad {
    id: string;              // Hash semántico
    coord: PolarCoord;       // Ubicación radial
    content: string;         // Código fuente original
    name: string;            // Nombre (ej. nombre de funcion)
    ring: number;            // Nivel de expansión
    kind: MonadKind;         // Tipo
    semantic_hash: string;   // Hash ignorando espacios
    line_start: number;
    line_end: number;
    language: string;
    identity?: SemanticIdentity;
}

export interface EdgeDto {
    parent_id: string;
    child_id: string;
}

export interface Constellation {
    ring_level: number;
    monads: Monad[];
}

export interface MandalaState {
    bindu_name: string;
    constellations: Constellation[];
    edges: EdgeDto[];
    total_monads?: number;
    max_ring?: number;
}

export interface WovenNodeTrace {
    symbol_name: string;
    implementation_hash: string;
    contract_hash: string;
    selected_ring: number;
    selected_angle_rad: number;
    angular_distance_rad: number;
    alternatives_considered: number;
}

export interface WovenBridgeTrace {
    caller_symbol: string;
    target_symbol: string;
    bridge_alias: string;
    bridge_impl_hash: string;
    from_expected_contract: string;
    to_actual_contract: string;
    is_lossy: boolean;
}

export interface DistilledArtifact {
    recipe_name: string;
    assembled_source: string;
    traces: WovenNodeTrace[];
    bridge_traces: WovenBridgeTrace[];
}
