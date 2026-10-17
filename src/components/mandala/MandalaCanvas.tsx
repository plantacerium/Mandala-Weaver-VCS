import React, { useEffect, useRef, useCallback, useState } from 'react';
import * as d3 from 'd3';
import { 
  drawMandalaGrid, 
  drawBindu, 
  renderMonads, 
  highlightMonad, 
  renderEdges,
  getGoldenRingBand,
  GOLDEN_ANGLE_DEG
} from '../../lib/d3/renderer';
import { enableLassoSelection, disableLassoSelection } from '../../lib/d3/interactions';
import { fetchMandalaState, invokeExpand } from '../../lib/tauri/commands';
import { listenForFileChanges } from '../../lib/tauri/events';
import { useWorkspaceStore } from '../../lib/state/workspaceStore';
import { setupSemanticZoom, getZoomMode } from '../../lib/d3/zoom';
import { isTauri } from '../../lib/tauri/isTauri';
import type { Monad, Constellation } from '../../types/ontology';

function classifyMonadRingClient(m: Monad): number {
  const lang = (m.language || '').toLowerCase();
  const name = (m.name || '').toLowerCase();
  const content = m.content || '';
  const trimmed = content.trimStart();

  // Ring 5: Sensory Periphery, UI Components, D3 Renderer, Views & Styles
  if (
    ['css', 'scss', 'sass', 'less', 'astro', 'html', 'tsx', 'jsx', 'vue', 'svelte'].includes(lang) ||
    content.includes('React.FC') ||
    content.includes('useState') ||
    content.includes('useEffect') ||
    content.includes('className=') ||
    content.includes('d3.select') ||
    content.includes('.attr(') ||
    (trimmed.startsWith('.') && content.includes('{') && (content.includes('color:') || content.includes('display:') || content.includes('background') || content.includes('padding') || content.includes('border'))) ||
    name.includes('canvas') ||
    name.includes('panel') ||
    name.includes('sidebar') ||
    name.includes('header') ||
    name.includes('dialog') ||
    name.includes('inspector') ||
    name.includes('palette') ||
    name.includes('render') ||
    name.includes('draw') ||
    name.includes('highlight') ||
    name.includes('lasso') ||
    name.includes('zoom')
  ) {
    return 5;
  }

  // Ring 1: Core Ontology, Sacred Geometry, Contracts, Traits, Enums & Configs
  if (
    ['toml', 'json', 'yaml', 'yml'].includes(lang) ||
    m.kind === 'Trait' ||
    m.kind === 'Enum' ||
    m.kind === 'TypeAlias' ||
    trimmed.startsWith('pub enum ') ||
    trimmed.startsWith('enum ') ||
    trimmed.startsWith('pub trait ') ||
    trimmed.startsWith('trait ') ||
    trimmed.startsWith('export interface ') ||
    trimmed.startsWith('interface ') ||
    trimmed.startsWith('export type ') ||
    name.includes('monad') ||
    name.includes('bindu') ||
    name.includes('polar') ||
    name.includes('coord') ||
    name.includes('golden') ||
    name.includes('geodesic') ||
    name.includes('identity') ||
    name.includes('canonical') ||
    name.includes('ontology') ||
    name.includes('delta')
  ) {
    return 1;
  }

  // Ring 4: Interfaces, Tauri IPC, CLI, MCP Server, Watcher & Git Collaboration
  if (
    name.startsWith('cmd_') ||
    name.startsWith('cli_') ||
    name.includes('mcp') ||
    name.includes('api') ||
    name.includes('invoke') ||
    name.includes('fetch') ||
    name.includes('watcher') ||
    name.includes('merge') ||
    name.includes('git') ||
    name.includes('export_') ||
    name.includes('open_project') ||
    name.includes('workspace') ||
    content.includes('#[tauri::command]') ||
    content.includes('jsonrpc')
  ) {
    return 4;
  }

  // Ring 2: Persistence, SurrealDB, Tripartite Store, Templates, Recipes & Structs
  if (
    m.kind === 'Struct' ||
    m.kind === 'Constant' ||
    trimmed.startsWith('pub struct ') ||
    trimmed.startsWith('struct ') ||
    name.includes('store') ||
    name.includes('surreal') ||
    name.includes('db') ||
    name.includes('upsert') ||
    name.includes('insert') ||
    name.includes('query') ||
    name.includes('template') ||
    name.includes('recipe') ||
    name.includes('contract') ||
    name.includes('port') ||
    name.includes('bridge') ||
    name.includes('adapter') ||
    name.includes('plugin') ||
    name.includes('registry') ||
    name.includes('project') ||
    name.includes('detect_')
  ) {
    return 2;
  }

  // Ring 3: Weaver Core Engine, AST Extractor, Compiler, Rewriter & Spatial Resolver
  return 3;
}

