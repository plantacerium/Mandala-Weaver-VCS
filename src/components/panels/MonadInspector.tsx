import React from 'react';
import { useWorkspaceStore } from '../../lib/state/workspaceStore';
import '../../styles/panels/panels.css';

const MonadInspector: React.FC = () => {
  const { selectedMonad, hoveredMonad } = useWorkspaceStore();
  const monad = selectedMonad || hoveredMonad;

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
        <span className="hash">#{monad.id.slice(0, 12)}</span>
        <div className="monad-meta">
          <span className="ring-badge">Anillo {monad.ring}</span>
          <span className="coord-badge">θ: {monad.coord.theta.toFixed(1)}°</span>
        </div>
      </header>
      <div className="code-viewer">
        <pre><code>{monad.content || '// Sin contenido'}</code></pre>
      </div>
    </div>
  );
};

export default MonadInspector;
