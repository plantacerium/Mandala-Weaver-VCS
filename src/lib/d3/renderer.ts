import * as d3 from 'd3';
import type { Monad, EdgeDto } from '../../types/ontology';

const COLORS = {
    primary: '#00e5ff',
    primaryLight: 'rgba(0, 229, 255, 0.5)',
    secondary: '#9d4edd',
    tertiary: '#ff3e83',
    quaternary: '#00ff88',
    ringStroke: 'rgba(255, 255, 255, 0.1)',
    ringFill: 'rgba(255, 255, 255, 0.02)',
};

const RING_COLORS = [
    'rgba(0, 229, 255, 0.08)',
    'rgba(157, 78, 221, 0.08)',
    'rgba(255, 62, 131, 0.08)',
    'rgba(0, 255, 136, 0.08)',
    'rgba(251, 191, 36, 0.06)',
    'rgba(248, 113, 113, 0.05)',
];

function getMonadColor(hash: string): string {
    if (!hash) return COLORS.primary;
    
    // DJB2 hash algorithm for better distribution
    let h = 0;
    for (let i = 0; i < hash.length; i++) {
        h = ((h << 5) - h) + hash.charCodeAt(i);
        h |= 0; // Convert to 32bit integer
    }
    
    // Map to Hue (0-360)
    const hue = Math.abs(h) % 360;
    // Keep it bright and saturated for the "Glow" aesthetic
    return `hsl(${hue}, 85%, 60%)`;
}

export const PHI = 1.618033988749895;
export const PHI_CONJUGATE = 0.618033988749895; // 1 / φ
export const GOLDEN_ANGLE_DEG = 137.50776405003785; // 360 / φ²
export const BINDU_SPHERE_RADIUS = 14;
export const BINDU_SANCTUARY_RADIUS = 26;

export interface GoldenRingBand {
    rMin: number;
    center: number;
    rMax: number;
    span: number;
}

/**
 * Computes the exact Golden Ratio annular band [rMin, center, rMax] for ring `ringLevel` (1..maxRings).
 * - Active Vogel seed zone occupies 1/φ (61.8%) of the ring span.
 * - Inter-ring breathing corridor occupies 1 - 1/φ (38.2%) of the ring span, preventing ring collisions.
 */
export function getGoldenRingBand(
    ringLevel: number,
    maxRadius: number = 320,
    maxRings: number = 5
): GoldenRingBand {
    const totalRings = Math.max(2, maxRings);
    const clampedRing = Math.max(1, Math.min(ringLevel, totalRings));
    const rStart = Math.max(BINDU_SANCTUARY_RADIUS + 18, maxRadius * 0.14);
    const rEnd = maxRadius * 0.92;
    const usableSpan = Math.max(100, rEnd - rStart);

    const weights: number[] = [];
    let weightSum = 0;
    for (let k = 1; k <= totalRings; k++) {
        const w = Math.pow(PHI, (k - 1) * 0.32);
        weights.push(w);
        weightSum += w;
    }

    let cursor = rStart;
    for (let k = 1; k <= totalRings; k++) {
        const span = usableSpan * (weights[k - 1] / weightSum);
        const center = cursor + span * 0.5;
        const halfActive = (span / PHI) * 0.5; // 61.8% active sunflower band
        if (k === clampedRing) {
            return {
                rMin: center - halfActive,
                center,
                rMax: center + halfActive,
                span,
            };
        }
        cursor += span;
    }

    return { rMin: rStart, center: rStart + 20, rMax: rStart + 40, span: 40 };
}

export function getGoldenRingRadius(ringLevel: number, maxRadius: number = 320, maxRings: number = 5): number {
    return Math.round(getGoldenRingBand(ringLevel, maxRadius, maxRings).center);
}

interface RingConfig {
    baseRadius: number;
    ringGap: number;
    ringCount: number;
}

export const config: RingConfig = {
    baseRadius: 55,
    ringGap: 89,
    ringCount: 5,
};

