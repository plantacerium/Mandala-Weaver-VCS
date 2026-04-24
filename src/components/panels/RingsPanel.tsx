import React from 'react';
import { useWorkspaceStore } from '../../lib/state/workspaceStore';
import '../../styles/panels/panels.css';

const RingsPanel: React.FC = () => {
  const { viewMode, mandalaState } = useWorkspaceStore();

  if (viewMode !== 'rings') {
    return null;
  }

  const rings = mandalaState?.constellations || [];

  return (
    <div className="inspector" style={{ left: '2rem', right: 'auto', width: '380px' }}>
      <header>
        <h2>Anillos de Expansión</h2>
      </header>
      <div className="code-viewer">
        {rings.length === 0 ? (
          <p style={{ color: 'var(--text-dim)' }}>No hay anillos disponibles.</p>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
            {rings.map((ring, idx) => (
              <div key={idx} style={{ padding: '0.5rem', background: 'var(--glass-bg)', border: '1px solid var(--glass-border)', borderRadius: '4px' }}>
                <h3 style={{ margin: '0 0 0.5rem 0', color: 'var(--accent-primary)' }}>Anillo Nivel {ring.ring_level}</h3>
                <p style={{ margin: 0, fontSize: '0.85rem', color: 'var(--text-main)' }}>
                  Contiene {ring.monads.length} mónada(s)
                </p>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

export default RingsPanel;
