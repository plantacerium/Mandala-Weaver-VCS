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

// ─── Fractal AST helpers ───────────────────────────────────────────────────────

interface AstNodeEntry {
  kind: string;
  snippet: string;
  depth: number;
}

/**
 * Parses a monad's raw source content into a set of logical AST sections
 * suitable for fractal rendering. Produces at most 8 sections to keep the
 * radial layout legible.
 */
function parseContentToAstNodes(content: string, monadKind: string): AstNodeEntry[] {
  const lines = content.split('\n').filter(l => l.trim().length > 0);
  const nodes: AstNodeEntry[] = [];

  // Always include the primary kind node at depth 0
  nodes.push({
    kind: monadKind.toUpperCase(),
    snippet: lines[0]?.trim().slice(0, 28) ?? '—',
    depth: 0,
  });

  for (const line of lines.slice(1)) {
    const trimmed = line.trim();
    if (nodes.length >= 8) break;

    if (/^(pub\s+)?fn\s+/.test(trimmed)) {
      nodes.push({ kind: 'fn', snippet: trimmed.slice(0, 28), depth: 1 });
    } else if (/^(pub\s+)?struct\s+/.test(trimmed)) {
      nodes.push({ kind: 'struct', snippet: trimmed.slice(0, 28), depth: 1 });
    } else if (/^(pub\s+)?enum\s+/.test(trimmed)) {
      nodes.push({ kind: 'enum', snippet: trimmed.slice(0, 28), depth: 1 });
    } else if (/^(pub\s+)?trait\s+/.test(trimmed)) {
      nodes.push({ kind: 'trait', snippet: trimmed.slice(0, 28), depth: 1 });
    } else if (/^(pub\s+)?impl\b/.test(trimmed)) {
      nodes.push({ kind: 'impl', snippet: trimmed.slice(0, 28), depth: 1 });
    } else if (/^(pub\s+)?(const|static)\s+/.test(trimmed)) {
      nodes.push({ kind: 'const', snippet: trimmed.slice(0, 28), depth: 2 });
    } else if (/^(pub\s+)?type\s+/.test(trimmed)) {
      nodes.push({ kind: 'type', snippet: trimmed.slice(0, 28), depth: 2 });
    } else if (/^use\s+/.test(trimmed)) {
      nodes.push({ kind: 'use', snippet: trimmed.slice(0, 28), depth: 2 });
    }
  }

  // Fill up to 4 nodes with body segments if sparse
  if (nodes.length < 4 && lines.length > 2) {
    const segmentSize = Math.floor(lines.length / 3);
    const segments = [
      { kind: 'body:start', lines: lines.slice(0, segmentSize) },
      { kind: 'body:mid',   lines: lines.slice(segmentSize, segmentSize * 2) },
      { kind: 'body:end',   lines: lines.slice(segmentSize * 2) },
    ];
    for (const seg of segments) {
      if (nodes.length >= 8) break;
      nodes.push({ kind: seg.kind, snippet: seg.lines[0]?.trim().slice(0, 28) ?? '—', depth: 2 });
    }
  }

  return nodes;
}

/** Maps AST node kinds to distinct HSL colors for visual differentiation. */
function getAstNodeColor(kind: string): string {
  const palette: Record<string, string> = {
    'FUNCTION':  '#00e5ff', fn:    '#00e5ff',
    'STRUCT':    '#9d4edd', struct:'#9d4edd',
    'ENUM':      '#ff6b9d', enum:  '#ff6b9d',
    'TRAIT':     '#ffd700', trait: '#ffd700',
    'IMPL':      '#00ffcc', impl:  '#00ffcc',
    const:       '#ff9500', static:'#ff9500',
    type:        '#88c0d0',
    use:         '#5e81ac',
    'MODULE':    '#a3be8c', mod:   '#a3be8c',
    'body:start':'#4a6a6a', 'body:mid':'#5a7a7a', 'body:end':'#3a5a5a',
  };
  return palette[kind] ?? '#8b949e';
}