function normalizeConstellationsForParity(constellations: Constellation[]): Constellation[] {
  const rawMonads = constellations.flatMap(c =>
    c.monads.map(m => ({ ...m, ring: m.ring || c.ring_level || 1 }))
  );
  if (rawMonads.length === 0) return constellations;

  const maxRing = Math.max(...rawMonads.map(m => m.ring), 0);
  const ringCounts = new Map<number, number>();
  for (const m of rawMonads) {
    ringCounts.set(m.ring, (ringCounts.get(m.ring) || 0) + 1);
  }
  const maxInSingleRing = Math.max(...Array.from(ringCounts.values()), 0);
  const minInAnyRing = ringCounts.size >= 5 ? Math.min(...Array.from(ringCounts.values())) : 0;

  const needsRebalance =
    maxRing > 5 ||
    maxRing === 0 ||
    (rawMonads.length >= 25 &&
      (ringCounts.size < 5 ||
        maxInSingleRing / rawMonads.length > 0.38 ||
        minInAnyRing / rawMonads.length < 0.04));

  let classifiedMonads = rawMonads.map(m => ({
    ...m,
    ring: needsRebalance ? classifyMonadRingClient(m) : m.ring,
  }));

  // If after architectural classification any of the 5 Golden Rings is still sparse (<5%) or overloaded (>36%),
  // apply a deterministic Golden Ratio (φ) harmonic distribution across Rings 1..=5 so every ring has proportional area density.
  if (classifiedMonads.length >= 25) {
    const postCounts = new Map<number, number>();
    for (const m of classifiedMonads) {
      postCounts.set(m.ring, (postCounts.get(m.ring) || 0) + 1);
    }
    const postMax = Math.max(...Array.from(postCounts.values()), 0);
    const hasAllFive = [1, 2, 3, 4, 5].every(r => (postCounts.get(r) || 0) >= Math.max(2, Math.floor(classifiedMonads.length * 0.05)));

    if (!hasAllFive || postMax / classifiedMonads.length > 0.38) {
      // Sort by primary architectural ring (1..5), then deterministically by kind, language, name, id
      const sortedByLayer = [...classifiedMonads].sort((a, b) => {
        if (a.ring !== b.ring) return a.ring - b.ring;
        if ((a.kind || '') !== (b.kind || '')) return String(a.kind).localeCompare(String(b.kind));
        if ((a.language || '') !== (b.language || '')) return (a.language || '').localeCompare(b.language || '');
        if ((a.name || '') !== (b.name || '')) return (a.name || '').localeCompare(b.name || '');
        return (a.id || '').localeCompare(b.id || '');
      });

      // Golden Ratio annular area weights φ^((k-1)*0.32): Ring 1 (~13%), Ring 2 (~16%), Ring 3 (~19%), Ring 4 (~23%), Ring 5 (~29%)
      const weights = [1, 2, 3, 4, 5].map(k => Math.pow(1.61803398875, (k - 1) * 0.32));
      const weightSum = weights.reduce((acc, w) => acc + w, 0);
      const total = sortedByLayer.length;

      let cumulative = 0;
      const thresholds = weights.map((w, idx) => {
        cumulative += w / weightSum;
        return idx === 4 ? total : Math.round(cumulative * total);
      });

      classifiedMonads = sortedByLayer.map((m, idx) => {
        let assignedRing = 5;
        for (let r = 0; r < 5; r++) {
          if (idx < thresholds[r]) {
            assignedRing = r + 1;
            break;
          }
        }
        return { ...m, ring: assignedRing };
      });
    }
  }

  const grouped = new Map<number, Monad[]>();
  for (const m of classifiedMonads) {
    const list = grouped.get(m.ring) || [];
    list.push(m);
    grouped.set(m.ring, list);
  }

  return Array.from(grouped.entries())
    .sort((a, b) => a[0] - b[0])
    .map(([ring_level, monads]) => ({ ring_level, monads }));
}

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

        const state = await fetchMandalaState();
        const normalizedConstellations = normalizeConstellationsForParity(state.constellations || []);
        const maxRingNumber = Math.max(
          ...normalizedConstellations.map(c => c.ring_level),
          4
        );
        const totalRings = Math.max(maxRingNumber, 5);

        // Redraw Golden Ratio background grid and Fibonacci parastichies
        gridGroup.selectAll('*').remove();
        drawMandalaGrid(gridGroup as any, maxRadius, totalRings);

        const allMonads = normalizedConstellations.flatMap(c => {
          // Sort deterministically so the same index `idx` drives BOTH θ_idx and r_idx (Vogel's law)
          const sortedMonads = [...c.monads].sort((a, b) => {
            if ((a.language || '') !== (b.language || '')) return (a.language || '').localeCompare(b.language || '');
            if ((a.kind || '') !== (b.kind || '')) return String(a.kind).localeCompare(String(b.kind));
            if ((a.name || '') !== (b.name || '')) return (a.name || '').localeCompare(b.name || '');
            if ((a.line_start || 0) !== (b.line_start || 0)) return (a.line_start || 0) - (b.line_start || 0);
            return (a.id || '').localeCompare(b.id || '');
          });

          const totalInRing = sortedMonads.length;
          const circleRadius = totalInRing > 250 ? 2.4 : totalInRing > 120 ? 3.1 : totalInRing > 40 ? 4.0 : 5.2;
          const band = getGoldenRingBand(c.ring_level, maxRadius, totalRings);
          const ringPhaseDeg = ((c.ring_level - 1) * 72) % 360;

          return sortedMonads.map((m, idx) => {
            // Coupled Golden Angle (137.507764°) + coupled area-preserving Sunflower Annular Radius
            const angle = ((idx * GOLDEN_ANGLE_DEG) + ringPhaseDeg) % 360;
            const u = (idx + 0.5) / Math.max(1, totalInRing);
            const r = totalInRing <= 6
              ? band.center
              : Math.sqrt(band.rMin * band.rMin + u * (band.rMax * band.rMax - band.rMin * band.rMin));

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

        const updatedConstellations = normalizedConstellations.map(c => ({
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
          binduGroup.raise();
          
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
            .on('mouseout', function(_event: MouseEvent, _d: Monad) {
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