function getRingColor(ringLevel: number): string {
    return RING_COLORS[(ringLevel - 1) % RING_COLORS.length];
}

export function drawMandalaGrid(
    svg: d3.Selection<SVGGElement, unknown, null, undefined>,
    maxRadius: number,
    activeRingCount: number = 5
) {
    const ringsGroup = svg.append('g').attr('class', 'rings-group');
    const spiralsGroup = svg.append('g').attr('class', 'spirals-group');
    const sectorsGroup = svg.append('g').attr('class', 'sectors-group');
    const count = Math.max(activeRingCount, 4);

    const firstBand = getGoldenRingBand(1, maxRadius, count);
    const lastBand = getGoldenRingBand(count, maxRadius, count);

    for (let i = 1; i <= count; i++) {
        const band = getGoldenRingBand(i, maxRadius, count);
        const activeWidth = Math.max(4, band.rMax - band.rMin);

        // Annular Golden Band (subtle hollow torus instead of overlapping filled disks)
        ringsGroup.append('circle')
            .attr('class', `ring-aura ring-aura-${i}`)
            .attr('r', band.center)
            .attr('fill', 'none')
            .attr('stroke', getRingColor(i))
            .attr('stroke-width', activeWidth)
            .attr('stroke-opacity', 0.45);

        // Inner and outer golden corridor boundaries
        ringsGroup.append('circle')
            .attr('r', band.rMin)
            .attr('fill', 'none')
            .attr('stroke', 'rgba(255, 255, 255, 0.04)')
            .attr('stroke-width', 0.5)
            .attr('stroke-dasharray', '2,6');

        ringsGroup.append('circle')
            .attr('r', band.rMax)
            .attr('fill', 'none')
            .attr('stroke', 'rgba(255, 255, 255, 0.05)')
            .attr('stroke-width', 0.5)
            .attr('stroke-dasharray', '2,6');

        // Fine concentric centerline ring
        ringsGroup.append('circle')
            .attr('class', `ring ring-${i}`)
            .attr('r', band.center)
            .attr('fill', 'none')
            .attr('stroke', COLORS.ringStroke)
            .attr('stroke-dasharray', i % 2 === 0 ? '4,4' : 'none')
            .attr('stroke-width', 0.8);
    }

    // Subtle Fibonacci Parastichy Spirals (21 clockwise & 13 counter-clockwise sunflower arms)
    // Starting strictly outside the Bindu Sanctuary so nothing crosses (0,0)
    const spiralLine = d3.line<[number, number]>().curve(d3.curveBasis);
    const r0 = firstBand.rMin;
    const r1 = lastBand.rMax;
    const fibonacciArms = 13;

    for (let arm = 0; arm < fibonacciArms; arm++) {
        const baseAngle = (arm * 360) / fibonacciArms;
        const cwPoints: [number, number][] = [];
        const ccwPoints: [number, number][] = [];
        const steps = 16;

        for (let s = 0; s <= steps; s++) {
            const t = s / steps;
            const r = Math.sqrt(r0 * r0 + t * (r1 * r1 - r0 * r0));
            const twist = t * (360 / PHI); // Golden rotation sweep
            const radCw = ((baseAngle + twist) * Math.PI) / 180;
            const radCcw = ((baseAngle - twist / PHI) * Math.PI) / 180;
            cwPoints.push([r * Math.cos(radCw), r * Math.sin(radCw)]);
            ccwPoints.push([r * Math.cos(radCcw), r * Math.sin(radCcw)]);
        }

        spiralsGroup.append('path')
            .attr('d', spiralLine(cwPoints) || '')
            .attr('fill', 'none')
            .attr('stroke', COLORS.primary)
            .attr('stroke-width', 0.45)
            .attr('stroke-opacity', 0.055);

        spiralsGroup.append('path')
            .attr('d', spiralLine(ccwPoints) || '')
            .attr('fill', 'none')
            .attr('stroke', COLORS.secondary)
            .attr('stroke-width', 0.45)
            .attr('stroke-opacity', 0.045);
    }

    // Sacred Golden Pentagonal domain sectors: 360° / 5 = 72° plus 36° sub-guides
    // Starts strictly at BINDU_SANCTUARY_RADIUS so lines NEVER cross over the Bindu (0,0)
    const goldenSectors = Array.from({ length: 10 }, (_, i) => i * 36);

    goldenSectors.forEach((angle) => {
        const rad = (angle * Math.PI) / 180;
        const isMainDomain = angle % 72 === 0;

        sectorsGroup.append('line')
            .attr('x1', BINDU_SANCTUARY_RADIUS * Math.cos(rad))
            .attr('y1', BINDU_SANCTUARY_RADIUS * Math.sin(rad))
            .attr('x2', maxRadius * 0.94 * Math.cos(rad))
            .attr('y2', maxRadius * 0.94 * Math.sin(rad))
            .attr('stroke', isMainDomain ? COLORS.secondary : COLORS.primary)
            .attr('stroke-width', isMainDomain ? 1.0 : 0.4)
            .attr('stroke-dasharray', isMainDomain ? 'none' : '2,6')
            .attr('stroke-opacity', isMainDomain ? 0.28 : 0.10);
    });
}

