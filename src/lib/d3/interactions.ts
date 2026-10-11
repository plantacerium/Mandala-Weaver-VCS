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
  config: InteractionConfig = { minZoom: 0.3, maxZoom: 8 }
): d3.ZoomBehavior<SVGSVGElement, unknown> {
  const zoom = d3.zoom<SVGSVGElement, unknown>()
    .scaleExtent([config.minZoom, config.maxZoom])
    .filter((event) => {
      // Natural wheel zooming without requiring Ctrl
      if (event.type === 'wheel') return true;
      // Do not initiate pan-drag if clicking directly on a node
      if (event.type === 'mousedown') {
        const target = event.target as SVGElement;
        if (target && target.closest && (target.closest('.monad') || target.closest('.bindu-group'))) {
          return false;
        }
      }
      return !event.button || event.button === 0 || event.button === 1;
    })
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

/**
 * Completely removes lasso selection listeners and artifacts from the SVG.
 */
export function disableLassoSelection(
  svgSelection: d3.Selection<SVGSVGElement, unknown, null, undefined>
): void {
  svgSelection.on('mousedown.lasso', null);
  svgSelection.on('mousemove.lasso', null);
  svgSelection.on('mouseup.lasso', null);

  const contentGroup = svgSelection.select<SVGGElement>('g.mandala-content');
  contentGroup.selectAll('.lasso-group').remove();
  contentGroup.selectAll('.lasso-path').remove();
}

/**
 * Enables lasso selection for harvesting monads (e.g. in Distill mode).
 * Uses window-level mouseup listener to guarantee that releasing the mouse anywhere
 * always terminates drawing and prevents dangling path lines.
 */
export function enableLassoSelection(
  svgSelection: d3.Selection<SVGSVGElement, unknown, null, undefined>,
  onSelect: (monads: Monad[]) => void
): () => void {
  // First clear any previous lasso state
  disableLassoSelection(svgSelection);

  const contentGroup = svgSelection.select<SVGGElement>('g.mandala-content');
  if (contentGroup.empty()) return () => {};

  const lassoGroup = contentGroup.append('g').attr('class', 'lasso-group');

  let lassoPath: d3.Selection<SVGPathElement, unknown, null, undefined> | null = null;
  let isDrawing = false;
  let points: [number, number][] = [];

  const svgNode = svgSelection.node();
  if (!svgNode) return () => {};

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

  const handleMouseDown = function(event: MouseEvent) {
    // Only primary left button, ignore if holding Ctrl/Meta/Alt
    if (event.button !== 0 || event.ctrlKey || event.metaKey || event.altKey) return;

    const target = event.target as SVGElement;
    if (target && target.closest && (target.closest('.monad') || target.closest('.bindu-group'))) {
      return;
    }

    isDrawing = true;
    points = [getMousePos(event)];

    // Remove any leftover paths
    lassoGroup.selectAll('.lasso-path').remove();

    lassoPath = lassoGroup.append('path')
      .attr('class', 'lasso-path')
      .attr('fill', 'rgba(136, 58, 234, 0.15)')
      .attr('stroke', '#883aea')
      .attr('stroke-width', 2)
      .attr('stroke-dasharray', '5,5');
  };

  const handleMouseMove = function(event: MouseEvent) {
    if (!isDrawing || !lassoPath) return;

    points.push(getMousePos(event));

    if (points.length > 1) {
      const pathData = `M ${points.map(p => `${p[0]},${p[1]}`).join(' L ')} Z`;
      lassoPath.attr('d', pathData);
    }
  };

  const handleMouseUp = function() {
    if (!isDrawing) return;
    isDrawing = false;

    if (lassoPath && points.length > 2) {
      const monadNodes = contentGroup.selectAll<SVGCircleElement, Monad>('.monad');
      const selectedMonads: Monad[] = [];

      monadNodes.each(function(d) {
        if (!d) return;
        const el = d3.select(this);
        const cx = parseFloat(el.attr('cx'));
        const cy = parseFloat(el.attr('cy'));
        if (!isNaN(cx) && !isNaN(cy) && d3.polygonContains(points, [cx, cy])) {
          selectedMonads.push(d);
        }
      });

      if (selectedMonads.length > 0) {
        onSelect(selectedMonads);
      }
    }

    // Always clean up path and points on mouseup so no lines remain on screen
    if (lassoPath) {
      lassoPath.remove();
      lassoPath = null;
    }
    points = [];
  };

  svgSelection.on('mousedown.lasso', handleMouseDown);
  svgSelection.on('mousemove.lasso', handleMouseMove);

  // Use window listener so releasing anywhere cleanly finishes the lasso
  window.addEventListener('mouseup', handleMouseUp);

  return () => {
    window.removeEventListener('mouseup', handleMouseUp);
    disableLassoSelection(svgSelection);
  };
}