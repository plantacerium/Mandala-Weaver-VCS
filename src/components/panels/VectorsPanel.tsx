import React, { useMemo, useState } from 'react';
import { useWorkspaceStore } from '../../lib/state/workspaceStore';
import type { Monad } from '../../types/ontology';
import '../../styles/panels/panels.css';

interface SectorInfo {
  id: string;
  label: string;
  rangeLabel: string;
  color: string;
  monads: Monad[];
}

interface RichEdge {
  parentId: string;
  parentName: string;
  parentRing?: number;
  childId: string;
  childName: string;
  childRing?: number;
  kind: 'evolution' | 'call';
}

const SECTORS_DEF = [
  { id: 'CORE', label: 'CORE & ENGINE', rangeLabel: '0° – 72°', min: 0, max: 72, color: '#00e5ff' },
  { id: 'IO', label: 'IO & SHELL', rangeLabel: '72° – 144°', min: 72, max: 144, color: '#7ee787' },
  { id: 'DATA', label: 'DATA & PERSISTENCE', rangeLabel: '144° – 216°', min: 144, max: 216, color: '#ffd700' },
  { id: 'UI', label: 'UI & COMPONENTS', rangeLabel: '216° – 288°', min: 216, max: 288, color: '#ff2d55' },
  { id: 'SERVICES', label: 'SERVICES & CONTRACTS', rangeLabel: '288° – 360°', min: 288, max: 360, color: '#9d4edd' },
];

