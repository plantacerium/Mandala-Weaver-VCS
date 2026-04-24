import React from 'react';
import type { ProjectEntry } from '../../types/synarchy';

interface ProjectCardProps {
  project: ProjectEntry;
  onSelect: (id: string) => void;
  onDelete: (id: string) => void;
}

export const ProjectCard: React.FC<ProjectCardProps> = ({
  project,
  onSelect,
  onDelete
}) => {
  const getStatusClass = (status: ProjectEntry['status']) => {
    if (typeof status === 'string') return status;
    return 'Error';
  };

  const formatDate = (timestamp: number) => {
    if (timestamp === 0) return '—';
    const now = Date.now() / 1000;
    const diff = now - timestamp;
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`;
    return new Date(timestamp * 1000).toLocaleDateString();
  };

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'Git': return '🐙';
      case 'Weaver': return '🌀';
      case 'Remote': return '🌐';
      default: return '📁';
    }
  };

  const ringCount = Math.min(project.ring_count, 5);

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
                r={(ringIdx + 1) * 12 + 8}
                fill="none"
                stroke={ringIdx === ringCount - 1 ? 'rgba(0, 229, 255, 0.4)' : 'rgba(255,255,255,0.12)'}
                strokeWidth={ringIdx === ringCount - 1 ? 1.5 : 1}
              />
            ))}

            {project.monad_count > 0 && (
              <g>
                {Array.from({ length: Math.min(project.monad_count, ringCount * 4) }).map((_, i) => {
                  const angle = (i / Math.min(project.monad_count, ringCount * 4)) * 360;
                  const rad = (angle * Math.PI) / 180;
                  const radius = 16 + ((i % ringCount) + 1) * 10;
                  return (
                    <circle
                      key={i}
                      cx={40 + Math.cos(rad) * radius}
                      cy={40 + Math.sin(rad) * radius}
                      r="2.5"
                      fill={`hsl(${(i * 45) % 360}, 70%, 55%)`}
                      opacity="0.85"
                    />
                  );
                })}
              </g>
            )}
          </svg>
        </div>

        <div className="project-info">
          <h3>{project.name}</h3>
          <span className="project-path">{project.path}</span>
          <span className="project-type-badge">
            {getTypeIcon(project.project_type)} {project.project_type}
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

      <div className="card-status">
        <span className={`status-badge ${getStatusClass(project.status)}`}>
          {typeof project.status === 'string' ? project.status : 'Error'}
        </span>

        <button
          className="btn-delete"
          onClick={(e) => {
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