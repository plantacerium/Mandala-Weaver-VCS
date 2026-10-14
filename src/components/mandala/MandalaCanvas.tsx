import React, { useEffect, useRef, useCallback, useState } from 'react';
import * as d3 from 'd3';
import { drawMandalaGrid, drawBindu, renderMonads, highlightMonad, renderEdges } from '../../lib/d3/renderer';
import { enableLassoSelection, disableLassoSelection } from '../../lib/d3/interactions';
import { fetchMandalaState, invokeExpand } from '../../lib/tauri/commands';
import { listenForFileChanges } from '../../lib/tauri/events';
import { useWorkspaceStore } from '../../lib/state/workspaceStore';
import { setupSemanticZoom, getZoomMode, setMacroZoom, setMicroZoom } from '../../lib/d3/zoom';
import { fetchLineageEdges } from '../../lib/tauri/commands';
import { isTauri } from '../../lib/tauri/isTauri';
import type { Monad } from '../../types/ontology';

interface CommitDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onCommit: (filePath: string) => void;
}

const CommitDialog: React.FC<CommitDialogProps> = ({ isOpen, onClose, onCommit }) => {
  const [filePath, setFilePath] = useState('');

  if (!isOpen) return null;

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog-content" onClick={e => e.stopPropagation()}>
        <div className="dialog-header">
          <h2>EXPAND RING</h2>
          <span className="dialog-subtitle">Crear nuevo anillo de expansión</span>
        </div>
        <div className="dialog-body">
          <label>Selecciona archivo fuente</label>
          <input 
            type="text" 
            placeholder="src/main.rs" 
            value={filePath}
            onChange={(e) => setFilePath(e.target.value)}
            className="dialog-input"
          />
          <p className="dialog-hint">Ingresa la ruta del archivo para extraer nuevas mónadas</p>
        </div>
        <div className="dialog-actions">
          <button className="btn-cancel" onClick={onClose}>CANCELAR</button>
          <button className="btn-confirm" onClick={() => onCommit(filePath)}>EXPAND</button>
        </div>
      </div>
    </div>
  );
};

