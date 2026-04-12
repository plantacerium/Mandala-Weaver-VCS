import type { PolarCoord } from './geometry';

export interface Monad {
    id: string;         // Hash semántico
    coord: PolarCoord;  // Ubicación radial
    content: string;    // Código fuente o metadatos
    name: string;       // Nombre de la función
    ring: number;       // Nivel de expansión
}

export interface Constellation {
    ring_level: number;
    monads: Monad[];
}

export interface MandalaState {
    bindu_name: string;
    constellations: Constellation[];
}
