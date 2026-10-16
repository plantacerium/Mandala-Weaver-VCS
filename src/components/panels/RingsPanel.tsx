import React, { useEffect, useState } from 'react';
import { useWorkspaceStore } from '../../lib/state/workspaceStore';
import {
  contractOuterRing,
  revertToLevel,
  getArchivedMonadsCount,
  purgeArchivedMonads,
  fetchMandalaState,
} from '../../lib/tauri/commands';
import type { Monad } from '../../types/ontology';
import '../../styles/panels/panels.css';

const RingsPanel: React.FC = () => {
  const { viewMode, mandalaState, setMandalaState, selectMonad, setViewMode } = useWorkspaceStore();
  const [archivedCount, setArchivedCount] = useState<number>(0);
  const [activeTab, setActiveTab] = useState<'list' | 'diff'>('list');
  const [ringA, setRingA] = useState<number>(1);
  const [ringB, setRingB] = useState<number>(2);
  const [panelWidth, setPanelWidth] = useState<number>(420);
  const [statusMsg, setStatusMsg] = useState<string | null>(null);

  const rings = mandalaState?.constellations || [];

  useEffect(() => {
    if (viewMode === 'rings') {
      getArchivedMonadsCount().then(setArchivedCount).catch(() => {});
      if (rings.length >= 2) {
        setRingA(rings[0].ring_level);
        setRingB(rings[rings.length - 1].ring_level);
      } else if (rings.length === 1) {
        setRingA(rings[0].ring_level);
        setRingB(rings[0].ring_level);
      }
    }
  }, [viewMode, rings.length]);

  if (viewMode !== 'rings') {
    return null;
  }

  const refreshAll = async () => {
    const updated = await fetchMandalaState();
    setMandalaState(updated);
    const archived = await getArchivedMonadsCount();
    setArchivedCount(archived);
  };

  const handleContract = async () => {
    try {
      const count = await contractOuterRing();
      await refreshAll();
      setStatusMsg(`✓ Anillo exterior contraído (${count} mónadas archivadas)`);
    } catch (error) {
      console.error('Contract failed', error);
      setStatusMsg(`⚠ Error al contraer: ${String(error)}`);
    }
  };

  const handleRevert = async (level: number) => {
    try {
      const count = await revertToLevel(level);
      await refreshAll();
      setStatusMsg(`✓ Revertido al Anillo ${level} (${count} mónadas archivadas)`);
    } catch (error) {
      console.error('Revert failed', error);
      setStatusMsg(`⚠ Error al revertir: ${String(error)}`);
    }
  };

  const handlePurge = async () => {
    try {
      const purged = await purgeArchivedMonads();
      await refreshAll();
      setStatusMsg(`✓ ${purged} mónadas latentes purgadas permanentemente`);
    } catch (error) {
      console.error('Purge failed', error);
    }
  };

  const startResize = (e: React.MouseEvent) => {
    e.preventDefault();
    const startX = e.clientX;
    const startW = panelWidth;
    const onMove = (moveEvent: MouseEvent) => {
      const next = Math.max(320, Math.min(780, startW + (moveEvent.clientX - startX)));
      setPanelWidth(next);
    };
    const onUp = () => {
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    };
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  };

  // Compute Visual Ring Diff between ringA and ringB
  const monadsA: Monad[] = rings.find((r) => r.ring_level === ringA)?.monads || [];
  const monadsB: Monad[] = rings.find((r) => r.ring_level === ringB)?.monads || [];

  const mapA = new Map(monadsA.map((m) => [m.name, m]));
  const mapB = new Map(monadsB.map((m) => [m.name, m]));

  const diffEntries: Array<{
    name: string;
    status: 'added' | 'modified' | 'unchanged' | 'removed';
    monad: Monad;
  }> = [];

  for (const mB of monadsB) {
    const mA = mapA.get(mB.name);
    if (!mA) {
      diffEntries.push({ name: mB.name, status: 'added', monad: mB });
    } else if (mA.semantic_hash !== mB.semantic_hash) {
      diffEntries.push({ name: mB.name, status: 'modified', monad: mB });
    } else {
      diffEntries.push({ name: mB.name, status: 'unchanged', monad: mB });
    }
  }
  for (const mA of monadsA) {
    if (!mapB.has(mA.name)) {
      diffEntries.push({ name: mA.name, status: 'removed', monad: mA });
    }
  }

  return (
    <div
      className="inspector"
      style={{
        left: '2rem',
        right: 'auto',
        width: `${panelWidth}px`,
        maxHeight: '85vh',
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      {/* Right-edge Resize Handle */}
      <div
        onMouseDown={startResize}
        title="Arrastrar para redimensionar panel"
        style={{
          position: 'absolute',
          top: 0,
          right: 0,
          bottom: 0,
          width: '6px',
          cursor: 'ew-resize',
          background: 'transparent',
        }}
      />

      <header style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', gap: '0.5rem' }}>
        <h2 style={{ margin: 0 }}>Anillos de Expansión</h2>
        <div style={{ display: 'flex', gap: '0.35rem' }}>
          <button
            className="btn-expand"
            style={{
              padding: '0.25rem 0.55rem',
              fontSize: '0.68rem',
              background: 'rgba(255, 62, 131, 0.12)',
              borderColor: 'var(--tertiary, #ff3e83)',
              color: 'var(--tertiary, #ff3e83)',
            }}
            onClick={handleContract}
            title="Contraer (archivar) el anillo más externo"
          >
            ➖ CONTRACT
          </button>
          {archivedCount > 0 && (
            <button
              className="btn-expand"
              style={{
                padding: '0.25rem 0.55rem',
                fontSize: '0.68rem',
                background: 'rgba(255, 183, 3, 0.12)',
                borderColor: '#ffb703',
                color: '#ffb703',
              }}
              onClick={handlePurge}
              title="Purgar permanentemente las mónadas archivadas"
            >
              🧹 PURGAR ({archivedCount})
            </button>
          )}
        </div>
      </header>

      {/* Sub-navigation: Lista de Anillos vs Comparador Visual (Ring Diff) */}
      <div
        style={{
          display: 'flex',
          gap: '0.3rem',
          margin: '0.75rem 0 0.5rem 0',
          background: 'rgba(0,0,0,0.35)',
          padding: '0.2rem',
          borderRadius: '5px',
        }}
      >
        <button
          onClick={() => setActiveTab('list')}
          style={{
            flex: 1,
            padding: '0.35rem',
            fontSize: '0.72rem',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
            background: activeTab === 'list' ? 'var(--accent-primary, #00e5ff)' : 'transparent',
            color: activeTab === 'list' ? '#000' : 'var(--text-dim)',
            fontWeight: activeTab === 'list' ? 'bold' : 'normal',
          }}
        >
          📀 Capas Concéntricas ({rings.length})
        </button>
        <button
          onClick={() => setActiveTab('diff')}
          style={{
            flex: 1,
            padding: '0.35rem',
            fontSize: '0.72rem',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
            background: activeTab === 'diff' ? 'var(--accent-primary, #00e5ff)' : 'transparent',
            color: activeTab === 'diff' ? '#000' : 'var(--text-dim)',
            fontWeight: activeTab === 'diff' ? 'bold' : 'normal',
          }}
        >
          ⚖️ Comparador Ring Diff
        </button>
      </div>

      {statusMsg && (
        <div
          style={{
            fontSize: '0.7rem',
            padding: '0.35rem 0.6rem',
            marginBottom: '0.5rem',
            borderRadius: '4px',
            background: 'rgba(0, 229, 255, 0.1)',
            color: 'var(--accent-primary)',
            border: '1px solid rgba(0, 229, 255, 0.25)',
          }}
        >
          {statusMsg}
        </div>
      )}

      <div className="code-viewer" style={{ flex: 1, maxHeight: '62vh' }}>
        {activeTab === 'list' ? (
          rings.length === 0 ? (
            <p style={{ color: 'var(--text-dim)' }}>No hay anillos disponibles.</p>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column-reverse', gap: '0.75rem' }}>
              {rings.map((ring, idx) => (
                <div
                  key={idx}
                  style={{
                    padding: '0.65rem 0.8rem',
                    background: 'var(--glass-bg)',
                    border: '1px solid var(--glass-border)',
                    borderRadius: '6px',
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                  }}
                >
                  <div>
                    <h3 style={{ margin: '0 0 0.2rem 0', color: 'var(--accent-primary)', fontSize: '0.9rem' }}>
                      Anillo Nivel {ring.ring_level}
                    </h3>
                    <p style={{ margin: 0, fontSize: '0.74rem', color: 'var(--text-dim)' }}>
                      {ring.monads.length} mónada(s) activas
                    </p>
                  </div>
                  <div style={{ display: 'flex', gap: '0.4rem' }}>
                    <button
                      style={{
                        background: 'rgba(0, 229, 255, 0.08)',
                        border: '1px solid var(--accent-primary)',
                        color: 'var(--accent-primary)',
                        padding: '0.25rem 0.5rem',
                        fontSize: '0.65rem',
                        borderRadius: '4px',
                        cursor: 'pointer',
                      }}
                      onClick={() => {
                        setRingB(ring.ring_level);
                        setActiveTab('diff');
                      }}
                    >
                      DIFF
                    </button>
                    <button
                      style={{
                        background: 'transparent',
                        border: '1px solid var(--accent-primary)',
                        color: 'var(--accent-primary)',
                        padding: '0.25rem 0.5rem',
                        fontSize: '0.65rem',
                        borderRadius: '4px',
                        cursor: 'pointer',
                      }}
                      onClick={() => handleRevert(ring.ring_level)}
                    >
                      REVERT
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )
        ) : (
          <div>
            {/* Ring A vs Ring B Selectors */}
            <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center', marginBottom: '0.75rem' }}>
              <label style={{ fontSize: '0.7rem', color: 'var(--text-dim)', flex: 1 }}>
                Base:
                <select
                  value={ringA}
                  onChange={(e) => setRingA(Number(e.target.value))}
                  style={{
                    width: '100%',
                    marginTop: '2px',
                    padding: '0.3rem',
                    background: '#0b0d14',
                    color: '#fff',
                    border: '1px solid var(--glass-border)',
                    borderRadius: '4px',
                    fontFamily: 'monospace',
                  }}
                >
                  {rings.map((r) => (
                    <option key={r.ring_level} value={r.ring_level}>
                      Anillo {r.ring_level} ({r.monads.length})
                    </option>
                  ))}
                </select>
              </label>
              <span style={{ color: 'var(--accent-primary)', fontWeight: 'bold', marginTop: '12px' }}>→</span>
              <label style={{ fontSize: '0.7rem', color: 'var(--text-dim)', flex: 1 }}>
                Comparar:
                <select
                  value={ringB}
                  onChange={(e) => setRingB(Number(e.target.value))}
                  style={{
                    width: '100%',
                    marginTop: '2px',
                    padding: '0.3rem',
                    background: '#0b0d14',
                    color: '#fff',
                    border: '1px solid var(--glass-border)',
                    borderRadius: '4px',
                    fontFamily: 'monospace',
                  }}
                >
                  {rings.map((r) => (
                    <option key={r.ring_level} value={r.ring_level}>
                      Anillo {r.ring_level} ({r.monads.length})
                    </option>
                  ))}
                </select>
              </label>
            </div>

            {/* Diff Summary Pills */}
            <div style={{ display: 'flex', gap: '0.4rem', flexWrap: 'wrap', marginBottom: '0.75rem', fontSize: '0.68rem' }}>
              <span style={{ padding: '2px 7px', borderRadius: '4px', background: 'rgba(46, 196, 182, 0.15)', color: '#2ec4b6' }}>
                + {diffEntries.filter((d) => d.status === 'added').length} nuevas
              </span>
              <span style={{ padding: '2px 7px', borderRadius: '4px', background: 'rgba(255, 183, 3, 0.15)', color: '#ffb703' }}>
                Δ {diffEntries.filter((d) => d.status === 'modified').length} modificadas
              </span>
              <span style={{ padding: '2px 7px', borderRadius: '4px', background: 'rgba(255, 62, 131, 0.15)', color: '#ff3e83' }}>
                - {diffEntries.filter((d) => d.status === 'removed').length} ausentes
              </span>
              <span style={{ padding: '2px 7px', borderRadius: '4px', background: 'rgba(255, 255, 255, 0.06)', color: '#aaa' }}>
                = {diffEntries.filter((d) => d.status === 'unchanged').length} iguales
              </span>
            </div>

            {/* Diff Items List */}
            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.35rem' }}>
              {diffEntries.map((item, idx) => {
                const badgeColor =
                  item.status === 'added'
                    ? '#2ec4b6'
                    : item.status === 'modified'
                    ? '#ffb703'
                    : item.status === 'removed'
                    ? '#ff3e83'
                    : '#888';
                const badgeLabel =
                  item.status === 'added'
                    ? '+ AÑADIDA'
                    : item.status === 'modified'
                    ? 'Δ MODIFICADA'
                    : item.status === 'removed'
                    ? '- AUSENTE'
                    : '= IGUAL';
                return (
                  <div
                    key={`${item.name}-${idx}`}
                    onClick={() => {
                      setViewMode('orbit');
                      selectMonad(item.monad);
                    }}
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      alignItems: 'center',
                      padding: '0.4rem 0.6rem',
                      borderRadius: '4px',
                      background: 'rgba(255,255,255,0.03)',
                      border: '1px solid rgba(255,255,255,0.07)',
                      cursor: 'pointer',
                    }}
                  >
                    <span style={{ fontSize: '0.76rem', color: '#fff', fontFamily: 'monospace' }}>{item.name}</span>
                    <span style={{ fontSize: '0.62rem', color: badgeColor, fontWeight: 'bold', fontFamily: 'monospace' }}>
                      {badgeLabel}
                    </span>
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default RingsPanel;

