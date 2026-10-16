import React, { useEffect, useState } from 'react';
import { useWorkspaceStore } from '../../lib/state/workspaceStore';
import { contractOuterRing, fetchMandalaState } from '../../lib/tauri/commands';
import ThemeSelector from './ThemeSelector';

export default function WorkspaceHeader() {
  const { mandalaState, setMandalaState } = useWorkspaceStore();
  const [activePath, setActivePath] = useState<string>('~/mandala-weaver-vcs');
  const [isContracting, setIsContracting] = useState(false);

  useEffect(() => {
    if (typeof window !== 'undefined') {
      const params = new URLSearchParams(window.location.search);
      const projId = params.get('project') || params.get('id');
      if (projId) {
        import('../../lib/tauri/synarchy_api').then(({ getProjectDetail }) => {
          getProjectDetail(projId).then((p) => {
            if (p && p.path) {
              setActivePath(p.path);
            }
          }).catch(() => {});
        });
      }
    }
  }, []);

  const handleOpenSearch = () => {
    if (typeof window !== 'undefined') {
      window.dispatchEvent(new CustomEvent('mandala:open-command-palette'));
    }
  };

  const handleContractRing = async () => {
    setIsContracting(true);
    try {
      await contractOuterRing();
      const updated = await fetchMandalaState();
      setMandalaState(updated);
    } catch (err) {
      console.error('Contract outer ring failed:', err);
    } finally {
      setIsContracting(false);
    }
  };

  const projectName = mandalaState?.bindu_name && mandalaState.bindu_name !== 'Unnamed_Project'
    ? mandalaState.bindu_name
    : 'Mandala Weaver Versions Cooperation System';

  return (
    <header className="workspace-header">
      <div className="project-info">
        <h1 id="active-project-name">{projectName}</h1>
        <span className="path" id="active-project-path">
          {activePath}
          {mandalaState?.total_monads ? ` • ${mandalaState.total_monads} mónadas (${mandalaState.constellations?.length || 0} anillos)` : ''}
        </span>
      </div>
      <div className="actions" style={{ display: 'flex', alignItems: 'center', gap: '0.6rem' }}>
        <button
          onClick={handleOpenSearch}
          title="Buscar mónadas, contratos o ejecutar acciones (Ctrl+K)"
          style={{
            padding: '0.45rem 0.85rem',
            background: 'rgba(0, 229, 255, 0.08)',
            border: '1px solid rgba(0, 229, 255, 0.3)',
            borderRadius: '4px',
            color: 'var(--accent-primary, #00e5ff)',
            fontFamily: 'var(--font-mono, monospace)',
            fontSize: '0.75rem',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            gap: '0.4rem',
          }}
        >
          <span>⌕ Buscar</span>
          <kbd style={{ fontSize: '0.65rem', opacity: 0.75, padding: '1px 4px', border: '1px solid rgba(0,229,255,0.3)', borderRadius: '3px' }}>
            Ctrl+K
          </kbd>
        </button>

        <ThemeSelector />

        <button
          className="btn-expand"
          onClick={handleContractRing}
          disabled={isContracting}
          title="Contraer (archivar) el anillo exterior activo"
          style={{
            background: 'rgba(255, 62, 131, 0.1)',
            borderColor: 'var(--tertiary, #ff3e83)',
            color: 'var(--tertiary, #ff3e83)',
          }}
        >
          {isContracting ? '...' : 'CONTRACT'}
        </button>

        <button 
          className="btn-expand"
          onClick={() => {
            if ((window as any).openCommitDialog) {
              (window as any).openCommitDialog();
            }
          }}
        >
          EXPAND (Commit)
        </button>
      </div>
    </header>
  );
}

