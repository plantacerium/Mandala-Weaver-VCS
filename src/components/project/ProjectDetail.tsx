import React, { useEffect, useState } from 'react';
import MandalaCanvas from '../mandala/MandalaCanvas';
import { getProjectDetail, rescanProject } from '../../lib/tauri/synarchy_api';
import type { ProjectEntry } from '../../types/synarchy';

function getProjectIdFromUrl(): string | null {
  if (typeof window === 'undefined') return null;
  const params = new URLSearchParams(window.location.search);
  return params.get('id');
}

const ProjectDetail: React.FC = () => {
  const [project, setProject] = useState<ProjectEntry | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const id = getProjectIdFromUrl();
    if (!id) {
      setLoading(false);
      return;
    }
    getProjectDetail(id)
      .then(setProject)
      .catch(() => setProject(null))
      .finally(() => setLoading(false));
  }, []);

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

  return (
    <div className="project-detail" id="project-detail">
      <div className="detail-header">
        <a href="/explorer" className="back-link">← Back to Explorer</a>
        <h1 id="project-name">{project.name}</h1>
        <button onClick={handleRescan} style={{
          marginLeft: 'auto',
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
            <span className="value" style={{ fontSize: '1rem' }}>{project.status}</span>
            <span className="label">Status</span>
          </div>
          <div className="stat-card full-width">
            <span className="value" style={{ fontSize: '0.85rem', wordBreak: 'break-all' }} id="project-path">{project.path}</span>
            <span className="label">Path</span>
          </div>
        </div>

        <div className="detail-canvas">
          <MandalaCanvas />
        </div>
      </div>
    </div>
  );
};

export default ProjectDetail;
