import React from 'react';
import { useWorkspaceStore } from '../../lib/state/workspaceStore';
import '../../styles/panels/panels.css';

const MonadInspector: React.FC = () => {
  const { selectedMonad, hoveredMonad, viewMode } = useWorkspaceStore();
  const monad = selectedMonad || hoveredMonad;

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
    </div>
  );
};

export default MonadInspector;
