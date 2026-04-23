import { invoke } from '@tauri-apps/api/core';
import type { ProjectEntry } from '../../types/synarchy';

function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI__' in window;
}

export async function getProjects(): Promise<ProjectEntry[]> {
  if (!isTauri()) {
    return getMockProjects();
  }
  
  try {
    return await invoke<ProjectEntry[]>('get_projects');
  } catch (error) {
    console.warn('Failed to get projects:', error);
    return getMockProjects();
  }
}

export async function addProject(path: string, name: string): Promise<ProjectEntry> {
  if (!isTauri()) {
    return {
      id: Math.random().toString(36).slice(2, 10),
      name: name || path.split(/[/\\]/).pop() || 'New Project',
      path,
      project_type: 'Local',
      last_scanned: Math.floor(Date.now() / 1000),
      ring_count: 3,
      monad_count: Math.floor(Math.random() * 50) + 10,
      status: 'Active',
    };
  }
  
  return await invoke<ProjectEntry>('add_project', { path, name });
}

export async function removeProject(id: string): Promise<void> {
  if (!isTauri()) {
    return;
  }
  
  await invoke('remove_project', { id });
}

export async function getProjectDetail(id: string): Promise<ProjectEntry> {
  if (!isTauri()) {
    return {
      id,
      name: 'Mock Project',
      path: '/mock/path',
      project_type: 'Local',
      last_scanned: Math.floor(Date.now() / 1000),
      ring_count: 3,
      monad_count: 25,
      status: 'Active',
    };
  }
  
  return await invoke<ProjectEntry>('get_project_detail', { id });
}

export async function rescanProject(id: string): Promise<ProjectEntry> {
  if (!isTauri()) {
    return {
      id,
      name: 'Mock Project',
      path: '/mock/path',
      project_type: 'Local',
      last_scanned: Math.floor(Date.now() / 1000),
      ring_count: 4,
      monad_count: 30,
      status: 'Active',
    };
  }
  
  return await invoke<ProjectEntry>('rescan_project', { id });
}

export async function getProjectMandala(id: string): Promise<any> {
  if (!isTauri()) {
    return {
      bindu_name: 'Mock Project',
      constellations: [],
      edges: [],
    };
  }
  
  return await invoke<any>('get_project_mandala', { id });
}