const MandalaCanvas: React.FC = () => {
  const svgRef = useRef<SVGSVGElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const { setMandalaState, selectMonad, hoverMonad, setSelectedForDistill, setZoomMode, viewMode } = useWorkspaceStore();
  const [showCommitDialog, setShowCommitDialog] = useState(false);
  const svgGroupRef = useRef<d3.Selection<SVGGElement, unknown, null, undefined> | null>(null);

  const handleSelect = useCallback((monads: Monad[]) => {
    setSelectedForDistill(monads);
  }, [setSelectedForDistill]);

  const handleCommit = useCallback(async (filePath: string) => {
    try {
      const ringLevel = await invokeExpand(filePath);
      console.log(`Expanded to ring ${ringLevel}`);
      setShowCommitDialog(false);
      const state = await fetchMandalaState();
      setMandalaState(state);
    } catch (error) {
      console.error('Expand failed:', error);
      alert('Expand failed: ' + String(error));
    }
  }, [setMandalaState]);

  useEffect(() => {
    if (!svgRef.current) return;
    const svg = d3.select(svgRef.current);

    if (viewMode === 'distill') {
      const cleanup = enableLassoSelection(svg, handleSelect);
      return cleanup;
    } else {
      disableLassoSelection(svg);
    }
  }, [viewMode, handleSelect]);

  useEffect(() => {
    if (!svgRef.current || !containerRef.current) return;

    const width = containerRef.current.clientWidth;
    const height = containerRef.current.clientHeight;
    const size = Math.min(width, height) * 0.95;
    
    const svg = d3.select(svgRef.current)
      .attr('viewBox', `-${size/2} -${size/2} ${size} ${size}`)
      .attr('preserveAspectRatio', 'xMidYMid meet')
      .style('width', '100%')
      .style('height', '100%')
      .style('display', 'block');

    svg.selectAll('*').remove();

    const defs = svg.append('defs');
    defs.append('filter')
      .attr('id', 'glow')
      .html(`<feGaussianBlur stdDeviation="2" result="coloredBlur"/><feMerge><feMergeNode in="coloredBlur"/><feMergeNode in="SourceGraphic"/></feMerge>`);

    const contentGroup = svg.append('g').attr('class', 'mandala-content');
    svgGroupRef.current = contentGroup;

    const maxRadius = size * 0.45;
    
    const gridGroup = contentGroup.append('g').attr('class', 'grid-layer');
    drawMandalaGrid(gridGroup as any, maxRadius);
    
    const binduGroup = contentGroup.append('g').attr('class', 'bindu-layer');
    drawBindu(binduGroup);

    const zoomBehavior = setupSemanticZoom(svg, contentGroup);
    zoomBehavior.on('zoom', (event) => {
      contentGroup.attr('transform', event.transform);
      const scale = event.transform.k;
      const mode = getZoomMode(scale);
      setZoomMode(mode);
    });

    svg.on('click.bg', function(event: MouseEvent) {
      const target = event.target as SVGElement;
      if (target && target.closest && !target.closest('.monad') && !target.closest('.bindu-group')) {
        selectMonad(null);
        if (svgGroupRef.current) {
          highlightMonad(svgGroupRef.current as any, null);
        }
      }
    });

    async function loadData() {
      try {
        if (typeof window !== 'undefined') {
          const params = new URLSearchParams(window.location.search);
          const projId = params.get('project') || params.get('id');
          if (projId && isTauri()) {
            try {
              const { getProjectDetail } = await import('../../lib/tauri/synarchy_api');
              const { invoke } = await import('@tauri-apps/api/core');
              const proj = await getProjectDetail(projId);
              if (proj && proj.path) {
                await invoke('open_project', { projectPath: proj.path });
              }
            } catch (e) {
              console.error('Failed to switch project context:', e);
            }
          }
        }

const PHI_CONJUGATE = 0.618033988749895;

/**
 * Computes an adaptive radial band width tailored to ring size and density.
 * As the ring radius grows (outer rings), it can host a wider band safely,
 * but inner rings maintain strict boundaries so they don't engulf the Bindu.
 */
function computeAdaptiveBandWidth(baseRingRadius: number, ringGap: number, totalInRing: number): number {
  if (totalInRing <= 36) return 0;

  const perimeter = 2 * Math.PI * baseRingRadius;
  const linearDensity = totalInRing / perimeter; // nodes per pixel along circumference
  const congestionFactor = Math.max(1.0, linearDensity / 0.07);

  // Desired band width scales smoothly with congestion
  const desiredWidth = (congestionFactor - 1.0) * 18;

  // Maximum allowed width: does NOT collide with adjacent rings (safe margin of 55% of ringGap)
  // and does not exceed 28% of the ring's own radius (protects inner rings)
  const maxSafeWidth = Math.min(ringGap * 0.55, baseRingRadius * 0.28);

  return Math.min(desiredWidth, maxSafeWidth);
}

/**
 * Low-discrepancy Weyl sequence based on the golden ratio conjugate (1 / φ).
 * Successive angular neighbors (idx, idx + 1) jump across opposite sides of the band [-W/2, +W/2],
 * eliminating the visual single-line "particle accelerator" collision.
 */
function computeGoldenRadiusOffset(index: number, bandWidth: number, kind?: string): number {
  if (bandWidth <= 0) return 0;

  const normalized = ((index * PHI_CONJUGATE) % 1); // [0, 1)
  let offset = (normalized - 0.5) * bandWidth; // [-W/2, +W/2]

  // Subtle semantic stratification:
  // Data structures (Struct, Enum) anchor slightly inward
  // Modules anchor near centerline
  if (kind === 'Struct' || kind === 'Enum') {
    offset = offset * 0.85 - (bandWidth * 0.08);
  } else if (kind === 'Module') {
    offset = offset * 0.5;
  }

  return offset;
}

        const state = await fetchMandalaState();
        
        const allMonads = state.constellations.flatMap(c => {
          const totalInRing = c.monads.length;
          const circleRadius = totalInRing > 150 ? 3 : totalInRing > 50 ? 4 : 6;
          const ringGap = 70;
          const baseRingRadius = 50 + (c.ring_level - 1) * ringGap;
          const bandWidth = computeAdaptiveBandWidth(baseRingRadius, ringGap, totalInRing);

          const uniqueThetas = new Set(c.monads.map(item => Math.round(item.coord.theta)));
          const shouldRedistribute = uniqueThetas.size < Math.min(totalInRing, 32);

          return c.monads.map((m, idx) => {
            const angle = shouldRedistribute
              ? ((360 / totalInRing) * idx + ((c.ring_level % 2) * (180 / totalInRing))) % 360
              : m.coord.theta;

            const radiusOffset = computeGoldenRadiusOffset(idx, bandWidth, m.kind);
            // Ensure nodes strictly follow the adaptive golden ratio band within visual grid space
            const r = (m.coord?.r && Math.abs(m.coord.r - baseRingRadius) <= (bandWidth / 2 + 2))
              ? m.coord.r
              : (baseRingRadius + radiusOffset);

            return {
              ...m,
              ring: c.ring_level,
              radius: circleRadius,
              coord: {
                r,
                theta: angle
              }
            };
          });
        });

        const updatedConstellations = state.constellations.map(c => ({
          ...c,
          monads: allMonads.filter(m => m.ring === c.ring_level)
        }));
        const updatedState = {
          ...state,
          total_monads: state.total_monads || allMonads.length,
          constellations: updatedConstellations
        };
        setMandalaState(updatedState);
        
        if (svgGroupRef.current) {
          if (state.edges) {
            renderEdges(svgGroupRef.current as any, state.edges, allMonads);
          }
          renderMonads(svgGroupRef.current as any, allMonads);
          
          svgGroupRef.current.selectAll('.monad')
            .on('click', function(event: MouseEvent, d: Monad) {
              event.stopPropagation();
              selectMonad(d);
              highlightMonad(svgGroupRef.current as any, d.id);
            })
            .on('mouseover', function(_event: MouseEvent, d: Monad) {
              hoverMonad(d);
              highlightMonad(svgGroupRef.current as any, d.id);
            })
            .on('mouseout', function(_event: MouseEvent, d: Monad) {
              hoverMonad(null);
              highlightMonad(svgGroupRef.current as any, null);
            });
        }
      } catch (err) {
        console.error('Failed to load mandala state:', err);
      }
    }

    loadData();

    let unlisten: (() => void) | undefined;
    listenForFileChanges(() => {
      console.log('File changed, reloading state...');
      loadData();
    }).then(fn => { unlisten = fn; });

    (window as any).openCommitDialog = () => setShowCommitDialog(true);

    return () => {
      svg.on('*', null);
      delete (window as any).openCommitDialog;
      if (unlisten) unlisten();
    };
  }, [setMandalaState, selectMonad, hoverMonad, handleSelect]);

  return (
    <>
      <div ref={containerRef} style={{ 
        position: 'absolute',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        display: 'flex', 
        justifyContent: 'center', 
        alignItems: 'center'
      }}>
        <svg ref={svgRef}></svg>
      </div>
      <CommitDialog 
        isOpen={showCommitDialog} 
        onClose={() => setShowCommitDialog(false)}
        onCommit={handleCommit}
      />
    </>
  );
};

export default MandalaCanvas;
