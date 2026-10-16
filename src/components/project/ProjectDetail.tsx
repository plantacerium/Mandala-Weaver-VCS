import React, { useEffect, useState } from 'react';
import MandalaCanvas from '../mandala/MandalaCanvas';
import { getProjectDetail, rescanProject, getProjectMandala } from '../../lib/tauri/synarchy_api';
import { useWorkspaceStore } from '../../lib/state/workspaceStore';
import type { ProjectEntry } from '../../types/synarchy';
import type { Monad } from '../../types/ontology';
import { isTauri } from '../../lib/tauri/isTauri';

function getProjectIdFromUrl(): string | null {
  if (typeof window === 'undefined') return null;
  const params = new URLSearchParams(window.location.search);
  return params.get('id');
}

function formatLastActivity(ts?: number): string {
  if (!ts || ts <= 0) return 'Reciente';
  const date = new Date(ts * 1000);
  const diffSec = Math.floor((Date.now() - date.getTime()) / 1000);
  if (diffSec < 60) return 'Hace unos segundos';
  if (diffSec < 3600) return `Hace ${Math.floor(diffSec / 60)} min`;
  if (diffSec < 86400) return `Hace ${Math.floor(diffSec / 3600)} h`;
  return date.toLocaleString();
}

const ProjectDetail: React.FC = () => {
  const { mandalaState, setMandalaState, selectMonad } = useWorkspaceStore();
  const [project, setProject] = useState<ProjectEntry | null>(null);
  const [loading, setLoading] = useState(true);
  const [ringA, setRingA] = useState<number>(1);
  const [ringB, setRingB] = useState<number>(2);

  useEffect(() => {
    const id = getProjectIdFromUrl();
    if (!id) {
      setLoading(false);
      return;
    }
    getProjectDetail(id)
      .then(async (proj) => {
        setProject(proj);
        if (proj && isTauri()) {
          const { invoke } = await import('@tauri-apps/api/core');
          await invoke('open_project', { projectPath: proj.path });
          try {
            const stateStr = await getProjectMandala(proj.id);
            const parsed = JSON.parse(stateStr);
            if (parsed && parsed.constellations) {
              setMandalaState(parsed);
            }
          } catch {
            // MandalaCanvas will fetch state directly
          }
        }
      })
      .catch(() => setProject(null))
      .finally(() => setLoading(false));
  }, [setMandalaState]);

  const rings = mandalaState?.constellations || [];

  useEffect(() => {
    if (rings.length >= 2) {
      setRingA(rings[0].ring_level);
      setRingB(rings[rings.length - 1].ring_level);
    } else if (rings.length === 1) {
      setRingA(rings[0].ring_level);
      setRingB(rings[0].ring_level);
    }
  }, [rings.length]);

  if (loading) {
    return (
      <div className="project-detail">
        <div className="detail-header">
          <a href="/explorer" className="back-link">← Back to Explorer</a>
          <h1>Loading...</h1>
        </div>
      </div>
    );
  }

  if (!project) {
    return (
      <div className="project-detail">
        <div className="detail-header">
          <a href="/explorer" className="back-link">← Back to Explorer</a>
          <h1>Project Not Found</h1>
        </div>
      </div>
    );
  }

  const handleRescan = async () => {
    try {
      const updated = await rescanProject(project.id);
      setProject(updated);
    } catch (err) {
      console.error('Rescan failed:', err);
    }
  };

  // Compute Ring Diff between ringA and ringB
  const monadsA: Monad[] = rings.find((r) => r.ring_level === ringA)?.monads || [];
  const monadsB: Monad[] = rings.find((r) => r.ring_level === ringB)?.monads || [];
  const mapA = new Map(monadsA.map((m) => [m.name, m]));
  const mapB = new Map(monadsB.map((m) => [m.name, m]));

  const diffItems: Array<{ name: string; status: 'added' | 'modified' | 'removed' | 'unchanged'; monad: Monad }> = [];
  for (const mB of monadsB) {
    const mA = mapA.get(mB.name);
    if (!mA) diffItems.push({ name: mB.name, status: 'added', monad: mB });
    else if (mA.semantic_hash !== mB.semantic_hash) diffItems.push({ name: mB.name, status: 'modified', monad: mB });
    else diffItems.push({ name: mB.name, status: 'unchanged', monad: mB });
  }
  for (const mA of monadsA) {
    if (!mapB.has(mA.name)) diffItems.push({ name: mA.name, status: 'removed', monad: mA });
  }

  return (
    <div className="project-detail" id="project-detail">
      <div className="detail-header">
        <a href="/explorer" className="back-link">← Back to Explorer</a>
        <h1 id="project-name">{project.name}</h1>
        <a
          href={`/?project=${encodeURIComponent(project.id)}`}
          style={{
            marginLeft: 'auto',
            padding: '6px 14px',
            background: 'rgba(0, 229, 255, 0.12)',
            color: 'var(--accent-primary, #00e5ff)',
            border: '1px solid var(--accent-primary, #00e5ff)',
            borderRadius: '4px',
            textDecoration: 'none',
            fontSize: '0.8rem',
          }}
        >
          Abrir en Workspace
        </a>
        <button onClick={handleRescan} style={{
          padding: '6px 16px',
          background: 'var(--accent-primary, #e85d04)',
          color: '#fff',
          border: 'none',
          borderRadius: '4px',
          cursor: 'pointer',
          fontSize: '0.8rem',
        }}>Rescan</button>
      </div>

      <div className="detail-content">
        <div className="detail-stats">
          <div className="stat-card">
            <span className="value" id="ring-count">{project.ring_count}</span>
            <span className="label">Rings</span>
          </div>
          <div className="stat-card">
            <span className="value" id="monad-count">{project.monad_count}</span>
            <span className="label">Monads</span>
          </div>
          <div className="stat-card">
            <span className="value" id="last-activity" style={{ fontSize: '0.95rem' }}>
              {formatLastActivity(project.last_scanned)}
            </span>
            <span className="label">Last Activity ({project.status})</span>
          </div>
          <div className="stat-card full-width">
            <span className="value" style={{ fontSize: '0.82rem', wordBreak: 'break-all' }} id="project-path">{project.path}</span>
            <span className="label">Path</span>
          </div>

          {/* Visual Ring Diff Comparator (Phase 11.4) */}
          {rings.length > 0 && (
            <div className="stat-card full-width" style={{ textAlign: 'left' }}>
              <span className="label" style={{ display: 'block', marginBottom: '8px', color: 'var(--accent-primary, #00e5ff)' }}>
                ⚖️ Comparador de Anillos (Ring Diff)
              </span>
              <div style={{ display: 'flex', gap: '6px', marginBottom: '8px' }}>
                <select
                  value={ringA}
                  onChange={(e) => setRingA(Number(e.target.value))}
                  style={{ flex: 1, padding: '4px', background: '#0d0d14', color: '#fff', border: '1px solid #333', borderRadius: '4px', fontSize: '0.75rem' }}
                >
                  {rings.map((r) => (
                    <option key={r.ring_level} value={r.ring_level}>R{r.ring_level} ({r.monads.length})</option>
                  ))}
                </select>
                <span style={{ color: '#888', alignSelf: 'center' }}>→</span>
                <select
                  value={ringB}
                  onChange={(e) => setRingB(Number(e.target.value))}
                  style={{ flex: 1, padding: '4px', background: '#0d0d14', color: '#fff', border: '1px solid #333', borderRadius: '4px', fontSize: '0.75rem' }}
                >
                  {rings.map((r) => (
                    <option key={r.ring_level} value={r.ring_level}>R{r.ring_level} ({r.monads.length})</option>
                  ))}
                </select>
              </div>
              <div style={{ display: 'flex', gap: '6px', flexWrap: 'wrap', fontSize: '0.68rem', marginBottom: '8px' }}>
                <span style={{ color: '#2ec4b6' }}>+{diffItems.filter(d => d.status === 'added').length} nuevas</span>
                <span style={{ color: '#ffb703' }}>Δ{diffItems.filter(d => d.status === 'modified').length} mod</span>
                <span style={{ color: '#ff3e83' }}>-{diffItems.filter(d => d.status === 'removed').length} elim</span>
              </div>
              <div style={{ maxHeight: '180px', overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '4px' }}>
                {diffItems.slice(0, 30).map((d, i) => (
                  <div
                    key={`${d.name}-${i}`}
                    onClick={() => selectMonad(d.monad)}
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      fontSize: '0.72rem',
                      padding: '3px 6px',
                      background: 'rgba(255,255,255,0.03)',
                      borderRadius: '3px',
                      cursor: 'pointer',
                      fontFamily: 'monospace',
                    }}
                  >
                    <span style={{ color: '#fff', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{d.name}</span>
                    <span style={{
                      color: d.status === 'added' ? '#2ec4b6' : d.status === 'modified' ? '#ffb703' : d.status === 'removed' ? '#ff3e83' : '#666',
                      fontWeight: 'bold',
                    }}>
                      {d.status === 'added' ? '+' : d.status === 'modified' ? 'Δ' : d.status === 'removed' ? '-' : '='}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>

        <div className="detail-canvas">
          <MandalaCanvas />
        </div>
      </div>
    </div>
  );
};

export default ProjectDetail;

