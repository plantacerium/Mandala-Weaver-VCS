import * as d3 from 'd3';
import type { Monad } from '../../types/ontology';

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
            .attr('stroke-dasharray', i % 2 === 0 ? '4,4' : 'none')
            .attr('stroke-width', 0.8);
    }

    const sectorAngles = Array.from({ length: 16 }, (_, i) => i * 22.5);
    
    sectorAngles.forEach((angle, idx) => {
        const rad = angle * Math.PI / 180;
        const isMainAxis = angle % 90 === 0;
        
        sectorsGroup.append('line')
            .attr('x1', 0)
            .attr('y1', 0)
            .attr('x2', maxRadius * Math.cos(rad))
            .attr('y2', maxRadius * Math.sin(rad))
            .attr('stroke', isMainAxis ? COLORS.secondary : COLORS.primary)
            .attr('stroke-width', isMainAxis ? 1 : 0.5)
            .attr('stroke-dasharray', isMainAxis ? 'none' : '2,4')
            .attr('stroke-opacity', isMainAxis ? 0.3 : 0.15);
    });
}

export function drawBindu(svg: d3.Selection<SVGGElement, unknown, null, undefined>) {
    const binduGroup = svg.append('g').attr('class', 'bindu-group');
    
    binduGroup.append('circle')
        .attr('r', 40)
        .attr('fill', 'url(#glow)')
        .attr('opacity', 0.1);

    binduGroup.append('circle')
        .attr('r', 20)
        .attr('fill', 'rgba(157, 78, 221, 0.1)')
        .attr('stroke', COLORS.secondary)
        .attr('stroke-width', 1.5)
        .attr('stroke-opacity', 0.8)
        .attr('stroke-dasharray', '2,2');

    binduGroup.append('circle')
        .attr('r', 8)
        .attr('fill', COLORS.primary)
        .style('filter', 'drop-shadow(0 0 12px rgba(0, 229, 255, 0.9))');

    binduGroup.append('circle')
        .attr('r', 2)
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
        .attr('fill', d => {
            if (d.kind === 'Struct' || d.kind === 'Enum') return 'rgba(157, 78, 221, 0.8)';
            if (d.kind === 'Module') return 'rgba(255, 62, 131, 0.8)';
            return 'rgba(0, 229, 255, 0.8)'; // Functions and traits
        })
        .attr('stroke', d => {
            if (d.kind === 'Struct' || d.kind === 'Enum') return COLORS.secondary;
            if (d.kind === 'Module') return COLORS.tertiary;
            return COLORS.primary;
        })
        .attr('stroke-width', 1.5)
        .style('cursor', 'pointer')
        .style('filter', d => {
            const color = d.kind === 'Struct' || d.kind === 'Enum' ? '157, 78, 221' :
                         d.kind === 'Module' ? '255, 62, 131' : '0, 229, 255';
            return `drop-shadow(0 0 6px rgba(${color}, 0.8))`;
        })
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
        .attr('stroke-width', (d: any) => d.id === monadId ? 3 : 1.5)
        .style('filter', (d: any) => {
            if (d.id === monadId) {
                return 'drop-shadow(0 0 12px rgba(255, 255, 255, 0.9))';
            }
            const color = d.kind === 'Struct' || d.kind === 'Enum' ? '157, 78, 221' :
                         d.kind === 'Module' ? '255, 62, 131' : '0, 229, 255';
            return `drop-shadow(0 0 6px rgba(${color}, 0.8))`;
        });
}