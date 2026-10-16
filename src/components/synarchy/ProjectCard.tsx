import React from 'react';
import type { ProjectEntry } from '../../types/synarchy';

interface ProjectCardProps {
  project: ProjectEntry;
  scanning?: boolean;
  onSelect: (id: string) => void;
  onRescan: (id: string) => void;
  onDelete: (id: string) => void;
}

export const ProjectCard: React.FC<ProjectCardProps> = ({
  project,
  scanning = false,
  onSelect,
  onRescan,
  onDelete,
}) => {
  const getStatusText = (status: ProjectEntry['status']) => {
    if (scanning) return 'Scanning...';
    if (typeof status === 'string') return status;
    if (status && typeof status === 'object') {
      if ('Error' in status) return `Error: ${(status as any).Error}`;
      if ('message' in status) return `Error: ${(status as any).message}`;
      if ('type' in status) return (status as any).type;
    }
    return 'Active';
  };

  const getStatusClass = (status: ProjectEntry['status']) => {
    if (scanning) return 'Scanning';
    const text = getStatusText(status).toLowerCase();
    if (text.includes('error')) return 'Error';
    if (text.includes('dormant')) return 'Dormant';
    return 'Active';
  };

  const formatDate = (timestamp: number) => {
    if (!timestamp || timestamp === 0) return '—';
    const now = Date.now() / 1000;
    const diff = Math.max(0, now - timestamp);
    if (diff < 60) return 'Just now';
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`;
    return new Date(timestamp * 1000).toLocaleDateString();
  };

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'Git':
        return '🐙';
      case 'Weaver':
        return '🌀';
      case 'Remote':
        return '🌐';
      default:
        return '📁';
    }
  };

  const ringCount = Math.min(Math.max(project.ring_count, 1), 5);

  return (
    <div className="project-card" onClick={() => onSelect(project.id)}>
      <div className="card-header">
        <div className="project-icon">
          <svg viewBox="0 0 80 80" className="mini-mandala">
            <circle cx="40" cy="40" r="5" fill="#e85d04" />

            {Array.from({ length: ringCount }).map((_, ringIdx) => (
              <circle
                key={ringIdx}
                cx="40"
                cy="40"
                r={(ringIdx + 1) * 6 + 6}
                fill="none"
                stroke={
                  ringIdx === ringCount - 1
                    ? 'rgba(0, 229, 255, 0.5)'
                    : 'rgba(255,255,255,0.14)'
                }
                strokeWidth={ringIdx === ringCount - 1 ? 1.5 : 1}
              />
            ))}

            {project.monad_count > 0 && (
              <g>
                {Array.from({
                  length: Math.min(project.monad_count, ringCount * 5),
                }).map((_, i) => {
                  const totalDots = Math.min(project.monad_count, ringCount * 5);
                  const angle = (i / totalDots) * 360;
                  const rad = (angle * Math.PI) / 180;
                  const radius = 10 + ((i % ringCount) + 1) * 5.5;
                  return (
                    <circle
                      key={i}
                      cx={40 + Math.cos(rad) * radius}
                      cy={40 + Math.sin(rad) * radius}
                      r="2.2"
                      fill={`hsl(${(i * 45) % 360}, 75%, 60%)`}
                      opacity="0.9"
                    />
                  );
                })}
              </g>
            )}
          </svg>
        </div>

        <div className="project-info">
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', gap: '8px' }}>
            <h3 title={project.name}>{project.name}</h3>
            <span className="project-type-badge">
              {getTypeIcon(project.project_type)} {project.project_type}
            </span>
          </div>
          <span className="project-path" title={project.path}>
            {project.path}
          </span>
        </div>
      </div>

      <div className="card-stats">
        <div className="stat">
          <span className="stat-value">{project.ring_count}</span>
          <span className="stat-label">Rings</span>
        </div>
        <div className="stat">
          <span className="stat-value">{project.monad_count}</span>
          <span className="stat-label">Monads</span>
        </div>
        <div className="stat">
          <span className="stat-value">{formatDate(project.last_scanned)}</span>
          <span className="stat-label">Last Scan</span>
        </div>
      </div>

      <div className="card-actions-row" onClick={e => e.stopPropagation()}>
        <button
          className="card-action-btn primary"
          onClick={() => onSelect(project.id)}
          title="Abrir este proyecto en el lienzo radial del Mandala"
        >
          🧭 Abrir Lienzo
        </button>
        <button
          className="card-action-btn"
          onClick={() => onRescan(project.id)}
          disabled={scanning}
          title="Re-escanear AST políglota y actualizar el grafo tripartito"
        >
          {scanning ? '⏳ Escaneando...' : '↻ Scan AST'}
        </button>
      </div>

      <div className="card-status">
        <span className={`status-badge ${getStatusClass(project.status)}`}>
          • {getStatusText(project.status)}
        </span>

        <button
          className="btn-delete"
          title="Quitar proyecto del registro Synarchy (no borra archivos en disco)"
          onClick={e => {
            e.stopPropagation();
            onDelete(project.id);
          }}
        >
          ×
        </button>
      </div>
    </div>
  );
};

export default ProjectCard;