function getMockProjects(): ProjectEntry[] {
  return [
    { id: 'mandala-vcs', name: 'Mandala Weaver', path: 'I:\\GITHUB\\Mandala-Weaver-VCS', project_type: 'Weaver', last_scanned: Math.floor(Date.now() / 1000) - 3600, ring_count: 7, monad_count: 156, status: 'Active' },
    { id: 'rust-core', name: 'Rust Core', path: 'I:\\Projects\\rust-core', project_type: 'Local', last_scanned: Math.floor(Date.now() / 1000) - 7200, ring_count: 12, monad_count: 482, status: 'Active' },
    { id: 'astro-blog', name: 'Astro Blog', path: 'I:\\Projects\\astro-blog', project_type: 'Local', last_scanned: Math.floor(Date.now() / 1000) - 86400, ring_count: 4, monad_count: 38, status: 'Active' },
    { id: 'react-ui', name: 'React UI', path: 'I:\\Projects\\react-ui-lib', project_type: 'Local', last_scanned: Math.floor(Date.now() / 1000) - 172800, ring_count: 6, monad_count: 89, status: 'Dormant' },
    { id: 'data-pipe', name: 'Data Pipeline', path: 'I:\\Projects\\data-pipeline', project_type: 'Git', last_scanned: Math.floor(Date.now() / 1000) - 259200, ring_count: 9, monad_count: 234, status: 'Active' },
    { id: 'ml-utils', name: 'ML Utils', path: 'I:\\Projects\\ml-utils', project_type: 'Local', last_scanned: Math.floor(Date.now() / 1000) - 345600, ring_count: 5, monad_count: 67, status: 'Active' },
    { id: 'api-gw', name: 'API Gateway', path: 'I:\\Projects\\api-gateway', project_type: 'Local', last_scanned: Math.floor(Date.now() / 1000) - 432000, ring_count: 8, monad_count: 145, status: 'Scanning' },
    { id: 'db-sync', name: 'DB Sync', path: 'I:\\Projects\\db-sync', project_type: 'Local', last_scanned: Math.floor(Date.now() / 1000) - 600000, ring_count: 4, monad_count: 52, status: 'Active' },
    { id: 'auth-svc', name: 'Auth Service', path: 'I:\\Projects\\auth-svc', project_type: 'Local', last_scanned: Math.floor(Date.now() / 1000) - 700000, ring_count: 6, monad_count: 78, status: 'Active' },
    { id: 'queue-mgr', name: 'Queue Manager', path: 'I:\\Projects\\queue-mgr', project_type: 'Git', last_scanned: Math.floor(Date.now() / 1000) - 800000, ring_count: 5, monad_count: 61, status: 'Active' },
    { id: 'cache-srv', name: 'Cache Server', path: 'I:\\Projects\\cache-srv', project_type: 'Local', last_scanned: Math.floor(Date.now() / 1000) - 900000, ring_count: 3, monad_count: 29, status: 'Dormant' },
    { id: 'log-agg', name: 'Log Aggregator', path: 'I:\\Projects\\log-agg', project_type: 'Local', last_scanned: Math.floor(Date.now() / 1000) - 1000000, ring_count: 7, monad_count: 93, status: 'Active' },
    { id: 'metrics', name: 'Metrics', path: 'I:\\Projects\\metrics', project_type: 'Git', last_scanned: Math.floor(Date.now() / 1000) - 1100000, ring_count: 4, monad_count: 45, status: 'Active' },
    { id: 'stream-proc', name: 'Stream Proc', path: 'I:\\Projects\\stream-proc', project_type: 'Local', last_scanned: Math.floor(Date.now() / 1000) - 1200000, ring_count: 8, monad_count: 112, status: 'Active' },
    { id: 'config-mgr', name: 'Config Mgr', path: 'I:\\Projects\\config-mgr', project_type: 'Local', last_scanned: Math.floor(Date.now() / 1000) - 1300000, ring_count: 3, monad_count: 21, status: 'Dormant' },
    { id: 'notify-svc', name: 'Notification', path: 'I:\\Projects\\notify-svc', project_type: 'Local', last_scanned: Math.floor(Date.now() / 1000) - 1400000, ring_count: 5, monad_count: 56, status: 'Active' },
    { id: 'sched-svc', name: 'Scheduler', path: 'I:\\Projects\\sched-svc', project_type: 'Git', last_scanned: Math.floor(Date.now() / 1000) - 1500000, ring_count: 6, monad_count: 73, status: 'Active' },
    { id: 'ws-server', name: 'WebSocket', path: 'I:\\Projects\\ws-server', project_type: 'Local', last_scanned: Math.floor(Date.now() / 1000) - 1600000, ring_count: 4, monad_count: 38, status: 'Active' },
    { id: 'grpc-proto', name: 'gRPC Proto', path: 'I:\\Projects\\grpc-proto', project_type: 'Git', last_scanned: Math.floor(Date.now() / 1000) - 1700000, ring_count: 2, monad_count: 15, status: 'Dormant' },
    { id: 'blob-store', name: 'Blob Storage', path: 'I:\\Projects\\blob-store', project_type: 'Local', last_scanned: Math.floor(Date.now() / 1000) - 1800000, ring_count: 5, monad_count: 48, status: 'Active' },
    { id: 'search-eng', name: 'Search Eng', path: 'I:\\Projects\\search-eng', project_type: 'Weaver', last_scanned: Math.floor(Date.now() / 1000) - 1900000, ring_count: 9, monad_count: 167, status: 'Active' },
    { id: 'cdn-edge', name: 'CDN Edge', path: 'I:\\Projects\\cdn-edge', project_type: 'Git', last_scanned: Math.floor(Date.now() / 1000) - 2000000, ring_count: 4, monad_count: 34, status: 'Active' },
    { id: 'email-svc', name: 'Email Svc', path: 'I:\\Projects\\email-svc', project_type: 'Local', last_scanned: Math.floor(Date.now() / 1000) - 2100000, ring_count: 5, monad_count: 42, status: 'Dormant' },
    { id: 'payment-gw', name: 'Payment GW', path: 'I:\\Projects\\payment-gw', project_type: 'Local', last_scanned: Math.floor(Date.now() / 1000) - 2200000, ring_count: 7, monad_count: 89, status: 'Active' },
    { id: 'analytics', name: 'Analytics', path: 'I:\\Projects\\analytics', project_type: 'Weaver', last_scanned: Math.floor(Date.now() / 1000) - 2300000, ring_count: 6, monad_count: 71, status: 'Active' },
    { id: 'backup-svc', name: 'Backup Svc', path: 'I:\\Projects\\backup-svc', project_type: 'Local', last_scanned: Math.floor(Date.now() / 1000) - 2400000, ring_count: 3, monad_count: 28, status: 'Dormant' },
    { id: 'monitoring', name: 'Monitoring', path: 'I:\\Projects\\monitoring', project_type: 'Local', last_scanned: Math.floor(Date.now() / 1000) - 2500000, ring_count: 5, monad_count: 53, status: 'Active' },
  ];
}