export function drawBindu(svg: d3.Selection<SVGGElement, unknown, null, undefined>) {
    const binduGroup = svg.append('g').attr('class', 'bindu-group');

    let defs = svg.select<SVGDefsElement>('defs');
    if (defs.empty()) {
        defs = svg.append('defs');
    }

    if (defs.select('#bindu-sphere-radial').empty()) {
        const radGrad = defs.append('radialGradient')
            .attr('id', 'bindu-sphere-radial')
            .attr('cx', '50%')
            .attr('cy', '50%')
            .attr('r', '55%');
        radGrad.append('stop').attr('offset', '0%').attr('stop-color', '#ffffff').attr('stop-opacity', 0.35);
        radGrad.append('stop').attr('offset', '35%').attr('stop-color', '#00f0ff').attr('stop-opacity', 0.16);
        radGrad.append('stop').attr('offset', '75%').attr('stop-color', '#ff2d55').attr('stop-opacity', 0.06);
        radGrad.append('stop').attr('offset', '100%').attr('stop-color', '#00f0ff').attr('stop-opacity', 0);

        const coronaGrad = defs.append('radialGradient')
            .attr('id', 'bindu-corona-radial')
            .attr('cx', '50%')
            .attr('cy', '50%')
            .attr('r', '50%');
        coronaGrad.append('stop').attr('offset', '0%').attr('stop-color', '#00f0ff').attr('stop-opacity', 0.12);
        coronaGrad.append('stop').attr('offset', '55%').attr('stop-color', '#ff2d55').attr('stop-opacity', 0.05);
        coronaGrad.append('stop').attr('offset', '100%').attr('stop-color', '#7928ca').attr('stop-opacity', 0);
    }

    // 1. Soft Sanctuary Aura (No edge or node ever crosses inside BINDU_SANCTUARY_RADIUS)
    binduGroup.append('circle')
        .attr('class', 'bindu-corona')
        .attr('r', BINDU_SANCTUARY_RADIUS)
        .attr('fill', 'url(#bindu-corona-radial)')
        .attr('stroke', 'none');

    // 2. Subtle Translucent Sphere around the Bindu (0,0) without distracting ellipses
    binduGroup.append('circle')
        .attr('class', 'bindu-sphere')
        .attr('r', 11)
        .attr('fill', 'url(#bindu-sphere-radial)')
        .attr('stroke', 'rgba(0, 240, 255, 0.22)')
        .attr('stroke-width', 0.65);

    // 3. Point Zero Singularity - The True Bindu at (0, 0)
    binduGroup.append('circle')
        .attr('r', 3.8)
        .attr('fill', '#00f0ff')
        .style('filter', 'drop-shadow(0 0 6px rgba(0, 240, 255, 0.85))');

    binduGroup.append('circle')
        .attr('r', 1.7)
        .attr('fill', '#ffffff');
}

