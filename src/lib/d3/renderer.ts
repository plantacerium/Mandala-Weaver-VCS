import * as d3 from 'd3';
import type { Monad } from '../../types/ontology';

const COLORS = {
    primary: '#8b5cf6',
    primaryLight: '#a78bfa',
    secondary: '#06b6d4',
    tertiary: '#f472b6',
    quaternary: '#34d399',
    ringStroke: 'rgba(255, 255, 255, 0.08)',
    ringFill: 'rgba(255, 255, 255, 0.02)',
};

const RING_COLORS = [
    'rgba(139, 92, 246, 0.15)',
    'rgba(6, 182, 212, 0.12)',
    'rgba(244, 114, 182, 0.10)',
    'rgba(52, 211, 153, 0.08)',
    'rgba(251, 191, 36, 0.06)',
    'rgba(248, 113, 113, 0.05)',
];

interface RingConfig {
    baseRadius: number;
    ringGap: number;
    ringCount: number;
}

const config: RingConfig = {
    baseRadius: 50,
    ringGap: 70,
    ringCount: 6,
};

function getRingColor(ringLevel: number): string {
    return RING_COLORS[(ringLevel - 1) % RING_COLORS.length];
}

function polarToCartesian(r: number, theta: number): { x: number; y: number } {
    const rad = theta * Math.PI / 180;
    return {
        x: r * Math.cos(rad),
        y: r * Math.sin(rad),
    };
}

export function drawMandalaGrid(
    svg: d3.Selection<SVGGElement, unknown, null, undefined>,
    maxRadius: number
) {
    const ringsGroup = svg.append('g').attr('class', 'rings-group');
    const sectorsGroup = svg.append('g').attr('class', 'sectors-group');
    
    for (let i = 1; i <= config.ringCount; i++) {
        const radius = config.baseRadius + (i - 1) * config.ringGap;
        
        ringsGroup.append('circle')
            .attr('class', `ring ring-${i}`)
            .attr('r', radius)
            .attr('fill', getRingColor(i))
            .attr('stroke', COLORS.ringStroke)
            .attr('stroke-width', 1);
    }

    const sectorAngles = [0, 45, 90, 135, 180, 225, 270, 315];
    const sectorColors = [
        COLORS.primary,
        COLORS.secondary,
        COLORS.tertiary,
        COLORS.quaternary,
        COLORS.primaryLight,
        '#f59e0b',
        '#ec4899',
        '#10b981',
    ];

    sectorAngles.forEach((angle, idx) => {
        const rad = angle * Math.PI / 180;
        sectorsGroup.append('line')
            .attr('x1', 0)
            .attr('y1', 0)
            .attr('x2', maxRadius * Math.cos(rad))
            .attr('y2', maxRadius * Math.sin(rad))
            .attr('stroke', sectorColors[idx])
            .attr('stroke-width', 0.5)
            .attr('stroke-opacity', 0.15);
    });
}

export function drawBindu(svg: d3.Selection<SVGGElement, unknown, null, undefined>) {
    const binduGroup = svg.append('g').attr('class', 'bindu-group');
    
    binduGroup.append('circle')
        .attr('r', 40)
        .attr('fill', 'rgba(139, 92, 246, 0.1)');

    binduGroup.append('circle')
        .attr('r', 20)
        .attr('fill', 'rgba(139, 92, 246, 0.2)')
        .attr('stroke', COLORS.primary)
        .attr('stroke-width', 1.5)
        .attr('stroke-opacity', 0.5);

    binduGroup.append('circle')
        .attr('r', 8)
        .attr('fill', COLORS.primary)
        .style('filter', 'drop-shadow(0 0 10px rgba(139, 92, 246, 0.8))');

    binduGroup.append('circle')
        .attr('r', 3)
        .attr('fill', '#fff');
}

export function renderMonads(
    svg: d3.Selection<SVGGElement, unknown, null, undefined>,
    data: Monad[]
) {
    const monadsGroup = svg.append('g').attr('class', 'monads-group');
    
    const node = monadsGroup.selectAll<SVGCircleElement, Monad>('.monad')
        .data(data, d => d.id);

    node.enter()
        .append('circle')
        .attr('class', 'monad')
        .attr('r', 0)
        .attr('cx', d => {
            const r = d.coord.r || (d.ring * config.ringGap);
            return r * Math.cos(d.coord.theta * Math.PI / 180);
        })
        .attr('cy', d => {
            const r = d.coord.r || (d.ring * config.ringGap);
            return r * Math.sin(d.coord.theta * Math.PI / 180);
        })
        .attr('fill', d => RING_COLORS[(d.ring - 1) % RING_COLORS.length])
        .attr('stroke', COLORS.primary)
        .attr('stroke-width', 1.5)
        .style('cursor', 'pointer')
        .style('filter', 'drop-shadow(0 0 4px rgba(139, 92, 246, 0.6))')
        .transition()
        .duration(600)
        .ease(d3.easeBackOut)
        .attr('r', 6);

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
    svg.selectAll('.monad')
        .transition()
        .duration(200)
        .attr('r', (d: any) => d.id === monadId ? 10 : 6)
        .attr('stroke', (d: any) => d.id === monadId ? '#00ff88' : COLORS.primary)
        .style('filter', (d: any) => d.id === monadId 
            ? 'drop-shadow(0 0 8px rgba(0, 255, 136, 0.9))' 
            : 'drop-shadow(0 0 4px rgba(139, 92, 246, 0.6))'
        );
}