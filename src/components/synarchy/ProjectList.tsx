import React, { useState, useEffect } from 'react';
import { ProjectCard } from './ProjectCard';
import { AddProjectDialog } from './AddProjectDialog';
import { getProjects, addProject, removeProject, rescanProject } from '../../lib/tauri/synarchy_api';
import type { ProjectEntry } from '../../types/synarchy';
import { isTauri } from '../../lib/tauri/isTauri';

const ProjectList: React.FC = () => {
  const [projects, setProjects] = useState<ProjectEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [scanningIds, setScanningIds] = useState<Set<string>>(new Set());
  const [scanningAll, setScanningAll] = useState(false);
  const [scanBanner, setScanBanner] = useState<string | null>(null);
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

  const isStatusMatch = (status: any, expected: string) => {
    if (typeof status === 'string') return status.toLowerCase() === expected.toLowerCase();
    if (status && typeof status === 'object') {
      if ('type' in status) return (status.type as string).toLowerCase() === expected.toLowerCase();
      if ('Error' in status) return expected.toLowerCase() === 'error';
    }
    return expected.toLowerCase() === 'active';
  };

  const filteredProjects = projects.filter(p => {
    const matchesSearch =
      p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      p.path.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesStatus = statusFilter === 'all' || isStatusMatch(p.status, statusFilter);
    return matchesSearch && matchesStatus;
  });

  const stats = {
    total: projects.length,
    active: projects.filter(p => isStatusMatch(p.status, 'active')).length,
    dormant: projects.filter(p => isStatusMatch(p.status, 'dormant')).length,
    totalMonads: projects.reduce((sum, p) => sum + p.monad_count, 0),
    totalRings: projects.reduce((sum, p) => sum + p.ring_count, 0),
  };

  const handleAddProject = async (path: string, name: string, scanMode: any) => {
    try {
      const newProject = await addProject(path, name, scanMode);
      setProjects(prev => {
        const remaining = prev.filter(p => p.id !== newProject.id);
        return [...remaining, newProject];
      });
      setShowAddDialog(false);
      setScanBanner(`✅ Proyecto "${newProject.name}" indexado (${newProject.monad_count} mónadas en ${newProject.ring_count} anillos).`);
    } catch (error) {
      console.error('Failed to add project:', error);
      throw error;
    }
  };

  const handleDeleteProject = async (id: string) => {
    if (!confirm('¿Quitar este proyecto del registro Synarchy? (No se borrará ningún archivo de tu disco)')) return;
    try {
      await removeProject(id);
      setProjects(prev => prev.filter(p => p.id !== id));
    } catch (error) {
      console.error('Failed to remove project:', error);
    }
  };

  const handleSelectProject = async (id: string) => {
    const project = projects.find(p => p.id === id);
    if (project && isTauri()) {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('open_project', { projectPath: project.path });
      } catch (err) {
        console.error('Failed to open project in backend:', err);
      }
    }
    window.location.href = `/?project=${encodeURIComponent(id)}`;
  };

  const handleRescan = async (id: string) => {
    setScanningIds(prev => new Set(prev).add(id));
    try {
      const updated = await rescanProject(id);
      setProjects(prev => prev.map(p => (p.id === id ? updated : p)));
      setScanBanner(
        `✅ "${updated.name}" re-escaneado: ${updated.monad_count} mónadas en ${updated.ring_count} anillos.`
      );
    } catch (error) {
      console.error('Failed to rescan:', error);
      setScanBanner(`⚠ Error al escanear proyecto: ${String(error)}`);
    } finally {
      setScanningIds(prev => {
        const next = new Set(prev);
        next.delete(id);
        return next;
      });
    }
  };

  const handleScanAll = async () => {
    if (scanningAll || projects.length === 0) return;
    setScanningAll(true);
    setScanBanner(`⏳ Escaneando ${projects.length} proyectos con el extractor AST políglota...`);
    let totalUpdatedMonads = 0;

    for (const proj of projects) {
      setScanningIds(prev => new Set(prev).add(proj.id));
      try {
        const updated = await rescanProject(proj.id);
        totalUpdatedMonads += updated.monad_count;
        setProjects(prev => prev.map(p => (p.id === proj.id ? updated : p)));
      } catch (err) {
        console.error(`Failed to rescan ${proj.name}:`, err);
      } finally {
        setScanningIds(prev => {
          const next = new Set(prev);
          next.delete(proj.id);
          return next;
        });
      }
    }

    setScanningAll(false);
    setScanBanner(
      `✨ Escaneo global completado: ${projects.length} proyectos sincronizados (${totalUpdatedMonads} mónadas totales).`
    );
  };

  if (loading) {
    return (
      <div className="explorer-loading">
        <div className="spinner"></div>
        <span style={{ color: 'var(--text-dim)', fontSize: '0.85rem' }}>
          Cargando registro Synarchy...
        </span>
      </div>
    );
  }

  return (
    <main className="explorer">
      <header className="synarchy-topbar">
        <div className="synarchy-title-group">
          <a href="/" className="back-link">
            ← Volver al Mandala
          </a>
          <div>
            <h1>SYNARCHY • ECOSISTEMA MULTI-PROYECTO</h1>
            <p className="synarchy-subtitle">
               Federación de repositorios indexados en el Grafo Tripartito SO(2) • 100% No-Destructivo
            </p>
          </div>
        </div>

        <div className="synarchy-top-actions">
          <button
            className="btn-scan-all"
            onClick={handleScanAll}
            disabled={scanningAll || projects.length === 0}
          >
            {scanningAll ? '⏳ Escaneando todos...' : '⚡ Escanear Todos (Scan All)'}
          </button>
          <button className="btn-add" onClick={() => setShowAddDialog(true)}>
            <span>+</span> Añadir Proyecto
          </button>
        </div>
      </header>

      {scanBanner && (
        <div className="synarchy-banner" onClick={() => setScanBanner(null)}>
          <span>{scanBanner}</span>
          <button className="banner-close">×</button>
        </div>
      )}

      {filteredProjects.length === 0 ? (
        <div className="empty-state">
          <div className="empty-icon">🌀</div>
          <h2>No se encontraron proyectos</h2>
          <p style={{ color: 'var(--text-dim)', fontSize: '0.85rem', marginTop: '8px' }}>
            Añade un directorio local o repositorio Git para indexar sus mónadas y anillos.
          </p>
        </div>
      ) : (
        <div className="project-scroll-container">
          <div className="project-grid">
            {filteredProjects.map(project => (
              <div key={project.id} className="project-wrapper">
                <ProjectCard
                  project={project}
                  scanning={scanningIds.has(project.id)}
                  onSelect={handleSelectProject}
                  onRescan={handleRescan}
                  onDelete={handleDeleteProject}
                />
              </div>
            ))}
          </div>
        </div>
      )}

      <div className="add-project-fixed">
        <div className="stats-group">
          <div className="stat-item">
            <span className="stat-num">{stats.total}</span>
            <span className="stat-lbl">Proyectos</span>
          </div>
          <div className="stat-item">
            <span className="stat-num active">{stats.active}</span>
            <span className="stat-lbl">Activos</span>
          </div>
          <div className="stat-item">
            <span className="stat-num">{stats.totalRings}</span>
            <span className="stat-lbl">Anillos</span>
          </div>
          <div className="stat-item">
            <span className="stat-num">{stats.totalMonads}</span>
            <span className="stat-lbl">Mónadas</span>
          </div>
        </div>

        <input
          type="text"
          className="search-input"
          placeholder="Buscar proyecto o ruta..."
          value={searchQuery}
          onChange={e => setSearchQuery(e.target.value)}
        />

        <div className="filter-buttons">
          <button
            className={`filter-btn ${statusFilter === 'all' ? 'active' : ''}`}
            onClick={() => setStatusFilter('all')}
          >
            Todos
          </button>
          <button
            className={`filter-btn ${statusFilter === 'Active' ? 'active' : ''}`}
            onClick={() => setStatusFilter('Active')}
          >
            Activos
          </button>
          <button
            className={`filter-btn ${statusFilter === 'Dormant' ? 'active' : ''}`}
            onClick={() => setStatusFilter('Dormant')}
          >
            Latentes
          </button>
        </div>

        <button className="btn-add" onClick={() => setShowAddDialog(true)}>
          <span>+</span> Añadir Proyecto
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