export function renderMonads(
    svg: d3.Selection<SVGGElement, unknown, null, undefined>,
    data: Monad[]
) {
    svg.selectAll('.monads-group').remove();
    const monadsGroup = svg.append('g').attr('class', 'monads-group');

    const node = monadsGroup.selectAll<SVGCircleElement, Monad>('.monad')
        .data(data, d => d.id);

    node.enter()
        .append('circle')
        .attr('class', 'monad')
        .attr('r', 0)
        .attr('cx', d => {
            const r = Math.max(d.coord.r || (d.ring * config.ringGap), BINDU_SANCTUARY_RADIUS + 14);
            return r * Math.cos((d.coord.theta * Math.PI) / 180);
        })
        .attr('cy', d => {
            const r = Math.max(d.coord.r || (d.ring * config.ringGap), BINDU_SANCTUARY_RADIUS + 14);
            return r * Math.sin((d.coord.theta * Math.PI) / 180);
        })
        .attr('fill', d => getMonadColor(d.semantic_hash))
        .attr('stroke', d => {
            if (d.kind === 'Struct' || d.kind === 'Enum') return COLORS.secondary;
            if (d.kind === 'Module') return COLORS.tertiary;
            return COLORS.primary;
        })
        .attr('stroke-width', 1.2)
        .style('cursor', 'pointer')
        .style('filter', d => {
            const color = getMonadColor(d.semantic_hash);
            return `drop-shadow(0 0 4px ${color})`;
        })
        .transition()
        .duration(500)
        .ease(d3.easeBackOut)
        .attr('r', (d: any) => d.radius || (data.length > 150 ? 3.2 : 5.5));

    node.exit()
        .transition()
        .duration(300)
        .attr('r', 0)
        .remove();
}

export function highlightMonad(
    svg: d3.Selection<SVGGElement, unknown, null, undefined>,
    monadId: string | null
) {
    const connectedPeers = new Set<string>();
    if (monadId) {
        svg.selectAll<SVGPathElement, EdgeDto>('.edge').each(e => {
            if (e.parent_id === monadId) connectedPeers.add(e.child_id);
            if (e.child_id === monadId) connectedPeers.add(e.parent_id);
        });
    }

    const totalNodes = svg.selectAll('.monad').size();

    svg.selectAll('.monad')
        .transition()
        .duration(180)
        .attr('r', (d: any) => {
            const baseR = d.radius || (totalNodes > 150 ? 3.2 : 5.5);
            if (d.id === monadId) return baseR * 2.25;
            if (connectedPeers.has(d.id)) return baseR * 1.5;
            return baseR;
        })
        .attr('stroke-width', (d: any) => {
            if (d.id === monadId) return 2.8;
            if (connectedPeers.has(d.id)) return 2.0;
            return 1.2;
        })
        .style('filter', (d: any) => {
            if (d.id === monadId) {
                return 'drop-shadow(0 0 12px rgba(255, 255, 255, 0.95))';
            }
            if (connectedPeers.has(d.id)) {
                return 'drop-shadow(0 0 8px rgba(0, 240, 255, 0.9))';
            }
            const color = getMonadColor(d.semantic_hash);
            return `drop-shadow(0 0 4px ${color})`;
        });

    // Dynamically illuminate connected orbital edges when hovering/selecting a Monad
    svg.selectAll<SVGPathElement, EdgeDto>('.edge')
        .transition()
        .duration(180)
        .attr('stroke', e => {
            if (!monadId) return 'rgba(0, 229, 255, 0.18)';
            return e.parent_id === monadId || e.child_id === monadId
                ? 'rgba(0, 245, 255, 0.95)'
                : 'rgba(0, 229, 255, 0.04)';
        })
        .attr('stroke-width', e => {
            if (!monadId) return 0.95;
            return e.parent_id === monadId || e.child_id === monadId ? 2.1 : 0.55;
        })
        .attr('stroke-dasharray', e => {
            if (!monadId) return '3,5';
            return e.parent_id === monadId || e.child_id === monadId ? 'none' : '3,5';
        });
}

