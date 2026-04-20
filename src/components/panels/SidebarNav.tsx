import React from 'react';
import { useWorkspaceStore } from '../../lib/state/workspaceStore';

const SidebarNav: React.FC = () => {
  const { viewMode, setViewMode } = useWorkspaceStore();

  return (
    <div className="nav-links">
      <button 
        className={viewMode === 'orbit' ? 'active' : ''} 
        onClick={() => setViewMode('orbit')}
      >
        Explorar
      </button>
      <button 
        className={viewMode === 'rings' ? 'active' : ''} 
        onClick={() => setViewMode('rings')}
      >
        Anillos
      </button>
      <button 
        className={viewMode === 'vectors' ? 'active' : ''} 
        onClick={() => setViewMode('vectors')}
      >
        Vectores
      </button>
      <button 
        className={viewMode === 'distill' ? 'active' : ''} 
        onClick={() => setViewMode('distill')}
      >
        Fuentes (Distill)
      </button>
    </div>
  );
};

export default SidebarNav;
