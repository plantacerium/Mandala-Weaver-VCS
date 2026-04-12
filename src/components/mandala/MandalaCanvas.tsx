import React, { useEffect, useRef, useCallback, useState } from 'react';
import * as d3 from 'd3';
import { drawMandalaGrid, drawBindu, renderMonads, highlightMonad } from '../../lib/d3/renderer';
import { setupZoom, enableLassoSelection } from '../../lib/d3/interactions';
import { fetchMandalaState } from '../../lib/tauri/commands';
import { useWorkspaceStore } from '../../lib/state/workspaceStore';
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
  const { setMandalaState, selectMonad, hoverMonad } = useWorkspaceStore();
  const [showCommitDialog, setShowCommitDialog] = useState(false);
  const svgGroupRef = useRef<d3.Selection<SVGGElement, unknown, null, undefined> | null>(null);

  const handleSelect = useCallback((monads: Monad[]) => {
    console.log('Selected monads:', monads);
  }, []);

  const handleCommit = useCallback(async (filePath: string) => {
    console.log('Expanding with file:', filePath);
    setShowCommitDialog(false);
  }, []);

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

    setupZoom(svg, { minZoom: 0.3, maxZoom: 5 });
    enableLassoSelection(svg, handleSelect);

    async function loadData() {
      try {
        const state = await fetchMandalaState();
        setMandalaState(state);
        
        const allMonads = state.constellations.flatMap(c => 
          c.monads.map(m => ({
            ...m,
            ring: c.ring_level,
            coord: { r: c.ring_level * 70, theta: m.coord.theta }
          }))
        );
        
        if (svgGroupRef.current) {
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

    (window as any).openCommitDialog = () => setShowCommitDialog(true);

    return () => {
      svg.on('*', null);
      delete (window as any).openCommitDialog;
    };
  }, [setMandalaState, selectMonad, hoverMonad, handleSelect]);

  return (
    <>
      <div ref={containerRef} style={{ 
        width: '100%', 
        height: '100%', 
        display: 'flex', 
        justifyContent: 'center', 
        alignItems: 'center',
        position: 'relative'
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