/**
 * Generates a 5-waypoint Polar Orbital Curve between two Monads.
 * Because every intermediate waypoint is evaluated in polar coordinates (r(u), θ(u))
 * with r(u) >= BINDU_SANCTUARY_RADIUS + 16, it is geometrically impossible for any edge
 * to sag inward or cross over the Bindu (0,0), even for diametrically opposed nodes (Δθ = 180°).
 */
const orbitalSpline = d3.line<[number, number]>().curve(d3.curveCatmullRom.alpha(0.5));

function buildOrbitalEdgePath(source: Monad, target: Monad, edgeSeed: number): string {
    const minSafeRadius = BINDU_SANCTUARY_RADIUS + 16; // >= 42px
    const sr = Math.max(source.coord.r || 55, minSafeRadius);
    const tr = Math.max(target.coord.r || 89, minSafeRadius);

    // Shortest signed angular geodesic distance on [-180°, +180°]
    let deltaTheta = ((target.coord.theta - source.coord.theta + 540) % 360) - 180;
    if (Math.abs(Math.abs(deltaTheta) - 180) < 1.5) {
        deltaTheta = edgeSeed % 2 === 0 ? 178 : -178;
    }

    const absDelta = Math.abs(deltaTheta);
    // Subtle outward golden bow so same-ring edges arc gracefully along their orbital band
    const archHeight = Math.min(18, absDelta * 0.10) * (source.ring === target.ring ? 1.0 : 0.45);

    const waypoints: [number, number][] = [];
    const steps = 4; // 5 polar waypoints: u = 0, 0.25, 0.50, 0.75, 1.0
    for (let i = 0; i <= steps; i++) {
        const u = i / steps;
        const thetaDeg = source.coord.theta + deltaTheta * u;
        const thetaRad = (thetaDeg * Math.PI) / 180;

        // Parabolic arch 4*u*(1-u) peaks at u=0.5 and is 0 at endpoints u=0, u=1
        const parabola = 4 * u * (1 - u);
        const rInterp = (1 - u) * sr + u * tr + parabola * archHeight;
        const rSafe = Math.max(rInterp, minSafeRadius);

        waypoints.push([rSafe * Math.cos(thetaRad), rSafe * Math.sin(thetaRad)]);
    }

    return orbitalSpline(waypoints) || '';
}

export function renderEdges(
    svg: d3.Selection<SVGGElement, unknown, null, undefined>,
    edges: EdgeDto[],
    monads: Monad[]
) {
    svg.selectAll('.edges-group').remove();
    const edgesGroup = svg.insert('g', '.monads-group').attr('class', 'edges-group');

    const monadMap = new Map(monads.map(m => [m.id, m]));
    const validEdges = edges.filter(
        e => e.parent_id !== e.child_id && monadMap.has(e.parent_id) && monadMap.has(e.child_id)
    );

    const link = edgesGroup.selectAll<SVGPathElement, EdgeDto>('.edge')
        .data(validEdges, e => `${e.parent_id}-${e.child_id}`);

    link.enter()
        .append('path')
        .attr('class', 'edge')
        .attr('fill', 'none')
        .attr('stroke', 'rgba(0, 229, 255, 0.18)')
        .attr('stroke-width', 0.95)
        .attr('stroke-dasharray', '3,5')
        .attr('d', (e, idx) => {
            const source = monadMap.get(e.parent_id)!;
            const target = monadMap.get(e.child_id)!;
            return buildOrbitalEdgePath(source, target, idx + e.parent_id.charCodeAt(0));
        })
        .style('opacity', 0)
        .transition()
        .duration(600)
        .style('opacity', 1);

    link.exit().remove();
}