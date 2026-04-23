export interface ProjectEntry {
  id: string;
  name: string;
  path: string;
  project_type: 'Local' | 'Remote' | 'Git' | 'Weaver';
  last_scanned: number;
  ring_count: number;
  monad_count: number;
  status: ProjectStatus;
}

export type ProjectStatus =
  | 'Active'
  | 'Dormant'
  | 'Scanning'
  | { Error: string };

export interface ProjectRegistry {
  projects: ProjectEntry[];
}

export interface ProjectChangeEvent {
  project_id: string;
  project_name: string;
  change_type: 'New' | 'Modified' | 'Removed' | 'Rescan';
  monads_added: number;
  rings_changed: number[];
}