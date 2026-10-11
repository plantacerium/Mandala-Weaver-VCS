import React, { useEffect, useState } from 'react';
import { useWorkspaceStore } from '../../lib/state/workspaceStore';
import ThemeSelector from './ThemeSelector';

export default function WorkspaceHeader() {
  const { mandalaState } = useWorkspaceStore();
  const [activePath, setActivePath] = useState<string>('~/mandala-weaver-vcs');

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
      <div className="actions">
        <ThemeSelector />
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
