import React from 'react';
import { useWorkspaceStore } from '../../lib/state/workspaceStore';

const TooltipNode: React.FC = () => {
  const { hoveredMonad } = useWorkspaceStore();
  
  if (!hoveredMonad) return null;
  
  const x = hoveredMonad.coord.r * Math.cos(hoveredMonad.coord.theta * Math.PI / 180);
  const y = hoveredMonad.coord.r * Math.sin(hoveredMonad.coord.theta * Math.PI / 180);
  
  return (
    <div 
      className="monad-tooltip"
      style={{
        position: 'absolute',
        left: `calc(50% + ${x}px + 20px)`,
        top: `calc(50% + ${y}px - 10px)`,
        transform: 'translateY(-100%)',
        pointerEvents: 'none',
        zIndex: 200,
      }}
    >
      <div className="tooltip-content">
        <span className="tooltip-name">{hoveredMonad.name}</span>
        <span className="tooltip-ring" style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
          <span>Anillo {hoveredMonad.ring}</span>
          {hoveredMonad.kind && hoveredMonad.kind !== 'Unknown' && (
            <span style={{ color: 'var(--accent-secondary)' }}>{hoveredMonad.kind}</span>
          )}
        </span>
      </div>
    </div>
  );
};

export default TooltipNode;