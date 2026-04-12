import React from 'react';
import { useWorkspaceStore } from '../../lib/state/workspaceStore';
import '../../styles/panels/panels.css';

const SidebarHistory: React.FC = () => {
  const { mandalaState } = useWorkspaceStore();
  
  const ringCount = mandalaState?.constellations.length || 0;
  const totalMonads = mandalaState?.constellations.reduce((sum, c) => sum + c.monads.length, 0) || 0;

  return (
    <div className="history-list">
      <h3>HISTORIAL RADIAL</h3>
      <div className="history-item">
        <span className="action">BINDU</span>
        <span className="target">{mandalaState?.bindu_name || 'Genesis'}</span>
        <span className="time">Origen</span>
      </div>
      <div className="history-item">
        <span className="action">ANILLOS</span>
        <span className="target">{ringCount}</span>
        <span className="time">Expansiones</span>
      </div>
      <div className="history-item">
        <span className="action">MÓNADAS</span>
        <span className="target">{totalMonads}</span>
        <span className="time">Unidades</span>
      </div>
    </div>
  );
};

export default SidebarHistory;