/**
 * Micro-Immersion: Fractal AST View
 *
 * When zoomed into a Monad past the micro threshold (scale > 3), replaces the
 * normal canvas with a fractal unfolding of the monad's internal structure:
 *
 *   1. Central identity node — name, kind, ring, hash fingerprint
 *   2. AST breakdown — content parsed into logical sections (signature, body
 *      blocks, types, constants) as radial sub-nodes with animated entrance
 *   3. Evolutionary lineage spiral — ancestor chain with chromatic identities
 *
 * Implements README_Core.md "Micro-Immersion: The Monad undergoes a fractal
 * unfolding — revealing its AST, evolutionary lineage, and atomic essence."
 */
export function setMicroZoom(
  svg: d3.Selection<SVGSVGElement, unknown, null, undefined>,
  monad: Monad,
  lineage: Monad[]
): void {
  svg.selectAll('.monad-node, .monad-label, .mandala-container, .micro-zoom-container').remove();

  const svgNode = svg.node();
  const width = svgNode ? (svgNode.clientWidth || 800) : 800;
  const height = svgNode ? (svgNode.clientHeight || 600) : 600;

  const container = svg.append('g')
    .attr('class', 'micro-zoom-container')
    .attr('transform', `translate(${width / 2}, ${height / 2})`);

  const monadColor = getMonadColor(monad.semantic_hash);

  // ── 1. Central identity node ──────────────────────────────────────────────
  container.append('circle')
    .attr('r', 0)
    .attr('fill', 'none')
    .attr('stroke', monadColor)
    .attr('stroke-width', 1)
    .attr('opacity', 0.3)
    .transition().duration(600)
    .attr('r', 60);

  container.append('circle')
    .attr('r', 0)
    .attr('fill', `${monadColor}22`)
    .attr('stroke', monadColor)
    .attr('stroke-width', 2)
    .style('filter', `drop-shadow(0 0 12px ${monadColor})`)
    .transition().duration(400)
    .attr('r', 32);

  container.append('text')
    .attr('y', -48)
    .attr('text-anchor', 'middle')
    .attr('fill', monadColor)
    .attr('font-size', '15px')
    .attr('font-weight', 'bold')
    .attr('font-family', 'JetBrains Mono, monospace')
    .attr('opacity', 0)
    .text(monad.name)
    .transition().delay(200).duration(400)
    .attr('opacity', 1);

  container.append('text')
    .attr('y', -32)
    .attr('text-anchor', 'middle')
    .attr('fill', '#8b949e')
    .attr('font-size', '11px')
    .attr('opacity', 0)
    .text(`${monad.kind}  ·  Ring ${monad.ring}  ·  L${monad.line_start}–${monad.line_end}`)
    .transition().delay(300).duration(400)
    .attr('opacity', 1);

  container.append('text')
    .attr('y', -16)
    .attr('text-anchor', 'middle')
    .attr('fill', '#4a6a6a')
    .attr('font-size', '9px')
    .attr('font-family', 'JetBrains Mono, monospace')
    .attr('opacity', 0)
    .text(`#${monad.semantic_hash.slice(0, 16)}`)
    .transition().delay(400).duration(400)
    .attr('opacity', 1);

  // ── 2. Fractal AST breakdown ──────────────────────────────────────────────
  const astNodes = parseContentToAstNodes(monad.content, monad.kind);
  const astRadius = 130;
  const astGroup = container.append('g').attr('class', 'fractal-ast-tree');

  astNodes.forEach((node, i) => {
    const angle = (2 * Math.PI * i) / astNodes.length - Math.PI / 2;
    const nx = astRadius * Math.cos(angle);
    const ny = astRadius * Math.sin(angle);
    const nodeColor = getAstNodeColor(node.kind);

    astGroup.append('line')
      .attr('x1', 0).attr('y1', 0)
      .attr('x2', 0).attr('y2', 0)
      .attr('stroke', `${nodeColor}55`)
      .attr('stroke-width', 1)
      .attr('stroke-dasharray', '3,3')
      .transition().delay(500 + i * 60).duration(400)
      .attr('x2', nx * 0.85).attr('y2', ny * 0.85);

    astGroup.append('circle')
      .attr('cx', nx).attr('cy', ny)
      .attr('r', 0)
      .attr('fill', `${nodeColor}33`)
      .attr('stroke', nodeColor)
      .attr('stroke-width', 1.5)
      .transition().delay(500 + i * 60).duration(350)
      .attr('r', node.depth === 0 ? 18 : 12);

    astGroup.append('text')
      .attr('x', nx)
      .attr('y', ny - (node.depth === 0 ? 24 : 18))
      .attr('text-anchor', 'middle')
      .attr('fill', nodeColor)
      .attr('font-size', '9px')
      .attr('font-family', 'JetBrains Mono, monospace')
      .attr('opacity', 0)
      .text(node.kind)
      .transition().delay(600 + i * 60).duration(300)
      .attr('opacity', 0.9);

    astGroup.append('text')
      .attr('x', nx)
      .attr('y', ny + (node.depth === 0 ? 30 : 22))
      .attr('text-anchor', 'middle')
      .attr('fill', '#6a7480')
      .attr('font-size', '8px')
      .attr('font-family', 'JetBrains Mono, monospace')
      .attr('opacity', 0)
      .text(node.snippet)
      .transition().delay(700 + i * 60).duration(300)
      .attr('opacity', 0.7);
  });

  // ── 3. Evolutionary lineage spiral ────────────────────────────────────────
  if (lineage.length > 0) {
    const lineageGroup = container.append('g')
      .attr('class', 'fractal-lineage-spiral')
      .attr('transform', `translate(0, ${astRadius + 80})`);

    lineageGroup.append('text')
      .attr('text-anchor', 'middle')
      .attr('fill', '#4a6a6a')
      .attr('font-size', '10px')
      .attr('letter-spacing', '0.1em')
      .attr('opacity', 0)
      .text('EVOLUTIONARY LINEAGE')
      .transition().delay(800).duration(300)
      .attr('opacity', 0.6);

    lineage.slice(0, 6).forEach((ancestor, i) => {
      const ancestorColor = getMonadColor(ancestor.semantic_hash);
      const y = 24 + i * 30;
      const spiralX = Math.sin(i * 0.8) * 20;

      lineageGroup.append('line')
        .attr('x1', spiralX).attr('y1', y - 26)
        .attr('x2', spiralX).attr('y2', y - 14)
        .attr('stroke', `${ancestorColor}44`)
        .attr('stroke-width', 1)
        .attr('opacity', 0)
        .transition().delay(900 + i * 80).duration(200)
        .attr('opacity', 1);

      lineageGroup.append('circle')
        .attr('cx', spiralX).attr('cy', y)
        .attr('r', 0)
        .attr('fill', `${ancestorColor}22`)
        .attr('stroke', ancestorColor)
        .attr('stroke-width', 1.5)
        .transition().delay(900 + i * 80).duration(300)
        .attr('r', 7);

      lineageGroup.append('text')
        .attr('x', spiralX + 14)
        .attr('y', y + 4)
        .attr('fill', '#c9d1d9')
        .attr('font-size', '10px')
        .attr('opacity', 0)
        .text(`${ancestor.name} · Ring ${ancestor.ring}`)
        .transition().delay(1000 + i * 80).duration(300)
        .attr('opacity', 0.85);
    });

    if (lineage.length > 6) {
      lineageGroup.append('text')
        .attr('x', 14).attr('y', 24 + 6 * 30 + 4)
        .attr('fill', '#4a6a6a')
        .attr('font-size', '9px')
        .attr('opacity', 0)
        .text(`+ ${lineage.length - 6} more ancestors…`)
        .transition().delay(1500).duration(300)
        .attr('opacity', 0.5);
    }
  }
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