import * as d3 from 'd3';
import type { EdgeDto, Monad } from '../../types/ontology';

interface PathData {
  from: { x: number; y: number };
  to: { x: number; y: number };
}

export function polarToCartesian(r: number, theta: number): { x: number; y: number } {
  const rad = theta * Math.PI / 180;
  return {
    x: r * Math.cos(rad),
    y: r * Math.sin(rad),
  };
}

function describeBezier(from: { x: number; y: number }, to: { x: number; y: number }): string {
  const midX = (from.x + to.x) / 2;
  const midY = (from.y + to.y) / 2;
  const cp1x = from.x + (midX - from.x) * 0.5;
  const cp1y = from.y;
  const cp2x = midX + (to.x - midX) * 0.5;
  const cp2y = to.y;
  
  return `M ${from.x},${from.y} C ${cp1x},${cp1y} ${cp2x},${cp2y} ${to.x},${to.y}`;
}

export function animateLineagePath(
  svg: d3.Selection<SVGSVGElement, unknown, null, undefined>,
  pathData: PathData[]
): void {
  pathData.forEach((path, i) => {
    const animPath = svg.append('path')
      .attr('d', describeBezier(path.from, path.to))
      .attr('fill', 'none')
      .attr('stroke', 'url(#lineage-gradient)')
      .attr('stroke-width', 2)
      .attr('stroke-linecap', 'round')
      .attr('opacity', 0);
    
    animPath.transition()
      .duration(800)
      .delay(i * 100)
      .attr('opacity', 1)
      .attr('stroke-dasharray', function() {
        return (this as SVGPathElement).getTotalLength().toString();
      })
      .attr('stroke-dashoffset', function() {
        return (this as SVGPathElement).getTotalLength().toString();
      })
      .transition()
      .duration(800)
      .attr('stroke-dashoffset', 0);
  });
}

export function pulseSelection(
  node: d3.Selection<SVGCircleElement, unknown, null, undefined>
): void {
  node.transition()
    .duration(300)
    .attr('r', 12)
    .attr('filter', 'url(#glow)')
    .transition()
    .duration(300)
    .attr('r', 8)
    .attr('filter', null);
}

export function fadeInNode(
  svg: d3.Selection<SVGCircleElement, unknown, null, undefined>,
  delay: number = 0
): void {
  svg.attr('opacity', 0)
    .attr('transform', 'scale(0)')
    .transition()
    .duration(400)
    .delay(delay)
    .attr('opacity', 1)
    .attr('transform', 'scale(1)')
    .attr('ease', d3.easeBackOut);
}

export function fadeOutNode(
  svg: d3.Selection<SVGCircleElement, unknown, null, undefined>,
  onComplete?: () => void
): void {
  svg.transition()
    .duration(300)
    .attr('opacity', 0)
    .attr('transform', 'scale(0)')
    .remove();
  
  if (onComplete) {
    setTimeout(onComplete, 300);
  }
}

export function animateRingExpand(
  svg: d3.Selection<SVGGElement, unknown, null, undefined>,
  ringLevel: number,
  baseRadius: number,
  ringGap: number
): void {
  const radius = baseRadius + (ringLevel - 1) * ringGap;
  
  svg.append('circle')
    .attr('r', 0)
    .attr('cx', 0)
    .attr('cy', 0)
    .attr('fill', 'none')
    .attr('stroke', 'rgba(0, 229, 255, 0.2)')
    .attr('stroke-width', 1)
    .transition()
    .duration(600)
    .attr('r', radius);
}

export function animateEdgeDraw(
  svg: d3.Selection<SVGPathElement, unknown, null, undefined>
): void {
  const length = (svg.node() as SVGPathElement)?.getTotalLength() || 100;
  
  svg
    .attr('stroke-dasharray', length)
    .attr('stroke-dashoffset', length)
    .transition()
    .duration(600)
    .attr('stroke-dashoffset', 0);
}

export function breatheAnimation(
  svg: d3.Selection<SVGGElement, unknown, null, undefined>,
  center: { x: number; y: number },
  maxRadius: number
): void {
  const circle = svg.append('circle')
    .attr('cx', center.x)
    .attr('cy', center.y)
    .attr('r', 0)
    .attr('fill', 'none')
    .attr('stroke', 'rgba(0, 229, 255, 0.3)')
    .attr('stroke-width', 2);
  
  const animate = () => {
    circle
      .attr('r', 0)
      .attr('opacity', 0.8)
      .transition()
      .duration(2000)
      .ease(d3.easeSinOut)
      .attr('r', maxRadius)
      .attr('opacity', 0)
      .on('end', animate);
  };
  
  animate();
}

export function shimmerEffect(
  node: d3.Selection<SVGCircleElement, unknown, null, undefined>
): void {
  const gradient = defs.select('#shimmer-gradient');
  
  if (gradient.empty()) {
    const defs = d3.select(node.node()?.ownerSVGElement).select('defs');
    const grad = defs.append('linearGradient')
      .attr('id', 'shimmer-gradient')
      .attr('gradientUnits', 'userSpaceOnUse');
    
    grad.append('stop')
      .attr('offset', '0%')
      .attr('stop-color', 'rgba(255,255,255,0)');
    
    grad.append('stop')
      .attr('offset', '50%')
      .attr('stop-color', 'rgba(255,255,255,0.8)');
    
    grad.append('stop')
      .attr('offset', '100%')
      .attr('stop-color', 'rgba(255,255,255,0)');
  }
  
  node
    .attr('stroke', 'url(#shimmer-gradient)')
    .attr('stroke-width', 3)
    .transition()
    .duration(1000)
    .attr('stroke-dashoffset', -20)
    .on('end', function() {
      d3.select(this).attr('stroke-dashoffset', 20);
    });
}

function get defs(): d3.Selection<SVGDefsElement, unknown, null, undefined> {
  let defs = d3.select('svg defs');
  if (defs.empty()) {
    defs = d3.select('svg').append('defs');
  }
  return defs as any;
}