import React from 'react';
import { useWorkspaceStore } from '../../lib/state/workspaceStore';
import '../../styles/panels/panels.css';

const RingsPanel: React.FC = () => {
  const { viewMode, mandalaState } = useWorkspaceStore();

  if (viewMode !== 'rings') {
    return null;
  }

  const rings = mandalaState?.constellations || [];

  const handleContract = async () => {
    try {
      if (typeof window !== 'undefined' && '__TAURI__' in window) {
        await invoke('contract_outer_ring');
        // The FS watcher should trigger a reload, but we might want to reload manually too
        window.location.reload(); 
      }
    } catch (error) {
      console.error('Contract failed', error);
    }
  };

  const handleRevert = async (level: number) => {
    try {
      if (typeof window !== 'undefined' && '__TAURI__' in window) {
        await invoke('revert_to_level', { ring: level });
        window.location.reload();
      }
    } catch (error) {
      console.error('Revert failed', error);
    }
  };

  return (
    <div className="inspector" style={{ left: '2rem', right: 'auto', width: '380px' }}>
      <header style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <h2>Anillos de Expansión</h2>
        <button 
          className="btn-expand" 
          style={{ padding: '0.2rem 0.5rem', fontSize: '0.7rem', background: 'rgba(255, 62, 131, 0.1)', borderColor: 'var(--tertiary)' }}
          onClick={handleContract}
        >
          CONTRACT
        </button>
      </header>
      <div className="code-viewer">
        {rings.length === 0 ? (
          <p style={{ color: 'var(--text-dim)' }}>No hay anillos disponibles.</p>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column-reverse', gap: '1rem' }}>
            {rings.map((ring, idx) => (
              <div key={idx} style={{ padding: '0.5rem', background: 'var(--glass-bg)', border: '1px solid var(--glass-border)', borderRadius: '4px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <div>
                  <h3 style={{ margin: '0 0 0.2rem 0', color: 'var(--accent-primary)', fontSize: '0.9rem' }}>Anillo Nivel {ring.ring_level}</h3>
                  <p style={{ margin: 0, fontSize: '0.75rem', color: 'var(--text-dim)' }}>
                    {ring.monads.length} mónada(s)
                  </p>
                </div>
                <button 
                  style={{ background: 'transparent', border: '1px solid var(--accent-primary)', color: 'var(--accent-primary)', padding: '0.2rem 0.4rem', fontSize: '0.65rem', borderRadius: '4px', cursor: 'pointer' }}
                  onClick={() => handleRevert(ring.ring_level)}
                >
                  REVERT
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

export default RingsPanel;
