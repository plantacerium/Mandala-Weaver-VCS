import React, { useEffect, useRef, useState } from 'react';
import { useWorkspaceStore } from '../../lib/state/workspaceStore';
import {
  searchMonads,
  contractOuterRing,
  purgeArchivedMonads,
  fetchMandalaState,
} from '../../lib/tauri/commands';
import type { Monad } from '../../types/ontology';

const CommandPalette: React.FC = () => {
  const { mandalaState, setMandalaState, selectMonad, setViewMode } = useWorkspaceStore();
  const [isOpen, setIsOpen] = useState(false);
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<Monad[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const allLocalMonads: Monad[] = React.useMemo(() => {
    if (!mandalaState?.constellations) return [];
    return mandalaState.constellations.flatMap((c) => c.monads);
  }, [mandalaState]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setIsOpen((prev) => !prev);
      } else if (e.key === 'Escape') {
        setIsOpen(false);
      }
    };

    const handleCustomOpen = () => {
      setIsOpen(true);
    };

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('mandala:open-command-palette', handleCustomOpen);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('mandala:open-command-palette', handleCustomOpen);
    };
  }, []);

  useEffect(() => {
    if (isOpen) {
      setQuery('');
      setSelectedIndex(0);
      setStatusMessage(null);
      setResults(allLocalMonads.slice(0, 25));
      setTimeout(() => inputRef.current?.focus(), 20);
    }
  }, [isOpen, allLocalMonads]);

  useEffect(() => {
    if (!isOpen) return;
    let cancelled = false;
    searchMonads(query, allLocalMonads).then((res) => {
      if (!cancelled) {
        setResults(res.slice(0, 40));
        setSelectedIndex(0);
      }
    });
    return () => {
      cancelled = true;
    };
  }, [query, isOpen, allLocalMonads]);

  if (!isOpen) return null;

  const handleSelectMonad = (monad: Monad) => {
    setViewMode('orbit');
    selectMonad(monad);
    setIsOpen(false);
  };

  const handleQuickAction = async (action: string) => {
    try {
      if (action === 'contract') {
        const count = await contractOuterRing();
        const nextState = await fetchMandalaState();
        setMandalaState(nextState);
        setStatusMessage(`✓ Anillo exterior contraído (${count} mónadas archivadas)`);
      } else if (action === 'purge') {
        const count = await purgeArchivedMonads();
        const nextState = await fetchMandalaState();
        setMandalaState(nextState);
        setStatusMessage(`✓ ${count} mónadas latentes purgadas permanentemente`);
      } else if (action.startsWith('view:')) {
        const mode = action.split(':')[1] as 'orbit' | 'rings' | 'vectors' | 'distill';
        setViewMode(mode);
        setIsOpen(false);
      }
    } catch (err) {
      setStatusMessage(`⚠ Error: ${String(err)}`);
    }
  };

  const handleInputKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex((prev) => (results.length ? (prev + 1) % results.length : 0));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex((prev) => (results.length ? (prev - 1 + results.length) % results.length : 0));
    } else if (e.key === 'Enter' && results[selectedIndex]) {
      e.preventDefault();
      handleSelectMonad(results[selectedIndex]);
    }
  };

  return (
    <div
      className="dialog-overlay"
      onClick={() => setIsOpen(false)}
      style={{ zIndex: 2000, alignItems: 'flex-start', paddingTop: '10vh' }}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          width: '620px',
          maxWidth: '92vw',
          background: 'rgba(10, 12, 22, 0.96)',
          backdropFilter: 'blur(24px)',
          border: '1px solid var(--accent-primary, #00e5ff)',
          borderRadius: '10px',
          boxShadow: '0 24px 80px rgba(0, 0, 0, 0.9), 0 0 30px rgba(0, 229, 255, 0.15)',
          overflow: 'hidden',
          fontFamily: 'var(--font-mono, monospace)',
        }}
      >
        {/* Search Input Header */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '0.75rem',
            padding: '0.9rem 1.2rem',
            borderBottom: '1px solid rgba(255, 255, 255, 0.1)',
          }}
        >
          <span style={{ color: 'var(--accent-primary, #00e5ff)', fontSize: '1.1rem' }}>⌕</span>
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleInputKeyDown}
            placeholder="Buscar mónada, contrato, lenguaje (ej. rust, ts) o anillo (ej. r1)..."
            style={{
              flex: 1,
              background: 'transparent',
              border: 'none',
              outline: 'none',
              color: '#fff',
              fontSize: '0.95rem',
              fontFamily: 'inherit',
            }}
          />
          <kbd
            style={{
              fontSize: '0.68rem',
              padding: '0.2rem 0.45rem',
              background: 'rgba(255,255,255,0.08)',
              border: '1px solid rgba(255,255,255,0.18)',
              borderRadius: '4px',
              color: 'var(--text-dim, #888)',
            }}
          >
            ESC
          </kbd>
        </div>

        {/* Quick Actions Bar */}
        <div
          style={{
            display: 'flex',
            flexWrap: 'wrap',
            gap: '0.4rem',
            padding: '0.55rem 1.2rem',
            background: 'rgba(0, 0, 0, 0.35)',
            borderBottom: '1px solid rgba(255, 255, 255, 0.06)',
          }}
        >
          <button
            onClick={() => handleQuickAction('view:orbit')}
            style={quickActionBtnStyle}
          >
            🌐 Órbita
          </button>
          <button
            onClick={() => handleQuickAction('view:rings')}
            style={quickActionBtnStyle}
          >
            ⭕ Anillos & Diff
          </button>
          <button
            onClick={() => handleQuickAction('view:distill')}
            style={quickActionBtnStyle}
          >
            🧭 Weaver SO(2)
          </button>
          <button
            onClick={() => handleQuickAction('contract')}
            style={{ ...quickActionBtnStyle, borderColor: 'rgba(255, 62, 131, 0.4)', color: '#ff3e83' }}
          >
            ➖ Contraer Anillo
          </button>
          <button
            onClick={() => handleQuickAction('purge')}
            style={{ ...quickActionBtnStyle, borderColor: 'rgba(255, 183, 3, 0.4)', color: '#ffb703' }}
          >
            🧹 Purgar Latentes
          </button>
        </div>

        {statusMessage && (
          <div
            style={{
              padding: '0.45rem 1.2rem',
              fontSize: '0.75rem',
              background: 'rgba(0, 229, 255, 0.1)',
              color: 'var(--accent-primary, #00e5ff)',
              borderBottom: '1px solid rgba(0, 229, 255, 0.2)',
            }}
          >
            {statusMessage}
          </div>
        )}

        {/* Results List */}
        <div style={{ maxHeight: '380px', overflowY: 'auto', padding: '0.5rem' }}>
          {results.length === 0 ? (
            <div style={{ padding: '2rem', textAlign: 'center', color: 'var(--text-dim, #888)', fontSize: '0.82rem' }}>
              No se encontraron mónadas para &ldquo;{query}&rdquo;
            </div>
          ) : (
            results.map((m, idx) => {
              const isSelected = idx === selectedIndex;
              return (
                <div
                  key={`${m.id}-${idx}`}
                  onClick={() => handleSelectMonad(m)}
                  onMouseEnter={() => setSelectedIndex(idx)}
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    padding: '0.6rem 0.9rem',
                    borderRadius: '6px',
                    cursor: 'pointer',
                    background: isSelected ? 'rgba(0, 229, 255, 0.14)' : 'transparent',
                    border: isSelected
                      ? '1px solid rgba(0, 229, 255, 0.35)'
                      : '1px solid transparent',
                    transition: 'all 0.1s ease',
                  }}
                >
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '2px', overflow: 'hidden' }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                      <span style={{ color: '#fff', fontWeight: 600, fontSize: '0.86rem' }}>{m.name}</span>
                      {m.kind && m.kind !== 'Unknown' && (
                        <span
                          style={{
                            fontSize: '0.62rem',
                            padding: '1px 6px',
                            borderRadius: '3px',
                            background: 'rgba(157, 78, 221, 0.2)',
                            color: '#c77dff',
                            border: '1px solid rgba(157, 78, 221, 0.35)',
                          }}
                        >
                          {m.kind}
                        </span>
                      )}
                      {m.language && (
                        <span style={{ fontSize: '0.62rem', color: 'var(--text-dim, #888)', textTransform: 'uppercase' }}>
                          {m.language}
                        </span>
                      )}
                    </div>
                    <span
                      style={{
                        fontSize: '0.7rem',
                        color: 'var(--text-dim, #888)',
                        whiteSpace: 'nowrap',
                        overflow: 'hidden',
                        textOverflow: 'ellipsis',
                        maxWidth: '420px',
                      }}
                    >
                      {m.identity?.canonical_signature || (m.content ? m.content.split('\n')[0] : '')}
                    </span>
                  </div>

                  <div style={{ display: 'flex', alignItems: 'center', gap: '0.45rem', flexShrink: 0 }}>
                    <span
                      style={{
                        fontSize: '0.65rem',
                        padding: '2px 7px',
                        borderRadius: '4px',
                        background: 'rgba(0, 229, 255, 0.1)',
                        color: 'var(--accent-primary, #00e5ff)',
                        border: '1px solid rgba(0, 229, 255, 0.25)',
                      }}
                    >
                      R{m.ring} • {m.coord.theta.toFixed(0)}°
                    </span>
                    <span style={{ fontSize: '0.65rem', color: 'var(--text-dim, #888)' }}>
                      L{m.line_start}-{m.line_end}
                    </span>
                  </div>
                </div>
              );
            })
          )}
        </div>

        {/* Footer Hints */}
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            padding: '0.5rem 1.2rem',
            background: 'rgba(0, 0, 0, 0.5)',
            borderTop: '1px solid rgba(255, 255, 255, 0.08)',
            fontSize: '0.68rem',
            color: 'var(--text-dim, #888)',
          }}
        >
          <span>↑↓ Navegar • ↵ Enfocar Mónada • ESC Cerrar</span>
          <span>{results.length} resultado(s)</span>
        </div>
      </div>
    </div>
  );
};

const quickActionBtnStyle: React.CSSProperties = {
  fontSize: '0.68rem',
  padding: '0.25rem 0.55rem',
  borderRadius: '4px',
  background: 'rgba(255, 255, 255, 0.05)',
  border: '1px solid rgba(0, 229, 255, 0.3)',
  color: 'var(--accent-primary, #00e5ff)',
  cursor: 'pointer',
  fontFamily: 'var(--font-mono, monospace)',
};

export default CommandPalette;
