import * as d3 from 'd3';
import type { Monad } from '../../types/ontology';

export interface InteractionConfig {
  minZoom: number;
  maxZoom: number;
  onSelect?: (monads: Monad[]) => void;
  onHover?: (monad: Monad | null) => void;
  onClick?: (monad: Monad | null) => void;
}

export function setupZoom(
  svgSelection: d3.Selection<SVGSVGElement, unknown, null, undefined>,
  config: InteractionConfig = { minZoom: 0.5, maxZoom: 4 }
): d3.ZoomBehavior<SVGSVGElement, unknown> {
  const zoom = d3.zoom<SVGSVGElement, unknown>()
    .scaleExtent([config.minZoom, config.maxZoom])
    .on('zoom', (event) => {
      svgSelection.select('g.mandala-content').attr('transform', event.transform);
    });

  svgSelection.call(zoom);
  return zoom;
}

export function setupPan(
  svgSelection: d3.Selection<SVGSVGElement, unknown, null, undefined>
): void {
  svgSelection.on('mousedown', function(event) {
    if (event.button === 1 || (event.button === 0 && event.ctrlKey)) {
      event.preventDefault();
      d3.select(this).style('cursor', 'grabbing');
    }
  });
}

export function enableLassoSelection(
  svgSelection: d3.Selection<SVGSVGElement, unknown, null, undefined>,
  onSelect: (monads: Monad[]) => void
): void {
  // Insert lasso-group INSIDE mandala-content so it shares the same
  // coordinate space as monad nodes (inheriting zoom/pan transform).
  const contentGroup = svgSelection.select<SVGGElement>('g.mandala-content');
  const lassoGroup = contentGroup.append('g').attr('class', 'lasso-group');

  let lassoPath: d3.Selection<SVGPathElement, unknown, null, undefined>;
  let isDrawing = false;
  const points: [number, number][] = [];

  const svgNode = svgSelection.node() as SVGSVGElement;

  // Convert screen (clientX/Y) → mandala-content local coordinates.
  // Using getScreenCTM() on the content group accounts for both the
  // SVG viewBox scaling and the d3 zoom/pan transform.
  const getMousePos = (event: MouseEvent): [number, number] => {
    const contentNode = contentGroup.node();
    if (!contentNode) return [0, 0];
    const ctm = contentNode.getScreenCTM();
    if (!ctm) return [0, 0];
    const pt = svgNode.createSVGPoint();
    pt.x = event.clientX;
    pt.y = event.clientY;
    const local = pt.matrixTransform(ctm.inverse());
    return [local.x, local.y];
  };

  svgSelection.on('mousedown.lasso', function(event) {
    if (event.button !== 0 || event.ctrlKey || event.shiftKey) return;

    const target = event.target as SVGElement;
    if (target.closest('.monad') || target.closest('.bindu-group')) return;

    isDrawing = true;
    points.length = 0;
    points.push(getMousePos(event));

    lassoPath = lassoGroup.append('path')
      .attr('class', 'lasso-path')
      .attr('fill', 'rgba(136, 58, 234, 0.1)')
      .attr('stroke', '#883aea')
      .attr('stroke-width', 2)
      .attr('stroke-dasharray', '5,5');
  });

  svgSelection.on('mousemove.lasso', function(event) {
    if (!isDrawing) return;

    points.push(getMousePos(event));

    const pathData = `M ${points.map(p => `${p[0]},${p[1]}`).join(' L ')} Z`;
    lassoPath?.attr('d', pathData);
  });

  svgSelection.on('mouseup.lasso', function() {
    if (!isDrawing) return;
    isDrawing = false;

    // Select monads whose cx/cy (in content-group space) fall inside the polygon.
    const monadNodes = svgSelection.selectAll('.monad');
    const selectedMonads: Monad[] = [];

    if (lassoPath) {
      if (points.length > 2) {
        const polygon = points;
        monadNodes.each(function(d: any) {
          const cx = parseFloat(d3.select(this).attr('cx'));
          const cy = parseFloat(d3.select(this).attr('cy'));
          if (d3.polygonContains(polygon, [cx, cy])) {
            selectedMonads.push(d);
          }
        });
      }
      lassoPath.remove();
    }

    if (selectedMonads.length > 0) {
      onSelect(selectedMonads);
    }
  });
}