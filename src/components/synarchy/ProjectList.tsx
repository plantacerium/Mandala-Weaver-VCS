import React, { useState, useEffect } from 'react';
import { ProjectCard } from './ProjectCard';
import { AddProjectDialog } from './AddProjectDialog';
import { getProjects, addProject, removeProject, rescanProject } from '../../lib/tauri/synarchy_api';
import type { ProjectEntry } from '../../types/synarchy';

const ProjectList: React.FC = () => {
  const [projects, setProjects] = useState<ProjectEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState<string>('all');

  useEffect(() => {
    loadProjects();
  }, []);

  const loadProjects = async () => {
    setLoading(true);
    try {
      const data = await getProjects();
      setProjects(data);
    } catch (error) {
      console.error('Failed to load projects:', error);
    } finally {
      setLoading(false);
    }
  };

  const filteredProjects = projects.filter(p => {
    const matchesSearch = p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      p.path.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesStatus = statusFilter === 'all' ||
      (typeof p.status === 'string' && p.status === statusFilter);
    return matchesSearch && matchesStatus;
  });

  const stats = {
    total: projects.length,
    active: projects.filter(p => p.status === 'Active').length,
    dormant: projects.filter(p => p.status === 'Dormant').length,
    totalMonads: projects.reduce((sum, p) => sum + p.monad_count, 0),
  };

  const handleAddProject = async (path: string, name: string) => {
    try {
      const newProject = await addProject(path, name);
      setProjects(prev => [...prev, newProject]);
      setShowAddDialog(false);
    } catch (error) {
      console.error('Failed to add project:', error);
    }
  };

  const handleDeleteProject = async (id: string) => {
    if (!confirm('Remove this project?')) return;
    try {
      await removeProject(id);
      setProjects(prev => prev.filter(p => p.id !== id));
    } catch (error) {
      console.error('Failed to remove project:', error);
    }
  };

  const handleSelectProject = (id: string) => {
    window.location.href = `/project/${id}`;
  };

  const handleRescan = async (id: string) => {
    try {
      const updated = await rescanProject(id);
      setProjects(prev => prev.map(p => p.id === id ? updated : p));
    } catch (error) {
      console.error('Failed to rescan:', error);
    }
  };

  if (loading) {
    return (
      <div className="explorer-loading">
        <div className="spinner"></div>
      </div>
    );
  }

  return (
    <main className="explorer">
      <a href="/" className="back-link">← Back to Mandala</a>

      {filteredProjects.length === 0 ? (
        <div className="empty-state">
          <div className="empty-icon">🌀</div>
          <h2>No Projects Found</h2>
        </div>
      ) : (
        <div className="project-scroll-container">
          <div className="project-grid">
            {filteredProjects.map(project => (
              <div key={project.id} className="project-wrapper">
                <ProjectCard
                  project={project}
                  onSelect={handleSelectProject}
                  onDelete={handleDeleteProject}
                />
                <button
                  className="btn-rescan"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleRescan(project.id);
                  }}
                >
                  ↻
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="add-project-fixed">
        <div className="stats-group">
          <div className="stat-item">
            <span className="stat-num">{stats.total}</span>
            <span className="stat-lbl">Total</span>
          </div>
          <div className="stat-item">
            <span className="stat-num active">{stats.active}</span>
            <span className="stat-lbl">Active</span>
          </div>
          <div className="stat-item">
            <span className="stat-num dormant">{stats.dormant}</span>
            <span className="stat-lbl">Dormant</span>
          </div>
          <div className="stat-item">
            <span className="stat-num">{stats.totalMonads}</span>
            <span className="stat-lbl">Monads</span>
          </div>
        </div>

        <input
          type="text"
          className="search-input"
          placeholder="Search projects..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />

        <div className="filter-buttons">
          <button
            className={`filter-btn ${statusFilter === 'all' ? 'active' : ''}`}
            onClick={() => setStatusFilter('all')}
          >
            All
          </button>
          <button
            className={`filter-btn ${statusFilter === 'Active' ? 'active' : ''}`}
            onClick={() => setStatusFilter('Active')}
          >
            Active
          </button>
          <button
            className={`filter-btn ${statusFilter === 'Dormant' ? 'active' : ''}`}
            onClick={() => setStatusFilter('Dormant')}
          >
            Dormant
          </button>
        </div>

        <button className="btn-add" onClick={() => setShowAddDialog(true)}>
          <span>+</span> Add Project
        </button>
      </div>

      {showAddDialog && (
        <AddProjectDialog
          onAdd={handleAddProject}
          onClose={() => setShowAddDialog(false)}
        />
      )}
    </main>
  );
};

export default ProjectList;