import React from 'react';
import { useWorkspaceStore } from '../../lib/state/workspaceStore';
import '../../styles/panels/panels.css';

const VectorsPanel: React.FC = () => {
  const { viewMode, mandalaState } = useWorkspaceStore();

  if (viewMode !== 'vectors') {
    return null;
  }

  const edges = mandalaState?.edges || [];

  return (
    <div className="inspector" style={{ left: '2rem', right: 'auto', width: '380px' }}>
      <header>
        <h2>Vectores (Lineage Edges)</h2>
      </header>
      <div className="code-viewer">
        {edges.length === 0 ? (
          <p style={{ color: 'var(--text-dim)' }}>No hay vectores disponibles.</p>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
            {edges.map((edge, idx) => (
              <div key={idx} style={{ padding: '0.5rem', background: 'var(--glass-bg)', border: '1px solid var(--glass-border)', borderRadius: '4px' }}>
                <div style={{ fontSize: '0.8rem', color: 'var(--accent-primary)', marginBottom: '0.2rem' }}>
                  Evolución
                </div>
                <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)', fontFamily: 'monospace' }}>
                  De: {edge.parent_id.slice(0, 8)}...<br/>
                  A: {edge.child_id.slice(0, 8)}...
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

export default VectorsPanel;
