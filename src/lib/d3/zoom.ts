import * as d3 from 'd3';
import type { Monad } from '../../types/ontology';

export type ZoomMode = 'macro' | 'micro' | 'normal';

interface RingSummary {
  ring: number;
  monad_count: number;
  density: number;
}

export function setMacroZoom(
  svg: d3.Selection<SVGSVGElement, unknown, null, undefined>,
  rings: RingSummary[]
): void {
  svg.selectAll('.monad-node').remove();
  svg.selectAll('.monad-label').remove();
  
  rings.forEach(ring => {
    const intensity = Math.min(ring.monad_count / 100, 1);
    
    svg.append('circle')
      .attr('class', 'ring-density')
      .attr('r', ring.ring * 80)
      .attr('fill', `rgba(88, 166, 255, ${intensity * 0.3})`)
      .attr('stroke', `rgba(88, 166, 255, ${intensity * 0.5})`)
      .attr('stroke-width', 2)
      .attr('stroke-dasharray', '5,5');
    
    svg.append('text')
      .attr('class', 'ring-label')
      .attr('x', 0)
      .attr('y', -(ring.ring * 80) + 10)
      .attr('text-anchor', 'middle')
      .attr('fill', '#8b949e')
      .attr('font-size', '12px')
      .text(`Ring ${ring.ring}: ${ring.monad_count} monads`);
  });
}

export function setMicroZoom(
  svg: d3.Selection<SVGSVGElement, unknown, null, undefined>,
  monad: Monad,
  lineage: Monad[]
): void {
  svg.selectAll('.monad-node').remove();
  svg.selectAll('.monad-label').remove();
  
  svg.select('.mandala-container')?.remove();
  
  const container = svg.append('g')
    .attr('class', 'mandala-container')
    .attr('transform', 'translate(300, 300)');
  
  container.append('text')
    .attr('y', -180)
    .attr('text-anchor', 'middle')
    .attr('fill', '#00e5ff')
    .attr('font-size', '16px')
    .attr('font-weight', 'bold')
    .text(monad.name);
  
  container.append('text')
    .attr('y', -155)
    .attr('text-anchor', 'middle')
    .attr('fill', '#8b949e')
    .attr('font-size', '12px')
    .text(`Ring ${monad.ring} • ${monad.kind}`);
  
  lineage.forEach((ancestor, i) => {
    const y = 20 + i * 35;
    
    container.append('circle')
      .attr('cx', 0)
      .attr('cy', y)
      .attr('r', 6)
      .attr('fill', getMonadColor(ancestor.semantic_hash))
      .attr('class', 'lineage-node');
    
    container.append('text')
      .attr('x', 15)
      .attr('y', y + 5)
      .attr('fill', '#c9d1d9')
      .attr('font-size', '11px')
      .text(`${ancestor.name} (Ring ${ancestor.ring})`);
  });
  
  const contentBox = container.append('g')
    .attr('transform', 'translate(0, 200)');
  
  contentBox.append('rect')
    .attr('x', -150)
    .attr('y', 0)
    .attr('width', 300)
    .attr('height', 150)
    .attr('rx', 8)
    .attr('fill', '#161b22')
    .attr('stroke', '#30363d');
  
  contentBox.append('text')
    .attr('x', -140)
    .attr('y', 20)
    .attr('fill', '#8b949e')
    .attr('font-size', '10px')
    .attr('font-family', 'monospace')
    .text(monad.content.substring(0, 500));
}

function getMonadColor(hash: string): string {
  if (!hash) return '#00e5ff';
  
  let h = 0;
  for (let i = 0; i < hash.length; i++) {
    h = ((h << 5) - h) + hash.charCodeAt(i);
    h |= 0;
  }
  
  const hue = Math.abs(h) % 360;
  return `hsl(${hue}, 85%, 60%)`;
}

export function animateZoom(
  svg: d3.Selection<SVGGElement, unknown, null, undefined>,
  mode: ZoomMode,
  config: { duration: number } = { duration: 500 }
): void {
  const zoom = d3.zoom<SVGSVGElement, unknown>()
    .scaleExtent([0.1, 10])
    .on('zoom', (event) => {
      svg.attr('transform', event.transform);
    });
  
  const scale = mode === 'macro' ? 0.3 : mode === 'micro' ? 3 : 1;
  
  svg.transition()
    .duration(config.duration)
    .call(zoom.scaleBy as any, scale);
}

export function setupSemanticZoom(
  svg: d3.Selection<SVGSVGElement, unknown, null, undefined>,
  container: d3.Selection<SVGGElement, unknown, null, undefined>
): d3.ZoomBehavior<SVGSVGElement, unknown> {
  const zoom = d3.zoom<SVGSVGElement, unknown>()
    .scaleExtent([0.1, 10])
    .filter((event) => {
      if (event.type === 'wheel') {
        return event.ctrlKey || event.metaKey;
      }
      return true;
    })
    .on('zoom', (event) => {
      container.attr('transform', event.transform);
      
      const currentScale = event.transform.k;
      if (currentScale < 0.3) {
        container.emit('zoom-mode', 'macro');
      } else if (currentScale > 3) {
        container.emit('zoom-mode', 'micro');
      } else {
        container.emit('zoom-mode', 'normal');
      }
    });
  
  svg.call(zoom);
  
  return zoom;
}

export function getZoomMode(scale: number): ZoomMode {
  if (scale < 0.3) return 'macro';
  if (scale > 3) return 'micro';
  return 'normal';
}