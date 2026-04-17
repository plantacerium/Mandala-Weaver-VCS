import type { PolarCoord } from './geometry';

export type MonadKind = 'Function' | 'Struct' | 'Enum' | 'Impl' | 'Trait' | 'Module' | 'Constant' | 'TypeAlias' | 'Unknown';

export type DeltaType = 'Added' | 'Modified' | 'Renamed' | 'Deleted' | 'Unchanged';

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
}

export interface Constellation {
    ring_level: number;
    monads: Monad[];
}

export interface MandalaState {
    bindu_name: string;
    constellations: Constellation[];
}