const VectorsPanel: React.FC = () => {
  const { viewMode, mandalaState, selectMonad } = useWorkspaceStore();
  const [activeTab, setActiveTab] = useState<'sectors' | 'edges'>('sectors');
  const [selectedSector, setSelectedSector] = useState<string>('ALL');
  const [panelWidth, setPanelWidth] = useState<number>(400);

  const allMonads = useMemo(() => {
    if (!mandalaState?.constellations) return [];
    return mandalaState.constellations
      .flatMap(c => c.monads)
      .filter(m => m.name !== 'mandala_state.json');
  }, [mandalaState]);

  const monadMap = useMemo(() => {
    const map = new Map<string, Monad>();
    for (const m of allMonads) {
      map.set(m.id, m);
    }
    return map;
  }, [allMonads]);

  const sectors: SectorInfo[] = useMemo(() => {
    return SECTORS_DEF.map(def => {
      const matching = allMonads.filter(m => {
        const norm = ((m.coord.theta % 360) + 360) % 360;
        return norm >= def.min && norm < def.max;
      });
      return {
        id: def.id,
        label: def.label,
        rangeLabel: def.rangeLabel,
        color: def.color,
        monads: matching,
      };
    });
  }, [allMonads]);

  const richEdges: RichEdge[] = useMemo(() => {
    const result: RichEdge[] = [];
    const seen = new Set<string>();

    // 1. Explicit edges from backend state
    for (const e of mandalaState?.edges || []) {
      const key = `${e.parent_id}->${e.child_id}`;
      if (seen.has(key)) continue;
      seen.add(key);
      const p = monadMap.get(e.parent_id);
      const c = monadMap.get(e.child_id);
      result.push({
        parentId: e.parent_id,
        parentName: p?.name || e.parent_id.slice(0, 10),
        parentRing: p?.ring,
        childId: e.child_id,
        childName: c?.name || e.child_id.slice(0, 10),
        childRing: c?.ring,
        kind: 'evolution',
      });
    }

    // 2. Synthesize cross-ring evolution and outbound-call edges if needed
    const sorted = [...allMonads].sort((a, b) => a.ring - b.ring);
    const byName = new Map<string, Monad>();

    for (const m of sorted) {
      const prev = byName.get(m.name);
      if (prev && prev.id !== m.id) {
        const key = `${prev.id}->${m.id}`;
        if (!seen.has(key)) {
          seen.add(key);
          result.push({
            parentId: prev.id,
            parentName: prev.name,
            parentRing: prev.ring,
            childId: m.id,
            childName: m.name,
            childRing: m.ring,
            kind: 'evolution',
          });
        }
      }

      const calls: string[] = (m as any).outbound_calls || [];
      for (const callee of calls) {
        const target = byName.get(callee);
        if (target && target.id !== m.id) {
          const key = `${target.id}->${m.id}`;
          if (!seen.has(key)) {
            seen.add(key);
            result.push({
              parentId: target.id,
              parentName: target.name,
              parentRing: target.ring,
              childId: m.id,
              childName: m.name,
              childRing: m.ring,
              kind: 'call',
            });
          }
        }
      }

      byName.set(m.name, m);
    }

    return result;
  }, [mandalaState, allMonads, monadMap]);

  if (viewMode !== 'vectors') {
    return null;
  }

  const startResize = (e: React.MouseEvent) => {
    e.preventDefault();
    const startX = e.clientX;
    const startW = panelWidth;
    const onMove = (moveEvent: MouseEvent) => {
      const next = Math.max(320, Math.min(700, startW + (moveEvent.clientX - startX)));
      setPanelWidth(next);
    };
    const onUp = () => {
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    };
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  };

  const displayedMonads =
    selectedSector === 'ALL'
      ? allMonads.slice(0, 60)
      : sectors.find(s => s.id === selectedSector)?.monads || [];

  return (
    <div
      className="inspector"
      style={{ left: '2rem', right: 'auto', width: `${panelWidth}px`, position: 'absolute' }}
    >
      <div
        onMouseDown={startResize}
        title="Arrastra para redimensionar"
        style={{
          position: 'absolute',
          top: 0,
          right: 0,
          width: '6px',
          height: '100%',
          cursor: 'ew-resize',
          zIndex: 20,
        }}
      />
      <header style={{ display: 'flex', flexDirection: 'column', gap: '0.6rem' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <h2 style={{ margin: 0 }}>Vectores & Linaje SO(2)</h2>
          <span
            style={{
              fontSize: '0.7rem',
              padding: '2px 8px',
              borderRadius: '999px',
              background: 'rgba(0, 229, 255, 0.12)',
              color: 'var(--accent-primary)',
              fontFamily: 'var(--font-mono)',
            }}
          >
            {allMonads.length} mónadas • {richEdges.length} vectores
          </span>
        </div>

        <div style={{ display: 'flex', gap: '0.4rem' }}>
          <button
            onClick={() => setActiveTab('sectors')}
            style={{
              flex: 1,
              padding: '0.38rem',
              fontSize: '0.73rem',
              borderRadius: '6px',
              cursor: 'pointer',
              border: '1px solid var(--glass-border)',
              background: activeTab === 'sectors' ? 'var(--accent-primary)' : 'rgba(255,255,255,0.04)',
              color: activeTab === 'sectors' ? '#050508' : 'var(--text-dim)',
              fontWeight: 600,
            }}
          >
            🧭 Sectores SO(2) ({sectors.filter(s => s.monads.length > 0).length})
          </button>
          <button
            onClick={() => setActiveTab('edges')}
            style={{
              flex: 1,
              padding: '0.38rem',
              fontSize: '0.73rem',
              borderRadius: '6px',
              cursor: 'pointer',
              border: '1px solid var(--glass-border)',
              background: activeTab === 'edges' ? 'var(--accent-primary)' : 'rgba(255,255,255,0.04)',
              color: activeTab === 'edges' ? '#050508' : 'var(--text-dim)',
              fontWeight: 600,
            }}
          >
            🔗 Aristas de Linaje ({richEdges.length})
          </button>
        </div>
      </header>

      <div className="code-viewer" style={{ maxHeight: '72vh', overflowY: 'auto' }}>
        {activeTab === 'sectors' ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.45rem' }}>
              {sectors.map(sec => {
                const pct = allMonads.length > 0 ? Math.round((sec.monads.length / allMonads.length) * 100) : 0;
                const isSelected = selectedSector === sec.id;
                return (
                  <div
                    key={sec.id}
                    onClick={() => setSelectedSector(isSelected ? 'ALL' : sec.id)}
                    style={{
                      padding: '0.55rem 0.7rem',
                      borderRadius: '6px',
                      background: isSelected ? 'rgba(0, 229, 255, 0.12)' : 'rgba(255, 255, 255, 0.03)',
                      border: isSelected ? `1px solid ${sec.color}` : '1px solid var(--glass-border)',
                      cursor: 'pointer',
                    }}
                  >
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '4px' }}>
                      <span style={{ fontSize: '0.76rem', fontWeight: 600, color: sec.color }}>
                        {sec.label}
                      </span>
                      <span style={{ fontSize: '0.68rem', color: 'var(--text-dim)', fontFamily: 'var(--font-mono)' }}>
                        {sec.rangeLabel} • {sec.monads.length} ({pct}%)
                      </span>
                    </div>
                    <div style={{ width: '100%', height: '4px', background: 'rgba(255,255,255,0.06)', borderRadius: '2px', overflow: 'hidden' }}>
                      <div style={{ width: `${pct}%`, height: '100%', background: sec.color }} />
                    </div>
                  </div>
                );
              })}
            </div>

            <div style={{ borderTop: '1px solid var(--glass-border)', paddingTop: '0.6rem' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.45rem' }}>
                <span style={{ fontSize: '0.72rem', color: 'var(--text-dim)', textTransform: 'uppercase' }}>
                  {selectedSector === 'ALL' ? 'Mónadas en todos los vectores' : `Sector ${selectedSector}`}
                </span>
                {selectedSector !== 'ALL' && (
                  <button
                    onClick={() => setSelectedSector('ALL')}
                    style={{
                      fontSize: '0.65rem',
                      background: 'transparent',
                      border: 'none',
                      color: 'var(--accent-primary)',
                      cursor: 'pointer',
                    }}
                  >
                    Ver todos
                  </button>
                )}
              </div>

              <div style={{ display: 'flex', flexDirection: 'column', gap: '0.35rem', maxHeight: '32vh', overflowY: 'auto' }}>
                {displayedMonads.map(m => (
                  <div
                    key={m.id}
                    onClick={() => selectMonad(m)}
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      alignItems: 'center',
                      padding: '0.4rem 0.6rem',
                      background: 'rgba(255, 255, 255, 0.025)',
                      border: '1px solid var(--glass-border)',
                      borderRadius: '5px',
                      cursor: 'pointer',
                      fontSize: '0.75rem',
                    }}
                  >
                    <div style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', maxWidth: '62%' }}>
                      <strong style={{ color: 'var(--text-main)', fontFamily: 'var(--font-mono)' }}>{m.name}</strong>
                      <span style={{ marginLeft: '6px', fontSize: '0.65rem', color: 'var(--text-dim)' }}>{m.kind}</span>
                    </div>
                    <span style={{ fontSize: '0.68rem', color: 'var(--accent-primary)', fontFamily: 'var(--font-mono)' }}>
                      R{m.ring} • {Math.round(m.coord.theta)}°
                    </span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        ) : richEdges.length === 0 ? (
          <p style={{ color: 'var(--text-dim)', fontSize: '0.82rem' }}>
            No se detectaron aristas de evolución o llamadas entre anillos en este estado.
          </p>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.45rem' }}>
            {richEdges.slice(0, 120).map((edge, idx) => (
              <div
                key={idx}
                onClick={() => {
                  const target = monadMap.get(edge.childId) || monadMap.get(edge.parentId);
                  if (target) selectMonad(target);
                }}
                style={{
                  padding: '0.55rem 0.7rem',
                  background: 'rgba(255, 255, 255, 0.03)',
                  border: '1px solid var(--glass-border)',
                  borderRadius: '6px',
                  cursor: 'pointer',
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.25rem' }}>
                  <span
                    style={{
                      fontSize: '0.66rem',
                      padding: '1px 6px',
                      borderRadius: '4px',
                      background: edge.kind === 'evolution' ? 'rgba(0, 229, 255, 0.14)' : 'rgba(157, 78, 221, 0.18)',
                      color: edge.kind === 'evolution' ? '#00e5ff' : '#c77dff',
                      fontWeight: 600,
                    }}
                  >
                    {edge.kind === 'evolution' ? 'EVOLUCIÓN RADIAL' : 'PUERTO / LLAMADA'}
                  </span>
                  <span style={{ fontSize: '0.66rem', color: 'var(--text-dim)', fontFamily: 'var(--font-mono)' }}>
                    {edge.parentRing ? `R${edge.parentRing}` : ''} → {edge.childRing ? `R${edge.childRing}` : ''}
                  </span>
                </div>
                <div style={{ fontSize: '0.76rem', color: 'var(--text-main)', fontFamily: 'var(--font-mono)', display: 'flex', alignItems: 'center', gap: '6px' }}>
                  <span style={{ color: 'var(--text-dim)' }}>{edge.parentName}</span>
                  <span style={{ color: 'var(--accent-primary)' }}>→</span>
                  <strong>{edge.childName}</strong>
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
