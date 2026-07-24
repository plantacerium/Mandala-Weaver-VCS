import React, { useEffect } from 'react';
import { useWorkspaceStore } from '../../lib/state/workspaceStore';
import { fetchLineageEdges } from '../../lib/tauri/commands';
import '../../styles/panels/panels.css';

const MonadInspector: React.FC = () => {
  const { selectedMonad, hoveredMonad, viewMode, lineageCache, setLineageCache } = useWorkspaceStore();
  const monad = selectedMonad || hoveredMonad;

  useEffect(() => {
    if (monad && !lineageCache.has(monad.id)) {
      fetchLineageEdges(monad.id).then((data) => {
        setLineageCache(monad.id, data);
      }).catch(() => {});
    }
  }, [monad, lineageCache, setLineageCache]);

  if (viewMode !== 'orbit') {
    return null;
  }

  if (!monad) {
    return (
      <div className="inspector">
        <header>
          <h2>Inspeccionar Mónada</h2>
        </header>
        <div className="code-viewer">
          <p style={{ color: 'var(--text-dim)' }}>Selecciona una mónada en el lienzo para inspeccionar</p>
        </div>
      </div>
    );
  }

  const lineage = lineageCache.get(monad.id);

  return (
    <div className="inspector">
      <header>
        <h2>{monad.name}</h2>
        <span className="hash">#{monad.semantic_hash ? monad.semantic_hash.slice(0, 12) : monad.id.slice(0, 12)}</span>
        <div className="monad-meta">
          {monad.kind && monad.kind !== 'Unknown' && (
            <span className="kind-badge">{monad.kind}</span>
          )}
          <span className="ring-badge">Anillo {monad.ring}</span>
          <span className="coord-badge">θ: {monad.coord.theta.toFixed(1)}°</span>
        </div>
        {monad.language && (
            <div style={{ fontSize: '0.65rem', color: 'var(--text-dim)', marginTop: '0.5rem', fontFamily: 'monospace' }}>
                {monad.language.toUpperCase()} | L: {monad.line_start} - {monad.line_end}
            </div>
        )}
      </header>
      <div className="code-viewer">
        <pre><code>{monad.content || '// Sin contenido'}</code></pre>
      </div>
      {lineage && lineage.monads.length > 0 && (
        <div style={{ padding: '0.75rem', borderTop: '1px solid var(--border)' }}>
          <h3 style={{ fontSize: '0.75rem', color: 'var(--text-dim)', marginBottom: '0.5rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
            Linaje ({lineage.depth} niveles)
          </h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
            {lineage.monads.slice(0, 10).map((ancestor) => (
              <div key={ancestor.id} style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                padding: '4px 8px',
                borderRadius: '4px',
                background: ancestor.id === monad.id ? 'var(--accent-primary-dim, rgba(232,93,4,0.15))' : 'transparent',
                fontSize: '0.7rem',
                fontFamily: 'monospace',
              }}>
                <span style={{ color: 'var(--text-main, #fff)' }}>{ancestor.name}</span>
                <span style={{ color: 'var(--text-dim, #666)' }}>R{ancestor.ring}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default MonadInspector;
