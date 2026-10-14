import React, { useEffect, useState } from 'react';
import { useWorkspaceStore } from '../../lib/state/workspaceStore';
import { fetchLineageEdges } from '../../lib/tauri/commands';
import '../../styles/panels/panels.css';

const MonadInspector: React.FC = () => {
  const { selectedMonad, hoveredMonad, viewMode, lineageCache, setLineageCache, selectMonad } = useWorkspaceStore();
  
  const [isCollapsed, setIsCollapsed] = useState<boolean>(false);
  const [isZenExpanded, setIsZenExpanded] = useState<boolean>(false);
  const [copied, setCopied] = useState<boolean>(false);
  const [showLineage, setShowLineage] = useState<boolean>(false);

  // When selectedMonad changes, ensure panel is open
  useEffect(() => {
    if (selectedMonad) {
      setIsCollapsed(false);
    }
  }, [selectedMonad?.id]);

  const activeMonad = selectedMonad || hoveredMonad;

  useEffect(() => {
    if (activeMonad && !lineageCache.has(activeMonad.id)) {
      fetchLineageEdges(activeMonad.id).then((data) => {
        setLineageCache(activeMonad.id, data);
      }).catch(() => {});
    }
  }, [activeMonad, lineageCache, setLineageCache]);

  if (viewMode !== 'orbit') {
    return null;
  }

  // 1. Case: Selected Monad exists but user chose to collapse the panel (100% Mandala screen)
  if (selectedMonad && isCollapsed) {
    return (
      <button
        onClick={() => setIsCollapsed(false)}
        className="inspector-pill-trigger"
        title="Desplegar vista de mónada al 30%"
      >
        <span style={{ fontSize: '0.85rem' }}>◀</span>
        <span style={{ fontWeight: 'bold' }}>{selectedMonad.name}</span>
        <span style={{ opacity: 0.6, fontSize: '0.68rem' }}>(Desplegar 30% Zen)</span>
      </button>
    );
  }

  // 2. Case: Selected Monad active in Split View (70% / 30% or 75% Zen Fullscreen)
  if (selectedMonad) {
    const lineage = lineageCache.get(selectedMonad.id);
    const codeLines = (selectedMonad.content || '// Sin contenido').split('\n');
    const hasValidIdentity = selectedMonad.identity && 
      selectedMonad.identity.contract_hash && 
      selectedMonad.identity.contract_hash.trim() !== '';

    const copyCode = () => {
      navigator.clipboard.writeText(selectedMonad.content || '');
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    };

    return (
      <aside className={`inspector-split-dock ${isZenExpanded ? 'zen-expanded' : ''}`}>
        {/* Controls Bar */}
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.6rem', borderBottom: '1px solid rgba(255,255,255,0.08)', paddingBottom: '0.6rem' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.4rem' }}>
            <span style={{ fontSize: '0.7rem', color: 'var(--accent-primary)', textTransform: 'uppercase', letterSpacing: '1px', fontWeight: 'bold' }}>
              {isZenExpanded ? 'Zen Mode (75%)' : 'Split View (30%)'}
            </span>
          </div>

          <div style={{ display: 'flex', gap: '0.35rem', alignItems: 'center' }}>
            {/* Copy Button */}
            <button
              onClick={copyCode}
              title="Copiar código al portapapeles"
              style={{
                fontSize: '0.68rem',
                padding: '0.25rem 0.5rem',
                background: 'rgba(255,255,255,0.06)',
                border: '1px solid rgba(255,255,255,0.15)',
                color: '#fff',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              {copied ? '✓ Copiado' : '📋 Copiar'}
            </button>

            {/* Zen Expand/Contract Button */}
            <button
              onClick={() => setIsZenExpanded(!isZenExpanded)}
              title={isZenExpanded ? "Reducir a 30% Split" : "Expandir a 75% Zen"}
              style={{
                fontSize: '0.68rem',
                padding: '0.25rem 0.5rem',
                background: 'rgba(255,255,255,0.06)',
                border: '1px solid rgba(255,255,255,0.15)',
                color: '#fff',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              {isZenExpanded ? '⤡ 30%' : '⤢ Zen'}
            </button>

            {/* Hide/Collapse Button (Return Mandala to 100%) */}
            <button
              onClick={() => setIsCollapsed(true)}
              title="Ocultar panel para ver el Mandala al 100%"
              style={{
                fontSize: '0.68rem',
                padding: '0.25rem 0.55rem',
                background: 'rgba(0, 229, 255, 0.1)',
                border: '1px solid var(--accent-primary)',
                color: 'var(--accent-primary)',
                borderRadius: '4px',
                cursor: 'pointer',
                fontWeight: 'bold',
              }}
            >
              ▶ Ocultar
            </button>

            {/* Close / Deselect Button */}
            <button
              onClick={() => selectMonad(null)}
              title="Cerrar selección"
              style={{
                fontSize: '1rem',
                padding: '0 0.4rem',
                background: 'transparent',
                border: 'none',
                color: 'var(--text-dim)',
                cursor: 'pointer',
                lineHeight: '1',
              }}
            >
              ✕
            </button>
          </div>
        </div>

        {/* Monad Header & Metadata */}
        <header style={{ marginBottom: '0.5rem' }}>
          <h2 style={{ margin: '0 0 0.2rem 0', fontSize: '1.2rem', color: 'var(--accent-primary)', textShadow: '0 0 10px rgba(0, 229, 255, 0.3)' }}>
            {selectedMonad.name}
          </h2>
          <span className="hash" style={{ margin: '0 0 0.5rem 0', fontSize: '0.72rem', color: 'var(--text-dim)' }}>
            #{selectedMonad.semantic_hash ? selectedMonad.semantic_hash.slice(0, 16) : selectedMonad.id.slice(0, 16)}
          </span>

          <div className="monad-meta" style={{ display: 'flex', alignItems: 'center', flexWrap: 'wrap', gap: '0.4rem', margin: '0.4rem 0' }}>
            {selectedMonad.kind && selectedMonad.kind !== 'Unknown' && (
              <span className="kind-badge">{selectedMonad.kind}</span>
            )}
            <span className="ring-badge">Anillo {selectedMonad.ring}</span>
            <span className="coord-badge">θ: {selectedMonad.coord.theta.toFixed(1)}°</span>
            {selectedMonad.language && (
              <span style={{ fontSize: '0.65rem', padding: '0.25rem 0.5rem', borderRadius: '4px', background: 'rgba(255,255,255,0.06)', border: '1px solid rgba(255,255,255,0.1)', color: '#ccc', fontFamily: 'monospace' }}>
                {selectedMonad.language.toUpperCase()} | L: {selectedMonad.line_start} - {selectedMonad.line_end} ({codeLines.length} líneas)
              </span>
            )}
          </div>

          {/* Ontological Identity Details (Only if present and valid) */}
          {hasValidIdentity && (
            <div style={{ marginTop: '0.4rem', padding: '0.4rem 0.6rem', background: 'rgba(0, 229, 255, 0.05)', borderRadius: '4px', border: '1px solid rgba(0, 229, 255, 0.2)', fontSize: '0.68rem', fontFamily: 'monospace' }}>
              <div style={{ color: 'var(--accent-primary)', fontWeight: 'bold', marginBottom: '2px' }}>
                {selectedMonad.identity?.canonical_signature || `fn ${selectedMonad.name}`}
              </div>
              <div style={{ color: 'var(--text-dim)', display: 'flex', justifyContent: 'space-between', gap: '0.5rem' }}>
                <span>Contrato: <span style={{ color: '#fff' }}>#{selectedMonad.identity?.contract_hash.slice(0, 8)}</span></span>
                <span>Impl: <span style={{ color: '#fff' }}>#{selectedMonad.identity?.implementation_hash.slice(0, 8)}</span></span>
              </div>
            </div>
          )}
        </header>

        {/* Zen Code Viewer with Line Numbers (Full Height) */}
        <div className="code-viewer-zen">
          <div className="gutter">
            {codeLines.map((_, i) => (
              <div key={i}>{i + 1}</div>
            ))}
          </div>
          <div className="code-body">
            <pre><code>{selectedMonad.content || '// Sin contenido'}</code></pre>
          </div>
        </div>

        {/* Lineage Section Toggle */}
        {lineage && lineage.monads.length > 0 && (
          <div style={{ marginTop: '0.6rem', borderTop: '1px solid rgba(255,255,255,0.08)', paddingTop: '0.4rem' }}>
            <button
              onClick={() => setShowLineage(!showLineage)}
              style={{
                width: '100%',
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                background: 'transparent',
                border: 'none',
                color: 'var(--text-dim)',
                fontSize: '0.72rem',
                cursor: 'pointer',
                padding: '0.2rem 0',
                fontFamily: 'monospace',
                textTransform: 'uppercase',
                letterSpacing: '0.5px',
              }}
            >
              <span>Linaje Evolutivo ({lineage.depth} niveles)</span>
              <span>{showLineage ? '▼' : '▶'}</span>
            </button>

            {showLineage && (
              <div style={{ maxHeight: '120px', overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '3px', marginTop: '0.4rem' }}>
                {lineage.monads.slice(0, 10).map((ancestor) => (
                  <div key={ancestor.id} style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    padding: '3px 6px',
                    borderRadius: '3px',
                    background: ancestor.id === selectedMonad.id ? 'rgba(0, 229, 255, 0.15)' : 'rgba(255,255,255,0.02)',
                    fontSize: '0.68rem',
                    fontFamily: 'monospace',
                  }}>
                    <span style={{ color: ancestor.id === selectedMonad.id ? 'var(--accent-primary)' : 'var(--text-main, #fff)' }}>
                      {ancestor.name}
                    </span>
                    <span style={{ color: 'var(--text-dim, #888)' }}>R{ancestor.ring}</span>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </aside>
    );
  }

  // 3. Case: Hovering over a Monad without clicking (Compact floating preview card)
  if (hoveredMonad) {
    const codeLines = (hoveredMonad.content || '').split('\n');
    return (
      <div className="inspector" style={{ pointerEvents: 'none' }}>
        <header>
          <h2>{hoveredMonad.name}</h2>
          <span className="hash">#{hoveredMonad.semantic_hash ? hoveredMonad.semantic_hash.slice(0, 12) : hoveredMonad.id.slice(0, 12)}</span>
          <div className="monad-meta">
            {hoveredMonad.kind && hoveredMonad.kind !== 'Unknown' && (
              <span className="kind-badge">{hoveredMonad.kind}</span>
            )}
            <span className="ring-badge">Anillo {hoveredMonad.ring}</span>
            <span className="coord-badge">θ: {hoveredMonad.coord.theta.toFixed(1)}°</span>
          </div>
        </header>
        <div className="code-viewer" style={{ maxHeight: '180px' }}>
          <pre><code>{codeLines.slice(0, 12).join('\n')}</code></pre>
          {codeLines.length > 12 && (
            <div style={{ color: 'var(--text-dim)', fontSize: '0.65rem', marginTop: '4px' }}>
              ... ({codeLines.length - 12} líneas más)
            </div>
          )}
        </div>
        <div style={{ marginTop: '0.6rem', fontSize: '0.68rem', color: 'var(--accent-primary)', textAlign: 'center', fontStyle: 'italic' }}>
          👉 Click para abrir en Split View 70/30 Zen
        </div>
      </div>
    );
  }

  // 4. Case: Nothing hovered and nothing selected
  return null;
};

export default MonadInspector;
